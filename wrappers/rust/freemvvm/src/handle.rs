use crate::error::UiError;
use crate::native_api::SharedApi;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

pub(crate) struct PendingToken {
    pub(crate) token: u64,
    pub(crate) runtime_token: u64,
    pub(crate) api: SharedApi,
    pub(crate) anchor: Option<Arc<RuntimeAnchor>>,
}

struct RegistrySlot {
    generation: u64,
    state: SlotState,
    pending: Option<PendingToken>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SlotState {
    Free,
    Reserved,
    Pending,
    InFlight,
}

#[derive(Default)]
struct CleanupRegistry {
    slots: Vec<RegistrySlot>,
}

impl CleanupRegistry {
    fn reserve(&mut self) -> Result<TicketReservation, ()> {
        if let Some((index, slot)) = self
            .slots
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.state == SlotState::Free)
        {
            slot.generation = slot.generation.checked_add(1).ok_or(())?;
            slot.state = SlotState::Reserved;
            return Ok(TicketReservation::new(index, slot.generation));
        }

        self.slots.try_reserve(1).map_err(|_| ())?;
        let index = self.slots.len();
        self.slots.push(RegistrySlot {
            generation: 1,
            state: SlotState::Reserved,
            pending: None,
        });
        Ok(TicketReservation::new(index, 1))
    }
}

fn cleanup_registry() -> &'static Mutex<CleanupRegistry> {
    static REGISTRY: OnceLock<Mutex<CleanupRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(CleanupRegistry::default()))
}

fn lock_registry() -> MutexGuard<'static, CleanupRegistry> {
    cleanup_registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) struct TicketReservation {
    index: usize,
    generation: u64,
    active: bool,
}

impl TicketReservation {
    fn new(index: usize, generation: u64) -> Self {
        Self {
            index,
            generation,
            active: true,
        }
    }

    pub(crate) fn reserve() -> Result<Self, ()> {
        lock_registry().reserve()
    }

    pub(crate) fn release(mut self) {
        release_slot(self.index, self.generation);
        self.active = false;
    }

    pub(crate) fn defer(mut self, pending: PendingToken) {
        let mut registry = lock_registry();
        if let Some(slot) = registry.slots.get_mut(self.index) {
            if slot.generation == self.generation && slot.state == SlotState::Reserved {
                slot.pending = Some(pending);
                slot.state = SlotState::Pending;
                self.active = false;
                return;
            }
        }

        // This is an invariant failure. Leak the still-owned token instead of losing it.
        std::mem::forget(pending);
        self.active = false;
    }
}

impl Drop for TicketReservation {
    fn drop(&mut self) {
        if self.active {
            release_slot(self.index, self.generation);
        }
    }
}

fn release_slot(index: usize, generation: u64) {
    let mut registry = lock_registry();
    if let Some(slot) = registry.slots.get_mut(index) {
        if slot.generation == generation && slot.state == SlotState::Reserved {
            slot.state = SlotState::Free;
        }
    }
}

/// A typed owned C ABI token. UI-affine owners are deliberately neither `Send` nor `Sync`.
///
/// ```compile_fail
/// use freemvvm::{Borrowed, Owned};
/// fn escape<'a>(owner: &'a Owned<u8>) -> Borrowed<'static, u8> {
///     owner.borrow().unwrap()
/// }
/// ```
pub struct Owned<T> {
    token: Option<u64>,
    api: SharedApi,
    ticket: Option<TicketReservation>,
    runtime_token: u64,
    anchor: Option<Arc<RuntimeAnchor>>,
    _marker: PhantomData<(T, Rc<()>)>,
}

impl<T> Owned<T> {
    pub(crate) fn from_token(
        token: u64,
        api: SharedApi,
        ticket: TicketReservation,
        runtime_token: u64,
        anchor: Option<Arc<RuntimeAnchor>>,
    ) -> Self {
        Self {
            token: Some(token),
            api,
            ticket: Some(ticket),
            runtime_token,
            anchor,
            _marker: PhantomData,
        }
    }

    pub fn borrow(&self) -> Result<Borrowed<'_, T>, UiError> {
        let token = self
            .token
            .ok_or_else(|| UiError::from_status(crate::native_api::CLOSED))?;
        Ok(Borrowed {
            token,
            _owner: PhantomData,
            _not_send: PhantomData,
        })
    }

    pub fn try_clone(&self) -> Result<Self, UiError> {
        let token = self
            .token
            .ok_or_else(|| UiError::from_status(crate::native_api::CLOSED))?;
        let ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(crate::native_api::OUT_OF_MEMORY))?;
        let (status, new_token) = self.api.handle_retain(token);
        if status != crate::native_api::OK {
            if new_token != 0 {
                ticket.defer(PendingToken {
                    token: new_token,
                    runtime_token: self.runtime_token,
                    api: Arc::clone(&self.api),
                    anchor: self.anchor.clone(),
                });
            }
            return Err(UiError::from_status(status));
        }
        if new_token == 0 {
            return Err(UiError::from_status(crate::native_api::INTERNAL_ERROR));
        }

        let runtime_token = if self.anchor.is_some() {
            self.runtime_token
        } else {
            new_token
        };
        Ok(Self::from_token(
            new_token,
            Arc::clone(&self.api),
            ticket,
            runtime_token,
            self.anchor.clone(),
        ))
    }

    pub fn close(&mut self) -> Result<(), UiError> {
        let Some(token) = self.token else {
            return Ok(());
        };
        let status = self.api.handle_release(token);
        if status != crate::native_api::OK {
            return Err(UiError::from_status(status));
        }

        self.token = None;
        if let Some(ticket) = self.ticket.take() {
            ticket.release();
        }
        self.anchor.take();
        Ok(())
    }

    pub(crate) fn api(&self) -> &SharedApi {
        &self.api
    }
}

impl<T> Drop for Owned<T> {
    fn drop(&mut self) {
        let Some(token) = self.token.take() else {
            return;
        };
        let Some(ticket) = self.ticket.take() else {
            // Missing a ticket is an invariant failure; do not consume the native token.
            self.token = Some(token);
            return;
        };

        if self.api.handle_release(token) == crate::native_api::OK {
            ticket.release();
            self.anchor.take();
            return;
        }

        ticket.defer(PendingToken {
            token,
            runtime_token: self.runtime_token,
            api: Arc::clone(&self.api),
            anchor: self.anchor.take(),
        });
    }
}

/// A non-owning token whose lifetime is tied to its owner.
pub struct Borrowed<'a, T> {
    token: u64,
    _owner: PhantomData<&'a Owned<T>>,
    _not_send: PhantomData<Rc<()>>,
}

impl<T> Borrowed<'_, T> {
    pub(crate) fn token(&self) -> u64 {
        self.token
    }
}

pub(crate) struct RuntimeAnchor {
    token: u64,
    api: SharedApi,
    ticket: Option<TicketReservation>,
    active: bool,
}

impl RuntimeAnchor {
    pub(crate) fn try_create(api: SharedApi, borrowed_runtime: u64) -> Result<Arc<Self>, UiError> {
        let ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(crate::native_api::OUT_OF_MEMORY))?;
        let mut anchor = Arc::new(Self {
            token: 0,
            api: Arc::clone(&api),
            ticket: Some(ticket),
            active: false,
        });
        let (status, token) = api.handle_retain(borrowed_runtime);
        if status != crate::native_api::OK || token == 0 {
            if token != 0 {
                let inner = Arc::get_mut(&mut anchor).expect("new anchor is uniquely owned");
                inner.token = token;
                inner.active = true;
            }
            return Err(UiError::from_status(if status == crate::native_api::OK {
                crate::native_api::INTERNAL_ERROR
            } else {
                status
            }));
        }

        let inner = Arc::get_mut(&mut anchor).expect("new anchor is uniquely owned");
        inner.token = token;
        inner.active = true;
        Ok(anchor)
    }

    pub(crate) fn token(&self) -> u64 {
        self.token
    }
}

impl Drop for RuntimeAnchor {
    fn drop(&mut self) {
        if !self.active {
            self.ticket.take();
            return;
        }
        let Some(ticket) = self.ticket.take() else {
            return;
        };
        if self.api.handle_release(self.token) == crate::native_api::OK {
            ticket.release();
            return;
        }

        ticket.defer(PendingToken {
            token: self.token,
            runtime_token: self.token,
            api: Arc::clone(&self.api),
            anchor: None,
        });
    }
}

pub(crate) struct AnyThreadOwned {
    token: Option<u64>,
    api: SharedApi,
    ticket: Option<TicketReservation>,
    anchor: Option<Arc<RuntimeAnchor>>,
}

impl AnyThreadOwned {
    pub(crate) fn from_token(
        token: u64,
        api: SharedApi,
        ticket: TicketReservation,
        anchor: Arc<RuntimeAnchor>,
    ) -> Self {
        Self {
            token: Some(token),
            api,
            ticket: Some(ticket),
            anchor: Some(anchor),
        }
    }

    pub(crate) fn token(&self) -> Result<u64, UiError> {
        self.token
            .ok_or_else(|| UiError::from_status(crate::native_api::CLOSED))
    }

    pub(crate) fn api(&self) -> &SharedApi {
        &self.api
    }

    pub(crate) fn anchor(&self) -> Option<Arc<RuntimeAnchor>> {
        self.anchor.clone()
    }

    pub(crate) fn try_close(&mut self) -> Result<(), UiError> {
        let Some(token) = self.token else {
            return Ok(());
        };
        let status = self.api.handle_release(token);
        if status != crate::native_api::OK {
            return Err(UiError::from_status(status));
        }
        self.token = None;
        if let Some(ticket) = self.ticket.take() {
            ticket.release();
        }
        self.anchor.take();
        Ok(())
    }
}

impl Drop for AnyThreadOwned {
    fn drop(&mut self) {
        let Some(token) = self.token.take() else {
            return;
        };
        let Some(ticket) = self.ticket.take() else {
            self.token = Some(token);
            return;
        };
        if self.api.handle_release(token) == crate::native_api::OK {
            ticket.release();
            self.anchor.take();
            return;
        }

        let anchor = self.anchor.take();
        let runtime_token = anchor.as_ref().map_or(0, |value| value.token());
        ticket.defer(PendingToken {
            token,
            runtime_token,
            api: Arc::clone(&self.api),
            anchor,
        });
    }
}

pub(crate) fn retry_pending_cleanup() -> usize {
    let slot_count = lock_registry().slots.len();
    let mut released = 0;
    for index in 0..slot_count {
        let pending = {
            let mut registry = lock_registry();
            let Some(slot) = registry.slots.get_mut(index) else {
                continue;
            };
            if slot.state != SlotState::Pending {
                continue;
            }
            let Some(pending) = slot.pending.take() else {
                slot.state = SlotState::Free;
                continue;
            };
            slot.state = SlotState::InFlight;
            (slot.generation, pending)
        };

        let (generation, mut pending) = pending;
        let on_ui_thread = pending.runtime_token == 0
            || (pending.api.runtime_is_ui_thread(pending.runtime_token)
                == (crate::native_api::OK, 1));
        let release_status = if on_ui_thread {
            pending.api.handle_release(pending.token)
        } else {
            crate::native_api::WRONG_THREAD
        };

        let mut anchor_to_drop = None;
        {
            let mut registry = lock_registry();
            let Some(slot) = registry.slots.get_mut(index) else {
                continue;
            };
            if slot.generation != generation || slot.state != SlotState::InFlight {
                continue;
            }
            if release_status == crate::native_api::OK {
                slot.state = SlotState::Free;
                anchor_to_drop = pending.anchor.take();
                released += 1;
            } else {
                slot.pending = Some(pending);
                slot.state = SlotState::Pending;
            }
        }
        drop(anchor_to_drop);
    }
    released
}

#[cfg(test)]
pub(crate) fn pending_cleanup_count() -> usize {
    lock_registry()
        .slots
        .iter()
        .filter(|slot| slot.state == SlotState::Pending)
        .count()
}

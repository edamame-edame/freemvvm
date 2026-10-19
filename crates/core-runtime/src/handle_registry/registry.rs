use super::entry::HandleEntry;
use super::handle::HandleToken;
use super::token::PROCESS_TOKEN_ISSUER;
use std::collections::HashMap;
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

/// Stable identity for a Runtime-owned object-store slot.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(dead_code)]
pub(crate) struct ObjectIdentity {
    slot: u64,
    generation: NonZeroU64,
}

#[allow(dead_code)]
impl ObjectIdentity {
    pub(crate) const fn new(slot: u64, generation: NonZeroU64) -> Self {
        Self { slot, generation }
    }

    pub(crate) const fn slot(self) -> u64 {
        self.slot
    }

    pub(crate) const fn generation(self) -> NonZeroU64 {
        self.generation
    }
}

/// A queue accepts a finalizer reservation before the last public token is consumed.
#[allow(dead_code)]
pub(crate) trait CleanupQueue: Send + Sync {
    fn track_ui_state(&self, _on_shutdown: Box<dyn Fn() + Send + Sync>) -> Result<(), ()> {
        Ok(())
    }

    fn reserve(
        &self,
        identity: ObjectIdentity,
        on_complete: Box<dyn FnOnce() + Send>,
    ) -> Result<Box<dyn CleanupReservation>, ()>;
}

#[allow(dead_code)]
pub(crate) trait CleanupReservation: Send {
    /// Activating an accepted reservation cannot fail.
    fn activate(self: Box<Self>);
}

struct Lifecycle {
    public_refs: usize,
    pins: usize,
    ui_state_alive: bool,
    pending_cleanup: Option<Box<dyn CleanupReservation>>,
}

/// Thread-safe lifetime metadata. UI state is resolved separately on its Runtime thread.
#[allow(dead_code)]
pub(crate) struct CoreObjectControl<R> {
    runtime_id: R,
    identity: ObjectIdentity,
    cleanup_queue: Arc<dyn CleanupQueue>,
    lifecycle: Mutex<Lifecycle>,
}

#[allow(dead_code)]
impl<R> CoreObjectControl<R> {
    pub(crate) fn new(
        runtime_id: R,
        identity: ObjectIdentity,
        cleanup_queue: Arc<dyn CleanupQueue>,
    ) -> Arc<Self> {
        Arc::new(Self {
            runtime_id,
            identity,
            cleanup_queue,
            lifecycle: Mutex::new(Lifecycle {
                public_refs: 0,
                pins: 0,
                ui_state_alive: true,
                pending_cleanup: None,
            }),
        })
    }

    pub(crate) fn runtime_id(&self) -> &R {
        &self.runtime_id
    }

    pub(crate) fn identity(&self) -> ObjectIdentity {
        self.identity
    }

    fn track_ui_state(self: &Arc<Self>) -> Result<(), RegistryError>
    where
        R: Send + Sync + 'static,
    {
        let weak = Arc::downgrade(self);
        self.cleanup_queue
            .track_ui_state(Box::new(move || {
                if let Some(control) = weak.upgrade() {
                    control.mark_ui_state_cleaned();
                }
            }))
            .map_err(|()| RegistryError::WrongRuntime)
    }

    /// Called by Runtime shutdown after it has destroyed this object's UI state.
    pub(crate) fn mark_ui_state_cleaned(&self) {
        let cleanup = {
            let mut lifecycle = match self.lifecycle.lock() {
                Ok(lifecycle) => lifecycle,
                Err(poisoned) => poisoned.into_inner(),
            };
            lifecycle.ui_state_alive = false;
            if lifecycle.pins == 0 && lifecycle.public_refs == 0 {
                lifecycle.pending_cleanup.take()
            } else {
                None
            }
        };
        if let Some(cleanup) = cleanup {
            cleanup.activate();
        }
    }
}

impl<R: Send + Sync + 'static> CoreObjectControl<R> {
    fn reserve_cleanup(self: &Arc<Self>) -> Result<Box<dyn CleanupReservation>, RegistryError> {
        let weak: Weak<Self> = Arc::downgrade(self);
        let completed = Box::new(move || {
            if let Some(control) = weak.upgrade() {
                control.mark_ui_state_cleaned();
            }
        });
        self.cleanup_queue
            .reserve(self.identity, completed)
            .map_err(|()| RegistryError::CleanupUnavailable)
    }
}

/// A call-duration pin that keeps the control block alive without exposing UI state.
#[allow(dead_code)]
pub(crate) struct ValidationPin<R> {
    control: Arc<CoreObjectControl<R>>,
}

#[allow(dead_code)]
impl<R> ValidationPin<R> {
    pub(crate) fn identity(&self) -> ObjectIdentity {
        self.control.identity()
    }
}

impl<R> Drop for ValidationPin<R> {
    fn drop(&mut self) {
        let mut lifecycle = match self.control.lifecycle.lock() {
            Ok(lifecycle) => lifecycle,
            Err(poisoned) => poisoned.into_inner(),
        };
        lifecycle.pins = match lifecycle.pins.checked_sub(1) {
            Some(pins) => pins,
            None => return,
        };
        let cleanup = if lifecycle.pins == 0 && lifecycle.public_refs == 0 {
            lifecycle.pending_cleanup.take()
        } else {
            None
        };
        drop(lifecycle);
        if let Some(cleanup) = cleanup {
            cleanup.activate();
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum RegistryError {
    InvalidToken,
    WrongKind,
    WrongRuntime,
    TokenExhausted,
    OutOfMemory,
    CleanupUnavailable,
    SynchronizationPoisoned,
    ReferenceCountExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) struct ReleaseOutcome {
    pub(crate) cleanup_accepted: bool,
}

type EntryMap<K, R> = HashMap<HandleToken, HandleEntry<K, R, Arc<CoreObjectControl<R>>>>;

/// Synchronized process token table for Runtime-owned objects.
#[allow(dead_code)]
pub(crate) struct HandleRegistry<K, R> {
    entries: Mutex<EntryMap<K, R>>,
    #[cfg(test)]
    fail_next_token_issue: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    fail_next_table_reservation: std::sync::atomic::AtomicBool,
}

#[allow(dead_code)]
impl<K, R> HandleRegistry<K, R>
where
    K: Clone + Eq + Send + Sync,
    R: Clone + Eq + Send + Sync + 'static,
{
    pub(crate) fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            #[cfg(test)]
            fail_next_token_issue: std::sync::atomic::AtomicBool::new(false),
            #[cfg(test)]
            fail_next_table_reservation: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub(crate) fn register(
        &self,
        kind: K,
        runtime_id: R,
        control: Arc<CoreObjectControl<R>>,
    ) -> Result<HandleToken, RegistryError> {
        if control.runtime_id() != &runtime_id {
            return Err(RegistryError::WrongRuntime);
        }

        let mut lifecycle = self.lock_lifecycle(&control)?;
        if !lifecycle.ui_state_alive {
            return Err(RegistryError::WrongRuntime);
        }
        control.track_ui_state()?;

        let mut entries = self.lock_entries()?;
        self.reserve_table_slot(&mut entries)?;
        let token = self.issue_token()?;
        let next_ref_count = lifecycle
            .public_refs
            .checked_add(1)
            .ok_or(RegistryError::ReferenceCountExhausted)?;
        entries.insert(
            token,
            HandleEntry::new(kind, Some(runtime_id), Arc::clone(&control)),
        );
        lifecycle.public_refs = next_ref_count;
        Ok(token)
    }

    pub(crate) fn retain(&self, token: HandleToken) -> Result<HandleToken, RegistryError> {
        let (kind, runtime_id, control) = self.snapshot(token)?;
        let mut lifecycle = self.lock_lifecycle(&control)?;
        let mut entries = self.lock_entries()?;
        self.check_live_entry(&entries, token, &control)?;
        if !lifecycle.ui_state_alive || lifecycle.public_refs == 0 {
            return Err(RegistryError::InvalidToken);
        }
        self.reserve_table_slot(&mut entries)?;
        let new_token = self.issue_token()?;
        let next_ref_count = lifecycle
            .public_refs
            .checked_add(1)
            .ok_or(RegistryError::ReferenceCountExhausted)?;
        entries.insert(
            new_token,
            HandleEntry::new(kind, Some(runtime_id), Arc::clone(&control)),
        );
        lifecycle.public_refs = next_ref_count;
        Ok(new_token)
    }

    pub(crate) fn validate(
        &self,
        token: HandleToken,
        expected_kind: &K,
        expected_runtime: &R,
    ) -> Result<ValidationPin<R>, RegistryError> {
        let (_, _, control) = self.snapshot(token)?;
        let mut lifecycle = self.lock_lifecycle(&control)?;
        let entries = self.lock_entries()?;
        let entry = self.check_live_entry(&entries, token, &control)?;
        if !entry.matches_kind(expected_kind) {
            return Err(RegistryError::WrongKind);
        }
        if !entry.matches_runtime(expected_runtime) {
            return Err(RegistryError::WrongRuntime);
        }
        if !lifecycle.ui_state_alive || lifecycle.public_refs == 0 {
            return Err(RegistryError::InvalidToken);
        }
        lifecycle.pins = lifecycle
            .pins
            .checked_add(1)
            .ok_or(RegistryError::ReferenceCountExhausted)?;
        drop(entries);
        drop(lifecycle);
        Ok(ValidationPin { control })
    }

    pub(crate) fn release(&self, token: HandleToken) -> Result<ReleaseOutcome, RegistryError> {
        let (_, _, control) = self.snapshot(token)?;
        let mut lifecycle = self.lock_lifecycle(&control)?;
        {
            let entries = self.lock_entries()?;
            self.check_live_entry(&entries, token, &control)?;
        }
        if lifecycle.public_refs == 0 {
            return Err(RegistryError::InvalidToken);
        }

        let is_final_public_ref = lifecycle.public_refs == 1;
        let reservation = if is_final_public_ref && lifecycle.ui_state_alive {
            Some(control.reserve_cleanup()?)
        } else {
            None
        };

        let removed = {
            let mut entries = self.lock_entries()?;
            self.check_live_entry(&entries, token, &control)?;
            entries.remove(&token).ok_or(RegistryError::InvalidToken)?
        };
        drop(removed);

        lifecycle.public_refs -= 1;
        let cleanup_accepted = reservation.is_some();
        let activate_now = if is_final_public_ref {
            if lifecycle.pins == 0 {
                reservation
            } else {
                lifecycle.pending_cleanup = reservation;
                None
            }
        } else {
            None
        };
        drop(lifecycle);
        if let Some(cleanup) = activate_now {
            cleanup.activate();
        }
        Ok(ReleaseOutcome { cleanup_accepted })
    }

    #[cfg(test)]
    pub(crate) fn fail_next_token_issue(&self) {
        self.fail_next_token_issue
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    #[cfg(test)]
    fn fail_next_table_reservation(&self) {
        self.fail_next_table_reservation
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    fn snapshot(
        &self,
        token: HandleToken,
    ) -> Result<(K, R, Arc<CoreObjectControl<R>>), RegistryError> {
        let entries = self.lock_entries()?;
        let entry = entries.get(&token).ok_or(RegistryError::InvalidToken)?;
        let runtime_id = entry
            .runtime_id()
            .cloned()
            .ok_or(RegistryError::WrongRuntime)?;
        let control = entry
            .object_ref()
            .cloned()
            .ok_or(RegistryError::InvalidToken)?;
        Ok((entry.kind().clone(), runtime_id, control))
    }

    fn check_live_entry<'a>(
        &self,
        entries: &'a EntryMap<K, R>,
        token: HandleToken,
        control: &Arc<CoreObjectControl<R>>,
    ) -> Result<&'a HandleEntry<K, R, Arc<CoreObjectControl<R>>>, RegistryError> {
        let entry = entries.get(&token).ok_or(RegistryError::InvalidToken)?;
        let current = entry.object_ref().ok_or(RegistryError::InvalidToken)?;
        if !Arc::ptr_eq(current, control) {
            return Err(RegistryError::InvalidToken);
        }
        Ok(entry)
    }

    fn lock_entries(&self) -> Result<MutexGuard<'_, EntryMap<K, R>>, RegistryError> {
        self.entries
            .lock()
            .map_err(|_| RegistryError::SynchronizationPoisoned)
    }

    fn lock_lifecycle<'a>(
        &self,
        control: &'a CoreObjectControl<R>,
    ) -> Result<MutexGuard<'a, Lifecycle>, RegistryError> {
        control
            .lifecycle
            .lock()
            .map_err(|_| RegistryError::SynchronizationPoisoned)
    }

    fn reserve_table_slot(&self, entries: &mut EntryMap<K, R>) -> Result<(), RegistryError> {
        #[cfg(test)]
        if self
            .fail_next_table_reservation
            .swap(false, std::sync::atomic::Ordering::SeqCst)
        {
            return Err(RegistryError::OutOfMemory);
        }
        entries
            .try_reserve(1)
            .map_err(|_| RegistryError::OutOfMemory)
    }

    fn issue_token(&self) -> Result<HandleToken, RegistryError> {
        #[cfg(test)]
        if self
            .fail_next_token_issue
            .swap(false, std::sync::atomic::Ordering::SeqCst)
        {
            return Err(RegistryError::TokenExhausted);
        }
        PROCESS_TOKEN_ISSUER
            .issue()
            .map(HandleToken::from_issued)
            .ok_or(RegistryError::TokenExhausted)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CleanupQueue, CleanupReservation, CoreObjectControl, HandleRegistry, ObjectIdentity,
        RegistryError,
    };
    use crate::handle_registry::handle::HandleToken;
    use std::collections::VecDeque;
    use std::num::NonZeroU64;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread::{self, ThreadId};

    struct FakeQueue {
        state: Arc<Mutex<FakeQueueState>>,
        fail_next: AtomicBool,
    }

    #[derive(Default)]
    struct FakeQueueState {
        pending: VecDeque<PendingCleanup>,
        completed: Vec<(ObjectIdentity, ThreadId, ThreadId)>,
    }

    struct PendingCleanup {
        identity: ObjectIdentity,
        activated_by: ThreadId,
        on_complete: Box<dyn FnOnce() + Send>,
    }

    struct FakeReservation {
        state: Arc<Mutex<FakeQueueState>>,
        identity: ObjectIdentity,
        on_complete: Option<Box<dyn FnOnce() + Send>>,
    }

    impl FakeQueue {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                state: Arc::new(Mutex::new(FakeQueueState::default())),
                fail_next: AtomicBool::new(false),
            })
        }

        fn fail_next_reservation(&self) {
            self.fail_next.store(true, Ordering::SeqCst);
        }

        fn pending_count(&self) -> usize {
            self.state.lock().expect("queue lock").pending.len()
        }

        fn completed(&self) -> Vec<(ObjectIdentity, ThreadId, ThreadId)> {
            self.state.lock().expect("queue lock").completed.clone()
        }

        fn drain_on_ui_thread(&self) {
            loop {
                let pending = self.state.lock().expect("queue lock").pending.pop_front();
                let Some(pending) = pending else {
                    return;
                };
                (pending.on_complete)();
                self.state.lock().expect("queue lock").completed.push((
                    pending.identity,
                    pending.activated_by,
                    thread::current().id(),
                ));
            }
        }
    }

    impl CleanupQueue for FakeQueue {
        fn reserve(
            &self,
            identity: ObjectIdentity,
            on_complete: Box<dyn FnOnce() + Send>,
        ) -> Result<Box<dyn CleanupReservation>, ()> {
            if self.fail_next.swap(false, Ordering::SeqCst) {
                return Err(());
            }
            Ok(Box::new(FakeReservation {
                state: Arc::clone(&self.state),
                identity,
                on_complete: Some(on_complete),
            }))
        }
    }

    impl CleanupReservation for FakeReservation {
        fn activate(mut self: Box<Self>) {
            self.state
                .lock()
                .expect("queue lock")
                .pending
                .push_back(PendingCleanup {
                    identity: self.identity,
                    activated_by: thread::current().id(),
                    on_complete: self.on_complete.take().expect("unused reservation"),
                });
        }
    }

    type TestRegistry = HandleRegistry<u8, u64>;

    fn make_control(
        queue: Arc<FakeQueue>,
        runtime_id: u64,
        slot: u64,
    ) -> Arc<CoreObjectControl<u64>> {
        CoreObjectControl::new(
            runtime_id,
            ObjectIdentity::new(slot, NonZeroU64::new(1).expect("nonzero generation")),
            queue,
        )
    }

    fn make_registry_and_token(
        queue: Arc<FakeQueue>,
    ) -> (Arc<TestRegistry>, Arc<CoreObjectControl<u64>>, HandleToken) {
        let registry = Arc::new(TestRegistry::new());
        let control = make_control(queue, 11, 3);
        let token = registry
            .register(7, 11, Arc::clone(&control))
            .expect("register object");
        (registry, control, token)
    }

    #[test]
    fn register_publishes_complete_runtime_entry() {
        let queue = FakeQueue::new();
        let registry = TestRegistry::new();
        let control = make_control(Arc::clone(&queue), 11, 3);

        registry.fail_next_table_reservation();
        assert_eq!(
            registry.register(7, 11, Arc::clone(&control)),
            Err(RegistryError::OutOfMemory)
        );
        registry.fail_next_token_issue();
        assert_eq!(
            registry.register(7, 11, Arc::clone(&control)),
            Err(RegistryError::TokenExhausted)
        );
        assert_eq!(control.lifecycle.lock().expect("lifecycle").public_refs, 0);

        let token = registry
            .register(7, 11, Arc::clone(&control))
            .expect("register object");
        let pin = registry.validate(token, &7, &11).expect("validate object");
        assert_eq!(pin.identity(), control.identity());
        assert_eq!(
            registry.validate(token, &8, &11).err(),
            Some(RegistryError::WrongKind)
        );
        assert_eq!(
            registry.validate(token, &7, &12).err(),
            Some(RegistryError::WrongRuntime)
        );
        drop(pin);
    }

    #[test]
    fn pin_defers_ui_cleanup_until_last_pin() {
        let queue = FakeQueue::new();
        let (registry, _, token) = make_registry_and_token(Arc::clone(&queue));
        let pin = registry.validate(token, &7, &11).expect("validation pin");

        let result = registry.release(token).expect("release is accepted");
        assert!(result.cleanup_accepted);
        assert_eq!(queue.pending_count(), 0);

        drop(pin);
        assert_eq!(queue.pending_count(), 1);
        queue.drain_on_ui_thread();
        let completed = queue.completed();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].0.slot(), 3);
        assert_eq!(completed[0].0.generation().get(), 1);
        assert_eq!(completed[0].2, thread::current().id());
    }

    #[test]
    fn worker_final_release_is_accepted_and_drained_on_ui_thread() {
        let queue = FakeQueue::new();
        let (registry, _, token) = make_registry_and_token(Arc::clone(&queue));
        let worker_id = thread::spawn(move || {
            let worker_id = thread::current().id();
            let result = registry.release(token).expect("release accepted");
            (worker_id, result)
        })
        .join()
        .expect("release worker completes");

        assert!(worker_id.1.cleanup_accepted);
        assert_eq!(queue.pending_count(), 1);
        assert!(queue.completed().is_empty());
        queue.drain_on_ui_thread();
        let completed = queue.completed();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].1, worker_id.0);
        assert_eq!(completed[0].2, thread::current().id());
    }

    #[test]
    fn failed_cleanup_reservation_preserves_live_token() {
        let queue = FakeQueue::new();
        let (registry, control, token) = make_registry_and_token(Arc::clone(&queue));
        queue.fail_next_reservation();

        assert_eq!(
            registry.release(token),
            Err(RegistryError::CleanupUnavailable)
        );
        let pin = registry
            .validate(token, &7, &11)
            .expect("failed release keeps token live");
        drop(pin);
        assert_eq!(control.lifecycle.lock().expect("lifecycle").public_refs, 1);

        assert!(
            registry
                .release(token)
                .expect("retry release")
                .cleanup_accepted
        );
        assert_eq!(queue.pending_count(), 1);
    }

    #[test]
    fn retain_release_race_is_linearizable_without_resurrection() {
        let queue = FakeQueue::new();
        let (registry, _, token) = make_registry_and_token(Arc::clone(&queue));
        let barrier = Arc::new(Barrier::new(3));
        let retain_registry = Arc::clone(&registry);
        let retain_barrier = Arc::clone(&barrier);
        let retain = thread::spawn(move || {
            retain_barrier.wait();
            retain_registry.retain(token)
        });
        let release_registry = Arc::clone(&registry);
        let release_barrier = Arc::clone(&barrier);
        let release = thread::spawn(move || {
            release_barrier.wait();
            release_registry.release(token)
        });
        barrier.wait();

        let retained = retain.join().expect("retain worker completes");
        let released = release.join().expect("release worker completes");
        assert!(released.is_ok());
        match retained {
            Ok(new_token) => {
                let pin = registry
                    .validate(new_token, &7, &11)
                    .expect("retained token remains live");
                drop(pin);
                registry.release(new_token).expect("release retained token");
            }
            Err(RegistryError::InvalidToken) => {
                assert_eq!(
                    registry.validate(token, &7, &11).err(),
                    Some(RegistryError::InvalidToken)
                );
            }
            Err(error) => panic!("unexpected retain error: {error:?}"),
        }
        assert_eq!(queue.pending_count(), 1);
        queue.drain_on_ui_thread();
        assert_eq!(queue.completed().len(), 1);
    }
}

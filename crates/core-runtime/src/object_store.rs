use crate::handle_registry::registry::ObjectIdentity;
use crate::runtime::UiThreadContext;
use std::num::NonZeroU64;

struct Slot<T> {
    generation: NonZeroU64,
    value: Option<T>,
    next_free: Option<u64>,
}

/// Runtime-owned arena for UI state, accessible only from its owning UI thread.
pub(crate) struct UiObjectStore<T> {
    ui_thread: UiThreadContext,
    slots: Vec<Slot<T>>,
    free_head: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiObjectStoreError {
    WrongThread,
    InvalidIdentity,
    IdentityExhausted,
    OutOfMemory,
}

impl<T> UiObjectStore<T> {
    pub(crate) fn new(ui_thread: UiThreadContext) -> Self {
        Self {
            ui_thread,
            slots: Vec::new(),
            free_head: None,
        }
    }

    pub(crate) fn insert(&mut self, value: T) -> Result<ObjectIdentity, UiObjectStoreError> {
        self.ensure_ui_thread()?;
        let mut value = Some(value);

        while let Some(slot_id) = self.free_head {
            let slot_index =
                usize::try_from(slot_id).map_err(|_| UiObjectStoreError::IdentityExhausted)?;
            let slot = self
                .slots
                .get_mut(slot_index)
                .ok_or(UiObjectStoreError::InvalidIdentity)?;
            self.free_head = slot.next_free.take();

            let Some(generation) = slot
                .generation
                .get()
                .checked_add(1)
                .and_then(NonZeroU64::new)
            else {
                // Retire exhausted slots instead of allowing an old identity to become current.
                continue;
            };

            slot.generation = generation;
            slot.value = value.take();
            return Ok(ObjectIdentity::new(slot_id, generation));
        }

        let slot_id =
            u64::try_from(self.slots.len()).map_err(|_| UiObjectStoreError::IdentityExhausted)?;
        self.slots
            .try_reserve(1)
            .map_err(|_| UiObjectStoreError::OutOfMemory)?;
        self.slots.push(Slot {
            generation: NonZeroU64::MIN,
            value,
            next_free: None,
        });
        Ok(ObjectIdentity::new(slot_id, NonZeroU64::MIN))
    }

    pub(crate) fn resolve(&self, identity: ObjectIdentity) -> Result<&T, UiObjectStoreError> {
        self.ensure_ui_thread()?;
        let slot = self.slot(identity)?;
        slot.value
            .as_ref()
            .ok_or(UiObjectStoreError::InvalidIdentity)
    }

    pub(crate) fn resolve_mut(
        &mut self,
        identity: ObjectIdentity,
    ) -> Result<&mut T, UiObjectStoreError> {
        self.ensure_ui_thread()?;
        let slot = self.slot_mut(identity)?;
        slot.value
            .as_mut()
            .ok_or(UiObjectStoreError::InvalidIdentity)
    }

    pub(crate) fn destroy(&mut self, identity: ObjectIdentity) -> Result<T, UiObjectStoreError> {
        self.ensure_ui_thread()?;
        let slot_index = self.slot_index(identity)?;
        let slot = self
            .slots
            .get_mut(slot_index)
            .ok_or(UiObjectStoreError::InvalidIdentity)?;
        if slot.generation != identity.generation() {
            return Err(UiObjectStoreError::InvalidIdentity);
        }
        let value = slot
            .value
            .take()
            .ok_or(UiObjectStoreError::InvalidIdentity)?;
        slot.next_free = self.free_head;
        self.free_head = Some(identity.slot());
        Ok(value)
    }

    pub(crate) fn visit_mut(&mut self, mut visit: impl FnMut(ObjectIdentity, &mut T)) {
        for (index, slot) in self.slots.iter_mut().enumerate() {
            if let Some(value) = slot.value.as_mut() {
                if let Ok(slot_id) = u64::try_from(index) {
                    visit(ObjectIdentity::new(slot_id, slot.generation), value);
                }
            }
        }
    }

    /// Invalidates every current identity and transfers live values to the caller for cleanup
    /// after the store lock has been released.
    pub(crate) fn take_all(
        &mut self,
    ) -> Result<impl Iterator<Item = (ObjectIdentity, T)> + use<T>, UiObjectStoreError> {
        self.ensure_ui_thread()?;
        self.free_head = None;
        Ok(std::mem::take(&mut self.slots)
            .into_iter()
            .enumerate()
            .filter_map(|(index, mut slot)| {
                let identity = ObjectIdentity::new(u64::try_from(index).ok()?, slot.generation);
                slot.value.take().map(|value| (identity, value))
            }))
    }

    /// Drops all live state on the owning UI thread and invalidates every current identity.
    pub(crate) fn clear(&mut self) -> Result<(), UiObjectStoreError> {
        self.ensure_ui_thread()?;
        for slot in &mut self.slots {
            slot.value.take();
            slot.next_free = None;
        }
        self.free_head = None;
        Ok(())
    }

    fn ensure_ui_thread(&self) -> Result<(), UiObjectStoreError> {
        if self.ui_thread.is_current() {
            Ok(())
        } else {
            Err(UiObjectStoreError::WrongThread)
        }
    }

    fn slot(&self, identity: ObjectIdentity) -> Result<&Slot<T>, UiObjectStoreError> {
        let slot = self
            .slots
            .get(self.slot_index(identity)?)
            .ok_or(UiObjectStoreError::InvalidIdentity)?;
        if slot.generation != identity.generation() || slot.value.is_none() {
            return Err(UiObjectStoreError::InvalidIdentity);
        }
        Ok(slot)
    }

    fn slot_mut(&mut self, identity: ObjectIdentity) -> Result<&mut Slot<T>, UiObjectStoreError> {
        let slot_index = self.slot_index(identity)?;
        let slot = self
            .slots
            .get_mut(slot_index)
            .ok_or(UiObjectStoreError::InvalidIdentity)?;
        if slot.generation != identity.generation() || slot.value.is_none() {
            return Err(UiObjectStoreError::InvalidIdentity);
        }
        Ok(slot)
    }

    fn slot_index(&self, identity: ObjectIdentity) -> Result<usize, UiObjectStoreError> {
        usize::try_from(identity.slot()).map_err(|_| UiObjectStoreError::InvalidIdentity)
    }
}

#[cfg(test)]
mod tests {
    use super::{Slot, UiObjectStore, UiObjectStoreError};
    use crate::handle_registry::registry::ObjectIdentity;
    use crate::runtime::UiThreadContext;
    use std::num::NonZeroU64;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    struct DropProbe(Arc<AtomicUsize>);

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn inserts_resolves_mutates_and_destroys_current_identity() {
        let mut store = UiObjectStore::new(UiThreadContext::capture());
        let identity = store
            .insert(String::from("first"))
            .expect("insert on UI thread");

        assert_eq!(
            store.resolve(identity).expect("resolve live state"),
            "first"
        );
        store
            .resolve_mut(identity)
            .expect("resolve mutable live state")
            .push_str(" updated");
        assert_eq!(
            store.destroy(identity).expect("destroy live state"),
            "first updated"
        );
        assert_eq!(
            store.resolve(identity),
            Err(UiObjectStoreError::InvalidIdentity)
        );
        assert_eq!(
            store.destroy(identity),
            Err(UiObjectStoreError::InvalidIdentity)
        );
    }

    #[test]
    fn reused_slot_advances_generation_and_rejects_previous_identity() {
        let mut store = UiObjectStore::new(UiThreadContext::capture());
        let previous = store.insert(10_u8).expect("initial insert");
        store.destroy(previous).expect("destroy initial state");
        let current = store.insert(20_u8).expect("reuse vacant slot");

        assert_eq!(current.slot(), previous.slot());
        assert!(current.generation() > previous.generation());
        assert_eq!(store.resolve(current), Ok(&20));
        assert_eq!(
            store.resolve(previous),
            Err(UiObjectStoreError::InvalidIdentity)
        );
    }

    #[test]
    fn rejects_all_off_thread_access_without_changing_store() {
        let mut store = UiObjectStore::new(UiThreadContext::capture());
        let identity = store.insert(10_u8).expect("initial insert");
        let shared = Arc::new(Mutex::new(store));
        let worker_store = Arc::clone(&shared);

        let outcomes = std::thread::spawn(move || {
            let mut store = worker_store.lock().expect("store mutex");
            (
                store.resolve(identity).copied(),
                store.resolve_mut(identity).map(|value| *value),
                store.destroy(identity),
                store.insert(20),
            )
        })
        .join()
        .expect("worker completes");

        assert_eq!(outcomes.0, Err(UiObjectStoreError::WrongThread));
        assert_eq!(outcomes.1, Err(UiObjectStoreError::WrongThread));
        assert_eq!(outcomes.2, Err(UiObjectStoreError::WrongThread));
        assert_eq!(outcomes.3, Err(UiObjectStoreError::WrongThread));
        assert_eq!(
            shared.lock().expect("store mutex").resolve(identity),
            Ok(&10)
        );
    }

    #[test]
    fn retires_slot_at_maximum_generation_without_wrapping() {
        let mut store = UiObjectStore::new(UiThreadContext::capture());
        store.slots.push(Slot {
            generation: NonZeroU64::MAX,
            value: None,
            next_free: None,
        });
        store.free_head = Some(0);

        let identity = store.insert(30_u8).expect("allocate a fresh slot");

        assert_eq!(identity.slot(), 1);
        assert_eq!(identity.generation(), NonZeroU64::MIN);
        assert_eq!(store.resolve(identity), Ok(&30));
    }

    #[test]
    fn rejects_identity_that_was_never_inserted() {
        let store = UiObjectStore::<u8>::new(UiThreadContext::capture());
        let identity = ObjectIdentity::new(5, NonZeroU64::new(1).expect("nonzero generation"));

        assert_eq!(
            store.resolve(identity),
            Err(UiObjectStoreError::InvalidIdentity)
        );
    }

    #[test]
    fn clear_drops_all_live_values_once_and_invalidates_identities() {
        let drop_count = Arc::new(AtomicUsize::new(0));
        let mut store = UiObjectStore::new(UiThreadContext::capture());
        let first = store
            .insert(DropProbe(Arc::clone(&drop_count)))
            .expect("insert first value");
        let second = store
            .insert(DropProbe(Arc::clone(&drop_count)))
            .expect("insert second value");

        store.clear().expect("clear on UI thread");

        assert_eq!(drop_count.load(Ordering::SeqCst), 2);
        assert!(matches!(
            store.resolve(first),
            Err(UiObjectStoreError::InvalidIdentity)
        ));
        assert!(matches!(
            store.resolve(second),
            Err(UiObjectStoreError::InvalidIdentity)
        ));
        store.clear().expect("clear is idempotent");
        assert_eq!(drop_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn off_thread_clear_preserves_live_values() {
        let drop_count = Arc::new(AtomicUsize::new(0));
        let mut store = UiObjectStore::new(UiThreadContext::capture());
        let identity = store
            .insert(DropProbe(Arc::clone(&drop_count)))
            .expect("insert value");
        let shared = Arc::new(Mutex::new(store));
        let worker_store = Arc::clone(&shared);

        let result = std::thread::spawn(move || worker_store.lock().expect("store mutex").clear())
            .join()
            .expect("worker completes");

        assert_eq!(result, Err(UiObjectStoreError::WrongThread));
        assert_eq!(drop_count.load(Ordering::SeqCst), 0);
        assert!(
            shared
                .lock()
                .expect("store mutex")
                .resolve(identity)
                .is_ok()
        );
    }
}

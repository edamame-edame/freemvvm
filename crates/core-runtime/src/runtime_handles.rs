use crate::handle_registry::{HandleToken, issue_handle_token};
use crate::runtime::RuntimeCore;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum RuntimeHandleError {
    InvalidToken,
    TokenUnavailable,
    OutOfMemory,
    SynchronizationPoisoned,
}

/// Synchronized table for opaque Runtime owner handles.
#[allow(dead_code)]
pub(crate) struct RuntimeHandleRegistry {
    entries: Mutex<HashMap<HandleToken, Arc<RuntimeCore>>>,
}

#[allow(dead_code)]
impl RuntimeHandleRegistry {
    pub(crate) fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn insert(
        &self,
        runtime: &Arc<RuntimeCore>,
    ) -> Result<HandleToken, RuntimeHandleError> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| RuntimeHandleError::SynchronizationPoisoned)?;
        entries
            .try_reserve(1)
            .map_err(|_| RuntimeHandleError::OutOfMemory)?;
        let token = issue_handle_token().ok_or(RuntimeHandleError::TokenUnavailable)?;
        entries.insert(token, Arc::clone(runtime));
        Ok(token)
    }

    /// Returns a temporary strong owner for one borrowed Runtime handle lookup.
    pub(crate) fn resolve(
        &self,
        token: HandleToken,
    ) -> Result<Arc<RuntimeCore>, RuntimeHandleError> {
        self.entries
            .lock()
            .map_err(|_| RuntimeHandleError::SynchronizationPoisoned)?
            .get(&token)
            .cloned()
            .ok_or(RuntimeHandleError::InvalidToken)
    }

    pub(crate) fn retain(&self, token: HandleToken) -> Result<HandleToken, RuntimeHandleError> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| RuntimeHandleError::SynchronizationPoisoned)?;
        let runtime = entries
            .get(&token)
            .cloned()
            .ok_or(RuntimeHandleError::InvalidToken)?;
        entries
            .try_reserve(1)
            .map_err(|_| RuntimeHandleError::OutOfMemory)?;
        let retained_token = issue_handle_token().ok_or(RuntimeHandleError::TokenUnavailable)?;
        entries.insert(retained_token, runtime);
        Ok(retained_token)
    }

    pub(crate) fn remove(&self, token: HandleToken) -> Result<(), RuntimeHandleError> {
        self.entries
            .lock()
            .map_err(|_| RuntimeHandleError::SynchronizationPoisoned)?
            .remove(&token)
            .map(|_| ())
            .ok_or(RuntimeHandleError::InvalidToken)
    }
}

static PROCESS_RUNTIME_HANDLES: OnceLock<RuntimeHandleRegistry> = OnceLock::new();

#[allow(dead_code)]
pub(crate) fn process_runtime_handles() -> &'static RuntimeHandleRegistry {
    PROCESS_RUNTIME_HANDLES.get_or_init(RuntimeHandleRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::{RuntimeHandleError, RuntimeHandleRegistry};
    use crate::handle_registry::HandleToken;
    use crate::runtime::RuntimeCore;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn runtime_handles_resolve_to_their_owners_from_any_thread() {
        let registry = Arc::new(RuntimeHandleRegistry::new());
        let first = Arc::new(RuntimeCore::new().expect("first Runtime is created"));
        let second = Arc::new(RuntimeCore::new().expect("second Runtime is created"));
        let first_token = registry
            .insert(&first)
            .expect("first Runtime handle is issued");
        let second_token = registry
            .insert(&second)
            .expect("second Runtime handle is issued");

        assert_ne!(first_token, second_token);
        assert!(Arc::ptr_eq(
            &registry
                .resolve(first_token)
                .expect("owner thread resolves"),
            &first
        ));
        assert!(Arc::ptr_eq(
            &registry
                .resolve(second_token)
                .expect("owner thread resolves"),
            &second
        ));

        let worker_registry = Arc::clone(&registry);
        let (worker, worker_is_ui_thread) = thread::spawn(move || {
            let worker = worker_registry
                .resolve(first_token)
                .expect("worker resolves borrowed Runtime handle");
            let is_ui_thread = worker.is_ui_thread();
            (worker, is_ui_thread)
        })
        .join()
        .expect("worker lookup completes");
        assert!(Arc::ptr_eq(&worker, &first));
        assert!(!worker_is_ui_thread);
        assert!(HandleToken::from_abi(0).is_none());
        assert_eq!(
            registry
                .resolve(HandleToken::from_abi(u64::MAX).expect("nonzero token"))
                .err(),
            Some(RuntimeHandleError::InvalidToken)
        );
        assert!(Arc::ptr_eq(
            &registry
                .resolve(first_token)
                .expect("invalid lookup preserves table"),
            &first
        ));
    }

    #[test]
    fn poisoned_runtime_handle_registry_fails_without_taking_owner() {
        let registry = Arc::new(RuntimeHandleRegistry::new());
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime is created"));
        let poisoned_registry = Arc::clone(&registry);
        let worker = thread::spawn(move || {
            let _entries = poisoned_registry
                .entries
                .lock()
                .expect("table starts synchronized");
            panic!("poison the Runtime handle table");
        });
        assert!(worker.join().is_err());

        assert_eq!(
            registry.insert(&runtime).err(),
            Some(RuntimeHandleError::SynchronizationPoisoned)
        );
        assert_eq!(Arc::strong_count(&runtime), 1);
    }

    #[test]
    fn retain_issues_independent_token_to_same_runtime() {
        let registry = RuntimeHandleRegistry::new();
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime is created"));
        let original = registry
            .insert(&runtime)
            .expect("original Runtime handle is issued");
        let retained = registry
            .retain(original)
            .expect("retained Runtime handle is issued");

        assert_ne!(original, retained);
        assert!(Arc::ptr_eq(
            &registry.resolve(original).expect("original remains valid"),
            &runtime
        ));
        assert!(Arc::ptr_eq(
            &registry
                .resolve(retained)
                .expect("new token resolves to same owner"),
            &runtime
        ));
        assert_eq!(Arc::strong_count(&runtime), 3);

        let unknown = HandleToken::from_abi(u64::MAX).expect("nonzero unknown token");
        assert_eq!(
            registry.retain(unknown).err(),
            Some(RuntimeHandleError::InvalidToken)
        );
        assert!(Arc::ptr_eq(
            &registry
                .resolve(original)
                .expect("rejected retain preserves source"),
            &runtime
        ));
    }
}

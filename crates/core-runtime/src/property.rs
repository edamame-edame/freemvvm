use crate::handle_registry::registry::ObjectIdentity;
use crate::value::{TypedValue, ValueCommit, ValueKind, ValueKindMismatch, ValueStorage};

/// Kind stored in the internal Runtime handle registry for an initial Property.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PropertyHandleKind {
    Property,
    #[cfg(test)]
    Other,
}

/// Runtime-owned state for one Property with a fixed initial value kind.
pub(crate) struct PropertyState {
    value: ValueStorage,
}

/// Runtime-local first-change-ordered set for the next pending Property generation.
#[derive(Default)]
pub(crate) struct PropertyChangeQueue {
    pending: Vec<ObjectIdentity>,
}

/// Active/in-flight state for one Property subscription.
///
/// This primitive does not own the callback closure. It reports the single transition at which
/// the owner may destroy that closure: immediately on cancellation with no delivery in flight,
/// or when the last in-flight delivery ends after cancellation.
#[derive(Default)]
#[allow(dead_code)] // Wired into Runtime-owned subscriptions in the follow-up integration slice.
pub(crate) struct SubscriptionLifecycle {
    state: std::sync::Mutex<SubscriptionLifecycleState>,
}

#[derive(Default)]
struct SubscriptionLifecycleState {
    active: bool,
    in_flight: usize,
    destroy_ready: bool,
}

#[allow(dead_code)] // Methods are exercised by the lifecycle oracle before ABI integration.
impl SubscriptionLifecycle {
    pub(crate) fn new() -> Self {
        Self {
            state: std::sync::Mutex::new(SubscriptionLifecycleState {
                active: true,
                in_flight: 0,
                destroy_ready: false,
            }),
        }
    }

    /// Starts a delivery only while active.
    pub(crate) fn begin_delivery(&self) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !state.active {
            return false;
        }
        let Some(next) = state.in_flight.checked_add(1) else {
            return false;
        };
        state.in_flight = next;
        true
    }

    /// Completes a started delivery and reports the one-time destroy-ready transition.
    pub(crate) fn finish_delivery(&self) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.in_flight == 0 {
            return false;
        }
        state.in_flight -= 1;
        if !state.active && state.in_flight == 0 && !state.destroy_ready {
            state.destroy_ready = true;
            return true;
        }
        false
    }

    /// Cancels future starts and reports whether destruction is immediately safe.
    pub(crate) fn cancel(&self) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.active = false;
        if state.in_flight == 0 && !state.destroy_ready {
            state.destroy_ready = true;
            return true;
        }
        false
    }
}

impl PropertyChangeQueue {
    /// Reserves one queue slot before the corresponding Property value is committed.
    pub(crate) fn reserve_change(
        &mut self,
        identity: ObjectIdentity,
    ) -> Result<bool, std::collections::TryReserveError> {
        if self.pending.contains(&identity) {
            return Ok(false);
        }
        self.pending.try_reserve(1)?;
        Ok(true)
    }

    /// Adds a previously reserved identity without allocating.
    pub(crate) fn commit_change(&mut self, identity: ObjectIdentity) {
        if !self.pending.contains(&identity) {
            self.pending.push(identity);
        }
    }

    /// Starts the next generation; subsequent changes are collected independently.
    pub(crate) fn take_generation(&mut self) -> Vec<ObjectIdentity> {
        std::mem::take(&mut self.pending)
    }
}

impl PropertyState {
    pub(crate) fn new(initial: TypedValue) -> Self {
        Self {
            value: ValueStorage::new(initial),
        }
    }

    pub(crate) fn read(&self) -> &TypedValue {
        self.value.read()
    }

    pub(crate) fn kind(&self) -> ValueKind {
        self.value.read().kind()
    }

    pub(crate) fn would_change(&self, candidate: &TypedValue) -> Result<bool, ValueKindMismatch> {
        self.value.would_change(candidate)
    }

    pub(crate) fn apply_validated(
        &mut self,
        candidate: TypedValue,
    ) -> Result<ValueCommit, ValueKindMismatch> {
        self.value.apply_validated(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriptionLifecycle;

    #[test]
    fn subscription_lifecycle_allows_delivery_until_cancel() {
        let lifecycle = SubscriptionLifecycle::new();

        assert!(lifecycle.begin_delivery());
        assert!(!lifecycle.finish_delivery());
        assert!(lifecycle.cancel());
        assert!(!lifecycle.begin_delivery());
        assert!(!lifecycle.cancel());
        assert!(!lifecycle.finish_delivery());
    }

    #[test]
    fn subscription_lifecycle_defers_destroy_until_all_deliveries_finish() {
        let lifecycle = SubscriptionLifecycle::new();

        assert!(lifecycle.begin_delivery());
        assert!(lifecycle.begin_delivery());
        assert!(!lifecycle.cancel());
        assert!(!lifecycle.begin_delivery());
        assert!(!lifecycle.finish_delivery());
        assert!(lifecycle.finish_delivery());
        assert!(!lifecycle.finish_delivery());
        assert!(!lifecycle.cancel());
    }

    #[test]
    fn subscription_lifecycle_cancel_without_delivery_is_destroy_ready_once() {
        let lifecycle = SubscriptionLifecycle::new();

        assert!(lifecycle.cancel());
        assert!(!lifecycle.cancel());
        assert!(!lifecycle.begin_delivery());
        assert!(!lifecycle.finish_delivery());
    }
}

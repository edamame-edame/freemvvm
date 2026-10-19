use crate::handle_registry::registry::ObjectIdentity;
use crate::value::{TypedValue, ValueCommit, ValueKind, ValueKindMismatch, ValueStorage};
use std::ffi::c_void;
use std::sync::Mutex;

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
pub(crate) struct SubscriptionLifecycle {
    state: Mutex<SubscriptionLifecycleState>,
}

#[derive(Default)]
struct SubscriptionLifecycleState {
    active: bool,
    in_flight: usize,
    destroy_ready: bool,
}

impl SubscriptionLifecycle {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(SubscriptionLifecycleState {
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

pub(crate) type SubscriptionCallback = unsafe extern "C" fn(*mut c_void);

#[derive(Clone, Copy)]
struct SubscriptionClosure {
    callback: SubscriptionCallback,
    userdata: usize,
    destroy: SubscriptionCallback,
}

/// One owned callback closure coupled to its cancel/in-flight state.
///
/// Delivery and cancellation must be called on the owning Runtime's UI thread. The Runtime
/// integration layer enforces that precondition; this primitive keeps user code outside locks.
#[allow(dead_code)] // Connected to Runtime-owned registration storage in the follow-up slice.
pub(crate) struct Subscription {
    lifecycle: SubscriptionLifecycle,
    closure: Mutex<Option<SubscriptionClosure>>,
}

#[allow(dead_code)] // Exercised now; Runtime integration consumes these methods in a later slice.
impl Subscription {
    /// Takes ownership of one callback closure.
    ///
    /// # Safety
    /// `callback`, `destroy`, and `userdata` must remain valid according to the Subscription ABI
    /// contract until `destroy` is called. The caller must cancel this Subscription on its UI
    /// thread before dropping it if no delivery is in flight.
    pub(crate) unsafe fn new(
        callback: SubscriptionCallback,
        userdata: *mut c_void,
        destroy: SubscriptionCallback,
    ) -> Self {
        Self {
            lifecycle: SubscriptionLifecycle::new(),
            closure: Mutex::new(Some(SubscriptionClosure {
                callback,
                userdata: userdata as usize,
                destroy,
            })),
        }
    }

    /// Runs one callback if the registration is still active.
    pub(crate) fn invoke(&self) -> bool {
        if !self.lifecycle.begin_delivery() {
            return false;
        }

        let closure = self
            .closure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .copied();
        if let Some(closure) = closure {
            // SAFETY: `new` establishes validity for the owned callback and userdata tuple.
            // Delivery is constrained to the owning Runtime's UI thread by its integration layer.
            unsafe { (closure.callback)(closure.userdata as *mut c_void) };
        }

        if self.lifecycle.finish_delivery() {
            self.destroy_closure();
        }
        closure.is_some()
    }

    /// Prevents future delivery starts and destroys the closure when no callback is in flight.
    pub(crate) fn cancel(&self) {
        if self.lifecycle.cancel() {
            self.destroy_closure();
        }
    }

    fn destroy_closure(&self) {
        let closure = self
            .closure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(closure) = closure {
            // SAFETY: the closure is taken once, after cancel and all in-flight callbacks finish.
            // The owning Runtime invokes cancel/delivery on its UI thread.
            unsafe { (closure.destroy)(closure.userdata as *mut c_void) };
        }
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
    use super::{Subscription, SubscriptionLifecycle};
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct CallbackProbe {
        callbacks: AtomicUsize,
        destroys: AtomicUsize,
        destroyed_during_callback: AtomicBool,
        cancel_on_callback: AtomicBool,
        subscription: Mutex<Option<Arc<Subscription>>>,
    }

    unsafe extern "C" fn record_callback(userdata: *mut c_void) {
        // SAFETY: the test keeps CallbackProbe alive until after the Subscription is canceled.
        let probe = unsafe { &*userdata.cast::<CallbackProbe>() };
        probe.callbacks.fetch_add(1, Ordering::SeqCst);
        if probe.cancel_on_callback.load(Ordering::SeqCst) {
            let subscription = probe
                .subscription
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_ref()
                .cloned()
                .expect("test Subscription is installed before invoke");
            subscription.cancel();
            probe
                .destroyed_during_callback
                .store(probe.destroys.load(Ordering::SeqCst) != 0, Ordering::SeqCst);
        }
    }

    unsafe extern "C" fn record_destroy(userdata: *mut c_void) {
        // SAFETY: the test keeps CallbackProbe alive until after the Subscription is canceled.
        let probe = unsafe { &*userdata.cast::<CallbackProbe>() };
        probe.destroys.fetch_add(1, Ordering::SeqCst);
    }

    fn make_subscription(probe: &CallbackProbe) -> Arc<Subscription> {
        let userdata = (std::ptr::from_ref(probe)).cast_mut().cast::<c_void>();
        // SAFETY: the test owns `probe` until after cancellation, and both callbacks use only it.
        Arc::new(unsafe { Subscription::new(record_callback, userdata, record_destroy) })
    }

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

    #[test]
    fn subscription_callback_is_owned_until_explicit_cancel() {
        let probe = CallbackProbe::default();
        let subscription = make_subscription(&probe);

        assert!(subscription.invoke());
        assert_eq!(probe.callbacks.load(Ordering::SeqCst), 1);
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 0);

        subscription.cancel();
        subscription.cancel();
        assert!(!subscription.invoke());
        assert_eq!(probe.callbacks.load(Ordering::SeqCst), 1);
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn subscription_callback_self_cancel_destroys_after_callback_returns() {
        let probe = Box::new(CallbackProbe::default());
        probe.cancel_on_callback.store(true, Ordering::SeqCst);
        let subscription = make_subscription(&probe);
        *probe
            .subscription
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::clone(&subscription));

        assert!(subscription.invoke());
        assert_eq!(probe.callbacks.load(Ordering::SeqCst), 1);
        assert!(!probe.destroyed_during_callback.load(Ordering::SeqCst));
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 1);
        assert!(!subscription.invoke());

        probe
            .subscription
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
}

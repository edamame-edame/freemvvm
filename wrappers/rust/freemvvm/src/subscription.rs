use crate::error::UiError;
use crate::handle::{Owned, RuntimeAnchor, TicketReservation};
use crate::native_api::{self, SharedApi};
use crate::runtime::UiContext;
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

pub(crate) struct SubscriptionTag;

struct CallbackState<F> {
    callback: F,
}

/// An owned Property notification registration.
///
/// Releasing this handle does not cancel the registration. Use [`Subscription::cancel`] to stop
/// future callback starts.
pub struct Subscription {
    owner: Owned<SubscriptionTag>,
    identity: Arc<()>,
}

impl Subscription {
    pub(crate) fn from_owner(owner: Owned<SubscriptionTag>, identity: Arc<()>) -> Self {
        Self { owner, identity }
    }

    pub(crate) fn register<F>(
        property: u64,
        api: SharedApi,
        runtime_token: u64,
        anchor: Arc<RuntimeAnchor>,
        identity: Arc<()>,
        callback: F,
    ) -> Result<Self, UiError>
    where
        F: FnMut() + 'static,
    {
        Self::register_with(
            property,
            api,
            runtime_token,
            anchor,
            identity,
            callback,
            false,
        )
    }

    pub(crate) fn register_binding_error<F>(
        binding: u64,
        api: SharedApi,
        runtime_token: u64,
        anchor: Arc<RuntimeAnchor>,
        identity: Arc<()>,
        callback: F,
    ) -> Result<Self, UiError>
    where
        F: FnMut() + 'static,
    {
        Self::register_with(
            binding,
            api,
            runtime_token,
            anchor,
            identity,
            callback,
            true,
        )
    }

    fn register_with<F>(
        source: u64,
        api: SharedApi,
        runtime_token: u64,
        anchor: Arc<RuntimeAnchor>,
        identity: Arc<()>,
        callback: F,
        binding_error: bool,
    ) -> Result<Self, UiError>
    where
        F: FnMut() + 'static,
    {
        let subscription_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let state = Box::new(CallbackState { callback });
        let userdata = Box::into_raw(state).cast::<c_void>();

        // SAFETY: userdata is a unique Box allocation kept alive until the native destroy callback.
        // The Core contract serializes and does not recursively invoke this UI-only callback.
        let (status, token, error) = unsafe {
            let mut token = 0;
            let mut error = 0;
            let status = if binding_error {
                freemvvm_sys::ui_abi_1_binding_subscribe_error(
                    source,
                    Some(notification_trampoline::<F>),
                    userdata,
                    Some(destroy_trampoline::<F>),
                    &mut token,
                    &mut error,
                )
            } else {
                freemvvm_sys::ui_abi_1_property_subscribe(
                    source,
                    Some(notification_trampoline::<F>),
                    userdata,
                    Some(destroy_trampoline::<F>),
                    &mut token,
                    &mut error,
                )
            };
            (status, token, error)
        };

        if status == native_api::OK && token != 0 && error == 0 {
            return Ok(Self {
                owner: Owned::from_token(
                    token,
                    api,
                    subscription_ticket,
                    runtime_token,
                    Some(anchor),
                ),
                identity,
            });
        }

        if status != native_api::OK && token == 0 {
            // Registration failure leaves callback userdata owned by this wrapper.
            // SAFETY: the native contract transfers userdata only when registration succeeds.
            drop(unsafe { Box::from_raw(userdata.cast::<CallbackState<F>>()) });
        } else if status != native_api::OK {
            // A malformed failure result unexpectedly returned a token. Preserve userdata unless
            // native cancellation succeeds and runs the registered destroy callback.
            // SAFETY: the C ABI permits a null optional error output and token is live.
            let _ =
                unsafe { freemvvm_sys::ui_abi_1_subscription_cancel(token, std::ptr::null_mut()) };
        } else if token != 0 {
            // A success status transfers callback ownership. Cancel the unexpected registration
            // before releasing its returned token and report the inconsistent result shape.
            // SAFETY: the C ABI permits a null optional error output and token is live.
            let _ =
                unsafe { freemvvm_sys::ui_abi_1_subscription_cancel(token, std::ptr::null_mut()) };
        }
        // On impossible success-without-token the raw allocation remains with Core, avoiding
        // use-after-free when the registration result itself is inconsistent.

        if token != 0 {
            drop(Owned::<SubscriptionTag>::from_token(
                token,
                Arc::clone(&api),
                subscription_ticket,
                runtime_token,
                Some(Arc::clone(&anchor)),
            ));
        } else {
            subscription_ticket.release();
        }

        let result_status = if status == native_api::OK {
            native_api::INTERNAL_ERROR
        } else {
            status
        };
        Err(UiError::capture(
            &api,
            result_status,
            error,
            error_ticket,
            runtime_token,
            Some(anchor),
        ))
    }

    /// Stops future callback starts. A callback already in flight is allowed to finish.
    pub fn cancel(&mut self, ui: &UiContext<'_>) -> Result<(), UiError> {
        self.check_context(ui)?;
        ui.check_ui_thread()?;
        let token = self.owner.borrow()?.token();
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, error) = self.owner.api().subscription_cancel(token);
        if status == native_api::OK && error == 0 {
            error_ticket.release();
            return Ok(());
        }

        let result_status = if status == native_api::OK {
            native_api::INTERNAL_ERROR
        } else {
            status
        };
        Err(UiError::capture(
            self.owner.api(),
            result_status,
            error,
            error_ticket,
            self.owner.runtime_token(),
            self.owner.anchor(),
        ))
    }

    /// Releases only the public registration handle; it does not cancel the registration.
    pub fn close(&mut self) -> Result<(), UiError> {
        self.owner.close()
    }

    fn check_context(&self, ui: &UiContext<'_>) -> Result<(), UiError> {
        if Arc::ptr_eq(&self.identity, &ui.identity()) {
            Ok(())
        } else {
            Err(UiError::from_status(native_api::WRONG_RUNTIME))
        }
    }
}

unsafe extern "C" fn notification_trampoline<F>(userdata: *mut c_void)
where
    F: FnMut() + 'static,
{
    if userdata.is_null() {
        return;
    }
    // SAFETY: Core calls with the registered allocation, serialized and non-recursively.
    let state = unsafe { &mut *userdata.cast::<CallbackState<F>>() };
    // The panic hook provides the diagnostic; catch_unwind prevents Rust unwinding across C ABI.
    let _ = catch_unwind(AssertUnwindSafe(|| (state.callback)()));
}

unsafe extern "C" fn destroy_trampoline<F>(userdata: *mut c_void)
where
    F: FnMut() + 'static,
{
    if userdata.is_null() {
        return;
    }
    // SAFETY: Core invokes destroy once after the last in-flight callback and transfers Box ownership.
    let state = unsafe { Box::from_raw(userdata.cast::<CallbackState<F>>()) };
    let _ = catch_unwind(AssertUnwindSafe(|| drop(state)));
}

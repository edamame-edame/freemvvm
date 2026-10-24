use crate::error::UiError;
use crate::handle::{
    AnyThreadOwned, Borrowed, Owned, RuntimeAnchor, TicketReservation, retry_pending_cleanup,
};
use crate::native_api::{self, LinkedApi, SharedApi};
use std::sync::Arc;

struct RuntimeTag;

/// A UI-affine owner for one Runtime handle.
///
/// ```compile_fail
/// fn assert_send<T: Send>() {}
/// assert_send::<freemvvm::UiRuntime>();
/// ```
pub struct UiRuntime {
    owner: Owned<RuntimeTag>,
    identity: Arc<()>,
}

impl UiRuntime {
    /// Creates a Runtime using the shared `freemvvm_abi_1` dynamic library.
    pub fn try_create() -> Result<Self, UiError> {
        Self::try_create_with_api(Arc::new(LinkedApi))
    }

    pub fn try_clone(&self) -> Result<Self, UiError> {
        Ok(Self {
            owner: self.owner.try_clone()?,
            identity: Arc::clone(&self.identity),
        })
    }

    /// Checks the creating thread and returns a context for UI-affine calls.
    pub fn ui_context(&self) -> Result<UiContext<'_>, UiError> {
        let handle = self.owner.borrow()?;
        let runtime = handle.token();
        let (status, is_ui_thread) = self.owner.api().runtime_is_ui_thread(runtime);
        if status != native_api::OK {
            return Err(UiError::from_status(status));
        }
        match is_ui_thread {
            1 => {}
            0 => return Err(UiError::from_status(native_api::WRONG_THREAD)),
            _ => return Err(UiError::from_status(native_api::INTERNAL_ERROR)),
        }

        retry_pending_cleanup();
        Ok(UiContext {
            handle,
            identity: Arc::clone(&self.identity),
            api: Arc::clone(self.owner.api()),
        })
    }

    pub fn is_ui_thread(&self) -> Result<bool, UiError> {
        let handle = self.owner.borrow()?;
        let runtime = handle.token();
        let (status, value) = self.owner.api().runtime_is_ui_thread(runtime);
        if status != native_api::OK {
            return Err(UiError::from_status(status));
        }
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(UiError::from_status(native_api::INTERNAL_ERROR)),
        }
    }

    pub fn shutdown(&self, ui: &UiContext<'_>) -> Result<ShutdownDisposition, UiError> {
        ui.check_runtime(self)?;
        let runtime = ui.handle.token();
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, error) = self.owner.api().runtime_shutdown(runtime);
        if status == native_api::OK {
            if error != 0 {
                return Err(UiError::capture(
                    self.owner.api(),
                    native_api::INTERNAL_ERROR,
                    error,
                    error_ticket,
                    runtime,
                    None,
                ));
            }
            retry_pending_cleanup();
            return Ok(ShutdownDisposition::Completed);
        }
        if status == native_api::PENDING {
            if error != 0 {
                return Err(UiError::capture(
                    self.owner.api(),
                    native_api::INTERNAL_ERROR,
                    error,
                    error_ticket,
                    runtime,
                    None,
                ));
            }
            return Ok(ShutdownDisposition::AcceptedPending);
        }
        Err(UiError::capture(
            self.owner.api(),
            status,
            error,
            error_ticket,
            runtime,
            None,
        ))
    }

    pub fn request_shutdown(
        &self,
        ui: &UiContext<'_>,
        mode: ShutdownMode,
    ) -> Result<ShutdownRequest, UiError> {
        ui.check_runtime(self)?;
        let runtime = ui.handle.token();
        let anchor = RuntimeAnchor::try_create(Arc::clone(self.owner.api()), runtime)?;
        let request_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, request, error) = self
            .owner
            .api()
            .runtime_request_shutdown(runtime, mode as u32);

        if status != native_api::OK && status != native_api::PENDING {
            if request != 0 {
                dispose_unexpected_token(
                    request,
                    request_ticket,
                    Arc::clone(self.owner.api()),
                    anchor.token(),
                    Some(Arc::clone(&anchor)),
                );
            }
            return Err(UiError::capture(
                self.owner.api(),
                status,
                error,
                error_ticket,
                anchor.token(),
                Some(anchor),
            ));
        }

        if request == 0 {
            request_ticket.release();
            let error = UiError::capture(
                self.owner.api(),
                native_api::INTERNAL_ERROR,
                error,
                error_ticket,
                anchor.token(),
                Some(Arc::clone(&anchor)),
            );
            return Err(error);
        }

        if error != 0 {
            dispose_unexpected_token(
                request,
                request_ticket,
                Arc::clone(self.owner.api()),
                anchor.token(),
                Some(Arc::clone(&anchor)),
            );
            return Err(UiError::capture(
                self.owner.api(),
                native_api::INTERNAL_ERROR,
                error,
                error_ticket,
                anchor.token(),
                Some(anchor),
            ));
        }

        Ok(ShutdownRequest {
            owner: AnyThreadOwned::from_token(
                request,
                Arc::clone(self.owner.api()),
                request_ticket,
                anchor,
            ),
        })
    }

    /// Releases the public Runtime handle without implicitly shutting the Runtime down.
    pub fn close(&mut self) -> Result<(), UiError> {
        self.owner.close()
    }

    fn try_create_with_api(api: SharedApi) -> Result<Self, UiError> {
        let (version_status, major, _, _) = api.get_version();
        if version_status != native_api::OK {
            return Err(UiError::from_status(version_status));
        }
        if major != 1 {
            return Err(UiError::from_status(native_api::UNSUPPORTED));
        }

        // Reserve both ownership records before the C ABI can create owned tokens.
        let runtime_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, runtime, error) = api.runtime_create();
        if status != native_api::OK {
            if runtime != 0 {
                dispose_unexpected_token(runtime, runtime_ticket, Arc::clone(&api), 0, None);
            }
            return Err(UiError::capture(&api, status, error, error_ticket, 0, None));
        }
        if runtime == 0 {
            runtime_ticket.release();
            return Err(UiError::capture(
                &api,
                native_api::INTERNAL_ERROR,
                error,
                error_ticket,
                0,
                None,
            ));
        }
        if error != 0 {
            dispose_unexpected_token(runtime, runtime_ticket, Arc::clone(&api), runtime, None);
            return Err(UiError::capture(
                &api,
                native_api::INTERNAL_ERROR,
                error,
                error_ticket,
                runtime,
                None,
            ));
        }

        Ok(Self {
            owner: Owned::from_token(runtime, api, runtime_ticket, runtime, None),
            identity: Arc::new(()),
        })
    }
}

pub struct UiContext<'a> {
    handle: Borrowed<'a, RuntimeTag>,
    identity: Arc<()>,
    api: SharedApi,
}

impl UiContext<'_> {
    pub(crate) fn check_ui_thread(&self) -> Result<(), UiError> {
        let (status, is_ui_thread) = self.api.runtime_is_ui_thread(self.handle.token());
        if status != native_api::OK {
            return Err(UiError::from_status(status));
        }
        match is_ui_thread {
            1 => Ok(()),
            0 => Err(UiError::from_status(native_api::WRONG_THREAD)),
            _ => Err(UiError::from_status(native_api::INTERNAL_ERROR)),
        }
    }

    pub(crate) fn runtime_token(&self) -> u64 {
        self.handle.token()
    }

    pub(crate) fn api(&self) -> &SharedApi {
        &self.api
    }

    pub(crate) fn identity(&self) -> Arc<()> {
        Arc::clone(&self.identity)
    }

    pub fn check_runtime(&self, runtime: &UiRuntime) -> Result<(), UiError> {
        if Arc::ptr_eq(&self.identity, &runtime.identity) {
            Ok(())
        } else {
            Err(UiError::from_status(native_api::WRONG_RUNTIME))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ShutdownMode {
    Async = freemvvm_sys::SHUTDOWN_ASYNC,
    Sync = freemvvm_sys::SHUTDOWN_SYNC,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShutdownDisposition {
    Completed,
    AcceptedPending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ShutdownRequestState {
    Pending = freemvvm_sys::SHUTDOWN_REQUEST_PENDING,
    Completed = freemvvm_sys::SHUTDOWN_REQUEST_COMPLETED,
    HostUnavailable = freemvvm_sys::SHUTDOWN_REQUEST_HOST_UNAVAILABLE,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShutdownSnapshot {
    pub raw_state: u32,
    pub reason: u32,
}

impl ShutdownSnapshot {
    pub fn known_state(self) -> Option<ShutdownRequestState> {
        match self.raw_state {
            freemvvm_sys::SHUTDOWN_REQUEST_PENDING => Some(ShutdownRequestState::Pending),
            freemvvm_sys::SHUTDOWN_REQUEST_COMPLETED => Some(ShutdownRequestState::Completed),
            freemvvm_sys::SHUTDOWN_REQUEST_HOST_UNAVAILABLE => {
                Some(ShutdownRequestState::HostUnavailable)
            }
            _ => None,
        }
    }
}

pub struct ShutdownRequest {
    owner: AnyThreadOwned,
}

impl ShutdownRequest {
    /// Queries the immutable shutdown completion record; this operation is thread-independent.
    pub fn query(&self) -> Result<ShutdownSnapshot, UiError> {
        let request = self.owner.token()?;
        let anchor = self.owner.anchor();
        let runtime_token = anchor.as_ref().map_or(0, |value| value.token());
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, state, reason, error) = self.owner.api().shutdown_request_query(request);
        if status != native_api::OK {
            return Err(UiError::capture(
                self.owner.api(),
                status,
                error,
                error_ticket,
                runtime_token,
                anchor,
            ));
        }
        if error != 0 {
            return Err(UiError::capture(
                self.owner.api(),
                native_api::INTERNAL_ERROR,
                error,
                error_ticket,
                runtime_token,
                anchor,
            ));
        }
        if state == freemvvm_sys::SHUTDOWN_REQUEST_COMPLETED {
            retry_pending_cleanup();
        }
        error_ticket.release();
        Ok(ShutdownSnapshot {
            raw_state: state,
            reason,
        })
    }

    /// Releases the observation handle; it does not cancel the shutdown request.
    pub fn close(&mut self) -> Result<(), UiError> {
        self.owner.try_close()
    }
}

fn dispose_unexpected_token(
    token: u64,
    ticket: TicketReservation,
    api: SharedApi,
    runtime_token: u64,
    anchor: Option<Arc<RuntimeAnchor>>,
) {
    if api.handle_release(token) == native_api::OK {
        ticket.release();
    } else {
        ticket.defer(crate::handle::PendingToken {
            token,
            runtime_token,
            api,
            anchor,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::{BindingDirection, BindingMode};
    use crate::handle::pending_cleanup_count;
    use crate::native_api::NativeApi;
    use crate::property::{Property, PropertyValue};
    use std::collections::HashMap;
    use std::sync::{Mutex, MutexGuard};
    use std::{cell::Cell, rc::Rc};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn serialize_registry_test() -> MutexGuard<'static, ()> {
        TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    unsafe extern "C" fn noop_wake(_userdata: *mut std::ffi::c_void, _priority: u32) {}

    unsafe extern "C" fn noop_bridge_destroy(_userdata: *mut std::ffi::c_void) {}

    fn dispatch_property_event(runtime: u64, action: impl FnOnce()) {
        let mut scope = 0;
        // SAFETY: scope is writable and runtime is a live token on this UI thread.
        let begin_status =
            unsafe { freemvvm_sys::ui_abi_1_runtime_host_begin_event(runtime, &mut scope) };
        assert_eq!(begin_status, native_api::OK);
        action();
        // SAFETY: scope was returned by begin_event and is ended once on the same UI thread.
        let end_status = unsafe { freemvvm_sys::ui_abi_1_runtime_host_end_event(runtime, scope) };
        assert_eq!(end_status, native_api::OK);
        // SAFETY: the no-op wake callback has returned; this call models the next host turn.
        let dispatch_status = unsafe { freemvvm_sys::ui_abi_1_runtime_dispatch_turn(runtime) };
        assert_eq!(dispatch_status, native_api::OK);
    }

    fn exercise_binding_modes<T>(
        ui: &UiContext<'_>,
        runtime: u64,
        initial: T,
        target_initial: T,
        next: T,
        reverse: T,
        equal: impl Fn(&T, &T) -> bool,
    ) where
        T: PropertyValue + Clone + std::fmt::Debug,
    {
        let mut source = Property::try_new(ui, initial.clone()).unwrap();
        let mut target = Property::try_new(ui, target_initial.clone()).unwrap();
        let once = source.bind_to(ui, &target, BindingMode::OneTime).unwrap();
        assert!(once.is_none(), "OneTime must not create a persistent owner");
        assert!(equal(&target.get(ui).unwrap(), &initial));
        dispatch_property_event(runtime, || source.set(ui, next.clone()).unwrap());
        assert!(equal(&target.get(ui).unwrap(), &initial));
        source.close().unwrap();
        target.close().unwrap();

        let mut source = Property::try_new(ui, initial.clone()).unwrap();
        let mut target = Property::try_new(ui, target_initial.clone()).unwrap();
        let mut one_way = source
            .bind_to(ui, &target, BindingMode::OneWay)
            .unwrap()
            .expect("OneWay creates an owned Binding");
        assert!(equal(&target.get(ui).unwrap(), &initial));
        dispatch_property_event(runtime, || target.set(ui, reverse.clone()).unwrap());
        assert!(equal(&source.get(ui).unwrap(), &initial));
        dispatch_property_event(runtime, || source.set(ui, next.clone()).unwrap());
        assert!(equal(&target.get(ui).unwrap(), &next));
        one_way.close().unwrap();
        dispatch_property_event(runtime, || source.set(ui, reverse.clone()).unwrap());
        assert!(equal(&target.get(ui).unwrap(), &reverse));
        source.close().unwrap();
        target.close().unwrap();

        let mut source = Property::try_new(ui, initial.clone()).unwrap();
        let mut target = Property::try_new(ui, target_initial).unwrap();
        let mut two_way = source
            .bind_to(ui, &target, BindingMode::TwoWay)
            .unwrap()
            .expect("TwoWay creates an owned Binding");
        assert!(equal(&target.get(ui).unwrap(), &initial));
        dispatch_property_event(runtime, || source.set(ui, next.clone()).unwrap());
        assert!(equal(&target.get(ui).unwrap(), &next));
        dispatch_property_event(runtime, || target.set(ui, reverse.clone()).unwrap());
        assert!(equal(&source.get(ui).unwrap(), &reverse));
        two_way.cancel(ui).unwrap();
        dispatch_property_event(runtime, || target.set(ui, next.clone()).unwrap());
        assert!(equal(&source.get(ui).unwrap(), &reverse));
        two_way.close().unwrap();
        source.close().unwrap();
        target.close().unwrap();
    }

    unsafe extern "C" fn reject_negative_i64(
        _userdata: *mut std::ffi::c_void,
        candidate: i64,
        _out_error: *mut u64,
    ) -> u32 {
        if candidate < 0 { 11 } else { native_api::OK }
    }

    unsafe extern "C" fn noop_validator_destroy(_userdata: *mut std::ffi::c_void) {}

    #[derive(Default)]
    struct FakeState {
        next_handle: u64,
        ui_thread: bool,
        create_status: u32,
        create_error: u64,
        shutdown_status: u32,
        shutdown_error: u64,
        request_status: u32,
        request_error: u64,
        query_status: u32,
        query_state: u32,
        query_reason: u32,
        query_error: u64,
        error_status: u32,
        error_domain: u32,
        error_code: u32,
        error_message: Vec<u8>,
        release_failures: HashMap<u64, usize>,
        release_calls: HashMap<u64, usize>,
    }

    #[derive(Default)]
    struct FakeApi {
        state: Mutex<FakeState>,
    }

    impl FakeApi {
        fn set_ui_thread(&self, value: bool) {
            self.state.lock().unwrap().ui_thread = value;
        }

        fn set_release_failure(&self, token: u64, count: usize) {
            self.state
                .lock()
                .unwrap()
                .release_failures
                .insert(token, count);
        }

        fn release_count(&self, token: u64) -> usize {
            *self
                .state
                .lock()
                .unwrap()
                .release_calls
                .get(&token)
                .unwrap_or(&0)
        }

        fn last_runtime(&self) -> u64 {
            self.state.lock().unwrap().next_handle
        }
    }

    impl NativeApi for FakeApi {
        fn get_version(&self) -> (u32, u32, u32, u32) {
            (native_api::OK, 1, 0, 0)
        }

        fn runtime_create(&self) -> (u32, u64, u64) {
            let mut state = self.state.lock().unwrap();
            if state.create_status != native_api::OK {
                return (state.create_status, 0, state.create_error);
            }
            state.next_handle += 1;
            (native_api::OK, state.next_handle, 0)
        }

        fn runtime_is_ui_thread(&self, _runtime: u64) -> (u32, u8) {
            let state = self.state.lock().unwrap();
            (native_api::OK, u8::from(state.ui_thread))
        }

        fn runtime_shutdown(&self, _runtime: u64) -> (u32, u64) {
            let state = self.state.lock().unwrap();
            (state.shutdown_status, state.shutdown_error)
        }

        fn runtime_request_shutdown(&self, _runtime: u64, _mode: u32) -> (u32, u64, u64) {
            let mut state = self.state.lock().unwrap();
            if state.request_status != native_api::OK && state.request_status != native_api::PENDING
            {
                return (state.request_status, 0, state.request_error);
            }
            state.next_handle += 1;
            (state.request_status, state.next_handle, state.request_error)
        }

        fn shutdown_request_query(&self, _request: u64) -> (u32, u32, u32, u64) {
            let state = self.state.lock().unwrap();
            (
                state.query_status,
                state.query_state,
                state.query_reason,
                state.query_error,
            )
        }

        fn handle_retain(&self, _borrowed: u64) -> (u32, u64) {
            let mut state = self.state.lock().unwrap();
            state.next_handle += 1;
            (native_api::OK, state.next_handle)
        }

        fn handle_release(&self, owned: u64) -> u32 {
            let mut state = self.state.lock().unwrap();
            *state.release_calls.entry(owned).or_default() += 1;
            let remaining = state.release_failures.entry(owned).or_default();
            if *remaining > 0 {
                *remaining -= 1;
                native_api::INTERNAL_ERROR
            } else {
                native_api::OK
            }
        }

        fn error_status(&self, _error: u64) -> (u32, u32) {
            let state = self.state.lock().unwrap();
            (native_api::OK, state.error_status)
        }

        fn error_code(&self, _error: u64) -> (u32, u32, u32) {
            let state = self.state.lock().unwrap();
            (native_api::OK, state.error_domain, state.error_code)
        }

        fn error_message_utf8(&self, _error: u64, buffer: Option<&mut [u8]>) -> (u32, u64) {
            let state = self.state.lock().unwrap();
            let required = state.error_message.len() as u64;
            match buffer {
                None => (native_api::OK, required),
                Some(buffer) if buffer.len() < state.error_message.len() => {
                    (native_api::BUFFER_TOO_SMALL, required)
                }
                Some(buffer) => {
                    buffer[..state.error_message.len()].copy_from_slice(&state.error_message);
                    (native_api::OK, required)
                }
            }
        }
    }

    #[test]
    fn create_context_and_explicit_release_retry_preserve_owner() {
        let _guard = serialize_registry_test();
        let api = Arc::new(FakeApi::default());
        api.set_ui_thread(false);
        let mut runtime = UiRuntime::try_create_with_api(api.clone()).unwrap();
        let token = api.last_runtime();
        let wrong_thread = match runtime.ui_context() {
            Err(error) => error,
            Ok(_) => panic!("wrong-thread context creation unexpectedly succeeded"),
        };
        assert_eq!(wrong_thread.raw_status(), native_api::WRONG_THREAD);

        api.set_ui_thread(true);
        let mut cloned_runtime = runtime.try_clone().unwrap();
        {
            let context = runtime.ui_context().unwrap();
            context.check_runtime(&runtime).unwrap();
            context.check_runtime(&cloned_runtime).unwrap();
        }
        cloned_runtime.close().unwrap();

        api.set_release_failure(token, 1);
        assert_eq!(
            runtime.close().unwrap_err().raw_status(),
            native_api::INTERNAL_ERROR
        );
        assert!(runtime.is_ui_thread().unwrap());
        runtime.close().unwrap();
        let closed_borrow = match runtime.owner.borrow() {
            Err(error) => error,
            Ok(_) => panic!("closed Runtime returned a borrowed handle"),
        };
        assert_eq!(closed_borrow.raw_status(), native_api::CLOSED);
        assert_eq!(api.release_count(token), 2);
    }

    #[test]
    fn drop_release_failure_is_retried_at_next_ui_context() {
        let _guard = serialize_registry_test();
        let api = Arc::new(FakeApi::default());
        api.set_ui_thread(true);
        let runtime = UiRuntime::try_create_with_api(api.clone()).unwrap();
        let token = api.last_runtime();
        api.set_release_failure(token, 1);
        drop(runtime);
        assert_eq!(pending_cleanup_count(), 1);

        let runtime = UiRuntime::try_create_with_api(api.clone()).unwrap();
        runtime.ui_context().unwrap();
        assert_eq!(pending_cleanup_count(), 0);
        assert_eq!(api.release_count(token), 2);
    }

    #[test]
    fn shutdown_receipt_is_thread_independent_and_anchors_runtime() {
        let _guard = serialize_registry_test();
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ShutdownRequest>();

        let api = Arc::new(FakeApi::default());
        api.set_ui_thread(true);
        {
            let mut state = api.state.lock().unwrap();
            state.request_status = native_api::PENDING;
            state.query_status = native_api::OK;
            state.query_state = freemvvm_sys::SHUTDOWN_REQUEST_HOST_UNAVAILABLE;
            state.query_reason = 0xAABB_CCDD;
        }
        let mut runtime = UiRuntime::try_create_with_api(api.clone()).unwrap();
        let request = {
            let context = runtime.ui_context().unwrap();
            runtime
                .request_shutdown(&context, ShutdownMode::Async)
                .unwrap()
        };
        runtime.close().unwrap();

        let snapshot = std::thread::spawn(move || {
            let snapshot = request.query().unwrap();
            (snapshot, request)
        })
        .join()
        .unwrap();
        assert_eq!(
            snapshot.0.known_state(),
            Some(ShutdownRequestState::HostUnavailable)
        );
        assert_eq!(snapshot.0.reason, 0xAABB_CCDD);
        drop(snapshot.1);
    }

    #[test]
    fn create_failure_preserves_unknown_status_and_error_details() {
        let _guard = serialize_registry_test();
        let api = Arc::new(FakeApi::default());
        {
            let mut state = api.state.lock().unwrap();
            state.create_status = 0xF000_0011;
            state.create_error = 901;
            state.error_status = 0xF000_0011;
            state.error_domain = 0xAABB_CCDD;
            state.error_code = 0x1020_3040;
            state.error_message = b"runtime unavailable".to_vec();
        }

        let error = match UiRuntime::try_create_with_api(api.clone()) {
            Err(error) => error,
            Ok(_) => panic!("failed Runtime creation unexpectedly succeeded"),
        };
        assert_eq!(error.raw_status(), 0xF000_0011);
        assert_eq!(error.domain(), Some(0xAABB_CCDD));
        assert_eq!(error.code(), Some(0x1020_3040));
        assert_eq!(error.message(), Some("runtime unavailable"));
        assert_eq!(api.release_count(901), 1);
    }

    #[test]
    fn property_subscription_separates_close_cancel_and_contains_callback_panic() {
        let _guard = serialize_registry_test();
        let mut runtime = UiRuntime::try_create().expect("native Runtime creation succeeds");
        let runtime_token = runtime
            .owner
            .borrow()
            .expect("Runtime owner is live")
            .token();
        // SAFETY: no-op bridge callbacks are valid for the Runtime lifetime and need no userdata.
        let bridge_status = unsafe {
            freemvvm_sys::ui_abi_1_runtime_set_ui_loop_bridge(
                runtime_token,
                Some(noop_wake),
                std::ptr::null_mut(),
                Some(noop_bridge_destroy),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(bridge_status, native_api::OK);

        {
            let ui = runtime
                .ui_context()
                .expect("Runtime UI context is available");
            let mut property = Property::try_new(&ui, 0_i64).expect("Property creation succeeds");
            let calls = Rc::new(Cell::new(0));
            let callback_calls = Rc::clone(&calls);
            let mut subscription = property
                .subscribe(&ui, move || callback_calls.set(callback_calls.get() + 1))
                .expect("subscription registration succeeds");
            assert_eq!(
                calls.get(),
                0,
                "subscribe does not send an initial notification"
            );

            dispatch_property_event(runtime_token, || property.set(&ui, 1).unwrap());
            assert_eq!(calls.get(), 1);
            dispatch_property_event(runtime_token, || property.set(&ui, 1).unwrap());
            assert_eq!(calls.get(), 1, "an equal set does not notify");

            subscription
                .close()
                .expect("close releases the public handle");
            dispatch_property_event(runtime_token, || property.set(&ui, 2).unwrap());
            assert_eq!(calls.get(), 2, "handle release alone does not cancel");

            let mut cancellable = property
                .subscribe(&ui, || {})
                .expect("second subscription registration succeeds");
            cancellable
                .cancel(&ui)
                .expect("explicit cancellation succeeds");
            dispatch_property_event(runtime_token, || property.set(&ui, 3).unwrap());
            cancellable.close().expect("inactive handle is releasable");

            let mut panicking = property
                .subscribe(&ui, || panic!("notification panic is contained"))
                .expect("panicking subscriber registration succeeds");
            let continued_calls = Rc::new(Cell::new(0));
            let callback_calls = Rc::clone(&continued_calls);
            let mut continued = property
                .subscribe(&ui, move || callback_calls.set(callback_calls.get() + 1))
                .expect("later subscriber registration succeeds");
            dispatch_property_event(runtime_token, || property.set(&ui, 4).unwrap());
            assert_eq!(
                continued_calls.get(),
                1,
                "a callback panic does not stop later subscribers"
            );
            panicking
                .cancel(&ui)
                .expect("panicking subscription can be canceled");
            continued
                .cancel(&ui)
                .expect("later subscription can be canceled");
            panicking
                .close()
                .expect("panicking subscription handle is releasable");
            continued
                .close()
                .expect("later subscription handle is releasable");

            property.close().expect("Property teardown succeeds");
        }
        runtime.close().expect("Runtime handle release succeeds");
    }

    #[test]
    fn property_binding_modes_work_for_all_supported_types() {
        let _guard = serialize_registry_test();
        let mut runtime = UiRuntime::try_create().expect("native Runtime creation succeeds");
        let runtime_token = runtime.owner.borrow().unwrap().token();
        // SAFETY: no-op bridge callbacks are valid for the Runtime lifetime.
        let bridge_status = unsafe {
            freemvvm_sys::ui_abi_1_runtime_set_ui_loop_bridge(
                runtime_token,
                Some(noop_wake),
                std::ptr::null_mut(),
                Some(noop_bridge_destroy),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(bridge_status, native_api::OK);

        {
            let ui = runtime.ui_context().unwrap();
            exercise_binding_modes(&ui, runtime_token, true, false, false, true, |a, b| a == b);
            exercise_binding_modes(&ui, runtime_token, 10_i64, 0_i64, 20_i64, 30_i64, |a, b| {
                a == b
            });
            exercise_binding_modes(
                &ui,
                runtime_token,
                String::from("あ\0source"),
                String::from("target"),
                String::from("next 🚀"),
                String::from("reverse"),
                |a, b| a == b,
            );
            exercise_binding_modes(
                &ui,
                runtime_token,
                1.25_f64,
                0.0_f64,
                -2.5_f64,
                4.0_f64,
                |a, b| a == b,
            );
            let initial = crate::decimal::Decimal64::try_from_parts(false, 1, 42).unwrap();
            let target_initial = crate::decimal::Decimal64::try_from_parts(false, 0, 7).unwrap();
            let next = crate::decimal::Decimal64::try_from_parts(true, 2, 314).unwrap();
            let reverse = crate::decimal::Decimal64::try_from_parts(false, 3, 2718).unwrap();
            exercise_binding_modes(
                &ui,
                runtime_token,
                initial,
                target_initial,
                next,
                reverse,
                |a, b| a.bits() == b.bits(),
            );
        }

        runtime.close().expect("Runtime handle release succeeds");
    }

    #[test]
    fn binding_error_snapshot_observes_rejection_and_recovery() {
        let _guard = serialize_registry_test();
        let mut runtime = UiRuntime::try_create().expect("native Runtime creation succeeds");
        let runtime_token = runtime.owner.borrow().unwrap().token();
        // SAFETY: no-op bridge callbacks are valid for the Runtime lifetime.
        let bridge_status = unsafe {
            freemvvm_sys::ui_abi_1_runtime_set_ui_loop_bridge(
                runtime_token,
                Some(noop_wake),
                std::ptr::null_mut(),
                Some(noop_bridge_destroy),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(bridge_status, native_api::OK);

        {
            let ui = runtime.ui_context().unwrap();
            let mut target = Property::try_new(&ui, 0_i64).unwrap();
            let mut validator = 0;
            // SAFETY: callback has the ABI signature and no userdata; registration is canceled below.
            let status = unsafe {
                freemvvm_sys::ui_abi_1_property_validator_register_i64(
                    target.token_for_call().unwrap(),
                    Some(reject_negative_i64),
                    std::ptr::null_mut(),
                    Some(noop_validator_destroy),
                    &mut validator,
                    std::ptr::null_mut(),
                )
            };
            assert_eq!(status, native_api::OK);

            let mut rejected_source = Property::try_new(&ui, -2_i64).unwrap();
            let initial_error = match rejected_source.bind_to(&ui, &target, BindingMode::OneWay) {
                Err(error) => error,
                Ok(_) => panic!("validator-rejected initial Binding unexpectedly succeeded"),
            };
            assert_eq!(initial_error.raw_status(), 11);
            assert_eq!(target.get(&ui).unwrap(), 0);
            rejected_source.close().unwrap();

            let mut source = Property::try_new(&ui, 4_i64).unwrap();

            let mut binding = source
                .bind_to(&ui, &target, BindingMode::OneWay)
                .unwrap()
                .expect("OneWay creates a persistent Binding");
            let error_notifications = Rc::new(Cell::new(0));
            let notification_counter = Rc::clone(&error_notifications);
            let mut subscription = binding
                .subscribe_error(&ui, move || {
                    notification_counter.set(notification_counter.get() + 1)
                })
                .unwrap();
            assert!(binding.error_snapshot(&ui).unwrap().is_none());

            dispatch_property_event(runtime_token, || source.set(&ui, -1).unwrap());
            assert_eq!(source.get(&ui).unwrap(), -1);
            assert_eq!(target.get(&ui).unwrap(), 4);
            let snapshot = binding
                .error_snapshot(&ui)
                .unwrap()
                .expect("rejected propagation produces an error snapshot");
            assert_eq!(snapshot.error().raw_status(), 11);
            assert_eq!(snapshot.direction(), BindingDirection::SourceToTarget);
            assert_eq!(snapshot.rejected_property().get(&ui).unwrap(), 4);

            dispatch_property_event(runtime_token, || source.set(&ui, 8).unwrap());
            assert_eq!(target.get(&ui).unwrap(), 8);
            assert!(binding.error_snapshot(&ui).unwrap().is_none());
            assert_eq!(snapshot.error().raw_status(), 11);
            assert_eq!(snapshot.rejected_property().get(&ui).unwrap(), 8);
            assert_eq!(error_notifications.get(), 2);

            subscription.cancel(&ui).unwrap();
            subscription.close().unwrap();
            binding.cancel(&ui).unwrap();
            binding.close().unwrap();
            // SAFETY: registration is live and the optional error output may be null.
            let cancel_status = unsafe {
                freemvvm_sys::ui_abi_1_property_validator_cancel(validator, std::ptr::null_mut())
            };
            assert_eq!(cancel_status, native_api::OK);
            source.close().unwrap();
            target.close().unwrap();
        }

        runtime.close().expect("Runtime handle release succeeds");
    }

    #[test]
    fn binding_rejects_properties_from_different_runtimes_before_abi_call() {
        let _guard = serialize_registry_test();
        let mut source_runtime = UiRuntime::try_create().unwrap();
        let mut target_runtime = UiRuntime::try_create().unwrap();
        {
            let source_ui = source_runtime.ui_context().unwrap();
            let target_ui = target_runtime.ui_context().unwrap();
            let mut source = Property::try_new(&source_ui, 1_i64).unwrap();
            let mut target = Property::try_new(&target_ui, 0_i64).unwrap();
            let error = match source.bind_to(&source_ui, &target, BindingMode::OneWay) {
                Err(error) => error,
                Ok(_) => panic!("cross-Runtime Binding unexpectedly succeeded"),
            };
            assert_eq!(error.raw_status(), native_api::WRONG_RUNTIME);
            source.close().unwrap();
            target.close().unwrap();
        }
        target_runtime.close().unwrap();
        source_runtime.close().unwrap();
    }
}

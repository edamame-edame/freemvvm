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
    use crate::command::{AsyncCommand, CommandOperation, CommandState};
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

    fn install_noop_bridge(runtime: &mut UiRuntime) -> u64 {
        let token = runtime.owner.borrow().unwrap().token();
        // SAFETY: callbacks are valid for this Runtime's lifetime and require no userdata.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_runtime_set_ui_loop_bridge(
                token,
                Some(noop_wake),
                std::ptr::null_mut(),
                Some(noop_bridge_destroy),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(status, native_api::OK);
        token
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
        ui_child_count: u64,
        ui_orientation: u32,
        close_subscribe_status: u32,
        close_subscribe_error: u64,
        close_subscription_token: u64,
        close_callback: Option<freemvvm_sys::WindowCloseCallback>,
        close_userdata: usize,
        close_destroy: Option<freemvvm_sys::DestroyCallback>,
        close_destroy_count: usize,
        close_defer_status: u32,
        close_defer_error: u64,
        close_resolve_status: u32,
        close_resolve_error: u64,
        close_resolutions: Vec<(u64, u32)>,
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

        fn invoke_close_request(&self, request: u64) {
            let (callback, userdata) = {
                let state = self.state.lock().unwrap();
                (state.close_callback, state.close_userdata)
            };
            if let Some(callback) = callback {
                // SAFETY: the fake retains callback userdata until it runs the destroy callback.
                unsafe { callback(userdata as *mut std::ffi::c_void, request) };
            }
        }

        fn close_destroy_count(&self) -> usize {
            self.state.lock().unwrap().close_destroy_count
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

        fn ui_create(
            &self,
            _kind: crate::native_api::UiControlKind,
            _runtime: u64,
            _bytes: &[u8],
        ) -> (u32, u64, u64) {
            let mut state = self.state.lock().unwrap();
            state.next_handle += 1;
            (native_api::OK, state.next_handle, 0)
        }

        fn ui_property(
            &self,
            _kind: crate::native_api::UiPropertyKind,
            _control: u64,
        ) -> (u32, u64) {
            let mut state = self.state.lock().unwrap();
            state.next_handle += 1;
            (native_api::OK, state.next_handle)
        }

        fn ui_tree(
            &self,
            action: crate::native_api::UiTreeAction,
            _parent: u64,
            _child: u64,
            _index: u64,
            _to_index: u64,
        ) -> (u32, u64, u64) {
            let mut state = self.state.lock().unwrap();
            match action {
                crate::native_api::UiTreeAction::Insert => {
                    state.ui_child_count += 1;
                    (native_api::OK, 0, 0)
                }
                crate::native_api::UiTreeAction::Remove => {
                    state.ui_child_count = state.ui_child_count.saturating_sub(1);
                    (native_api::OK, 0, 0)
                }
                crate::native_api::UiTreeAction::ChildCount => {
                    (native_api::OK, state.ui_child_count, 0)
                }
                crate::native_api::UiTreeAction::WindowContent
                | crate::native_api::UiTreeAction::ChildAt => (native_api::OK, 0, 0),
                _ => (native_api::OK, 0, 0),
            }
        }

        fn ui_stack_panel(
            &self,
            action: crate::native_api::UiStackPanelAction,
            _panel: u64,
            _child: u64,
            value: u32,
        ) -> (u32, u64, u64) {
            let mut state = self.state.lock().unwrap();
            match action {
                crate::native_api::UiStackPanelAction::Orientation => {
                    (native_api::OK, u64::from(state.ui_orientation), 0)
                }
                crate::native_api::UiStackPanelAction::SetOrientation => {
                    state.ui_orientation = value;
                    (native_api::OK, 0, 0)
                }
                crate::native_api::UiStackPanelAction::SetChildAlign => (native_api::OK, 0, 0),
            }
        }

        fn ui_button(
            &self,
            _action: crate::native_api::UiButtonAction,
            _button: u64,
            _command: u64,
        ) -> (u32, u64, u64) {
            (native_api::OK, 0, 0)
        }

        fn close_request_subscribe(
            &self,
            _window: u64,
            callback: Option<freemvvm_sys::WindowCloseCallback>,
            userdata: *mut std::ffi::c_void,
            destroy: Option<freemvvm_sys::DestroyCallback>,
        ) -> (u32, u64, u64) {
            let mut state = self.state.lock().unwrap();
            if state.close_subscribe_status != native_api::OK {
                return (state.close_subscribe_status, 0, state.close_subscribe_error);
            }
            state.next_handle += 1;
            let token = state.next_handle;
            state.close_subscription_token = token;
            state.close_callback = callback;
            state.close_userdata = userdata as usize;
            state.close_destroy = destroy;
            (native_api::OK, token, 0)
        }

        fn close_request_defer(&self, _borrowed: u64) -> (u32, u64, u64) {
            let mut state = self.state.lock().unwrap();
            if state.close_defer_status != native_api::OK {
                return (state.close_defer_status, 0, state.close_defer_error);
            }
            state.next_handle += 1;
            (native_api::OK, state.next_handle, 0)
        }

        fn close_request_resolve(&self, request: u64, decision: u32) -> (u32, u64) {
            let mut state = self.state.lock().unwrap();
            state.close_resolutions.push((request, decision));
            (state.close_resolve_status, state.close_resolve_error)
        }

        fn subscription_cancel(&self, token: u64) -> (u32, u64) {
            let (destroy, userdata) = {
                let mut state = self.state.lock().unwrap();
                if token != state.close_subscription_token || token == 0 {
                    drop(state);
                    let mut error = 0;
                    // SAFETY: fake API falls back to native cancellation for real subscriptions.
                    let status =
                        unsafe { freemvvm_sys::ui_abi_1_subscription_cancel(token, &mut error) };
                    return (status, error);
                }
                state.close_destroy_count += 1;
                let userdata = state.close_userdata;
                state.close_callback = None;
                state.close_userdata = 0;
                state.close_subscription_token = 0;
                (state.close_destroy, userdata)
            };
            if let Some(destroy) = destroy {
                // SAFETY: successful fake cancellation ends the registered userdata lifetime.
                unsafe { destroy(userdata as *mut std::ffi::c_void) };
            }
            (native_api::OK, 0)
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

    #[test]
    fn async_command_reports_worker_completion_through_ui_post() {
        let _guard = serialize_registry_test();
        fn assert_send<T: Send>() {}
        assert_send::<CommandOperation>();

        let mut runtime = UiRuntime::try_create().unwrap();
        let runtime_token = install_noop_bridge(&mut runtime);
        {
            let ui = runtime.ui_context().unwrap();
            let (sender, receiver) = std::sync::mpsc::channel();
            let mut command = AsyncCommand::create(&ui, None, move |operation| {
                sender.send(operation).unwrap();
            })
            .unwrap();
            let mut state = command.state_property(&ui).unwrap();
            let mut can_execute = command.can_execute_property(&ui).unwrap();

            let operation_id = command.execute(&ui).unwrap();
            assert_eq!(state.get(&ui).unwrap(), freemvvm_sys::COMMAND_STATE_RUNNING);
            assert!(!can_execute.get(&ui).unwrap());
            let operation = receiver.recv().unwrap();
            assert_eq!(operation.id(), operation_id);

            let (first_report, duplicate_status) = std::thread::spawn(move || {
                let first = operation
                    .report_success()
                    .map_err(|error| error.raw_status());
                let duplicate = operation
                    .report_success()
                    .expect_err("one operation accepts only one completion report")
                    .raw_status();
                (first, duplicate)
            })
            .join()
            .unwrap();
            assert_eq!(first_report, Ok(()));
            assert_eq!(duplicate_status, native_api::INVALID_ARGUMENT);

            dispatch_property_event(runtime_token, || {});
            assert_eq!(
                CommandState::from_raw(state.get(&ui).unwrap()),
                CommandState::Succeeded
            );
            assert!(can_execute.get(&ui).unwrap());
            command.close(&ui).unwrap();
            state.close().unwrap();
            can_execute.close().unwrap();
        }
        runtime.close().unwrap();
    }

    #[test]
    fn async_command_cooperative_cancel_and_failure_snapshot_follow_worker_result() {
        let _guard = serialize_registry_test();
        let mut runtime = UiRuntime::try_create().unwrap();
        let runtime_token = install_noop_bridge(&mut runtime);
        {
            let ui = runtime.ui_context().unwrap();
            let (sender, receiver) = std::sync::mpsc::channel();
            let mut command = AsyncCommand::create(&ui, None, move |operation| {
                sender.send(operation).unwrap();
            })
            .unwrap();
            let mut state = command.state_property(&ui).unwrap();
            let mut can_execute = command.can_execute_property(&ui).unwrap();

            let cancelled_id = command.execute(&ui).unwrap();
            let cancelled = receiver.recv().unwrap();
            assert_eq!(cancelled.id(), cancelled_id);
            command.cancel(&ui, cancelled_id).unwrap();
            assert_eq!(
                state.get(&ui).unwrap(),
                freemvvm_sys::COMMAND_STATE_CANCELLING
            );
            let (requested, report) = std::thread::spawn(move || {
                let requested = cancelled.cancellation().is_cancel_requested().unwrap();
                (requested, cancelled.report_cancelled())
            })
            .join()
            .unwrap();
            assert!(requested);
            report.unwrap();
            dispatch_property_event(runtime_token, || {});
            assert_eq!(
                state.get(&ui).unwrap(),
                freemvvm_sys::COMMAND_STATE_CANCELLED
            );

            let failed_id = command.execute(&ui).unwrap();
            let failed = receiver.recv().unwrap();
            assert_ne!(failed.id(), cancelled_id);
            let report = std::thread::spawn(move || {
                failed.report_failure(0xAABB_CCDD, 0x1020_3040, "worker failed あ🚀")
            })
            .join()
            .unwrap();
            report.unwrap();
            dispatch_property_event(runtime_token, || {});
            assert_eq!(state.get(&ui).unwrap(), freemvvm_sys::COMMAND_STATE_FAILED);
            assert!(can_execute.get(&ui).unwrap());
            let failure = command.failure_snapshot(&ui).unwrap().unwrap();
            assert_eq!(failure.raw_status(), freemvvm_sys::COMMAND_EXECUTION_FAILED);
            assert_eq!(failure.domain(), Some(0xAABB_CCDD));
            assert_eq!(failure.code(), Some(0x1020_3040));
            assert_eq!(failure.message(), Some("worker failed あ🚀"));

            let next_id = command.execute(&ui).unwrap();
            assert_ne!(next_id, failed_id);
            assert!(command.failure_snapshot(&ui).unwrap().is_none());
            let next = receiver.recv().unwrap();
            next.report_success().unwrap();
            dispatch_property_event(runtime_token, || {});
            assert_eq!(
                state.get(&ui).unwrap(),
                freemvvm_sys::COMMAND_STATE_SUCCEEDED
            );
            assert!(can_execute.get(&ui).unwrap());

            command.close(&ui).unwrap();
            state.close().unwrap();
            can_execute.close().unwrap();
        }
        runtime.close().unwrap();
    }

    #[test]
    fn async_command_close_does_not_wait_for_worker_or_apply_late_report() {
        let _guard = serialize_registry_test();
        let mut runtime = UiRuntime::try_create().unwrap();
        let runtime_token = install_noop_bridge(&mut runtime);
        {
            let ui = runtime.ui_context().unwrap();
            let (sender, receiver) = std::sync::mpsc::channel();
            let mut command = AsyncCommand::create(&ui, None, move |operation| {
                sender.send(operation).unwrap();
            })
            .unwrap();
            let mut state = command.state_property(&ui).unwrap();
            command.execute(&ui).unwrap();
            let operation = receiver.recv().unwrap();
            command.close(&ui).unwrap();
            let report = std::thread::spawn(move || operation.report_success())
                .join()
                .unwrap();
            report.unwrap();
            dispatch_property_event(runtime_token, || {});
            assert_eq!(state.get(&ui).unwrap(), freemvvm_sys::COMMAND_STATE_RUNNING);
            state.close().unwrap();
        }
        runtime.close().unwrap();
    }

    #[test]
    fn async_command_contains_start_callback_panic_as_operation_failure() {
        let _guard = serialize_registry_test();
        let mut runtime = UiRuntime::try_create().unwrap();
        install_noop_bridge(&mut runtime);
        {
            let ui = runtime.ui_context().unwrap();
            let mut command = AsyncCommand::create(&ui, None, |_operation| {
                panic!("start callback panic stays inside the Rust wrapper");
            })
            .unwrap();
            let mut state = command.state_property(&ui).unwrap();
            let operation = command.execute(&ui).unwrap();
            assert_ne!(operation.get(), 0);
            assert_eq!(state.get(&ui).unwrap(), freemvvm_sys::COMMAND_STATE_FAILED);
            assert_eq!(
                command.failure_snapshot(&ui).unwrap().unwrap().raw_status(),
                freemvvm_sys::COMMAND_EXECUTION_FAILED
            );
            command.close(&ui).unwrap();
            state.close().unwrap();
        }
        runtime.close().unwrap();
    }

    #[test]
    fn async_command_ineligible_execute_does_not_start_or_change_state() {
        let _guard = serialize_registry_test();
        let mut runtime = UiRuntime::try_create().unwrap();
        let runtime_token = install_noop_bridge(&mut runtime);
        {
            let ui = runtime.ui_context().unwrap();
            let mut eligibility = Property::try_new(&ui, false).unwrap();
            let starts = Rc::new(Cell::new(0));
            let callback_starts = Rc::clone(&starts);
            let mut command = AsyncCommand::create(&ui, Some(&eligibility), move |_operation| {
                callback_starts.set(callback_starts.get() + 1);
            })
            .unwrap();
            let mut state = command.state_property(&ui).unwrap();
            let mut can_execute = command.can_execute_property(&ui).unwrap();
            assert!(!can_execute.get(&ui).unwrap());
            let error = match command.execute(&ui) {
                Err(error) => error,
                Ok(_) => panic!("ineligible Command unexpectedly issued an operation"),
            };
            assert_eq!(error.raw_status(), native_api::UNSUPPORTED);
            assert_eq!(starts.get(), 0);
            assert_eq!(state.get(&ui).unwrap(), freemvvm_sys::COMMAND_STATE_IDLE);

            dispatch_property_event(runtime_token, || eligibility.set(&ui, true).unwrap());
            assert!(can_execute.get(&ui).unwrap());
            command.close(&ui).unwrap();
            state.close().unwrap();
            can_execute.close().unwrap();
            eligibility.close().unwrap();
        }
        runtime.close().unwrap();
    }

    #[test]
    fn ui_controls_preserve_runtime_identity_tree_and_layout_contracts() {
        let _guard = serialize_registry_test();
        let api = Arc::new(FakeApi::default());
        api.set_ui_thread(true);
        let mut runtime = UiRuntime::try_create_with_api(api).unwrap();
        let other_api = Arc::new(FakeApi::default());
        other_api.set_ui_thread(true);
        let mut other_runtime = UiRuntime::try_create_with_api(other_api).unwrap();
        {
            let ui = runtime.ui_context().unwrap();
            let other_ui = other_runtime.ui_context().unwrap();
            let mut window = crate::ui::Window::create(&ui).unwrap();
            let mut panel = crate::ui::StackPanel::create(&ui).unwrap();
            let mut text = crate::ui::Text::create(&ui, "hello").unwrap();
            let mut content = text.content_property(&ui).unwrap();
            assert_eq!(panel.child_count(&ui).unwrap(), 0);
            panel.insert(&ui, 0, &text).unwrap();
            assert_eq!(panel.child_count(&ui).unwrap(), 1);
            panel
                .set_orientation(&ui, crate::ui::Orientation::Horizontal)
                .unwrap();
            assert_eq!(
                panel.orientation(&ui).unwrap(),
                crate::ui::Orientation::Horizontal
            );
            window.set_content(&ui, &panel).unwrap();
            assert!(window.content(&ui).unwrap().is_none());
            assert_eq!(
                panel
                    .set_orientation(&other_ui, crate::ui::Orientation::Vertical)
                    .unwrap_err()
                    .raw_status(),
                native_api::WRONG_RUNTIME
            );
            assert_eq!(
                content
                    .set(&other_ui, String::from("foreign"))
                    .unwrap_err()
                    .raw_status(),
                native_api::WRONG_RUNTIME
            );
            content.close().unwrap();
            text.close().unwrap();
            panel.close().unwrap();
            window.close().unwrap();
        }
        other_runtime.close().unwrap();
        runtime.close().unwrap();
    }

    #[test]
    fn window_close_callback_is_owned_until_cancel_and_destroy_runs_once() {
        let _guard = serialize_registry_test();
        struct DropProbe(Rc<Cell<usize>>);
        impl Drop for DropProbe {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }

        let api = Arc::new(FakeApi::default());
        api.set_ui_thread(true);
        let mut runtime = UiRuntime::try_create_with_api(api.clone()).unwrap();
        {
            let ui = runtime.ui_context().unwrap();
            let mut window = crate::ui::Window::create(&ui).unwrap();
            let calls = Rc::new(Cell::new(0));
            let callback_drop_count = Rc::new(Cell::new(0));
            let probe = DropProbe(Rc::clone(&callback_drop_count));
            let callback_calls = Rc::clone(&calls);
            let mut subscription = window
                .on_close_request(&ui, move |_request| {
                    let _keep_probe_until_destroy = &probe;
                    callback_calls.set(callback_calls.get() + 1);
                })
                .unwrap();

            api.invoke_close_request(800);
            assert_eq!(calls.get(), 1);
            assert_eq!(callback_drop_count.get(), 0);
            subscription.cancel(&ui).unwrap();
            assert_eq!(api.close_destroy_count(), 1);
            assert_eq!(callback_drop_count.get(), 1);
            api.invoke_close_request(802);
            assert_eq!(calls.get(), 1, "cancel prevents later callback starts");
            subscription.close().unwrap();
            assert_eq!(api.close_destroy_count(), 1);
            window.close().unwrap();
        }
        runtime.close().unwrap();
    }

    #[test]
    fn failed_window_close_registration_keeps_callback_on_rust_side() {
        let _guard = serialize_registry_test();
        struct DropProbe(Rc<Cell<usize>>);
        impl Drop for DropProbe {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }

        let api = Arc::new(FakeApi::default());
        api.set_ui_thread(true);
        api.state.lock().unwrap().close_subscribe_status = native_api::UNSUPPORTED;
        let mut runtime = UiRuntime::try_create_with_api(api.clone()).unwrap();
        {
            let ui = runtime.ui_context().unwrap();
            let mut window = crate::ui::Window::create(&ui).unwrap();
            let dropped = Rc::new(Cell::new(0));
            let probe = DropProbe(Rc::clone(&dropped));
            let error = match window.on_close_request(&ui, move |_request| {
                let _keep_probe_in_callback = &probe;
            }) {
                Err(error) => error,
                Ok(_) => panic!("fake close registration unexpectedly succeeded"),
            };
            assert_eq!(error.raw_status(), native_api::UNSUPPORTED);
            assert_eq!(dropped.get(), 1);
            window.close().unwrap();
        }
        runtime.close().unwrap();
    }

    #[test]
    fn window_close_callback_panic_does_not_cross_the_abi_boundary() {
        let _guard = serialize_registry_test();
        let api = Arc::new(FakeApi::default());
        api.set_ui_thread(true);
        let mut runtime = UiRuntime::try_create_with_api(api.clone()).unwrap();
        {
            let ui = runtime.ui_context().unwrap();
            let mut window = crate::ui::Window::create(&ui).unwrap();
            let mut subscription = window
                .on_close_request(&ui, |_request| panic!("test callback panic"))
                .unwrap();
            api.invoke_close_request(801);
            subscription.cancel(&ui).unwrap();
            subscription.close().unwrap();
            window.close().unwrap();
        }
        runtime.close().unwrap();
    }

    #[test]
    fn close_request_view_defers_without_capturing_ui_context_and_resolves() {
        let _guard = serialize_registry_test();
        let api = Arc::new(FakeApi::default());
        api.set_ui_thread(true);
        let mut runtime = UiRuntime::try_create_with_api(api.clone()).unwrap();
        {
            let ui = runtime.ui_context().unwrap();
            let mut window = crate::ui::Window::create(&ui).unwrap();
            let deferred = Rc::new(std::cell::RefCell::new(None));
            let callback_deferred = Rc::clone(&deferred);
            let mut subscription = window
                .on_close_request(&ui, move |request| {
                    *callback_deferred.borrow_mut() = Some(request.defer().unwrap());
                })
                .unwrap();

            api.invoke_close_request(803);
            let mut request = deferred.borrow_mut().take().unwrap();
            request
                .resolve(&ui, crate::ui::CloseDecision::Accept)
                .unwrap();
            assert_eq!(api.state.lock().unwrap().close_resolutions.len(), 1);

            request.close().unwrap();
            subscription.cancel(&ui).unwrap();
            subscription.close().unwrap();
            window.close().unwrap();
        }
        runtime.close().unwrap();
    }

    #[test]
    fn close_request_view_defer_checks_thread_and_reports_abi_failure() {
        let _guard = serialize_registry_test();
        let api = Arc::new(FakeApi::default());
        api.set_ui_thread(true);
        let mut runtime = UiRuntime::try_create_with_api(api.clone()).unwrap();
        {
            let ui = runtime.ui_context().unwrap();
            let mut window = crate::ui::Window::create(&ui).unwrap();
            let defer_status = Rc::new(Cell::new(None));
            let callback_status = Rc::clone(&defer_status);
            let mut subscription = window
                .on_close_request(&ui, move |request| {
                    callback_status.set(Some(request.defer().err().unwrap().raw_status()));
                })
                .unwrap();

            api.set_ui_thread(false);
            api.invoke_close_request(804);
            assert_eq!(defer_status.get(), Some(native_api::WRONG_THREAD));

            api.set_ui_thread(true);
            api.state.lock().unwrap().close_defer_status = native_api::UNSUPPORTED;
            api.invoke_close_request(805);
            assert_eq!(defer_status.get(), Some(native_api::UNSUPPORTED));

            subscription.cancel(&ui).unwrap();
            subscription.close().unwrap();
            window.close().unwrap();
        }
        runtime.close().unwrap();
    }
}

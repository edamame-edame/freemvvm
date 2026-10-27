//! Internal Rust Core crate. Product implementation starts in subsequent units.
//!
//! Language wrappers must use the versioned C ABI rather than depend on this crate.

#[allow(dead_code)]
mod binding;
mod command;
mod dispatcher;
mod handle_registry;
// Runtime-owned UI state and the initial internal Property integration.
pub mod numeric;
#[allow(dead_code)]
mod object_store;
mod platform;
mod property;
pub use property::ValidationError;
mod runtime;
mod runtime_handles;
mod ui_controls;
mod ui_focus;
// Typed value semantics back the internal Property state; ABI validation remains a later unit.
mod ui_tree;
mod ui_window;
#[allow(dead_code)]
mod value;

/// Narrow support surface used by the C ABI crate while the Core remains private to wrappers.
#[doc(hidden)]
pub mod abi_support {
    use crate::command::{AsyncCommand, CancelToken, CommandError, CommandSnapshot};
    use crate::dispatcher::{PostErrorKind, PostedWork, UiLoopBridge, WakePriority};
    use crate::handle_registry::HandleToken;
    use crate::handle_registry::registry::{
        HandleRegistry, InternalOwner, ObjectIdentity, RegistryError,
    };
    use crate::object_store::UiObjectStoreError;
    pub use crate::platform::frame::{FrameError, FramePhase, FrameTicket, FrameTracker};
    pub use crate::platform::scene::{
        Damage, DeviceScale, DrawOp, GlyphPlacement, GlyphRunSnapshot, Rect, SceneError,
        SceneSnapshot,
    };
    use crate::property::{
        PropertyHandleKind, PropertyValidator, Subscription, SubscriptionCallback, ValidationError,
        ValidatorResult,
    };
    use crate::runtime::{
        RuntimeBindingError, RuntimeCore, RuntimeCoreError, RuntimeIdentity,
        RuntimeObjectRegistrationError, RuntimePropertyError, RuntimeTreeError,
        ShutdownCompletionState, ShutdownRequest, ShutdownRequestMode, ShutdownRequestOutcome,
    };
    use crate::runtime_handles::{RuntimeHandleError, process_runtime_handles};
    use crate::ui_controls::{
        UiControlKind, UiControlProperty, UiControlPropertyOwner, UiControlState,
    };
    use crate::ui_tree::TreeError;
    use crate::ui_window::WindowLifecycleError;
    pub use crate::ui_window::{
        CloseDecision, PlatformFrameCompletionSink, PlatformOperationError, PlatformRuntimeFactory,
        PlatformWindowPeer,
    };
    use crate::value::{
        DecimalValue, OwnedUtf8, OwnedUtf8Error, TypedValue, Utf8CopyOutcome, ValueKind,
    };
    use std::collections::HashMap;
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Condvar, Mutex, OnceLock, Weak};

    /// Failure categories needed by the versioned C ABI Runtime creation entrypoint.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum RuntimeCreateError {
        OutOfMemory,
        Internal,
    }

    /// Failure categories needed by the versioned C ABI Runtime query entrypoint.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum RuntimeQueryError {
        InvalidHandle,
        Internal,
    }

    /// Failure categories needed by the versioned C ABI handle retain entrypoint.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum RuntimeRetainError {
        InvalidHandle,
        OutOfMemory,
        Internal,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ErrorApiError {
        InvalidHandle,
        InvalidArgument,
        WrongThread,
        OutOfMemory,
        BufferTooSmall { required: usize },
        Internal,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ShutdownApiError {
        InvalidHandle,
        InvalidArgument,
        WrongThread,
        Closed,
        HostUnavailable { reason: u32 },
        OutOfMemory,
        Unsupported,
        Internal,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ShutdownApiState {
        Pending,
        Completed,
        HostUnavailable { reason: u32 },
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum PropertyApiError {
        InvalidHandle,
        InvalidArgument,
        WrongType,
        WrongThread,
        Closed,
        Unsupported,
        ValidationFailed(Option<Arc<ValidationError>>),
        OutOfMemory,
        BufferTooSmall { required: usize },
        Internal,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum BindingApiError {
        InvalidHandle,
        InvalidArgument,
        WrongType,
        WrongRuntime,
        WrongThread,
        Closed,
        Unsupported,
        OutOfMemory,
        ValidationFailed(Option<Arc<ValidationError>>),
        Internal,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum CommandApiError {
        InvalidHandle,
        InvalidArgument,
        WrongType,
        WrongRuntime,
        WrongThread,
        Closed,
        NotExecutable,
        InvalidOperation,
        OutOfMemory,
        Internal,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum UiControlApiError {
        InvalidHandle,
        InvalidArgument,
        WrongType,
        WrongRuntime,
        WrongThread,
        Closed,
        Unsupported,
        OutOfMemory,
        Internal,
    }

    pub type ValidateBoolCallback = unsafe extern "C" fn(*mut c_void, u8, *mut u64) -> u32;
    pub type ValidateI64Callback = unsafe extern "C" fn(*mut c_void, i64, *mut u64) -> u32;
    pub type ValidateUtf8Callback =
        unsafe extern "C" fn(*mut c_void, *const u8, u64, *mut u64) -> u32;

    type ImmutableError = ValidationError;

    static ERROR_HANDLES: OnceLock<Mutex<HashMap<HandleToken, Arc<ImmutableError>>>> =
        OnceLock::new();

    fn error_handles() -> &'static Mutex<HashMap<HandleToken, Arc<ImmutableError>>> {
        ERROR_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
    }

    thread_local! {
        static VALIDATOR_CALLBACK_ACTIVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
        static CURRENT_WINDOW_CLOSE_REQUEST: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    }

    struct CloseRequestState {
        callback_active: AtomicBool,
        owned_handles: AtomicUsize,
        release_requested: AtomicBool,
        resolved: AtomicBool,
        decision: Mutex<Option<CloseDecision>>,
    }

    struct CloseRequestHandleEntry {
        runtime: Arc<RuntimeCore>,
        identity: ObjectIdentity,
        request: u64,
        generation: u64,
        state: Arc<CloseRequestState>,
        owned: bool,
    }

    static CLOSE_REQUEST_HANDLES: OnceLock<Mutex<HashMap<HandleToken, CloseRequestHandleEntry>>> =
        OnceLock::new();

    fn close_request_handles() -> &'static Mutex<HashMap<HandleToken, CloseRequestHandleEntry>> {
        CLOSE_REQUEST_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
    }

    struct ValidatorCallbackScope;

    impl ValidatorCallbackScope {
        fn enter() -> Option<Self> {
            VALIDATOR_CALLBACK_ACTIVE.with(|active| {
                if active.replace(true) {
                    None
                } else {
                    Some(Self)
                }
            })
        }
    }

    impl Drop for ValidatorCallbackScope {
        fn drop(&mut self) {
            VALIDATOR_CALLBACK_ACTIVE.with(|active| active.set(false));
        }
    }

    #[derive(Clone, Copy)]
    enum CValidatorCallback {
        Bool(ValidateBoolCallback),
        I64(ValidateI64Callback),
        Utf8(ValidateUtf8Callback),
    }

    struct CValidatorClosure {
        callback: CValidatorCallback,
        userdata: usize,
        destroy: DestroyCallback,
        armed: std::sync::atomic::AtomicBool,
    }

    impl Drop for CValidatorClosure {
        fn drop(&mut self) {
            if self.armed.swap(false, std::sync::atomic::Ordering::AcqRel) {
                // SAFETY: ownership is armed only after successful registration.
                unsafe { (self.destroy)(self.userdata as *mut c_void) };
            }
        }
    }

    struct ValidatorRegistrationEntry {
        runtime: Arc<RuntimeCore>,
        identity: crate::handle_registry::registry::ObjectIdentity,
        validator: Weak<PropertyValidator>,
        cancelled: std::sync::atomic::AtomicBool,
    }

    static VALIDATOR_HANDLES: OnceLock<
        Mutex<HashMap<HandleToken, Arc<ValidatorRegistrationEntry>>>,
    > = OnceLock::new();

    fn validator_handles() -> &'static Mutex<HashMap<HandleToken, Arc<ValidatorRegistrationEntry>>>
    {
        VALIDATOR_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
    }

    fn issue_immutable_error(
        status: u32,
        domain: u32,
        code: u32,
        message: &[u8],
    ) -> Result<u64, ErrorApiError> {
        let mut owned_message = Vec::new();
        owned_message
            .try_reserve_exact(message.len())
            .map_err(|_| ErrorApiError::OutOfMemory)?;
        owned_message.extend_from_slice(message);
        issue_validation_error_snapshot(Arc::new(ImmutableError {
            status,
            domain,
            code,
            message: owned_message,
        }))
    }

    pub fn issue_validation_error_snapshot(
        snapshot: Arc<ValidationError>,
    ) -> Result<u64, ErrorApiError> {
        let mut handles = error_handles()
            .lock()
            .map_err(|_| ErrorApiError::Internal)?;
        handles
            .try_reserve(1)
            .map_err(|_| ErrorApiError::OutOfMemory)?;
        let token = crate::handle_registry::issue_handle_token().ok_or(ErrorApiError::Internal)?;
        handles.insert(token, snapshot);
        Ok(token.into_abi())
    }

    fn resolve_immutable_error(handle: u64) -> Result<Arc<ImmutableError>, ErrorApiError> {
        let token = HandleToken::from_abi(handle).ok_or(ErrorApiError::InvalidHandle)?;
        error_handles()
            .lock()
            .map_err(|_| ErrorApiError::Internal)?
            .get(&token)
            .cloned()
            .ok_or(ErrorApiError::InvalidHandle)
    }

    fn is_valid_error_handle(handle: u64) -> bool {
        let Some(token) = HandleToken::from_abi(handle) else {
            return false;
        };
        error_handles()
            .lock()
            .map(|handles| handles.contains_key(&token))
            .unwrap_or(false)
    }

    fn consume_error_handle_if_valid(handle: u64) {
        if let Some(token) = HandleToken::from_abi(handle) {
            if let Ok(mut handles) = error_handles().lock() {
                handles.remove(&token);
            }
        }
    }

    fn take_immutable_error_if_valid(handle: u64) -> Option<Arc<ImmutableError>> {
        let token = HandleToken::from_abi(handle)?;
        error_handles().lock().ok()?.remove(&token)
    }

    impl CValidatorClosure {
        fn invoke(&self, candidate: &TypedValue) -> ValidatorResult {
            let Some(_scope) = ValidatorCallbackScope::enter() else {
                return ValidatorResult::Failed;
            };
            let mut error_handle = 0;
            // SAFETY: successful registration retains this callback/userdata tuple until destroy.
            let status = unsafe {
                match (self.callback, candidate) {
                    (CValidatorCallback::Bool(callback), TypedValue::Bool(value)) => callback(
                        self.userdata as *mut c_void,
                        u8::from(*value),
                        &mut error_handle,
                    ),
                    (CValidatorCallback::I64(callback), TypedValue::I64(value)) => {
                        callback(self.userdata as *mut c_void, *value, &mut error_handle)
                    }
                    (CValidatorCallback::Utf8(callback), TypedValue::Utf8(value)) => callback(
                        self.userdata as *mut c_void,
                        value.as_bytes().as_ptr(),
                        value.as_bytes().len() as u64,
                        &mut error_handle,
                    ),
                    _ => return ValidatorResult::Failed,
                }
            };
            match status {
                0 if error_handle == 0 => ValidatorResult::Accepted,
                11 if error_handle == 0 => ValidatorResult::Rejected(None),
                11 if is_valid_error_handle(error_handle) => {
                    ValidatorResult::Rejected(take_immutable_error_if_valid(error_handle))
                }
                _ => {
                    consume_error_handle_if_valid(error_handle);
                    ValidatorResult::Failed
                }
            }
        }
    }

    pub fn validation_error_create(
        domain: u32,
        code: u32,
        message: &[u8],
    ) -> Result<u64, ErrorApiError> {
        let active = VALIDATOR_CALLBACK_ACTIVE.with(std::cell::Cell::get);
        if !active {
            return Err(ErrorApiError::WrongThread);
        }
        std::str::from_utf8(message).map_err(|_| ErrorApiError::InvalidArgument)?;
        issue_immutable_error(11, domain, code, message)
    }

    pub fn error_status(handle: u64) -> Result<u32, ErrorApiError> {
        resolve_immutable_error(handle).map(|error| error.status)
    }

    pub fn error_code(handle: u64) -> Result<(u32, u32), ErrorApiError> {
        resolve_immutable_error(handle).map(|error| (error.domain, error.code))
    }

    pub fn error_message_utf8(
        handle: u64,
        buffer: Option<&mut [u8]>,
    ) -> Result<usize, ErrorApiError> {
        let error = resolve_immutable_error(handle)?;
        let Some(buffer) = buffer else {
            return Ok(error.message.len());
        };
        if buffer.len() < error.message.len() {
            return Err(ErrorApiError::BufferTooSmall {
                required: error.message.len(),
            });
        }
        buffer[..error.message.len()].copy_from_slice(&error.message);
        Ok(error.message.len())
    }

    pub const BIND_ONCE: u32 = 1;
    pub const BIND_ONE_WAY: u32 = 2;
    pub const BIND_TWO_WAY: u32 = 3;

    pub type WakeCallback = unsafe extern "C" fn(*mut c_void, u32);
    pub type DestroyCallback = unsafe extern "C" fn(*mut c_void);
    pub type PostCallback = unsafe extern "C" fn(*mut c_void);
    pub type CommandStartCallback = unsafe extern "C" fn(*mut c_void, u64, u64, u64) -> u32;

    struct CommandClosure {
        callback: CommandStartCallback,
        userdata: usize,
        destroy: DestroyCallback,
        armed: AtomicBool,
    }

    impl Drop for CommandClosure {
        fn drop(&mut self) {
            if self.armed.swap(false, Ordering::AcqRel) {
                // SAFETY: ownership is armed only after command creation succeeds.
                unsafe { (self.destroy)(self.userdata as *mut c_void) };
            }
        }
    }

    pub(crate) struct CommandHandleEntry {
        runtime: Arc<RuntimeCore>,
        command: Arc<AsyncCommand>,
        state_property: HandleToken,
        can_execute_property: HandleToken,
        eligibility_property: Option<HandleToken>,
        eligibility_identity: Option<crate::handle_registry::registry::ObjectIdentity>,
        start: Mutex<Option<Arc<CommandClosure>>>,
        in_start: AtomicBool,
        owned_properties_released: AtomicBool,
    }

    static COMMAND_HANDLES: OnceLock<Mutex<HashMap<HandleToken, Arc<CommandHandleEntry>>>> =
        OnceLock::new();
    static COMMAND_CANCEL_TOKENS: OnceLock<Mutex<HashMap<HandleToken, CancelToken>>> =
        OnceLock::new();

    fn command_handles() -> &'static Mutex<HashMap<HandleToken, Arc<CommandHandleEntry>>> {
        COMMAND_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
    }

    fn command_cancel_tokens() -> &'static Mutex<HashMap<HandleToken, CancelToken>> {
        COMMAND_CANCEL_TOKENS.get_or_init(|| Mutex::new(HashMap::new()))
    }

    impl Drop for CommandHandleEntry {
        fn drop(&mut self) {
            self.start
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            self.release_owned_properties();
        }
    }

    impl CommandHandleEntry {
        fn release_owned_properties(&self) {
            if self.owned_properties_released.swap(true, Ordering::AcqRel) {
                return;
            }
            let _ = handle_release(self.state_property.into_abi());
            let _ = handle_release(self.can_execute_property.into_abi());
            if let Some(eligibility) = self.eligibility_property {
                let _ = handle_release(eligibility.into_abi());
            }
        }
    }

    struct BridgeState {
        armed: bool,
        destroyed: bool,
        in_flight: usize,
    }

    struct CAbiUiLoopBridge {
        wake: WakeCallback,
        userdata: usize,
        destroy: Option<DestroyCallback>,
        state: Mutex<BridgeState>,
        drained: Condvar,
    }

    // SAFETY: the ABI contract requires userdata callbacks to be thread-safe and permits wake
    // from any thread. The raw userdata pointer is stored as an integer and only passed back to
    // those callbacks; mutable adapter state is protected by `state`.
    unsafe impl Send for CAbiUiLoopBridge {}
    // SAFETY: same contract as `Send`; every shared mutable field is synchronized by `state`.
    unsafe impl Sync for CAbiUiLoopBridge {}

    impl CAbiUiLoopBridge {
        fn new(wake: WakeCallback, userdata: usize, destroy: Option<DestroyCallback>) -> Self {
            Self {
                wake,
                userdata,
                destroy,
                state: Mutex::new(BridgeState {
                    armed: false,
                    destroyed: false,
                    in_flight: 0,
                }),
                drained: Condvar::new(),
            }
        }

        fn arm(&self) {
            if let Ok(mut state) = self.state.lock() {
                state.armed = true;
            }
        }
    }

    impl UiLoopBridge for CAbiUiLoopBridge {
        fn schedule_wake(&self, priority: WakePriority) {
            let Ok(mut state) = self.state.lock() else {
                return;
            };
            if !state.armed || state.destroyed {
                return;
            }
            state.in_flight += 1;
            drop(state);

            let priority = match priority {
                WakePriority::Normal => 1,
                WakePriority::Control => 2,
            };
            // SAFETY: registration transfers this callback and userdata under the C ABI contract.
            unsafe { (self.wake)(self.userdata as *mut c_void, priority) };

            if let Ok(mut state) = self.state.lock() {
                state.in_flight -= 1;
                if state.in_flight == 0 {
                    self.drained.notify_all();
                }
            }
        }

        fn destroy_at_safe_point(&self) {
            let Ok(mut state) = self.state.lock() else {
                return;
            };
            if state.destroyed {
                return;
            }
            while state.in_flight != 0 {
                let Ok(next) = self.drained.wait(state) else {
                    return;
                };
                state = next;
            }
            state.destroyed = true;
            let call_destroy = state.armed;
            drop(state);
            if call_destroy {
                if let Some(destroy) = self.destroy {
                    // SAFETY: destroy is called exactly once after in-flight wake callbacks drain.
                    unsafe { destroy(self.userdata as *mut c_void) };
                }
            }
        }
    }

    struct RequestHandleEntry {
        _runtime: Arc<RuntimeCore>,
        request: ShutdownRequest,
    }

    static SHUTDOWN_REQUEST_HANDLES: OnceLock<
        Mutex<HashMap<HandleToken, Option<RequestHandleEntry>>>,
    > = OnceLock::new();

    struct PropertyHandles {
        registry: HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        owners: Mutex<HashMap<HandleToken, Arc<RuntimeCore>>>,
    }

    struct UiControlHandleEntry {
        runtime: Arc<RuntimeCore>,
        kind: UiControlKind,
        identity: ObjectIdentity,
    }

    struct UiControlHandles {
        registry: HandleRegistry<UiControlKind, RuntimeIdentity>,
        owners: Mutex<HashMap<HandleToken, UiControlHandleEntry>>,
    }

    struct SubscriptionHandleEntry {
        runtime: Arc<RuntimeCore>,
        owner: SubscriptionOwner,
        subscription: Arc<Subscription>,
    }

    #[derive(Clone, Copy)]
    enum SubscriptionOwner {
        Property(crate::handle_registry::registry::ObjectIdentity),
        Binding(crate::binding::BindingConnectionId),
        Window(ObjectIdentity),
    }

    struct BindingHandleEntry {
        runtime: Arc<RuntimeCore>,
        connection: crate::binding::BindingConnectionId,
    }

    static PROPERTY_HANDLES: OnceLock<PropertyHandles> = OnceLock::new();
    static UI_CONTROL_HANDLES: OnceLock<UiControlHandles> = OnceLock::new();
    static SUBSCRIPTION_HANDLES: OnceLock<
        Mutex<HashMap<HandleToken, Arc<SubscriptionHandleEntry>>>,
    > = OnceLock::new();
    static BINDING_HANDLES: OnceLock<Mutex<HashMap<HandleToken, BindingHandleEntry>>> =
        OnceLock::new();

    fn property_handles() -> &'static PropertyHandles {
        PROPERTY_HANDLES.get_or_init(|| PropertyHandles {
            registry: HandleRegistry::new(),
            owners: Mutex::new(HashMap::new()),
        })
    }

    fn ui_control_handles() -> &'static UiControlHandles {
        UI_CONTROL_HANDLES.get_or_init(|| UiControlHandles {
            registry: HandleRegistry::new(),
            owners: Mutex::new(HashMap::new()),
        })
    }

    fn subscription_handles() -> &'static Mutex<HashMap<HandleToken, Arc<SubscriptionHandleEntry>>>
    {
        SUBSCRIPTION_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
    }

    fn binding_handles() -> &'static Mutex<HashMap<HandleToken, BindingHandleEntry>> {
        BINDING_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
    }

    fn command_entry(handle: u64) -> Result<Arc<CommandHandleEntry>, CommandApiError> {
        let token = HandleToken::from_abi(handle).ok_or(CommandApiError::InvalidHandle)?;
        command_handles()
            .lock()
            .map_err(|_| CommandApiError::Internal)?
            .get(&token)
            .cloned()
            .ok_or(CommandApiError::InvalidHandle)
    }

    fn map_command_error(error: CommandError) -> CommandApiError {
        match error {
            CommandError::Closed => CommandApiError::Closed,
            CommandError::NotExecutable => CommandApiError::NotExecutable,
            CommandError::InvalidOperation => CommandApiError::InvalidOperation,
            CommandError::OperationIdExhausted | CommandError::SynchronizationPoisoned => {
                CommandApiError::Internal
            }
            CommandError::OutOfMemory => CommandApiError::OutOfMemory,
        }
    }

    fn map_command_runtime_error(error: RuntimeCoreError) -> CommandApiError {
        match error {
            RuntimeCoreError::WrongThread
            | RuntimeCoreError::Host(crate::runtime::HostRegistrationError::WrongThread) => {
                CommandApiError::WrongThread
            }
            RuntimeCoreError::Closed
            | RuntimeCoreError::Host(crate::runtime::HostRegistrationError::InvalidState)
            | RuntimeCoreError::Property(RuntimePropertyError::Closed) => CommandApiError::Closed,
            RuntimeCoreError::Host(crate::runtime::HostRegistrationError::OutOfMemory)
            | RuntimeCoreError::Property(RuntimePropertyError::NotificationQueueOutOfMemory) => {
                CommandApiError::OutOfMemory
            }
            RuntimeCoreError::Property(RuntimePropertyError::WrongThread) => {
                CommandApiError::WrongThread
            }
            _ => CommandApiError::Internal,
        }
    }

    fn apply_command_snapshot(
        entry: &CommandHandleEntry,
        snapshot: &CommandSnapshot,
    ) -> Result<(), CommandApiError> {
        let scope = entry
            .runtime
            .begin_host_event()
            .map_err(map_command_runtime_error)?;
        let update = apply_command_snapshot_in_event(entry, snapshot);
        let close = entry.runtime.end_host_event(scope);
        match (update, close) {
            (Err(error), _) => Err(error),
            (Ok(()), Err(error)) => Err(map_command_runtime_error(error)),
            (Ok(()), Ok(_)) => Ok(()),
        }
    }

    fn apply_command_snapshot_in_event(
        entry: &CommandHandleEntry,
        snapshot: &CommandSnapshot,
    ) -> Result<(), CommandApiError> {
        let state = snapshot.state as i64;
        entry
            .runtime
            .apply_command_properties(
                &property_handles().registry,
                entry.state_property,
                entry.can_execute_property,
                state,
                snapshot.can_execute,
            )
            .map_err(|error| map_command_runtime_error(RuntimeCoreError::Property(error)))
    }

    fn refresh_command_eligibility(
        entry: &CommandHandleEntry,
    ) -> Result<CommandSnapshot, CommandApiError> {
        if let Some(token) = entry.eligibility_property {
            let value = entry
                .runtime
                .read_property(&property_handles().registry, token)
                .map_err(|error| match error {
                    RuntimePropertyError::WrongThread => CommandApiError::WrongThread,
                    RuntimePropertyError::Closed => CommandApiError::Closed,
                    RuntimePropertyError::Registry(RegistryError::InvalidToken) => {
                        CommandApiError::InvalidHandle
                    }
                    _ => CommandApiError::Internal,
                })?;
            let TypedValue::Bool(eligible) = value else {
                return Err(CommandApiError::WrongType);
            };
            let snapshot = entry
                .command
                .set_eligible(eligible)
                .map_err(map_command_error)?;
            apply_command_snapshot(entry, &snapshot)?;
        }
        entry.command.snapshot().map_err(map_command_error)
    }

    /// Creates one asynchronous Command and its Core-owned read-only state Properties.
    pub fn command_create_async(
        runtime_handle: u64,
        eligibility_property: u64,
        callback: CommandStartCallback,
        userdata: usize,
        destroy: DestroyCallback,
    ) -> Result<u64, CommandApiError> {
        let runtime = resolve_runtime(runtime_handle).map_err(|error| match error {
            ShutdownApiError::InvalidHandle => CommandApiError::InvalidHandle,
            ShutdownApiError::WrongThread => CommandApiError::WrongThread,
            ShutdownApiError::Closed => CommandApiError::Closed,
            _ => CommandApiError::Internal,
        })?;
        if !runtime.is_ui_thread() {
            return Err(CommandApiError::WrongThread);
        }

        let handles = property_handles();
        let (eligibility_token, eligibility_identity, eligible) = if eligibility_property == 0 {
            (None, None, true)
        } else {
            let token = HandleToken::from_abi(eligibility_property)
                .ok_or(CommandApiError::InvalidHandle)?;
            let owner = handles
                .owners
                .lock()
                .map_err(|_| CommandApiError::Internal)?
                .get(&token)
                .cloned()
                .ok_or(CommandApiError::InvalidHandle)?;
            if !Arc::ptr_eq(&owner, &runtime) {
                return Err(CommandApiError::WrongRuntime);
            }
            let identity = handles
                .registry
                .validate(token, &PropertyHandleKind::Property, &runtime.identity())
                .map_err(|error| match error {
                    RegistryError::InvalidToken => CommandApiError::InvalidHandle,
                    RegistryError::WrongRuntime => CommandApiError::WrongRuntime,
                    RegistryError::WrongKind => CommandApiError::WrongType,
                    _ => CommandApiError::Internal,
                })?
                .identity();
            let kind = runtime
                .property_kind(&handles.registry, token)
                .map_err(|_| CommandApiError::WrongType)?;
            if kind != ValueKind::Bool {
                return Err(CommandApiError::WrongType);
            }
            let TypedValue::Bool(eligible) = runtime
                .read_property(&handles.registry, token)
                .map_err(|_| CommandApiError::InvalidHandle)?
            else {
                return Err(CommandApiError::WrongType);
            };
            let retained =
                runtime_retain_handle(token.into_abi()).map_err(|error| match error {
                    RuntimeRetainError::InvalidHandle => CommandApiError::InvalidHandle,
                    RuntimeRetainError::OutOfMemory => CommandApiError::OutOfMemory,
                    _ => CommandApiError::Internal,
                })?;
            (HandleToken::from_abi(retained), Some(identity), eligible)
        };

        let state_property = match register_read_only_property(
            runtime_handle,
            TypedValue::I64(crate::command::CommandState::Idle as i64),
        ) {
            Ok(token) => HandleToken::from_abi(token).ok_or(CommandApiError::Internal)?,
            Err(error) => {
                if let Some(token) = eligibility_token {
                    let _ = handle_release(token.into_abi());
                }
                return Err(match error {
                    ShutdownApiError::OutOfMemory => CommandApiError::OutOfMemory,
                    ShutdownApiError::WrongThread => CommandApiError::WrongThread,
                    ShutdownApiError::Closed => CommandApiError::Closed,
                    _ => CommandApiError::Internal,
                });
            }
        };
        let can_execute_property =
            match register_read_only_property(runtime_handle, TypedValue::Bool(eligible)) {
                Ok(token) => HandleToken::from_abi(token).ok_or(CommandApiError::Internal)?,
                Err(error) => {
                    let _ = handle_release(state_property.into_abi());
                    if let Some(token) = eligibility_token {
                        let _ = handle_release(token.into_abi());
                    }
                    return Err(match error {
                        ShutdownApiError::OutOfMemory => CommandApiError::OutOfMemory,
                        ShutdownApiError::WrongThread => CommandApiError::WrongThread,
                        ShutdownApiError::Closed => CommandApiError::Closed,
                        _ => CommandApiError::Internal,
                    });
                }
            };

        let command = Arc::new(AsyncCommand::new(eligible));
        let entry = Arc::new(CommandHandleEntry {
            runtime: Arc::clone(&runtime),
            command: Arc::clone(&command),
            state_property,
            can_execute_property,
            eligibility_property: eligibility_token,
            eligibility_identity,
            start: Mutex::new(Some(Arc::new(CommandClosure {
                callback,
                userdata,
                destroy,
                armed: AtomicBool::new(false),
            }))),
            in_start: AtomicBool::new(false),
            owned_properties_released: AtomicBool::new(false),
        });
        let mut commands = command_handles()
            .lock()
            .map_err(|_| CommandApiError::Internal)?;
        commands
            .try_reserve(1)
            .map_err(|_| CommandApiError::OutOfMemory)?;
        let token =
            crate::handle_registry::issue_handle_token().ok_or(CommandApiError::Internal)?;
        commands.insert(token, Arc::clone(&entry));
        let shutdown_entry = Arc::clone(&entry);
        if runtime
            .track_command_shutdown(move || {
                let _ = shutdown_entry.command.close();
                shutdown_entry
                    .start
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take();
                shutdown_entry.release_owned_properties();
            })
            .is_err()
        {
            commands.remove(&token);
            return Err(CommandApiError::OutOfMemory);
        }
        if let Some(start) = entry
            .start
            .lock()
            .map_err(|_| CommandApiError::Internal)?
            .as_ref()
        {
            start.armed.store(true, Ordering::Release);
        }
        Ok(token.into_abi())
    }

    /// Starts an operation and invokes the transferred start closure exactly once.
    pub fn command_execute(handle: u64) -> Result<u64, CommandApiError> {
        let entry = command_entry(handle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(CommandApiError::WrongThread);
        }
        let scope = entry
            .runtime
            .begin_host_event()
            .map_err(map_command_runtime_error)?;
        let result = (|| {
            refresh_command_eligibility(&entry)?;
            let callback = entry
                .start
                .lock()
                .map_err(|_| CommandApiError::Internal)?
                .as_ref()
                .cloned()
                .ok_or(CommandApiError::Closed)?;
            entry
                .runtime
                .prepare_command_property_update(
                    &property_handles().registry,
                    entry.state_property,
                    entry.can_execute_property,
                )
                .map_err(|error| match map_command_property_error(error) {
                    PropertyApiError::WrongThread => CommandApiError::WrongThread,
                    PropertyApiError::Closed => CommandApiError::Closed,
                    PropertyApiError::OutOfMemory => CommandApiError::OutOfMemory,
                    _ => CommandApiError::Internal,
                })?;
            let cancel_handle =
                crate::handle_registry::issue_handle_token().ok_or(CommandApiError::Internal)?;
            command_cancel_tokens()
                .lock()
                .map_err(|_| CommandApiError::Internal)?
                .try_reserve(1)
                .map_err(|_| CommandApiError::OutOfMemory)?;

            if !entry.command.can_execute().map_err(map_command_error)? {
                return Err(CommandApiError::NotExecutable);
            }
            let operation_id = entry
                .runtime
                .issue_command_operation_id()
                .map_err(map_command_error)?;
            let (_, cancel_token, snapshot) = entry
                .command
                .execute_with_id(operation_id)
                .map_err(map_command_error)?;
            apply_command_snapshot_in_event(&entry, &snapshot)?;
            command_cancel_tokens()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(cancel_handle, cancel_token);
            entry
                .in_start
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .map_err(|_| CommandApiError::InvalidOperation)?;
            struct StartGuard<'a>(&'a AtomicBool);
            impl Drop for StartGuard<'_> {
                fn drop(&mut self) {
                    self.0.store(false, Ordering::Release);
                }
            }
            let _guard = StartGuard(&entry.in_start);
            let callback_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // SAFETY: callback state is transferred at successful creation and the two handles
                // are borrowed for this synchronous start invocation only.
                unsafe {
                    (callback.callback)(
                        callback.userdata as *mut c_void,
                        handle,
                        operation_id,
                        cancel_handle.into_abi(),
                    )
                }
            }));
            command_cancel_tokens()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&cancel_handle);

            if !matches!(callback_result, Ok(0)) {
                let failed = entry
                    .command
                    .complete_failure(operation_id, 0, 0, b"Command start callback failed")
                    .map_err(map_command_error)?;
                apply_command_snapshot_in_event(&entry, &failed)?;
            }
            Ok(operation_id)
        })();
        let close = entry.runtime.end_host_event(scope);
        match (result, close) {
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(map_command_runtime_error(error)),
            (Ok(operation_id), Ok(_)) => Ok(operation_id),
        }
    }

    pub fn command_cancel(handle: u64, operation_id: u64) -> Result<(), CommandApiError> {
        let entry = command_entry(handle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(CommandApiError::WrongThread);
        }
        let snapshot = entry
            .command
            .cancel(operation_id)
            .map_err(map_command_error)?;
        apply_command_snapshot(&entry, &snapshot)
    }

    pub fn command_close(handle: u64) -> Result<(), CommandApiError> {
        let entry = command_entry(handle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(CommandApiError::WrongThread);
        }
        if entry.in_start.load(Ordering::Acquire) {
            return Err(CommandApiError::InvalidOperation);
        }
        let snapshot = entry.command.close().map_err(map_command_error)?;
        apply_command_snapshot(&entry, &snapshot)?;
        entry
            .start
            .lock()
            .map_err(|_| CommandApiError::Internal)?
            .take();
        entry.release_owned_properties();
        Ok(())
    }

    pub fn command_state_property(handle: u64) -> Result<u64, CommandApiError> {
        let entry = command_entry(handle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(CommandApiError::WrongThread);
        }
        entry.command.ensure_open().map_err(map_command_error)?;
        runtime_retain_handle(entry.state_property.into_abi()).map_err(|error| match error {
            RuntimeRetainError::InvalidHandle => CommandApiError::InvalidHandle,
            RuntimeRetainError::OutOfMemory => CommandApiError::OutOfMemory,
            _ => CommandApiError::Internal,
        })
    }

    pub fn command_can_execute_property(handle: u64) -> Result<u64, CommandApiError> {
        let entry = command_entry(handle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(CommandApiError::WrongThread);
        }
        entry.command.ensure_open().map_err(map_command_error)?;
        refresh_command_eligibility(&entry)?;
        runtime_retain_handle(entry.can_execute_property.into_abi()).map_err(|error| match error {
            RuntimeRetainError::InvalidHandle => CommandApiError::InvalidHandle,
            RuntimeRetainError::OutOfMemory => CommandApiError::OutOfMemory,
            _ => CommandApiError::Internal,
        })
    }

    pub fn command_failure_snapshot(handle: u64) -> Result<u64, CommandApiError> {
        let entry = command_entry(handle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(CommandApiError::WrongThread);
        }
        entry.command.ensure_open().map_err(map_command_error)?;
        let snapshot = entry.command.snapshot().map_err(map_command_error)?;
        match snapshot.failure {
            None => Ok(0),
            Some(failure) => {
                issue_immutable_error(12, failure.domain, failure.code, &failure.message)
                    .map_err(|_| CommandApiError::OutOfMemory)
            }
        }
    }

    pub fn command_cancel_token_requested(handle: u64) -> Result<bool, CommandApiError> {
        let token = HandleToken::from_abi(handle).ok_or(CommandApiError::InvalidHandle)?;
        let cancel_tokens = command_cancel_tokens()
            .lock()
            .map_err(|_| CommandApiError::Internal)?;
        cancel_tokens
            .get(&token)
            .map(CancelToken::is_requested)
            .ok_or(CommandApiError::InvalidHandle)
    }

    pub fn command_complete_success(handle: u64, operation_id: u64) -> Result<(), CommandApiError> {
        let entry = command_entry(handle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(CommandApiError::WrongThread);
        }
        let snapshot = entry
            .command
            .complete_success(operation_id)
            .map_err(map_command_error)?;
        apply_command_snapshot(&entry, &snapshot)
    }

    pub fn command_complete_failure(
        handle: u64,
        operation_id: u64,
        domain: u32,
        code: u32,
        message: &[u8],
    ) -> Result<(), CommandApiError> {
        let message_text =
            std::str::from_utf8(message).map_err(|_| CommandApiError::InvalidArgument)?;
        let entry = command_entry(handle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(CommandApiError::WrongThread);
        }
        let snapshot = entry
            .command
            .complete_failure(operation_id, domain, code, message_text.as_bytes())
            .map_err(map_command_error)?;
        apply_command_snapshot(&entry, &snapshot)
    }

    pub fn command_complete_cancelled(
        handle: u64,
        operation_id: u64,
    ) -> Result<(), CommandApiError> {
        let entry = command_entry(handle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(CommandApiError::WrongThread);
        }
        let snapshot = entry
            .command
            .complete_cancelled(operation_id)
            .map_err(map_command_error)?;
        apply_command_snapshot(&entry, &snapshot)
    }

    fn map_binding_runtime_error(error: RuntimeBindingError) -> BindingApiError {
        match error {
            RuntimeBindingError::Property(RuntimePropertyError::Unsupported) => {
                BindingApiError::Unsupported
            }
            RuntimeBindingError::Property(RuntimePropertyError::WrongThread) => {
                BindingApiError::WrongThread
            }
            RuntimeBindingError::Property(RuntimePropertyError::Closed) => BindingApiError::Closed,
            RuntimeBindingError::Property(RuntimePropertyError::ValidationFailed(error)) => {
                BindingApiError::ValidationFailed(error)
            }
            RuntimeBindingError::Property(_) => BindingApiError::Internal,
            RuntimeBindingError::Registry(RegistryError::InvalidToken) => {
                BindingApiError::InvalidHandle
            }
            RuntimeBindingError::Registry(RegistryError::WrongRuntime) => {
                BindingApiError::WrongRuntime
            }
            RuntimeBindingError::Registry(RegistryError::WrongKind) => BindingApiError::WrongType,
            RuntimeBindingError::Registry(RegistryError::OutOfMemory) => {
                BindingApiError::OutOfMemory
            }
            RuntimeBindingError::Registry(_) => BindingApiError::Internal,
            RuntimeBindingError::Graph(crate::binding::BindingGraphError::OutOfMemory) => {
                BindingApiError::OutOfMemory
            }
            RuntimeBindingError::Graph(
                crate::binding::BindingGraphError::SelfConnection
                | crate::binding::BindingGraphError::DuplicateEdge
                | crate::binding::BindingGraphError::TargetAlreadyHasInput
                | crate::binding::BindingGraphError::Cycle,
            ) => BindingApiError::InvalidArgument,
            RuntimeBindingError::Graph(_) => BindingApiError::Internal,
            RuntimeBindingError::InitialCopy(crate::value::BindingCopyError::KindMismatch(_)) => {
                BindingApiError::WrongType
            }
            RuntimeBindingError::InitialCopy(crate::value::BindingCopyError::Copy(
                OwnedUtf8Error::OutOfMemory,
            ))
            | RuntimeBindingError::SourceSnapshotCopy(OwnedUtf8Error::OutOfMemory)
            | RuntimeBindingError::NotificationQueueOutOfMemory
            | RuntimeBindingError::BindingOwnerOutOfMemory
            | RuntimeBindingError::PropagationPreparationOutOfMemory => {
                BindingApiError::OutOfMemory
            }
            RuntimeBindingError::InitialCopy(crate::value::BindingCopyError::Copy(_))
            | RuntimeBindingError::SourceSnapshotCopy(_)
            | RuntimeBindingError::Store(_)
            | RuntimeBindingError::WrongObjectState
            | RuntimeBindingError::SynchronizationPoisoned => BindingApiError::Internal,
        }
    }

    fn request_handles() -> &'static Mutex<HashMap<HandleToken, Option<RequestHandleEntry>>> {
        SHUTDOWN_REQUEST_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
    }

    /// Creates one Runtime and registers its owning opaque handle.
    pub fn runtime_create_handle() -> Result<u64, RuntimeCreateError> {
        register_runtime_handle(None)
    }

    /// Creates one Runtime bound to a private platform factory without changing the public C ABI.
    pub fn runtime_create_handle_with_platform_factory(
        factory: Arc<dyn PlatformRuntimeFactory>,
    ) -> Result<u64, RuntimeCreateError> {
        register_runtime_handle(Some(factory))
    }

    fn register_runtime_handle(
        factory: Option<Arc<dyn PlatformRuntimeFactory>>,
    ) -> Result<u64, RuntimeCreateError> {
        let runtime = Arc::new(
            RuntimeCore::new_with_platform_factory(factory)
                .map_err(|_| RuntimeCreateError::Internal)?,
        );
        process_runtime_handles()
            .insert(&runtime)
            .map(HandleToken::into_abi)
            .map_err(|error| match error {
                RuntimeHandleError::OutOfMemory => RuntimeCreateError::OutOfMemory,
                RuntimeHandleError::InvalidToken
                | RuntimeHandleError::TokenUnavailable
                | RuntimeHandleError::SynchronizationPoisoned => RuntimeCreateError::Internal,
            })
    }

    /// Reports whether the current thread is the one captured by this Runtime.
    pub fn runtime_is_ui_thread(handle: u64) -> Result<bool, RuntimeQueryError> {
        let token = HandleToken::from_abi(handle).ok_or(RuntimeQueryError::InvalidHandle)?;
        let runtime = process_runtime_handles()
            .resolve(token)
            .map_err(|error| match error {
                RuntimeHandleError::InvalidToken => RuntimeQueryError::InvalidHandle,
                RuntimeHandleError::TokenUnavailable
                | RuntimeHandleError::OutOfMemory
                | RuntimeHandleError::SynchronizationPoisoned => RuntimeQueryError::Internal,
            })?;
        Ok(runtime.is_ui_thread())
    }

    /// Creates an independent owner token for an existing Runtime handle.
    pub fn runtime_retain_handle(handle: u64) -> Result<u64, RuntimeRetainError> {
        let token = HandleToken::from_abi(handle).ok_or(RuntimeRetainError::InvalidHandle)?;
        {
            let mut requests = close_request_handles()
                .lock()
                .map_err(|_| RuntimeRetainError::Internal)?;
            if let Some((runtime, identity, request, generation, state, owned)) =
                requests.get(&token).map(|entry| {
                    (
                        Arc::clone(&entry.runtime),
                        entry.identity,
                        entry.request,
                        entry.generation,
                        Arc::clone(&entry.state),
                        entry.owned,
                    )
                })
            {
                if !owned {
                    return Err(RuntimeRetainError::InvalidHandle);
                }
                requests
                    .try_reserve(1)
                    .map_err(|_| RuntimeRetainError::OutOfMemory)?;
                let retained = crate::handle_registry::issue_handle_token()
                    .ok_or(RuntimeRetainError::Internal)?;
                let entry = CloseRequestHandleEntry {
                    runtime,
                    identity,
                    request,
                    generation,
                    state: Arc::clone(&state),
                    owned: true,
                };
                state.owned_handles.fetch_add(1, Ordering::AcqRel);
                requests.insert(retained, entry);
                return Ok(retained.into_abi());
            }
        }
        {
            let mut errors = error_handles()
                .lock()
                .map_err(|_| RuntimeRetainError::Internal)?;
            if let Some(error) = errors.get(&token).cloned() {
                errors
                    .try_reserve(1)
                    .map_err(|_| RuntimeRetainError::OutOfMemory)?;
                let retained = crate::handle_registry::issue_handle_token()
                    .ok_or(RuntimeRetainError::Internal)?;
                errors.insert(retained, error);
                return Ok(retained.into_abi());
            }
        }
        {
            let mut validators = validator_handles()
                .lock()
                .map_err(|_| RuntimeRetainError::Internal)?;
            if let Some(entry) = validators.get(&token).cloned() {
                validators
                    .try_reserve(1)
                    .map_err(|_| RuntimeRetainError::OutOfMemory)?;
                let retained = crate::handle_registry::issue_handle_token()
                    .ok_or(RuntimeRetainError::Internal)?;
                validators.insert(retained, entry);
                return Ok(retained.into_abi());
            }
        }
        {
            let mut bindings = binding_handles()
                .lock()
                .map_err(|_| RuntimeRetainError::Internal)?;
            if let Some((runtime, connection)) = bindings
                .get(&token)
                .map(|entry| (Arc::clone(&entry.runtime), entry.connection))
            {
                bindings
                    .try_reserve(1)
                    .map_err(|_| RuntimeRetainError::OutOfMemory)?;
                let retained = crate::handle_registry::issue_handle_token()
                    .ok_or(RuntimeRetainError::Internal)?;
                let retained_entry = BindingHandleEntry {
                    runtime,
                    connection,
                };
                bindings.insert(retained, retained_entry);
                return Ok(retained.into_abi());
            }
        }
        {
            let mut subscriptions = subscription_handles()
                .lock()
                .map_err(|_| RuntimeRetainError::Internal)?;
            if let Some(entry) = subscriptions.get(&token).cloned() {
                subscriptions
                    .try_reserve(1)
                    .map_err(|_| RuntimeRetainError::OutOfMemory)?;
                let retained = crate::handle_registry::issue_handle_token()
                    .ok_or(RuntimeRetainError::Internal)?;
                subscriptions.insert(retained, entry);
                return Ok(retained.into_abi());
            }
        }
        {
            let mut commands = command_handles()
                .lock()
                .map_err(|_| RuntimeRetainError::Internal)?;
            if let Some(entry) = commands.get(&token).cloned() {
                commands
                    .try_reserve(1)
                    .map_err(|_| RuntimeRetainError::OutOfMemory)?;
                let retained = crate::handle_registry::issue_handle_token()
                    .ok_or(RuntimeRetainError::Internal)?;
                commands.insert(retained, entry);
                return Ok(retained.into_abi());
            }
        }
        {
            let mut cancel_tokens = command_cancel_tokens()
                .lock()
                .map_err(|_| RuntimeRetainError::Internal)?;
            if let Some(cancel) = cancel_tokens.get(&token).cloned() {
                cancel_tokens
                    .try_reserve(1)
                    .map_err(|_| RuntimeRetainError::OutOfMemory)?;
                let retained = crate::handle_registry::issue_handle_token()
                    .ok_or(RuntimeRetainError::Internal)?;
                cancel_tokens.insert(retained, cancel);
                return Ok(retained.into_abi());
            }
        }
        {
            let handles = property_handles();
            let mut owners = handles
                .owners
                .lock()
                .map_err(|_| RuntimeRetainError::Internal)?;
            if let Some(runtime) = owners.get(&token).cloned() {
                owners
                    .try_reserve(1)
                    .map_err(|_| RuntimeRetainError::OutOfMemory)?;
                let retained = handles
                    .registry
                    .retain(token)
                    .map_err(|error| match error {
                        RegistryError::OutOfMemory => RuntimeRetainError::OutOfMemory,
                        RegistryError::InvalidToken => RuntimeRetainError::InvalidHandle,
                        _ => RuntimeRetainError::Internal,
                    })?;
                owners.insert(retained, runtime);
                return Ok(retained.into_abi());
            }
        }
        {
            let handles = ui_control_handles();
            let mut owners = handles
                .owners
                .lock()
                .map_err(|_| RuntimeRetainError::Internal)?;
            if let Some((runtime, kind, identity)) = owners
                .get(&token)
                .map(|entry| (Arc::clone(&entry.runtime), entry.kind, entry.identity))
            {
                owners
                    .try_reserve(1)
                    .map_err(|_| RuntimeRetainError::OutOfMemory)?;
                let retained = handles
                    .registry
                    .retain(token)
                    .map_err(|error| match error {
                        RegistryError::OutOfMemory => RuntimeRetainError::OutOfMemory,
                        RegistryError::InvalidToken => RuntimeRetainError::InvalidHandle,
                        _ => RuntimeRetainError::Internal,
                    })?;
                owners.insert(
                    retained,
                    UiControlHandleEntry {
                        runtime,
                        kind,
                        identity,
                    },
                );
                return Ok(retained.into_abi());
            }
        }
        process_runtime_handles()
            .retain(token)
            .map(HandleToken::into_abi)
            .map_err(|error| match error {
                RuntimeHandleError::InvalidToken => RuntimeRetainError::InvalidHandle,
                RuntimeHandleError::OutOfMemory => RuntimeRetainError::OutOfMemory,
                RuntimeHandleError::TokenUnavailable
                | RuntimeHandleError::SynchronizationPoisoned => RuntimeRetainError::Internal,
            })
    }

    fn map_runtime_error(error: RuntimeCoreError) -> ShutdownApiError {
        use crate::runtime::{HostRegistrationError, HostShutdownCommitError};
        match error {
            RuntimeCoreError::WrongThread
            | RuntimeCoreError::Host(HostRegistrationError::WrongThread) => {
                ShutdownApiError::WrongThread
            }
            RuntimeCoreError::BridgeNotRegistered
            | RuntimeCoreError::Host(HostRegistrationError::HostUnavailable { .. }) => {
                ShutdownApiError::Unsupported
            }
            RuntimeCoreError::BridgeAlreadyRegistered => ShutdownApiError::InvalidArgument,
            RuntimeCoreError::Dispatch(crate::dispatcher::DispatchError::AlreadyDispatching) => {
                ShutdownApiError::InvalidArgument
            }
            RuntimeCoreError::Dispatch(crate::dispatcher::DispatchError::WrongThread) => {
                ShutdownApiError::WrongThread
            }
            RuntimeCoreError::Dispatch(crate::dispatcher::DispatchError::Closed) => {
                ShutdownApiError::Closed
            }
            RuntimeCoreError::Dispatch(
                crate::dispatcher::DispatchError::SynchronizationPoisoned,
            ) => ShutdownApiError::Internal,
            RuntimeCoreError::Closed
            | RuntimeCoreError::Host(HostRegistrationError::InvalidState) => {
                ShutdownApiError::Closed
            }
            RuntimeCoreError::Host(HostRegistrationError::OutOfMemory) => {
                ShutdownApiError::OutOfMemory
            }
            RuntimeCoreError::Host(HostRegistrationError::InvalidContext) => {
                ShutdownApiError::InvalidArgument
            }
            RuntimeCoreError::Shutdown(HostShutdownCommitError::Boundary(_))
            | RuntimeCoreError::Shutdown(HostShutdownCommitError::Commit(_))
            | RuntimeCoreError::CleanupRegistrationFailed
            | RuntimeCoreError::IdentityUnavailable
            | RuntimeCoreError::SynchronizationPoisoned
            | RuntimeCoreError::Property(_)
            | RuntimeCoreError::Host(_) => ShutdownApiError::Internal,
        }
    }

    fn resolve_runtime(handle: u64) -> Result<Arc<RuntimeCore>, ShutdownApiError> {
        let token = HandleToken::from_abi(handle).ok_or(ShutdownApiError::InvalidHandle)?;
        process_runtime_handles()
            .resolve(token)
            .map_err(|error| match error {
                RuntimeHandleError::InvalidToken => ShutdownApiError::InvalidHandle,
                RuntimeHandleError::OutOfMemory => ShutdownApiError::OutOfMemory,
                RuntimeHandleError::TokenUnavailable
                | RuntimeHandleError::SynchronizationPoisoned => ShutdownApiError::Internal,
            })
    }

    fn map_registry_error(error: RegistryError) -> ShutdownApiError {
        match error {
            RegistryError::InvalidToken
            | RegistryError::WrongKind
            | RegistryError::WrongRuntime => ShutdownApiError::InvalidHandle,
            RegistryError::OutOfMemory => ShutdownApiError::OutOfMemory,
            RegistryError::TokenExhausted
            | RegistryError::CleanupUnavailable
            | RegistryError::SynchronizationPoisoned
            | RegistryError::ReferenceCountExhausted => ShutdownApiError::Internal,
        }
    }

    fn map_property_registration_error(error: RuntimeObjectRegistrationError) -> ShutdownApiError {
        match error {
            RuntimeObjectRegistrationError::WrongThread => ShutdownApiError::WrongThread,
            RuntimeObjectRegistrationError::Closed => ShutdownApiError::Closed,
            RuntimeObjectRegistrationError::SynchronizationPoisoned
            | RuntimeObjectRegistrationError::Store(UiObjectStoreError::WrongThread)
            | RuntimeObjectRegistrationError::Store(UiObjectStoreError::InvalidIdentity)
            | RuntimeObjectRegistrationError::Store(UiObjectStoreError::IdentityExhausted)
            | RuntimeObjectRegistrationError::Registry(RegistryError::TokenExhausted)
            | RuntimeObjectRegistrationError::Registry(RegistryError::CleanupUnavailable)
            | RuntimeObjectRegistrationError::Registry(RegistryError::SynchronizationPoisoned)
            | RuntimeObjectRegistrationError::Registry(RegistryError::ReferenceCountExhausted) => {
                ShutdownApiError::Internal
            }
            RuntimeObjectRegistrationError::Store(UiObjectStoreError::OutOfMemory)
            | RuntimeObjectRegistrationError::Registry(RegistryError::OutOfMemory) => {
                ShutdownApiError::OutOfMemory
            }
            RuntimeObjectRegistrationError::Registry(
                RegistryError::InvalidToken
                | RegistryError::WrongKind
                | RegistryError::WrongRuntime,
            ) => ShutdownApiError::Internal,
        }
    }

    fn register_property(
        runtime_handle: u64,
        initial: TypedValue,
    ) -> Result<u64, ShutdownApiError> {
        let runtime = resolve_runtime(runtime_handle)?;
        let handles = property_handles();
        let mut owners = handles
            .owners
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?;
        owners
            .try_reserve(1)
            .map_err(|_| ShutdownApiError::OutOfMemory)?;
        let token = runtime
            .register_property(&handles.registry, initial)
            .map_err(map_property_registration_error)?;
        owners.insert(token, runtime);
        Ok(token.into_abi())
    }

    fn register_read_only_property(
        runtime_handle: u64,
        initial: TypedValue,
    ) -> Result<u64, ShutdownApiError> {
        let runtime = resolve_runtime(runtime_handle)?;
        let handles = property_handles();
        let mut owners = handles
            .owners
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?;
        owners
            .try_reserve(1)
            .map_err(|_| ShutdownApiError::OutOfMemory)?;
        let token = runtime
            .register_read_only_property(&handles.registry, initial)
            .map_err(map_property_registration_error)?;
        owners.insert(token, runtime);
        Ok(token.into_abi())
    }

    fn map_ui_control_registry_error(error: RegistryError) -> UiControlApiError {
        match error {
            RegistryError::InvalidToken => UiControlApiError::InvalidHandle,
            RegistryError::WrongKind => UiControlApiError::WrongType,
            RegistryError::WrongRuntime => UiControlApiError::WrongRuntime,
            RegistryError::OutOfMemory => UiControlApiError::OutOfMemory,
            RegistryError::TokenExhausted
            | RegistryError::CleanupUnavailable
            | RegistryError::SynchronizationPoisoned
            | RegistryError::ReferenceCountExhausted => UiControlApiError::Internal,
        }
    }

    fn map_ui_control_registration_error(
        error: RuntimeObjectRegistrationError,
    ) -> UiControlApiError {
        match error {
            RuntimeObjectRegistrationError::WrongThread => UiControlApiError::WrongThread,
            RuntimeObjectRegistrationError::Closed => UiControlApiError::Closed,
            RuntimeObjectRegistrationError::Store(UiObjectStoreError::OutOfMemory)
            | RuntimeObjectRegistrationError::Registry(RegistryError::OutOfMemory) => {
                UiControlApiError::OutOfMemory
            }
            RuntimeObjectRegistrationError::Registry(error) => map_ui_control_registry_error(error),
            RuntimeObjectRegistrationError::SynchronizationPoisoned
            | RuntimeObjectRegistrationError::Store(_) => UiControlApiError::Internal,
        }
    }

    fn map_ui_control_tree_error(error: RuntimeTreeError) -> UiControlApiError {
        match error {
            RuntimeTreeError::WrongThread => UiControlApiError::WrongThread,
            RuntimeTreeError::Closed => UiControlApiError::Closed,
            RuntimeTreeError::Registry(error) => map_ui_control_registry_error(error),
            RuntimeTreeError::Registration(error) => map_ui_control_registration_error(error),
            RuntimeTreeError::Tree(TreeError::WrongThread) => UiControlApiError::WrongThread,
            RuntimeTreeError::Tree(TreeError::RuntimeMismatch) => UiControlApiError::WrongRuntime,
            RuntimeTreeError::Tree(
                TreeError::ParentNotContainer
                | TreeError::AlreadyParented
                | TreeError::Cycle
                | TreeError::ChildNotFound,
            ) => UiControlApiError::InvalidArgument,
            RuntimeTreeError::Tree(TreeError::AllocationFailed) => UiControlApiError::OutOfMemory,
            RuntimeTreeError::SynchronizationPoisoned | RuntimeTreeError::Tree(_) => {
                UiControlApiError::Internal
            }
        }
    }

    fn map_ui_control_runtime_error(error: ShutdownApiError) -> UiControlApiError {
        match error {
            ShutdownApiError::InvalidHandle => UiControlApiError::InvalidHandle,
            ShutdownApiError::InvalidArgument => UiControlApiError::InvalidArgument,
            ShutdownApiError::WrongThread => UiControlApiError::WrongThread,
            ShutdownApiError::Closed => UiControlApiError::Closed,
            ShutdownApiError::Unsupported | ShutdownApiError::HostUnavailable { .. } => {
                UiControlApiError::Unsupported
            }
            ShutdownApiError::OutOfMemory => UiControlApiError::OutOfMemory,
            ShutdownApiError::Internal => UiControlApiError::Internal,
        }
    }

    fn map_window_lifecycle_error(error: WindowLifecycleError) -> UiControlApiError {
        match error {
            WindowLifecycleError::Unsupported => UiControlApiError::Unsupported,
            WindowLifecycleError::InvalidState | WindowLifecycleError::StaleRequest => {
                UiControlApiError::InvalidArgument
            }
            WindowLifecycleError::OutOfMemory => UiControlApiError::OutOfMemory,
            WindowLifecycleError::IdentifierExhausted | WindowLifecycleError::PeerFailure => {
                UiControlApiError::Internal
            }
            WindowLifecycleError::Frame(crate::platform::frame::FrameError::Busy)
            | WindowLifecycleError::Frame(crate::platform::frame::FrameError::InvalidTransition)
            | WindowLifecycleError::Frame(crate::platform::frame::FrameError::InvalidGeneration)
            | WindowLifecycleError::Frame(crate::platform::frame::FrameError::StaleTicket) => {
                UiControlApiError::InvalidArgument
            }
            WindowLifecycleError::Frame(
                crate::platform::frame::FrameError::IdentifierExhausted,
            ) => UiControlApiError::Internal,
        }
    }

    fn register_internal_control_property(
        runtime: &Arc<RuntimeCore>,
        initial: TypedValue,
    ) -> Result<InternalOwner<RuntimeIdentity>, UiControlApiError> {
        let handles = property_handles();
        let token = {
            let mut owners = handles
                .owners
                .lock()
                .map_err(|_| UiControlApiError::Internal)?;
            owners
                .try_reserve(1)
                .map_err(|_| UiControlApiError::OutOfMemory)?;
            let token = runtime
                .register_property(&handles.registry, initial)
                .map_err(map_ui_control_registration_error)?;
            owners.insert(token, Arc::clone(runtime));
            token
        };

        let owner = (|| {
            let pin = handles
                .registry
                .validate(token, &PropertyHandleKind::Property, &runtime.identity())
                .map_err(map_ui_control_registry_error)?;
            pin.acquire_internal_owner()
                .map_err(map_ui_control_registry_error)
        })();
        let released = property_release(token).map_err(map_ui_control_runtime_error)?;
        if !released {
            return Err(UiControlApiError::Internal);
        }
        owner
    }

    fn empty_utf8_value() -> Result<TypedValue, UiControlApiError> {
        OwnedUtf8::copy_from_bytes(b"")
            .map(TypedValue::Utf8)
            .map_err(|error| match error {
                OwnedUtf8Error::InvalidUtf8 => UiControlApiError::InvalidArgument,
                OwnedUtf8Error::OutOfMemory => UiControlApiError::OutOfMemory,
            })
    }

    fn input_utf8_value(bytes: &[u8]) -> Result<TypedValue, UiControlApiError> {
        OwnedUtf8::copy_from_bytes(bytes)
            .map(TypedValue::Utf8)
            .map_err(|error| match error {
                OwnedUtf8Error::InvalidUtf8 => UiControlApiError::InvalidArgument,
                OwnedUtf8Error::OutOfMemory => UiControlApiError::OutOfMemory,
            })
    }

    fn create_ui_control(
        runtime_handle: u64,
        kind: UiControlKind,
        initial_text: Option<&[u8]>,
    ) -> Result<u64, UiControlApiError> {
        let runtime = resolve_runtime(runtime_handle).map_err(map_ui_control_runtime_error)?;
        if !runtime.is_ui_thread() {
            return Err(UiControlApiError::WrongThread);
        }
        let initial_text = match kind {
            UiControlKind::Text | UiControlKind::TextBox | UiControlKind::Button => {
                Some(input_utf8_value(initial_text.unwrap_or_default())?)
            }
            UiControlKind::Window | UiControlKind::StackPanel if initial_text.is_none() => None,
            UiControlKind::Window | UiControlKind::StackPanel => {
                return Err(UiControlApiError::InvalidArgument);
            }
        };

        let controls = ui_control_handles();
        let mut entries = controls
            .owners
            .lock()
            .map_err(|_| UiControlApiError::Internal)?;
        entries
            .try_reserve(1)
            .map_err(|_| UiControlApiError::OutOfMemory)?;

        let mut properties = Vec::new();
        let property_count = match kind {
            UiControlKind::Window => 3,
            UiControlKind::StackPanel => 2,
            UiControlKind::Text | UiControlKind::TextBox | UiControlKind::Button => 3,
        };
        properties
            .try_reserve_exact(property_count)
            .map_err(|_| UiControlApiError::OutOfMemory)?;
        properties.push(UiControlPropertyOwner {
            property: UiControlProperty::IsEnabled,
            owner: register_internal_control_property(&runtime, TypedValue::Bool(true))?,
        });
        properties.push(UiControlPropertyOwner {
            property: UiControlProperty::AccessibleName,
            owner: register_internal_control_property(&runtime, empty_utf8_value()?)?,
        });
        let text_property = match kind {
            UiControlKind::Window => Some((UiControlProperty::WindowTitle, empty_utf8_value()?)),
            UiControlKind::Text => Some((
                UiControlProperty::TextContent,
                initial_text.expect("Text requires initialized UTF-8 content"),
            )),
            UiControlKind::TextBox => Some((
                UiControlProperty::TextBoxValue,
                initial_text.expect("TextBox requires initialized UTF-8 value"),
            )),
            UiControlKind::Button => Some((
                UiControlProperty::ButtonLabel,
                initial_text.expect("Button requires initialized UTF-8 label"),
            )),
            UiControlKind::StackPanel => None,
        };
        if let Some((property, value)) = text_property {
            properties.push(UiControlPropertyOwner {
                property,
                owner: register_internal_control_property(&runtime, value)?,
            });
        }

        let token = runtime
            .register_tree_ui_object::<_, UiControlState, _>(
                &controls.registry,
                kind,
                kind.is_container(),
                move || UiControlState::new(kind, properties),
            )
            .map_err(map_ui_control_tree_error)?;
        let identity = controls
            .registry
            .validate(token, &kind, &runtime.identity())
            .map_err(map_ui_control_registry_error)?
            .identity();
        entries.insert(
            token,
            UiControlHandleEntry {
                runtime,
                kind,
                identity,
            },
        );
        Ok(token.into_abi())
    }

    fn ui_control_property(
        control: u64,
        property: UiControlProperty,
    ) -> Result<u64, UiControlApiError> {
        let token = HandleToken::from_abi(control).ok_or(UiControlApiError::InvalidHandle)?;
        let controls = ui_control_handles();
        let (runtime, kind) = {
            let entries = controls
                .owners
                .lock()
                .map_err(|_| UiControlApiError::Internal)?;
            let entry = entries
                .get(&token)
                .ok_or(UiControlApiError::InvalidHandle)?;
            (Arc::clone(&entry.runtime), entry.kind)
        };
        if !kind.supports_property(property) {
            return Err(UiControlApiError::WrongType);
        }
        if !runtime.is_ui_thread() {
            return Err(UiControlApiError::WrongThread);
        }
        let pin = controls
            .registry
            .validate(token, &kind, &runtime.identity())
            .map_err(map_ui_control_registry_error)?;
        let owner = runtime
            .ui_control_property_owner(pin.identity(), property)
            .map_err(|error| match error {
                RuntimePropertyError::WrongThread => UiControlApiError::WrongThread,
                RuntimePropertyError::Closed => UiControlApiError::Closed,
                RuntimePropertyError::Store(UiObjectStoreError::OutOfMemory) => {
                    UiControlApiError::OutOfMemory
                }
                RuntimePropertyError::Registry(error) => map_ui_control_registry_error(error),
                _ => UiControlApiError::Internal,
            })?;
        let properties = property_handles();
        let mut property_owners = properties
            .owners
            .lock()
            .map_err(|_| UiControlApiError::Internal)?;
        property_owners
            .try_reserve(1)
            .map_err(|_| UiControlApiError::OutOfMemory)?;
        let property_token = properties
            .registry
            .register_from_internal_owner(PropertyHandleKind::Property, runtime.identity(), &owner)
            .map_err(map_ui_control_registry_error)?;
        property_owners.insert(property_token, runtime);
        Ok(property_token.into_abi())
    }

    fn ui_control_entry(
        control: u64,
    ) -> Result<(HandleToken, Arc<RuntimeCore>, UiControlKind, ObjectIdentity), UiControlApiError>
    {
        let token = HandleToken::from_abi(control).ok_or(UiControlApiError::InvalidHandle)?;
        let controls = ui_control_handles();
        let entries = controls
            .owners
            .lock()
            .map_err(|_| UiControlApiError::Internal)?;
        let entry = entries
            .get(&token)
            .ok_or(UiControlApiError::InvalidHandle)?;
        Ok((
            token,
            Arc::clone(&entry.runtime),
            entry.kind,
            entry.identity,
        ))
    }

    fn validate_ui_control(
        control: u64,
        expected_kind: UiControlKind,
    ) -> Result<(HandleToken, Arc<RuntimeCore>, ObjectIdentity), UiControlApiError> {
        let (token, runtime, kind, identity) = ui_control_entry(control)?;
        if kind != expected_kind {
            return Err(UiControlApiError::WrongType);
        }
        if !runtime.is_ui_thread() {
            return Err(UiControlApiError::WrongThread);
        }
        ui_control_handles()
            .registry
            .validate(token, &kind, &runtime.identity())
            .map_err(map_ui_control_registry_error)?;
        Ok((token, runtime, identity))
    }

    fn tree_child_handle(
        parent_token: HandleToken,
        runtime: &Arc<RuntimeCore>,
        parent_kind: UiControlKind,
        child_identity: ObjectIdentity,
    ) -> Result<u64, UiControlApiError> {
        let child_kind = runtime
            .ui_control_kind(child_identity)
            .map_err(|error| match error {
                RuntimePropertyError::WrongThread => UiControlApiError::WrongThread,
                RuntimePropertyError::Closed => UiControlApiError::Closed,
                RuntimePropertyError::Store(UiObjectStoreError::OutOfMemory) => {
                    UiControlApiError::OutOfMemory
                }
                RuntimePropertyError::Store(UiObjectStoreError::InvalidIdentity)
                | RuntimePropertyError::WrongObjectState => UiControlApiError::InvalidHandle,
                _ => UiControlApiError::Internal,
            })?;
        let controls = ui_control_handles();
        let mut owners = controls
            .owners
            .lock()
            .map_err(|_| UiControlApiError::Internal)?;
        owners
            .try_reserve(1)
            .map_err(|_| UiControlApiError::OutOfMemory)?;
        let token = runtime
            .tree_child_handle_of_kinds(
                &controls.registry,
                parent_token,
                parent_kind,
                child_identity,
                child_kind,
            )
            .map_err(map_ui_control_tree_error)?;
        owners.insert(
            token,
            UiControlHandleEntry {
                runtime: Arc::clone(runtime),
                kind: child_kind,
                identity: child_identity,
            },
        );
        Ok(token.into_abi())
    }

    pub fn window_content(window: u64) -> Result<Option<u64>, UiControlApiError> {
        let (token, runtime, _) = validate_ui_control(window, UiControlKind::Window)?;
        let count = runtime
            .tree_child_count(&ui_control_handles().registry, token, UiControlKind::Window)
            .map_err(map_ui_control_tree_error)?;
        if count == 0 {
            return Ok(None);
        }
        let identity = runtime
            .tree_child_identity_at(
                &ui_control_handles().registry,
                token,
                UiControlKind::Window,
                0,
            )
            .map_err(map_ui_control_tree_error)?;
        tree_child_handle(token, &runtime, UiControlKind::Window, identity).map(Some)
    }

    pub fn window_set_content(window: u64, child: u64) -> Result<(), UiControlApiError> {
        window_set_content_with_peer(window, child, || Ok(()))
    }

    pub(crate) fn window_set_content_with_peer<F>(
        window: u64,
        child: u64,
        prepare_peer: F,
    ) -> Result<(), UiControlApiError>
    where
        F: FnOnce() -> Result<(), ()>,
    {
        let (window_token, runtime, _) = validate_ui_control(window, UiControlKind::Window)?;
        let (child_token, child_runtime, child_kind, _) = ui_control_entry(child)?;
        if child_kind == UiControlKind::Window {
            return Err(UiControlApiError::WrongType);
        }
        if !Arc::ptr_eq(&runtime, &child_runtime) {
            return Err(UiControlApiError::WrongRuntime);
        }
        if !runtime.is_ui_thread() {
            return Err(UiControlApiError::WrongThread);
        }
        runtime
            .replace_tree_single_child(
                &ui_control_handles().registry,
                window_token,
                UiControlKind::Window,
                child_token,
                child_kind,
                prepare_peer,
            )
            .map_err(map_ui_control_tree_error)
    }

    pub fn window_clear_content(window: u64) -> Result<(), UiControlApiError> {
        let (token, runtime, _) = validate_ui_control(window, UiControlKind::Window)?;
        let count = runtime
            .tree_child_count(&ui_control_handles().registry, token, UiControlKind::Window)
            .map_err(map_ui_control_tree_error)?;
        if count == 0 {
            return Ok(());
        }
        let child = runtime
            .tree_child_identity_at(
                &ui_control_handles().registry,
                token,
                UiControlKind::Window,
                0,
            )
            .map_err(map_ui_control_tree_error)?;
        runtime
            .detach_tree_child_identity(
                &ui_control_handles().registry,
                token,
                UiControlKind::Window,
                child,
            )
            .map_err(map_ui_control_tree_error)
    }

    pub fn stack_panel_insert(
        panel: u64,
        index: usize,
        child: u64,
    ) -> Result<(), UiControlApiError> {
        let (panel_token, runtime, _) = validate_ui_control(panel, UiControlKind::StackPanel)?;
        let (child_token, child_runtime, child_kind, _) = ui_control_entry(child)?;
        if child_kind == UiControlKind::Window {
            return Err(UiControlApiError::WrongType);
        }
        if !Arc::ptr_eq(&runtime, &child_runtime) {
            return Err(UiControlApiError::WrongRuntime);
        }
        runtime
            .attach_tree_child_at(
                &ui_control_handles().registry,
                panel_token,
                UiControlKind::StackPanel,
                child_token,
                child_kind,
                index,
                || Ok(()),
            )
            .map_err(map_ui_control_tree_error)
    }

    pub fn stack_panel_remove(panel: u64, index: usize) -> Result<(), UiControlApiError> {
        let (token, runtime, _) = validate_ui_control(panel, UiControlKind::StackPanel)?;
        let child = runtime
            .tree_child_identity_at(
                &ui_control_handles().registry,
                token,
                UiControlKind::StackPanel,
                index,
            )
            .map_err(map_ui_control_tree_error)?;
        runtime
            .detach_tree_child_identity(
                &ui_control_handles().registry,
                token,
                UiControlKind::StackPanel,
                child,
            )
            .map_err(map_ui_control_tree_error)
    }

    pub fn container_remove(container: u64, child: u64) -> Result<(), UiControlApiError> {
        let (container_token, runtime, container_identity) =
            validate_ui_control(container, UiControlKind::StackPanel)?;
        let (child_token, child_runtime, child_kind, child_identity) = ui_control_entry(child)?;
        if child_kind == UiControlKind::Window {
            return Err(UiControlApiError::WrongType);
        }
        if !Arc::ptr_eq(&runtime, &child_runtime) {
            return Err(UiControlApiError::WrongRuntime);
        }
        ui_control_handles()
            .registry
            .validate(child_token, &child_kind, &runtime.identity())
            .map_err(map_ui_control_registry_error)?;
        if runtime
            .tree_parent_identity(child_identity)
            .map_err(map_ui_control_tree_error)?
            != Some(container_identity)
        {
            return Err(UiControlApiError::InvalidArgument);
        }
        runtime
            .detach_tree_child_identity(
                &ui_control_handles().registry,
                container_token,
                UiControlKind::StackPanel,
                child_identity,
            )
            .map_err(map_ui_control_tree_error)
    }

    pub fn stack_panel_move(
        panel: u64,
        from_index: usize,
        to_index: usize,
    ) -> Result<(), UiControlApiError> {
        let (token, runtime, _) = validate_ui_control(panel, UiControlKind::StackPanel)?;
        runtime
            .tree_move_child(
                &ui_control_handles().registry,
                token,
                UiControlKind::StackPanel,
                from_index,
                to_index,
            )
            .map_err(map_ui_control_tree_error)
    }

    pub fn stack_panel_child_count(panel: u64) -> Result<usize, UiControlApiError> {
        let (token, runtime, _) = validate_ui_control(panel, UiControlKind::StackPanel)?;
        runtime
            .tree_child_count(
                &ui_control_handles().registry,
                token,
                UiControlKind::StackPanel,
            )
            .map_err(map_ui_control_tree_error)
    }

    pub fn stack_panel_child_at(panel: u64, index: usize) -> Result<u64, UiControlApiError> {
        let (token, runtime, _) = validate_ui_control(panel, UiControlKind::StackPanel)?;
        let identity = runtime
            .tree_child_identity_at(
                &ui_control_handles().registry,
                token,
                UiControlKind::StackPanel,
                index,
            )
            .map_err(map_ui_control_tree_error)?;
        tree_child_handle(token, &runtime, UiControlKind::StackPanel, identity)
    }

    fn map_ui_control_property_error(error: RuntimePropertyError) -> UiControlApiError {
        match error {
            RuntimePropertyError::WrongThread => UiControlApiError::WrongThread,
            RuntimePropertyError::Closed => UiControlApiError::Closed,
            RuntimePropertyError::Store(UiObjectStoreError::OutOfMemory)
            | RuntimePropertyError::NotificationQueueOutOfMemory => UiControlApiError::OutOfMemory,
            RuntimePropertyError::Registry(error) => map_ui_control_registry_error(error),
            RuntimePropertyError::WrongObjectState => UiControlApiError::InvalidArgument,
            _ => UiControlApiError::Internal,
        }
    }

    pub fn stack_panel_orientation(panel: u64) -> Result<u32, UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(panel, UiControlKind::StackPanel)?;
        runtime
            .stack_panel_config(identity)
            .map(|(orientation, _)| orientation)
            .map_err(map_ui_control_property_error)
    }

    pub fn stack_panel_set_orientation(
        panel: u64,
        orientation: u32,
    ) -> Result<(), UiControlApiError> {
        if !matches!(orientation, 1 | 2) {
            return Err(UiControlApiError::InvalidArgument);
        }
        let (_, runtime, identity) = validate_ui_control(panel, UiControlKind::StackPanel)?;
        runtime
            .stack_panel_set_orientation(identity, orientation)
            .map(|_| ())
            .map_err(map_ui_control_property_error)
    }

    pub fn stack_panel_child_align(panel: u64, child: u64) -> Result<u32, UiControlApiError> {
        let (_, runtime, panel_identity) = validate_ui_control(panel, UiControlKind::StackPanel)?;
        let (child_token, child_runtime, child_kind, child_identity) = ui_control_entry(child)?;
        if child_kind == UiControlKind::Window {
            return Err(UiControlApiError::WrongType);
        }
        if !Arc::ptr_eq(&runtime, &child_runtime) {
            return Err(UiControlApiError::WrongRuntime);
        }
        ui_control_handles()
            .registry
            .validate(child_token, &child_kind, &runtime.identity())
            .map_err(map_ui_control_registry_error)?;
        runtime
            .stack_panel_child_align(panel_identity, child_identity)
            .map_err(map_ui_control_property_error)
    }

    pub fn stack_panel_set_child_align(
        panel: u64,
        child: u64,
        align: u32,
    ) -> Result<(), UiControlApiError> {
        if !matches!(align, 1..=4) {
            return Err(UiControlApiError::InvalidArgument);
        }
        let (_, runtime, panel_identity) = validate_ui_control(panel, UiControlKind::StackPanel)?;
        let (child_token, child_runtime, child_kind, child_identity) = ui_control_entry(child)?;
        if child_kind == UiControlKind::Window {
            return Err(UiControlApiError::WrongType);
        }
        if !Arc::ptr_eq(&runtime, &child_runtime) {
            return Err(UiControlApiError::WrongRuntime);
        }
        ui_control_handles()
            .registry
            .validate(child_token, &child_kind, &runtime.identity())
            .map_err(map_ui_control_registry_error)?;
        runtime
            .stack_panel_set_child_align(panel_identity, child_identity, align)
            .map(|_| ())
            .map_err(map_ui_control_property_error)
    }

    pub fn ui_control_layout_dirty(control: u64) -> Result<bool, UiControlApiError> {
        let (_, runtime, _, identity) = ui_control_entry(control)?;
        if !runtime.is_ui_thread() {
            return Err(UiControlApiError::WrongThread);
        }
        runtime
            .ui_control_layout_dirty(identity)
            .map_err(map_ui_control_property_error)
    }

    pub fn ui_control_layout_clean(control: u64) -> Result<(), UiControlApiError> {
        let (_, runtime, _, identity) = ui_control_entry(control)?;
        if !runtime.is_ui_thread() {
            return Err(UiControlApiError::WrongThread);
        }
        runtime
            .ui_control_layout_clean(identity)
            .map_err(map_ui_control_property_error)
    }

    pub fn widget_request_focus(widget: u64) -> Result<(), UiControlApiError> {
        let (_, runtime, kind, identity) = ui_control_entry(widget)?;
        if !matches!(kind, UiControlKind::TextBox | UiControlKind::Button) {
            return Err(UiControlApiError::WrongType);
        }
        runtime
            .widget_request_focus(identity)
            .map_err(map_ui_control_property_error)
    }

    pub fn widget_has_focus(widget: u64) -> Result<bool, UiControlApiError> {
        let (_, runtime, kind, identity) = ui_control_entry(widget)?;
        if !matches!(kind, UiControlKind::TextBox | UiControlKind::Button) {
            return Err(UiControlApiError::WrongType);
        }
        if !runtime.is_ui_thread() {
            return Err(UiControlApiError::WrongThread);
        }
        runtime
            .focus_state()
            .map(|(_, focused)| focused == Some(identity))
            .map_err(map_ui_control_property_error)
    }

    pub fn window_open(window: u64) -> Result<u64, UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(window, UiControlKind::Window)?;
        runtime
            .window_open(identity)
            .map_err(map_window_lifecycle_error)
    }

    /// Internal Core/Renderer bridge; not part of the public C ABI.
    pub fn window_submit_frame(
        window: u64,
        scene: Arc<SceneSnapshot>,
    ) -> Result<FrameTicket, UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(window, UiControlKind::Window)?;
        runtime
            .window_submit_frame(identity, scene)
            .map_err(map_window_lifecycle_error)
    }

    /// Internal observation hook used by Core-side integration tests.
    pub fn window_frame_phase(window: u64) -> Result<FramePhase, UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(window, UiControlKind::Window)?;
        runtime
            .window_frame_phase(identity)
            .map_err(map_window_lifecycle_error)
    }

    pub fn window_close(window: u64) -> Result<(), UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(window, UiControlKind::Window)?;
        runtime
            .window_close(identity)
            .map_err(map_window_lifecycle_error)
    }

    pub fn window_platform_activation(window: u64, active: bool) -> Result<(), UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(window, UiControlKind::Window)?;
        runtime
            .window_set_active(identity, active)
            .map_err(map_ui_control_property_error)
    }

    pub fn window_close_intent(
        window: u64,
        callback: Option<Box<dyn FnOnce(u64, u64) -> CloseDecision>>,
    ) -> Result<Option<(u64, u64)>, UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(window, UiControlKind::Window)?;
        runtime
            .window_close_intent(identity, callback)
            .map_err(map_window_lifecycle_error)
    }

    /// Dispatches a native close intent through the Window's active close subscription.
    pub fn window_close_intent_subscribed(
        window: u64,
    ) -> Result<Option<(u64, u64)>, UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(window, UiControlKind::Window)?;
        let subscription = runtime
            .snapshot_window_close_subscription(identity)
            .map_err(map_ui_control_property_error)?;
        let Some(subscription) = subscription else {
            return runtime
                .window_close_intent(identity, None)
                .map_err(map_window_lifecycle_error);
        };
        let callback_runtime = Arc::clone(&runtime);
        runtime
            .window_close_intent(
                identity,
                Some(Box::new(move |request, generation| {
                    let state = Arc::new(CloseRequestState {
                        callback_active: AtomicBool::new(true),
                        owned_handles: AtomicUsize::new(0),
                        release_requested: AtomicBool::new(false),
                        resolved: AtomicBool::new(false),
                        decision: Mutex::new(None),
                    });
                    let borrowed_token = crate::handle_registry::issue_handle_token();
                    let Some(borrowed_token) = borrowed_token else {
                        return CloseDecision::Accept;
                    };
                    let entry = CloseRequestHandleEntry {
                        runtime: Arc::clone(&callback_runtime),
                        identity,
                        request,
                        generation,
                        state: Arc::clone(&state),
                        owned: false,
                    };
                    let inserted = close_request_handles()
                        .lock()
                        .map(|mut handles| {
                            if handles.try_reserve(1).is_err() {
                                false
                            } else {
                                handles.insert(borrowed_token, entry);
                                true
                            }
                        })
                        .unwrap_or(false);
                    if !inserted {
                        return CloseDecision::Accept;
                    }
                    CURRENT_WINDOW_CLOSE_REQUEST
                        .with(|current| current.set(borrowed_token.into_abi()));
                    let _ = subscription.invoke();
                    CURRENT_WINDOW_CLOSE_REQUEST.with(|current| current.set(0));
                    state.callback_active.store(false, Ordering::Release);
                    if let Ok(mut handles) = close_request_handles().lock() {
                        handles.remove(&borrowed_token);
                    }
                    if state.release_requested.load(Ordering::Acquire) {
                        CloseDecision::Decline
                    } else {
                        let decision = state
                            .decision
                            .lock()
                            .map(|decision| decision.unwrap_or(CloseDecision::Accept))
                            .unwrap_or(CloseDecision::Decline);
                        if decision != CloseDecision::Defer {
                            state.resolved.store(true, Ordering::Release);
                        }
                        decision
                    }
                })),
            )
            .map_err(map_window_lifecycle_error)
    }

    /// Registers the Window's single close-request callback using the shared Subscription owner.
    pub fn window_close_request_subscribe(
        window: u64,
        callback: PostCallback,
        userdata: usize,
        destroy: DestroyCallback,
    ) -> Result<u64, UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(window, UiControlKind::Window)?;
        let mut subscriptions = subscription_handles()
            .lock()
            .map_err(|_| UiControlApiError::Internal)?;
        subscriptions
            .try_reserve(1)
            .map_err(|_| UiControlApiError::OutOfMemory)?;
        let token =
            crate::handle_registry::issue_handle_token().ok_or(UiControlApiError::Internal)?;
        // SAFETY: the ABI caller transfers callback/userdata/destroy ownership on success.
        let subscription =
            Arc::new(unsafe { Subscription::new(callback, userdata as *mut c_void, destroy) });
        runtime
            .register_subscription_owner(identity, &subscription)
            .map_err(map_ui_control_property_error)?;
        subscriptions.insert(
            token,
            Arc::new(SubscriptionHandleEntry {
                runtime,
                owner: SubscriptionOwner::Window(identity),
                subscription,
            }),
        );
        Ok(token.into_abi())
    }

    /// Returns the borrowed request handle currently passed to a close callback, or zero.
    pub fn window_close_request_current() -> u64 {
        CURRENT_WINDOW_CLOSE_REQUEST.with(std::cell::Cell::get)
    }

    /// Explicitly defers the borrowed request received by the active close callback.
    pub fn window_close_request_defer(request: u64) -> Result<u64, UiControlApiError> {
        let token = HandleToken::from_abi(request).ok_or(UiControlApiError::InvalidHandle)?;
        let mut handles = close_request_handles()
            .lock()
            .map_err(|_| UiControlApiError::Internal)?;
        let entry = handles
            .get(&token)
            .map(|entry| CloseRequestHandleEntry {
                runtime: Arc::clone(&entry.runtime),
                identity: entry.identity,
                request: entry.request,
                generation: entry.generation,
                state: Arc::clone(&entry.state),
                owned: entry.owned,
            })
            .ok_or(UiControlApiError::InvalidHandle)?;
        if entry.owned
            || !entry.state.callback_active.load(Ordering::Acquire)
            || entry.state.resolved.load(Ordering::Acquire)
            || CURRENT_WINDOW_CLOSE_REQUEST.with(std::cell::Cell::get) != request
        {
            return Err(UiControlApiError::InvalidHandle);
        }
        if !entry.runtime.is_ui_thread() {
            return Err(UiControlApiError::WrongThread);
        }
        handles
            .try_reserve(1)
            .map_err(|_| UiControlApiError::OutOfMemory)?;
        let owned_token =
            crate::handle_registry::issue_handle_token().ok_or(UiControlApiError::Internal)?;
        let mut decision = entry
            .state
            .decision
            .lock()
            .map_err(|_| UiControlApiError::Internal)?;
        entry.state.owned_handles.fetch_add(1, Ordering::AcqRel);
        *decision = Some(CloseDecision::Defer);
        drop(decision);
        let owned = CloseRequestHandleEntry {
            runtime: entry.runtime,
            identity: entry.identity,
            request: entry.request,
            generation: entry.generation,
            state: entry.state,
            owned: true,
        };
        handles.insert(owned_token, owned);
        Ok(owned_token.into_abi())
    }

    /// Resolves an active borrowed request or a previously deferred owned request.
    pub fn window_close_request_resolve(
        request: u64,
        decision: u32,
    ) -> Result<(), UiControlApiError> {
        let decision = match decision {
            1 => CloseDecision::Accept,
            2 => CloseDecision::Decline,
            _ => return Err(UiControlApiError::InvalidArgument),
        };
        let token = HandleToken::from_abi(request).ok_or(UiControlApiError::InvalidHandle)?;
        let entry = close_request_handles()
            .lock()
            .map_err(|_| UiControlApiError::Internal)?
            .get(&token)
            .map(|entry| CloseRequestHandleEntry {
                runtime: Arc::clone(&entry.runtime),
                identity: entry.identity,
                request: entry.request,
                generation: entry.generation,
                state: Arc::clone(&entry.state),
                owned: entry.owned,
            })
            .ok_or(UiControlApiError::InvalidHandle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(UiControlApiError::WrongThread);
        }
        if entry.state.callback_active.load(Ordering::Acquire) {
            if entry.state.resolved.load(Ordering::Acquire) {
                return Err(UiControlApiError::InvalidArgument);
            }
            *entry
                .state
                .decision
                .lock()
                .map_err(|_| UiControlApiError::Internal)? = Some(decision);
            entry.state.resolved.store(true, Ordering::Release);
            return Ok(());
        }
        if !entry.owned || entry.state.resolved.load(Ordering::Acquire) {
            return Err(UiControlApiError::InvalidHandle);
        }
        entry
            .runtime
            .window_close_resolve(entry.identity, entry.request, entry.generation, decision)
            .map_err(map_window_lifecycle_error)?;
        entry.state.resolved.store(true, Ordering::Release);
        Ok(())
    }

    fn window_close_request_release_token(token: HandleToken) -> Result<bool, ShutdownApiError> {
        let mut handles = close_request_handles()
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?;
        let Some(entry) = handles.get(&token) else {
            return Ok(false);
        };
        if !entry.owned {
            return Err(ShutdownApiError::InvalidHandle);
        }
        let entry = handles.remove(&token).ok_or(ShutdownApiError::Internal)?;
        drop(handles);
        let remaining = entry.state.owned_handles.fetch_sub(1, Ordering::AcqRel) - 1;
        if remaining == 0 && !entry.state.resolved.load(Ordering::Acquire) {
            if entry.state.callback_active.load(Ordering::Acquire) {
                entry.state.release_requested.store(true, Ordering::Release);
                *entry
                    .state
                    .decision
                    .lock()
                    .map_err(|_| ShutdownApiError::Internal)? = Some(CloseDecision::Decline);
            } else {
                let _ = entry.runtime.window_close_request_release(
                    entry.identity,
                    entry.request,
                    entry.generation,
                );
            }
        }
        Ok(true)
    }

    pub fn window_close_request_resolve_by_ids(
        window: u64,
        request: u64,
        generation: u64,
        decision: CloseDecision,
    ) -> Result<(), UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(window, UiControlKind::Window)?;
        runtime
            .window_close_resolve(identity, request, generation, decision)
            .map_err(map_window_lifecycle_error)
    }

    pub fn window_close_request_release_by_ids(
        window: u64,
        request: u64,
        generation: u64,
    ) -> Result<(), UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(window, UiControlKind::Window)?;
        runtime
            .window_close_request_release(identity, request, generation)
            .map_err(map_window_lifecycle_error)
    }

    #[cfg(test)]
    pub(crate) fn window_install_test_peer(
        window: u64,
        peer: Arc<dyn crate::ui_window::WindowPeerAdapter>,
    ) -> Result<(), UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(window, UiControlKind::Window)?;
        runtime
            .window_set_peer(identity, peer)
            .map_err(map_ui_control_property_error)
    }

    pub fn ui_control_create_window(runtime: u64) -> Result<u64, UiControlApiError> {
        create_ui_control(runtime, UiControlKind::Window, None)
    }

    pub fn ui_control_create_stack_panel(runtime: u64) -> Result<u64, UiControlApiError> {
        create_ui_control(runtime, UiControlKind::StackPanel, None)
    }

    pub fn ui_control_create_text(runtime: u64, content: &[u8]) -> Result<u64, UiControlApiError> {
        create_ui_control(runtime, UiControlKind::Text, Some(content))
    }

    pub fn ui_control_create_text_box(
        runtime: u64,
        value: &[u8],
    ) -> Result<u64, UiControlApiError> {
        create_ui_control(runtime, UiControlKind::TextBox, Some(value))
    }

    pub fn ui_control_create_button(runtime: u64, label: &[u8]) -> Result<u64, UiControlApiError> {
        create_ui_control(runtime, UiControlKind::Button, Some(label))
    }

    pub fn ui_control_is_enabled_property(control: u64) -> Result<u64, UiControlApiError> {
        ui_control_property(control, UiControlProperty::IsEnabled)
    }

    pub fn ui_control_accessible_name_property(control: u64) -> Result<u64, UiControlApiError> {
        ui_control_property(control, UiControlProperty::AccessibleName)
    }

    pub fn ui_control_window_title_property(control: u64) -> Result<u64, UiControlApiError> {
        ui_control_property(control, UiControlProperty::WindowTitle)
    }

    pub fn ui_control_text_content_property(control: u64) -> Result<u64, UiControlApiError> {
        ui_control_property(control, UiControlProperty::TextContent)
    }

    pub fn ui_control_text_box_value_property(control: u64) -> Result<u64, UiControlApiError> {
        ui_control_property(control, UiControlProperty::TextBoxValue)
    }

    pub fn ui_control_button_label_property(control: u64) -> Result<u64, UiControlApiError> {
        ui_control_property(control, UiControlProperty::ButtonLabel)
    }

    pub fn button_set_command(button: u64, command: u64) -> Result<(), UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(button, UiControlKind::Button)?;
        let command_entry = if command == 0 {
            None
        } else {
            let entry = command_entry(command).map_err(|error| match error {
                CommandApiError::InvalidHandle => UiControlApiError::InvalidHandle,
                CommandApiError::WrongType => UiControlApiError::WrongType,
                CommandApiError::WrongRuntime => UiControlApiError::WrongRuntime,
                CommandApiError::WrongThread => UiControlApiError::WrongThread,
                CommandApiError::Closed => UiControlApiError::Closed,
                CommandApiError::OutOfMemory => UiControlApiError::OutOfMemory,
                _ => UiControlApiError::Internal,
            })?;
            if !Arc::ptr_eq(&runtime, &entry.runtime) {
                return Err(UiControlApiError::WrongRuntime);
            }
            Some(entry)
        };
        runtime
            .button_set_command_entry(identity, command_entry)
            .map_err(map_ui_control_property_error)
    }

    pub fn button_command(button: u64) -> Result<Option<u64>, UiControlApiError> {
        let (_, runtime, identity) = validate_ui_control(button, UiControlKind::Button)?;
        let Some(entry) = runtime
            .button_command_entry(identity)
            .map_err(map_ui_control_property_error)?
        else {
            return Ok(None);
        };
        let mut commands = command_handles()
            .lock()
            .map_err(|_| UiControlApiError::Internal)?;
        commands
            .try_reserve(1)
            .map_err(|_| UiControlApiError::OutOfMemory)?;
        let token =
            crate::handle_registry::issue_handle_token().ok_or(UiControlApiError::Internal)?;
        commands.insert(token, entry);
        Ok(Some(token.into_abi()))
    }

    pub fn command_state_property_create(runtime: u64) -> Result<u64, ShutdownApiError> {
        register_read_only_property(
            runtime,
            TypedValue::I64(crate::command::CommandState::Idle as i64),
        )
    }

    pub fn command_can_execute_property_create(
        runtime: u64,
        eligible: bool,
    ) -> Result<u64, ShutdownApiError> {
        register_read_only_property(runtime, TypedValue::Bool(eligible))
    }

    pub fn command_properties_update(
        runtime_handle: u64,
        state_property: u64,
        can_execute_property: u64,
        state_value: i64,
        can_execute_value: bool,
    ) -> Result<(), PropertyApiError> {
        let runtime = resolve_runtime(runtime_handle).map_err(|error| match error {
            ShutdownApiError::InvalidHandle => PropertyApiError::InvalidHandle,
            ShutdownApiError::WrongThread => PropertyApiError::WrongThread,
            ShutdownApiError::Closed => PropertyApiError::Closed,
            ShutdownApiError::OutOfMemory => PropertyApiError::OutOfMemory,
            _ => PropertyApiError::Internal,
        })?;
        let state_token =
            HandleToken::from_abi(state_property).ok_or(PropertyApiError::InvalidHandle)?;
        let can_execute_token =
            HandleToken::from_abi(can_execute_property).ok_or(PropertyApiError::InvalidHandle)?;
        let handles = property_handles();
        let owners = handles
            .owners
            .lock()
            .map_err(|_| PropertyApiError::Internal)?;
        if owners
            .get(&state_token)
            .is_none_or(|owner| !Arc::ptr_eq(owner, &runtime))
            || owners
                .get(&can_execute_token)
                .is_none_or(|owner| !Arc::ptr_eq(owner, &runtime))
        {
            return Err(PropertyApiError::InvalidHandle);
        }
        drop(owners);
        runtime
            .apply_command_properties_in_host_event(
                &handles.registry,
                state_token,
                can_execute_token,
                state_value,
                can_execute_value,
            )
            .map_err(|error| match error {
                RuntimeCoreError::Property(RuntimePropertyError::Unsupported) => {
                    PropertyApiError::Unsupported
                }
                RuntimeCoreError::Property(RuntimePropertyError::WrongThread)
                | RuntimeCoreError::WrongThread
                | RuntimeCoreError::Host(crate::runtime::HostRegistrationError::WrongThread) => {
                    PropertyApiError::WrongThread
                }
                RuntimeCoreError::Closed
                | RuntimeCoreError::Property(RuntimePropertyError::Closed)
                | RuntimeCoreError::Host(crate::runtime::HostRegistrationError::InvalidState) => {
                    PropertyApiError::Closed
                }
                RuntimeCoreError::Host(crate::runtime::HostRegistrationError::OutOfMemory)
                | RuntimeCoreError::Property(
                    RuntimePropertyError::NotificationQueueOutOfMemory,
                ) => PropertyApiError::OutOfMemory,
                _ => PropertyApiError::Internal,
            })
    }

    pub fn property_create_bool(runtime: u64, initial: bool) -> Result<u64, ShutdownApiError> {
        register_property(runtime, TypedValue::Bool(initial))
    }

    pub fn property_create_i64(runtime: u64, initial: i64) -> Result<u64, ShutdownApiError> {
        register_property(runtime, TypedValue::I64(initial))
    }

    pub fn property_create_utf8(runtime: u64, initial: &[u8]) -> Result<u64, ShutdownApiError> {
        let value = OwnedUtf8::copy_from_bytes(initial).map_err(|error| match error {
            OwnedUtf8Error::InvalidUtf8 => ShutdownApiError::InvalidArgument,
            OwnedUtf8Error::OutOfMemory => ShutdownApiError::OutOfMemory,
        })?;
        register_property(runtime, TypedValue::Utf8(value))
    }

    pub fn property_create_f64(runtime: u64, initial: f64) -> Result<u64, ShutdownApiError> {
        register_property(runtime, TypedValue::F64(initial))
    }

    pub fn property_create_decimal(
        runtime: u64,
        initial_bits: u64,
    ) -> Result<u64, ShutdownApiError> {
        register_property(
            runtime,
            TypedValue::Decimal(crate::value::DecimalValue::from_canonical_bits(
                initial_bits,
            )),
        )
    }

    /// Registers one Property callback and returns its independently owned public token.
    pub fn property_subscribe(
        property: u64,
        callback: PostCallback,
        userdata: usize,
        destroy: DestroyCallback,
    ) -> Result<u64, PropertyApiError> {
        let token = HandleToken::from_abi(property).ok_or(PropertyApiError::InvalidHandle)?;
        let handles = property_handles();
        let runtime = handles
            .owners
            .lock()
            .map_err(|_| PropertyApiError::Internal)?
            .get(&token)
            .cloned()
            .ok_or(PropertyApiError::InvalidHandle)?;
        if !runtime.is_ui_thread() {
            return Err(PropertyApiError::WrongThread);
        }
        let pin = handles
            .registry
            .validate(token, &PropertyHandleKind::Property, &runtime.identity())
            .map_err(|error| match error {
                RegistryError::InvalidToken
                | RegistryError::WrongKind
                | RegistryError::WrongRuntime => PropertyApiError::InvalidHandle,
                RegistryError::OutOfMemory => PropertyApiError::OutOfMemory,
                _ => PropertyApiError::Internal,
            })?;
        let mut subscriptions = subscription_handles()
            .lock()
            .map_err(|_| PropertyApiError::Internal)?;
        subscriptions
            .try_reserve(1)
            .map_err(|_| PropertyApiError::OutOfMemory)?;
        let subscription_token =
            crate::handle_registry::issue_handle_token().ok_or(PropertyApiError::Internal)?;
        // SAFETY: callback, userdata, and destroy are the ownership tuple supplied by the ABI.
        let subscription =
            Arc::new(unsafe { Subscription::new(callback, userdata as *mut c_void, destroy) });
        runtime
            .register_subscription_owner(pin.identity(), &subscription)
            .map_err(map_property_read_error)?;
        subscriptions.insert(
            subscription_token,
            Arc::new(SubscriptionHandleEntry {
                runtime,
                owner: SubscriptionOwner::Property(pin.identity()),
                subscription,
            }),
        );
        Ok(subscription_token.into_abi())
    }

    fn register_property_validator_entry(
        property: u64,
        callback: CValidatorCallback,
        userdata: usize,
        destroy: DestroyCallback,
        expected_kind: ValueKind,
    ) -> Result<u64, PropertyApiError> {
        let token = HandleToken::from_abi(property).ok_or(PropertyApiError::InvalidHandle)?;
        let handles = property_handles();
        let runtime = handles
            .owners
            .lock()
            .map_err(|_| PropertyApiError::Internal)?
            .get(&token)
            .cloned()
            .ok_or(PropertyApiError::InvalidHandle)?;
        if !runtime.is_ui_thread() {
            return Err(PropertyApiError::WrongThread);
        }
        let pin = handles
            .registry
            .validate(token, &PropertyHandleKind::Property, &runtime.identity())
            .map_err(|error| match error {
                RegistryError::InvalidToken => PropertyApiError::InvalidHandle,
                RegistryError::WrongKind => PropertyApiError::WrongType,
                RegistryError::WrongRuntime => PropertyApiError::InvalidHandle,
                RegistryError::OutOfMemory => PropertyApiError::OutOfMemory,
                _ => PropertyApiError::Internal,
            })?;
        let actual_kind = runtime
            .property_kind(&handles.registry, token)
            .map_err(map_property_read_error)?;
        if actual_kind != expected_kind {
            return Err(PropertyApiError::WrongType);
        }

        let mut registrations = validator_handles()
            .lock()
            .map_err(|_| PropertyApiError::Internal)?;
        registrations
            .try_reserve(1)
            .map_err(|_| PropertyApiError::OutOfMemory)?;
        let registration_token =
            crate::handle_registry::issue_handle_token().ok_or(PropertyApiError::Internal)?;
        let context = Arc::new(CValidatorClosure {
            callback,
            userdata,
            destroy,
            armed: std::sync::atomic::AtomicBool::new(false),
        });
        let invoke_context = Arc::clone(&context);
        let validator: Arc<PropertyValidator> =
            Arc::new(move |candidate| invoke_context.invoke(candidate));
        runtime
            .register_property_validator(&handles.registry, token, Arc::clone(&validator))
            .map_err(map_property_read_error)?;
        context
            .armed
            .store(true, std::sync::atomic::Ordering::Release);
        registrations.insert(
            registration_token,
            Arc::new(ValidatorRegistrationEntry {
                runtime,
                identity: pin.identity(),
                validator: Arc::downgrade(&validator),
                cancelled: std::sync::atomic::AtomicBool::new(false),
            }),
        );
        Ok(registration_token.into_abi())
    }

    pub fn property_validator_register_bool(
        property: u64,
        callback: ValidateBoolCallback,
        userdata: usize,
        destroy: DestroyCallback,
    ) -> Result<u64, PropertyApiError> {
        register_property_validator_entry(
            property,
            CValidatorCallback::Bool(callback),
            userdata,
            destroy,
            ValueKind::Bool,
        )
    }

    pub fn property_validator_register_i64(
        property: u64,
        callback: ValidateI64Callback,
        userdata: usize,
        destroy: DestroyCallback,
    ) -> Result<u64, PropertyApiError> {
        register_property_validator_entry(
            property,
            CValidatorCallback::I64(callback),
            userdata,
            destroy,
            ValueKind::I64,
        )
    }

    pub fn property_validator_register_utf8(
        property: u64,
        callback: ValidateUtf8Callback,
        userdata: usize,
        destroy: DestroyCallback,
    ) -> Result<u64, PropertyApiError> {
        register_property_validator_entry(
            property,
            CValidatorCallback::Utf8(callback),
            userdata,
            destroy,
            ValueKind::Utf8,
        )
    }

    pub fn property_validator_cancel(registration: u64) -> Result<(), PropertyApiError> {
        let token = HandleToken::from_abi(registration).ok_or(PropertyApiError::InvalidHandle)?;
        let entry = validator_handles()
            .lock()
            .map_err(|_| PropertyApiError::Internal)?
            .get(&token)
            .cloned()
            .ok_or(PropertyApiError::InvalidHandle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(PropertyApiError::WrongThread);
        }
        if entry.cancelled.load(std::sync::atomic::Ordering::Acquire) {
            return Ok(());
        }
        if let Some(validator) = entry.validator.upgrade() {
            entry
                .runtime
                .cancel_property_validator(entry.identity, &validator)
                .map_err(map_property_read_error)?;
        }
        entry
            .cancelled
            .store(true, std::sync::atomic::Ordering::Release);
        Ok(())
    }

    /// Explicitly cancels one valid Subscription token on its Runtime UI thread.
    pub fn subscription_cancel(handle: u64) -> Result<(), PropertyApiError> {
        let token = HandleToken::from_abi(handle).ok_or(PropertyApiError::InvalidHandle)?;
        let entry = subscription_handles()
            .lock()
            .map_err(|_| PropertyApiError::Internal)?
            .get(&token)
            .cloned()
            .ok_or(PropertyApiError::InvalidHandle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(PropertyApiError::WrongThread);
        }
        if entry.subscription.is_active() {
            match entry.owner {
                SubscriptionOwner::Property(identity) => {
                    entry
                        .runtime
                        .cancel_subscription_owner(identity, &entry.subscription)
                        .map_err(map_property_read_error)?;
                }
                SubscriptionOwner::Binding(connection) => {
                    entry
                        .runtime
                        .cancel_binding_error_subscription(connection, &entry.subscription)
                        .map_err(|error| match error {
                            RuntimeBindingError::Property(RuntimePropertyError::WrongThread) => {
                                PropertyApiError::WrongThread
                            }
                            _ => PropertyApiError::Internal,
                        })?;
                }
                SubscriptionOwner::Window(identity) => {
                    entry
                        .runtime
                        .cancel_subscription_owner(identity, &entry.subscription)
                        .map_err(map_property_read_error)?;
                }
            }
        }
        Ok(())
    }

    pub fn binding_error_snapshot(handle: u64) -> Result<(u64, u64, u32), BindingApiError> {
        let token = HandleToken::from_abi(handle).ok_or(BindingApiError::InvalidHandle)?;
        let bindings = binding_handles()
            .lock()
            .map_err(|_| BindingApiError::Internal)?;
        let entry = bindings.get(&token).ok_or(BindingApiError::InvalidHandle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(BindingApiError::WrongThread);
        }
        let Some(snapshot) = entry
            .runtime
            .binding_validation_snapshot(entry.connection)
            .map_err(map_binding_runtime_error)?
        else {
            return Ok((0, 0, 0));
        };
        let pin = entry
            .runtime
            .binding_property_pin(entry.connection, snapshot.rejected_property)
            .map_err(map_binding_runtime_error)?;
        let error = issue_validation_error_snapshot(Arc::clone(&snapshot.error))
            .map_err(|_| BindingApiError::OutOfMemory)?;
        let handles = property_handles();
        let mut owners = handles.owners.lock().map_err(|_| {
            let _ = handle_release(error);
            BindingApiError::Internal
        })?;
        if owners.try_reserve(1).is_err() {
            drop(owners);
            let _ = handle_release(error);
            return Err(BindingApiError::OutOfMemory);
        }
        let property_token = match handles.registry.register_from_pin(
            PropertyHandleKind::Property,
            entry.runtime.identity(),
            &pin,
        ) {
            Ok(token) => token,
            Err(error_kind) => {
                drop(owners);
                let _ = handle_release(error);
                return Err(match error_kind {
                    RegistryError::OutOfMemory => BindingApiError::OutOfMemory,
                    RegistryError::WrongRuntime => BindingApiError::WrongRuntime,
                    _ => BindingApiError::Internal,
                });
            }
        };
        owners.insert(property_token, Arc::clone(&entry.runtime));
        Ok((error, property_token.into_abi(), snapshot.direction))
    }

    pub fn binding_subscribe_error(
        handle: u64,
        callback: SubscriptionCallback,
        userdata: usize,
        destroy: DestroyCallback,
    ) -> Result<u64, BindingApiError> {
        let token = HandleToken::from_abi(handle).ok_or(BindingApiError::InvalidHandle)?;
        let bindings = binding_handles()
            .lock()
            .map_err(|_| BindingApiError::Internal)?;
        let entry = bindings.get(&token).ok_or(BindingApiError::InvalidHandle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(BindingApiError::WrongThread);
        }
        let mut subscriptions = subscription_handles()
            .lock()
            .map_err(|_| BindingApiError::Internal)?;
        subscriptions
            .try_reserve(1)
            .map_err(|_| BindingApiError::OutOfMemory)?;
        let subscription_token =
            crate::handle_registry::issue_handle_token().ok_or(BindingApiError::Internal)?;
        // SAFETY: ownership transfers only after Runtime registration succeeds.
        let subscription =
            Arc::new(unsafe { Subscription::new(callback, userdata as *mut c_void, destroy) });
        entry
            .runtime
            .register_binding_error_subscription(entry.connection, &subscription)
            .map_err(map_binding_runtime_error)?;
        subscriptions.insert(
            subscription_token,
            Arc::new(SubscriptionHandleEntry {
                runtime: Arc::clone(&entry.runtime),
                owner: SubscriptionOwner::Binding(entry.connection),
                subscription,
            }),
        );
        Ok(subscription_token.into_abi())
    }

    /// Creates a Property Binding and returns an owned handle for persistent modes.
    pub fn binding_create(source: u64, target: u64, mode: u32) -> Result<u64, BindingApiError> {
        if !matches!(mode, BIND_ONCE | BIND_ONE_WAY | BIND_TWO_WAY) {
            return Err(BindingApiError::InvalidArgument);
        }
        let source_token = HandleToken::from_abi(source).ok_or(BindingApiError::InvalidHandle)?;
        let target_token = HandleToken::from_abi(target).ok_or(BindingApiError::InvalidHandle)?;
        let handles = property_handles();
        let owners = handles
            .owners
            .lock()
            .map_err(|_| BindingApiError::Internal)?;
        let runtime = owners
            .get(&source_token)
            .cloned()
            .ok_or(BindingApiError::InvalidHandle)?;
        let target_runtime = owners
            .get(&target_token)
            .ok_or(BindingApiError::InvalidHandle)?;
        if !Arc::ptr_eq(&runtime, target_runtime) {
            return Err(BindingApiError::WrongRuntime);
        }
        drop(owners);
        if !runtime.is_ui_thread() {
            return Err(BindingApiError::WrongThread);
        }
        runtime
            .validate_binding_initial_candidates(
                &handles.registry,
                source_token,
                target_token,
                mode == BIND_TWO_WAY,
            )
            .map_err(map_binding_runtime_error)?;
        let mut binding_map = binding_handles()
            .lock()
            .map_err(|_| BindingApiError::Internal)?;
        if mode != BIND_ONCE {
            binding_map
                .try_reserve(1)
                .map_err(|_| BindingApiError::OutOfMemory)?;
        }
        let token = if mode == BIND_ONCE {
            None
        } else {
            Some(crate::handle_registry::issue_handle_token().ok_or(BindingApiError::Internal)?)
        };
        let registry = &handles.registry;
        let created = match mode {
            BIND_ONCE | BIND_ONE_WAY => runtime
                .create_one_way_binding_initial_with_id(registry, source_token, target_token)
                .map_err(map_binding_runtime_error)?,
            BIND_TWO_WAY => runtime
                .create_two_way_binding_initial_with_id(registry, source_token, target_token)
                .map_err(map_binding_runtime_error)?,
            _ => unreachable!("mode checked above"),
        };
        let (_, connection) = created;
        if let Some(token) = token {
            binding_map.insert(
                token,
                BindingHandleEntry {
                    runtime: Arc::clone(&runtime),
                    connection,
                },
            );
            Ok(token.into_abi())
        } else {
            runtime
                .cancel_binding_connection(connection)
                .map_err(map_binding_runtime_error)?;
            Ok(0)
        }
    }

    /// Explicitly cancels one Binding while leaving its owned handle valid for release.
    pub fn binding_cancel(handle: u64) -> Result<(), BindingApiError> {
        let token = HandleToken::from_abi(handle).ok_or(BindingApiError::InvalidHandle)?;
        let bindings = binding_handles()
            .lock()
            .map_err(|_| BindingApiError::Internal)?;
        let entry = bindings.get(&token).ok_or(BindingApiError::InvalidHandle)?;
        if !entry.runtime.is_ui_thread() {
            return Err(BindingApiError::WrongThread);
        }
        let runtime = Arc::clone(&entry.runtime);
        let connection = entry.connection;
        drop(bindings);
        runtime
            .cancel_binding_connection(connection)
            .map(|_| ())
            .map_err(map_binding_runtime_error)
    }

    pub fn property_kind(handle: u64) -> Result<u32, ShutdownApiError> {
        let token = HandleToken::from_abi(handle).ok_or(ShutdownApiError::InvalidHandle)?;
        let handles = property_handles();
        let runtime = handles
            .owners
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?
            .get(&token)
            .cloned()
            .ok_or(ShutdownApiError::InvalidHandle)?;
        let kind =
            runtime
                .property_kind(&handles.registry, token)
                .map_err(|error| match error {
                    RuntimePropertyError::WrongThread => ShutdownApiError::WrongThread,
                    RuntimePropertyError::Closed => ShutdownApiError::Closed,
                    RuntimePropertyError::Registry(error) => map_registry_error(error),
                    RuntimePropertyError::SynchronizationPoisoned
                    | RuntimePropertyError::Store(_)
                    | RuntimePropertyError::WrongObjectState
                    | RuntimePropertyError::NotificationQueueOutOfMemory
                    | RuntimePropertyError::SubscriptionRegistryOutOfMemory
                    | RuntimePropertyError::Dispatch(_)
                    | RuntimePropertyError::ValueKindMismatch(_)
                    | RuntimePropertyError::BindingGraph(_)
                    | RuntimePropertyError::BindingValueCopy(_)
                    | RuntimePropertyError::BindingPropagationOutOfMemory
                    | RuntimePropertyError::Unsupported
                    | RuntimePropertyError::ValidationFailed(_)
                    | RuntimePropertyError::ValidatorFailed
                    | RuntimePropertyError::ValidatorReentrancy
                    | RuntimePropertyError::ValidatorAlreadyRegistered => {
                        ShutdownApiError::Internal
                    }
                })?;
        Ok(match kind {
            ValueKind::Bool => 1,
            ValueKind::I64 => 2,
            ValueKind::Utf8 => 3,
            ValueKind::F64 => 5,
            ValueKind::Decimal => 6,
        })
    }

    fn map_property_read_error(error: RuntimePropertyError) -> PropertyApiError {
        match error {
            RuntimePropertyError::WrongThread => PropertyApiError::WrongThread,
            RuntimePropertyError::Closed => PropertyApiError::Closed,
            RuntimePropertyError::Unsupported => PropertyApiError::Unsupported,
            RuntimePropertyError::Registry(RegistryError::InvalidToken)
            | RuntimePropertyError::Registry(RegistryError::WrongKind)
            | RuntimePropertyError::Registry(RegistryError::WrongRuntime) => {
                PropertyApiError::InvalidHandle
            }
            RuntimePropertyError::Registry(RegistryError::OutOfMemory) => {
                PropertyApiError::Internal
            }
            RuntimePropertyError::NotificationQueueOutOfMemory
            | RuntimePropertyError::SubscriptionRegistryOutOfMemory => {
                PropertyApiError::OutOfMemory
            }
            RuntimePropertyError::ValidationFailed(error) => {
                PropertyApiError::ValidationFailed(error)
            }
            RuntimePropertyError::ValidatorFailed => PropertyApiError::Internal,
            RuntimePropertyError::Registry(
                RegistryError::TokenExhausted
                | RegistryError::CleanupUnavailable
                | RegistryError::SynchronizationPoisoned
                | RegistryError::ReferenceCountExhausted,
            )
            | RuntimePropertyError::SynchronizationPoisoned
            | RuntimePropertyError::Store(_)
            | RuntimePropertyError::Dispatch(_)
            | RuntimePropertyError::WrongObjectState
            | RuntimePropertyError::ValueKindMismatch(_)
            | RuntimePropertyError::BindingGraph(_)
            | RuntimePropertyError::BindingValueCopy(_)
            | RuntimePropertyError::BindingPropagationOutOfMemory
            | RuntimePropertyError::ValidatorReentrancy => PropertyApiError::Internal,
            RuntimePropertyError::ValidatorAlreadyRegistered => PropertyApiError::InvalidArgument,
        }
    }

    fn inspect_property_value<T>(
        handle: u64,
        inspect: impl FnOnce(&TypedValue) -> Result<T, PropertyApiError>,
    ) -> Result<T, PropertyApiError> {
        let token = HandleToken::from_abi(handle).ok_or(PropertyApiError::InvalidHandle)?;
        let handles = property_handles();
        let runtime = handles
            .owners
            .lock()
            .map_err(|_| PropertyApiError::Internal)?
            .get(&token)
            .cloned()
            .ok_or(PropertyApiError::InvalidHandle)?;
        runtime
            .with_property_value(&handles.registry, token, inspect)
            .map_err(map_property_read_error)?
    }

    pub fn property_get_bool(handle: u64) -> Result<bool, PropertyApiError> {
        inspect_property_value(handle, |value| match value {
            TypedValue::Bool(value) => Ok(*value),
            _ => Err(PropertyApiError::WrongType),
        })
    }

    pub fn property_get_i64(handle: u64) -> Result<i64, PropertyApiError> {
        inspect_property_value(handle, |value| match value {
            TypedValue::I64(value) => Ok(*value),
            _ => Err(PropertyApiError::WrongType),
        })
    }

    pub fn property_get_utf8(
        handle: u64,
        buffer: Option<&mut [u8]>,
    ) -> Result<usize, PropertyApiError> {
        inspect_property_value(handle, move |value| match value {
            TypedValue::Utf8(value) => match value.copy_to_buffer(buffer) {
                Ok(Utf8CopyOutcome::LengthOnly { required }) => Ok(required),
                Ok(Utf8CopyOutcome::Written { length }) => Ok(length),
                Err(error) => Err(PropertyApiError::BufferTooSmall {
                    required: error.required,
                }),
            },
            _ => Err(PropertyApiError::WrongType),
        })
    }

    pub fn property_get_f64(handle: u64) -> Result<f64, PropertyApiError> {
        inspect_property_value(handle, |value| match value {
            TypedValue::F64(value) => Ok(*value),
            _ => Err(PropertyApiError::WrongType),
        })
    }

    pub fn property_get_decimal(handle: u64) -> Result<u64, PropertyApiError> {
        inspect_property_value(handle, |value| match value {
            TypedValue::Decimal(value) => Ok(value.to_canonical_bits()),
            _ => Err(PropertyApiError::WrongType),
        })
    }

    fn command_entries_for_eligibility(
        runtime: &Arc<RuntimeCore>,
        identity: crate::handle_registry::registry::ObjectIdentity,
    ) -> Result<Vec<Arc<CommandHandleEntry>>, PropertyApiError> {
        let commands = command_handles()
            .lock()
            .map_err(|_| PropertyApiError::Internal)?;
        let mut matching = Vec::new();
        matching
            .try_reserve(commands.len())
            .map_err(|_| PropertyApiError::OutOfMemory)?;
        for entry in commands.values() {
            if Arc::ptr_eq(&entry.runtime, runtime)
                && entry.eligibility_identity == Some(identity)
                && !matching.iter().any(|previous| Arc::ptr_eq(previous, entry))
            {
                matching.push(Arc::clone(entry));
            }
        }
        Ok(matching)
    }

    fn map_command_property_error(error: RuntimePropertyError) -> PropertyApiError {
        match error {
            RuntimePropertyError::WrongThread => PropertyApiError::WrongThread,
            RuntimePropertyError::Closed => PropertyApiError::Closed,
            RuntimePropertyError::Unsupported => PropertyApiError::Unsupported,
            RuntimePropertyError::NotificationQueueOutOfMemory => PropertyApiError::OutOfMemory,
            RuntimePropertyError::Registry(RegistryError::InvalidToken) => {
                PropertyApiError::InvalidHandle
            }
            _ => PropertyApiError::Internal,
        }
    }

    fn restore_command_eligibility(updated: &[(Arc<CommandHandleEntry>, bool)]) {
        for (entry, eligible) in updated.iter().rev() {
            if let Ok(snapshot) = entry.command.set_eligible(*eligible) {
                let _ = apply_command_snapshot_in_event(entry, &snapshot);
            }
        }
    }

    fn prepare_command_eligibility(
        entries: &[Arc<CommandHandleEntry>],
    ) -> Result<(), PropertyApiError> {
        for entry in entries {
            if entry.command.snapshot().is_err() {
                continue;
            }
            entry
                .runtime
                .prepare_command_property_update(
                    &property_handles().registry,
                    entry.state_property,
                    entry.can_execute_property,
                )
                .map_err(map_command_property_error)?;
        }
        Ok(())
    }

    fn update_command_eligibility(
        entries: &[Arc<CommandHandleEntry>],
        eligible: bool,
    ) -> Result<Vec<(Arc<CommandHandleEntry>, bool)>, PropertyApiError> {
        let mut updated = Vec::new();
        updated
            .try_reserve(entries.len())
            .map_err(|_| PropertyApiError::OutOfMemory)?;
        for entry in entries {
            let Ok(previous) = entry.command.snapshot() else {
                continue;
            };
            match entry.command.set_eligible(eligible) {
                Ok(snapshot) => {
                    if let Err(error) = apply_command_snapshot_in_event(entry, &snapshot) {
                        restore_command_eligibility(&updated);
                        return Err(match error {
                            CommandApiError::WrongThread => PropertyApiError::WrongThread,
                            CommandApiError::Closed => PropertyApiError::Closed,
                            CommandApiError::OutOfMemory => PropertyApiError::OutOfMemory,
                            _ => PropertyApiError::Internal,
                        });
                    }
                    updated.push((Arc::clone(entry), previous.eligible));
                }
                Err(CommandError::Closed) => {}
                Err(error) => {
                    restore_command_eligibility(&updated);
                    return Err(match map_command_error(error) {
                        CommandApiError::OutOfMemory => PropertyApiError::OutOfMemory,
                        CommandApiError::Closed => PropertyApiError::Closed,
                        _ => PropertyApiError::Internal,
                    });
                }
            }
        }
        Ok(updated)
    }

    fn apply_property_value(handle: u64, candidate: TypedValue) -> Result<(), PropertyApiError> {
        let token = HandleToken::from_abi(handle).ok_or(PropertyApiError::InvalidHandle)?;
        let handles = property_handles();
        let runtime = handles
            .owners
            .lock()
            .map_err(|_| PropertyApiError::Internal)?
            .get(&token)
            .cloned()
            .ok_or(PropertyApiError::InvalidHandle)?;
        match runtime.apply_property_candidate_in_host_event(&handles.registry, token, candidate) {
            Ok(_) => Ok(()),
            Err(RuntimeCoreError::Property(RuntimePropertyError::ValueKindMismatch(_))) => {
                Err(PropertyApiError::WrongType)
            }
            Err(RuntimeCoreError::Property(RuntimePropertyError::ValidationFailed(
                error_handle,
            ))) => Err(PropertyApiError::ValidationFailed(error_handle)),
            Err(RuntimeCoreError::Property(error)) => Err(map_property_read_error(error)),
            Err(RuntimeCoreError::WrongThread)
            | Err(RuntimeCoreError::Host(crate::runtime::HostRegistrationError::WrongThread)) => {
                Err(PropertyApiError::WrongThread)
            }
            Err(RuntimeCoreError::Closed)
            | Err(RuntimeCoreError::Host(crate::runtime::HostRegistrationError::InvalidState)) => {
                Err(PropertyApiError::Closed)
            }
            Err(RuntimeCoreError::Host(crate::runtime::HostRegistrationError::OutOfMemory)) => {
                Err(PropertyApiError::OutOfMemory)
            }
            Err(_) => Err(PropertyApiError::Internal),
        }
    }

    pub fn property_set_bool(handle: u64, value: bool) -> Result<(), PropertyApiError> {
        let token = HandleToken::from_abi(handle).ok_or(PropertyApiError::InvalidHandle)?;
        let handles = property_handles();
        let runtime = handles
            .owners
            .lock()
            .map_err(|_| PropertyApiError::Internal)?
            .get(&token)
            .cloned()
            .ok_or(PropertyApiError::InvalidHandle)?;
        let identity = handles
            .registry
            .validate(token, &PropertyHandleKind::Property, &runtime.identity())
            .map_err(|error| match error {
                RegistryError::WrongRuntime | RegistryError::InvalidToken => {
                    PropertyApiError::InvalidHandle
                }
                RegistryError::WrongKind => PropertyApiError::WrongType,
                _ => PropertyApiError::Internal,
            })?
            .identity();
        let current = runtime
            .read_property(&handles.registry, token)
            .map_err(map_command_property_error)?;
        if !matches!(current, TypedValue::Bool(_)) {
            return Err(PropertyApiError::WrongType);
        }
        let entries = command_entries_for_eligibility(&runtime, identity)?;
        if entries.is_empty() {
            return apply_property_value(handle, TypedValue::Bool(value));
        }
        let scope = runtime.begin_host_event().map_err(|error| match error {
            RuntimeCoreError::WrongThread
            | RuntimeCoreError::Host(crate::runtime::HostRegistrationError::WrongThread) => {
                PropertyApiError::WrongThread
            }
            RuntimeCoreError::Closed
            | RuntimeCoreError::Host(crate::runtime::HostRegistrationError::InvalidState) => {
                PropertyApiError::Closed
            }
            RuntimeCoreError::Host(crate::runtime::HostRegistrationError::OutOfMemory) => {
                PropertyApiError::OutOfMemory
            }
            _ => PropertyApiError::Internal,
        })?;
        if let Err(error) = prepare_command_eligibility(&entries) {
            let _ = runtime.end_host_event(scope);
            return Err(error);
        }
        if let Err(error) = apply_property_value(handle, TypedValue::Bool(value)) {
            let _ = runtime.end_host_event(scope);
            return Err(error);
        }
        match update_command_eligibility(&entries, value) {
            Ok(_) => {}
            Err(error) => {
                let _ = runtime.end_host_event(scope);
                return Err(error);
            }
        }
        let close = runtime.end_host_event(scope).map_err(|error| match error {
            RuntimeCoreError::WrongThread
            | RuntimeCoreError::Host(crate::runtime::HostRegistrationError::WrongThread) => {
                PropertyApiError::WrongThread
            }
            RuntimeCoreError::Closed
            | RuntimeCoreError::Host(crate::runtime::HostRegistrationError::InvalidState) => {
                PropertyApiError::Closed
            }
            RuntimeCoreError::Host(crate::runtime::HostRegistrationError::OutOfMemory) => {
                PropertyApiError::OutOfMemory
            }
            _ => PropertyApiError::Internal,
        });
        close.map(|_| ())
    }

    pub fn property_set_i64(handle: u64, value: i64) -> Result<(), PropertyApiError> {
        apply_property_value(handle, TypedValue::I64(value))
    }

    pub fn property_set_utf8(handle: u64, bytes: &[u8]) -> Result<(), PropertyApiError> {
        let value = OwnedUtf8::copy_from_bytes(bytes).map_err(|error| match error {
            OwnedUtf8Error::InvalidUtf8 => PropertyApiError::InvalidArgument,
            OwnedUtf8Error::OutOfMemory => PropertyApiError::OutOfMemory,
        })?;
        apply_property_value(handle, TypedValue::Utf8(value))
    }

    pub fn property_set_f64(handle: u64, value: f64) -> Result<(), PropertyApiError> {
        apply_property_value(handle, TypedValue::F64(value))
    }

    pub fn property_set_decimal(handle: u64, bits: u64) -> Result<(), PropertyApiError> {
        apply_property_value(
            handle,
            TypedValue::Decimal(DecimalValue::from_canonical_bits(bits)),
        )
    }

    fn property_release(token: HandleToken) -> Result<bool, ShutdownApiError> {
        let handles = property_handles();
        let runtime = handles
            .owners
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?
            .get(&token)
            .cloned();
        let Some(runtime) = runtime else {
            return Ok(false);
        };
        handles
            .registry
            .release(token)
            .map_err(map_registry_error)?;
        // Registry release has committed. Recover a poisoned owner map so a successful release
        // cannot be reported as failed after the token was already consumed.
        let mut owners = handles
            .owners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        owners.remove(&token);
        drop(runtime);
        Ok(true)
    }

    fn ui_control_release(token: HandleToken) -> Result<bool, ShutdownApiError> {
        let handles = ui_control_handles();
        if !handles
            .owners
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?
            .contains_key(&token)
        {
            return Ok(false);
        }
        handles
            .registry
            .release(token)
            .map_err(map_registry_error)?;
        let entry = handles
            .owners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&token);
        if entry.is_none() {
            return Err(ShutdownApiError::Internal);
        }
        drop(entry);
        Ok(true)
    }

    pub fn runtime_set_ui_loop_bridge(
        handle: u64,
        wake: WakeCallback,
        userdata: usize,
        destroy: Option<DestroyCallback>,
    ) -> Result<(), ShutdownApiError> {
        let runtime = resolve_runtime(handle)?;
        let bridge = Arc::new(CAbiUiLoopBridge::new(wake, userdata, destroy));
        let bridge_trait: Arc<dyn UiLoopBridge> = bridge.clone();
        runtime
            .set_ui_loop_bridge(&bridge_trait)
            .map_err(map_runtime_error)?;
        bridge.arm();
        Ok(())
    }

    pub fn runtime_request_shutdown(
        handle: u64,
        mode: u32,
    ) -> Result<(u64, ShutdownApiState), ShutdownApiError> {
        let mode = match mode {
            0 => ShutdownRequestMode::Async,
            1 => ShutdownRequestMode::Sync,
            _ => return Err(ShutdownApiError::InvalidArgument),
        };
        let runtime = resolve_runtime(handle)?;
        let mut handles = request_handles()
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?;
        handles
            .try_reserve(1)
            .map_err(|_| ShutdownApiError::OutOfMemory)?;
        let token =
            crate::handle_registry::issue_handle_token().ok_or(ShutdownApiError::Internal)?;
        handles.insert(token, None);
        drop(handles);

        let result = runtime.request_shutdown(mode);
        let (request, state) = match result {
            Ok((request, state)) => (request, state),
            Err(error) => {
                if let Ok(mut handles) = request_handles().lock() {
                    handles.remove(&token);
                }
                return Err(map_runtime_error(error));
            }
        };
        let state = match state {
            ShutdownRequestOutcome::Pending => ShutdownApiState::Pending,
            ShutdownRequestOutcome::Completed => ShutdownApiState::Completed,
            ShutdownRequestOutcome::HostUnavailable { reason } => {
                ShutdownApiState::HostUnavailable { reason }
            }
        };
        let mut handles = request_handles()
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?;
        let slot = handles.get_mut(&token).ok_or(ShutdownApiError::Internal)?;
        *slot = Some(RequestHandleEntry {
            _runtime: runtime,
            request,
        });
        Ok((token.into_abi(), state))
    }

    pub fn shutdown_request_query(handle: u64) -> Result<ShutdownApiState, ShutdownApiError> {
        let token = HandleToken::from_abi(handle).ok_or(ShutdownApiError::InvalidHandle)?;
        let handles = request_handles()
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?;
        let entry = handles
            .get(&token)
            .and_then(Option::as_ref)
            .ok_or(ShutdownApiError::InvalidHandle)?;
        entry
            .request
            .query()
            .map(|state| match state {
                ShutdownCompletionState::Pending => ShutdownApiState::Pending,
                ShutdownCompletionState::Completed => ShutdownApiState::Completed,
                ShutdownCompletionState::HostUnavailable { reason } => {
                    ShutdownApiState::HostUnavailable { reason }
                }
            })
            .map_err(|_| ShutdownApiError::Internal)
    }

    pub fn runtime_shutdown(handle: u64) -> Result<ShutdownApiState, ShutdownApiError> {
        let runtime = resolve_runtime(handle)?;
        let (_, state) = runtime
            .request_shutdown(ShutdownRequestMode::Sync)
            .map_err(map_runtime_error)?;
        Ok(match state {
            ShutdownRequestOutcome::Pending => ShutdownApiState::Pending,
            ShutdownRequestOutcome::Completed => ShutdownApiState::Completed,
            ShutdownRequestOutcome::HostUnavailable { reason } => {
                ShutdownApiState::HostUnavailable { reason }
            }
        })
    }

    pub fn runtime_host_begin_event(handle: u64) -> Result<u64, ShutdownApiError> {
        resolve_runtime(handle)?
            .begin_host_event()
            .map_err(map_runtime_error)
    }

    pub fn runtime_host_end_event(handle: u64, scope_id: u64) -> Result<bool, ShutdownApiError> {
        resolve_runtime(handle)?
            .end_host_event(scope_id)
            .map_err(map_runtime_error)
    }

    pub fn runtime_host_stopped(handle: u64, reason: u32) -> Result<(), ShutdownApiError> {
        resolve_runtime(handle)?
            .notify_host_stopped(reason)
            .map_err(map_runtime_error)
    }

    pub fn runtime_dispatch_turn(handle: u64) -> Result<(), ShutdownApiError> {
        resolve_runtime(handle)?
            .dispatch_turn()
            .map_err(map_runtime_error)
    }

    pub fn runtime_post(
        handle: u64,
        callback: PostCallback,
        userdata: usize,
        destroy: DestroyCallback,
    ) -> Result<(), ShutdownApiError> {
        let runtime = resolve_runtime(handle)?;
        let dispatcher = runtime
            .dispatcher()
            .map_err(map_runtime_error)?
            .ok_or(ShutdownApiError::Unsupported)?;
        let work = PostedWork::new(
            Box::new(move || {
                // SAFETY: successful post transfers the callback and userdata to Core.
                unsafe { callback(userdata as *mut c_void) };
            }),
            Box::new(move || {
                // SAFETY: destroy is called once after execution or cancellation.
                unsafe { destroy(userdata as *mut c_void) };
            }),
        );
        dispatcher.post(work).map_err(|error| match error.kind {
            PostErrorKind::SynchronizationPoisoned => ShutdownApiError::Internal,
            PostErrorKind::OutOfMemory => ShutdownApiError::OutOfMemory,
            PostErrorKind::Closed => ShutdownApiError::Closed,
        })
    }

    pub fn handle_release(handle: u64) -> Result<(), ShutdownApiError> {
        let token = HandleToken::from_abi(handle).ok_or(ShutdownApiError::InvalidHandle)?;
        if window_close_request_release_token(token)? {
            return Ok(());
        }
        if error_handles()
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?
            .remove(&token)
            .is_some()
        {
            return Ok(());
        }
        if validator_handles()
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?
            .remove(&token)
            .is_some()
        {
            return Ok(());
        }
        {
            let mut requests = request_handles()
                .lock()
                .map_err(|_| ShutdownApiError::Internal)?;
            if requests.remove(&token).is_some() {
                return Ok(());
            }
        }
        if subscription_handles()
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?
            .remove(&token)
            .is_some()
        {
            return Ok(());
        }
        if binding_handles()
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?
            .remove(&token)
            .is_some()
        {
            return Ok(());
        }
        let command = command_handles()
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?
            .remove(&token);
        if command.is_some() {
            drop(command);
            return Ok(());
        }
        if command_cancel_tokens()
            .lock()
            .map_err(|_| ShutdownApiError::Internal)?
            .remove(&token)
            .is_some()
        {
            return Ok(());
        }
        if property_release(token)? {
            return Ok(());
        }
        if ui_control_release(token)? {
            return Ok(());
        }
        process_runtime_handles()
            .remove(token)
            .map_err(|error| match error {
                RuntimeHandleError::InvalidToken => ShutdownApiError::InvalidHandle,
                RuntimeHandleError::SynchronizationPoisoned
                | RuntimeHandleError::OutOfMemory
                | RuntimeHandleError::TokenUnavailable => ShutdownApiError::Internal,
            })
    }
}

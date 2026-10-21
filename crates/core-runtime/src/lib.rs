//! Internal Rust Core crate. Product implementation starts in subsequent units.
//!
//! Language wrappers must use the versioned C ABI rather than depend on this crate.

#[allow(dead_code)]
mod binding;
mod dispatcher;
mod handle_registry;
// Runtime-owned UI state and the initial internal Property integration.
pub mod numeric;
#[allow(dead_code)]
mod object_store;
mod property;
mod runtime;
mod runtime_handles;
// Typed value semantics back the internal Property state; ABI validation remains a later unit.
#[allow(dead_code)]
mod value;

/// Narrow support surface used by the C ABI crate while the Core remains private to wrappers.
#[doc(hidden)]
pub mod abi_support {
    use crate::dispatcher::{PostErrorKind, PostedWork, UiLoopBridge, WakePriority};
    use crate::handle_registry::HandleToken;
    use crate::handle_registry::registry::{HandleRegistry, RegistryError};
    use crate::object_store::UiObjectStoreError;
    use crate::property::{PropertyHandleKind, Subscription};
    use crate::runtime::{
        RuntimeBindingError, RuntimeCore, RuntimeCoreError, RuntimeIdentity,
        RuntimeObjectRegistrationError, RuntimePropertyError, ShutdownCompletionState,
        ShutdownRequest, ShutdownRequestMode, ShutdownRequestOutcome,
    };
    use crate::runtime_handles::{RuntimeHandleError, process_runtime_handles};
    use crate::value::{
        DecimalValue, OwnedUtf8, OwnedUtf8Error, TypedValue, Utf8CopyOutcome, ValueKind,
    };
    use std::collections::HashMap;
    use std::ffi::c_void;
    use std::sync::{Arc, Condvar, Mutex, OnceLock};

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

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum PropertyApiError {
        InvalidHandle,
        InvalidArgument,
        WrongType,
        WrongThread,
        Closed,
        ValidationFailed,
        OutOfMemory,
        BufferTooSmall { required: usize },
        Internal,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum BindingApiError {
        InvalidHandle,
        InvalidArgument,
        WrongType,
        WrongRuntime,
        WrongThread,
        Closed,
        OutOfMemory,
        Internal,
    }

    pub const BIND_ONCE: u32 = 1;
    pub const BIND_ONE_WAY: u32 = 2;
    pub const BIND_TWO_WAY: u32 = 3;

    pub type WakeCallback = unsafe extern "C" fn(*mut c_void, u32);
    pub type DestroyCallback = unsafe extern "C" fn(*mut c_void);
    pub type PostCallback = unsafe extern "C" fn(*mut c_void);

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
            if call_destroy && let Some(destroy) = self.destroy {
                // SAFETY: destroy is called exactly once after in-flight wake callbacks drain.
                unsafe { destroy(self.userdata as *mut c_void) };
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

    struct SubscriptionHandleEntry {
        runtime: Arc<RuntimeCore>,
        identity: crate::handle_registry::registry::ObjectIdentity,
        subscription: Arc<Subscription>,
    }

    struct BindingHandleEntry {
        runtime: Arc<RuntimeCore>,
        connection: crate::binding::BindingConnectionId,
    }

    static PROPERTY_HANDLES: OnceLock<PropertyHandles> = OnceLock::new();
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

    fn subscription_handles() -> &'static Mutex<HashMap<HandleToken, Arc<SubscriptionHandleEntry>>>
    {
        SUBSCRIPTION_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
    }

    fn binding_handles() -> &'static Mutex<HashMap<HandleToken, BindingHandleEntry>> {
        BINDING_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
    }

    fn map_binding_runtime_error(error: RuntimeBindingError) -> BindingApiError {
        match error {
            RuntimeBindingError::Property(RuntimePropertyError::WrongThread) => {
                BindingApiError::WrongThread
            }
            RuntimeBindingError::Property(RuntimePropertyError::Closed) => BindingApiError::Closed,
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
        let runtime = Arc::new(RuntimeCore::new().map_err(|_| RuntimeCreateError::Internal)?);
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
                identity: pin.identity(),
                subscription,
            }),
        );
        Ok(subscription_token.into_abi())
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
            entry
                .runtime
                .cancel_subscription_owner(entry.identity, &entry.subscription)
                .map_err(map_property_read_error)?;
        }
        Ok(())
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
        if !runtime.is_ui_thread() {
            return Err(BindingApiError::WrongThread);
        }
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
        entry
            .runtime
            .cancel_binding_connection(entry.connection)
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
                    | RuntimePropertyError::ValidationFailed
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
            RuntimePropertyError::ValidationFailed => PropertyApiError::ValidationFailed,
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
            | RuntimePropertyError::ValidatorReentrancy
            | RuntimePropertyError::ValidatorAlreadyRegistered => PropertyApiError::Internal,
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
        apply_property_value(handle, TypedValue::Bool(value))
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
        if property_release(token)? {
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

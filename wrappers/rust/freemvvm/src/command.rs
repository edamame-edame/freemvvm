use crate::error::UiError;
use crate::handle::{AnyThreadOwned, Owned, PendingToken, RuntimeAnchor, TicketReservation};
use crate::native_api::{self, SharedApi};
use crate::property::{Property, PropertyTag};
use crate::runtime::UiContext;
use std::ffi::c_void;
use std::num::NonZeroU64;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

pub(crate) struct CommandTag;

const REPORT_PENDING: u8 = 0;
const REPORT_POSTING: u8 = 1;
const REPORT_SUBMITTED: u8 = 2;

/// A nonzero identity for one execution of an asynchronous Command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OperationId(NonZeroU64);

impl OperationId {
    /// Returns the underlying Runtime-issued operation identifier.
    pub const fn get(self) -> u64 {
        self.0.get()
    }

    fn from_raw(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }
}

/// The known states exposed by a Command state Property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandState {
    Idle,
    Running,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
    Unknown(i64),
}

impl CommandState {
    /// Preserves future state values instead of collapsing them into a known state.
    pub const fn from_raw(value: i64) -> Self {
        match value {
            freemvvm_sys::COMMAND_STATE_IDLE => Self::Idle,
            freemvvm_sys::COMMAND_STATE_RUNNING => Self::Running,
            freemvvm_sys::COMMAND_STATE_CANCELLING => Self::Cancelling,
            freemvvm_sys::COMMAND_STATE_SUCCEEDED => Self::Succeeded,
            freemvvm_sys::COMMAND_STATE_FAILED => Self::Failed,
            freemvvm_sys::COMMAND_STATE_CANCELLED => Self::Cancelled,
            value => Self::Unknown(value),
        }
    }
}

struct StartState<F> {
    callback: F,
    api: SharedApi,
    anchor: Arc<RuntimeAnchor>,
}

/// A UI-affine owner of a reusable asynchronous Command.
///
/// Dropping the public handle does not close the Core-owned Command. Call [`AsyncCommand::close`]
/// to stop new operations and request cancellation of the active one.
///
/// ```compile_fail
/// use freemvvm::AsyncCommand;
/// fn require_send<T: Send>() {}
/// require_send::<AsyncCommand>();
/// ```
pub struct AsyncCommand {
    owner: Owned<CommandTag>,
    identity: Arc<()>,
}

impl AsyncCommand {
    pub(crate) fn from_owner(owner: Owned<CommandTag>, identity: Arc<()>) -> Self {
        Self { owner, identity }
    }

    /// Creates a Command whose start callback transfers one worker context per accepted execution.
    ///
    /// The callback runs synchronously on the UI thread and should start work, then return. It does
    /// not need to be `Send`; the [`CommandOperation`] passed to it can be moved to a worker.
    pub fn create<F>(
        ui: &UiContext<'_>,
        eligibility: Option<&Property<bool>>,
        start: F,
    ) -> Result<Self, UiError>
    where
        F: FnMut(CommandOperation) + 'static,
    {
        ui.check_ui_thread()?;
        if let Some(property) = eligibility {
            property.check_context(ui)?;
        }

        let runtime = ui.runtime_token();
        let api = Arc::clone(ui.api());
        let anchor = RuntimeAnchor::try_create(Arc::clone(&api), runtime)?;
        let identity = ui.identity();
        let eligibility_token = match eligibility {
            Some(property) => property.token_for_call()?,
            None => 0,
        };
        let command_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let callback_state = Box::new(StartState {
            callback: start,
            api: Arc::clone(&api),
            anchor: Arc::clone(&anchor),
        });
        let userdata = Box::into_raw(callback_state).cast::<c_void>();
        let mut command = 0;
        let mut error = 0;
        // SAFETY: callback state is a stable Box allocation. The ABI transfers it only after
        // successful registration and invokes its destroy callback once after close/shutdown.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_command_create_async(
                runtime,
                eligibility_token,
                Some(start_trampoline::<F>),
                userdata,
                Some(start_destroy_trampoline::<F>),
                &mut command,
                &mut error,
            )
        };

        if status == native_api::OK && command != 0 && error == 0 {
            error_ticket.release();
            return Ok(Self {
                owner: Owned::from_token(command, api, command_ticket, runtime, Some(anchor)),
                identity,
            });
        }

        let result_status = if status == native_api::OK {
            native_api::INTERNAL_ERROR
        } else {
            status
        };
        if status == native_api::OK {
            if command == 0 {
                // An impossible success without a handle may still have transferred userdata.
                // Keep it alive rather than risk a later Core callback using freed memory.
                command_ticket.release();
                // SAFETY: `userdata` is the allocation created above; reconstruct it only to leak it.
                std::mem::forget(unsafe { Box::from_raw(userdata.cast::<StartState<F>>()) });
            } else {
                dispose_unexpected_command(
                    command,
                    command_ticket,
                    &api,
                    runtime,
                    Some(Arc::clone(&anchor)),
                );
            }
        } else {
            // Failed create retains callback userdata on the caller side by contract.
            // SAFETY: the allocation was created above and failed registration did not transfer it.
            drop(unsafe { Box::from_raw(userdata.cast::<StartState<F>>()) });
            dispose_unexpected_command(
                command,
                command_ticket,
                &api,
                runtime,
                Some(anchor.clone()),
            );
        }
        Err(UiError::capture(
            &api,
            result_status,
            error,
            error_ticket,
            runtime,
            Some(anchor),
        ))
    }

    /// Starts one operation or returns an error without issuing an operation identifier.
    pub fn execute(&self, ui: &UiContext<'_>) -> Result<OperationId, UiError> {
        self.check_context(ui)?;
        ui.check_ui_thread()?;
        let command = self.owner.borrow()?.token();
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let mut operation = 0;
        let mut error = 0;
        // SAFETY: outputs are writable and the Command token is borrowed for this call.
        let status =
            unsafe { freemvvm_sys::ui_abi_1_command_execute(command, &mut operation, &mut error) };
        if status == native_api::OK && error == 0 {
            if let Some(operation) = OperationId::from_raw(operation) {
                error_ticket.release();
                return Ok(operation);
            }
        }
        Err(UiError::capture(
            self.owner.api(),
            if status == native_api::OK {
                native_api::INTERNAL_ERROR
            } else {
                status
            },
            error,
            error_ticket,
            self.owner.runtime_token(),
            self.owner.anchor(),
        ))
    }

    /// Requests cooperative cancellation for the specified execution without waiting for it.
    pub fn cancel(&self, ui: &UiContext<'_>, operation: OperationId) -> Result<(), UiError> {
        self.check_context(ui)?;
        ui.check_ui_thread()?;
        let command = self.owner.borrow()?.token();
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let mut error = 0;
        // SAFETY: `error` is writable storage and the Command token is borrowed for this call.
        let status =
            unsafe { freemvvm_sys::ui_abi_1_command_cancel(command, operation.get(), &mut error) };
        if status == native_api::OK && error == 0 {
            error_ticket.release();
            return Ok(());
        }
        Err(UiError::capture(
            self.owner.api(),
            if status == native_api::OK {
                native_api::INTERNAL_ERROR
            } else {
                status
            },
            error,
            error_ticket,
            self.owner.runtime_token(),
            self.owner.anchor(),
        ))
    }

    /// Gets an independently owned read-only I64 Property for the current Command state.
    pub fn state_property(&self, ui: &UiContext<'_>) -> Result<Property<i64>, UiError> {
        self.property(ui, true)
    }

    /// Gets an independently owned read-only BOOL Property for current executability.
    pub fn can_execute_property(&self, ui: &UiContext<'_>) -> Result<Property<bool>, UiError> {
        self.property(ui, false)
    }

    /// Returns the current immutable failure snapshot, if the last operation failed.
    pub fn failure_snapshot(&self, ui: &UiContext<'_>) -> Result<Option<UiError>, UiError> {
        self.check_context(ui)?;
        ui.check_ui_thread()?;
        let command = self.owner.borrow()?.token();
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let mut error = 0;
        // SAFETY: `error` is writable storage and the Command token is borrowed for this call.
        let status =
            unsafe { freemvvm_sys::ui_abi_1_command_failure_snapshot(command, &mut error) };
        if status != native_api::OK {
            if error == 0 {
                error_ticket.release();
            } else {
                let _ = UiError::capture(
                    self.owner.api(),
                    status,
                    error,
                    error_ticket,
                    self.owner.runtime_token(),
                    self.owner.anchor(),
                );
            }
            return Err(UiError::from_status(status));
        }
        if error == 0 {
            error_ticket.release();
            return Ok(None);
        }
        Ok(Some(UiError::capture(
            self.owner.api(),
            native_api::OK,
            error,
            error_ticket,
            self.owner.runtime_token(),
            self.owner.anchor(),
        )))
    }

    /// Closes the Core Command, then releases this public handle. A failed release remains retryable.
    pub fn close(&mut self, ui: &UiContext<'_>) -> Result<(), UiError> {
        self.check_context(ui)?;
        ui.check_ui_thread()?;
        let command = self.owner.borrow()?.token();
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let mut error = 0;
        // SAFETY: `error` is writable storage and the Command token is borrowed for this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_command_close(command, &mut error) };
        if status != native_api::OK || error != 0 {
            return Err(UiError::capture(
                self.owner.api(),
                if status == native_api::OK {
                    native_api::INTERNAL_ERROR
                } else {
                    status
                },
                error,
                error_ticket,
                self.owner.runtime_token(),
                self.owner.anchor(),
            ));
        }
        error_ticket.release();
        self.owner.close()
    }

    fn property<T: crate::property::PropertyValue>(
        &self,
        ui: &UiContext<'_>,
        state: bool,
    ) -> Result<Property<T>, UiError> {
        self.check_context(ui)?;
        ui.check_ui_thread()?;
        let command = self.owner.borrow()?.token();
        let ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let mut property = 0;
        // SAFETY: output storage is writable and the Command token is borrowed for this call.
        let status = unsafe {
            if state {
                freemvvm_sys::ui_abi_1_command_state_property(command, &mut property)
            } else {
                freemvvm_sys::ui_abi_1_command_can_execute_property(command, &mut property)
            }
        };
        if status != native_api::OK || property == 0 {
            if property == 0 {
                ticket.release();
            } else {
                drop(Owned::<PropertyTag>::from_token(
                    property,
                    Arc::clone(self.owner.api()),
                    ticket,
                    self.owner.runtime_token(),
                    self.owner.anchor(),
                ));
            }
            return Err(UiError::from_status(if status == native_api::OK {
                native_api::INTERNAL_ERROR
            } else {
                status
            }));
        }
        Ok(Property::from_owned(
            Owned::from_token(
                property,
                Arc::clone(self.owner.api()),
                ticket,
                self.owner.runtime_token(),
                self.owner.anchor(),
            ),
            Arc::clone(&self.identity),
        ))
    }

    fn check_context(&self, ui: &UiContext<'_>) -> Result<(), UiError> {
        if Arc::ptr_eq(&self.identity, &ui.identity()) {
            Ok(())
        } else {
            Err(UiError::from_status(native_api::WRONG_RUNTIME))
        }
    }

    pub(crate) fn token_for_call(&self) -> Result<u64, UiError> {
        Ok(self.owner.borrow()?.token())
    }

    pub(crate) fn check_ui_context(&self, ui: &UiContext<'_>) -> Result<(), UiError> {
        self.check_context(ui)
    }
}

/// The worker-safe, read-only cancellation view for one execution.
pub struct CancellationToken {
    owner: AnyThreadOwned,
}

impl CancellationToken {
    /// Queries the cooperative cancellation request; it does not mutate Command or UI state.
    pub fn is_cancel_requested(&self) -> Result<bool, UiError> {
        let token = self.owner.token()?;
        let mut requested = 0;
        // SAFETY: `requested` is writable storage; the ABI allows this query from any thread.
        let status =
            unsafe { freemvvm_sys::ui_abi_1_command_cancel_token_requested(token, &mut requested) };
        if status != native_api::OK {
            return Err(UiError::from_status(status));
        }
        match requested {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(UiError::from_status(native_api::INTERNAL_ERROR)),
        }
    }
}

/// Per-execution context transferred from the UI start callback to an app-owned worker.
pub struct CommandOperation {
    id: OperationId,
    cancellation: CancellationToken,
    completion: CompletionReporter,
}

impl CommandOperation {
    /// Returns this execution's Runtime-issued identifier.
    pub const fn id(&self) -> OperationId {
        self.id
    }

    /// Returns the thread-safe, read-only cancellation query for this execution.
    pub fn cancellation(&self) -> &CancellationToken {
        &self.cancellation
    }

    /// Enqueues successful completion for UI-thread application.
    pub fn report_success(&self) -> Result<(), UiError> {
        self.completion.post(CompletionOutcome::Success)
    }

    /// Enqueues a machine-readable failure snapshot for UI-thread application.
    pub fn report_failure(&self, domain: u32, code: u32, message: &str) -> Result<(), UiError> {
        let mut owned_message = String::new();
        owned_message
            .try_reserve_exact(message.len())
            .map_err(|_| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        owned_message.push_str(message);
        let length = u64::try_from(owned_message.len())
            .map_err(|_| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        self.completion.post(CompletionOutcome::Failure {
            domain,
            code,
            message: owned_message,
            byte_length: length,
        })
    }

    /// Enqueues a worker-confirmed cancellation result for UI-thread application.
    pub fn report_cancelled(&self) -> Result<(), UiError> {
        self.completion.post(CompletionOutcome::Cancelled)
    }
}

struct CompletionReporter {
    command: AnyThreadOwned,
    api: SharedApi,
    anchor: Arc<RuntimeAnchor>,
    operation: OperationId,
    state: Arc<AtomicU8>,
}

enum CompletionOutcome {
    Success,
    Failure {
        domain: u32,
        code: u32,
        message: String,
        byte_length: u64,
    },
    Cancelled,
}

struct PostedCompletion {
    command: AnyThreadOwned,
    operation: OperationId,
    outcome: CompletionOutcome,
}

impl CompletionReporter {
    fn post(&self, outcome: CompletionOutcome) -> Result<(), UiError> {
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let command = self.command.try_clone()?;
        self.state
            .compare_exchange(
                REPORT_PENDING,
                REPORT_POSTING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map_err(|_| UiError::from_status(native_api::INVALID_ARGUMENT))?;
        let mut posting = PostingGuard {
            state: Arc::clone(&self.state),
            submitted: false,
        };
        let payload = Box::new(PostedCompletion {
            command,
            operation: self.operation,
            outcome,
        });
        let userdata = Box::into_raw(payload).cast::<c_void>();
        let mut error = 0;
        // SAFETY: `userdata` is a stable Box allocation. Ownership transfers only on success,
        // after which Core invokes callback and destroy exactly once.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_runtime_post(
                self.anchor.token(),
                Some(apply_completion_trampoline),
                userdata,
                Some(completion_destroy_trampoline),
                &mut error,
            )
        };
        if status != native_api::OK {
            // SAFETY: runtime_post failure means ownership of userdata stayed with this caller.
            drop(unsafe { Box::from_raw(userdata.cast::<PostedCompletion>()) });
            return Err(UiError::capture(
                &self.api,
                status,
                error,
                error_ticket,
                self.anchor.token(),
                Some(Arc::clone(&self.anchor)),
            ));
        }
        if error != 0 {
            // Success transfers the Box even if the result shape is malformed; keep it alive.
            posting.submitted = true;
            return Err(UiError::capture(
                &self.api,
                native_api::INTERNAL_ERROR,
                error,
                error_ticket,
                self.anchor.token(),
                Some(Arc::clone(&self.anchor)),
            ));
        }
        error_ticket.release();
        posting.submitted = true;
        Ok(())
    }
}

struct PostingGuard {
    state: Arc<AtomicU8>,
    submitted: bool,
}

impl Drop for PostingGuard {
    fn drop(&mut self) {
        self.state.store(
            if self.submitted {
                REPORT_SUBMITTED
            } else {
                REPORT_PENDING
            },
            Ordering::Release,
        );
    }
}

unsafe extern "C" fn apply_completion_trampoline(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    // SAFETY: Core owns this stable Box until its destroy callback after this invocation.
    let completion = unsafe { &mut *userdata.cast::<PostedCompletion>() };
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let Ok(command) = completion.command.token() else {
            return;
        };
        let operation = completion.operation.get();
        // The C ABI returns a status here; a stale, closed, or already terminal report is
        // intentionally ignored because the worker-facing API reports enqueue acceptance only.
        // SAFETY: runtime_post runs this callback on the Runtime UI thread with a live Command token.
        unsafe {
            match &completion.outcome {
                CompletionOutcome::Success => {
                    let _ = freemvvm_sys::ui_abi_1_command_complete_success(command, operation);
                }
                CompletionOutcome::Failure {
                    domain,
                    code,
                    message,
                    byte_length,
                } => {
                    let _ = freemvvm_sys::ui_abi_1_command_complete_failure_utf8(
                        command,
                        operation,
                        *domain,
                        *code,
                        message.as_bytes().as_ptr(),
                        *byte_length,
                    );
                }
                CompletionOutcome::Cancelled => {
                    let _ = freemvvm_sys::ui_abi_1_command_complete_cancelled(command, operation);
                }
            }
        }
    }));
}

unsafe extern "C" fn completion_destroy_trampoline(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    // SAFETY: Core invokes destroy exactly once after completion or shutdown cancellation.
    // Reconstructing this Box is the unique ownership transfer back to this wrapper.
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: this callback owns the Box allocation transferred to runtime_post.
        drop(unsafe { Box::from_raw(userdata.cast::<PostedCompletion>()) })
    }));
}

unsafe extern "C" fn start_trampoline<F>(
    userdata: *mut c_void,
    borrowed_command: u64,
    operation_id: u64,
    borrowed_cancel_token: u64,
) -> u32
where
    F: FnMut(CommandOperation) + 'static,
{
    if userdata.is_null() {
        return native_api::INVALID_ARGUMENT;
    }
    // SAFETY: Core invokes the registered callback on the UI thread and serializes start calls.
    let state = unsafe { &mut *userdata.cast::<StartState<F>>() };
    let result = catch_unwind(AssertUnwindSafe(|| {
        let Some(operation) = OperationId::from_raw(operation_id) else {
            return Err(native_api::INTERNAL_ERROR);
        };
        let command = retain_worker_handle(borrowed_command, &state.api, &state.anchor)?;
        let cancellation = retain_worker_handle(borrowed_cancel_token, &state.api, &state.anchor)?;
        let operation = CommandOperation {
            id: operation,
            cancellation: CancellationToken {
                owner: cancellation,
            },
            completion: CompletionReporter {
                command,
                api: Arc::clone(&state.api),
                anchor: Arc::clone(&state.anchor),
                operation,
                state: Arc::new(AtomicU8::new(REPORT_PENDING)),
            },
        };
        (state.callback)(operation);
        Ok(())
    }));
    match result {
        Ok(Ok(())) => native_api::OK,
        Ok(Err(status)) => status,
        Err(_) => native_api::INTERNAL_ERROR,
    }
}

unsafe extern "C" fn start_destroy_trampoline<F>(userdata: *mut c_void)
where
    F: FnMut(CommandOperation) + 'static,
{
    if userdata.is_null() {
        return;
    }
    // SAFETY: Core transfers the registered Box to destroy exactly once after in-flight start exits.
    // Reconstructing this Box is the unique ownership transfer back to this wrapper.
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: Core transferred this Box to the destroy callback exactly once.
        drop(unsafe { Box::from_raw(userdata.cast::<StartState<F>>()) })
    }));
}

fn retain_worker_handle(
    borrowed: u64,
    api: &SharedApi,
    anchor: &Arc<RuntimeAnchor>,
) -> Result<AnyThreadOwned, u32> {
    let ticket = TicketReservation::reserve().map_err(|()| native_api::OUT_OF_MEMORY)?;
    let (status, owned) = api.handle_retain(borrowed);
    if status != native_api::OK {
        if owned != 0 {
            ticket.defer(PendingToken {
                token: owned,
                runtime_token: anchor.token(),
                api: Arc::clone(api),
                anchor: Some(Arc::clone(anchor)),
            });
        }
        return Err(status);
    }
    if owned == 0 {
        ticket.release();
        return Err(native_api::INTERNAL_ERROR);
    }
    Ok(AnyThreadOwned::from_token(
        owned,
        Arc::clone(api),
        ticket,
        Arc::clone(anchor),
    ))
}

fn dispose_unexpected_command(
    token: u64,
    ticket: TicketReservation,
    api: &SharedApi,
    runtime: u64,
    anchor: Option<Arc<RuntimeAnchor>>,
) {
    if token == 0 {
        ticket.release();
        return;
    }
    // SAFETY: the token was unexpectedly returned as an owned Command handle; out_error is optional.
    let _ = unsafe { freemvvm_sys::ui_abi_1_command_close(token, std::ptr::null_mut()) };
    drop(Owned::<CommandTag>::from_token(
        token,
        Arc::clone(api),
        ticket,
        runtime,
        anchor,
    ));
}

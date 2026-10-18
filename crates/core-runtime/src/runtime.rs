use crate::dispatcher::{DispatchError, Dispatcher, ShutdownRequestAdmission, UiLoopBridge};
use crate::handle_registry::registry::{CleanupQueue, CleanupReservation, ObjectIdentity};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::ThreadId;

type Finalizer = Box<dyn FnOnce() + Send>;

/// Thread identity captured by Runtime-affine components at creation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiThreadContext {
    thread_id: ThreadId,
}

impl UiThreadContext {
    pub(crate) fn capture() -> Self {
        Self {
            thread_id: std::thread::current().id(),
        }
    }

    pub(crate) fn is_current(self) -> bool {
        std::thread::current().id() == self.thread_id
    }
}

struct Ticket {
    identity: ObjectIdentity,
    on_complete: Finalizer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum RuntimeState {
    Running,
    Closing,
    Closed,
}

struct QueueState {
    lifecycle: RuntimeState,
    outstanding: usize,
    tickets: VecDeque<Ticket>,
    tracked_ui_state: Vec<Box<dyn Fn() + Send + Sync>>,
}

struct QueueInner {
    ui_thread: UiThreadContext,
    state: Mutex<QueueState>,
    changed: Condvar,
}

/// Runtime-owned finalizer queue. Tickets are accepted before token release commits,
/// and their callbacks run only while the Runtime UI thread drains the queue.
pub(crate) struct RuntimeCleanupQueue {
    inner: Arc<QueueInner>,
}

#[allow(dead_code)]
impl RuntimeCleanupQueue {
    pub(crate) fn new() -> Arc<Self> {
        Self::new_for_ui_thread(UiThreadContext::capture())
    }

    pub(crate) fn new_for_ui_thread(ui_thread: UiThreadContext) -> Arc<Self> {
        Arc::new(Self {
            inner: Arc::new(QueueInner {
                ui_thread,
                state: Mutex::new(QueueState {
                    lifecycle: RuntimeState::Running,
                    outstanding: 0,
                    tickets: VecDeque::new(),
                    tracked_ui_state: Vec::new(),
                }),
                changed: Condvar::new(),
            }),
        })
    }

    pub(crate) fn state(&self) -> RuntimeState {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .lifecycle
    }

    pub(crate) fn queued_count(&self) -> usize {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .tickets
            .len()
    }

    pub(crate) fn begin_shutdown(&self) -> Result<(), ShutdownError> {
        self.ensure_ui_thread()?;
        let mut state = self.lock_state()?;
        if state.lifecycle == RuntimeState::Running {
            state.lifecycle = RuntimeState::Closing;
        }
        Ok(())
    }

    /// Runs every accepted finalizer before publishing Closed.
    pub(crate) fn drain_shutdown(&self) -> Result<(), ShutdownError> {
        self.ensure_ui_thread()?;
        self.begin_shutdown()?;

        loop {
            let work = {
                let mut state = self.lock_state()?;
                if let Some(ticket) = state.tickets.pop_front() {
                    Some(ShutdownWork::Ticket(ticket))
                } else if state.outstanding == 0 && !state.tracked_ui_state.is_empty() {
                    Some(ShutdownWork::UiStateCleanup(
                        state.tracked_ui_state.remove(0),
                    ))
                } else if state.outstanding == 0 {
                    state.lifecycle = RuntimeState::Closed;
                    self.inner.changed.notify_all();
                    return Ok(());
                } else {
                    state = self
                        .inner
                        .changed
                        .wait(state)
                        .map_err(|_| ShutdownError::SynchronizationPoisoned)?;
                    drop(state);
                    None
                }
            };

            if let Some(work) = work {
                match work {
                    ShutdownWork::Ticket(ticket) => {
                        let _identity = ticket.identity;
                        (ticket.on_complete)();
                        let mut state = self.lock_state()?;
                        state.outstanding -= 1;
                        self.inner.changed.notify_all();
                    }
                    ShutdownWork::UiStateCleanup(cleanup) => cleanup(),
                }
            }
        }
    }

    fn ensure_ui_thread(&self) -> Result<(), ShutdownError> {
        if self.inner.ui_thread.is_current() {
            Ok(())
        } else {
            Err(ShutdownError::WrongThread)
        }
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, QueueState>, ShutdownError> {
        self.inner
            .state
            .lock()
            .map_err(|_| ShutdownError::SynchronizationPoisoned)
    }
}

impl CleanupQueue for RuntimeCleanupQueue {
    fn track_ui_state(&self, on_shutdown: Box<dyn Fn() + Send + Sync>) -> Result<(), ()> {
        let mut state = self.inner.state.lock().map_err(|_| ())?;
        if state.lifecycle != RuntimeState::Running {
            return Err(());
        }
        state.tracked_ui_state.try_reserve(1).map_err(|_| ())?;
        state.tracked_ui_state.push(on_shutdown);
        Ok(())
    }

    fn reserve(
        &self,
        identity: ObjectIdentity,
        on_complete: Finalizer,
    ) -> Result<Box<dyn CleanupReservation>, ()> {
        let mut state = self.inner.state.lock().map_err(|_| ())?;
        if state.lifecycle == RuntimeState::Closed {
            return Err(());
        }
        let additional_capacity = state.outstanding.checked_add(1).ok_or(())?;
        state
            .tickets
            .try_reserve(additional_capacity)
            .map_err(|_| ())?;
        state.outstanding = state.outstanding.checked_add(1).ok_or(())?;
        Ok(Box::new(RuntimeCleanupReservation {
            inner: Arc::clone(&self.inner),
            ticket: Some(Ticket {
                identity,
                on_complete,
            }),
        }))
    }
}

enum ShutdownWork {
    Ticket(Ticket),
    UiStateCleanup(Box<dyn Fn() + Send + Sync>),
}

struct RuntimeCleanupReservation {
    inner: Arc<QueueInner>,
    ticket: Option<Ticket>,
}

impl CleanupReservation for RuntimeCleanupReservation {
    fn activate(mut self: Box<Self>) {
        let ticket = self.ticket.take().expect("reservation activates once");
        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.tickets.push_back(ticket);
        self.inner.changed.notify_all();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum ShutdownError {
    WrongThread,
    SynchronizationPoisoned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum ShutdownCompletionState {
    Pending,
    Completed,
    HostUnavailable { reason: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum ShutdownWaitResult {
    Completed,
    HostUnavailable { reason: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum ShutdownRequestMode {
    Async,
    Sync,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum ShutdownRequestOutcome {
    Pending,
    Completed,
    HostUnavailable { reason: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum ShutdownCompletionError {
    WrongThread,
    InvalidState,
    SynchronizationPoisoned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum ShutdownCommitError {
    Cleanup(ShutdownError),
    Completion(ShutdownCompletionError),
}

struct CompletionInner {
    state: Mutex<ShutdownCompletionState>,
    changed: Condvar,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
enum HostPumpState {
    Available,
    Stopping,
    Stopped { reason: u32 },
}

struct HostRegistrationState {
    pump: HostPumpState,
    next_scope_id: Option<u64>,
    completion_records: Vec<Arc<CompletionInner>>,
}

thread_local! {
    static HOST_SCOPE_STACK: RefCell<Vec<(usize, u64)>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum HostRegistrationError {
    WrongThread,
    InvalidContext,
    InvalidState,
    HostUnavailable { reason: u32 },
    ScopeIdExhausted,
    OutOfMemory,
    SynchronizationPoisoned,
    Completion(ShutdownCompletionError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum HostBoundaryError {
    Host(HostRegistrationError),
    Dispatch(DispatchError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum HostShutdownCommitError {
    Boundary(HostBoundaryError),
    Commit(ShutdownCommitError),
}

/// Records one host's UI event boundaries and the shutdown requests it can run.
#[allow(dead_code)]
pub(crate) struct HostRegistration {
    ui_thread: UiThreadContext,
    scope_owner: Arc<()>,
    state: Mutex<HostRegistrationState>,
}

#[allow(dead_code)]
impl HostRegistration {
    pub(crate) fn new() -> Self {
        Self::new_for_ui_thread(UiThreadContext::capture())
    }

    pub(crate) fn new_for_ui_thread(ui_thread: UiThreadContext) -> Self {
        Self {
            ui_thread,
            scope_owner: Arc::new(()),
            state: Mutex::new(HostRegistrationState {
                pump: HostPumpState::Available,
                next_scope_id: Some(1),
                completion_records: Vec::new(),
            }),
        }
    }

    pub(crate) fn begin_event(&self) -> Result<u64, HostRegistrationError> {
        self.ensure_ui_thread()?;
        let mut state = self.lock_state()?;
        self.ensure_available(state.pump)?;
        let scope_id = state
            .next_scope_id
            .ok_or(HostRegistrationError::ScopeIdExhausted)?;
        let owner = Arc::as_ptr(&self.scope_owner) as usize;
        HOST_SCOPE_STACK.with(|stack| {
            let mut stack = stack
                .try_borrow_mut()
                .map_err(|_| HostRegistrationError::InvalidContext)?;
            stack
                .try_reserve(1)
                .map_err(|_| HostRegistrationError::OutOfMemory)?;
            stack.push((owner, scope_id));
            Ok(())
        })?;
        state.next_scope_id = scope_id.checked_add(1);
        Ok(scope_id)
    }

    /// Returns true when the matching scope was the outermost host event.
    pub(crate) fn end_event(&self, scope_id: u64) -> Result<bool, HostRegistrationError> {
        self.ensure_ui_thread()?;
        let _state = self.lock_state()?;
        let owner = Arc::as_ptr(&self.scope_owner) as usize;
        HOST_SCOPE_STACK.with(|stack| {
            let mut stack = stack
                .try_borrow_mut()
                .map_err(|_| HostRegistrationError::InvalidContext)?;
            if stack.last().copied() != Some((owner, scope_id)) {
                return Err(HostRegistrationError::InvalidContext);
            }
            stack.pop();
            Ok(stack.is_empty())
        })
    }

    /// Close one event scope and service priority shutdown only at the outer boundary.
    pub(crate) fn end_event_and_dispatch_shutdown<F>(
        &self,
        scope_id: u64,
        dispatcher: &Dispatcher,
        on_shutdown: F,
    ) -> Result<bool, HostBoundaryError>
    where
        F: FnOnce(),
    {
        let outermost = self.end_event(scope_id).map_err(HostBoundaryError::Host)?;
        if !outermost {
            return Ok(false);
        }
        dispatcher
            .run_pending_shutdown(on_shutdown)
            .map_err(HostBoundaryError::Dispatch)
    }

    /// Drain Runtime-owned UI cleanup and publish completion at the outer shutdown boundary.
    pub(crate) fn end_event_and_commit_shutdown(
        &self,
        scope_id: u64,
        dispatcher: &Dispatcher,
        cleanup: &RuntimeCleanupQueue,
    ) -> Result<bool, HostShutdownCommitError> {
        let mut commit_result = None;
        let dispatched = self
            .end_event_and_dispatch_shutdown(scope_id, dispatcher, || {
                commit_result = Some(self.drain_shutdown_and_complete_requests(cleanup));
            })
            .map_err(HostShutdownCommitError::Boundary)?;
        if !dispatched {
            return Ok(false);
        }
        commit_result.expect("shutdown handler stores its commit result")?;
        Ok(true)
    }

    fn drain_shutdown_and_complete_requests(
        &self,
        cleanup: &RuntimeCleanupQueue,
    ) -> Result<(), HostShutdownCommitError> {
        cleanup
            .drain_shutdown()
            .map_err(ShutdownCommitError::Cleanup)
            .map_err(HostShutdownCommitError::Commit)?;
        let records = {
            let mut state = self.lock_state().map_err(|error| {
                HostShutdownCommitError::Boundary(HostBoundaryError::Host(error))
            })?;
            std::mem::take(&mut state.completion_records)
        };
        for record in records {
            ShutdownCompletion {
                ui_thread: self.ui_thread,
                inner: record,
            }
            .mark_completed()
            .map_err(ShutdownCommitError::Completion)
            .map_err(HostShutdownCommitError::Commit)?;
        }
        Ok(())
    }

    /// Drain an accepted shutdown before closing host admission for loop teardown.
    pub(crate) fn before_stop(
        &self,
        dispatcher: &Dispatcher,
        cleanup: &RuntimeCleanupQueue,
    ) -> Result<bool, HostShutdownCommitError> {
        self.ensure_ui_thread()
            .map_err(|error| HostShutdownCommitError::Boundary(HostBoundaryError::Host(error)))?;
        if !dispatcher.belongs_to_thread(self.ui_thread) {
            return Err(HostShutdownCommitError::Boundary(HostBoundaryError::Host(
                HostRegistrationError::InvalidContext,
            )));
        }
        {
            let mut state = self.lock_state().map_err(|error| {
                HostShutdownCommitError::Boundary(HostBoundaryError::Host(error))
            })?;
            if HOST_SCOPE_STACK.with(|stack| !stack.borrow().is_empty()) {
                return Err(HostShutdownCommitError::Boundary(HostBoundaryError::Host(
                    HostRegistrationError::InvalidContext,
                )));
            }
            match state.pump {
                HostPumpState::Available => state.pump = HostPumpState::Stopping,
                HostPumpState::Stopping => {}
                HostPumpState::Stopped { .. } => {
                    return Err(HostShutdownCommitError::Boundary(HostBoundaryError::Host(
                        HostRegistrationError::InvalidState,
                    )));
                }
            }
        }

        let mut commit_result = None;
        let dispatched = match dispatcher.run_pending_shutdown(|| {
            commit_result = Some(self.drain_shutdown_and_complete_requests(cleanup));
        }) {
            Ok(dispatched) => dispatched,
            Err(DispatchError::AlreadyDispatching) => {
                if let Ok(mut state) = self.state.lock() {
                    if state.pump == HostPumpState::Stopping {
                        state.pump = HostPumpState::Available;
                    }
                }
                return Err(HostShutdownCommitError::Boundary(
                    HostBoundaryError::Dispatch(DispatchError::AlreadyDispatching),
                ));
            }
            Err(error) => {
                return Err(HostShutdownCommitError::Boundary(
                    HostBoundaryError::Dispatch(error),
                ));
            }
        };
        if !dispatched {
            return Ok(false);
        }
        commit_result.expect("shutdown handler stores its commit result")?;
        Ok(true)
    }

    pub(crate) fn register_completion(
        &self,
        completion: &ShutdownCompletion,
    ) -> Result<(), HostRegistrationError> {
        if self.ui_thread != completion.ui_thread {
            return Err(HostRegistrationError::InvalidContext);
        }
        let mut state = self.lock_state()?;
        self.ensure_available(state.pump)?;
        Self::reserve_completion_record(&mut state, &completion.inner)?;
        Self::store_completion_record(&mut state, &completion.inner);
        Ok(())
    }

    /// Admit one prepared completion record and arm priority shutdown atomically with host stop.
    pub(crate) fn request_shutdown(
        &self,
        dispatcher: &Dispatcher,
        completion: &ShutdownCompletion,
    ) -> Result<(), HostRegistrationError> {
        if self.ui_thread != completion.ui_thread || !dispatcher.belongs_to_thread(self.ui_thread) {
            return Err(HostRegistrationError::InvalidContext);
        }
        if completion
            .query()
            .map_err(HostRegistrationError::Completion)?
            != ShutdownCompletionState::Pending
        {
            return Err(HostRegistrationError::InvalidState);
        }

        let admission = {
            let mut state = self.lock_state()?;
            self.ensure_available(state.pump)?;
            Self::reserve_completion_record(&mut state, &completion.inner)?;
            let admission = dispatcher
                .prepare_shutdown_request()
                .map_err(|error| match error {
                    DispatchError::WrongThread | DispatchError::AlreadyDispatching => {
                        HostRegistrationError::InvalidContext
                    }
                    DispatchError::SynchronizationPoisoned => {
                        HostRegistrationError::SynchronizationPoisoned
                    }
                })?;
            if admission == ShutdownRequestAdmission::Closed {
                return Err(HostRegistrationError::InvalidState);
            }
            Self::store_completion_record(&mut state, &completion.inner);
            admission
        };
        dispatcher.schedule_shutdown_wake(admission);
        Ok(())
    }

    /// Admit a shutdown request and return its independently owned observation receipt.
    pub(crate) fn request_shutdown_receipt(
        &self,
        dispatcher: &Dispatcher,
        completion: &ShutdownCompletion,
    ) -> Result<ShutdownRequest, HostRegistrationError> {
        self.request_shutdown(dispatcher, completion)?;
        Ok(ShutdownRequest {
            completion: completion.clone(),
        })
    }

    fn reserve_completion_record(
        state: &mut HostRegistrationState,
        completion: &Arc<CompletionInner>,
    ) -> Result<(), HostRegistrationError> {
        if !state
            .completion_records
            .iter()
            .any(|record| Arc::ptr_eq(record, completion))
        {
            state
                .completion_records
                .try_reserve(1)
                .map_err(|_| HostRegistrationError::OutOfMemory)?;
        }
        Ok(())
    }

    fn store_completion_record(
        state: &mut HostRegistrationState,
        completion: &Arc<CompletionInner>,
    ) {
        if !state
            .completion_records
            .iter()
            .any(|record| Arc::ptr_eq(record, completion))
        {
            state.completion_records.push(Arc::clone(completion));
        }
    }

    /// Wait for shutdown only when synchronous mode is requested from a worker thread.
    pub(crate) fn request_shutdown_with_mode(
        &self,
        dispatcher: &Dispatcher,
        completion: &ShutdownCompletion,
        mode: ShutdownRequestMode,
    ) -> Result<ShutdownRequestOutcome, HostRegistrationError> {
        let receipt = self.request_shutdown_receipt(dispatcher, completion)?;
        if mode == ShutdownRequestMode::Sync && !self.ui_thread.is_current() {
            return receipt
                .wait()
                .map(|result| match result {
                    ShutdownWaitResult::Completed => ShutdownRequestOutcome::Completed,
                    ShutdownWaitResult::HostUnavailable { reason } => {
                        ShutdownRequestOutcome::HostUnavailable { reason }
                    }
                })
                .map_err(HostRegistrationError::Completion);
        }
        receipt
            .query()
            .map(|state| match state {
                ShutdownCompletionState::Pending => ShutdownRequestOutcome::Pending,
                ShutdownCompletionState::Completed => ShutdownRequestOutcome::Completed,
                ShutdownCompletionState::HostUnavailable { reason } => {
                    ShutdownRequestOutcome::HostUnavailable { reason }
                }
            })
            .map_err(HostRegistrationError::Completion)
    }

    /// Notify registered waiters without claiming that cleanup has completed.
    pub(crate) fn notify_stopped(&self, reason: u32) -> Result<(), HostRegistrationError> {
        let records = {
            let mut state = self.lock_state()?;
            state.pump = HostPumpState::Stopped { reason };
            std::mem::take(&mut state.completion_records)
        };
        let mut first_error = None;
        for record in records {
            let completion = ShutdownCompletion {
                ui_thread: self.ui_thread,
                inner: record,
            };
            if let Err(error) = completion.mark_host_unavailable(reason) {
                first_error.get_or_insert(HostRegistrationError::Completion(error));
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    pub(crate) fn resume_on_ui_thread(&self) -> Result<(), HostRegistrationError> {
        self.ensure_ui_thread()?;
        let mut state = self.lock_state()?;
        match state.pump {
            HostPumpState::Stopped { .. } => {
                state.pump = HostPumpState::Available;
                Ok(())
            }
            HostPumpState::Available => Ok(()),
            HostPumpState::Stopping => Err(HostRegistrationError::InvalidState),
        }
    }

    /// Re-register and re-arm one request after the same UI host resumes.
    pub(crate) fn retry_on_ui_thread(
        &self,
        completion: &ShutdownCompletion,
        dispatcher: &Dispatcher,
    ) -> Result<(), HostRegistrationError> {
        self.ensure_ui_thread()?;
        if self.ui_thread != completion.ui_thread || !dispatcher.belongs_to_thread(self.ui_thread) {
            return Err(HostRegistrationError::InvalidContext);
        }
        {
            let mut state = self.lock_state()?;
            self.ensure_available(state.pump)?;
            if !matches!(
                completion
                    .query()
                    .map_err(HostRegistrationError::Completion)?,
                ShutdownCompletionState::HostUnavailable { .. }
            ) {
                return Err(HostRegistrationError::InvalidState);
            }
            Self::reserve_completion_record(&mut state, &completion.inner)?;
            if !dispatcher
                .prepare_shutdown_retry()
                .map_err(|error| match error {
                    DispatchError::WrongThread | DispatchError::AlreadyDispatching => {
                        HostRegistrationError::InvalidContext
                    }
                    DispatchError::SynchronizationPoisoned => {
                        HostRegistrationError::SynchronizationPoisoned
                    }
                })?
            {
                return Err(HostRegistrationError::InvalidState);
            }
            completion
                .retry_on_ui_thread()
                .map_err(HostRegistrationError::Completion)?;
            Self::store_completion_record(&mut state, &completion.inner);
        }
        dispatcher.schedule_shutdown_retry_wake();
        Ok(())
    }

    fn ensure_available(&self, pump: HostPumpState) -> Result<(), HostRegistrationError> {
        match pump {
            HostPumpState::Available => Ok(()),
            HostPumpState::Stopping => Err(HostRegistrationError::InvalidState),
            HostPumpState::Stopped { reason } => {
                Err(HostRegistrationError::HostUnavailable { reason })
            }
        }
    }

    fn ensure_ui_thread(&self) -> Result<(), HostRegistrationError> {
        if self.ui_thread.is_current() {
            Ok(())
        } else {
            Err(HostRegistrationError::WrongThread)
        }
    }

    fn lock_state(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HostRegistrationState>, HostRegistrationError> {
        self.state
            .lock()
            .map_err(|_| HostRegistrationError::SynchronizationPoisoned)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum RuntimeCoreError {
    WrongThread,
    Closed,
    BridgeAlreadyRegistered,
    SynchronizationPoisoned,
}

/// Internal Runtime owner. A host loop bridge is optional at creation.
#[allow(dead_code)]
pub(crate) struct RuntimeCore {
    ui_thread: UiThreadContext,
    cleanup: Arc<RuntimeCleanupQueue>,
    host: HostRegistration,
    dispatcher: Mutex<Option<Dispatcher>>,
}

#[allow(dead_code)]
impl RuntimeCore {
    pub(crate) fn new() -> Self {
        let ui_thread = UiThreadContext::capture();
        Self {
            ui_thread,
            cleanup: RuntimeCleanupQueue::new_for_ui_thread(ui_thread),
            host: HostRegistration::new_for_ui_thread(ui_thread),
            dispatcher: Mutex::new(None),
        }
    }

    pub(crate) fn is_ui_thread(&self) -> bool {
        self.ui_thread.is_current()
    }

    pub(crate) fn host_registration(&self) -> &HostRegistration {
        &self.host
    }

    pub(crate) fn cleanup_queue(&self) -> Arc<RuntimeCleanupQueue> {
        Arc::clone(&self.cleanup)
    }

    pub(crate) fn set_ui_loop_bridge(
        &self,
        bridge: &Arc<dyn UiLoopBridge>,
    ) -> Result<(), RuntimeCoreError> {
        if !self.is_ui_thread() {
            return Err(RuntimeCoreError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeCoreError::Closed);
        }

        let mut dispatcher = self
            .dispatcher
            .lock()
            .map_err(|_| RuntimeCoreError::SynchronizationPoisoned)?;
        if dispatcher.is_some() {
            return Err(RuntimeCoreError::BridgeAlreadyRegistered);
        }
        *dispatcher = Some(Dispatcher::new_for_ui_thread(
            Arc::clone(bridge),
            self.ui_thread,
        ));
        Ok(())
    }

    pub(crate) fn dispatcher(&self) -> Result<Option<Dispatcher>, RuntimeCoreError> {
        self.dispatcher
            .lock()
            .map(|dispatcher| dispatcher.clone())
            .map_err(|_| RuntimeCoreError::SynchronizationPoisoned)
    }
}

/// Thread-independent observation state for one accepted shutdown request.
/// The record holds no Runtime UI objects and outlives any individual waiter.
#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct ShutdownCompletion {
    ui_thread: UiThreadContext,
    inner: Arc<CompletionInner>,
}

/// Owned observation handle for one accepted request. Dropping it releases only this observer.
#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct ShutdownRequest {
    completion: ShutdownCompletion,
}

#[allow(dead_code)]
impl ShutdownRequest {
    pub(crate) fn query(&self) -> Result<ShutdownCompletionState, ShutdownCompletionError> {
        self.completion.query()
    }

    pub(crate) fn wait(&self) -> Result<ShutdownWaitResult, ShutdownCompletionError> {
        self.completion.wait()
    }
}

#[allow(dead_code)]
impl ShutdownCompletion {
    pub(crate) fn new() -> Self {
        Self::new_for_ui_thread(UiThreadContext::capture())
    }

    pub(crate) fn new_for_ui_thread(ui_thread: UiThreadContext) -> Self {
        Self {
            ui_thread,
            inner: Arc::new(CompletionInner {
                state: Mutex::new(ShutdownCompletionState::Pending),
                changed: Condvar::new(),
            }),
        }
    }

    pub(crate) fn query(&self) -> Result<ShutdownCompletionState, ShutdownCompletionError> {
        self.inner
            .state
            .lock()
            .map(|state| *state)
            .map_err(|_| ShutdownCompletionError::SynchronizationPoisoned)
    }

    pub(crate) fn wait(&self) -> Result<ShutdownWaitResult, ShutdownCompletionError> {
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| ShutdownCompletionError::SynchronizationPoisoned)?;
        loop {
            match *state {
                ShutdownCompletionState::Pending => {
                    state = self
                        .inner
                        .changed
                        .wait(state)
                        .map_err(|_| ShutdownCompletionError::SynchronizationPoisoned)?;
                }
                ShutdownCompletionState::Completed => return Ok(ShutdownWaitResult::Completed),
                ShutdownCompletionState::HostUnavailable { reason } => {
                    return Ok(ShutdownWaitResult::HostUnavailable { reason });
                }
            }
        }
    }

    /// Commit this only after Runtime cleanup has completed on the UI thread.
    fn mark_completed(&self) -> Result<(), ShutdownCompletionError> {
        self.ensure_ui_thread()?;
        let mut state = self.lock_state()?;
        *state = ShutdownCompletionState::Completed;
        self.inner.changed.notify_all();
        Ok(())
    }

    /// A stop notification ends current waiters but does not imply cleanup completed.
    pub(crate) fn mark_host_unavailable(&self, reason: u32) -> Result<(), ShutdownCompletionError> {
        let mut state = self.lock_state()?;
        if *state == ShutdownCompletionState::Pending {
            *state = ShutdownCompletionState::HostUnavailable { reason };
            self.inner.changed.notify_all();
        }
        Ok(())
    }

    /// Explicitly re-arm the same accepted request after its UI host resumes.
    pub(crate) fn retry_on_ui_thread(&self) -> Result<(), ShutdownCompletionError> {
        self.ensure_ui_thread()?;
        let mut state = self.lock_state()?;
        if !matches!(*state, ShutdownCompletionState::HostUnavailable { .. }) {
            return Err(ShutdownCompletionError::InvalidState);
        }
        *state = ShutdownCompletionState::Pending;
        Ok(())
    }

    fn ensure_ui_thread(&self) -> Result<(), ShutdownCompletionError> {
        if self.ui_thread.is_current() {
            Ok(())
        } else {
            Err(ShutdownCompletionError::WrongThread)
        }
    }

    fn lock_state(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, ShutdownCompletionState>, ShutdownCompletionError> {
        self.inner
            .state
            .lock()
            .map_err(|_| ShutdownCompletionError::SynchronizationPoisoned)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        HostBoundaryError, HostRegistration, HostRegistrationError, HostShutdownCommitError,
        RuntimeCleanupQueue, RuntimeCore, RuntimeCoreError, RuntimeState, ShutdownCompletion,
        ShutdownCompletionError, ShutdownCompletionState, ShutdownRequestMode,
        ShutdownRequestOutcome, ShutdownWaitResult, UiThreadContext,
    };
    use crate::dispatcher::{Dispatcher, PostedWork, UiLoopBridge};
    use crate::handle_registry::registry::{
        CleanupQueue, CoreObjectControl, HandleRegistry, ObjectIdentity,
    };
    use std::num::NonZeroU64;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn runtime_core_registers_bridge_once_on_owner_thread() {
        let runtime = Arc::new(RuntimeCore::new());
        assert!(runtime.is_ui_thread());
        let host_scope = runtime
            .host_registration()
            .begin_event()
            .expect("owner begins host event");
        runtime
            .host_registration()
            .end_event(host_scope)
            .expect("owner ends host event");
        assert!(runtime.dispatcher().expect("dispatcher state").is_none());
        assert_eq!(runtime.cleanup_queue().state(), RuntimeState::Running);

        let worker_runtime = Arc::clone(&runtime);
        let wrong_thread = thread::spawn(move || {
            let bridge: Arc<dyn UiLoopBridge> = Arc::new(TestUiLoopBridge::default());
            let result = worker_runtime.set_ui_loop_bridge(&bridge);
            (result, Arc::strong_count(&bridge))
        })
        .join()
        .expect("worker registration attempt completes");
        assert_eq!(wrong_thread.0, Err(RuntimeCoreError::WrongThread));
        assert_eq!(wrong_thread.1, 1);
        assert!(runtime.dispatcher().expect("dispatcher state").is_none());

        let bridge = Arc::new(TestUiLoopBridge::default());
        let bridge_trait: Arc<dyn UiLoopBridge> = bridge.clone();
        runtime
            .set_ui_loop_bridge(&bridge_trait)
            .expect("owner thread installs bridge");
        assert_eq!(Arc::strong_count(&bridge), 3);
        assert!(runtime.dispatcher().expect("dispatcher state").is_some());
        let duplicate_bridge: Arc<dyn UiLoopBridge> = Arc::new(TestUiLoopBridge::default());
        assert_eq!(
            runtime.set_ui_loop_bridge(&duplicate_bridge),
            Err(RuntimeCoreError::BridgeAlreadyRegistered)
        );
        assert_eq!(Arc::strong_count(&duplicate_bridge), 1);
        runtime
            .dispatcher()
            .expect("installed dispatcher")
            .expect("bridge registered")
            .post(PostedWork::callback(Box::new(|| {})))
            .expect("dispatcher keeps the first bridge");
        assert_eq!(bridge.wakes.load(Ordering::SeqCst), 1);

        runtime.cleanup.begin_shutdown().expect("begin shutdown");
        let late_bridge: Arc<dyn UiLoopBridge> = Arc::new(TestUiLoopBridge::default());
        assert_eq!(
            runtime.set_ui_loop_bridge(&late_bridge),
            Err(RuntimeCoreError::Closed)
        );
        assert_eq!(Arc::strong_count(&late_bridge), 1);
        assert!(
            runtime
                .dispatcher()
                .expect("installed dispatcher remains")
                .is_some()
        );
    }

    #[test]
    fn ui_thread_context_distinguishes_creator_from_worker() {
        let context = UiThreadContext::capture();
        assert!(context.is_current());

        let worker_context = thread::spawn(move || context.is_current())
            .join()
            .expect("worker query completes");
        assert!(!worker_context);
        assert!(context.is_current());
    }

    #[derive(Default)]
    struct TestUiLoopBridge {
        wakes: AtomicUsize,
    }

    impl UiLoopBridge for TestUiLoopBridge {
        fn schedule_wake(&self) {
            self.wakes.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct WakeSignalBridge {
        wake_sender: mpsc::Sender<()>,
    }

    impl UiLoopBridge for WakeSignalBridge {
        fn schedule_wake(&self) {
            let _ = self.wake_sender.send(());
        }
    }

    #[test]
    fn shutdown_drains_reserved_finalizers_before_closed() {
        let runtime = RuntimeCleanupQueue::new();
        let registry = HandleRegistry::<u8, u64>::new();
        let control = CoreObjectControl::new(
            1,
            ObjectIdentity::new(8, NonZeroU64::new(2).expect("nonzero generation")),
            runtime.clone(),
        );
        let token = registry
            .register(3, 1, Arc::clone(&control))
            .expect("register object");
        let shell_control = CoreObjectControl::new(
            1,
            ObjectIdentity::new(9, NonZeroU64::new(1).expect("nonzero generation")),
            runtime.clone(),
        );
        let shell_token = registry
            .register(3, 1, shell_control)
            .expect("register object that remains publicly owned");

        let released = registry.release(token).expect("accept final release");
        assert!(released.cleanup_accepted);
        assert_eq!(runtime.queued_count(), 1);

        runtime.begin_shutdown().expect("begin shutdown");
        assert_eq!(runtime.state(), RuntimeState::Closing);
        assert_eq!(runtime.queued_count(), 1);
        runtime.drain_shutdown().expect("drain finalizers");
        assert_eq!(runtime.state(), RuntimeState::Closed);
        assert_eq!(runtime.queued_count(), 0);

        let closed_shell_release = registry
            .release(shell_token)
            .expect("closed shell release does not access cleaned UI state");
        assert!(!closed_shell_release.cleanup_accepted);
    }

    #[test]
    fn closed_runtime_shell_releases_on_worker_without_ui_cleanup() {
        let runtime = RuntimeCleanupQueue::new();
        let registry = Arc::new(HandleRegistry::<u8, u64>::new());
        let control = CoreObjectControl::new(
            1,
            ObjectIdentity::new(12, NonZeroU64::new(1).expect("nonzero generation")),
            runtime.clone(),
        );
        let control_probe = Arc::downgrade(&control);
        let token = registry
            .register(4, 1, control.clone())
            .expect("register Runtime object");
        drop(control);

        runtime
            .drain_shutdown()
            .expect("clean UI state and close Runtime");
        assert_eq!(runtime.state(), RuntimeState::Closed);
        assert_eq!(runtime.queued_count(), 0);

        let registry_for_worker = Arc::clone(&registry);
        let release = thread::spawn(move || registry_for_worker.release(token))
            .join()
            .expect("worker release joins");
        assert_eq!(
            release,
            Ok(crate::handle_registry::registry::ReleaseOutcome {
                cleanup_accepted: false,
            })
        );
        assert!(
            control_probe.upgrade().is_none(),
            "last closed shell owner is released by the worker"
        );
        assert_eq!(runtime.queued_count(), 0);
        assert_eq!(
            registry.release(token),
            Err(crate::handle_registry::registry::RegistryError::InvalidToken)
        );
    }

    #[test]
    fn completion_record_distinguishes_host_stop_and_allows_ui_retry() {
        let no_op_host = HostRegistration::new();
        let no_op_dispatcher = Dispatcher::new(Arc::new(TestUiLoopBridge::default()));
        let no_op_completion = ShutdownCompletion::new();
        let no_op_cleanup = RuntimeCleanupQueue::new();
        let no_op_scope = no_op_host.begin_event().expect("begin no-op boundary");
        assert!(
            !no_op_host
                .end_event_and_commit_shutdown(no_op_scope, &no_op_dispatcher, &no_op_cleanup,)
                .expect("no pending shutdown remains a no-op")
        );
        assert_eq!(no_op_cleanup.state(), RuntimeState::Running);
        assert_eq!(
            no_op_completion.query().expect("no-op completion state"),
            ShutdownCompletionState::Pending
        );
        assert_eq!(
            no_op_host.before_stop(&no_op_dispatcher, &no_op_cleanup),
            Ok(false)
        );

        let graceful_host = HostRegistration::new();
        let graceful_bridge = Arc::new(TestUiLoopBridge::default());
        let graceful_dispatcher = Dispatcher::new(graceful_bridge);
        let graceful_completion = ShutdownCompletion::new();
        let graceful_cleanup = RuntimeCleanupQueue::new();
        let cleanup_count = Arc::new(AtomicUsize::new(0));
        let cleanup_count_for_callback = Arc::clone(&cleanup_count);
        graceful_cleanup
            .track_ui_state(Box::new(move || {
                cleanup_count_for_callback.fetch_add(1, Ordering::SeqCst);
            }))
            .expect("track graceful-stop UI state");
        graceful_host
            .request_shutdown(&graceful_dispatcher, &graceful_completion)
            .expect("admit graceful-stop request");
        assert!(
            graceful_host
                .before_stop(&graceful_dispatcher, &graceful_cleanup,)
                .expect("before_stop drains accepted shutdown")
        );
        assert_eq!(graceful_cleanup.state(), RuntimeState::Closed);
        assert_eq!(cleanup_count.load(Ordering::SeqCst), 1);
        assert_eq!(
            graceful_completion.query().expect("graceful completion"),
            ShutdownCompletionState::Completed
        );
        graceful_host
            .notify_stopped(17)
            .expect("stop notification follows graceful cleanup");
        assert_eq!(
            graceful_completion
                .query()
                .expect("stop does not overwrite completion"),
            ShutdownCompletionState::Completed
        );

        let receipt_host = HostRegistration::new();
        let receipt_dispatcher = Dispatcher::new(Arc::new(TestUiLoopBridge::default()));
        let receipt_cleanup = RuntimeCleanupQueue::new();
        let receipt_cleanup_count = Arc::new(AtomicUsize::new(0));
        let receipt_cleanup_count_for_callback = Arc::clone(&receipt_cleanup_count);
        receipt_cleanup
            .track_ui_state(Box::new(move || {
                receipt_cleanup_count_for_callback.fetch_add(1, Ordering::SeqCst);
            }))
            .expect("track receipt cleanup");
        let receipt_completion = ShutdownCompletion::new();
        let receipt_record = Arc::downgrade(&receipt_completion.inner);
        let receipt = receipt_host
            .request_shutdown_receipt(&receipt_dispatcher, &receipt_completion)
            .expect("admit request and create owned receipt");
        let receipt_for_query = receipt.clone();
        assert_eq!(
            thread::spawn(move || receipt_for_query.query())
                .join()
                .expect("cross-thread receipt query joins")
                .expect("cross-thread receipt query succeeds"),
            ShutdownCompletionState::Pending
        );
        drop(receipt);
        drop(receipt_completion);
        assert!(
            receipt_record.upgrade().is_some(),
            "accepted shutdown remains owned after receipt and observer are dropped"
        );
        let receipt_scope = receipt_host
            .begin_event()
            .expect("begin receipt shutdown boundary");
        assert!(
            receipt_host
                .end_event_and_commit_shutdown(
                    receipt_scope,
                    &receipt_dispatcher,
                    &receipt_cleanup,
                )
                .expect("commit request after receipt release")
        );
        assert_eq!(receipt_cleanup.state(), RuntimeState::Closed);
        assert_eq!(receipt_cleanup_count.load(Ordering::SeqCst), 1);
        assert!(
            receipt_record.upgrade().is_none(),
            "host releases the completion record after cleanup commit"
        );

        let (wake_sender, wake_receiver) = mpsc::channel();
        let sync_host = Arc::new(HostRegistration::new());
        let sync_dispatcher = Dispatcher::new(Arc::new(WakeSignalBridge { wake_sender }));
        let sync_completion = ShutdownCompletion::new();
        let request_host = Arc::clone(&sync_host);
        let request_dispatcher = sync_dispatcher.clone();
        let request_completion = sync_completion.clone();
        let sync_request = thread::spawn(move || {
            request_host.request_shutdown_with_mode(
                &request_dispatcher,
                &request_completion,
                ShutdownRequestMode::Sync,
            )
        });
        wake_receiver
            .recv()
            .expect("worker synchronous request is admitted before waiting");
        let sync_cleanup = RuntimeCleanupQueue::new();
        let sync_scope = sync_host
            .begin_event()
            .expect("begin sync shutdown boundary");
        assert!(
            sync_host
                .end_event_and_commit_shutdown(sync_scope, &sync_dispatcher, &sync_cleanup,)
                .expect("commit worker synchronous request")
        );
        assert_eq!(
            sync_request
                .join()
                .expect("worker sync request joins")
                .expect("worker sync request completes"),
            ShutdownRequestOutcome::Completed
        );

        let completion = ShutdownCompletion::new();
        let host = Arc::new(HostRegistration::new());
        let dispatcher_bridge = Arc::new(TestUiLoopBridge::default());
        let dispatcher = Dispatcher::new(dispatcher_bridge.clone());
        let ui_scope = host
            .begin_event()
            .expect("begin callback-like request scope");
        assert_eq!(
            host.request_shutdown_with_mode(&dispatcher, &completion, ShutdownRequestMode::Async,),
            Ok(ShutdownRequestOutcome::Pending)
        );
        assert_eq!(
            host.request_shutdown_with_mode(&dispatcher, &completion, ShutdownRequestMode::Sync,),
            Ok(ShutdownRequestOutcome::Pending)
        );
        assert!(host.end_event(ui_scope).expect("end request scope"));
        let coalesced_completion = ShutdownCompletion::new();
        host.request_shutdown(&dispatcher, &coalesced_completion)
            .expect("admit a distinct request onto pending shutdown");
        let abandoned_completion = ShutdownCompletion::new();
        let abandoned_record = Arc::downgrade(&abandoned_completion.inner);
        host.request_shutdown(&dispatcher, &abandoned_completion)
            .expect("admit request whose observer is released");
        drop(abandoned_completion);
        assert!(
            abandoned_record.upgrade().is_some(),
            "host keeps accepted completion alive after observer release"
        );
        assert_eq!(dispatcher_bridge.wakes.load(Ordering::SeqCst), 1);
        assert_eq!(
            completion.query().expect("initial state"),
            ShutdownCompletionState::Pending
        );

        let outer_scope = host.begin_event().expect("begin outer host event");
        let inner_scope = host.begin_event().expect("begin nested host event");
        let nested_host = HostRegistration::new();
        let nested_host_scope = nested_host
            .begin_event()
            .expect("begin nested event from another Runtime");
        assert_eq!(
            host.end_event(inner_scope),
            Err(HostRegistrationError::InvalidContext)
        );
        assert_eq!(
            host.before_stop(&dispatcher, &no_op_cleanup),
            Err(HostShutdownCommitError::Boundary(HostBoundaryError::Host(
                HostRegistrationError::InvalidContext
            )))
        );
        assert!(
            !nested_host
                .end_event(nested_host_scope)
                .expect("end nested Runtime event")
        );
        assert!(!host.end_event(inner_scope).expect("end nested scope"));
        assert!(host.end_event(outer_scope).expect("end outer scope"));
        let next_scope = host.begin_event().expect("begin next host event");
        assert!(next_scope > inner_scope, "scope IDs are never reused");
        assert!(host.end_event(next_scope).expect("end next outer scope"));
        let stopped_waiter = completion.clone();
        let stopped_barrier = Arc::new(Barrier::new(2));
        let stopped_waiter_barrier = Arc::clone(&stopped_barrier);
        let stopped_waiter = thread::spawn(move || {
            stopped_waiter_barrier.wait();
            stopped_waiter.wait().expect("host-stop result")
        });
        stopped_barrier.wait();
        let stopping_host = Arc::clone(&host);
        thread::spawn(move || {
            stopping_host
                .notify_stopped(7)
                .expect("publish host stop from any thread");
        })
        .join()
        .expect("host stop notification completes");
        assert_eq!(
            stopped_waiter.join().expect("stopped waiter completes"),
            ShutdownWaitResult::HostUnavailable { reason: 7 }
        );
        assert!(
            abandoned_record.upgrade().is_none(),
            "host releases stopped completion after terminal notification"
        );

        assert_eq!(
            completion.query().expect("host-stop state"),
            ShutdownCompletionState::HostUnavailable { reason: 7 }
        );
        assert_eq!(
            coalesced_completion
                .query()
                .expect("coalesced host-stop state"),
            ShutdownCompletionState::HostUnavailable { reason: 7 }
        );
        let stopped_bridge = Arc::new(TestUiLoopBridge::default());
        let stopped_dispatcher = Dispatcher::new(stopped_bridge.clone());
        let rejected_completion = ShutdownCompletion::new();
        assert_eq!(
            host.request_shutdown(&stopped_dispatcher, &rejected_completion),
            Err(HostRegistrationError::HostUnavailable { reason: 7 })
        );
        assert_eq!(stopped_bridge.wakes.load(Ordering::SeqCst), 0);
        assert!(
            !stopped_dispatcher
                .run_pending_shutdown(|| {})
                .expect("stopped host request did not arm dispatcher")
        );
        assert_eq!(
            rejected_completion.query().expect("rejected record state"),
            ShutdownCompletionState::Pending
        );

        host.resume_on_ui_thread().expect("same UI host resumes");
        let idle_bridge = Arc::new(TestUiLoopBridge::default());
        let idle_dispatcher = Dispatcher::new(idle_bridge.clone());
        assert_eq!(
            host.retry_on_ui_thread(&completion, &idle_dispatcher),
            Err(HostRegistrationError::InvalidState)
        );
        assert_eq!(idle_bridge.wakes.load(Ordering::SeqCst), 0);
        assert_eq!(
            completion.query().expect("idle dispatcher retry state"),
            ShutdownCompletionState::HostUnavailable { reason: 7 }
        );
        let closed_bridge = Arc::new(TestUiLoopBridge::default());
        let closed_dispatcher = Dispatcher::new(closed_bridge.clone());
        closed_dispatcher
            .close_and_cancel()
            .expect("prepare closed dispatcher");
        assert_eq!(
            host.retry_on_ui_thread(&completion, &closed_dispatcher),
            Err(HostRegistrationError::InvalidState)
        );
        assert_eq!(
            completion.query().expect("rejected retry state"),
            ShutdownCompletionState::HostUnavailable { reason: 7 }
        );
        host.retry_on_ui_thread(&completion, &dispatcher)
            .expect("UI host retry explicitly re-arms request");
        host.retry_on_ui_thread(&coalesced_completion, &dispatcher)
            .expect("retry distinct coalesced request");
        assert_eq!(dispatcher_bridge.wakes.load(Ordering::SeqCst), 3);
        let unaccepted_completion = ShutdownCompletion::new();
        assert_eq!(
            host.request_shutdown(&closed_dispatcher, &unaccepted_completion),
            Err(HostRegistrationError::InvalidState)
        );
        assert_eq!(closed_bridge.wakes.load(Ordering::SeqCst), 0);
        assert_eq!(
            unaccepted_completion
                .query()
                .expect("closed dispatcher record state"),
            ShutdownCompletionState::Pending
        );
        assert_eq!(
            completion.query().expect("retry is pending"),
            ShutdownCompletionState::Pending
        );

        let completed_waiter = completion.clone();
        let completed_barrier = Arc::new(Barrier::new(2));
        let completed_waiter_barrier = Arc::clone(&completed_barrier);
        let completed_waiter = thread::spawn(move || {
            completed_waiter_barrier.wait();
            completed_waiter.wait().expect("completion result")
        });
        completed_barrier.wait();
        let runtime = RuntimeCleanupQueue::new();
        let cleanup_count = Arc::new(AtomicUsize::new(0));
        let cleanup_count_for_callback = Arc::clone(&cleanup_count);
        runtime
            .track_ui_state(Box::new(move || {
                cleanup_count_for_callback.fetch_add(1, Ordering::SeqCst);
            }))
            .expect("track UI state for shutdown cleanup");
        let request_host = Arc::clone(&host);
        let request_dispatcher = dispatcher.clone();
        let request_completion = completion.clone();
        thread::spawn(move || {
            request_host.request_shutdown(&request_dispatcher, &request_completion)
        })
        .join()
        .expect("worker request admission completes")
        .expect("shutdown request is admitted");
        let duplicate_host = Arc::clone(&host);
        let duplicate_dispatcher = dispatcher.clone();
        let duplicate_completion = completion.clone();
        thread::spawn(move || {
            duplicate_host.request_shutdown(&duplicate_dispatcher, &duplicate_completion)
        })
        .join()
        .expect("duplicate worker request completes")
        .expect("same request coalesces");
        assert_eq!(dispatcher_bridge.wakes.load(Ordering::SeqCst), 3);
        let shutdown_scope = host.begin_event().expect("begin shutdown boundary");
        assert!(
            host.end_event_and_commit_shutdown(shutdown_scope, &dispatcher, &runtime)
                .expect("outer boundary drains and commits shutdown")
        );
        assert_eq!(
            completed_waiter.join().expect("completed waiter wakes"),
            ShutdownWaitResult::Completed
        );
        assert_eq!(runtime.state(), RuntimeState::Closed);
        assert_eq!(cleanup_count.load(Ordering::SeqCst), 1);
        assert_eq!(
            coalesced_completion
                .query()
                .expect("coalesced request completes with shared cleanup"),
            ShutdownCompletionState::Completed
        );

        assert_eq!(host.before_stop(&dispatcher, &runtime), Ok(false));
        host.notify_stopped(9)
            .expect("late host-stop does not overwrite completion");
        assert_eq!(
            completion.query().expect("completion is terminal"),
            ShutdownCompletionState::Completed
        );

        let wrong_thread = completion.clone();
        assert_eq!(
            thread::spawn(move || wrong_thread.retry_on_ui_thread())
                .join()
                .expect("wrong-thread retry check completes"),
            Err(ShutdownCompletionError::WrongThread)
        );
    }
}

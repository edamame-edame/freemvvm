use crate::dispatcher::{DispatchError, Dispatcher};
use crate::handle_registry::registry::{CleanupQueue, CleanupReservation, ObjectIdentity};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::thread::ThreadId;

type Finalizer = Box<dyn FnOnce() + Send>;

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
    ui_thread: ThreadId,
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
        Arc::new(Self {
            inner: Arc::new(QueueInner {
                ui_thread: std::thread::current().id(),
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

    /// Publish completion only after all accepted cleanup has reached Closed.
    pub(crate) fn drain_shutdown_and_complete(
        &self,
        completion: &ShutdownCompletion,
    ) -> Result<(), ShutdownCommitError> {
        self.drain_shutdown()
            .map_err(ShutdownCommitError::Cleanup)?;
        completion
            .mark_completed()
            .map_err(ShutdownCommitError::Completion)
    }

    fn ensure_ui_thread(&self) -> Result<(), ShutdownError> {
        if std::thread::current().id() == self.inner.ui_thread {
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
    completion_records: Vec<Weak<CompletionInner>>,
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
    ui_thread: ThreadId,
    scope_owner: Arc<()>,
    state: Mutex<HostRegistrationState>,
}

#[allow(dead_code)]
impl HostRegistration {
    pub(crate) fn new() -> Self {
        Self {
            ui_thread: std::thread::current().id(),
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
        completion: &ShutdownCompletion,
    ) -> Result<bool, HostShutdownCommitError> {
        let mut commit_result = None;
        let dispatched = self
            .end_event_and_dispatch_shutdown(scope_id, dispatcher, || {
                commit_result = Some(cleanup.drain_shutdown_and_complete(completion));
            })
            .map_err(HostShutdownCommitError::Boundary)?;
        if !dispatched {
            return Ok(false);
        }
        commit_result
            .expect("shutdown handler stores its commit result")
            .map_err(HostShutdownCommitError::Commit)?;
        Ok(true)
    }

    /// Begin host shutdown only at an outer event boundary.
    pub(crate) fn before_stop(&self) -> Result<(), HostRegistrationError> {
        self.ensure_ui_thread()?;
        let mut state = self.lock_state()?;
        if HOST_SCOPE_STACK.with(|stack| !stack.borrow().is_empty()) {
            return Err(HostRegistrationError::InvalidContext);
        }
        match state.pump {
            HostPumpState::Available => {
                state.pump = HostPumpState::Stopping;
                Ok(())
            }
            HostPumpState::Stopping => Ok(()),
            HostPumpState::Stopped { .. } => Err(HostRegistrationError::InvalidState),
        }
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
        state
            .completion_records
            .retain(|record| record.strong_count() > 0);
        state
            .completion_records
            .try_reserve(1)
            .map_err(|_| HostRegistrationError::OutOfMemory)?;
        state
            .completion_records
            .push(Arc::downgrade(&completion.inner));
        Ok(())
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
            if let Some(record) = record.upgrade() {
                let completion = ShutdownCompletion {
                    ui_thread: self.ui_thread,
                    inner: record,
                };
                if let Err(error) = completion.mark_host_unavailable(reason) {
                    first_error.get_or_insert(HostRegistrationError::Completion(error));
                }
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
    ) -> Result<(), HostRegistrationError> {
        self.ensure_ui_thread()?;
        if self.ui_thread != completion.ui_thread {
            return Err(HostRegistrationError::InvalidContext);
        }
        let mut state = self.lock_state()?;
        self.ensure_available(state.pump)?;
        state
            .completion_records
            .retain(|record| record.strong_count() > 0);
        state
            .completion_records
            .try_reserve(1)
            .map_err(|_| HostRegistrationError::OutOfMemory)?;
        completion
            .retry_on_ui_thread()
            .map_err(HostRegistrationError::Completion)?;
        state
            .completion_records
            .push(Arc::downgrade(&completion.inner));
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
        if std::thread::current().id() == self.ui_thread {
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

/// Thread-independent observation state for one accepted shutdown request.
/// The record holds no Runtime UI objects and outlives any individual waiter.
#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct ShutdownCompletion {
    ui_thread: ThreadId,
    inner: Arc<CompletionInner>,
}

#[allow(dead_code)]
impl ShutdownCompletion {
    pub(crate) fn new() -> Self {
        Self {
            ui_thread: std::thread::current().id(),
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
        if std::thread::current().id() == self.ui_thread {
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
        HostRegistration, HostRegistrationError, RuntimeCleanupQueue, RuntimeState,
        ShutdownCompletion, ShutdownCompletionError, ShutdownCompletionState, ShutdownWaitResult,
    };
    use crate::dispatcher::{Dispatcher, UiLoopBridge};
    use crate::handle_registry::registry::{
        CleanupQueue, CoreObjectControl, HandleRegistry, ObjectIdentity,
    };
    use std::num::NonZeroU64;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[derive(Default)]
    struct TestUiLoopBridge;

    impl UiLoopBridge for TestUiLoopBridge {
        fn schedule_wake(&self) {}
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
    fn completion_record_distinguishes_host_stop_and_allows_ui_retry() {
        let completion = ShutdownCompletion::new();
        let host = Arc::new(HostRegistration::new());
        host.register_completion(&completion)
            .expect("register pending shutdown with host");
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
            host.before_stop(),
            Err(HostRegistrationError::InvalidContext)
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
        host.before_stop()
            .expect("begin stop at outer event boundary");

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

        assert_eq!(
            completion.query().expect("host-stop state"),
            ShutdownCompletionState::HostUnavailable { reason: 7 }
        );
        host.resume_on_ui_thread().expect("same UI host resumes");
        host.retry_on_ui_thread(&completion)
            .expect("UI host retry explicitly re-arms request");
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
        let dispatcher = Dispatcher::new(Arc::new(TestUiLoopBridge));
        let no_shutdown_scope = host.begin_event().expect("begin no-op boundary");
        assert!(
            !host
                .end_event_and_commit_shutdown(
                    no_shutdown_scope,
                    &dispatcher,
                    &runtime,
                    &completion,
                )
                .expect("no pending shutdown remains a no-op")
        );
        assert_eq!(runtime.state(), RuntimeState::Running);
        assert_eq!(
            completion.query().expect("still-pending request"),
            ShutdownCompletionState::Pending
        );

        dispatcher
            .request_shutdown()
            .expect("request priority shutdown");
        let shutdown_scope = host.begin_event().expect("begin shutdown boundary");
        assert!(
            host.end_event_and_commit_shutdown(shutdown_scope, &dispatcher, &runtime, &completion,)
                .expect("outer boundary drains and commits shutdown")
        );
        assert_eq!(
            completed_waiter.join().expect("completed waiter wakes"),
            ShutdownWaitResult::Completed
        );
        assert_eq!(runtime.state(), RuntimeState::Closed);
        assert_eq!(cleanup_count.load(Ordering::SeqCst), 1);

        host.before_stop()
            .expect("host can stop again after completion");
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

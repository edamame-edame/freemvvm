use crate::binding::{BindingConnectionId, BindingGraph, BindingGraphError};
use crate::command::{CommandError, OperationIdIssuer};
use crate::dispatcher::{
    DispatchError, Dispatcher, PostedWork, ShutdownRequestAdmission, UiLoopBridge,
};
use crate::handle_registry::HandleToken;
use crate::handle_registry::registry::{
    CleanupQueue, CleanupReservation, CoreObjectControl, HandleRegistry, InternalOwner,
    ObjectIdentity, RegistryError, ValidationPin,
};
use crate::object_store::{UiObjectStore, UiObjectStoreError};
use crate::platform::frame::{FramePhase, FrameTicket};
use crate::platform::scene::SceneSnapshot;
use crate::property::{
    PropertyChangeQueue, PropertyHandleKind, PropertyState, PropertyValidator, Subscription,
    ValidationError, ValidatorResult,
};
use crate::ui_controls::{UiControlKind, UiControlProperty, UiControlState};
use crate::ui_focus::FocusManager;
use crate::ui_layout::{
    Align, IntrinsicMeasureProvider, LayoutEngine, LayoutError, LayoutNodeSnapshot, LayoutSnapshot,
    Orientation, Size,
};
use crate::ui_tree::{LogicalNode, RuntimeTreeStore, TreeError};
use crate::ui_window::{
    CloseDecision, FactoryWindowPeer, PlatformFrameCompletionSink, PlatformOperationError,
    PlatformRuntimeFactory, WindowLifecycleError, WindowPeerAdapter,
};
use crate::value::{
    BindingCopyError, OwnedUtf8Error, TypedValue, ValueCommit, ValueKind, ValueKindMismatch,
};
use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::thread::ThreadId;

type Finalizer = Box<dyn FnOnce() + Send>;

struct RuntimeFlagGuard<'a>(&'a AtomicBool);

impl Drop for RuntimeFlagGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

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
    pending_work_waker: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
    changed: Condvar,
}

/// Runtime-owned finalizer queue. Tickets are accepted before token release commits,
/// and their callbacks run only while the Runtime UI thread drains the queue.
pub(crate) struct RuntimeCleanupQueue {
    inner: Arc<QueueInner>,
}

/// Routes object-specific finalizers through the Runtime queue and its UI state store.
struct RuntimeObjectCleanupQueue {
    queue: Arc<RuntimeCleanupQueue>,
    ui_objects: Arc<Mutex<UiObjectStore<Box<dyn Any + Send>>>>,
    ui_tree: Arc<Mutex<RuntimeTreeStore<RuntimeIdentity>>>,
    ui_state_cleared: Arc<AtomicBool>,
    property_changes: Arc<Mutex<PropertyChangeQueue>>,
    property_delivery: Arc<Mutex<Option<VecDeque<PropertyDeliveryItem>>>>,
    subscription_owners: Arc<Mutex<HashMap<ObjectIdentity, Vec<Arc<Subscription>>>>>,
}

impl CleanupQueue for RuntimeObjectCleanupQueue {
    fn track_ui_state(&self, on_shutdown: Box<dyn Fn() + Send + Sync>) -> Result<(), ()> {
        self.queue.track_ui_state(on_shutdown)
    }

    fn reserve(
        &self,
        identity: ObjectIdentity,
        on_complete: Finalizer,
    ) -> Result<Box<dyn CleanupReservation>, ()> {
        let ui_objects = Arc::clone(&self.ui_objects);
        let ui_state_cleared = Arc::clone(&self.ui_state_cleared);
        let property_changes = Arc::clone(&self.property_changes);
        let property_delivery = Arc::clone(&self.property_delivery);
        let subscription_owners = Arc::clone(&self.subscription_owners);
        let ui_tree = Arc::clone(&self.ui_tree);
        let finalizer = Box::new(move || {
            property_changes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(identity);
            let mut pending_delivery = property_delivery
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(delivery) = pending_delivery.as_mut() {
                delivery.retain(|item| item.identity != identity);
                if delivery.is_empty() {
                    *pending_delivery = None;
                }
            }
            drop(pending_delivery);
            let tree_node = {
                let mut tree = ui_tree
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                tree.remove_for_cleanup(identity)
            };
            match tree_node {
                Ok(node) => drop(node),
                Err(error) => {
                    debug_assert!(false, "unexpected Logical Tree cleanup failure: {error:?}")
                }
            }
            let subscriptions = subscription_owners
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&identity)
                .unwrap_or_default();
            for subscription in subscriptions {
                subscription.cancel();
            }
            let state = {
                let mut store = ui_objects
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                store.destroy(identity)
            };
            match state {
                Ok(mut state) => {
                    if let Some(control) = state.downcast_mut::<UiControlState>() {
                        if let Some(window) = &mut control.window {
                            window.shutdown(identity);
                        }
                    }
                    drop(state);
                }
                // Shutdown may clear the store before a racing release ticket is drained.
                Err(UiObjectStoreError::InvalidIdentity)
                    if ui_state_cleared.load(Ordering::Acquire) => {}
                Err(error) => debug_assert!(false, "unexpected object cleanup failure: {error:?}"),
            }
            on_complete();
        });
        self.queue.reserve(identity, finalizer)
    }
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
                pending_work_waker: Mutex::new(None),
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

    fn set_pending_work_waker(&self, waker: Arc<dyn Fn() + Send + Sync>) {
        *self
            .inner
            .pending_work_waker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::clone(&waker));
        if self.queued_count() > 0 {
            waker();
        }
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
                } else if !state.tracked_ui_state.is_empty() {
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

    /// Runs accepted object finalizers at a UI safe point without closing the Runtime.
    pub(crate) fn drain_pending(&self) -> Result<(), ShutdownError> {
        self.ensure_ui_thread()?;
        loop {
            let ticket = {
                let mut state = self.lock_state()?;
                if state.lifecycle != RuntimeState::Running {
                    return Ok(());
                }
                state.tickets.pop_front()
            };
            let Some(ticket) = ticket else {
                return Ok(());
            };
            let _identity = ticket.identity;
            (ticket.on_complete)();
            let mut state = self.lock_state()?;
            state.outstanding -= 1;
            self.inner.changed.notify_all();
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
        drop(state);
        if let Some(waker) = self
            .inner
            .pending_work_waker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .cloned()
        {
            waker();
        }
    }
}

impl Drop for RuntimeCleanupReservation {
    fn drop(&mut self) {
        let Some(unused_ticket) = self.ticket.take() else {
            return;
        };
        {
            let mut state = self
                .inner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match state.outstanding.checked_sub(1) {
                Some(outstanding) => state.outstanding = outstanding,
                None => debug_assert!(false, "discarded cleanup reservation was not outstanding"),
            }
            self.inner.changed.notify_all();
        }
        drop(unused_ticket);
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

    /// Close one scope, commit pending shutdown first, then run the outer safe point if still active.
    pub(crate) fn end_event_and_commit_shutdown_or_else<F>(
        &self,
        scope_id: u64,
        dispatcher: &Dispatcher,
        cleanup: &RuntimeCleanupQueue,
        on_running_boundary: F,
    ) -> Result<bool, HostShutdownCommitError>
    where
        F: FnOnce(),
    {
        let outermost = self
            .end_event(scope_id)
            .map_err(HostBoundaryError::Host)
            .map_err(HostShutdownCommitError::Boundary)?;
        if !outermost {
            return Ok(false);
        }

        if self.commit_pending_shutdown(dispatcher, cleanup)? {
            return Ok(true);
        }
        on_running_boundary();
        Ok(false)
    }

    fn commit_pending_shutdown(
        &self,
        dispatcher: &Dispatcher,
        cleanup: &RuntimeCleanupQueue,
    ) -> Result<bool, HostShutdownCommitError> {
        let mut commit_result = None;
        let dispatched = dispatcher
            .run_pending_shutdown(|| {
                commit_result = Some(self.drain_shutdown_and_complete_requests(cleanup));
            })
            .map_err(HostBoundaryError::Dispatch)
            .map_err(HostShutdownCommitError::Boundary)?;
        if dispatched {
            commit_result.expect("shutdown handler stores its commit result")?;
        }
        Ok(dispatched)
    }

    pub(crate) fn commit_pending_shutdown_at_safe_point(
        &self,
        dispatcher: &Dispatcher,
        cleanup: &RuntimeCleanupQueue,
    ) -> Result<bool, HostShutdownCommitError> {
        self.commit_pending_shutdown(dispatcher, cleanup)
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
                    DispatchError::Closed => HostRegistrationError::InvalidState,
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
                    DispatchError::Closed => HostRegistrationError::InvalidState,
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum RuntimeCoreError {
    IdentityUnavailable,
    CleanupRegistrationFailed,
    WrongThread,
    Closed,
    BridgeAlreadyRegistered,
    BridgeNotRegistered,
    SynchronizationPoisoned,
    Host(HostRegistrationError),
    Shutdown(HostShutdownCommitError),
    Dispatch(DispatchError),
    Property(RuntimePropertyError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RuntimeObjectRegistrationError {
    WrongThread,
    Closed,
    SynchronizationPoisoned,
    Store(UiObjectStoreError),
    Registry(RegistryError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum RuntimeTreeError {
    WrongThread,
    Closed,
    SynchronizationPoisoned,
    Registration(RuntimeObjectRegistrationError),
    Registry(RegistryError),
    Tree(TreeError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RuntimePropertyError {
    WrongThread,
    Closed,
    Unsupported,
    SynchronizationPoisoned,
    Store(UiObjectStoreError),
    Registry(RegistryError),
    WrongObjectState,
    ValueKindMismatch(ValueKindMismatch),
    BindingGraph(BindingGraphError),
    BindingValueCopy(BindingCopyError),
    BindingPropagationOutOfMemory,
    NotificationQueueOutOfMemory,
    SubscriptionRegistryOutOfMemory,
    ValidationFailed(Option<Arc<ValidationError>>),
    ValidatorFailed,
    ValidatorReentrancy,
    ValidatorAlreadyRegistered,
    Dispatch(DispatchError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RuntimeBindingError {
    Property(RuntimePropertyError),
    Registry(RegistryError),
    Store(UiObjectStoreError),
    WrongObjectState,
    Graph(BindingGraphError),
    InitialCopy(BindingCopyError),
    SourceSnapshotCopy(OwnedUtf8Error),
    SynchronizationPoisoned,
    NotificationQueueOutOfMemory,
    PropagationPreparationOutOfMemory,
    BindingOwnerOutOfMemory,
}

impl RuntimeBindingError {
    fn into_property_delivery_error(self) -> RuntimePropertyError {
        match self {
            Self::Property(error) => error,
            Self::Registry(error) => RuntimePropertyError::Registry(error),
            Self::Store(error) => RuntimePropertyError::Store(error),
            Self::WrongObjectState => RuntimePropertyError::WrongObjectState,
            Self::Graph(error) => RuntimePropertyError::BindingGraph(error),
            Self::InitialCopy(error) => RuntimePropertyError::BindingValueCopy(error),
            Self::SourceSnapshotCopy(error) => {
                RuntimePropertyError::BindingValueCopy(BindingCopyError::Copy(error))
            }
            Self::SynchronizationPoisoned => RuntimePropertyError::SynchronizationPoisoned,
            Self::NotificationQueueOutOfMemory => {
                RuntimePropertyError::NotificationQueueOutOfMemory
            }
            Self::PropagationPreparationOutOfMemory => {
                RuntimePropertyError::BindingPropagationOutOfMemory
            }
            Self::BindingOwnerOutOfMemory => RuntimePropertyError::BindingPropagationOutOfMemory,
        }
    }
}

/// Process-unique internal identity for one Runtime owner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct RuntimeIdentity(NonZeroU64);

impl RuntimeIdentity {
    fn issue() -> Option<Self> {
        crate::handle_registry::issue_runtime_identity().map(Self)
    }

    #[cfg(test)]
    fn get(self) -> NonZeroU64 {
        self.0
    }
}

/// Internal Runtime owner. A host loop bridge is optional at creation.
#[allow(dead_code)]
pub(crate) struct RuntimeCore {
    identity: RuntimeIdentity,
    platform_factory: Option<Arc<dyn PlatformRuntimeFactory>>,
    command_operation_ids: OperationIdIssuer,
    ui_thread: UiThreadContext,
    cleanup: Arc<RuntimeCleanupQueue>,
    ui_objects: Arc<Mutex<UiObjectStore<Box<dyn Any + Send>>>>,
    ui_tree: Arc<Mutex<RuntimeTreeStore<RuntimeIdentity>>>,
    focus: Mutex<FocusManager>,
    binding_graph: Arc<Mutex<BindingGraph>>,
    active_bindings: Arc<Mutex<HashMap<BindingConnectionId, RuntimeBindingOwner>>>,
    binding_error_delivery: Arc<Mutex<VecDeque<BindingErrorDeliveryItem>>>,
    property_changes: Arc<Mutex<PropertyChangeQueue>>,
    property_delivery: Arc<Mutex<Option<VecDeque<PropertyDeliveryItem>>>>,
    property_delivery_active: AtomicBool,
    validator_active: AtomicBool,
    subscription_owners: Arc<Mutex<HashMap<ObjectIdentity, Vec<Arc<Subscription>>>>>,
    object_cleanup: Arc<RuntimeObjectCleanupQueue>,
    host: HostRegistration,
    dispatcher: Mutex<Option<Dispatcher>>,
    shutdown_completion: Mutex<Option<ShutdownCompletion>>,
}

#[derive(Clone, Debug, PartialEq)]
struct LayoutTreeInput {
    identity: ObjectIdentity,
    children: Vec<Self>,
}

fn capture_layout_tree(
    tree: &RuntimeTreeStore<RuntimeIdentity>,
    identity: ObjectIdentity,
) -> Result<LayoutTreeInput, LayoutError> {
    let count = tree
        .child_count(identity)
        .map_err(|_| LayoutError::InvalidTree)?;
    let mut children = Vec::new();
    children
        .try_reserve(count)
        .map_err(|_| LayoutError::AllocationFailed)?;
    for index in 0..count {
        let child = tree
            .child_identity_at(identity, index)
            .map_err(|_| LayoutError::InvalidTree)?;
        children.push(capture_layout_tree(tree, child)?);
    }
    Ok(LayoutTreeInput { identity, children })
}

fn layout_node_from_tree(
    input: &LayoutTreeInput,
    controls: &UiObjectStore<Box<dyn Any + Send>>,
    align: Align,
    identities: &mut Vec<ObjectIdentity>,
) -> Result<LayoutNodeSnapshot, LayoutError> {
    identities
        .try_reserve(1)
        .map_err(|_| LayoutError::AllocationFailed)?;
    identities.push(input.identity);
    let control = controls
        .resolve(input.identity)
        .map_err(|_| LayoutError::InvalidTree)?
        .downcast_ref::<UiControlState>()
        .ok_or(LayoutError::InvalidTree)?;
    match control.kind {
        UiControlKind::StackPanel => {
            let orientation = match control.orientation {
                1 => Orientation::Vertical,
                2 => Orientation::Horizontal,
                _ => return Err(LayoutError::InvalidTree),
            };
            let mut children = Vec::new();
            children
                .try_reserve(input.children.len())
                .map_err(|_| LayoutError::AllocationFailed)?;
            for child in &input.children {
                let child_align = match control.child_align(child.identity) {
                    1 => Align::Start,
                    2 => Align::Center,
                    3 => Align::End,
                    4 => Align::Stretch,
                    _ => return Err(LayoutError::InvalidTree),
                };
                children.push(layout_node_from_tree(
                    child,
                    controls,
                    child_align,
                    identities,
                )?);
            }
            let mut node = LayoutNodeSnapshot::stack_panel(input.identity, orientation, children);
            node.align = align;
            Ok(node)
        }
        UiControlKind::Text | UiControlKind::TextBox | UiControlKind::Button
            if input.children.is_empty() =>
        {
            let mut node = LayoutNodeSnapshot::leaf(input.identity);
            node.align = align;
            Ok(node)
        }
        UiControlKind::Window
        | UiControlKind::Text
        | UiControlKind::TextBox
        | UiControlKind::Button => Err(LayoutError::InvalidTree),
    }
}

struct RuntimeFrameCompletionSink {
    runtime: Weak<RuntimeCore>,
    window: ObjectIdentity,
}

struct RuntimeWindowClientAreaSink {
    runtime: Weak<RuntimeCore>,
    window: ObjectIdentity,
}

impl crate::ui_window::PlatformWindowClientAreaSink for RuntimeWindowClientAreaSink {
    fn client_area_changed(
        &self,
        generation: u64,
        area: crate::ui_window::WindowClientArea,
    ) -> Result<(), PlatformOperationError> {
        if !area.is_valid() {
            return Err(PlatformOperationError);
        }
        let runtime = self.runtime.upgrade().ok_or(PlatformOperationError)?;
        let dispatcher = runtime
            .dispatcher()
            .map_err(|_| PlatformOperationError)?
            .ok_or(PlatformOperationError)?;
        let runtime = Arc::downgrade(&runtime);
        let window = self.window;
        dispatcher
            .post_completion(PostedWork::callback(Box::new(move || {
                if let Some(runtime) = runtime.upgrade() {
                    runtime.window_client_area_changed(window, generation, area);
                }
            })))
            .map_err(|_| PlatformOperationError)
    }
}

impl RuntimeFrameCompletionSink {
    fn post(&self, ticket: FrameTicket, success: bool) -> Result<(), PlatformOperationError> {
        let runtime = self.runtime.upgrade().ok_or(PlatformOperationError)?;
        let dispatcher = runtime
            .dispatcher()
            .map_err(|_| PlatformOperationError)?
            .ok_or(PlatformOperationError)?;
        let runtime = Arc::downgrade(&runtime);
        let window = self.window;
        dispatcher
            .post_completion(PostedWork::callback(Box::new(move || {
                if let Some(runtime) = runtime.upgrade() {
                    runtime.window_frame_completion(window, ticket, success);
                }
            })))
            .map_err(|_| PlatformOperationError)
    }
}

impl PlatformFrameCompletionSink for RuntimeFrameCompletionSink {
    fn complete_frame(&self, ticket: FrameTicket) -> Result<(), PlatformOperationError> {
        self.post(ticket, true)
    }

    fn fail_frame(&self, ticket: FrameTicket) -> Result<(), PlatformOperationError> {
        self.post(ticket, false)
    }
}

struct RuntimeBindingOwner {
    _source: ValidationPin<RuntimeIdentity>,
    _target: ValidationPin<RuntimeIdentity>,
    current_error: Option<BindingValidationSnapshot>,
    error_subscribers: Vec<Arc<Subscription>>,
}

pub(crate) struct BindingErrorDeliveryItem {
    pub(crate) connection: BindingConnectionId,
    pub(crate) subscribers: Vec<Arc<Subscription>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BindingValidationSnapshot {
    pub(crate) error: Arc<ValidationError>,
    pub(crate) rejected_property: ObjectIdentity,
    pub(crate) direction: u32,
}

/// One changed Property and its ordered, temporary subscriber pins.
#[allow(dead_code)] // Connected to the Runtime delivery loop in a later unit.
pub(crate) struct PropertyDeliveryItem {
    pub(crate) identity: ObjectIdentity,
    pub(crate) subscribers: Vec<Arc<Subscription>>,
}

/// An owned target value prepared by the Runtime's OneWay propagation pass.
#[derive(Debug)]
pub(crate) struct PreparedBindingValue {
    pub(crate) target: ObjectIdentity,
    pub(crate) value: TypedValue,
}

struct PreparedValidatorCheck {
    property: ObjectIdentity,
    validator: Arc<PropertyValidator>,
    candidate: TypedValue,
}

impl PropertyDeliveryItem {
    /// Invokes active subscribers in snapshot order, without holding Runtime locks.
    #[allow(dead_code)] // Connected to the Runtime delivery loop in a later unit.
    pub(crate) fn deliver(&self) -> usize {
        self.deliver_with(|| Ok(()))
            .expect("no-op safe point succeeds")
    }

    fn deliver_with(
        &self,
        mut after_each: impl FnMut() -> Result<(), RuntimePropertyError>,
    ) -> Result<usize, RuntimePropertyError> {
        let mut invoked = 0;
        for subscription in &self.subscribers {
            invoked += usize::from(subscription.invoke());
            after_each()?;
        }
        Ok(invoked)
    }
}

struct PropertyDeliveryGuard<'a>(&'a AtomicBool);

impl Drop for PropertyDeliveryGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[allow(dead_code)]
impl RuntimeCore {
    pub(crate) fn new() -> Result<Self, RuntimeCoreError> {
        Self::new_with_platform_factory(None)
    }

    pub(crate) fn new_with_platform_factory(
        platform_factory: Option<Arc<dyn PlatformRuntimeFactory>>,
    ) -> Result<Self, RuntimeCoreError> {
        let identity = RuntimeIdentity::issue().ok_or(RuntimeCoreError::IdentityUnavailable)?;
        let ui_thread = UiThreadContext::capture();
        let cleanup = RuntimeCleanupQueue::new_for_ui_thread(ui_thread);
        let ui_objects = Arc::new(Mutex::new(UiObjectStore::new(ui_thread)));
        let ui_tree = Arc::new(Mutex::new(RuntimeTreeStore::new(ui_thread)));
        let ui_state_cleared = Arc::new(AtomicBool::new(false));
        let property_changes = Arc::new(Mutex::new(PropertyChangeQueue::default()));
        let property_delivery = Arc::new(Mutex::new(None));
        let binding_graph = Arc::new(Mutex::new(BindingGraph::default()));
        let active_bindings: Arc<Mutex<HashMap<BindingConnectionId, RuntimeBindingOwner>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let binding_error_delivery = Arc::new(Mutex::new(VecDeque::new()));
        let subscription_owners: Arc<Mutex<HashMap<ObjectIdentity, Vec<Arc<Subscription>>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let shutdown_property_changes = Arc::clone(&property_changes);
        let shutdown_property_delivery = Arc::clone(&property_delivery);
        let shutdown_subscription_owners = Arc::clone(&subscription_owners);
        let shutdown_active_bindings = Arc::clone(&active_bindings);
        let shutdown_binding_error_delivery = Arc::clone(&binding_error_delivery);
        let shutdown_binding_graph = Arc::clone(&binding_graph);
        cleanup
            .track_ui_state(Box::new(move || {
                let delivery = shutdown_property_delivery
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take();
                *shutdown_property_changes
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) =
                    PropertyChangeQueue::default();
                let owners = std::mem::take(
                    &mut *shutdown_subscription_owners
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner),
                );
                for subscription in owners.into_values().flatten() {
                    subscription.cancel();
                }
                drop(delivery);
                shutdown_binding_error_delivery
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clear();
                let bindings = std::mem::take(
                    &mut *shutdown_active_bindings
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner),
                );
                shutdown_binding_graph
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clear();
                for owner in bindings.values() {
                    for subscription in &owner.error_subscribers {
                        subscription.cancel();
                    }
                }
                drop(bindings);
            }))
            .map_err(|()| RuntimeCoreError::CleanupRegistrationFailed)?;
        let object_cleanup = Arc::new(RuntimeObjectCleanupQueue {
            queue: Arc::clone(&cleanup),
            ui_objects: Arc::clone(&ui_objects),
            ui_tree: Arc::clone(&ui_tree),
            ui_state_cleared: Arc::clone(&ui_state_cleared),
            property_changes: Arc::clone(&property_changes),
            property_delivery: Arc::clone(&property_delivery),
            subscription_owners: Arc::clone(&subscription_owners),
        });
        let shutdown_tree = Arc::clone(&ui_tree);
        cleanup
            .track_ui_state(Box::new(move || {
                let nodes = {
                    let mut tree = shutdown_tree
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    tree.take_all()
                };
                match nodes {
                    Ok(nodes) => drop(nodes),
                    Err(error) => {
                        debug_assert!(
                            false,
                            "Logical Tree cleanup must run on its UI thread: {error:?}"
                        );
                    }
                }
            }))
            .map_err(|()| RuntimeCoreError::CleanupRegistrationFailed)?;
        let shutdown_objects = Arc::clone(&ui_objects);
        let shutdown_cleared = Arc::clone(&ui_state_cleared);
        cleanup
            .track_ui_state(Box::new(move || {
                let objects = shutdown_objects
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .take_all();
                match objects {
                    Ok(objects) => {
                        // Cleanup tickets racing with reentrant peer teardown now observe the
                        // store as cleared even though external close callbacks are still running.
                        shutdown_cleared.store(true, Ordering::Release);
                        for (identity, mut object) in objects {
                            if let Some(state) = object.downcast_mut::<UiControlState>() {
                                if let Some(window) = &mut state.window {
                                    window.shutdown(identity);
                                }
                            }
                        }
                    }
                    Err(error) => {
                        debug_assert!(
                            false,
                            "Runtime store cleanup must run on its UI thread: {error:?}"
                        );
                    }
                }
            }))
            .map_err(|()| RuntimeCoreError::CleanupRegistrationFailed)?;

        Ok(Self {
            identity,
            platform_factory,
            command_operation_ids: OperationIdIssuer::new(),
            ui_thread,
            cleanup,
            ui_objects,
            ui_tree,
            focus: Mutex::new(FocusManager::default()),
            binding_graph,
            active_bindings,
            binding_error_delivery,
            property_changes,
            property_delivery,
            property_delivery_active: AtomicBool::new(false),
            validator_active: AtomicBool::new(false),
            subscription_owners,
            object_cleanup,
            host: HostRegistration::new_for_ui_thread(ui_thread),
            dispatcher: Mutex::new(None),
            shutdown_completion: Mutex::new(None),
        })
    }

    pub(crate) fn identity(&self) -> RuntimeIdentity {
        self.identity
    }

    pub(crate) fn issue_command_operation_id(&self) -> Result<u64, CommandError> {
        self.command_operation_ids.issue()
    }

    /// Creates thread-safe ownership metadata bound to this Runtime and its cleanup queue.
    pub(crate) fn new_object_control(
        &self,
        identity: ObjectIdentity,
    ) -> Arc<CoreObjectControl<RuntimeIdentity>> {
        let cleanup_queue: Arc<dyn CleanupQueue> = self.object_cleanup.clone();
        CoreObjectControl::new(self.identity, identity, cleanup_queue)
    }

    /// Closes one Command and requests cancellation at Runtime shutdown without joining workers.
    pub(crate) fn track_command_shutdown(
        &self,
        on_shutdown: impl Fn() + Send + Sync + 'static,
    ) -> Result<(), ()> {
        self.cleanup.track_ui_state(Box::new(on_shutdown))
    }

    /// Creates UI state, binds its identity to this Runtime, and registers its owner token.
    pub(crate) fn register_ui_object<K, T, F>(
        &self,
        registry: &HandleRegistry<K, RuntimeIdentity>,
        kind: K,
        make_state: F,
    ) -> Result<HandleToken, RuntimeObjectRegistrationError>
    where
        K: Clone + Eq + Send + Sync,
        T: Any + Send + 'static,
        F: FnOnce() -> T + Send,
    {
        if !self.is_ui_thread() {
            return Err(RuntimeObjectRegistrationError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeObjectRegistrationError::Closed);
        }

        let state = Box::new(make_state()) as Box<dyn Any + Send>;
        let identity = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimeObjectRegistrationError::SynchronizationPoisoned)?
            .insert(state)
            .map_err(RuntimeObjectRegistrationError::Store)?;
        let control = self.new_object_control(identity);

        match registry.register(kind, self.identity, Arc::clone(&control)) {
            Ok(token) => Ok(token),
            Err(error) => {
                let rollback = self
                    .ui_objects
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .destroy(identity);
                debug_assert!(rollback.is_ok(), "failed to roll back UI object state");
                drop(rollback);
                Err(RuntimeObjectRegistrationError::Registry(error))
            }
        }
    }

    /// Registers a UI object and its Logical Tree node as one internal Runtime operation.
    pub(crate) fn register_tree_ui_object<K, T, F>(
        &self,
        registry: &HandleRegistry<K, RuntimeIdentity>,
        kind: K,
        is_container: bool,
        make_state: F,
    ) -> Result<HandleToken, RuntimeTreeError>
    where
        K: Clone + Eq + Send + Sync,
        T: Any + Send + 'static,
        F: FnOnce() -> T + Send,
    {
        if !self.is_ui_thread() {
            return Err(RuntimeTreeError::WrongThread);
        }
        let token = self
            .register_ui_object(registry, kind.clone(), make_state)
            .map_err(RuntimeTreeError::Registration)?;
        let pin = match registry.validate(token, &kind, &self.identity) {
            Ok(pin) => pin,
            Err(error) => {
                let rollback = registry.release(token);
                debug_assert!(rollback.is_ok(), "failed to roll back tree object token");
                return Err(RuntimeTreeError::Registry(error));
            }
        };
        let node = LogicalNode::new(&pin, is_container, self.ui_thread);
        let result = self
            .ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)
            .and_then(|mut tree| tree.insert(node).map_err(RuntimeTreeError::Tree));
        drop(pin);
        if let Err(error) = result {
            let rollback = registry.release(token);
            debug_assert!(
                rollback.is_ok(),
                "failed to roll back tree object registration"
            );
            return Err(error);
        }
        Ok(token)
    }

    pub(crate) fn attach_tree_child<K, F>(
        &self,
        registry: &HandleRegistry<K, RuntimeIdentity>,
        parent_token: HandleToken,
        child_token: HandleToken,
        kind: K,
        prepare_peer: F,
    ) -> Result<(), RuntimeTreeError>
    where
        K: Clone + Eq + Send + Sync,
        F: FnOnce() -> Result<(), ()>,
    {
        if !self.is_ui_thread() {
            return Err(RuntimeTreeError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeTreeError::Closed);
        }
        let parent_pin = registry
            .validate(parent_token, &kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        let child_pin = registry
            .validate(child_token, &kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        let parent_identity = parent_pin.identity();
        self.ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)?
            .attach(&parent_pin, &child_pin, prepare_peer)
            .map_err(RuntimeTreeError::Tree)?;
        self.mark_layout_dirty(parent_identity);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn attach_tree_child_at<K, F>(
        &self,
        registry: &HandleRegistry<K, RuntimeIdentity>,
        parent_token: HandleToken,
        parent_kind: K,
        child_token: HandleToken,
        child_kind: K,
        index: usize,
        prepare_peer: F,
    ) -> Result<(), RuntimeTreeError>
    where
        K: Clone + Eq + Send + Sync,
        F: FnOnce() -> Result<(), ()>,
    {
        if !self.is_ui_thread() {
            return Err(RuntimeTreeError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeTreeError::Closed);
        }
        let parent_pin = registry
            .validate(parent_token, &parent_kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        let child_pin = registry
            .validate(child_token, &child_kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        let parent_identity = parent_pin.identity();
        self.ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)?
            .attach_at(&parent_pin, &child_pin, index, prepare_peer)
            .map_err(RuntimeTreeError::Tree)?;
        self.mark_layout_dirty(parent_identity);
        Ok(())
    }

    pub(crate) fn replace_tree_single_child<K, F>(
        &self,
        registry: &HandleRegistry<K, RuntimeIdentity>,
        parent_token: HandleToken,
        parent_kind: K,
        child_token: HandleToken,
        child_kind: K,
        prepare_peer: F,
    ) -> Result<(), RuntimeTreeError>
    where
        K: Clone + Eq + Send + Sync,
        F: FnOnce() -> Result<(), ()>,
    {
        if !self.is_ui_thread() {
            return Err(RuntimeTreeError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeTreeError::Closed);
        }
        let parent_pin = registry
            .validate(parent_token, &parent_kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        let child_pin = registry
            .validate(child_token, &child_kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        let parent_identity = parent_pin.identity();
        let previous_child = self
            .ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)?
            .child_identity_at(parent_identity, 0)
            .ok();
        let detached_owner = self
            .ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)?
            .replace_single_child(&parent_pin, &child_pin, prepare_peer)
            .map_err(RuntimeTreeError::Tree)?;
        drop(detached_owner);
        self.mark_layout_dirty(parent_identity);
        if let Some(previous) = previous_child.filter(|previous| *previous != child_pin.identity())
        {
            self.stack_panel_clear_child_align(parent_identity, previous);
            self.clear_focus_for_detached_subtree(previous);
        }
        Ok(())
    }

    pub(crate) fn detach_tree_child<K>(
        &self,
        registry: &HandleRegistry<K, RuntimeIdentity>,
        parent_token: HandleToken,
        child_token: HandleToken,
        kind: K,
    ) -> Result<(), RuntimeTreeError>
    where
        K: Clone + Eq + Send + Sync,
    {
        if !self.is_ui_thread() {
            return Err(RuntimeTreeError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeTreeError::Closed);
        }
        let parent_pin = registry
            .validate(parent_token, &kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        let child_pin = registry
            .validate(child_token, &kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        let parent_identity = parent_pin.identity();
        let child_identity = child_pin.identity();
        let detached = self
            .ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)?
            .detach(parent_pin.identity(), child_pin.identity())
            .map_err(RuntimeTreeError::Tree)?;
        drop(detached);
        self.mark_layout_dirty(parent_identity);
        self.stack_panel_clear_child_align(parent_identity, child_identity);
        self.clear_focus_for_detached_subtree(child_identity);
        Ok(())
    }

    pub(crate) fn detach_tree_child_of_kinds<K>(
        &self,
        registry: &HandleRegistry<K, RuntimeIdentity>,
        parent_token: HandleToken,
        parent_kind: K,
        child_token: HandleToken,
        child_kind: K,
    ) -> Result<(), RuntimeTreeError>
    where
        K: Clone + Eq + Send + Sync,
    {
        if !self.is_ui_thread() {
            return Err(RuntimeTreeError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeTreeError::Closed);
        }
        let parent_pin = registry
            .validate(parent_token, &parent_kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        let child_pin = registry
            .validate(child_token, &child_kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        let parent_identity = parent_pin.identity();
        let child_identity = child_pin.identity();
        let detached = self
            .ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)?
            .detach(parent_pin.identity(), child_pin.identity())
            .map_err(RuntimeTreeError::Tree)?;
        drop(detached);
        self.mark_layout_dirty(parent_identity);
        self.stack_panel_clear_child_align(parent_identity, child_identity);
        self.clear_focus_for_detached_subtree(child_identity);
        Ok(())
    }

    pub(crate) fn detach_tree_child_identity<K>(
        &self,
        registry: &HandleRegistry<K, RuntimeIdentity>,
        parent_token: HandleToken,
        parent_kind: K,
        child: ObjectIdentity,
    ) -> Result<(), RuntimeTreeError>
    where
        K: Clone + Eq + Send + Sync,
    {
        if !self.is_ui_thread() {
            return Err(RuntimeTreeError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeTreeError::Closed);
        }
        let parent_pin = registry
            .validate(parent_token, &parent_kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        let parent_identity = parent_pin.identity();
        let detached = self
            .ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)?
            .detach(parent_pin.identity(), child)
            .map_err(RuntimeTreeError::Tree)?;
        drop(detached);
        self.mark_layout_dirty(parent_identity);
        self.stack_panel_clear_child_align(parent_identity, child);
        self.clear_focus_for_detached_subtree(child);
        Ok(())
    }

    pub(crate) fn tree_child_handle<K>(
        &self,
        registry: &HandleRegistry<K, RuntimeIdentity>,
        parent_token: HandleToken,
        child: ObjectIdentity,
        kind: K,
    ) -> Result<HandleToken, RuntimeTreeError>
    where
        K: Clone + Eq + Send + Sync,
    {
        if !self.is_ui_thread() {
            return Err(RuntimeTreeError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeTreeError::Closed);
        }
        let parent_pin = registry
            .validate(parent_token, &kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        self.ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)?
            .child_handle(registry, parent_pin.identity(), child, kind)
            .map_err(RuntimeTreeError::Tree)
    }

    pub(crate) fn tree_child_handle_of_kinds<K>(
        &self,
        registry: &HandleRegistry<K, RuntimeIdentity>,
        parent_token: HandleToken,
        parent_kind: K,
        child: ObjectIdentity,
        child_kind: K,
    ) -> Result<HandleToken, RuntimeTreeError>
    where
        K: Clone + Eq + Send + Sync,
    {
        if !self.is_ui_thread() {
            return Err(RuntimeTreeError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeTreeError::Closed);
        }
        let parent_pin = registry
            .validate(parent_token, &parent_kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        self.ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)?
            .child_handle(registry, parent_pin.identity(), child, child_kind)
            .map_err(RuntimeTreeError::Tree)
    }

    pub(crate) fn tree_child_count<K>(
        &self,
        registry: &HandleRegistry<K, RuntimeIdentity>,
        parent_token: HandleToken,
        parent_kind: K,
    ) -> Result<usize, RuntimeTreeError>
    where
        K: Clone + Eq + Send + Sync,
    {
        if !self.is_ui_thread() {
            return Err(RuntimeTreeError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeTreeError::Closed);
        }
        let parent_pin = registry
            .validate(parent_token, &parent_kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        self.ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)?
            .child_count(parent_pin.identity())
            .map_err(RuntimeTreeError::Tree)
    }

    pub(crate) fn tree_child_identity_at<K>(
        &self,
        registry: &HandleRegistry<K, RuntimeIdentity>,
        parent_token: HandleToken,
        parent_kind: K,
        index: usize,
    ) -> Result<ObjectIdentity, RuntimeTreeError>
    where
        K: Clone + Eq + Send + Sync,
    {
        if !self.is_ui_thread() {
            return Err(RuntimeTreeError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeTreeError::Closed);
        }
        let parent_pin = registry
            .validate(parent_token, &parent_kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        self.ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)?
            .child_identity_at(parent_pin.identity(), index)
            .map_err(RuntimeTreeError::Tree)
    }

    pub(crate) fn tree_move_child<K>(
        &self,
        registry: &HandleRegistry<K, RuntimeIdentity>,
        parent_token: HandleToken,
        parent_kind: K,
        from_index: usize,
        to_index: usize,
    ) -> Result<(), RuntimeTreeError>
    where
        K: Clone + Eq + Send + Sync,
    {
        if !self.is_ui_thread() {
            return Err(RuntimeTreeError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeTreeError::Closed);
        }
        let parent_pin = registry
            .validate(parent_token, &parent_kind, &self.identity)
            .map_err(RuntimeTreeError::Registry)?;
        let parent_identity = parent_pin.identity();
        self.ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)?
            .move_child(parent_pin.identity(), from_index, to_index)
            .map_err(RuntimeTreeError::Tree)?;
        self.mark_layout_dirty(parent_identity);
        Ok(())
    }

    /// Registers a Property with its initial typed value in this Runtime.
    pub(crate) fn register_property(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        initial: TypedValue,
    ) -> Result<HandleToken, RuntimeObjectRegistrationError> {
        self.register_ui_object(registry, PropertyHandleKind::Property, move || {
            PropertyState::new(initial)
        })
    }

    /// Registers a Core-owned Property that can be observed but only changed by its owner.
    pub(crate) fn register_read_only_property(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        initial: TypedValue,
    ) -> Result<HandleToken, RuntimeObjectRegistrationError> {
        self.register_ui_object(registry, PropertyHandleKind::Property, move || {
            PropertyState::new_read_only(initial)
        })
    }

    /// Returns an independent internal owner for a Property attached to one Core Control.
    pub(crate) fn ui_control_property_owner(
        &self,
        identity: ObjectIdentity,
        property: UiControlProperty,
    ) -> Result<InternalOwner<RuntimeIdentity>, RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        let store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let state = store
            .resolve(identity)
            .map_err(RuntimePropertyError::Store)?
            .downcast_ref::<UiControlState>()
            .ok_or(RuntimePropertyError::WrongObjectState)?;
        if !state.kind.supports_property(property) {
            return Err(RuntimePropertyError::WrongObjectState);
        }
        state
            .property_owner(property)
            .ok_or(RuntimePropertyError::WrongObjectState)?
            .try_clone()
            .map_err(RuntimePropertyError::Registry)
    }

    pub(crate) fn ui_control_kind(
        &self,
        identity: ObjectIdentity,
    ) -> Result<UiControlKind, RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        let store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        store
            .resolve(identity)
            .map_err(RuntimePropertyError::Store)?
            .downcast_ref::<UiControlState>()
            .map(|state| state.kind)
            .ok_or(RuntimePropertyError::WrongObjectState)
    }

    pub(crate) fn button_set_command_entry(
        &self,
        identity: ObjectIdentity,
        command: Option<Arc<crate::abi_support::CommandHandleEntry>>,
    ) -> Result<(), RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        let previous = {
            let mut store = self
                .ui_objects
                .lock()
                .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
            let state = store
                .resolve_mut(identity)
                .map_err(RuntimePropertyError::Store)?
                .downcast_mut::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::Button)
                .ok_or(RuntimePropertyError::WrongObjectState)?;
            std::mem::replace(&mut state.button_command, command)
        };
        drop(previous);
        Ok(())
    }

    pub(crate) fn button_command_entry(
        &self,
        identity: ObjectIdentity,
    ) -> Result<Option<Arc<crate::abi_support::CommandHandleEntry>>, RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        let store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let state = store
            .resolve(identity)
            .map_err(RuntimePropertyError::Store)?
            .downcast_ref::<UiControlState>()
            .filter(|state| state.kind == UiControlKind::Button)
            .ok_or(RuntimePropertyError::WrongObjectState)?;
        Ok(state.button_command.clone())
    }

    pub(crate) fn stack_panel_config(
        &self,
        identity: ObjectIdentity,
    ) -> Result<(u32, bool), RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        let store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let state = store
            .resolve(identity)
            .map_err(RuntimePropertyError::Store)?
            .downcast_ref::<UiControlState>()
            .filter(|state| state.kind == UiControlKind::StackPanel)
            .ok_or(RuntimePropertyError::WrongObjectState)?;
        Ok((state.orientation, state.layout_dirty))
    }

    pub(crate) fn stack_panel_set_orientation(
        &self,
        identity: ObjectIdentity,
        orientation: u32,
    ) -> Result<bool, RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        if !matches!(orientation, 1 | 2) {
            return Err(RuntimePropertyError::WrongObjectState);
        }
        let changed = {
            let mut store = self
                .ui_objects
                .lock()
                .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
            let state = store
                .resolve_mut(identity)
                .map_err(RuntimePropertyError::Store)?
                .downcast_mut::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::StackPanel)
                .ok_or(RuntimePropertyError::WrongObjectState)?;
            if state.orientation == orientation {
                false
            } else {
                state.orientation = orientation;
                state.layout_dirty = true;
                true
            }
        };
        if changed {
            self.mark_layout_dirty(identity);
        }
        Ok(changed)
    }

    pub(crate) fn tree_parent_identity(
        &self,
        identity: ObjectIdentity,
    ) -> Result<Option<ObjectIdentity>, RuntimeTreeError> {
        self.ui_tree
            .lock()
            .map_err(|_| RuntimeTreeError::SynchronizationPoisoned)?
            .parent_identity(identity)
            .map_err(RuntimeTreeError::Tree)
    }

    pub(crate) fn stack_panel_child_align(
        &self,
        panel: ObjectIdentity,
        child: ObjectIdentity,
    ) -> Result<u32, RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        let parent = self
            .tree_parent_identity(child)
            .map_err(|error| match error {
                RuntimeTreeError::WrongThread => RuntimePropertyError::WrongThread,
                RuntimeTreeError::Closed => RuntimePropertyError::Closed,
                _ => RuntimePropertyError::WrongObjectState,
            })?;
        if parent != Some(panel) {
            return Err(RuntimePropertyError::WrongObjectState);
        }
        let store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let state = store
            .resolve(panel)
            .map_err(RuntimePropertyError::Store)?
            .downcast_ref::<UiControlState>()
            .filter(|state| state.kind == UiControlKind::StackPanel)
            .ok_or(RuntimePropertyError::WrongObjectState)?;
        Ok(state.child_align(child))
    }

    pub(crate) fn stack_panel_set_child_align(
        &self,
        panel: ObjectIdentity,
        child: ObjectIdentity,
        align: u32,
    ) -> Result<bool, RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        if !matches!(align, 1..=4) {
            return Err(RuntimePropertyError::WrongObjectState);
        }
        let parent = self
            .tree_parent_identity(child)
            .map_err(|error| match error {
                RuntimeTreeError::WrongThread => RuntimePropertyError::WrongThread,
                RuntimeTreeError::Closed => RuntimePropertyError::Closed,
                _ => RuntimePropertyError::WrongObjectState,
            })?;
        if parent != Some(panel) {
            return Err(RuntimePropertyError::WrongObjectState);
        }
        let changed = {
            let mut store = self
                .ui_objects
                .lock()
                .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
            let state = store
                .resolve_mut(panel)
                .map_err(RuntimePropertyError::Store)?
                .downcast_mut::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::StackPanel)
                .ok_or(RuntimePropertyError::WrongObjectState)?;
            if state.child_align(child) == align {
                false
            } else {
                state
                    .child_align
                    .try_reserve(1)
                    .map_err(|_| RuntimePropertyError::NotificationQueueOutOfMemory)?;
                state.child_align.insert(child, align);
                state.layout_dirty = true;
                true
            }
        };
        if changed {
            self.mark_layout_dirty(panel);
        }
        Ok(changed)
    }

    fn stack_panel_clear_child_align(&self, panel: ObjectIdentity, child: ObjectIdentity) {
        let changed = {
            let mut store = self
                .ui_objects
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Ok(object) = store.resolve_mut(panel) {
                if let Some(state) = object
                    .downcast_mut::<UiControlState>()
                    .filter(|state| state.kind == UiControlKind::StackPanel)
                {
                    state.child_align.remove(&child).is_some()
                } else {
                    false
                }
            } else {
                false
            }
        };
        if changed {
            self.mark_layout_dirty(panel);
        }
    }

    pub(crate) fn ui_control_layout_dirty(
        &self,
        identity: ObjectIdentity,
    ) -> Result<bool, RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        let store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        store
            .resolve(identity)
            .map_err(RuntimePropertyError::Store)?
            .downcast_ref::<UiControlState>()
            .map(|state| state.layout_dirty)
            .ok_or(RuntimePropertyError::WrongObjectState)
    }

    pub(crate) fn ui_control_layout_clean(
        &self,
        identity: ObjectIdentity,
    ) -> Result<(), RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let state = store
            .resolve_mut(identity)
            .map_err(RuntimePropertyError::Store)?
            .downcast_mut::<UiControlState>()
            .ok_or(RuntimePropertyError::WrongObjectState)?;
        state.layout_dirty = false;
        Ok(())
    }

    pub(crate) fn window_set_peer(
        &self,
        identity: ObjectIdentity,
        peer: Arc<dyn WindowPeerAdapter>,
    ) -> Result<(), RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let state = store
            .resolve_mut(identity)
            .map_err(RuntimePropertyError::Store)?
            .downcast_mut::<UiControlState>()
            .filter(|state| state.kind == UiControlKind::Window)
            .ok_or(RuntimePropertyError::WrongObjectState)?;
        state
            .window
            .as_mut()
            .ok_or(RuntimePropertyError::WrongObjectState)?
            .set_peer(peer);
        Ok(())
    }

    pub(crate) fn window_open(
        self: &Arc<Self>,
        identity: ObjectIdentity,
    ) -> Result<u64, WindowLifecycleError> {
        self.ensure_property_access_thread()
            .map_err(|error| match error {
                RuntimePropertyError::WrongThread => WindowLifecycleError::InvalidState,
                RuntimePropertyError::Closed => WindowLifecycleError::InvalidState,
                _ => WindowLifecycleError::InvalidState,
            })?;
        let has_peer = {
            let store = self
                .ui_objects
                .lock()
                .map_err(|_| WindowLifecycleError::InvalidState)?;
            store
                .resolve(identity)
                .map_err(|_| WindowLifecycleError::InvalidState)?
                .downcast_ref::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::Window)
                .and_then(|state| state.window.as_ref())
                .ok_or(WindowLifecycleError::InvalidState)?
                .has_peer()
        };
        if !has_peer {
            let factory = self
                .platform_factory
                .as_ref()
                .ok_or(WindowLifecycleError::Unsupported)?;
            let peer = factory
                .create_window_peer(Arc::new(RuntimeFrameCompletionSink {
                    runtime: Arc::downgrade(self),
                    window: identity,
                }))
                .map_err(|_| WindowLifecycleError::PeerFailure)?;
            peer.set_client_area_sink(Arc::new(RuntimeWindowClientAreaSink {
                runtime: Arc::downgrade(self),
                window: identity,
            }));
            self.window_set_peer(identity, Arc::new(FactoryWindowPeer(peer)))
                .map_err(|_| WindowLifecycleError::InvalidState)?;
        }
        let (generation, peer) = {
            let mut store = self
                .ui_objects
                .lock()
                .map_err(|_| WindowLifecycleError::InvalidState)?;
            let state = store
                .resolve_mut(identity)
                .map_err(|_| WindowLifecycleError::InvalidState)?
                .downcast_mut::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::Window)
                .ok_or(WindowLifecycleError::InvalidState)?;
            state
                .window
                .as_mut()
                .ok_or(WindowLifecycleError::InvalidState)?
                .prepare_open()?
        };
        if let Some(peer) = peer {
            let mut result = peer.open(identity, generation);
            if result.as_ref().is_ok_and(|area| !area.is_valid()) {
                let _ = peer.close(identity);
                result = Err(());
            }
            let mut store = self
                .ui_objects
                .lock()
                .map_err(|_| WindowLifecycleError::InvalidState)?;
            let state = store
                .resolve_mut(identity)
                .map_err(|_| WindowLifecycleError::StaleRequest)?
                .downcast_mut::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::Window)
                .ok_or(WindowLifecycleError::StaleRequest)?;
            state
                .window
                .as_mut()
                .ok_or(WindowLifecycleError::StaleRequest)?
                .finish_open(generation, result)?;
            drop(store);
            self.mark_layout_subtree_dirty(identity);
        }
        Ok(generation)
    }

    fn window_client_area_changed(
        &self,
        identity: ObjectIdentity,
        generation: u64,
        area: crate::ui_window::WindowClientArea,
    ) {
        if !self.is_ui_thread() || !area.is_valid() {
            return;
        }
        let changed = {
            let mut store = self
                .ui_objects
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            store
                .resolve_mut(identity)
                .ok()
                .and_then(|object| object.downcast_mut::<UiControlState>())
                .filter(|state| state.kind == UiControlKind::Window)
                .and_then(|state| state.window.as_mut())
                .is_some_and(|window| window.update_client_area(generation, area))
        };
        if changed {
            self.mark_layout_subtree_dirty(identity);
        }
    }

    pub(crate) fn window_submit_frame(
        &self,
        identity: ObjectIdentity,
        scene: Arc<SceneSnapshot>,
    ) -> Result<FrameTicket, WindowLifecycleError> {
        self.ensure_property_access_thread()
            .map_err(|_| WindowLifecycleError::InvalidState)?;
        let prepared = {
            let mut store = self
                .ui_objects
                .lock()
                .map_err(|_| WindowLifecycleError::InvalidState)?;
            store
                .resolve_mut(identity)
                .map_err(|_| WindowLifecycleError::InvalidState)?
                .downcast_mut::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::Window)
                .and_then(|state| state.window.as_mut())
                .ok_or(WindowLifecycleError::InvalidState)?
                .begin_frame(scene)
                .map_err(WindowLifecycleError::Frame)?
        };
        if prepared
            .peer
            .submit_frame(prepared.ticket, prepared.scene)
            .is_err()
        {
            self.window_frame_completion(identity, prepared.ticket, false);
            return Err(WindowLifecycleError::PeerFailure);
        }
        Ok(prepared.ticket)
    }

    pub(crate) fn window_frame_phase(
        &self,
        identity: ObjectIdentity,
    ) -> Result<FramePhase, WindowLifecycleError> {
        let store = self
            .ui_objects
            .lock()
            .map_err(|_| WindowLifecycleError::InvalidState)?;
        store
            .resolve(identity)
            .map_err(|_| WindowLifecycleError::InvalidState)?
            .downcast_ref::<UiControlState>()
            .filter(|state| state.kind == UiControlKind::Window)
            .and_then(|state| state.window.as_ref())
            .map(|window| window.frame_phase())
            .ok_or(WindowLifecycleError::InvalidState)
    }

    fn window_frame_completion(
        &self,
        identity: ObjectIdentity,
        ticket: FrameTicket,
        success: bool,
    ) {
        let Ok(mut store) = self.ui_objects.lock() else {
            return;
        };
        let Ok(state) = store.resolve_mut(identity) else {
            return;
        };
        let Some(window) = state
            .downcast_mut::<UiControlState>()
            .filter(|state| state.kind == UiControlKind::Window)
            .and_then(|state| state.window.as_mut())
        else {
            return;
        };
        if success {
            window.complete_frame(ticket);
        } else {
            window.fail_frame(ticket);
        }
    }

    pub(crate) fn window_close(
        &self,
        identity: ObjectIdentity,
    ) -> Result<(), WindowLifecycleError> {
        let (generation, peer) = {
            let mut store = self
                .ui_objects
                .lock()
                .map_err(|_| WindowLifecycleError::InvalidState)?;
            let state = store
                .resolve_mut(identity)
                .map_err(|_| WindowLifecycleError::InvalidState)?
                .downcast_mut::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::Window)
                .ok_or(WindowLifecycleError::InvalidState)?;
            let window = state
                .window
                .as_mut()
                .ok_or(WindowLifecycleError::InvalidState)?;
            if !window.is_open() {
                return Ok(());
            }
            let generation = window.generation();
            let peer = window.prepare_close()?;
            (generation, peer)
        };
        let result = peer.close(identity);
        {
            let mut store = self
                .ui_objects
                .lock()
                .map_err(|_| WindowLifecycleError::InvalidState)?;
            let state = store
                .resolve_mut(identity)
                .map_err(|_| WindowLifecycleError::StaleRequest)?
                .downcast_mut::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::Window)
                .ok_or(WindowLifecycleError::StaleRequest)?;
            state
                .window
                .as_mut()
                .ok_or(WindowLifecycleError::StaleRequest)?
                .finish_close(generation, result)?;
        }
        self.focus
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear_for_window(identity);
        Ok(())
    }

    pub(crate) fn window_close_intent(
        &self,
        identity: ObjectIdentity,
        callback: Option<Box<dyn FnOnce(u64, u64) -> CloseDecision>>,
    ) -> Result<Option<(u64, u64)>, WindowLifecycleError> {
        let request = {
            let mut store = self
                .ui_objects
                .lock()
                .map_err(|_| WindowLifecycleError::InvalidState)?;
            let state = store
                .resolve_mut(identity)
                .map_err(|_| WindowLifecycleError::InvalidState)?
                .downcast_mut::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::Window)
                .ok_or(WindowLifecycleError::InvalidState)?;
            state
                .window
                .as_mut()
                .ok_or(WindowLifecycleError::InvalidState)?
                .begin_close_intent()?
        };
        let decision = callback.map_or(CloseDecision::Accept, |callback| {
            callback(request.0, request.1)
        });
        let peer = {
            let mut store = self
                .ui_objects
                .lock()
                .map_err(|_| WindowLifecycleError::InvalidState)?;
            let state = store
                .resolve_mut(identity)
                .map_err(|_| WindowLifecycleError::StaleRequest)?
                .downcast_mut::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::Window)
                .ok_or(WindowLifecycleError::StaleRequest)?;
            state
                .window
                .as_mut()
                .ok_or(WindowLifecycleError::StaleRequest)?
                .complete_close_intent(request.0, request.1, decision)?
        };
        if let Some(peer) = peer {
            let result = peer.close(identity);
            let finish = {
                let mut store = self
                    .ui_objects
                    .lock()
                    .map_err(|_| WindowLifecycleError::InvalidState)?;
                let state = store
                    .resolve_mut(identity)
                    .map_err(|_| WindowLifecycleError::StaleRequest)?
                    .downcast_mut::<UiControlState>()
                    .filter(|state| state.kind == UiControlKind::Window)
                    .ok_or(WindowLifecycleError::StaleRequest)?;
                state
                    .window
                    .as_mut()
                    .ok_or(WindowLifecycleError::StaleRequest)?
                    .finish_close_intent(request.1, result)
            };
            finish?;
            self.focus
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clear_for_window(identity);
            Ok(None)
        } else if decision == CloseDecision::Defer {
            Ok(Some(request))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn window_close_resolve(
        &self,
        identity: ObjectIdentity,
        request: u64,
        generation: u64,
        decision: CloseDecision,
    ) -> Result<(), WindowLifecycleError> {
        let peer = {
            let mut store = self
                .ui_objects
                .lock()
                .map_err(|_| WindowLifecycleError::InvalidState)?;
            let state = store
                .resolve_mut(identity)
                .map_err(|_| WindowLifecycleError::InvalidState)?
                .downcast_mut::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::Window)
                .ok_or(WindowLifecycleError::InvalidState)?;
            state
                .window
                .as_mut()
                .ok_or(WindowLifecycleError::InvalidState)?
                .prepare_resolve(request, generation, decision)?
        };
        if let Some(peer) = peer {
            let result = peer.close(identity);
            let mut store = self
                .ui_objects
                .lock()
                .map_err(|_| WindowLifecycleError::InvalidState)?;
            let state = store
                .resolve_mut(identity)
                .map_err(|_| WindowLifecycleError::StaleRequest)?
                .downcast_mut::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::Window)
                .ok_or(WindowLifecycleError::StaleRequest)?;
            state
                .window
                .as_mut()
                .ok_or(WindowLifecycleError::StaleRequest)?
                .finish_close_intent(generation, result)?;
            self.focus
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clear_for_window(identity);
        }
        Ok(())
    }

    pub(crate) fn window_close_request_release(
        &self,
        identity: ObjectIdentity,
        request: u64,
        generation: u64,
    ) -> Result<(), WindowLifecycleError> {
        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| WindowLifecycleError::InvalidState)?;
        let state = store
            .resolve_mut(identity)
            .map_err(|_| WindowLifecycleError::InvalidState)?
            .downcast_mut::<UiControlState>()
            .filter(|state| state.kind == UiControlKind::Window)
            .ok_or(WindowLifecycleError::InvalidState)?;
        state
            .window
            .as_mut()
            .ok_or(WindowLifecycleError::InvalidState)?
            .release_unresolved(request, generation)
    }

    pub(crate) fn window_is_open(&self, identity: ObjectIdentity) -> bool {
        let Ok(store) = self.ui_objects.lock() else {
            return false;
        };
        store
            .resolve(identity)
            .ok()
            .and_then(|object| object.downcast_ref::<UiControlState>())
            .and_then(|state| state.window.as_ref())
            .is_some_and(|window| window.is_open())
    }

    #[cfg(test)]
    pub(crate) fn window_client_area(
        &self,
        identity: ObjectIdentity,
    ) -> Result<Option<(u64, crate::ui_window::WindowClientArea)>, RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        self.ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?
            .resolve(identity)
            .map_err(RuntimePropertyError::Store)?
            .downcast_ref::<UiControlState>()
            .filter(|state| state.kind == UiControlKind::Window)
            .and_then(|state| state.window.as_ref())
            .map(|window| window.client_area())
            .ok_or(RuntimePropertyError::WrongObjectState)
    }

    pub(crate) fn window_set_active(
        &self,
        identity: ObjectIdentity,
        active: bool,
    ) -> Result<(), RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        if self.ui_control_kind(identity)? != UiControlKind::Window {
            return Err(RuntimePropertyError::WrongObjectState);
        }
        if active && !self.window_is_open(identity) {
            return Err(RuntimePropertyError::WrongObjectState);
        }
        let mut focus = self
            .focus
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        if active {
            let candidate = focus
                .candidate(identity)
                .filter(|candidate| self.focus_target_is_eligible(identity, *candidate));
            focus.activate(identity, candidate);
        } else {
            focus.deactivate(identity);
        }
        Ok(())
    }

    fn focus_target_is_eligible(&self, window: ObjectIdentity, target: ObjectIdentity) -> bool {
        let Ok(root) = self
            .ui_tree
            .lock()
            .map_err(|_| ())
            .and_then(|tree| tree.root_identity(target).map_err(|_| ()))
        else {
            return false;
        };
        if root != window {
            return false;
        }
        let Ok(store) = self.ui_objects.lock() else {
            return false;
        };
        let Ok(object) = store.resolve(target) else {
            return false;
        };
        let Some(state) = object.downcast_ref::<UiControlState>() else {
            return false;
        };
        if !state.is_focusable_by_default() {
            return false;
        }
        let Some(owner) = state.property_owner(UiControlProperty::IsEnabled) else {
            return false;
        };
        store
            .resolve(owner.identity())
            .ok()
            .and_then(|value| value.downcast_ref::<PropertyState>())
            .is_some_and(|property| matches!(property.read(), TypedValue::Bool(true)))
    }

    pub(crate) fn widget_request_focus(
        &self,
        identity: ObjectIdentity,
    ) -> Result<(), RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        let root = self
            .ui_tree
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?
            .root_identity(identity)
            .map_err(|_| RuntimePropertyError::WrongObjectState)?;
        if !self.focus_target_is_eligible(root, identity) {
            return Err(RuntimePropertyError::WrongObjectState);
        }
        let mut focus = self
            .focus
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        if focus.active_window() != Some(root) {
            return Err(RuntimePropertyError::WrongObjectState);
        }
        focus.request(root, identity);
        Ok(())
    }

    pub(crate) fn focus_state(
        &self,
    ) -> Result<(Option<ObjectIdentity>, Option<ObjectIdentity>), RuntimePropertyError> {
        let focus = self
            .focus
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        Ok((focus.active_window(), focus.focused()))
    }

    fn clear_focus_for_detached_subtree(&self, root: ObjectIdentity) {
        let Ok(tree) = self.ui_tree.lock() else {
            return;
        };
        let Ok(mut focus) = self.focus.lock() else {
            return;
        };
        focus.clear_detached_subtree(root, |candidate| {
            tree.is_in_subtree(root, candidate).unwrap_or(false)
        });
    }

    pub(crate) fn calculate_window_layout(
        &self,
        identity: ObjectIdentity,
        measurer: &impl IntrinsicMeasureProvider,
    ) -> Result<Arc<LayoutSnapshot>, LayoutError> {
        if !self.is_ui_thread() || self.cleanup.state() != RuntimeState::Running {
            return Err(LayoutError::InvalidTree);
        }

        let (generation, client_area, current_layout_revision) = {
            let controls = self
                .ui_objects
                .lock()
                .map_err(|_| LayoutError::InvalidTree)?;
            controls
                .resolve(identity)
                .map_err(|_| LayoutError::InvalidTree)?
                .downcast_ref::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::Window)
                .and_then(|state| state.window.as_ref())
                .and_then(|window| window.layout_input())
                .ok_or(LayoutError::InvalidTree)?
        };
        let (tree_revision, root_input) = {
            let tree = self.ui_tree.lock().map_err(|_| LayoutError::InvalidTree)?;
            let count = tree
                .child_count(identity)
                .map_err(|_| LayoutError::InvalidTree)?;
            if count > 1 {
                return Err(LayoutError::InvalidTree);
            }
            let root = if count == 0 {
                None
            } else {
                let content = tree
                    .child_identity_at(identity, 0)
                    .map_err(|_| LayoutError::InvalidTree)?;
                Some(capture_layout_tree(&tree, content)?)
            };
            (tree.revision(), root)
        };
        let (root_snapshot, identities) = {
            let controls = self
                .ui_objects
                .lock()
                .map_err(|_| LayoutError::InvalidTree)?;
            let mut identities = Vec::new();
            identities
                .try_reserve(1)
                .map_err(|_| LayoutError::AllocationFailed)?;
            identities.push(identity);
            let root = root_input
                .as_ref()
                .map(|input| {
                    layout_node_from_tree(input, &controls, Align::Stretch, &mut identities)
                })
                .transpose()?;
            (root, identities)
        };
        let size = Size::new(client_area.width_dip, client_area.height_dip)?;
        let next_layout_revision = current_layout_revision
            .checked_add(1)
            .ok_or(LayoutError::ArithmeticOverflow)?;
        let mut candidate = match root_snapshot.as_ref() {
            Some(root) => {
                LayoutEngine::calculate(root, tree_revision, next_layout_revision, size, measurer)?
            }
            None => LayoutSnapshot::empty(tree_revision, next_layout_revision, size),
        };
        candidate.window_generation = generation;

        let (current_tree_revision, current_root_input) = {
            let tree = self.ui_tree.lock().map_err(|_| LayoutError::InvalidTree)?;
            let count = tree
                .child_count(identity)
                .map_err(|_| LayoutError::InvalidTree)?;
            if count > 1 {
                return Err(LayoutError::InvalidTree);
            }
            let root = if count == 0 {
                None
            } else {
                let content = tree
                    .child_identity_at(identity, 0)
                    .map_err(|_| LayoutError::InvalidTree)?;
                Some(capture_layout_tree(&tree, content)?)
            };
            (tree.revision(), root)
        };
        if current_tree_revision != tree_revision || current_root_input != root_input {
            return Err(LayoutError::InvalidTree);
        }
        let (current_root_snapshot, current_identities, current_window_input) = {
            let controls = self
                .ui_objects
                .lock()
                .map_err(|_| LayoutError::InvalidTree)?;
            let window_input = controls
                .resolve(identity)
                .map_err(|_| LayoutError::InvalidTree)?
                .downcast_ref::<UiControlState>()
                .filter(|state| state.kind == UiControlKind::Window)
                .and_then(|state| state.window.as_ref())
                .and_then(|window| window.layout_input());
            let mut identities = Vec::new();
            identities
                .try_reserve(1)
                .map_err(|_| LayoutError::AllocationFailed)?;
            identities.push(identity);
            let root = current_root_input
                .as_ref()
                .map(|input| {
                    layout_node_from_tree(input, &controls, Align::Stretch, &mut identities)
                })
                .transpose()?;
            (root, identities, window_input)
        };
        if current_root_snapshot != root_snapshot
            || current_identities != identities
            || current_window_input != Some((generation, client_area, current_layout_revision))
        {
            return Err(LayoutError::InvalidTree);
        }
        let mut controls = self
            .ui_objects
            .lock()
            .map_err(|_| LayoutError::InvalidTree)?;
        let window = controls
            .resolve_mut(identity)
            .map_err(|_| LayoutError::InvalidTree)?
            .downcast_mut::<UiControlState>()
            .filter(|state| state.kind == UiControlKind::Window)
            .and_then(|state| state.window.as_mut())
            .ok_or(LayoutError::InvalidTree)?;
        if window.layout_input() != Some((generation, client_area, current_layout_revision)) {
            return Err(LayoutError::InvalidTree);
        }
        let published = window
            .publish_layout(generation, client_area, candidate)
            .ok_or(LayoutError::InvalidTree)?;
        for control_identity in identities {
            if let Ok(object) = controls.resolve_mut(control_identity) {
                if let Some(state) = object.downcast_mut::<UiControlState>() {
                    state.layout_dirty = false;
                }
            }
        }
        Ok(published)
    }

    pub(crate) fn window_layout_snapshot(
        &self,
        identity: ObjectIdentity,
    ) -> Result<Option<Arc<LayoutSnapshot>>, LayoutError> {
        let controls = self
            .ui_objects
            .lock()
            .map_err(|_| LayoutError::InvalidTree)?;
        controls
            .resolve(identity)
            .map_err(|_| LayoutError::InvalidTree)?
            .downcast_ref::<UiControlState>()
            .filter(|state| state.kind == UiControlKind::Window)
            .and_then(|state| state.window.as_ref())
            .map(|window| window.layout_snapshot())
            .ok_or(LayoutError::InvalidTree)
    }

    fn mark_layout_dirty(&self, identity: ObjectIdentity) {
        let mut current = Some(identity);
        while let Some(candidate) = current {
            {
                let mut store = self
                    .ui_objects
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Ok(object) = store.resolve_mut(candidate) {
                    if let Some(state) = object.downcast_mut::<UiControlState>() {
                        state.layout_dirty = true;
                    }
                }
            }
            current = self
                .ui_tree
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .parent_identity(candidate)
                .ok()
                .flatten();
        }
    }

    fn mark_layout_subtree_dirty(&self, root: ObjectIdentity) {
        let mut pending = Vec::new();
        if pending.try_reserve(1).is_err() {
            self.mark_layout_dirty(root);
            return;
        }
        pending.push(root);
        while let Some(identity) = pending.pop() {
            if let Ok(mut store) = self.ui_objects.lock() {
                if let Ok(object) = store.resolve_mut(identity) {
                    if let Some(state) = object.downcast_mut::<UiControlState>() {
                        state.layout_dirty = true;
                    }
                }
            }
            let Ok(tree) = self.ui_tree.lock() else {
                continue;
            };
            let Ok(child_count) = tree.child_count(identity) else {
                continue;
            };
            if pending.try_reserve(child_count).is_err() {
                continue;
            }
            for index in 0..child_count {
                if let Ok(child) = tree.child_identity_at(identity, index) {
                    pending.push(child);
                }
            }
        }
    }

    /// Installs the single synchronous validator for a Property without validating its current value.
    pub(crate) fn register_property_validator(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        token: HandleToken,
        validator: Arc<PropertyValidator>,
    ) -> Result<(), RuntimePropertyError> {
        let pin = registry
            .validate(token, &PropertyHandleKind::Property, &self.identity)
            .map_err(RuntimePropertyError::Registry)?;
        self.ensure_property_access_thread()?;
        if self.validator_active.load(Ordering::Acquire) {
            return Err(RuntimePropertyError::ValidatorReentrancy);
        }
        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        store
            .resolve_mut(pin.identity())
            .map_err(RuntimePropertyError::Store)?
            .downcast_mut::<PropertyState>()
            .ok_or(RuntimePropertyError::WrongObjectState)?
            .register_validator(validator)
            .map_err(|_| RuntimePropertyError::ValidatorAlreadyRegistered)
    }

    pub(crate) fn cancel_property_validator(
        &self,
        identity: ObjectIdentity,
        validator: &Arc<PropertyValidator>,
    ) -> Result<bool, RuntimePropertyError> {
        if self.validator_active.load(Ordering::Acquire) {
            return Err(RuntimePropertyError::ValidatorReentrancy);
        }
        self.ensure_property_access_thread()?;
        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let state = store
            .resolve_mut(identity)
            .map_err(RuntimePropertyError::Store)?
            .downcast_mut::<PropertyState>()
            .ok_or(RuntimePropertyError::WrongObjectState)?;
        Ok(state.remove_validator(validator))
    }

    /// Validates target-group values before an initial Binding is published.
    /// Validators run only after the object and graph locks have been released. The caller then
    /// performs the create operation on this UI thread; callback reentrancy guards prevent state
    /// changes between this preflight and publication.
    pub(crate) fn validate_binding_initial_candidates(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        source_token: HandleToken,
        target_token: HandleToken,
        two_way: bool,
    ) -> Result<(), RuntimeBindingError> {
        if self.validator_active.load(Ordering::Acquire) {
            return Err(RuntimeBindingError::Property(
                RuntimePropertyError::ValidatorReentrancy,
            ));
        }
        let source_pin = registry
            .validate(source_token, &PropertyHandleKind::Property, &self.identity)
            .map_err(RuntimeBindingError::Registry)?;
        let target_pin = registry
            .validate(target_token, &PropertyHandleKind::Property, &self.identity)
            .map_err(RuntimeBindingError::Registry)?;
        self.ensure_property_access_thread()
            .map_err(RuntimeBindingError::Property)?;
        let source_identity = source_pin.identity();
        let target_identity = target_pin.identity();

        let checks = {
            let store = self
                .ui_objects
                .lock()
                .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
            let source = store
                .resolve(source_identity)
                .map_err(RuntimeBindingError::Store)?
                .downcast_ref::<PropertyState>()
                .ok_or(RuntimeBindingError::WrongObjectState)?;
            let source_value = source
                .read()
                .try_owned_copy()
                .map_err(RuntimeBindingError::SourceSnapshotCopy)?;
            let mut graph = self
                .binding_graph
                .lock()
                .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
            if two_way {
                graph
                    .prepare_two_way_merge(source_identity, target_identity)
                    .map_err(RuntimeBindingError::Graph)?;
            } else {
                graph
                    .prepare_one_way(source_identity, target_identity)
                    .map_err(RuntimeBindingError::Graph)?;
            }
            let members = graph
                .group_members(target_identity)
                .map_err(RuntimeBindingError::Graph)?;
            let mut checks = Vec::new();
            checks
                .try_reserve(members.len())
                .map_err(|_| RuntimeBindingError::PropagationPreparationOutOfMemory)?;
            for member in members {
                let state = store
                    .resolve(member)
                    .map_err(RuntimeBindingError::Store)?
                    .downcast_ref::<PropertyState>()
                    .ok_or(RuntimeBindingError::WrongObjectState)?;
                if state.is_read_only() {
                    return Err(RuntimeBindingError::Property(
                        RuntimePropertyError::Unsupported,
                    ));
                }
                state.would_change(&source_value).map_err(|error| {
                    RuntimeBindingError::InitialCopy(crate::value::BindingCopyError::KindMismatch(
                        error,
                    ))
                })?;
                if let Some(validator) = state.validator() {
                    let candidate = source_value.try_owned_copy().map_err(|error| {
                        RuntimeBindingError::InitialCopy(crate::value::BindingCopyError::Copy(
                            error,
                        ))
                    })?;
                    checks.push((validator, candidate));
                }
            }
            checks
        };

        if checks.is_empty() {
            return Ok(());
        }
        self.validator_active
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| {
                RuntimeBindingError::Property(RuntimePropertyError::ValidatorReentrancy)
            })?;
        let _guard = RuntimeFlagGuard(&self.validator_active);
        for (validator, candidate) in checks {
            match validator(&candidate) {
                ValidatorResult::Accepted => {}
                ValidatorResult::Rejected(error) => {
                    return Err(RuntimeBindingError::Property(
                        RuntimePropertyError::ValidationFailed(error),
                    ));
                }
                ValidatorResult::Failed => {
                    return Err(RuntimeBindingError::Property(
                        RuntimePropertyError::ValidatorFailed,
                    ));
                }
            }
        }
        Ok(())
    }

    /// Applies the initial OneWay source value and publishes an active Runtime connection.
    ///
    /// Returns only the value outcome; the ABI path uses the identity-returning companion below.
    pub(crate) fn create_one_way_binding_initial(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        source_token: HandleToken,
        target_token: HandleToken,
    ) -> Result<ValueCommit, RuntimeBindingError> {
        self.create_one_way_binding_initial_with_id(registry, source_token, target_token)
            .map(|(commit, _)| commit)
    }

    pub(crate) fn create_one_way_binding_initial_with_id(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        source_token: HandleToken,
        target_token: HandleToken,
    ) -> Result<(ValueCommit, BindingConnectionId), RuntimeBindingError> {
        if self.validator_active.load(Ordering::Acquire) {
            return Err(RuntimeBindingError::Property(
                RuntimePropertyError::ValidatorReentrancy,
            ));
        }
        let source_pin = registry
            .validate(source_token, &PropertyHandleKind::Property, &self.identity)
            .map_err(RuntimeBindingError::Registry)?;
        let target_pin = registry
            .validate(target_token, &PropertyHandleKind::Property, &self.identity)
            .map_err(RuntimeBindingError::Registry)?;
        self.ensure_property_access_thread()
            .map_err(RuntimeBindingError::Property)?;

        let source_identity = source_pin.identity();
        let target_identity = target_pin.identity();
        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        let source_value = store
            .resolve(source_identity)
            .map_err(RuntimeBindingError::Store)?
            .downcast_ref::<PropertyState>()
            .ok_or(RuntimeBindingError::WrongObjectState)?
            .read()
            .try_owned_copy()
            .map_err(RuntimeBindingError::SourceSnapshotCopy)?;

        let mut graph = self
            .binding_graph
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        let proposal = graph
            .prepare_one_way(source_identity, target_identity)
            .map_err(RuntimeBindingError::Graph)?;
        let connection_id = proposal.connection_id();
        let mut candidates = Vec::new();
        Self::prepare_group_value_candidates_locked(
            &store,
            &graph,
            target_identity,
            &source_value,
            &mut candidates,
        )?;

        if candidates.is_empty() {
            let mut owners = self
                .active_bindings
                .lock()
                .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
            owners
                .try_reserve(1)
                .map_err(|_| RuntimeBindingError::BindingOwnerOutOfMemory)?;
            graph
                .commit_one_way_with(proposal, || {
                    owners.insert(
                        connection_id,
                        RuntimeBindingOwner {
                            _source: source_pin,
                            _target: target_pin,
                            current_error: None,
                            error_subscribers: Vec::new(),
                        },
                    );
                })
                .map_err(RuntimeBindingError::Graph)?;
            return Ok((ValueCommit::Unchanged, connection_id));
        };

        let mut changes = self
            .property_changes
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        for candidate in &candidates {
            changes
                .reserve_change(candidate.target)
                .map_err(|_| RuntimeBindingError::NotificationQueueOutOfMemory)?;
        }
        let mut owners = self
            .active_bindings
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        owners
            .try_reserve(1)
            .map_err(|_| RuntimeBindingError::BindingOwnerOutOfMemory)?;
        graph
            .commit_one_way_with(proposal, || {
                for PreparedBindingValue {
                    target: identity,
                    value,
                } in candidates
                {
                    let target = store
                        .resolve_mut(identity)
                        .expect("prepared OneWay target remains in the locked object store")
                        .downcast_mut::<PropertyState>()
                        .expect("prepared OneWay target remains a Property state");
                    let outcome = target
                        .apply_validated(value)
                        .expect("prepared OneWay candidate has the target's fixed kind");
                    debug_assert_eq!(outcome, ValueCommit::Changed);
                    changes.commit_change(identity);
                }
                owners.insert(
                    connection_id,
                    RuntimeBindingOwner {
                        _source: source_pin,
                        _target: target_pin,
                        current_error: None,
                        error_subscribers: Vec::new(),
                    },
                );
            })
            .map_err(RuntimeBindingError::Graph)?;
        Ok((ValueCommit::Changed, connection_id))
    }

    /// Applies the initial TwoWay source value and publishes an active Runtime connection.
    ///
    /// Returns only the value outcome; the ABI path uses the identity-returning companion below.
    pub(crate) fn create_two_way_binding_initial(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        source_token: HandleToken,
        target_token: HandleToken,
    ) -> Result<ValueCommit, RuntimeBindingError> {
        self.create_two_way_binding_initial_with_id(registry, source_token, target_token)
            .map(|(commit, _)| commit)
    }

    pub(crate) fn create_two_way_binding_initial_with_id(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        source_token: HandleToken,
        target_token: HandleToken,
    ) -> Result<(ValueCommit, BindingConnectionId), RuntimeBindingError> {
        if self.validator_active.load(Ordering::Acquire) {
            return Err(RuntimeBindingError::Property(
                RuntimePropertyError::ValidatorReentrancy,
            ));
        }
        let source_pin = registry
            .validate(source_token, &PropertyHandleKind::Property, &self.identity)
            .map_err(RuntimeBindingError::Registry)?;
        let target_pin = registry
            .validate(target_token, &PropertyHandleKind::Property, &self.identity)
            .map_err(RuntimeBindingError::Registry)?;
        self.ensure_property_access_thread()
            .map_err(RuntimeBindingError::Property)?;

        let source_identity = source_pin.identity();
        let target_identity = target_pin.identity();
        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        let source_value = store
            .resolve(source_identity)
            .map_err(RuntimeBindingError::Store)?
            .downcast_ref::<PropertyState>()
            .ok_or(RuntimeBindingError::WrongObjectState)?
            .read()
            .try_owned_copy()
            .map_err(RuntimeBindingError::SourceSnapshotCopy)?;

        let mut graph = self
            .binding_graph
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        let proposal = graph
            .prepare_two_way_merge(source_identity, target_identity)
            .map_err(RuntimeBindingError::Graph)?;
        let connection_id = proposal.connection_id();
        let mut candidates = Vec::new();
        Self::prepare_group_value_candidates_locked(
            &store,
            &graph,
            target_identity,
            &source_value,
            &mut candidates,
        )?;

        if candidates.is_empty() {
            let mut owners = self
                .active_bindings
                .lock()
                .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
            owners
                .try_reserve(1)
                .map_err(|_| RuntimeBindingError::BindingOwnerOutOfMemory)?;
            graph
                .commit_two_way_merge_with(proposal, || {
                    owners.insert(
                        connection_id,
                        RuntimeBindingOwner {
                            _source: source_pin,
                            _target: target_pin,
                            current_error: None,
                            error_subscribers: Vec::new(),
                        },
                    );
                })
                .map_err(RuntimeBindingError::Graph)?;
            return Ok((ValueCommit::Unchanged, connection_id));
        }

        let mut changes = self
            .property_changes
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        for candidate in &candidates {
            changes
                .reserve_change(candidate.target)
                .map_err(|_| RuntimeBindingError::NotificationQueueOutOfMemory)?;
        }
        let mut owners = self
            .active_bindings
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        owners
            .try_reserve(1)
            .map_err(|_| RuntimeBindingError::BindingOwnerOutOfMemory)?;
        graph
            .commit_two_way_merge_with(proposal, || {
                for PreparedBindingValue {
                    target: identity,
                    value,
                } in candidates
                {
                    let target = store
                        .resolve_mut(identity)
                        .expect("prepared TwoWay target remains in the locked object store")
                        .downcast_mut::<PropertyState>()
                        .expect("prepared TwoWay target remains a Property state");
                    let outcome = target
                        .apply_validated(value)
                        .expect("prepared TwoWay candidate has the target's fixed kind");
                    debug_assert_eq!(outcome, ValueCommit::Changed);
                    changes.commit_change(identity);
                }
                owners.insert(
                    connection_id,
                    RuntimeBindingOwner {
                        _source: source_pin,
                        _target: target_pin,
                        current_error: None,
                        error_subscribers: Vec::new(),
                    },
                );
            })
            .map_err(RuntimeBindingError::Graph)?;
        Ok((ValueCommit::Changed, connection_id))
    }

    /// Cancels one active Runtime connection, preserving its already committed Property values.
    ///
    /// The caller validates an explicit Binding owner before passing the internal connection ID.
    pub(crate) fn cancel_binding_connection(
        &self,
        connection_id: BindingConnectionId,
    ) -> Result<bool, RuntimeBindingError> {
        if self.validator_active.load(Ordering::Acquire) {
            return Err(RuntimeBindingError::Property(
                RuntimePropertyError::ValidatorReentrancy,
            ));
        }
        self.ensure_property_access_thread()
            .map_err(RuntimeBindingError::Property)?;
        let mut graph = self
            .binding_graph
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        let mut owners = self
            .active_bindings
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        if !graph
            .remove_connection(connection_id)
            .map_err(RuntimeBindingError::Graph)?
        {
            return Ok(false);
        }
        let owner = owners.remove(&connection_id);
        debug_assert!(
            owner.is_some(),
            "active graph connection has a Runtime owner"
        );
        drop(owners);
        drop(graph);
        if let Some(owner) = owner {
            for subscription in owner.error_subscribers {
                subscription.cancel();
            }
        }
        self.binding_error_delivery
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?
            .retain(|item| item.connection != connection_id);
        Ok(true)
    }

    pub(crate) fn set_binding_validation_snapshot(
        &self,
        connection_id: BindingConnectionId,
        snapshot: Option<BindingValidationSnapshot>,
    ) -> Result<(), RuntimeBindingError> {
        let mut owners = self
            .active_bindings
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        let mut delivery = self
            .binding_error_delivery
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        let owner = owners
            .get_mut(&connection_id)
            .ok_or(RuntimeBindingError::WrongObjectState)?;
        if owner.current_error == snapshot {
            return Ok(());
        }
        let mut snapshot_subscribers = Vec::new();
        if !owner.error_subscribers.is_empty() {
            snapshot_subscribers
                .try_reserve(owner.error_subscribers.len())
                .map_err(|_| {
                    RuntimeBindingError::Property(
                        RuntimePropertyError::NotificationQueueOutOfMemory,
                    )
                })?;
            snapshot_subscribers.extend(owner.error_subscribers.iter().cloned());
            if let Some(index) = delivery
                .iter()
                .position(|item| item.connection == connection_id)
            {
                delivery[index].subscribers = snapshot_subscribers;
            } else {
                delivery.try_reserve(1).map_err(|_| {
                    RuntimeBindingError::Property(
                        RuntimePropertyError::NotificationQueueOutOfMemory,
                    )
                })?;
                delivery.push_back(BindingErrorDeliveryItem {
                    connection: connection_id,
                    subscribers: snapshot_subscribers,
                });
            }
        }
        owner.current_error = snapshot;
        Ok(())
    }

    pub(crate) fn register_binding_error_subscription(
        &self,
        connection_id: BindingConnectionId,
        subscription: &Arc<Subscription>,
    ) -> Result<(), RuntimeBindingError> {
        self.ensure_property_access_thread()
            .map_err(RuntimeBindingError::Property)?;
        let mut owners = self
            .active_bindings
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        let owner = owners
            .get_mut(&connection_id)
            .ok_or(RuntimeBindingError::WrongObjectState)?;
        owner.error_subscribers.try_reserve(1).map_err(|_| {
            RuntimeBindingError::Property(RuntimePropertyError::NotificationQueueOutOfMemory)
        })?;
        owner.error_subscribers.push(Arc::clone(subscription));
        Ok(())
    }

    pub(crate) fn cancel_binding_error_subscription(
        &self,
        connection_id: BindingConnectionId,
        subscription: &Arc<Subscription>,
    ) -> Result<(), RuntimeBindingError> {
        self.ensure_property_access_thread()
            .map_err(RuntimeBindingError::Property)?;
        let mut owners = self
            .active_bindings
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        if let Some(owner) = owners.get_mut(&connection_id) {
            owner
                .error_subscribers
                .retain(|item| !Arc::ptr_eq(item, subscription));
        }
        drop(owners);
        subscription.cancel();
        Ok(())
    }

    pub(crate) fn binding_property_pin(
        &self,
        connection_id: BindingConnectionId,
        identity: ObjectIdentity,
    ) -> Result<ValidationPin<RuntimeIdentity>, RuntimeBindingError> {
        let owners = self
            .active_bindings
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        owners
            .get(&connection_id)
            .ok_or(RuntimeBindingError::WrongObjectState)?;
        if let Some(pin) = owners.values().find_map(|owner| {
            if owner._source.identity() == identity {
                Some(&owner._source)
            } else if owner._target.identity() == identity {
                Some(&owner._target)
            } else {
                None
            }
        }) {
            return self.clone_property_pin(pin);
        }
        Err(RuntimeBindingError::WrongObjectState)
    }

    fn clone_property_pin(
        &self,
        pin: &ValidationPin<RuntimeIdentity>,
    ) -> Result<ValidationPin<RuntimeIdentity>, RuntimeBindingError> {
        pin.try_clone().map_err(|error| match error {
            RegistryError::OutOfMemory => {
                RuntimeBindingError::Property(RuntimePropertyError::NotificationQueueOutOfMemory)
            }
            _ => RuntimeBindingError::Property(RuntimePropertyError::WrongObjectState),
        })
    }

    pub(crate) fn take_binding_error_deliveries(
        &self,
    ) -> Result<VecDeque<BindingErrorDeliveryItem>, RuntimePropertyError> {
        let mut pending = self
            .binding_error_delivery
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        Ok(std::mem::take(&mut *pending))
    }

    #[allow(dead_code)] // Read by the Core binding error snapshot API in the next step.
    pub(crate) fn binding_validation_snapshot(
        &self,
        connection_id: BindingConnectionId,
    ) -> Result<Option<BindingValidationSnapshot>, RuntimeBindingError> {
        let owners = self
            .active_bindings
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        owners
            .get(&connection_id)
            .map(|owner| owner.current_error.clone())
            .ok_or(RuntimeBindingError::WrongObjectState)
    }

    /// Prepares owned target candidates for reachable OneWay edges without committing them.
    ///
    /// Earlier target candidates feed later edges in the same topological pass, so a chain uses
    /// the propagated value all the way downstream. This Runtime slice has no TwoWay value-group
    /// update path; it is used with the current Runtime OneWay registrations only.
    pub(crate) fn prepare_one_way_propagation_candidates(
        &self,
        dirty_properties: &[ObjectIdentity],
    ) -> Result<Vec<PreparedBindingValue>, RuntimeBindingError> {
        self.ensure_property_access_thread()
            .map_err(RuntimeBindingError::Property)?;

        // Keep the established object-store -> graph lock order used by Binding creation.
        let store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        let graph = self
            .binding_graph
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        Self::prepare_one_way_propagation_candidates_locked(&store, &graph, dirty_properties)
    }

    /// Prepares and commits OneWay target values, queuing their notifications for the next pass.
    ///
    /// The object store, graph, and pending-change queue remain locked from candidate preparation
    /// through notification reservation and value commit. No logical state changes until every
    /// candidate and notification slot has been prepared successfully.
    pub(crate) fn propagate_one_way_changes(
        &self,
        dirty_properties: &[ObjectIdentity],
    ) -> Result<Vec<ObjectIdentity>, RuntimeBindingError> {
        self.ensure_property_access_thread()
            .map_err(RuntimeBindingError::Property)?;
        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        let graph = self
            .binding_graph
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        let mut changes = self
            .property_changes
            .lock()
            .map_err(|_| RuntimeBindingError::SynchronizationPoisoned)?;
        Self::propagate_one_way_changes_locked(
            &mut store,
            &graph,
            &mut changes,
            dirty_properties,
            dirty_properties,
            &[],
        )
    }

    fn propagate_one_way_changes_locked(
        store: &mut UiObjectStore<Box<dyn Any + Send>>,
        graph: &BindingGraph,
        changes: &mut PropertyChangeQueue,
        dirty_properties: &[ObjectIdentity],
        direct_changes: &[ObjectIdentity],
        current_generation: &[ObjectIdentity],
    ) -> Result<Vec<ObjectIdentity>, RuntimeBindingError> {
        let prepared = Self::prepare_binding_propagation_candidates_locked(
            store,
            graph,
            dirty_properties,
            direct_changes,
        )?;
        let mut changed_targets = Vec::new();
        changed_targets
            .try_reserve(prepared.len())
            .map_err(|_| RuntimeBindingError::PropagationPreparationOutOfMemory)?;
        for candidate in &prepared {
            if !current_generation.contains(&candidate.target) {
                changes
                    .reserve_change(candidate.target)
                    .map_err(|_| RuntimeBindingError::NotificationQueueOutOfMemory)?;
            }
            changed_targets.push(candidate.target);
        }

        for candidate in prepared {
            let target = store
                .resolve_mut(candidate.target)
                .expect("prepared propagation target remains in the locked object store")
                .downcast_mut::<PropertyState>()
                .expect("prepared propagation target remains a Property state");
            let outcome = target
                .apply_validated(candidate.value)
                .expect("prepared propagation candidate has the target's fixed kind");
            debug_assert_eq!(outcome, ValueCommit::Changed);
            if !current_generation.contains(&candidate.target) {
                changes.commit_change(candidate.target);
            }
        }
        Ok(changed_targets)
    }

    fn propagate_validated_binding_changes(
        &self,
        dirty_properties: &[ObjectIdentity],
        direct_changes: &[ObjectIdentity],
        current_generation: &[ObjectIdentity],
    ) -> Result<(), RuntimePropertyError> {
        let direct_groups = {
            let graph = self
                .binding_graph
                .lock()
                .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
            let mut groups: Vec<(ObjectIdentity, ObjectIdentity)> = Vec::new();
            groups
                .try_reserve(direct_changes.len())
                .map_err(|_| RuntimePropertyError::BindingPropagationOutOfMemory)?;
            for identity in direct_changes {
                let representative = graph.representative(*identity);
                if let Some((_, winner)) = groups
                    .iter_mut()
                    .find(|(group, _)| *group == representative)
                {
                    *winner = *identity;
                } else {
                    groups.push((representative, *identity));
                }
            }
            groups
        };

        for (representative, winner) in direct_groups {
            let (connection_id, candidates, checks) = {
                let store = self
                    .ui_objects
                    .lock()
                    .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
                let graph = self
                    .binding_graph
                    .lock()
                    .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
                let Some(connection_id) = graph.two_way_connection_for_group(representative) else {
                    continue;
                };
                let value = store
                    .resolve(winner)
                    .map_err(RuntimePropertyError::Store)?
                    .downcast_ref::<PropertyState>()
                    .ok_or(RuntimePropertyError::WrongObjectState)?
                    .read()
                    .try_owned_copy()
                    .map_err(|error| {
                        RuntimePropertyError::BindingValueCopy(BindingCopyError::Copy(error))
                    })?;
                let mut candidates = Vec::new();
                Self::prepare_group_value_candidates_locked(
                    &store,
                    &graph,
                    representative,
                    &value,
                    &mut candidates,
                )
                .map_err(|error| error.into_property_delivery_error())?;
                let checks =
                    Self::validator_checks_for_group(&store, &graph, representative, &value)?;
                (connection_id, candidates, checks)
            };
            if let Some((rejected_property, error)) = self.validate_binding_candidates(checks)? {
                self.set_binding_validation_snapshot(
                    connection_id,
                    Some(BindingValidationSnapshot {
                        error,
                        rejected_property,
                        direction: connection_id.direction_from(winner),
                    }),
                )
                .map_err(|error| error.into_property_delivery_error())?;
                continue;
            }
            self.commit_binding_candidates(candidates, current_generation)?;
            self.set_binding_validation_snapshot(connection_id, None)
                .map_err(|error| error.into_property_delivery_error())?;
        }

        let edges = self
            .binding_graph
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?
            .prepare_one_way_propagation(dirty_properties)
            .map_err(RuntimePropertyError::BindingGraph)?;

        for edge in edges {
            let (candidates, checks) = {
                let store = self
                    .ui_objects
                    .lock()
                    .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
                let graph = self
                    .binding_graph
                    .lock()
                    .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;

                // A TwoWay source group uses the most recently changed member as its winner.
                let winner = direct_changes
                    .iter()
                    .rev()
                    .copied()
                    .find(|identity| graph.same_group(*identity, edge.source))
                    .unwrap_or(edge.source);
                let source_value = store
                    .resolve(winner)
                    .map_err(RuntimePropertyError::Store)?
                    .downcast_ref::<PropertyState>()
                    .ok_or(RuntimePropertyError::WrongObjectState)?
                    .read()
                    .try_owned_copy()
                    .map_err(|error| {
                        RuntimePropertyError::BindingValueCopy(BindingCopyError::Copy(error))
                    })?;
                let mut candidates = Vec::new();
                Self::prepare_group_value_candidates_locked(
                    &store,
                    &graph,
                    edge.target,
                    &source_value,
                    &mut candidates,
                )
                .map_err(|error| error.into_property_delivery_error())?;
                let checks =
                    Self::validator_checks_for_group(&store, &graph, edge.target, &source_value)?;
                (candidates, checks)
            };

            if let Some((rejected_property, error)) = self.validate_binding_candidates(checks)? {
                self.set_binding_validation_snapshot(
                    edge.id,
                    Some(BindingValidationSnapshot {
                        error,
                        rejected_property,
                        direction: edge.id.direction_from(edge.source),
                    }),
                )
                .map_err(|error| error.into_property_delivery_error())?;
                // The target group did not change, so later edges read its committed value.
                continue;
            }

            self.commit_binding_candidates(candidates, current_generation)?;
            self.set_binding_validation_snapshot(edge.id, None)
                .map_err(|error| error.into_property_delivery_error())?;
        }
        Ok(())
    }

    fn validator_checks_for_group(
        store: &UiObjectStore<Box<dyn Any + Send>>,
        graph: &BindingGraph,
        representative: ObjectIdentity,
        source_value: &TypedValue,
    ) -> Result<Vec<PreparedValidatorCheck>, RuntimePropertyError> {
        let members = graph
            .group_members(representative)
            .map_err(RuntimePropertyError::BindingGraph)?;
        let mut checks = Vec::new();
        checks
            .try_reserve(members.len())
            .map_err(|_| RuntimePropertyError::BindingPropagationOutOfMemory)?;
        for member in members {
            let state = store
                .resolve(member)
                .map_err(RuntimePropertyError::Store)?
                .downcast_ref::<PropertyState>()
                .ok_or(RuntimePropertyError::WrongObjectState)?;
            state.would_change(source_value).map_err(|error| {
                RuntimePropertyError::BindingValueCopy(BindingCopyError::KindMismatch(error))
            })?;
            if let Some(validator) = state.validator() {
                checks.push(PreparedValidatorCheck {
                    property: member,
                    validator,
                    candidate: source_value.try_owned_copy().map_err(|error| {
                        RuntimePropertyError::BindingValueCopy(BindingCopyError::Copy(error))
                    })?,
                });
            }
        }
        Ok(checks)
    }

    fn commit_binding_candidates(
        &self,
        candidates: Vec<PreparedBindingValue>,
        current_generation: &[ObjectIdentity],
    ) -> Result<(), RuntimePropertyError> {
        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let mut changes = self
            .property_changes
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        for candidate in &candidates {
            if !current_generation.contains(&candidate.target) {
                changes
                    .reserve_change(candidate.target)
                    .map_err(|_| RuntimePropertyError::NotificationQueueOutOfMemory)?;
            }
        }
        for candidate in candidates {
            let target = store
                .resolve_mut(candidate.target)
                .map_err(RuntimePropertyError::Store)?
                .downcast_mut::<PropertyState>()
                .ok_or(RuntimePropertyError::WrongObjectState)?;
            target
                .apply_validated(candidate.value)
                .expect("prepared propagation candidate has the target's fixed kind");
            if !current_generation.contains(&candidate.target) {
                changes.commit_change(candidate.target);
            }
        }
        Ok(())
    }

    fn validate_binding_candidates(
        &self,
        checks: Vec<PreparedValidatorCheck>,
    ) -> Result<Option<(ObjectIdentity, Arc<ValidationError>)>, RuntimePropertyError> {
        if checks.is_empty() {
            return Ok(None);
        }
        self.validator_active
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| RuntimePropertyError::ValidatorReentrancy)?;
        let _guard = RuntimeFlagGuard(&self.validator_active);
        for check in checks {
            let result = (check.validator)(&check.candidate);
            match result {
                ValidatorResult::Accepted => {}
                ValidatorResult::Rejected(error) => {
                    return Ok(Some((
                        check.property,
                        error.unwrap_or_else(|| {
                            Arc::new(ValidationError {
                                status: 11,
                                domain: 2,
                                code: 1,
                                message: b"binding target rejected the value".to_vec(),
                            })
                        }),
                    )));
                }
                ValidatorResult::Failed => {
                    return Ok(Some((
                        check.property,
                        Arc::new(ValidationError {
                            status: 11,
                            domain: 2,
                            code: 2,
                            message: b"binding target validator failed".to_vec(),
                        }),
                    )));
                }
            }
        }
        Ok(None)
    }

    fn prepare_one_way_propagation_candidates_locked(
        store: &UiObjectStore<Box<dyn Any + Send>>,
        graph: &BindingGraph,
        dirty_properties: &[ObjectIdentity],
    ) -> Result<Vec<PreparedBindingValue>, RuntimeBindingError> {
        Self::prepare_binding_propagation_candidates_locked(
            store,
            graph,
            dirty_properties,
            dirty_properties,
        )
    }

    fn prepare_binding_propagation_candidates_locked(
        store: &UiObjectStore<Box<dyn Any + Send>>,
        graph: &BindingGraph,
        dirty_properties: &[ObjectIdentity],
        direct_changes: &[ObjectIdentity],
    ) -> Result<Vec<PreparedBindingValue>, RuntimeBindingError> {
        let edges = graph
            .prepare_one_way_propagation(dirty_properties)
            .map_err(RuntimeBindingError::Graph)?;
        let mut prepared = Vec::new();
        prepared
            .try_reserve(edges.len().saturating_add(direct_changes.len()))
            .map_err(|_| RuntimeBindingError::PropagationPreparationOutOfMemory)?;

        let mut direct_group_winners: Vec<(ObjectIdentity, ObjectIdentity)> = Vec::new();
        direct_group_winners
            .try_reserve(direct_changes.len())
            .map_err(|_| RuntimeBindingError::PropagationPreparationOutOfMemory)?;
        for direct in direct_changes {
            let representative = graph.representative(*direct);
            if let Some((_, winner)) = direct_group_winners
                .iter_mut()
                .find(|(group, _)| *group == representative)
            {
                *winner = *direct;
            } else {
                direct_group_winners.push((representative, *direct));
            }
        }

        for &(representative, winner) in &direct_group_winners {
            let winner_value = store
                .resolve(winner)
                .map_err(RuntimeBindingError::Store)?
                .downcast_ref::<PropertyState>()
                .ok_or(RuntimeBindingError::WrongObjectState)?
                .read()
                .try_owned_copy()
                .map_err(RuntimeBindingError::SourceSnapshotCopy)?;
            Self::prepare_group_value_candidates_locked(
                store,
                graph,
                representative,
                &winner_value,
                &mut prepared,
            )?;
        }

        for edge in edges {
            let source_value = prepared
                .iter()
                .rev()
                .find(|candidate: &&PreparedBindingValue| candidate.target == edge.source)
                .map(|candidate| &candidate.value);
            let source_value = match source_value {
                Some(value) => value
                    .try_owned_copy()
                    .map_err(RuntimeBindingError::SourceSnapshotCopy)?,
                None => store
                    .resolve(edge.source)
                    .map_err(RuntimeBindingError::Store)?
                    .downcast_ref::<PropertyState>()
                    .ok_or(RuntimeBindingError::WrongObjectState)?
                    .read()
                    .try_owned_copy()
                    .map_err(RuntimeBindingError::SourceSnapshotCopy)?,
            };
            Self::prepare_group_value_candidates_locked(
                store,
                graph,
                edge.target,
                &source_value,
                &mut prepared,
            )?;
        }
        Ok(prepared)
    }

    fn prepare_group_value_candidates_locked(
        store: &UiObjectStore<Box<dyn Any + Send>>,
        graph: &BindingGraph,
        representative: ObjectIdentity,
        source_value: &TypedValue,
        prepared: &mut Vec<PreparedBindingValue>,
    ) -> Result<(), RuntimeBindingError> {
        let members = graph
            .group_members(representative)
            .map_err(RuntimeBindingError::Graph)?;
        for member in members {
            let target = store
                .resolve(member)
                .map_err(RuntimeBindingError::Store)?
                .downcast_ref::<PropertyState>()
                .ok_or(RuntimeBindingError::WrongObjectState)?;
            let candidate = target
                .value_for_binding_prepare()
                .prepare_binding_initial_copy(source_value)
                .map_err(RuntimeBindingError::InitialCopy)?;
            let position = prepared.iter().position(|item| item.target == member);
            match (candidate, position) {
                (Some(value), Some(position)) => prepared[position].value = value,
                (Some(value), None) => {
                    prepared
                        .try_reserve(1)
                        .map_err(|_| RuntimeBindingError::PropagationPreparationOutOfMemory)?;
                    prepared.push(PreparedBindingValue {
                        target: member,
                        value,
                    });
                }
                (None, Some(position)) => {
                    prepared.remove(position);
                }
                (None, None) => {}
            }
        }
        Ok(())
    }

    /// Reads an owned snapshot of a Property value on this Runtime's UI thread.
    pub(crate) fn read_property(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        token: HandleToken,
    ) -> Result<TypedValue, RuntimePropertyError> {
        let pin = registry
            .validate(token, &PropertyHandleKind::Property, &self.identity)
            .map_err(RuntimePropertyError::Registry)?;
        self.ensure_property_access_thread()?;
        let store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let state = store
            .resolve(pin.identity())
            .map_err(RuntimePropertyError::Store)?
            .downcast_ref::<PropertyState>()
            .ok_or(RuntimePropertyError::WrongObjectState)?;
        Ok(state.read().clone())
    }

    /// Inspects a borrowed Property value while its registry pin and UI-state guard are held.
    ///
    /// The operation is an internal adapter callback, not user code; callers must not retain the
    /// borrowed value or re-enter Runtime APIs from it.
    pub(crate) fn with_property_value<T>(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        token: HandleToken,
        inspect: impl FnOnce(&TypedValue) -> T,
    ) -> Result<T, RuntimePropertyError> {
        let pin = registry
            .validate(token, &PropertyHandleKind::Property, &self.identity)
            .map_err(RuntimePropertyError::Registry)?;
        self.ensure_property_access_thread()?;
        let store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let state = store
            .resolve(pin.identity())
            .map_err(RuntimePropertyError::Store)?
            .downcast_ref::<PropertyState>()
            .ok_or(RuntimePropertyError::WrongObjectState)?;
        Ok(inspect(state.read()))
    }

    /// Returns the fixed initial kind of a Property on this Runtime's UI thread.
    pub(crate) fn property_kind(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        token: HandleToken,
    ) -> Result<ValueKind, RuntimePropertyError> {
        let pin = registry
            .validate(token, &PropertyHandleKind::Property, &self.identity)
            .map_err(RuntimePropertyError::Registry)?;
        self.ensure_property_access_thread()?;
        let store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let state = store
            .resolve(pin.identity())
            .map_err(RuntimePropertyError::Store)?
            .downcast_ref::<PropertyState>()
            .ok_or(RuntimePropertyError::WrongObjectState)?;
        Ok(state.kind())
    }

    /// Applies an already-validated typed candidate to a Property on the UI thread.
    pub(crate) fn apply_property_candidate(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        token: HandleToken,
        candidate: TypedValue,
    ) -> Result<ValueCommit, RuntimePropertyError> {
        let pin = registry
            .validate(token, &PropertyHandleKind::Property, &self.identity)
            .map_err(RuntimePropertyError::Registry)?;
        self.ensure_property_access_thread()?;
        if self.validator_active.load(Ordering::Acquire) {
            return Err(RuntimePropertyError::ValidatorReentrancy);
        }
        let validator = {
            let store = self
                .ui_objects
                .lock()
                .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
            let state = store
                .resolve(pin.identity())
                .map_err(RuntimePropertyError::Store)?
                .downcast_ref::<PropertyState>()
                .ok_or(RuntimePropertyError::WrongObjectState)?;
            if state.is_read_only() {
                return Err(RuntimePropertyError::Unsupported);
            }
            state
                .would_change(&candidate)
                .map_err(RuntimePropertyError::ValueKindMismatch)?;
            state.validator()
        };
        if let Some(validator) = validator {
            self.validator_active
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .map_err(|_| RuntimePropertyError::ValidatorReentrancy)?;
            let _guard = RuntimeFlagGuard(&self.validator_active);
            match validator(&candidate) {
                ValidatorResult::Accepted => {}
                ValidatorResult::Rejected(error) => {
                    return Err(RuntimePropertyError::ValidationFailed(error));
                }
                ValidatorResult::Failed => return Err(RuntimePropertyError::ValidatorFailed),
            }
            drop(_guard);
        }
        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let state = store
            .resolve_mut(pin.identity())
            .map_err(RuntimePropertyError::Store)?
            .downcast_mut::<PropertyState>()
            .ok_or(RuntimePropertyError::WrongObjectState)?;
        let changed = state
            .would_change(&candidate)
            .map_err(RuntimePropertyError::ValueKindMismatch)?;
        if !changed {
            return state
                .apply_validated(candidate)
                .map_err(RuntimePropertyError::ValueKindMismatch);
        }

        let identity = pin.identity();
        let mut changes = self
            .property_changes
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        changes
            .reserve_direct_change(identity)
            .map_err(|_| RuntimePropertyError::NotificationQueueOutOfMemory)?;
        let outcome = state
            .apply_validated(candidate)
            .map_err(RuntimePropertyError::ValueKindMismatch)?;
        if outcome == ValueCommit::Changed {
            changes.commit_direct_change(identity);
        }
        Ok(outcome)
    }

    /// Atomically updates a Command's read-only state and can-execute Properties.
    pub(crate) fn apply_command_properties(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        state_token: HandleToken,
        can_execute_token: HandleToken,
        state_value: i64,
        can_execute_value: bool,
    ) -> Result<(), RuntimePropertyError> {
        let state_pin = registry
            .validate(state_token, &PropertyHandleKind::Property, &self.identity)
            .map_err(RuntimePropertyError::Registry)?;
        let can_execute_pin = registry
            .validate(
                can_execute_token,
                &PropertyHandleKind::Property,
                &self.identity,
            )
            .map_err(RuntimePropertyError::Registry)?;
        self.ensure_property_access_thread()?;
        if self.validator_active.load(Ordering::Acquire) {
            return Err(RuntimePropertyError::ValidatorReentrancy);
        }

        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let next_state = TypedValue::I64(state_value);
        let next_can_execute = TypedValue::Bool(can_execute_value);
        let (state_changed, can_execute_changed) = {
            let state_property = store
                .resolve(state_pin.identity())
                .map_err(RuntimePropertyError::Store)?
                .downcast_ref::<PropertyState>()
                .ok_or(RuntimePropertyError::WrongObjectState)?;
            let can_execute_property = store
                .resolve(can_execute_pin.identity())
                .map_err(RuntimePropertyError::Store)?
                .downcast_ref::<PropertyState>()
                .ok_or(RuntimePropertyError::WrongObjectState)?;
            if !state_property.is_read_only() || !can_execute_property.is_read_only() {
                return Err(RuntimePropertyError::WrongObjectState);
            }
            (
                state_property
                    .would_change(&next_state)
                    .map_err(RuntimePropertyError::ValueKindMismatch)?,
                can_execute_property
                    .would_change(&next_can_execute)
                    .map_err(RuntimePropertyError::ValueKindMismatch)?,
            )
        };
        let mut changes = self
            .property_changes
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        if state_changed {
            changes
                .reserve_direct_change(state_pin.identity())
                .map_err(|_| RuntimePropertyError::NotificationQueueOutOfMemory)?;
        }
        if can_execute_changed {
            changes
                .reserve_direct_change(can_execute_pin.identity())
                .map_err(|_| RuntimePropertyError::NotificationQueueOutOfMemory)?;
        }
        if state_changed {
            store
                .resolve_mut(state_pin.identity())
                .map_err(RuntimePropertyError::Store)?
                .downcast_mut::<PropertyState>()
                .ok_or(RuntimePropertyError::WrongObjectState)?
                .apply_validated(next_state)
                .map_err(RuntimePropertyError::ValueKindMismatch)?;
            changes.commit_direct_change(state_pin.identity());
        }
        if can_execute_changed {
            store
                .resolve_mut(can_execute_pin.identity())
                .map_err(RuntimePropertyError::Store)?
                .downcast_mut::<PropertyState>()
                .ok_or(RuntimePropertyError::WrongObjectState)?
                .apply_validated(next_can_execute)
                .map_err(RuntimePropertyError::ValueKindMismatch)?;
            changes.commit_direct_change(can_execute_pin.identity());
        }
        Ok(())
    }

    pub(crate) fn prepare_command_property_update(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        state_token: HandleToken,
        can_execute_token: HandleToken,
    ) -> Result<(), RuntimePropertyError> {
        let state_pin = registry
            .validate(state_token, &PropertyHandleKind::Property, &self.identity)
            .map_err(RuntimePropertyError::Registry)?;
        let can_execute_pin = registry
            .validate(
                can_execute_token,
                &PropertyHandleKind::Property,
                &self.identity,
            )
            .map_err(RuntimePropertyError::Registry)?;
        self.ensure_property_access_thread()?;
        let store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let state = store
            .resolve(state_pin.identity())
            .map_err(RuntimePropertyError::Store)?
            .downcast_ref::<PropertyState>()
            .ok_or(RuntimePropertyError::WrongObjectState)?;
        let can_execute = store
            .resolve(can_execute_pin.identity())
            .map_err(RuntimePropertyError::Store)?
            .downcast_ref::<PropertyState>()
            .ok_or(RuntimePropertyError::WrongObjectState)?;
        if !state.is_read_only() || !can_execute.is_read_only() {
            return Err(RuntimePropertyError::WrongObjectState);
        }
        let mut changes = self
            .property_changes
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        changes
            .reserve_direct_change(state_pin.identity())
            .map_err(|_| RuntimePropertyError::NotificationQueueOutOfMemory)?;
        changes
            .reserve_direct_change(can_execute_pin.identity())
            .map_err(|_| RuntimePropertyError::NotificationQueueOutOfMemory)
    }

    pub(crate) fn apply_command_properties_in_host_event(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        state_token: HandleToken,
        can_execute_token: HandleToken,
        state_value: i64,
        can_execute_value: bool,
    ) -> Result<(), RuntimeCoreError> {
        let scope_id = self.begin_host_event()?;
        let update = self
            .apply_command_properties(
                registry,
                state_token,
                can_execute_token,
                state_value,
                can_execute_value,
            )
            .map_err(RuntimeCoreError::Property);
        let close = self.end_host_event(scope_id);
        match (update, close) {
            (Err(error), _) => Err(error),
            (Ok(()), Err(error)) => Err(error),
            (Ok(()), Ok(_)) => Ok(()),
        }
    }

    /// Applies one public Property setter operation within a host event boundary.
    pub(crate) fn apply_property_candidate_in_host_event(
        &self,
        registry: &HandleRegistry<PropertyHandleKind, RuntimeIdentity>,
        token: HandleToken,
        candidate: TypedValue,
    ) -> Result<ValueCommit, RuntimeCoreError> {
        if self.validator_active.load(Ordering::Acquire) {
            return Err(RuntimeCoreError::Property(
                RuntimePropertyError::ValidatorReentrancy,
            ));
        }
        let scope_id = self.begin_host_event()?;
        let update = self
            .apply_property_candidate(registry, token, candidate)
            .map_err(RuntimeCoreError::Property);
        let close = self.end_host_event(scope_id);
        match (update, close) {
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error),
            (Ok(outcome), Ok(_)) => Ok(outcome),
        }
    }

    /// Takes the current pending Property generation on the owning UI thread.
    pub(crate) fn take_property_change_generation(
        &self,
    ) -> Result<Vec<ObjectIdentity>, RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        self.property_changes
            .lock()
            .map(|mut changes| {
                let generation = changes.take_generation();
                let _ = changes.take_direct_changes();
                generation
            })
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)
    }

    /// Prepares the pending generation as ordered delivery snapshots without invoking callbacks.
    pub(crate) fn prepare_property_delivery_generation(
        &self,
    ) -> Result<Vec<PropertyDeliveryItem>, RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let graph = self
            .binding_graph
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let mut changes = self
            .property_changes
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;

        let mut items = Vec::new();
        items
            .try_reserve(changes.pending_len())
            .map_err(|_| RuntimePropertyError::NotificationQueueOutOfMemory)?;
        let generation = changes.take_generation();
        let direct_changes = changes.take_direct_changes();
        for identity in &generation {
            let snapshot = match store
                .resolve_mut(*identity)
                .map_err(RuntimePropertyError::Store)
                .and_then(|state| {
                    state
                        .downcast_mut::<PropertyState>()
                        .ok_or(RuntimePropertyError::WrongObjectState)
                })
                .and_then(|property| {
                    property
                        .snapshot_subscriptions()
                        .map_err(|_| RuntimePropertyError::NotificationQueueOutOfMemory)
                }) {
                Ok(snapshot) => snapshot,
                Err(error) => {
                    changes.restore_generation(generation);
                    changes.restore_direct_changes(direct_changes);
                    return Err(error);
                }
            };
            items.push(PropertyDeliveryItem {
                identity: *identity,
                subscribers: snapshot,
            });
        }
        drop(changes);
        drop(graph);
        drop(store);
        if let Err(error) =
            self.propagate_validated_binding_changes(&generation, &direct_changes, &generation)
        {
            let mut changes = self
                .property_changes
                .lock()
                .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
            changes.restore_generation(generation);
            changes.restore_direct_changes(direct_changes);
            return Err(error);
        }
        Ok(items)
    }

    /// Delivers at most 128 complete Property items and retains any remaining work.
    pub(crate) fn process_property_delivery_slice(
        &self,
    ) -> Result<(usize, bool), RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        if self
            .property_delivery_active
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Ok((0, true));
        }
        let _active = PropertyDeliveryGuard(&self.property_delivery_active);
        let mut completed = 0;

        loop {
            if completed == 128 {
                return Ok((completed, self.has_pending_property_delivery()?));
            }

            let item = {
                let mut pending = self
                    .property_delivery
                    .lock()
                    .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
                if pending.as_ref().is_some_and(VecDeque::is_empty) {
                    *pending = None;
                }
                pending.as_mut().and_then(VecDeque::pop_front)
            };

            if let Some(item) = item {
                item.deliver_with(|| {
                    self.cleanup.drain_pending().map_err(|error| match error {
                        ShutdownError::WrongThread => RuntimePropertyError::WrongThread,
                        ShutdownError::SynchronizationPoisoned => {
                            RuntimePropertyError::SynchronizationPoisoned
                        }
                    })
                })?;
                completed += 1;
                let shutdown_pending = self
                    .dispatcher
                    .lock()
                    .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?
                    .as_ref()
                    .map(Dispatcher::shutdown_is_pending)
                    .transpose()
                    .map_err(RuntimePropertyError::Dispatch)?
                    .unwrap_or(false);
                if shutdown_pending {
                    return Ok((completed, self.has_pending_property_delivery()?));
                }
                continue;
            }

            let generation = self.prepare_property_delivery_generation()?;
            if generation.is_empty() {
                if self.deliver_binding_error_notifications()? {
                    continue;
                }
                return Ok((completed, false));
            }
            self.deliver_binding_error_notifications()?;
            let mut pending = self
                .property_delivery
                .lock()
                .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
            *pending = Some(generation.into());
        }
    }

    fn deliver_binding_error_notifications(&self) -> Result<bool, RuntimePropertyError> {
        let items = self.take_binding_error_deliveries()?;
        if items.is_empty() {
            return Ok(false);
        }
        for item in items {
            for subscription in item.subscribers {
                subscription.invoke();
                self.cleanup.drain_pending().map_err(|error| match error {
                    ShutdownError::WrongThread => RuntimePropertyError::WrongThread,
                    ShutdownError::SynchronizationPoisoned => {
                        RuntimePropertyError::SynchronizationPoisoned
                    }
                })?;
            }
        }
        Ok(true)
    }

    fn has_pending_property_delivery(&self) -> Result<bool, RuntimePropertyError> {
        let pending = self
            .property_delivery
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        if pending.as_ref().is_some_and(|items| !items.is_empty()) {
            return Ok(true);
        }
        drop(pending);
        self.property_changes
            .lock()
            .map(|changes| changes.pending_len() > 0)
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)
    }

    /// Adds an independent Runtime owner for an active Subscription.
    pub(crate) fn register_subscription_owner(
        &self,
        identity: ObjectIdentity,
        subscription: &Arc<Subscription>,
    ) -> Result<(), RuntimePropertyError> {
        if self.validator_active.load(Ordering::Acquire) {
            return Err(RuntimePropertyError::ValidatorReentrancy);
        }
        self.ensure_property_access_thread()?;
        if !subscription.is_active() {
            return Err(RuntimePropertyError::WrongObjectState);
        }

        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let owner_is_property = store
            .resolve_mut(identity)
            .map_err(RuntimePropertyError::Store)?
            .downcast_mut::<PropertyState>()
            .is_some();
        if !owner_is_property {
            let is_window = store
                .resolve(identity)
                .ok()
                .and_then(|object| object.downcast_ref::<UiControlState>())
                .is_some_and(|state| state.kind == UiControlKind::Window);
            if !is_window {
                return Err(RuntimePropertyError::WrongObjectState);
            }
        }

        let mut owners = self
            .subscription_owners
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        if owners
            .values()
            .flatten()
            .any(|owner| Arc::ptr_eq(owner, subscription))
        {
            return Err(RuntimePropertyError::WrongObjectState);
        }
        if !owner_is_property
            && owners
                .get(&identity)
                .is_some_and(|bucket| !bucket.is_empty())
        {
            return Err(RuntimePropertyError::WrongObjectState);
        }

        let mut new_bucket = None;
        if let Some(bucket) = owners.get_mut(&identity) {
            bucket
                .try_reserve(1)
                .map_err(|_| RuntimePropertyError::SubscriptionRegistryOutOfMemory)?;
        } else {
            owners
                .try_reserve(1)
                .map_err(|_| RuntimePropertyError::SubscriptionRegistryOutOfMemory)?;
            let mut bucket = Vec::new();
            bucket
                .try_reserve(1)
                .map_err(|_| RuntimePropertyError::SubscriptionRegistryOutOfMemory)?;
            new_bucket = Some(bucket);
        }
        if owner_is_property {
            store
                .resolve_mut(identity)
                .map_err(RuntimePropertyError::Store)?
                .downcast_mut::<PropertyState>()
                .ok_or(RuntimePropertyError::WrongObjectState)?
                .add_subscription(subscription)
                .map_err(|_| RuntimePropertyError::SubscriptionRegistryOutOfMemory)?;
        }
        let owner = Arc::clone(subscription);
        if let Some(bucket) = owners.get_mut(&identity) {
            bucket.push(owner);
        } else if let Some(mut bucket) = new_bucket {
            bucket.push(owner);
            owners.insert(identity, bucket);
        }
        Ok(())
    }

    pub(crate) fn snapshot_window_close_subscription(
        &self,
        identity: ObjectIdentity,
    ) -> Result<Option<Arc<Subscription>>, RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        let is_window = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?
            .resolve(identity)
            .ok()
            .and_then(|object| object.downcast_ref::<UiControlState>())
            .is_some_and(|state| state.kind == UiControlKind::Window);
        if !is_window {
            return Err(RuntimePropertyError::WrongObjectState);
        }
        let owners = self
            .subscription_owners
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        Ok(owners
            .get(&identity)
            .and_then(|bucket| bucket.first())
            .filter(|subscription| subscription.is_active())
            .cloned())
    }

    /// Pins the Property's active subscriptions in registration order.
    pub(crate) fn snapshot_property_subscriptions(
        &self,
        identity: ObjectIdentity,
    ) -> Result<Vec<Arc<Subscription>>, RuntimePropertyError> {
        self.ensure_property_access_thread()?;
        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        store
            .resolve_mut(identity)
            .map_err(RuntimePropertyError::Store)?
            .downcast_mut::<PropertyState>()
            .ok_or(RuntimePropertyError::WrongObjectState)?
            .snapshot_subscriptions()
            .map_err(|_| RuntimePropertyError::SubscriptionRegistryOutOfMemory)
    }

    /// Removes one Runtime owner and cancels it outside the registry lock.
    pub(crate) fn cancel_subscription_owner(
        &self,
        identity: ObjectIdentity,
        subscription: &Arc<Subscription>,
    ) -> Result<bool, RuntimePropertyError> {
        if self.validator_active.load(Ordering::Acquire) {
            return Err(RuntimePropertyError::ValidatorReentrancy);
        }
        if !subscription.is_active() {
            return Ok(false);
        }
        self.ensure_property_access_thread()?;
        let mut store = self
            .ui_objects
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let mut owners = self
            .subscription_owners
            .lock()
            .map_err(|_| RuntimePropertyError::SynchronizationPoisoned)?;
        let Some(bucket) = owners.get_mut(&identity) else {
            return Ok(false);
        };
        let Some(index) = bucket
            .iter()
            .position(|owner| Arc::ptr_eq(owner, subscription))
        else {
            return Ok(false);
        };
        let removed = bucket.remove(index);
        if let Ok(state) = store.resolve_mut(identity) {
            if let Some(property) = state.downcast_mut::<PropertyState>() {
                property.remove_subscription(&removed);
            }
        }
        if bucket.is_empty() {
            owners.remove(&identity);
        }
        drop(owners);
        drop(store);
        removed.cancel();
        Ok(true)
    }

    fn ensure_property_access_thread(&self) -> Result<(), RuntimePropertyError> {
        if !self.is_ui_thread() {
            return Err(RuntimePropertyError::WrongThread);
        }
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimePropertyError::Closed);
        }
        Ok(())
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
        let registered = Dispatcher::new_for_ui_thread(Arc::clone(bridge), self.ui_thread);
        let pending_work_dispatcher = registered.clone();
        *dispatcher = Some(registered);
        drop(dispatcher);
        self.cleanup.set_pending_work_waker(Arc::new(move || {
            pending_work_dispatcher.schedule_external_work_wake();
        }));
        Ok(())
    }

    pub(crate) fn dispatcher(&self) -> Result<Option<Dispatcher>, RuntimeCoreError> {
        self.dispatcher
            .lock()
            .map(|dispatcher| dispatcher.clone())
            .map_err(|_| RuntimeCoreError::SynchronizationPoisoned)
    }

    pub(crate) fn dispatch_turn(&self) -> Result<(), RuntimeCoreError> {
        if !self.is_ui_thread() {
            return Err(RuntimeCoreError::WrongThread);
        }
        if self.property_delivery_active.load(Ordering::Acquire) {
            return Ok(());
        }
        let dispatcher = self
            .dispatcher()?
            .ok_or(RuntimeCoreError::BridgeNotRegistered)?;
        let scope_id = self.begin_host_event()?;
        let dispatch = dispatcher
            .dispatch_normal_turn()
            .map_err(RuntimeCoreError::Dispatch);
        let close = self.end_host_event(scope_id);
        match (dispatch, close) {
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error),
            (Ok(_), Ok(_)) => Ok(()),
        }
    }

    pub(crate) fn begin_host_event(&self) -> Result<u64, RuntimeCoreError> {
        if self.cleanup.state() != RuntimeState::Running {
            return Err(RuntimeCoreError::Closed);
        }
        self.host
            .resume_on_ui_thread()
            .and_then(|()| self.host.begin_event())
            .map_err(RuntimeCoreError::Host)
    }

    /// Close one host scope and commit shutdown cleanup at the outer boundary.
    pub(crate) fn end_host_event(&self, scope_id: u64) -> Result<bool, RuntimeCoreError> {
        let Some(dispatcher) = self.dispatcher()? else {
            let outermost = self
                .host
                .end_event(scope_id)
                .map_err(RuntimeCoreError::Host)?;
            if outermost {
                self.cleanup.drain_pending().map_err(|error| {
                    RuntimeCoreError::Shutdown(HostShutdownCommitError::Commit(
                        ShutdownCommitError::Cleanup(error),
                    ))
                })?;
            }
            return Ok(false);
        };
        let mut boundary_result: Option<Result<(usize, bool), RuntimeCoreError>> = None;
        let committed = self
            .host
            .end_event_and_commit_shutdown_or_else(scope_id, &dispatcher, &self.cleanup, || {
                let drain = self.cleanup.drain_pending().map_err(|error| {
                    RuntimeCoreError::Shutdown(HostShutdownCommitError::Commit(
                        ShutdownCommitError::Cleanup(error),
                    ))
                });
                match drain {
                    Err(error) => boundary_result = Some(Err(error)),
                    Ok(()) => match dispatcher.shutdown_is_pending() {
                        Ok(true) => boundary_result = Some(Ok((0, false))),
                        Ok(false) => {
                            boundary_result = Some(
                                self.process_property_delivery_slice()
                                    .map_err(RuntimeCoreError::Property),
                            );
                        }
                        Err(error) => {
                            boundary_result = Some(Err(RuntimeCoreError::Dispatch(error)));
                        }
                    },
                }
            })
            .map_err(RuntimeCoreError::Shutdown)?;
        if committed {
            dispatcher
                .destroy_bridge_at_safe_point()
                .map_err(|_| RuntimeCoreError::Closed)?;
            return Ok(true);
        }
        if let Some(delivery_result) = boundary_result {
            let committed = self
                .host
                .commit_pending_shutdown_at_safe_point(&dispatcher, &self.cleanup)
                .map_err(RuntimeCoreError::Shutdown)?;
            if committed {
                dispatcher
                    .destroy_bridge_at_safe_point()
                    .map_err(|_| RuntimeCoreError::Closed)?;
                return Ok(true);
            }
            let (_, has_more) = delivery_result?;
            if has_more {
                dispatcher
                    .schedule_pending_work_wake()
                    .map_err(RuntimeCoreError::Dispatch)?;
            }
        }
        Ok(false)
    }

    pub(crate) fn notify_host_stopped(&self, reason: u32) -> Result<(), RuntimeCoreError> {
        self.host
            .notify_stopped(reason)
            .map_err(RuntimeCoreError::Host)
    }

    pub(crate) fn request_shutdown(
        &self,
        mode: ShutdownRequestMode,
    ) -> Result<(ShutdownRequest, ShutdownRequestOutcome), RuntimeCoreError> {
        let dispatcher = self
            .dispatcher()?
            .ok_or(RuntimeCoreError::BridgeNotRegistered)?;
        let mut stored = self
            .shutdown_completion
            .lock()
            .map_err(|_| RuntimeCoreError::SynchronizationPoisoned)?;
        let completion = stored
            .clone()
            .unwrap_or_else(|| ShutdownCompletion::new_for_ui_thread(self.ui_thread));

        match completion
            .query()
            .map_err(|_| RuntimeCoreError::SynchronizationPoisoned)?
        {
            ShutdownCompletionState::Completed => {
                return Ok((
                    ShutdownRequest { completion },
                    ShutdownRequestOutcome::Completed,
                ));
            }
            ShutdownCompletionState::HostUnavailable { .. } => {
                self.host
                    .retry_on_ui_thread(&completion, &dispatcher)
                    .map_err(RuntimeCoreError::Host)?;
            }
            ShutdownCompletionState::Pending => {}
        }

        let result = self
            .host
            .request_shutdown_with_mode(&dispatcher, &completion, mode)
            .map_err(RuntimeCoreError::Host)?;
        if stored.is_none() {
            *stored = Some(completion.clone());
        }
        Ok((ShutdownRequest { completion }, result))
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
        BindingValidationSnapshot, HostBoundaryError, HostRegistration, HostRegistrationError,
        HostShutdownCommitError, RuntimeBindingError, RuntimeCleanupQueue, RuntimeCore,
        RuntimeCoreError, RuntimeIdentity, RuntimeObjectRegistrationError, RuntimePropertyError,
        RuntimeState, ShutdownCompletion, ShutdownCompletionError, ShutdownCompletionState,
        ShutdownRequestMode, ShutdownRequestOutcome, ShutdownWaitResult, UiThreadContext,
        ValidatorResult,
    };
    use crate::dispatcher::{Dispatcher, PostedWork, UiLoopBridge, WakePriority};
    use crate::handle_registry::registry::{
        CleanupQueue, CoreObjectControl, HandleRegistry, ObjectIdentity, RegistryError,
    };
    use crate::object_store::UiObjectStoreError;
    use crate::property::ValidationError;
    use crate::property::{PropertyHandleKind, Subscription};
    use crate::value::{
        BindingCopyError, DecimalValue, OwnedUtf8, TypedValue, ValueCommit, ValueKind,
        ValueKindMismatch,
    };
    use std::ffi::c_void;
    use std::num::NonZeroU64;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread::{self, ThreadId};

    struct RuntimeUiStateProbe {
        drop_count: Arc<AtomicUsize>,
        drop_threads: Arc<Mutex<Vec<ThreadId>>>,
    }

    #[derive(Default)]
    struct SubscriptionOwnerProbe {
        callbacks: AtomicUsize,
        destroys: AtomicUsize,
    }

    #[derive(Default)]
    struct DeliveryProbe {
        callbacks: AtomicUsize,
        destroys: AtomicUsize,
    }

    struct DeliveryCallbackContext {
        runtime: Arc<RuntimeCore>,
        registry: Arc<HandleRegistry<PropertyHandleKind, RuntimeIdentity>>,
        token: crate::handle_registry::HandleToken,
        identity: ObjectIdentity,
        later_subscription: Arc<Subscription>,
        calls: Arc<AtomicUsize>,
        outcomes: Arc<AtomicUsize>,
        destroys: Arc<AtomicUsize>,
    }

    struct PropertyReleaseCallbackContext {
        registry: Arc<HandleRegistry<PropertyHandleKind, RuntimeIdentity>>,
        token: crate::handle_registry::HandleToken,
        callbacks: Arc<AtomicUsize>,
        destroys: Arc<AtomicUsize>,
    }

    #[derive(Default)]
    struct OrderedDeliveryProbe {
        callbacks: Mutex<Vec<usize>>,
        destroys: AtomicUsize,
        destroy_threads: Mutex<Vec<ThreadId>>,
        reentrant_slice_blocked: AtomicBool,
        reentrant_turn_blocked: AtomicBool,
    }

    struct OrderedDeliveryContext {
        probe: Arc<OrderedDeliveryProbe>,
        item: usize,
        runtime: Option<Arc<RuntimeCore>>,
        shutdown_dispatcher: Option<Dispatcher>,
    }

    struct BindingDeliveryContext {
        runtime: Arc<RuntimeCore>,
        registry: Arc<HandleRegistry<PropertyHandleKind, RuntimeIdentity>>,
        observed_property: crate::handle_registry::HandleToken,
        item: usize,
        observations: Arc<Mutex<Vec<(usize, i64)>>>,
    }

    unsafe extern "C" fn subscription_owner_callback(userdata: *mut c_void) {
        // SAFETY: each test keeps its boxed probe alive until after explicit cancellation.
        let probe = unsafe { &*userdata.cast::<SubscriptionOwnerProbe>() };
        probe.callbacks.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn subscription_owner_destroy(userdata: *mut c_void) {
        // SAFETY: each test keeps its boxed probe alive until after explicit cancellation.
        let probe = unsafe { &*userdata.cast::<SubscriptionOwnerProbe>() };
        probe.destroys.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn delivery_probe_callback(userdata: *mut c_void) {
        // SAFETY: the Subscription owns this boxed Arc until its destroy callback runs.
        let probe = unsafe { &*userdata.cast::<Arc<DeliveryProbe>>() };
        probe.callbacks.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn delivery_probe_destroy(userdata: *mut c_void) {
        // SAFETY: userdata was produced by Box::into_raw in the test below.
        let probe = unsafe { &*userdata.cast::<Arc<DeliveryProbe>>() };
        probe.destroys.fetch_add(1, Ordering::SeqCst);
        // SAFETY: this callback consumes the unique Box allocated for this userdata.
        drop(unsafe { Box::from_raw(userdata.cast::<Arc<DeliveryProbe>>()) });
    }

    unsafe extern "C" fn delivery_callback(userdata: *mut c_void) {
        // SAFETY: the Subscription owns this boxed context until its destroy callback runs.
        let context = unsafe { &*userdata.cast::<DeliveryCallbackContext>() };
        match context.calls.fetch_add(1, Ordering::SeqCst) {
            0 => {
                if context.runtime.apply_property_candidate(
                    &context.registry,
                    context.token,
                    TypedValue::I64(2),
                ) == Ok(ValueCommit::Changed)
                {
                    context.outcomes.fetch_or(1, Ordering::SeqCst);
                }
                if context
                    .runtime
                    .register_subscription_owner(context.identity, &context.later_subscription)
                    .is_ok()
                {
                    context.outcomes.fetch_or(2, Ordering::SeqCst);
                }
            }
            1 => {
                if context
                    .runtime
                    .cancel_subscription_owner(context.identity, &context.later_subscription)
                    == Ok(true)
                {
                    context.outcomes.fetch_or(4, Ordering::SeqCst);
                }
            }
            _ => {}
        }
    }

    unsafe extern "C" fn delivery_callback_destroy(userdata: *mut c_void) {
        // SAFETY: userdata was produced by Box::into_raw in the test below.
        let context = unsafe { Box::from_raw(userdata.cast::<DeliveryCallbackContext>()) };
        context.destroys.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn release_property_callback(userdata: *mut c_void) {
        // SAFETY: the Subscription owns this context until release_property_destroy runs.
        let context = unsafe { &*userdata.cast::<PropertyReleaseCallbackContext>() };
        context.callbacks.fetch_add(1, Ordering::SeqCst);
        context
            .registry
            .release(context.token)
            .expect("callback releases the final Property owner");
    }

    unsafe extern "C" fn release_property_destroy(userdata: *mut c_void) {
        // SAFETY: userdata was produced by Box::into_raw in the test below.
        let context = unsafe { Box::from_raw(userdata.cast::<PropertyReleaseCallbackContext>()) };
        context.destroys.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn ordered_delivery_callback(userdata: *mut c_void) {
        // SAFETY: the Subscription owns this boxed context until its destroy callback runs.
        let context = unsafe { &*userdata.cast::<OrderedDeliveryContext>() };
        context
            .probe
            .callbacks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(context.item);
        if let Some(runtime) = &context.runtime {
            context.probe.reentrant_slice_blocked.store(
                runtime.process_property_delivery_slice() == Ok((0, true)),
                Ordering::SeqCst,
            );
            context
                .probe
                .reentrant_turn_blocked
                .store(runtime.dispatch_turn() == Ok(()), Ordering::SeqCst);
        }
        if context.item == 0 {
            if let Some(dispatcher) = &context.shutdown_dispatcher {
                dispatcher
                    .request_shutdown()
                    .expect("notification callback accepts priority shutdown");
            }
        }
    }

    unsafe extern "C" fn ordered_delivery_destroy(userdata: *mut c_void) {
        // SAFETY: userdata was produced by Box::into_raw in the test below.
        let context = unsafe { Box::from_raw(userdata.cast::<OrderedDeliveryContext>()) };
        context.probe.destroys.fetch_add(1, Ordering::SeqCst);
        context
            .probe
            .destroy_threads
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(thread::current().id());
    }

    unsafe extern "C" fn binding_delivery_callback(userdata: *mut c_void) {
        // SAFETY: the Subscription owns this context until binding_delivery_destroy runs.
        let context = unsafe { &*userdata.cast::<BindingDeliveryContext>() };
        let value = context
            .runtime
            .read_property(&context.registry, context.observed_property)
            .expect("subscriber can read the propagated Property value");
        let TypedValue::I64(value) = value else {
            panic!("test Property retains its I64 kind");
        };
        context
            .observations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push((context.item, value));
    }

    unsafe extern "C" fn binding_delivery_destroy(userdata: *mut c_void) {
        // SAFETY: userdata was produced by Box::into_raw in the test below.
        drop(unsafe { Box::from_raw(userdata.cast::<BindingDeliveryContext>()) });
    }

    fn new_binding_delivery_subscription(
        runtime: &Arc<RuntimeCore>,
        registry: &Arc<HandleRegistry<PropertyHandleKind, RuntimeIdentity>>,
        observed_property: crate::handle_registry::HandleToken,
        item: usize,
        observations: &Arc<Mutex<Vec<(usize, i64)>>>,
    ) -> Arc<Subscription> {
        let context = Box::new(BindingDeliveryContext {
            runtime: Arc::clone(runtime),
            registry: Arc::clone(registry),
            observed_property,
            item,
            observations: Arc::clone(observations),
        });
        let userdata = Box::into_raw(context).cast::<c_void>();
        // SAFETY: the Subscription owns its boxed context until explicit cancellation.
        Arc::new(unsafe {
            Subscription::new(
                binding_delivery_callback,
                userdata,
                binding_delivery_destroy,
            )
        })
    }

    #[test]
    fn runtime_two_way_binding_initial_sync_commits_target_group_and_notification() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let source = runtime
            .register_property(&registry, TypedValue::I64(42))
            .expect("register source Property");
        let target = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register target Property");
        let source_identity = registry
            .validate(source, &PropertyHandleKind::Property, &runtime.identity())
            .expect("source remains valid")
            .identity();
        let target_identity = registry
            .validate(target, &PropertyHandleKind::Property, &runtime.identity())
            .expect("target remains valid")
            .identity();

        assert_eq!(
            runtime.apply_property_candidate(&registry, target, TypedValue::I64(7)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.create_two_way_binding_initial(&registry, source, target),
            Ok(ValueCommit::Changed)
        );
        assert!(matches!(
            runtime.read_property(&registry, target),
            Ok(TypedValue::I64(42))
        ));
        assert!(
            runtime
                .binding_graph
                .lock()
                .expect("graph lock")
                .same_group(source_identity, target_identity)
        );
        assert_eq!(
            runtime.take_property_change_generation(),
            Ok(vec![target_identity])
        );

        assert_eq!(
            runtime.create_two_way_binding_initial(&registry, source, target),
            Err(RuntimeBindingError::Graph(
                crate::binding::BindingGraphError::DuplicateEdge
            ))
        );

        let same_source = runtime
            .register_property(&registry, TypedValue::I64(9))
            .expect("register same-value source");
        let same_target = runtime
            .register_property(&registry, TypedValue::I64(9))
            .expect("register same-value target");
        let same_source_identity = registry
            .validate(
                same_source,
                &PropertyHandleKind::Property,
                &runtime.identity(),
            )
            .expect("same source remains valid")
            .identity();
        let same_target_identity = registry
            .validate(
                same_target,
                &PropertyHandleKind::Property,
                &runtime.identity(),
            )
            .expect("same target remains valid")
            .identity();
        assert_eq!(
            runtime.create_two_way_binding_initial(&registry, same_source, same_target),
            Ok(ValueCommit::Unchanged)
        );
        assert!(
            runtime
                .binding_graph
                .lock()
                .expect("graph lock")
                .same_group(same_source_identity, same_target_identity)
        );
        assert_eq!(runtime.take_property_change_generation(), Ok(Vec::new()));
    }

    #[test]
    fn runtime_property_validator_checks_changed_and_equivalent_candidates_before_commit() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let property = runtime
            .register_property(&registry, TypedValue::I64(3))
            .expect("register Property");
        let calls = Arc::new(AtomicUsize::new(0));
        let validator_calls = Arc::clone(&calls);
        runtime
            .register_property_validator(
                &registry,
                property,
                Arc::new(move |candidate| {
                    validator_calls.fetch_add(1, Ordering::SeqCst);
                    match candidate {
                        TypedValue::I64(value) if *value >= 0 => ValidatorResult::Accepted,
                        _ => ValidatorResult::Rejected(None),
                    }
                }),
            )
            .expect("register synchronous Validator");
        assert_eq!(
            runtime.register_property_validator(
                &registry,
                property,
                Arc::new(|_| ValidatorResult::Accepted),
            ),
            Err(RuntimePropertyError::ValidatorAlreadyRegistered)
        );
        assert!(matches!(
            runtime.apply_property_candidate(&registry, property, TypedValue::Bool(true)),
            Err(RuntimePropertyError::ValueKindMismatch(_))
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);

        assert_eq!(
            runtime.apply_property_candidate(&registry, property, TypedValue::I64(4)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, property, TypedValue::I64(4)),
            Ok(ValueCommit::Unchanged)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, property, TypedValue::I64(-1)),
            Err(RuntimePropertyError::ValidationFailed(None))
        );
        assert!(matches!(
            runtime.read_property(&registry, property),
            Ok(TypedValue::I64(4))
        ));
        let identity = registry
            .validate(property, &PropertyHandleKind::Property, &runtime.identity())
            .expect("Property stays registered")
            .identity();
        assert_eq!(
            runtime.take_property_change_generation(),
            Ok(vec![identity])
        );
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn runtime_binding_validation_rejection_preserves_group_stops_downstream_and_recovers() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let source = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register source");
        let middle = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register middle");
        let downstream = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register downstream");
        let independent = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register independent target");

        let (_, source_middle) = runtime
            .create_one_way_binding_initial_with_id(&registry, source, middle)
            .expect("create first edge");
        let (_, middle_downstream) = runtime
            .create_one_way_binding_initial_with_id(&registry, middle, downstream)
            .expect("create dependent edge");
        runtime
            .create_one_way_binding_initial_with_id(&registry, source, independent)
            .expect("create independent edge");

        let rejection = Arc::new(ValidationError {
            status: 11,
            domain: 5,
            code: 9,
            message: b"ten is temporarily rejected".to_vec(),
        });
        let rejection_for_validator = Arc::clone(&rejection);
        let middle_identity = registry
            .validate(middle, &PropertyHandleKind::Property, &runtime.identity())
            .expect("middle handle is valid")
            .identity();
        runtime
            .register_property_validator(
                &registry,
                middle,
                Arc::new(move |candidate| match candidate {
                    TypedValue::I64(10) => {
                        ValidatorResult::Rejected(Some(Arc::clone(&rejection_for_validator)))
                    }
                    _ => ValidatorResult::Accepted,
                }),
            )
            .expect("register middle validator after initial Binding creation");
        runtime
            .take_property_change_generation()
            .expect("discard initial Binding notifications");

        assert_eq!(
            runtime.apply_property_candidate(&registry, source, TypedValue::I64(10)),
            Ok(ValueCommit::Changed),
            "the direct source setter remains successful"
        );
        assert_eq!(
            runtime.process_property_delivery_slice(),
            Ok((2, false)),
            "source and independent-target notifications run; rejected targets do not"
        );
        assert!(matches!(
            runtime.read_property(&registry, source),
            Ok(TypedValue::I64(10))
        ));
        assert!(matches!(
            runtime.read_property(&registry, middle),
            Ok(TypedValue::I64(0))
        ));
        assert!(
            matches!(
                runtime.read_property(&registry, downstream),
                Ok(TypedValue::I64(0))
            ),
            "an edge downstream of the rejected target sees its committed value"
        );
        assert!(
            matches!(
                runtime.read_property(&registry, independent),
                Ok(TypedValue::I64(10))
            ),
            "an independent edge continues"
        );
        let current = runtime
            .binding_validation_snapshot(source_middle)
            .expect("read edge error")
            .expect("rejected edge stores an error");
        assert_eq!(current.rejected_property, middle_identity);
        assert_eq!(current.direction, 1);
        assert!(Arc::ptr_eq(&current.error, &rejection));
        assert_eq!(
            runtime.binding_validation_snapshot(middle_downstream),
            Ok(None),
            "the blocked dependent edge itself did not reject a candidate"
        );

        assert_eq!(
            runtime.apply_property_candidate(&registry, source, TypedValue::I64(20)),
            Ok(ValueCommit::Changed)
        );
        runtime
            .process_property_delivery_slice()
            .expect("process successful recovery propagation");
        assert!(matches!(
            runtime.read_property(&registry, middle),
            Ok(TypedValue::I64(20))
        ));
        assert!(matches!(
            runtime.read_property(&registry, downstream),
            Ok(TypedValue::I64(20))
        ));
        assert_eq!(runtime.binding_validation_snapshot(source_middle), Ok(None));
        assert_eq!(current.error.message, b"ten is temporarily rejected");
        assert_eq!(rejection.message, b"ten is temporarily rejected");
    }

    #[test]
    fn runtime_binding_validation_rejection_keeps_two_way_target_group_atomic() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let source = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register source");
        let first = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register first group member");
        let rejecting = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register rejecting group member");
        runtime
            .create_two_way_binding_initial(&registry, first, rejecting)
            .expect("create target group");
        let (_, connection_id) = runtime
            .create_one_way_binding_initial_with_id(&registry, source, first)
            .expect("create incoming edge");
        let rejecting_identity = registry
            .validate(
                rejecting,
                &PropertyHandleKind::Property,
                &runtime.identity(),
            )
            .expect("rejecting target handle is valid")
            .identity();
        let error = Arc::new(ValidationError {
            status: 11,
            domain: 5,
            code: 10,
            message: b"group member rejected".to_vec(),
        });
        let error_for_validator = Arc::clone(&error);
        runtime
            .register_property_validator(
                &registry,
                rejecting,
                Arc::new(move |_| {
                    ValidatorResult::Rejected(Some(Arc::clone(&error_for_validator)))
                }),
            )
            .expect("register group-member validator");
        runtime
            .take_property_change_generation()
            .expect("discard initial Binding notifications");

        assert_eq!(
            runtime.apply_property_candidate(&registry, source, TypedValue::I64(1)),
            Ok(ValueCommit::Changed)
        );
        runtime
            .process_property_delivery_slice()
            .expect("process rejected group propagation");
        assert!(matches!(
            runtime.read_property(&registry, first),
            Ok(TypedValue::I64(0))
        ));
        assert!(matches!(
            runtime.read_property(&registry, rejecting),
            Ok(TypedValue::I64(0))
        ));
        assert_eq!(
            runtime.take_property_change_generation(),
            Ok(Vec::new()),
            "rejected group members queue no value notifications"
        );
        let snapshot = runtime
            .binding_validation_snapshot(connection_id)
            .expect("read incoming edge error")
            .expect("incoming edge retains rejection");
        assert_eq!(snapshot.rejected_property, rejecting_identity);
        assert!(Arc::ptr_eq(&snapshot.error, &error));
    }

    #[test]
    fn runtime_property_validator_rejects_reentrant_setter_without_blocking_outer_commit() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let registry = Arc::new(HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new());
        let property = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register Property");
        let runtime_ref = Arc::new(Mutex::new(Some(Arc::downgrade(&runtime))));
        let nested_result = Arc::new(Mutex::new(None));
        let nested_binding_result = Arc::new(Mutex::new(None));
        let callback_runtime = Arc::clone(&runtime_ref);
        let callback_registry = Arc::clone(&registry);
        let callback_result = Arc::clone(&nested_result);
        let callback_binding_result = Arc::clone(&nested_binding_result);
        runtime
            .register_property_validator(
                &registry,
                property,
                Arc::new(move |_| {
                    let runtime = callback_runtime
                        .lock()
                        .expect("Runtime weak reference lock")
                        .as_ref()
                        .and_then(|runtime| runtime.upgrade())
                        .expect("Runtime is alive during Validator callback");
                    let result = runtime.apply_property_candidate(
                        &callback_registry,
                        property,
                        TypedValue::I64(99),
                    );
                    *callback_result.lock().expect("nested result lock") = Some(result);
                    let binding_result = runtime.create_one_way_binding_initial(
                        &callback_registry,
                        property,
                        property,
                    );
                    *callback_binding_result
                        .lock()
                        .expect("nested binding result lock") = Some(binding_result);
                    ValidatorResult::Accepted
                }),
            )
            .expect("register synchronous Validator");

        assert_eq!(
            runtime.apply_property_candidate(&registry, property, TypedValue::I64(7)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            *nested_result.lock().expect("nested result lock"),
            Some(Err(RuntimePropertyError::ValidatorReentrancy))
        );
        assert_eq!(
            *nested_binding_result
                .lock()
                .expect("nested binding result lock"),
            Some(Err(RuntimeBindingError::Property(
                RuntimePropertyError::ValidatorReentrancy
            )))
        );
        assert!(matches!(
            runtime.read_property(&registry, property),
            Ok(TypedValue::I64(7))
        ));
    }

    #[test]
    fn runtime_two_way_binding_initial_rejections_preserve_values_and_graph() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let source = runtime
            .register_property(&registry, TypedValue::I64(42))
            .expect("register source Property");
        let target = runtime
            .register_property(&registry, TypedValue::Bool(false))
            .expect("register mismatched target Property");
        let source_identity = registry
            .validate(source, &PropertyHandleKind::Property, &runtime.identity())
            .expect("source remains valid")
            .identity();
        let target_identity = registry
            .validate(target, &PropertyHandleKind::Property, &runtime.identity())
            .expect("target remains valid")
            .identity();

        assert!(matches!(
            runtime.create_two_way_binding_initial(&registry, source, target),
            Err(RuntimeBindingError::InitialCopy(
                BindingCopyError::KindMismatch(_)
            ))
        ));
        assert!(
            !runtime
                .binding_graph
                .lock()
                .expect("graph lock")
                .same_group(source_identity, target_identity)
        );
        assert!(matches!(
            runtime.read_property(&registry, target),
            Ok(TypedValue::Bool(false))
        ));
        assert_eq!(runtime.take_property_change_generation(), Ok(Vec::new()));

        let cycle_source = runtime
            .register_property(&registry, TypedValue::I64(1))
            .expect("register cycle source");
        let cycle_target = runtime
            .register_property(&registry, TypedValue::I64(2))
            .expect("register cycle target");
        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, cycle_source, cycle_target),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime
                .take_property_change_generation()
                .map(|items| items.len()),
            Ok(1)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, cycle_target, TypedValue::I64(2)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime
                .take_property_change_generation()
                .map(|items| items.len()),
            Ok(1)
        );
        assert_eq!(
            runtime.create_two_way_binding_initial(&registry, cycle_source, cycle_target),
            Err(RuntimeBindingError::Graph(
                crate::binding::BindingGraphError::SelfConnection
            ))
        );
        assert!(matches!(
            runtime.read_property(&registry, cycle_target),
            Ok(TypedValue::I64(2))
        ));
        assert_eq!(runtime.take_property_change_generation(), Ok(Vec::new()));
    }

    #[test]
    fn runtime_one_way_binding_initial_sync_commits_target_edge_and_notification() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let registry = Arc::new(HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new());
        let source = runtime
            .register_property(&registry, TypedValue::I64(42))
            .expect("register source Property");
        let target = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register target Property");

        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, source, target),
            Ok(ValueCommit::Changed)
        );
        assert!(matches!(
            runtime.read_property(&registry, target),
            Ok(TypedValue::I64(42))
        ));
        let target_identity = registry
            .validate(target, &PropertyHandleKind::Property, &runtime.identity())
            .expect("target remains valid")
            .identity();
        assert_eq!(
            runtime.take_property_change_generation(),
            Ok(vec![target_identity])
        );
        assert!(matches!(
            runtime.create_one_way_binding_initial(&registry, source, target),
            Err(RuntimeBindingError::Graph(
                crate::binding::BindingGraphError::DuplicateEdge
            ))
        ));
    }

    #[test]
    fn runtime_one_way_binding_initial_sync_same_value_commits_edge_without_notification() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let source = runtime
            .register_property(&registry, TypedValue::I64(42))
            .expect("register source Property");
        let target = runtime
            .register_property(&registry, TypedValue::I64(42))
            .expect("register target Property");

        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, source, target),
            Ok(ValueCommit::Unchanged)
        );
        assert_eq!(runtime.take_property_change_generation(), Ok(Vec::new()));
        assert!(matches!(
            runtime.create_one_way_binding_initial(&registry, source, target),
            Err(RuntimeBindingError::Graph(
                crate::binding::BindingGraphError::DuplicateEdge
            ))
        ));
    }

    #[test]
    fn runtime_one_way_binding_initial_sync_rejections_preserve_values_graph_and_queue() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let foreign = RuntimeCore::new().expect("second Runtime identity is available");
        let registry = Arc::new(HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new());
        let source = runtime
            .register_property(&registry, TypedValue::I64(20))
            .expect("register source Property");
        let target = runtime
            .register_property(&registry, TypedValue::I64(10))
            .expect("register target Property");
        let bool_target = runtime
            .register_property(&registry, TypedValue::Bool(false))
            .expect("register mismatched target Property");
        let bool_source = runtime
            .register_property(&registry, TypedValue::Bool(true))
            .expect("register matching bool source Property");
        let other_source = runtime
            .register_property(&registry, TypedValue::I64(30))
            .expect("register second source Property");
        let foreign_source = foreign
            .register_property(&registry, TypedValue::I64(40))
            .expect("register foreign source Property");

        assert!(matches!(
            runtime.create_one_way_binding_initial(&registry, source, bool_target),
            Err(RuntimeBindingError::InitialCopy(
                BindingCopyError::KindMismatch(_)
            ))
        ));
        assert!(matches!(
            runtime.create_one_way_binding_initial(&registry, foreign_source, target),
            Err(RuntimeBindingError::Registry(RegistryError::WrongRuntime))
        ));
        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, bool_source, bool_target),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime
                .take_property_change_generation()
                .map(|items| items.len()),
            Ok(1)
        );
        assert!(matches!(
            runtime.create_one_way_binding_initial(&registry, source, target),
            Ok(ValueCommit::Changed)
        ));
        assert_eq!(
            runtime
                .take_property_change_generation()
                .map(|items| items.len()),
            Ok(1)
        );

        assert!(matches!(
            runtime.create_one_way_binding_initial(&registry, other_source, target),
            Err(RuntimeBindingError::Graph(
                crate::binding::BindingGraphError::TargetAlreadyHasInput
            ))
        ));
        assert!(matches!(
            runtime.read_property(&registry, target),
            Ok(TypedValue::I64(20))
        ));
        assert_eq!(runtime.take_property_change_generation(), Ok(Vec::new()));

        let worker_runtime = Arc::clone(&runtime);
        let worker_registry = Arc::clone(&registry);
        assert!(matches!(
            thread::spawn(move || worker_runtime.create_one_way_binding_initial(
                &worker_registry,
                other_source,
                bool_target,
            ))
            .join()
            .expect("worker request completes"),
            Err(RuntimeBindingError::Property(
                RuntimePropertyError::WrongThread
            ))
        ));
        assert!(matches!(
            runtime.read_property(&registry, bool_target),
            Ok(TypedValue::Bool(true))
        ));
        assert_eq!(runtime.take_property_change_generation(), Ok(Vec::new()));
    }

    #[test]
    fn runtime_binding_lifecycle_retains_endpoints_until_explicit_cancel() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let source = runtime
            .register_property(&registry, TypedValue::I64(10))
            .expect("register source Property");
        let target = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register target Property");
        let source_identity = registry
            .validate(source, &PropertyHandleKind::Property, &runtime.identity())
            .expect("source remains valid")
            .identity();
        let target_identity = registry
            .validate(target, &PropertyHandleKind::Property, &runtime.identity())
            .expect("target remains valid")
            .identity();

        runtime
            .create_one_way_binding_initial(&registry, source, target)
            .expect("active OneWay Binding is created");
        let connection_id = runtime
            .binding_graph
            .lock()
            .expect("graph lock")
            .connection_id(source_identity, target_identity, false)
            .expect("active edge has an identity");
        let error = Arc::new(ValidationError {
            status: 11,
            domain: 4,
            code: 7,
            message: b"rejected".to_vec(),
        });
        let snapshot = BindingValidationSnapshot {
            error: Arc::clone(&error),
            rejected_property: target_identity,
            direction: 1,
        };
        runtime
            .set_binding_validation_snapshot(connection_id, Some(snapshot.clone()))
            .expect("active Binding stores its current validation error");
        let read = runtime
            .binding_validation_snapshot(connection_id)
            .expect("active Binding returns its current error")
            .expect("error snapshot is present");
        assert_eq!(read, snapshot);
        assert!(Arc::ptr_eq(&read.error, &error));
        runtime
            .set_binding_validation_snapshot(connection_id, None)
            .expect("successful propagation clears the current error");
        assert_eq!(runtime.binding_validation_snapshot(connection_id), Ok(None));
        registry
            .release(source)
            .expect("release source owner token");
        registry
            .release(target)
            .expect("release target owner token");

        assert_eq!(runtime.active_bindings.lock().expect("owner lock").len(), 1);
        assert!(
            runtime
                .ui_objects
                .lock()
                .expect("store lock")
                .resolve(source_identity)
                .is_ok()
        );
        assert!(
            runtime
                .ui_objects
                .lock()
                .expect("store lock")
                .resolve(target_identity)
                .is_ok()
        );

        assert_eq!(runtime.cancel_binding_connection(connection_id), Ok(true));
        assert_eq!(runtime.cancel_binding_connection(connection_id), Ok(false));
        assert!(
            runtime
                .active_bindings
                .lock()
                .expect("owner lock")
                .is_empty()
        );
        runtime
            .cleanup
            .drain_pending()
            .expect("drain released endpoints");
        assert!(
            runtime
                .ui_objects
                .lock()
                .expect("store lock")
                .resolve(source_identity)
                .is_err()
        );
    }

    #[test]
    fn runtime_binding_cancel_suppresses_pending_and_future_one_way_propagation() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let source = runtime
            .register_property(&registry, TypedValue::I64(10))
            .expect("register source Property");
        let target = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register target Property");
        runtime
            .create_one_way_binding_initial(&registry, source, target)
            .expect("active OneWay Binding is created");
        let source_identity = registry
            .validate(source, &PropertyHandleKind::Property, &runtime.identity())
            .expect("source remains valid")
            .identity();
        let target_identity = registry
            .validate(target, &PropertyHandleKind::Property, &runtime.identity())
            .expect("target remains valid")
            .identity();
        let connection_id = runtime
            .binding_graph
            .lock()
            .expect("graph lock")
            .connection_id(source_identity, target_identity, false)
            .expect("active edge has an identity");
        runtime
            .take_property_change_generation()
            .expect("discard initial target notice");

        assert_eq!(
            runtime.apply_property_candidate(&registry, source, TypedValue::I64(20)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(runtime.cancel_binding_connection(connection_id), Ok(true));
        runtime
            .prepare_property_delivery_generation()
            .expect("pending source notice prepares without the canceled edge");
        assert!(matches!(
            runtime.read_property(&registry, target),
            Ok(TypedValue::I64(10))
        ));

        assert_eq!(
            runtime.apply_property_candidate(&registry, source, TypedValue::I64(30)),
            Ok(ValueCommit::Changed)
        );
        runtime
            .prepare_property_delivery_generation()
            .expect("future source notice prepares without the canceled edge");
        assert!(matches!(
            runtime.read_property(&registry, target),
            Ok(TypedValue::I64(10))
        ));
    }

    #[test]
    fn runtime_two_way_cancel_splits_group_and_shutdown_clears_active_owners() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let source = runtime
            .register_property(&registry, TypedValue::I64(10))
            .expect("register source Property");
        let target = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register target Property");
        let source_identity = registry
            .validate(source, &PropertyHandleKind::Property, &runtime.identity())
            .expect("source remains valid")
            .identity();
        let target_identity = registry
            .validate(target, &PropertyHandleKind::Property, &runtime.identity())
            .expect("target remains valid")
            .identity();
        runtime
            .create_two_way_binding_initial(&registry, source, target)
            .expect("active TwoWay Binding is created");
        let connection_id = runtime
            .binding_graph
            .lock()
            .expect("graph lock")
            .connection_id(source_identity, target_identity, true)
            .expect("active merge has an identity");

        assert_eq!(runtime.cancel_binding_connection(connection_id), Ok(true));
        assert!(
            !runtime
                .binding_graph
                .lock()
                .expect("graph lock")
                .same_group(source_identity, target_identity)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, source, TypedValue::I64(20)),
            Ok(ValueCommit::Changed)
        );
        runtime
            .prepare_property_delivery_generation()
            .expect("source notification is prepared");
        assert!(matches!(
            runtime.read_property(&registry, target),
            Ok(TypedValue::I64(10))
        ));

        runtime
            .create_one_way_binding_initial(&registry, source, target)
            .expect("a new connection can use the released topology");
        assert_eq!(runtime.active_bindings.lock().expect("owner lock").len(), 1);
        runtime
            .cleanup
            .drain_shutdown()
            .expect("Runtime shutdown cleanup");
        assert!(
            runtime
                .active_bindings
                .lock()
                .expect("owner lock")
                .is_empty()
        );
        assert!(
            runtime
                .binding_graph
                .lock()
                .expect("graph lock")
                .connection_id(source_identity, target_identity, false)
                .is_none()
        );
    }

    #[test]
    fn runtime_two_way_binding_initial_updates_the_entire_existing_target_group() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let source = runtime
            .register_property(&registry, TypedValue::I64(5))
            .expect("register source");
        let first = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register first target-group member");
        let second = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register second target-group member");

        assert_eq!(
            runtime.create_two_way_binding_initial(&registry, first, second),
            Ok(ValueCommit::Unchanged)
        );
        runtime
            .take_property_change_generation()
            .expect("drain first group setup");
        assert_eq!(
            runtime.create_two_way_binding_initial(&registry, source, second),
            Ok(ValueCommit::Changed)
        );
        for token in [first, second] {
            assert!(matches!(
                runtime.read_property(&registry, token),
                Ok(TypedValue::I64(5))
            ));
        }
        assert_eq!(
            runtime
                .take_property_change_generation()
                .map(|items| items.len()),
            Ok(2)
        );
    }

    #[test]
    fn runtime_one_way_propagation_prepares_chain_candidates_without_mutating_state() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let source = runtime
            .register_property(&registry, TypedValue::I64(10))
            .expect("register source");
        let middle = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register middle");
        let target = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register target");
        let branch = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register branch");
        let unrelated_source = runtime
            .register_property(&registry, TypedValue::I64(99))
            .expect("register unrelated source");
        let unrelated_target = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register unrelated target");

        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, source, middle),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, middle, target),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, source, branch),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, unrelated_source, unrelated_target),
            Ok(ValueCommit::Changed)
        );
        runtime
            .take_property_change_generation()
            .expect("drain initial-sync notifications");

        assert_eq!(
            runtime.apply_property_candidate(&registry, source, TypedValue::I64(5)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, middle, TypedValue::I64(7)),
            Ok(ValueCommit::Changed)
        );
        let identity = |token| {
            registry
                .validate(token, &PropertyHandleKind::Property, &runtime.identity())
                .expect("Property token resolves")
                .identity()
        };
        let source_identity = identity(source);
        let middle_identity = identity(middle);
        let target_identity = identity(target);
        let branch_identity = identity(branch);

        let prepared = runtime
            .prepare_one_way_propagation_candidates(&[source_identity, middle_identity])
            .expect("OneWay targets should prepare in propagation order");
        assert_eq!(prepared.len(), 3);
        assert_eq!(prepared[0].target, middle_identity);
        assert!(matches!(prepared[0].value, TypedValue::I64(5)));
        assert_eq!(prepared[1].target, target_identity);
        assert!(matches!(prepared[1].value, TypedValue::I64(5)));
        assert_eq!(prepared[2].target, branch_identity);
        assert!(matches!(prepared[2].value, TypedValue::I64(5)));

        assert!(matches!(
            runtime.read_property(&registry, source),
            Ok(TypedValue::I64(5))
        ));
        assert!(matches!(
            runtime.read_property(&registry, middle),
            Ok(TypedValue::I64(7))
        ));
        assert!(matches!(
            runtime.read_property(&registry, target),
            Ok(TypedValue::I64(10))
        ));
        assert!(matches!(
            runtime.read_property(&registry, branch),
            Ok(TypedValue::I64(10))
        ));
        assert!(matches!(
            runtime.read_property(&registry, unrelated_target),
            Ok(TypedValue::I64(99))
        ));
        assert_eq!(
            runtime.take_property_change_generation(),
            Ok(vec![source_identity, middle_identity])
        );
        assert!(matches!(
            runtime.create_one_way_binding_initial(&registry, source, middle),
            Err(RuntimeBindingError::Graph(
                crate::binding::BindingGraphError::DuplicateEdge
            ))
        ));
    }

    #[test]
    fn runtime_one_way_propagation_commits_candidates_and_queues_next_generation_once() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let source = runtime
            .register_property(&registry, TypedValue::I64(10))
            .expect("register source");
        let middle = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register middle");
        let target = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register target");
        let branch = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register branch");

        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, source, middle),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, middle, target),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, source, branch),
            Ok(ValueCommit::Changed)
        );
        runtime
            .take_property_change_generation()
            .expect("drain initial-sync notifications");
        assert_eq!(
            runtime.apply_property_candidate(&registry, source, TypedValue::I64(5)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, middle, TypedValue::I64(7)),
            Ok(ValueCommit::Changed)
        );

        let identity = |token| {
            registry
                .validate(token, &PropertyHandleKind::Property, &runtime.identity())
                .expect("Property token resolves")
                .identity()
        };
        let source_identity = identity(source);
        let middle_identity = identity(middle);
        let target_identity = identity(target);
        let branch_identity = identity(branch);
        assert_eq!(
            runtime.propagate_one_way_changes(&[source_identity, middle_identity]),
            Ok(vec![middle_identity, target_identity, branch_identity])
        );
        for token in [middle, target, branch] {
            assert!(matches!(
                runtime.read_property(&registry, token),
                Ok(TypedValue::I64(5))
            ));
        }
        assert_eq!(
            runtime.take_property_change_generation(),
            Ok(vec![
                source_identity,
                middle_identity,
                target_identity,
                branch_identity
            ])
        );

        assert_eq!(
            runtime.propagate_one_way_changes(&[source_identity, middle_identity]),
            Ok(Vec::new())
        );
        assert_eq!(runtime.take_property_change_generation(), Ok(Vec::new()));
        assert!(matches!(
            runtime.create_one_way_binding_initial(&registry, source, middle),
            Err(RuntimeBindingError::Graph(
                crate::binding::BindingGraphError::DuplicateEdge
            ))
        ));
    }

    #[test]
    fn runtime_one_way_delivery_propagates_before_subscribers_and_uses_next_generation() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let registry = Arc::new(HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new());
        let source = runtime
            .register_property(&registry, TypedValue::I64(10))
            .expect("register source");
        let middle = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register middle");
        let target = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register target");
        let branch = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register branch");
        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, source, middle),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, middle, target),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, source, branch),
            Ok(ValueCommit::Changed)
        );
        runtime
            .take_property_change_generation()
            .expect("drain initial-sync notifications");

        let identity = |token| {
            registry
                .validate(token, &PropertyHandleKind::Property, &runtime.identity())
                .expect("Property token resolves")
                .identity()
        };
        let source_identity = identity(source);
        let middle_identity = identity(middle);
        let target_identity = identity(target);
        let branch_identity = identity(branch);
        let observations = Arc::new(Mutex::new(Vec::new()));
        let source_subscription =
            new_binding_delivery_subscription(&runtime, &registry, middle, 0, &observations);
        runtime
            .register_subscription_owner(source_identity, &source_subscription)
            .expect("register source subscriber");
        let middle_subscription =
            new_binding_delivery_subscription(&runtime, &registry, target, 1, &observations);
        runtime
            .register_subscription_owner(middle_identity, &middle_subscription)
            .expect("register directly changed middle subscriber");
        let target_subscription =
            new_binding_delivery_subscription(&runtime, &registry, target, 2, &observations);
        runtime
            .register_subscription_owner(target_identity, &target_subscription)
            .expect("register target subscriber");
        let branch_subscription =
            new_binding_delivery_subscription(&runtime, &registry, branch, 3, &observations);
        runtime
            .register_subscription_owner(branch_identity, &branch_subscription)
            .expect("register branch subscriber");

        assert_eq!(
            runtime.apply_property_candidate(&registry, source, TypedValue::I64(5)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, middle, TypedValue::I64(7)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(runtime.process_property_delivery_slice(), Ok((4, false)));
        assert_eq!(
            *observations
                .lock()
                .expect("read ordered subscriber observations"),
            vec![(0, 5), (1, 5), (2, 5), (3, 5)]
        );
        for token in [middle, target, branch] {
            assert!(matches!(
                runtime.read_property(&registry, token),
                Ok(TypedValue::I64(5))
            ));
        }

        assert_eq!(
            runtime.cancel_subscription_owner(source_identity, &source_subscription),
            Ok(true)
        );
        assert_eq!(
            runtime.cancel_subscription_owner(middle_identity, &middle_subscription),
            Ok(true)
        );
        assert_eq!(
            runtime.cancel_subscription_owner(target_identity, &target_subscription),
            Ok(true)
        );
        assert_eq!(
            runtime.cancel_subscription_owner(branch_identity, &branch_subscription),
            Ok(true)
        );
    }

    #[test]
    fn runtime_two_way_group_propagation_uses_latest_successful_direct_setter() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let registry = Arc::new(HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new());
        let first = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register first group member");
        let second = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register second group member");
        let third = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register third group member");
        assert_eq!(
            runtime.create_two_way_binding_initial(&registry, first, second),
            Ok(ValueCommit::Unchanged)
        );
        assert_eq!(
            runtime.create_two_way_binding_initial(&registry, second, third),
            Ok(ValueCommit::Unchanged)
        );
        runtime
            .take_property_change_generation()
            .expect("drain initial group notifications");

        let identity = |token| {
            registry
                .validate(token, &PropertyHandleKind::Property, &runtime.identity())
                .expect("Property token resolves")
                .identity()
        };
        let observations = Arc::new(Mutex::new(Vec::new()));
        let subscriptions = [first, second, third]
            .into_iter()
            .enumerate()
            .map(|(item, token)| {
                let subscription = new_binding_delivery_subscription(
                    &runtime,
                    &registry,
                    token,
                    item,
                    &observations,
                );
                runtime
                    .register_subscription_owner(identity(token), &subscription)
                    .expect("register group subscriber");
                subscription
            })
            .collect::<Vec<_>>();

        assert_eq!(
            runtime.apply_property_candidate(&registry, first, TypedValue::I64(10)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, second, TypedValue::I64(20)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, first, TypedValue::I64(30)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, second, TypedValue::Bool(false)),
            Err(RuntimePropertyError::ValueKindMismatch(ValueKindMismatch {
                expected: ValueKind::I64,
                actual: ValueKind::Bool,
            }))
        );
        assert_eq!(runtime.process_property_delivery_slice(), Ok((3, false)));
        assert_eq!(
            *observations
                .lock()
                .expect("read group subscriber observations"),
            vec![(0, 30), (1, 30), (2, 30)]
        );
        for token in [first, second, third] {
            assert!(matches!(
                runtime.read_property(&registry, token),
                Ok(TypedValue::I64(30))
            ));
        }

        assert_eq!(
            runtime.apply_property_candidate(&registry, second, TypedValue::I64(40)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, third, TypedValue::I64(50)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, second, TypedValue::I64(60)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(runtime.process_property_delivery_slice(), Ok((3, false)));
        assert_eq!(
            *observations
                .lock()
                .expect("read second group delivery observations"),
            vec![(0, 30), (1, 30), (2, 30), (1, 60), (2, 60), (0, 60)]
        );
        for token in [first, second, third] {
            assert!(matches!(
                runtime.read_property(&registry, token),
                Ok(TypedValue::I64(60))
            ));
        }

        for (token, subscription) in [first, second, third].into_iter().zip(&subscriptions) {
            assert_eq!(
                runtime.cancel_subscription_owner(identity(token), subscription),
                Ok(true)
            );
        }
    }

    #[test]
    fn runtime_one_way_edges_propagate_into_and_out_of_two_way_groups() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let registry = Arc::new(HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new());
        let source = runtime
            .register_property(&registry, TypedValue::I64(5))
            .expect("register OneWay source");
        let first = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register first group member");
        let second = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register second group member");
        let target = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register OneWay target");
        assert_eq!(
            runtime.create_two_way_binding_initial(&registry, first, second),
            Ok(ValueCommit::Unchanged)
        );
        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, source, first),
            Ok(ValueCommit::Changed)
        );
        assert!(matches!(
            runtime.read_property(&registry, first),
            Ok(TypedValue::I64(5))
        ));
        assert!(matches!(
            runtime.read_property(&registry, second),
            Ok(TypedValue::I64(5))
        ));
        assert_eq!(
            runtime.create_one_way_binding_initial(&registry, second, target),
            Ok(ValueCommit::Changed)
        );
        runtime
            .take_property_change_generation()
            .expect("drain initial Binding notifications");

        let identity = |token| {
            registry
                .validate(token, &PropertyHandleKind::Property, &runtime.identity())
                .expect("Property token resolves")
                .identity()
        };
        let observations = Arc::new(Mutex::new(Vec::new()));
        let subscriptions = [source, first, second, target]
            .into_iter()
            .enumerate()
            .map(|(item, token)| {
                let subscription = new_binding_delivery_subscription(
                    &runtime,
                    &registry,
                    token,
                    item,
                    &observations,
                );
                runtime
                    .register_subscription_owner(identity(token), &subscription)
                    .expect("register group-chain subscriber");
                subscription
            })
            .collect::<Vec<_>>();

        assert_eq!(
            runtime.apply_property_candidate(&registry, source, TypedValue::I64(15)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, first, TypedValue::I64(20)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, second, TypedValue::I64(21)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(runtime.process_property_delivery_slice(), Ok((4, false)));
        assert_eq!(
            *observations
                .lock()
                .expect("read group-chain subscriber observations"),
            vec![(0, 15), (1, 15), (2, 15), (3, 15)]
        );
        for token in [first, second, target] {
            assert!(matches!(
                runtime.read_property(&registry, token),
                Ok(TypedValue::I64(15))
            ));
        }

        assert_eq!(
            runtime.apply_property_candidate(&registry, second, TypedValue::I64(30)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, first, TypedValue::I64(35)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(runtime.process_property_delivery_slice(), Ok((3, false)));
        assert_eq!(
            *observations
                .lock()
                .expect("read downstream group observations"),
            vec![
                (0, 15),
                (1, 15),
                (2, 15),
                (3, 15),
                (2, 35),
                (1, 35),
                (3, 35),
            ]
        );
        for token in [source, first, second, target] {
            let expected = if token == source { 15 } else { 35 };
            assert!(matches!(
                runtime.read_property(&registry, token),
                Ok(TypedValue::I64(value)) if value == expected
            ));
        }

        for (token, subscription) in [source, first, second, target]
            .into_iter()
            .zip(&subscriptions)
        {
            assert_eq!(
                runtime.cancel_subscription_owner(identity(token), subscription),
                Ok(true)
            );
        }
    }

    #[test]
    fn property_state_registration_read_and_update() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let registry =
            Arc::new(HandleRegistry::<PropertyHandleKind, super::RuntimeIdentity>::new());
        let token = runtime
            .register_property(&registry, TypedValue::I64(7))
            .expect("register Property");

        assert!(matches!(
            runtime.read_property(&registry, token),
            Ok(TypedValue::I64(7))
        ));
        assert_eq!(
            runtime.apply_property_candidate(&registry, token, TypedValue::I64(9)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, token, TypedValue::I64(9)),
            Ok(ValueCommit::Unchanged)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, token, TypedValue::Bool(true)),
            Err(RuntimePropertyError::ValueKindMismatch(ValueKindMismatch {
                expected: ValueKind::I64,
                actual: ValueKind::Bool,
            }))
        );
        assert!(matches!(
            runtime.read_property(&registry, token),
            Ok(TypedValue::I64(9))
        ));

        let wrong_kind = runtime
            .register_ui_object(&registry, PropertyHandleKind::Other, || ())
            .expect("register other handle kind");
        assert!(matches!(
            runtime.read_property(&registry, wrong_kind),
            Err(RuntimePropertyError::Registry(RegistryError::WrongKind))
        ));
        assert!(matches!(
            runtime.read_property(
                &registry,
                crate::handle_registry::HandleToken::from_abi(u64::MAX)
                    .expect("nonzero test token")
            ),
            Err(RuntimePropertyError::Registry(RegistryError::InvalidToken))
        ));

        let other_runtime = RuntimeCore::new().expect("second Runtime identity is available");
        let other_token = other_runtime
            .register_property(&registry, TypedValue::I64(11))
            .expect("register Property in second Runtime");
        assert!(matches!(
            runtime.read_property(&registry, other_token),
            Err(RuntimePropertyError::Registry(RegistryError::WrongRuntime))
        ));

        let worker_runtime = Arc::clone(&runtime);
        let worker_registry = Arc::clone(&registry);
        assert!(matches!(
            std::thread::spawn(move || worker_runtime.read_property(&worker_registry, token))
                .join()
                .expect("worker access completes"),
            Err(RuntimePropertyError::WrongThread)
        ));
        let worker_runtime = Arc::clone(&runtime);
        let worker_registry = Arc::clone(&registry);
        assert!(matches!(
            std::thread::spawn(move || {
                worker_runtime.apply_property_candidate(
                    &worker_registry,
                    token,
                    TypedValue::I64(17),
                )
            })
            .join()
            .expect("worker update completes"),
            Err(RuntimePropertyError::WrongThread)
        ));
        assert!(matches!(
            runtime.read_property(&registry, token),
            Ok(TypedValue::I64(9))
        ));

        runtime
            .cleanup
            .drain_shutdown()
            .expect("Runtime shutdown clears Property state");
        assert!(matches!(
            runtime.register_property(&registry, TypedValue::I64(13)),
            Err(RuntimeObjectRegistrationError::Closed)
        ));
    }

    #[test]
    fn property_change_queue_coalesces_per_pending_generation() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let first = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register first Property");
        let second = runtime
            .register_property(&registry, TypedValue::I64(10))
            .expect("register second Property");
        let first_identity = registry
            .validate(first, &PropertyHandleKind::Property, &runtime.identity)
            .expect("first Property remains live")
            .identity();
        let second_identity = registry
            .validate(second, &PropertyHandleKind::Property, &runtime.identity)
            .expect("second Property remains live")
            .identity();

        assert_eq!(
            runtime.apply_property_candidate(&registry, first, TypedValue::I64(1)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, second, TypedValue::I64(11)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, first, TypedValue::I64(2)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime
                .take_property_change_generation()
                .expect("take first pending generation"),
            vec![first_identity, second_identity]
        );
        assert!(matches!(
            runtime.read_property(&registry, first),
            Ok(TypedValue::I64(2))
        ));

        assert_eq!(
            runtime.apply_property_candidate(&registry, second, TypedValue::I64(12)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime
                .take_property_change_generation()
                .expect("take next pending generation"),
            vec![second_identity]
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, second, TypedValue::I64(12)),
            Ok(ValueCommit::Unchanged)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, first, TypedValue::Bool(false)),
            Err(RuntimePropertyError::ValueKindMismatch(ValueKindMismatch {
                expected: ValueKind::I64,
                actual: ValueKind::Bool,
            }))
        );
        let foreign_registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let foreign_token = runtime
            .register_property(&foreign_registry, TypedValue::I64(1))
            .expect("register Property in separate registry");
        assert!(matches!(
            runtime.apply_property_candidate(&registry, foreign_token, TypedValue::I64(1),),
            Err(RuntimePropertyError::Registry(RegistryError::InvalidToken))
        ));
        assert!(
            runtime
                .take_property_change_generation()
                .expect("empty generation after unchanged and failed updates")
                .is_empty()
        );
    }

    #[test]
    fn property_delivery_item_invocation_defers_updates_and_rechecks_activity() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let registry = Arc::new(HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new());
        let token = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register Property");
        let identity = registry
            .validate(token, &PropertyHandleKind::Property, &runtime.identity)
            .expect("Property remains live")
            .identity();

        let later_probe = Arc::new(DeliveryProbe::default());
        let later_userdata = Box::into_raw(Box::new(Arc::clone(&later_probe))).cast::<c_void>();
        // SAFETY: the Subscription owns its Box until delivery_probe_destroy runs.
        let later_subscription = Arc::new(unsafe {
            Subscription::new(
                delivery_probe_callback,
                later_userdata,
                delivery_probe_destroy,
            )
        });

        let calls = Arc::new(AtomicUsize::new(0));
        let outcomes = Arc::new(AtomicUsize::new(0));
        let destroys = Arc::new(AtomicUsize::new(0));
        let callback_context = Box::new(DeliveryCallbackContext {
            runtime: Arc::clone(&runtime),
            registry: Arc::clone(&registry),
            token,
            identity,
            later_subscription: Arc::clone(&later_subscription),
            calls: Arc::clone(&calls),
            outcomes: Arc::clone(&outcomes),
            destroys: Arc::clone(&destroys),
        });
        let callback_userdata = Box::into_raw(callback_context).cast::<c_void>();
        // SAFETY: the Subscription owns its Box until delivery_callback_destroy runs.
        let first_subscription = Arc::new(unsafe {
            Subscription::new(
                delivery_callback,
                callback_userdata,
                delivery_callback_destroy,
            )
        });
        runtime
            .register_subscription_owner(identity, &first_subscription)
            .expect("register first subscriber");

        assert_eq!(
            runtime.apply_property_candidate(&registry, token, TypedValue::I64(1)),
            Ok(ValueCommit::Changed)
        );
        let first_generation = runtime
            .prepare_property_delivery_generation()
            .expect("prepare first delivery generation");
        assert_eq!(first_generation.len(), 1);
        assert_eq!(first_generation[0].subscribers.len(), 1);
        assert_eq!(first_generation[0].deliver(), 1);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(outcomes.load(Ordering::SeqCst), 3);
        assert_eq!(later_probe.callbacks.load(Ordering::SeqCst), 0);

        let second_generation = runtime
            .prepare_property_delivery_generation()
            .expect("prepare callback-triggered generation");
        assert_eq!(second_generation.len(), 1);
        assert_eq!(second_generation[0].identity, identity);
        assert_eq!(second_generation[0].subscribers.len(), 2);
        assert_eq!(second_generation[0].deliver(), 1);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(outcomes.load(Ordering::SeqCst), 7);
        assert_eq!(later_probe.callbacks.load(Ordering::SeqCst), 0);
        assert_eq!(later_probe.destroys.load(Ordering::SeqCst), 1);

        assert!(matches!(
            runtime.cancel_subscription_owner(identity, &first_subscription),
            Ok(true)
        ));
        drop(first_generation);
        drop(second_generation);
        drop(first_subscription);
        drop(later_subscription);
        assert_eq!(destroys.load(Ordering::SeqCst), 1);
        assert_eq!(later_probe.destroys.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn property_delivery_slice_limits_completed_items_and_preserves_remainder_order() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let probe = Arc::new(OrderedDeliveryProbe::default());
        let mut properties = Vec::new();
        let mut subscriptions = Vec::new();

        for item in 0..129 {
            let token = runtime
                .register_property(&registry, TypedValue::I64(0))
                .expect("register Property");
            let identity = registry
                .validate(token, &PropertyHandleKind::Property, &runtime.identity)
                .expect("Property remains live")
                .identity();
            let context = Box::new(OrderedDeliveryContext {
                probe: Arc::clone(&probe),
                item,
                runtime: (item == 0).then(|| Arc::clone(&runtime)),
                shutdown_dispatcher: None,
            });
            let userdata = Box::into_raw(context).cast::<c_void>();
            // SAFETY: the Subscription owns its Box until ordered_delivery_destroy runs.
            let subscription = Arc::new(unsafe {
                Subscription::new(
                    ordered_delivery_callback,
                    userdata,
                    ordered_delivery_destroy,
                )
            });
            runtime
                .register_subscription_owner(identity, &subscription)
                .expect("register Property subscriber");
            properties.push((token, identity));
            subscriptions.push(subscription);
        }

        for (token, _) in &properties {
            assert_eq!(
                runtime.apply_property_candidate(&registry, *token, TypedValue::I64(1)),
                Ok(ValueCommit::Changed)
            );
        }

        let worker_runtime = Arc::clone(&runtime);
        assert_eq!(
            thread::spawn(move || worker_runtime.process_property_delivery_slice())
                .join()
                .expect("worker slice attempt completes"),
            Err(RuntimePropertyError::WrongThread)
        );
        assert!(
            probe
                .callbacks
                .lock()
                .expect("read callback order before UI slice")
                .is_empty()
        );

        assert_eq!(runtime.process_property_delivery_slice(), Ok((128, true)));
        assert_eq!(
            *probe
                .callbacks
                .lock()
                .expect("read callback order after first slice"),
            (0..128).collect::<Vec<_>>()
        );
        assert_eq!(runtime.process_property_delivery_slice(), Ok((1, false)));
        assert_eq!(
            *probe
                .callbacks
                .lock()
                .expect("read callback order after final slice"),
            (0..129).collect::<Vec<_>>()
        );
        assert_eq!(runtime.process_property_delivery_slice(), Ok((0, false)));
        assert!(probe.reentrant_slice_blocked.load(Ordering::SeqCst));

        for ((_, identity), subscription) in properties.iter().zip(&subscriptions) {
            assert!(matches!(
                runtime.cancel_subscription_owner(*identity, subscription),
                Ok(true)
            ));
        }
        drop(subscriptions);
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 129);
    }

    #[test]
    fn property_notification_host_boundary_orders_delivery_and_yields() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let bridge = Arc::new(TestUiLoopBridge::default());
        let bridge_trait: Arc<dyn UiLoopBridge> = bridge.clone();
        runtime
            .set_ui_loop_bridge(&bridge_trait)
            .expect("register UI-loop bridge");
        let dispatcher = runtime
            .dispatcher()
            .expect("dispatcher lock")
            .expect("registered dispatcher");
        let registry = Arc::new(HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new());
        let token = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register Property");
        let identity = registry
            .validate(token, &PropertyHandleKind::Property, &runtime.identity)
            .expect("Property remains live")
            .identity();
        let events = Arc::new(Mutex::new(Vec::new()));
        let callback_context = Box::new(OrderedDeliveryContext {
            probe: Arc::new(OrderedDeliveryProbe::default()),
            item: 1,
            runtime: Some(Arc::clone(&runtime)),
            shutdown_dispatcher: None,
        });
        let callback_probe = Arc::clone(&callback_context.probe);
        let userdata = Box::into_raw(callback_context).cast::<c_void>();
        // SAFETY: the Subscription owns the callback context until its destroy callback runs.
        let subscription = Arc::new(unsafe {
            Subscription::new(
                ordered_delivery_callback,
                userdata,
                ordered_delivery_destroy,
            )
        });
        runtime
            .register_subscription_owner(identity, &subscription)
            .expect("register subscriber");

        let first_runtime = Arc::clone(&runtime);
        let first_registry = Arc::clone(&registry);
        let first_events = Arc::clone(&events);
        dispatcher
            .post(PostedWork::callback(Box::new(move || {
                first_events.lock().expect("event log").push("first-post");
                assert_eq!(
                    first_runtime.apply_property_candidate(
                        &first_registry,
                        token,
                        TypedValue::I64(1),
                    ),
                    Ok(ValueCommit::Changed)
                );
            })))
            .expect("post Property update");
        let second_events = Arc::clone(&events);
        dispatcher
            .post(PostedWork::callback(Box::new(move || {
                second_events.lock().expect("event log").push("second-post");
            })))
            .expect("post following callback");

        runtime.dispatch_turn().expect("dispatch first post");
        assert_eq!(
            *events.lock().expect("read event order"),
            vec!["first-post"]
        );
        assert_eq!(
            *callback_probe
                .callbacks
                .lock()
                .expect("read notification count"),
            vec![1]
        );
        assert!(callback_probe.reentrant_turn_blocked.load(Ordering::SeqCst));
        runtime.dispatch_turn().expect("dispatch second post");
        assert_eq!(
            *events.lock().expect("read event order"),
            vec!["first-post", "second-post"]
        );
        assert_eq!(bridge.wakes.load(Ordering::SeqCst), 2);
        assert_eq!(
            *bridge.wake_priorities.lock().expect("read wake priorities"),
            vec![WakePriority::Normal, WakePriority::Normal]
        );
        assert!(
            runtime
                .cancel_subscription_owner(identity, &subscription)
                .expect("cancel subscriber")
        );
    }

    #[test]
    fn property_notification_host_boundary_yields_and_resumes_in_order() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let bridge = Arc::new(TestUiLoopBridge::default());
        let bridge_trait: Arc<dyn UiLoopBridge> = bridge.clone();
        runtime
            .set_ui_loop_bridge(&bridge_trait)
            .expect("register UI-loop bridge");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let probe = Arc::new(OrderedDeliveryProbe::default());
        let mut properties = Vec::new();
        let mut subscriptions = Vec::new();
        for item in 0..129 {
            let token = runtime
                .register_property(&registry, TypedValue::I64(0))
                .expect("register Property");
            let identity = registry
                .validate(token, &PropertyHandleKind::Property, &runtime.identity)
                .expect("Property remains live")
                .identity();
            let context = Box::new(OrderedDeliveryContext {
                probe: Arc::clone(&probe),
                item,
                runtime: None,
                shutdown_dispatcher: None,
            });
            let userdata = Box::into_raw(context).cast::<c_void>();
            // SAFETY: the Subscription owns its boxed context until cancellation.
            let subscription = Arc::new(unsafe {
                Subscription::new(
                    ordered_delivery_callback,
                    userdata,
                    ordered_delivery_destroy,
                )
            });
            runtime
                .register_subscription_owner(identity, &subscription)
                .expect("register subscriber");
            assert_eq!(
                runtime.apply_property_candidate(&registry, token, TypedValue::I64(1)),
                Ok(ValueCommit::Changed)
            );
            properties.push(identity);
            subscriptions.push(subscription);
        }

        let outer_scope = runtime.begin_host_event().expect("begin outer host event");
        let nested_scope = runtime.begin_host_event().expect("begin nested host event");
        assert!(
            !runtime
                .end_host_event(nested_scope)
                .expect("close nested event")
        );
        assert!(
            probe
                .callbacks
                .lock()
                .expect("no nested delivery")
                .is_empty()
        );
        assert!(
            !runtime
                .end_host_event(outer_scope)
                .expect("close outer event")
        );
        assert_eq!(
            *probe.callbacks.lock().expect("first slice order"),
            (0..128).collect::<Vec<_>>()
        );
        assert_eq!(bridge.wakes.load(Ordering::SeqCst), 1);
        assert_eq!(
            *bridge
                .wake_priorities
                .lock()
                .expect("first slice wake priority"),
            vec![WakePriority::Normal]
        );

        runtime
            .dispatch_turn()
            .expect("resume pending notifications");
        assert_eq!(
            *probe.callbacks.lock().expect("completed slice order"),
            (0..129).collect::<Vec<_>>()
        );
        for (identity, subscription) in properties.iter().zip(&subscriptions) {
            assert!(
                runtime
                    .cancel_subscription_owner(*identity, subscription)
                    .expect("cancel subscriber")
            );
        }
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 129);
    }

    #[test]
    fn property_setter_host_scope_delivers_at_outer_boundary_and_closes_after_error() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let bridge = Arc::new(TestUiLoopBridge::default());
        let bridge_trait: Arc<dyn UiLoopBridge> = bridge;
        runtime
            .set_ui_loop_bridge(&bridge_trait)
            .expect("register UI-loop bridge");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let token = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register Property");
        let identity = registry
            .validate(token, &PropertyHandleKind::Property, &runtime.identity)
            .expect("Property remains live")
            .identity();
        let probe = Arc::new(OrderedDeliveryProbe::default());
        let userdata = Box::into_raw(Box::new(OrderedDeliveryContext {
            probe: Arc::clone(&probe),
            item: 0,
            runtime: None,
            shutdown_dispatcher: None,
        }))
        .cast::<c_void>();
        // SAFETY: the Subscription owns its boxed context until cancellation.
        let subscription = Arc::new(unsafe {
            Subscription::new(
                ordered_delivery_callback,
                userdata,
                ordered_delivery_destroy,
            )
        });
        runtime
            .register_subscription_owner(identity, &subscription)
            .expect("register Property subscriber");

        assert_eq!(
            runtime.apply_property_candidate_in_host_event(&registry, token, TypedValue::I64(1),),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            *probe.callbacks.lock().expect("read first delivery"),
            vec![0]
        );

        let outer_scope = runtime.begin_host_event().expect("begin outer scope");
        assert_eq!(
            runtime.apply_property_candidate_in_host_event(&registry, token, TypedValue::I64(2),),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            *probe.callbacks.lock().expect("nested setter is deferred"),
            vec![0]
        );
        assert_eq!(
            runtime.apply_property_candidate_in_host_event(
                &registry,
                token,
                TypedValue::Bool(true),
            ),
            Err(RuntimeCoreError::Property(
                RuntimePropertyError::ValueKindMismatch(ValueKindMismatch {
                    expected: ValueKind::I64,
                    actual: ValueKind::Bool,
                })
            ))
        );
        assert_eq!(
            *probe
                .callbacks
                .lock()
                .expect("failed nested setter also defers"),
            vec![0]
        );
        assert_eq!(runtime.end_host_event(outer_scope), Ok(false));
        assert_eq!(
            *probe.callbacks.lock().expect("outer boundary delivery"),
            vec![0, 0]
        );
        assert!(
            runtime
                .cancel_subscription_owner(identity, &subscription)
                .expect("cancel subscriber")
        );
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn property_notification_shutdown_cancels_pending_and_destroys_on_ui_thread() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let bridge = Arc::new(TestUiLoopBridge::default());
        let bridge_trait: Arc<dyn UiLoopBridge> = bridge.clone();
        runtime
            .set_ui_loop_bridge(&bridge_trait)
            .expect("register UI-loop bridge");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let token = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register Property");
        let identity = registry
            .validate(token, &PropertyHandleKind::Property, &runtime.identity)
            .expect("Property remains live")
            .identity();
        let probe = Arc::new(OrderedDeliveryProbe::default());
        let userdata = Box::into_raw(Box::new(OrderedDeliveryContext {
            probe: Arc::clone(&probe),
            item: 0,
            runtime: None,
            shutdown_dispatcher: None,
        }))
        .cast::<c_void>();
        // SAFETY: the Subscription owns its boxed context until cancellation.
        let subscription = Arc::new(unsafe {
            Subscription::new(
                ordered_delivery_callback,
                userdata,
                ordered_delivery_destroy,
            )
        });
        runtime
            .register_subscription_owner(identity, &subscription)
            .expect("register subscriber");
        assert_eq!(
            runtime.apply_property_candidate(&registry, token, TypedValue::I64(1)),
            Ok(ValueCommit::Changed)
        );

        let scope = runtime.begin_host_event().expect("begin host event");
        let (request, outcome) = runtime
            .request_shutdown(ShutdownRequestMode::Async)
            .expect("request shutdown before safe point");
        assert_eq!(outcome, ShutdownRequestOutcome::Pending);
        assert!(runtime.end_host_event(scope).expect("commit shutdown"));
        assert_eq!(request.query(), Ok(ShutdownCompletionState::Completed));
        assert!(probe.callbacks.lock().expect("read callbacks").is_empty());
        assert!(!subscription.is_active());
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 1);
        assert_eq!(
            *probe.destroy_threads.lock().expect("read destroy thread"),
            vec![thread::current().id()]
        );
        assert!(
            runtime
                .subscription_owners
                .lock()
                .expect("subscription owners are drained")
                .is_empty()
        );
        assert_eq!(
            runtime.has_pending_property_delivery(),
            Ok(false),
            "shutdown discards pending generations and prepared items"
        );
        assert_eq!(runtime.cleanup_queue().state(), RuntimeState::Closed);
        assert_eq!(bridge.destroys.load(Ordering::SeqCst), 1);
        assert_eq!(
            *bridge
                .wake_priorities
                .lock()
                .expect("shutdown wake priority"),
            vec![WakePriority::Control]
        );
    }

    #[test]
    fn property_owner_teardown_cancels_subscriptions_and_discards_delivery() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = Arc::new(HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new());
        let bridge = Arc::new(TestUiLoopBridge::default());
        let bridge_api: Arc<dyn UiLoopBridge> = bridge.clone();
        runtime
            .set_ui_loop_bridge(&bridge_api)
            .expect("register UI loop bridge");
        let token = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register Property");
        let identity = registry
            .validate(token, &PropertyHandleKind::Property, &runtime.identity)
            .expect("Property remains live")
            .identity();
        let probe = Arc::new(OrderedDeliveryProbe::default());
        let context = Box::new(OrderedDeliveryContext {
            probe: Arc::clone(&probe),
            item: 1,
            runtime: None,
            shutdown_dispatcher: None,
        });
        let userdata = Box::into_raw(context).cast::<c_void>();
        // SAFETY: the Subscription owns the callback context until its destroy function runs.
        let subscription = Arc::new(unsafe {
            Subscription::new(
                ordered_delivery_callback,
                userdata,
                ordered_delivery_destroy,
            )
        });
        runtime
            .register_subscription_owner(identity, &subscription)
            .expect("register Runtime Subscription owner");
        runtime
            .apply_property_candidate(&registry, token, TypedValue::I64(1))
            .expect("queue changed Property");
        let prepared = runtime
            .prepare_property_delivery_generation()
            .expect("prepare pending delivery");
        *runtime
            .property_delivery
            .lock()
            .expect("delivery queue lock") = Some(prepared.into());

        let worker_registry = Arc::clone(&registry);
        let outcome = thread::spawn(move || worker_registry.release(token))
            .join()
            .expect("worker release completes")
            .expect("release final Property token");
        assert!(outcome.cleanup_accepted);
        assert_eq!(
            *bridge
                .wake_priorities
                .lock()
                .expect("wake priority log lock"),
            vec![WakePriority::Normal]
        );
        runtime
            .dispatch_turn()
            .expect("safe point drains Property cleanup");

        assert!(!subscription.is_active());
        assert!(
            probe
                .callbacks
                .lock()
                .expect("callback log lock")
                .is_empty()
        );
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 1);
        assert_eq!(
            *probe
                .destroy_threads
                .lock()
                .expect("destroy thread log lock"),
            vec![thread::current().id()]
        );
        assert!(
            !runtime
                .subscription_owners
                .lock()
                .expect("owner registry lock")
                .contains_key(&identity)
        );
        assert!(
            runtime
                .property_delivery
                .lock()
                .expect("delivery queue lock")
                .is_none()
        );
        assert_eq!(
            runtime
                .property_changes
                .lock()
                .expect("change queue lock")
                .pending_len(),
            0
        );
        assert!(
            runtime
                .ui_objects
                .lock()
                .expect("UI object store lock")
                .resolve(identity)
                .is_err()
        );

        let surviving = runtime
            .register_property(&registry, TypedValue::I64(9))
            .expect("Runtime remains usable after Property teardown");
        assert!(matches!(
            runtime.read_property(&registry, surviving),
            Ok(TypedValue::I64(9))
        ));
        runtime
            .cleanup
            .drain_shutdown()
            .expect("Runtime shutdown succeeds after owner teardown");
    }

    #[test]
    fn property_owner_teardown_inside_callback_stops_later_subscribers() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let registry = Arc::new(HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new());
        let token = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register Property");
        let identity = registry
            .validate(token, &PropertyHandleKind::Property, &runtime.identity)
            .expect("Property remains live")
            .identity();
        let first_calls = Arc::new(AtomicUsize::new(0));
        let first_destroys = Arc::new(AtomicUsize::new(0));
        let context = Box::new(PropertyReleaseCallbackContext {
            registry: Arc::clone(&registry),
            token,
            callbacks: Arc::clone(&first_calls),
            destroys: Arc::clone(&first_destroys),
        });
        let userdata = Box::into_raw(context).cast::<c_void>();
        // SAFETY: the Subscription owns the callback context until its destroy function runs.
        let first = Arc::new(unsafe {
            Subscription::new(
                release_property_callback,
                userdata,
                release_property_destroy,
            )
        });
        let later_probe = Arc::new(DeliveryProbe::default());
        let later_userdata = Box::into_raw(Box::new(Arc::clone(&later_probe))).cast::<c_void>();
        // SAFETY: the Subscription owns the probe Arc until delivery_probe_destroy runs.
        let later = Arc::new(unsafe {
            Subscription::new(
                delivery_probe_callback,
                later_userdata,
                delivery_probe_destroy,
            )
        });
        runtime
            .register_subscription_owner(identity, &first)
            .expect("register first Subscription");
        runtime
            .register_subscription_owner(identity, &later)
            .expect("register later Subscription");
        runtime
            .apply_property_candidate(&registry, token, TypedValue::I64(1))
            .expect("queue changed Property");

        assert_eq!(runtime.process_property_delivery_slice(), Ok((1, false)));
        assert_eq!(first_calls.load(Ordering::SeqCst), 1);
        assert_eq!(first_destroys.load(Ordering::SeqCst), 1);
        assert_eq!(later_probe.callbacks.load(Ordering::SeqCst), 0);
        assert_eq!(later_probe.destroys.load(Ordering::SeqCst), 1);
        assert!(!first.is_active());
        assert!(!later.is_active());
        assert_eq!(runtime.cleanup.state(), RuntimeState::Running);

        runtime
            .cleanup
            .drain_shutdown()
            .expect("Runtime shutdown succeeds after callback teardown");
    }

    #[test]
    fn property_notification_shutdown_stops_after_current_item_and_commits_control_work() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let bridge = Arc::new(TestUiLoopBridge::default());
        let bridge_trait: Arc<dyn UiLoopBridge> = bridge.clone();
        runtime
            .set_ui_loop_bridge(&bridge_trait)
            .expect("register UI-loop bridge");
        let dispatcher = runtime
            .dispatcher()
            .expect("dispatcher lock")
            .expect("registered dispatcher");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let probe = Arc::new(OrderedDeliveryProbe::default());
        let first_token = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register first Property");
        let second_token = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register second Property");
        let first_identity = registry
            .validate(
                first_token,
                &PropertyHandleKind::Property,
                &runtime.identity,
            )
            .expect("first Property remains live")
            .identity();
        let second_identity = registry
            .validate(
                second_token,
                &PropertyHandleKind::Property,
                &runtime.identity,
            )
            .expect("second Property remains live")
            .identity();
        for (identity, item, requests_shutdown) in [
            (first_identity, 0, true),
            (first_identity, 100, false),
            (second_identity, 1, false),
        ] {
            let context = Box::new(OrderedDeliveryContext {
                probe: Arc::clone(&probe),
                item,
                runtime: None,
                shutdown_dispatcher: requests_shutdown.then(|| dispatcher.clone()),
            });
            let userdata = Box::into_raw(context).cast::<c_void>();
            // SAFETY: the Subscription owns its boxed context through shutdown.
            let subscription = Arc::new(unsafe {
                Subscription::new(
                    ordered_delivery_callback,
                    userdata,
                    ordered_delivery_destroy,
                )
            });
            runtime
                .register_subscription_owner(identity, &subscription)
                .expect("register subscriber");
        }
        assert_eq!(
            runtime.apply_property_candidate(&registry, first_token, TypedValue::I64(1)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, second_token, TypedValue::I64(1)),
            Ok(ValueCommit::Changed)
        );

        let scope = runtime.begin_host_event().expect("begin host event");
        assert!(
            runtime
                .end_host_event(scope)
                .expect("commit callback shutdown")
        );
        assert_eq!(
            *probe
                .callbacks
                .lock()
                .expect("read complete delivery items"),
            vec![0, 100],
            "all subscribers in the current item finish before shutdown skips later items"
        );
        assert_eq!(runtime.cleanup_queue().state(), RuntimeState::Closed);
        assert_eq!(bridge.destroys.load(Ordering::SeqCst), 1);
        assert_eq!(
            *bridge.wake_priorities.lock().expect("read wake priorities"),
            vec![WakePriority::Control]
        );
    }

    #[test]
    fn property_delivery_generation_snapshot_preserves_property_and_subscriber_order() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let first_token = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register first Property");
        let second_token = runtime
            .register_property(&registry, TypedValue::I64(10))
            .expect("register second Property");
        let first_identity = registry
            .validate(
                first_token,
                &PropertyHandleKind::Property,
                &runtime.identity,
            )
            .expect("first Property remains live")
            .identity();
        let second_identity = registry
            .validate(
                second_token,
                &PropertyHandleKind::Property,
                &runtime.identity,
            )
            .expect("second Property remains live")
            .identity();

        let first_probe = Box::new(SubscriptionOwnerProbe::default());
        let first_userdata = (std::ptr::from_ref(&*first_probe))
            .cast_mut()
            .cast::<c_void>();
        // SAFETY: the probe outlives its registration and explicit cancellation.
        let first_subscriber = Arc::new(unsafe {
            Subscription::new(
                subscription_owner_callback,
                first_userdata,
                subscription_owner_destroy,
            )
        });
        let first_subscriber_weak = Arc::downgrade(&first_subscriber);

        let second_probe = Box::new(SubscriptionOwnerProbe::default());
        let second_userdata = (std::ptr::from_ref(&*second_probe))
            .cast_mut()
            .cast::<c_void>();
        // SAFETY: the probe outlives its registration and explicit cancellation.
        let second_subscriber = Arc::new(unsafe {
            Subscription::new(
                subscription_owner_callback,
                second_userdata,
                subscription_owner_destroy,
            )
        });
        let second_subscriber_weak = Arc::downgrade(&second_subscriber);

        let third_probe = Box::new(SubscriptionOwnerProbe::default());
        let third_userdata = (std::ptr::from_ref(&*third_probe))
            .cast_mut()
            .cast::<c_void>();
        // SAFETY: the probe outlives its registration and explicit cancellation.
        let third_subscriber = Arc::new(unsafe {
            Subscription::new(
                subscription_owner_callback,
                third_userdata,
                subscription_owner_destroy,
            )
        });
        let third_subscriber_weak = Arc::downgrade(&third_subscriber);

        runtime
            .register_subscription_owner(first_identity, &first_subscriber)
            .expect("register first Property subscriber");
        runtime
            .register_subscription_owner(first_identity, &second_subscriber)
            .expect("register second Property subscriber");
        runtime
            .register_subscription_owner(second_identity, &third_subscriber)
            .expect("register second Property's subscriber");
        drop(first_subscriber);
        drop(second_subscriber);
        drop(third_subscriber);

        assert_eq!(
            runtime.apply_property_candidate(&registry, second_token, TypedValue::I64(11)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, first_token, TypedValue::I64(1)),
            Ok(ValueCommit::Changed)
        );
        assert_eq!(
            runtime.apply_property_candidate(&registry, second_token, TypedValue::I64(12)),
            Ok(ValueCommit::Changed)
        );

        let items = runtime
            .prepare_property_delivery_generation()
            .expect("prepare delivery generation");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].identity, second_identity);
        assert_eq!(items[1].identity, first_identity);
        assert_eq!(items[0].subscribers.len(), 1);
        assert_eq!(items[1].subscribers.len(), 2);
        assert!(Arc::ptr_eq(
            &items[1].subscribers[0],
            &first_subscriber_weak.upgrade().unwrap()
        ));
        assert!(Arc::ptr_eq(
            &items[1].subscribers[1],
            &second_subscriber_weak.upgrade().unwrap()
        ));
        assert_eq!(first_probe.callbacks.load(Ordering::SeqCst), 0);
        assert_eq!(second_probe.callbacks.load(Ordering::SeqCst), 0);
        assert_eq!(third_probe.callbacks.load(Ordering::SeqCst), 0);
        assert!(
            runtime
                .prepare_property_delivery_generation()
                .expect("generation was consumed once")
                .is_empty()
        );

        assert_eq!(
            runtime.apply_property_candidate(&registry, first_token, TypedValue::I64(2)),
            Ok(ValueCommit::Changed)
        );
        let next_items = runtime
            .prepare_property_delivery_generation()
            .expect("prepare next generation");
        assert_eq!(next_items.len(), 1);
        assert_eq!(next_items[0].identity, first_identity);
        assert_eq!(next_items[0].subscribers.len(), 2);

        assert!(
            runtime
                .cancel_subscription_owner(first_identity, &items[1].subscribers[0])
                .expect("cancel a subscriber after snapshot creation")
        );
        assert!(
            runtime
                .cancel_subscription_owner(second_identity, &items[0].subscribers[0])
                .expect("cancel second Property subscriber")
        );
        assert!(first_subscriber_weak.upgrade().is_some());
        drop(items);
        assert!(first_subscriber_weak.upgrade().is_some());
        assert!(second_subscriber_weak.upgrade().is_some());
        assert!(third_subscriber_weak.upgrade().is_none());

        let current = runtime
            .snapshot_property_subscriptions(first_identity)
            .expect("remaining subscriber stays registered");
        assert_eq!(current.len(), 1);
        assert!(Arc::ptr_eq(
            &current[0],
            &second_subscriber_weak.upgrade().unwrap()
        ));
        assert!(
            runtime
                .cancel_subscription_owner(first_identity, &current[0])
                .expect("cancel remaining first Property subscriber")
        );
        drop(current);
        assert!(second_subscriber_weak.upgrade().is_some());
        drop(next_items);
        assert!(first_subscriber_weak.upgrade().is_none());
        assert!(second_subscriber_weak.upgrade().is_none());
        assert_eq!(first_probe.destroys.load(Ordering::SeqCst), 1);
        assert_eq!(second_probe.destroys.load(Ordering::SeqCst), 1);
        assert_eq!(third_probe.destroys.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn property_delivery_generation_snapshot_restores_on_stale_property() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let token = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register Property");
        let valid_identity = registry
            .validate(token, &PropertyHandleKind::Property, &runtime.identity)
            .expect("Property remains live")
            .identity();
        let stale_identity = ObjectIdentity::new(
            u64::MAX,
            NonZeroU64::new(u64::MAX).expect("nonzero generation"),
        );
        {
            let mut changes = runtime
                .property_changes
                .lock()
                .expect("lock Property change queue");
            changes.commit_change(valid_identity);
            changes.commit_change(stale_identity);
        }

        assert!(runtime.prepare_property_delivery_generation().is_err());
        assert_eq!(
            runtime
                .take_property_change_generation()
                .expect("take restored generation"),
            vec![valid_identity, stale_identity]
        );
    }

    #[test]
    fn runtime_subscription_owner_keeps_callback_alive_until_cancel() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let token = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register an object identity for the subscription owner");
        let identity = registry
            .validate(token, &PropertyHandleKind::Property, &runtime.identity)
            .expect("object remains registered")
            .identity();
        let probe = Box::new(SubscriptionOwnerProbe::default());
        let userdata = (std::ptr::from_ref(&*probe)).cast_mut().cast::<c_void>();
        // SAFETY: the boxed probe outlives the Runtime owner and explicit cancellation.
        let subscription = Arc::new(unsafe {
            Subscription::new(
                subscription_owner_callback,
                userdata,
                subscription_owner_destroy,
            )
        });
        let weak = Arc::downgrade(&subscription);

        runtime
            .register_subscription_owner(identity, &subscription)
            .expect("register active Runtime owner");
        drop(subscription);
        let snapshot = runtime
            .snapshot_property_subscriptions(identity)
            .expect("snapshot registered subscription");
        assert_eq!(snapshot.len(), 1);
        assert!(weak.upgrade().is_some());
        assert!(snapshot[0].invoke());
        assert_eq!(probe.callbacks.load(Ordering::SeqCst), 1);
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 0);

        assert!(
            runtime
                .cancel_subscription_owner(identity, &snapshot[0])
                .expect("cancel active subscription")
        );
        assert!(
            !runtime
                .cancel_subscription_owner(identity, &snapshot[0])
                .expect("repeated cancel is harmless")
        );
        assert!(
            runtime
                .snapshot_property_subscriptions(identity)
                .expect("canceled subscription is pruned")
                .is_empty()
        );
        drop(snapshot);
        assert!(weak.upgrade().is_none());
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn runtime_property_subscription_index_preserves_order_and_cancels_individually() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new();
        let token = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register a Property");
        let identity = registry
            .validate(token, &PropertyHandleKind::Property, &runtime.identity)
            .expect("Property remains registered")
            .identity();

        let first_probe = Box::new(SubscriptionOwnerProbe::default());
        let first_userdata = (std::ptr::from_ref(&*first_probe))
            .cast_mut()
            .cast::<c_void>();
        // SAFETY: the probe outlives Runtime ownership and explicit cancellation.
        let first = Arc::new(unsafe {
            Subscription::new(
                subscription_owner_callback,
                first_userdata,
                subscription_owner_destroy,
            )
        });
        let first_weak = Arc::downgrade(&first);

        let second_probe = Box::new(SubscriptionOwnerProbe::default());
        let second_userdata = (std::ptr::from_ref(&*second_probe))
            .cast_mut()
            .cast::<c_void>();
        // SAFETY: the probe outlives Runtime ownership and explicit cancellation.
        let second = Arc::new(unsafe {
            Subscription::new(
                subscription_owner_callback,
                second_userdata,
                subscription_owner_destroy,
            )
        });
        let second_weak = Arc::downgrade(&second);

        runtime
            .register_subscription_owner(identity, &first)
            .expect("register first subscriber");
        runtime
            .register_subscription_owner(identity, &second)
            .expect("register second subscriber on the same Property");
        drop(first);
        drop(second);

        let snapshot = runtime
            .snapshot_property_subscriptions(identity)
            .expect("snapshot subscribers in registration order");
        assert_eq!(snapshot.len(), 2);
        assert!(Arc::ptr_eq(&snapshot[0], &first_weak.upgrade().unwrap()));
        assert!(Arc::ptr_eq(&snapshot[1], &second_weak.upgrade().unwrap()));

        assert!(
            runtime
                .cancel_subscription_owner(identity, &snapshot[0])
                .expect("cancel only the first subscriber")
        );
        assert!(!snapshot[0].is_active());
        assert!(snapshot[1].is_active());
        let after_first_cancel = runtime
            .snapshot_property_subscriptions(identity)
            .expect("prune canceled first subscriber");
        assert_eq!(after_first_cancel.len(), 1);
        assert!(Arc::ptr_eq(
            &after_first_cancel[0],
            &second_weak.upgrade().unwrap()
        ));
        drop(snapshot);
        assert!(first_weak.upgrade().is_none());
        assert!(second_weak.upgrade().is_some());

        assert!(
            runtime
                .cancel_subscription_owner(identity, &after_first_cancel[0])
                .expect("cancel the remaining subscriber")
        );
        drop(after_first_cancel);
        assert!(second_weak.upgrade().is_none());
        assert_eq!(first_probe.destroys.load(Ordering::SeqCst), 1);
        assert_eq!(second_probe.destroys.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn runtime_subscription_owner_rejects_duplicate_and_worker_registration() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let registry = Arc::new(HandleRegistry::<PropertyHandleKind, RuntimeIdentity>::new());
        let token = runtime
            .register_property(&registry, TypedValue::I64(0))
            .expect("register an object identity for the subscription owner");
        let identity = registry
            .validate(token, &PropertyHandleKind::Property, &runtime.identity)
            .expect("object remains registered")
            .identity();
        let probe = Box::new(SubscriptionOwnerProbe::default());
        let userdata = (std::ptr::from_ref(&*probe)).cast_mut().cast::<c_void>();
        // SAFETY: the boxed probe outlives the failed attempts and explicit cancellation.
        let first = Arc::new(unsafe {
            Subscription::new(
                subscription_owner_callback,
                userdata,
                subscription_owner_destroy,
            )
        });
        runtime
            .register_subscription_owner(identity, &first)
            .expect("first owner registration");

        assert_eq!(
            runtime.register_subscription_owner(identity, &first),
            Err(RuntimePropertyError::WrongObjectState)
        );
        assert_eq!(Arc::strong_count(&first), 2);

        // SAFETY: the boxed probe outlives the failed worker registration and explicit cancel.
        let worker_candidate = Arc::new(unsafe {
            Subscription::new(
                subscription_owner_callback,
                userdata,
                subscription_owner_destroy,
            )
        });

        let worker_runtime = Arc::clone(&runtime);
        let worker_subscription = Arc::clone(&worker_candidate);
        let worker_result = std::thread::spawn(move || {
            worker_runtime.register_subscription_owner(identity, &worker_subscription)
        })
        .join()
        .expect("worker registration thread completes");
        assert_eq!(worker_result, Err(RuntimePropertyError::WrongThread));
        let snapshot = runtime
            .snapshot_property_subscriptions(identity)
            .expect("snapshot after rejected registrations");
        assert_eq!(snapshot.len(), 1);
        assert!(Arc::ptr_eq(&snapshot[0], &first));

        assert!(
            runtime
                .cancel_subscription_owner(identity, &first)
                .expect("cancel first owner")
        );
        drop(snapshot);
        worker_candidate.cancel();
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn property_kind_query_returns_initial_kind_on_ui_thread() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let registry =
            Arc::new(HandleRegistry::<PropertyHandleKind, super::RuntimeIdentity>::new());
        let values = [
            (TypedValue::Bool(true), ValueKind::Bool),
            (TypedValue::I64(-7), ValueKind::I64),
            (
                TypedValue::Utf8(OwnedUtf8::copy_from_bytes("初期値".as_bytes()).expect("UTF-8")),
                ValueKind::Utf8,
            ),
            (TypedValue::F64(f64::INFINITY), ValueKind::F64),
            (
                TypedValue::Decimal(DecimalValue::new(true, 2, 123).expect("valid decimal")),
                ValueKind::Decimal,
            ),
        ];

        let tokens = values
            .iter()
            .map(|(initial, _)| {
                runtime
                    .register_property(&registry, initial.clone())
                    .expect("register typed Property")
            })
            .collect::<Vec<_>>();
        for ((_, expected), token) in values.iter().zip(tokens.iter().copied()) {
            assert_eq!(runtime.property_kind(&registry, token), Ok(*expected));
        }

        let wrong_kind = runtime
            .register_ui_object(&registry, PropertyHandleKind::Other, || ())
            .expect("register other handle kind");
        assert!(matches!(
            runtime.property_kind(&registry, wrong_kind),
            Err(RuntimePropertyError::Registry(RegistryError::WrongKind))
        ));
        let other_runtime = RuntimeCore::new().expect("second Runtime identity is available");
        let other_token = other_runtime
            .register_property(&registry, TypedValue::Bool(false))
            .expect("register Property in second Runtime");
        assert!(matches!(
            runtime.property_kind(&registry, other_token),
            Err(RuntimePropertyError::Registry(RegistryError::WrongRuntime))
        ));
        let invalid_token =
            crate::handle_registry::HandleToken::from_abi(u64::MAX).expect("nonzero test token");
        assert!(matches!(
            runtime.property_kind(&registry, invalid_token),
            Err(RuntimePropertyError::Registry(RegistryError::InvalidToken))
        ));

        let worker_runtime = Arc::clone(&runtime);
        let worker_registry = Arc::clone(&registry);
        let token = tokens[0];
        assert!(matches!(
            std::thread::spawn(move || worker_runtime.property_kind(&worker_registry, token))
                .join()
                .expect("worker query completes"),
            Err(RuntimePropertyError::WrongThread)
        ));

        runtime
            .cleanup
            .drain_shutdown()
            .expect("shutdown clears Property state");
        assert!(matches!(
            runtime.property_kind(&registry, tokens[0]),
            Err(RuntimePropertyError::Registry(RegistryError::InvalidToken))
        ));
    }

    impl Drop for RuntimeUiStateProbe {
        fn drop(&mut self) {
            self.drop_count.fetch_add(1, Ordering::SeqCst);
            self.drop_threads
                .lock()
                .expect("drop thread record lock")
                .push(thread::current().id());
        }
    }

    #[test]
    fn runtime_core_clears_ui_object_store_before_closed() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let ui_thread = thread::current().id();
        let drop_count = Arc::new(AtomicUsize::new(0));
        let drop_threads = Arc::new(Mutex::new(Vec::new()));
        let identities = {
            let mut store = runtime.ui_objects.lock().expect("UI object store lock");
            (0..2)
                .map(|_| {
                    store
                        .insert(Box::new(RuntimeUiStateProbe {
                            drop_count: Arc::clone(&drop_count),
                            drop_threads: Arc::clone(&drop_threads),
                        }))
                        .expect("insert Runtime UI state")
                })
                .collect::<Vec<_>>()
        };

        let worker_store = Arc::clone(&runtime.ui_objects);
        let worker_identity = identities[0];
        let worker_result = thread::spawn(move || {
            worker_store
                .lock()
                .expect("UI object store lock")
                .resolve(worker_identity)
                .map(|state| state.is::<RuntimeUiStateProbe>())
        })
        .join()
        .expect("worker completes");
        assert_eq!(worker_result, Err(UiObjectStoreError::WrongThread));
        assert_eq!(drop_count.load(Ordering::SeqCst), 0);

        runtime
            .cleanup
            .drain_shutdown()
            .expect("shutdown drains Runtime UI state");

        assert_eq!(drop_count.load(Ordering::SeqCst), 2);
        assert_eq!(runtime.cleanup.state(), RuntimeState::Closed);
        assert!(
            drop_threads
                .lock()
                .expect("drop thread record lock")
                .iter()
                .all(|thread_id| *thread_id == ui_thread)
        );
        let store = runtime.ui_objects.lock().expect("UI object store lock");
        for identity in identities {
            assert!(matches!(
                store.resolve(identity),
                Err(UiObjectStoreError::InvalidIdentity)
            ));
        }
        runtime
            .cleanup
            .drain_shutdown()
            .expect("repeated shutdown drain is safe");
        assert_eq!(drop_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn runtime_core_registers_ui_object_and_routes_cleanup() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
        let registry = Arc::new(HandleRegistry::<u8, super::RuntimeIdentity>::new());
        let ui_thread = thread::current().id();
        let drop_count = Arc::new(AtomicUsize::new(0));
        let drop_threads = Arc::new(Mutex::new(Vec::new()));
        let token = runtime
            .register_ui_object(&registry, 7, || RuntimeUiStateProbe {
                drop_count: Arc::clone(&drop_count),
                drop_threads: Arc::clone(&drop_threads),
            })
            .expect("register Runtime UI object");

        let worker_runtime = Arc::clone(&runtime);
        let worker_registry = Arc::clone(&registry);
        let worker_factory_calls = Arc::new(AtomicUsize::new(0));
        let factory_calls = Arc::clone(&worker_factory_calls);
        let worker_registration = thread::spawn(move || {
            worker_runtime.register_ui_object(&worker_registry, 7, move || {
                factory_calls.fetch_add(1, Ordering::SeqCst);
            })
        })
        .join()
        .expect("worker registration attempt completes");
        assert_eq!(
            worker_registration,
            Err(RuntimeObjectRegistrationError::WrongThread)
        );
        assert_eq!(worker_factory_calls.load(Ordering::SeqCst), 0);

        let pin = registry
            .validate(token, &7, &runtime.identity())
            .expect("validate registered Runtime object");
        let identity = pin.identity();
        assert!(
            runtime
                .ui_objects
                .lock()
                .expect("UI object store lock")
                .resolve(identity)
                .expect("resolve registered state")
                .is::<RuntimeUiStateProbe>()
        );
        drop(pin);

        let release_registry = Arc::clone(&registry);
        let release = thread::spawn(move || release_registry.release(token))
            .join()
            .expect("worker release completes")
            .expect("release final Runtime object owner");
        assert!(release.cleanup_accepted);
        assert_eq!(drop_count.load(Ordering::SeqCst), 0);
        assert!(
            runtime
                .ui_objects
                .lock()
                .expect("UI object store lock")
                .resolve(identity)
                .is_ok()
        );

        runtime
            .cleanup
            .drain_shutdown()
            .expect("UI thread drains object finalizer");
        assert_eq!(drop_count.load(Ordering::SeqCst), 1);
        assert_eq!(runtime.cleanup.state(), RuntimeState::Closed);
        assert_eq!(
            *drop_threads.lock().expect("drop thread record lock"),
            vec![ui_thread]
        );
        assert!(matches!(
            runtime
                .ui_objects
                .lock()
                .expect("UI object store lock")
                .resolve(identity),
            Err(UiObjectStoreError::InvalidIdentity)
        ));
        let closed_factory_calls = Arc::new(AtomicUsize::new(0));
        let factory_calls = Arc::clone(&closed_factory_calls);
        assert_eq!(
            runtime.register_ui_object(&registry, 7, move || {
                factory_calls.fetch_add(1, Ordering::SeqCst);
            }),
            Err(RuntimeObjectRegistrationError::Closed)
        );
        assert_eq!(closed_factory_calls.load(Ordering::SeqCst), 0);

        let failed_runtime = RuntimeCore::new().expect("second Runtime identity is available");
        let failed_registry = HandleRegistry::<u8, super::RuntimeIdentity>::new();
        failed_registry.fail_next_token_issue();
        let failed_drop_count = Arc::new(AtomicUsize::new(0));
        let failed_drop_threads = Arc::new(Mutex::new(Vec::new()));
        let failed =
            failed_runtime.register_ui_object(&failed_registry, 7, || RuntimeUiStateProbe {
                drop_count: Arc::clone(&failed_drop_count),
                drop_threads: Arc::clone(&failed_drop_threads),
            });
        assert_eq!(
            failed.err(),
            Some(RuntimeObjectRegistrationError::Registry(
                RegistryError::TokenExhausted
            ))
        );
        assert_eq!(failed_drop_count.load(Ordering::SeqCst), 1);
        assert_eq!(
            *failed_drop_threads
                .lock()
                .expect("failed drop thread record lock"),
            vec![ui_thread]
        );
    }

    #[test]
    fn tree_owner_tree_runtime_store_detaches_child_on_parent_cleanup() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<u8, RuntimeIdentity>::new();
        let drop_count = Arc::new(AtomicUsize::new(0));
        let drop_threads = Arc::new(Mutex::new(Vec::new()));
        let parent = runtime
            .register_tree_ui_object(&registry, 7, true, || ())
            .expect("register parent node");
        let child_drop_count = Arc::clone(&drop_count);
        let child_drop_threads = Arc::clone(&drop_threads);
        let child = runtime
            .register_tree_ui_object(&registry, 7, false, move || RuntimeUiStateProbe {
                drop_count: child_drop_count,
                drop_threads: child_drop_threads,
            })
            .expect("register child node");
        let child_pin = registry
            .validate(child, &7, &runtime.identity())
            .expect("child token is live");
        let child_identity = child_pin.identity();
        drop(child_pin);

        runtime
            .attach_tree_child(&registry, parent, child, 7, || Ok(()))
            .expect("attach child into Runtime tree");
        assert!(
            !registry
                .release(child)
                .expect("release child public owner while attached")
                .cleanup_accepted
        );
        let reissued_child = runtime
            .tree_child_handle(&registry, parent, child_identity, 7)
            .expect("parent TreeOwner can issue an independent public owner");
        assert!(
            !registry
                .release(reissued_child)
                .expect("release reissued owner while still attached")
                .cleanup_accepted
        );
        assert_eq!(drop_count.load(Ordering::SeqCst), 0);

        assert!(
            registry
                .release(parent)
                .expect("release final parent owner")
                .cleanup_accepted
        );
        runtime
            .cleanup
            .drain_pending()
            .expect("parent and newly-unowned child finalizers drain");
        assert_eq!(drop_count.load(Ordering::SeqCst), 1);
        assert!(matches!(
            runtime
                .ui_objects
                .lock()
                .expect("UI object store lock")
                .resolve(child_identity),
            Err(UiObjectStoreError::InvalidIdentity)
        ));
        assert!(
            runtime
                .ui_tree
                .lock()
                .expect("Logical Tree store lock")
                .is_empty()
        );
    }

    #[test]
    fn tree_owner_tree_runtime_parent_cleanup_preserves_detached_subtree() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<u8, RuntimeIdentity>::new();
        let root = runtime
            .register_tree_ui_object(&registry, 7, true, || ())
            .expect("register root container");
        let middle = runtime
            .register_tree_ui_object(&registry, 7, true, || ())
            .expect("register middle container");
        let leaf = runtime
            .register_tree_ui_object(&registry, 7, false, || ())
            .expect("register leaf");
        let leaf_identity = registry
            .validate(leaf, &7, &runtime.identity())
            .expect("leaf is live")
            .identity();

        runtime
            .attach_tree_child(&registry, root, middle, 7, || Ok(()))
            .expect("attach middle to root");
        runtime
            .attach_tree_child(&registry, middle, leaf, 7, || Ok(()))
            .expect("attach leaf to middle");
        registry.release(root).expect("release root public owner");
        runtime
            .cleanup
            .drain_pending()
            .expect("root cleanup detaches and preserves live subtree");

        let new_leaf_owner = runtime
            .tree_child_handle(&registry, middle, leaf_identity, 7)
            .expect("detached middle retains its child subtree");
        registry
            .validate(new_leaf_owner, &7, &runtime.identity())
            .expect("reissued leaf owner remains valid");
        registry
            .release(new_leaf_owner)
            .expect("release reissued leaf");
        runtime
            .detach_tree_child(&registry, middle, leaf, 7)
            .expect("detach leaf through Runtime tree store");
        assert!(
            runtime
                .tree_child_handle(&registry, middle, leaf_identity, 7)
                .is_err()
        );
        registry.release(leaf).expect("release leaf owner");
        registry
            .release(middle)
            .expect("release detached middle owner");
        runtime
            .cleanup
            .drain_pending()
            .expect("remaining subtree finalizers drain");
        assert!(
            runtime
                .ui_tree
                .lock()
                .expect("Logical Tree store lock")
                .is_empty()
        );
    }

    #[test]
    fn tree_owner_tree_runtime_shutdown_drops_tree_owners_before_ui_state() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = HandleRegistry::<u8, RuntimeIdentity>::new();
        let drop_count = Arc::new(AtomicUsize::new(0));
        let drop_threads = Arc::new(Mutex::new(Vec::new()));
        let root_drop_count = Arc::clone(&drop_count);
        let root_drop_threads = Arc::clone(&drop_threads);
        let root = runtime
            .register_tree_ui_object(&registry, 7, true, move || RuntimeUiStateProbe {
                drop_count: root_drop_count,
                drop_threads: root_drop_threads,
            })
            .expect("register root");
        let child_drop_count = Arc::clone(&drop_count);
        let child_drop_threads = Arc::clone(&drop_threads);
        let child = runtime
            .register_tree_ui_object(&registry, 7, false, move || RuntimeUiStateProbe {
                drop_count: child_drop_count,
                drop_threads: child_drop_threads,
            })
            .expect("register child");
        let identities = [root, child].map(|token| {
            registry
                .validate(token, &7, &runtime.identity())
                .expect("registered tree object is live")
                .identity()
        });
        runtime
            .attach_tree_child(&registry, root, child, 7, || Ok(()))
            .expect("attach before shutdown");

        runtime
            .cleanup
            .drain_shutdown()
            .expect("shutdown drops tree owners before clearing UI state");
        assert_eq!(runtime.cleanup.state(), RuntimeState::Closed);
        assert_eq!(drop_count.load(Ordering::SeqCst), 2);
        assert_eq!(
            *drop_threads.lock().expect("drop thread record lock"),
            vec![thread::current().id(); 2]
        );
        assert!(
            runtime
                .ui_tree
                .lock()
                .expect("Logical Tree store lock")
                .is_empty()
        );
        let store = runtime.ui_objects.lock().expect("UI object store lock");
        for identity in identities {
            assert!(matches!(
                store.resolve(identity),
                Err(UiObjectStoreError::InvalidIdentity)
            ));
        }
        drop(store);
        for token in [root, child] {
            assert!(
                !registry
                    .release(token)
                    .expect("release remaining public token after shutdown")
                    .cleanup_accepted
            );
        }
        assert_eq!(drop_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn runtime_core_registers_bridge_once_on_owner_thread() {
        let runtime = Arc::new(RuntimeCore::new().expect("Runtime identity is available"));
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
    fn runtime_core_identities_are_unique_across_concurrent_creation() {
        const THREADS: usize = 8;
        const RUNTIMES_PER_THREAD: usize = 32;

        let identities = thread::scope(|scope| {
            (0..THREADS)
                .map(|_| {
                    scope.spawn(|| {
                        (0..RUNTIMES_PER_THREAD)
                            .map(|_| {
                                RuntimeCore::new()
                                    .expect("Runtime identity is available")
                                    .identity()
                                    .get()
                                    .get()
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .map(|task| task.join().expect("Runtime creation worker completes"))
                .collect::<Vec<_>>()
        });

        let mut identities = identities.into_iter().flatten().collect::<Vec<_>>();
        assert_eq!(identities.len(), THREADS * RUNTIMES_PER_THREAD);
        assert!(identities.iter().all(|identity| *identity > 0));
        identities.sort_unstable();
        assert!(identities.windows(2).all(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn runtime_core_binds_object_control_to_its_identity_and_cleanup_queue() {
        let runtime = RuntimeCore::new().expect("Runtime identity is available");
        let other_runtime = RuntimeCore::new().expect("Runtime identity is available");
        let registry = Arc::new(HandleRegistry::<u8, super::RuntimeIdentity>::new());
        let identity = runtime
            .ui_objects
            .lock()
            .expect("UI object store lock")
            .insert(Box::new(()))
            .expect("insert state for CoreObjectControl");
        let control = runtime.new_object_control(identity);

        assert_eq!(*control.runtime_id(), runtime.identity());
        assert_eq!(control.identity(), identity);
        let token = registry
            .register(7, runtime.identity(), Arc::clone(&control))
            .expect("register Runtime-owned control");
        let pin = registry
            .validate(token, &7, &runtime.identity())
            .expect("owning Runtime validates control");
        assert_eq!(pin.identity(), identity);
        assert_eq!(
            registry
                .validate(token, &7, &other_runtime.identity())
                .err(),
            Some(RegistryError::WrongRuntime)
        );
        drop(pin);

        let worker_registry = Arc::clone(&registry);
        let outcome = thread::spawn(move || worker_registry.release(token))
            .join()
            .expect("worker release completes")
            .expect("release final owner");
        assert!(outcome.cleanup_accepted);
        assert_eq!(runtime.cleanup_queue().queued_count(), 1);
        assert_eq!(other_runtime.cleanup_queue().queued_count(), 0);
        runtime
            .cleanup_queue()
            .drain_shutdown()
            .expect("owning Runtime drains object cleanup");
        assert_eq!(runtime.cleanup_queue().state(), RuntimeState::Closed);
        assert_eq!(other_runtime.cleanup_queue().state(), RuntimeState::Running);
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
        wake_priorities: Mutex<Vec<WakePriority>>,
        destroys: AtomicUsize,
    }

    impl UiLoopBridge for TestUiLoopBridge {
        fn schedule_wake(&self, priority: WakePriority) {
            self.wakes.fetch_add(1, Ordering::SeqCst);
            self.wake_priorities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(priority);
        }

        fn destroy_at_safe_point(&self) {
            self.destroys.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct WakeSignalBridge {
        wake_sender: mpsc::Sender<()>,
    }

    impl UiLoopBridge for WakeSignalBridge {
        fn schedule_wake(&self, _priority: WakePriority) {
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

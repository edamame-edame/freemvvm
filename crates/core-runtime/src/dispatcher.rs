// This slice is intentionally internal until the Runtime/ABI host-bridge unit connects it.
#![allow(dead_code)]

use crate::runtime::UiThreadContext;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) type PostedCallback = Box<dyn FnOnce() + Send + 'static>;

pub(crate) struct PostedWork {
    callback: Option<PostedCallback>,
    destroy: Option<PostedCallback>,
}

impl PostedWork {
    pub(crate) fn new(callback: PostedCallback, destroy: PostedCallback) -> Self {
        Self {
            callback: Some(callback),
            destroy: Some(destroy),
        }
    }

    pub(crate) fn callback(callback: PostedCallback) -> Self {
        Self::new(callback, Box::new(|| {}))
    }

    pub(crate) fn destroy_by_caller(self) {
        self.run_destroy();
    }

    fn run_destroy(mut self) {
        self.callback.take();
        if let Some(destroy) = self.destroy.take() {
            destroy();
        }
    }

    fn invoke(mut self) {
        let destroy = DestroyGuard(self.destroy.take());
        if let Some(callback) = self.callback.take() {
            callback();
        }
        drop(destroy);
    }

    fn cancel(self) {
        self.run_destroy();
    }
}

struct DestroyGuard(Option<PostedCallback>);

impl Drop for DestroyGuard {
    fn drop(&mut self) {
        if let Some(destroy) = self.0.take() {
            destroy();
        }
    }
}

/// The host bridge schedules a future UI-loop turn and never invokes it inline.
pub(crate) trait UiLoopBridge: Send + Sync {
    fn schedule_wake(&self, priority: WakePriority);
    fn destroy_at_safe_point(&self) {}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WakePriority {
    Normal,
    Control,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DispatchError {
    WrongThread,
    AlreadyDispatching,
    Closed,
    SynchronizationPoisoned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PostErrorKind {
    SynchronizationPoisoned,
    OutOfMemory,
    Closed,
}

pub(crate) struct PostError {
    pub(crate) kind: PostErrorKind,
    #[allow(dead_code)]
    pub(crate) work: PostedWork,
}

impl std::fmt::Debug for PostError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PostError")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl PostError {
    #[allow(dead_code)]
    pub(crate) fn into_work(self) -> PostedWork {
        self.work
    }
}

struct QueueState {
    lifecycle: DispatcherLifecycle,
    callbacks: VecDeque<PostedWork>,
    shutdown_pending: bool,
    wake_pending: bool,
    turn_active: bool,
    #[cfg(test)]
    fail_next_reservation: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DispatcherLifecycle {
    Running,
    Closing,
    Closed,
}

struct DispatcherInner {
    ui_thread: UiThreadContext,
    bridge: Arc<dyn UiLoopBridge>,
    state: Mutex<QueueState>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShutdownRequestAdmission {
    NewlyPending { schedule_wake: bool },
    AlreadyPending,
    Closed,
}

/// FIFO, non-reentrant post dispatcher. Each call to `dispatch_turn` invokes
/// at most one callback and schedules another turn when work remains.
#[derive(Clone)]
pub(crate) struct Dispatcher {
    inner: Arc<DispatcherInner>,
}

impl Dispatcher {
    pub(crate) fn new(bridge: Arc<dyn UiLoopBridge>) -> Self {
        Self::new_for_ui_thread(bridge, UiThreadContext::capture())
    }

    pub(crate) fn new_for_ui_thread(
        bridge: Arc<dyn UiLoopBridge>,
        ui_thread: UiThreadContext,
    ) -> Self {
        Self {
            inner: Arc::new(DispatcherInner {
                ui_thread,
                bridge,
                state: Mutex::new(QueueState {
                    lifecycle: DispatcherLifecycle::Running,
                    callbacks: VecDeque::new(),
                    shutdown_pending: false,
                    wake_pending: false,
                    turn_active: false,
                    #[cfg(test)]
                    fail_next_reservation: false,
                }),
            }),
        }
    }

    /// Ownership transfers only after queue capacity is secured and insertion commits.
    pub(crate) fn post(&self, work: PostedWork) -> Result<(), PostError> {
        let mut state = match self.inner.state.lock() {
            Ok(state) => state,
            Err(_) => {
                return Err(PostError {
                    kind: PostErrorKind::SynchronizationPoisoned,
                    work,
                });
            }
        };
        if state.lifecycle != DispatcherLifecycle::Running {
            return Err(PostError {
                kind: PostErrorKind::Closed,
                work,
            });
        }
        #[cfg(test)]
        if std::mem::replace(&mut state.fail_next_reservation, false) {
            return Err(PostError {
                kind: PostErrorKind::OutOfMemory,
                work,
            });
        }
        if state.callbacks.try_reserve(1).is_err() {
            return Err(PostError {
                kind: PostErrorKind::OutOfMemory,
                work,
            });
        }
        state.callbacks.push_back(work);
        let schedule = !state.wake_pending;
        state.wake_pending = true;
        drop(state);

        if schedule {
            self.inner.bridge.schedule_wake(WakePriority::Normal);
        }
        Ok(())
    }

    /// Invoke at most one callback on the Runtime's creation thread.
    pub(crate) fn dispatch_turn(&self) -> Result<bool, DispatchError> {
        self.dispatch_turn_with_shutdown(|| {})
    }

    /// Request shutdown without interrupting the active UI callback. Repeated requests coalesce.
    pub(crate) fn request_shutdown(&self) -> Result<(), DispatchError> {
        let admission = self.prepare_shutdown_request()?;
        self.schedule_shutdown_wake(admission);
        Ok(())
    }

    /// Reserve the priority shutdown state without invoking the host wake bridge.
    pub(crate) fn prepare_shutdown_request(
        &self,
    ) -> Result<ShutdownRequestAdmission, DispatchError> {
        let mut state = self.lock_state()?;
        if state.lifecycle != DispatcherLifecycle::Running {
            return Ok(ShutdownRequestAdmission::Closed);
        }
        if state.shutdown_pending {
            return Ok(ShutdownRequestAdmission::AlreadyPending);
        }
        state.shutdown_pending = true;
        let schedule_wake = !state.wake_pending;
        state.wake_pending = true;
        Ok(ShutdownRequestAdmission::NewlyPending { schedule_wake })
    }

    /// Notify the host only after any coordinating owner lock has been released.
    pub(crate) fn schedule_shutdown_wake(&self, admission: ShutdownRequestAdmission) {
        if matches!(
            admission,
            ShutdownRequestAdmission::NewlyPending {
                schedule_wake: true
            }
        ) {
            self.inner.bridge.schedule_wake(WakePriority::Control);
        }
    }

    /// Re-arm a pending shutdown after an explicitly requested host restart retry.
    pub(crate) fn prepare_shutdown_retry(&self) -> Result<bool, DispatchError> {
        let mut state = self.lock_state()?;
        if state.lifecycle != DispatcherLifecycle::Running || !state.shutdown_pending {
            return Ok(false);
        }
        state.wake_pending = true;
        Ok(true)
    }

    /// Always schedule a fresh turn for an explicit retry; a prior host wake may have been lost.
    pub(crate) fn schedule_shutdown_retry_wake(&self) {
        self.inner.bridge.schedule_wake(WakePriority::Control);
    }

    pub(crate) fn belongs_to_thread(&self, ui_thread: UiThreadContext) -> bool {
        self.inner.ui_thread == ui_thread
    }

    /// Destroy the host callback only after shutdown has closed the dispatcher.
    pub(crate) fn destroy_bridge_at_safe_point(&self) -> Result<(), DispatchError> {
        if !self.inner.ui_thread.is_current() {
            return Err(DispatchError::WrongThread);
        }
        let state = self.lock_state()?;
        if state.lifecycle != DispatcherLifecycle::Closed {
            return Err(DispatchError::Closed);
        }
        drop(state);
        self.inner.bridge.destroy_at_safe_point();
        Ok(())
    }

    /// Dispatch at most one normal callback, or service the priority shutdown request.
    pub(crate) fn dispatch_turn_with_shutdown<F>(
        &self,
        on_shutdown: F,
    ) -> Result<bool, DispatchError>
    where
        F: FnOnce(),
    {
        self.dispatch_turn_with(|| {}, on_shutdown)
    }

    /// Service only a pending priority shutdown at an outer host-event boundary.
    /// This never starts an ordinary callback from the boundary itself.
    pub(crate) fn run_pending_shutdown<F>(&self, on_shutdown: F) -> Result<bool, DispatchError>
    where
        F: FnOnce(),
    {
        if !self.inner.ui_thread.is_current() {
            return Err(DispatchError::WrongThread);
        }

        let cancelled = {
            let mut state = self.lock_state()?;
            if state.turn_active {
                return Err(DispatchError::AlreadyDispatching);
            }
            if state.lifecycle != DispatcherLifecycle::Running || !state.shutdown_pending {
                return Ok(false);
            }
            state.lifecycle = DispatcherLifecycle::Closing;
            state.shutdown_pending = false;
            state.wake_pending = false;
            std::mem::take(&mut state.callbacks)
        };

        for work in cancelled {
            work.cancel();
        }
        on_shutdown();

        let mut state = self.lock_state()?;
        state.lifecycle = DispatcherLifecycle::Closed;
        state.wake_pending = false;
        Ok(true)
    }

    pub(crate) fn close_and_cancel(&self) -> Result<(), DispatchError> {
        if !self.inner.ui_thread.is_current() {
            return Err(DispatchError::WrongThread);
        }
        let work = {
            let mut state = self.lock_state()?;
            if state.turn_active {
                return Err(DispatchError::AlreadyDispatching);
            }
            if state.lifecycle == DispatcherLifecycle::Closed {
                return Ok(());
            }
            state.lifecycle = DispatcherLifecycle::Closed;
            state.wake_pending = false;
            std::mem::take(&mut state.callbacks)
        };
        for work in work {
            work.cancel();
        }
        Ok(())
    }

    #[cfg(test)]
    fn fail_next_reservation(&self) {
        self.inner
            .state
            .lock()
            .expect("dispatcher lock")
            .fail_next_reservation = true;
    }

    fn dispatch_turn_with<F, S>(
        &self,
        before_recheck: F,
        on_shutdown: S,
    ) -> Result<bool, DispatchError>
    where
        F: FnOnce(),
        S: FnOnce(),
    {
        if !self.inner.ui_thread.is_current() {
            return Err(DispatchError::WrongThread);
        }

        let (callback, cancelled) = {
            let mut state = self.lock_state()?;
            if state.turn_active {
                return Err(DispatchError::AlreadyDispatching);
            }
            if state.lifecycle == DispatcherLifecycle::Closed {
                return Ok(false);
            }
            state.wake_pending = false;
            if state.shutdown_pending {
                state.shutdown_pending = false;
                state.lifecycle = DispatcherLifecycle::Closing;
                state.turn_active = true;
                (None, Some(std::mem::take(&mut state.callbacks)))
            } else {
                let callback = state.callbacks.pop_front();
                state.turn_active = callback.is_some();
                (callback, None)
            }
        };
        if let Some(cancelled) = cancelled {
            for work in cancelled {
                work.cancel();
            }
            on_shutdown();
            let mut state = self.lock_state()?;
            state.lifecycle = DispatcherLifecycle::Closed;
            state.turn_active = false;
            state.wake_pending = false;
            return Ok(true);
        }
        let Some(callback) = callback else {
            return Ok(false);
        };

        let guard = TurnGuard {
            inner: Arc::clone(&self.inner),
            before_recheck: Some(Box::new(move || {
                before_recheck();
                let _ = on_shutdown;
            })),
        };
        callback.invoke();
        drop(guard);
        Ok(true)
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, QueueState>, DispatchError> {
        self.inner
            .state
            .lock()
            .map_err(|_| DispatchError::SynchronizationPoisoned)
    }
}

struct TurnGuard<F>
where
    F: FnOnce(),
{
    inner: Arc<DispatcherInner>,
    before_recheck: Option<Box<F>>,
}

impl<F> Drop for TurnGuard<F>
where
    F: FnOnce(),
{
    fn drop(&mut self) {
        if let Some(before_recheck) = self.before_recheck.take() {
            before_recheck();
        }
        let schedule = {
            let mut state = self
                .inner
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.turn_active = false;
            let schedule = !state.callbacks.is_empty() && !state.wake_pending;
            if schedule {
                state.wake_pending = true;
            }
            schedule
        };
        if schedule {
            self.inner.bridge.schedule_wake(WakePriority::Normal);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DispatchError, Dispatcher, PostErrorKind, PostedCallback, PostedWork, UiLoopBridge,
        WakePriority,
    };
    use crate::runtime::{HostRegistration, UiThreadContext};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier, Mutex, mpsc};
    use std::thread;

    #[derive(Default)]
    struct FakeBridge {
        wakes: AtomicUsize,
        priorities: Mutex<Vec<WakePriority>>,
    }

    impl FakeBridge {
        fn wake_count(&self) -> usize {
            self.wakes.load(Ordering::SeqCst)
        }
    }

    impl UiLoopBridge for FakeBridge {
        fn schedule_wake(&self, priority: WakePriority) {
            self.wakes.fetch_add(1, Ordering::SeqCst);
            self.priorities
                .lock()
                .expect("bridge priority list is healthy")
                .push(priority);
        }
    }

    fn new_dispatcher() -> (Dispatcher, Arc<FakeBridge>) {
        let bridge = Arc::new(FakeBridge::default());
        let dispatcher = Dispatcher::new(bridge.clone());
        (dispatcher, bridge)
    }

    #[test]
    fn normal_and_shutdown_work_use_distinct_host_wake_priorities() {
        let (normal_dispatcher, normal_bridge) = new_dispatcher();
        post_callback(&normal_dispatcher, Box::new(|| {})).expect("post is accepted");
        assert_eq!(
            *normal_bridge
                .priorities
                .lock()
                .expect("bridge priority list is healthy"),
            [WakePriority::Normal]
        );

        let (control_dispatcher, control_bridge) = new_dispatcher();
        control_dispatcher
            .request_shutdown()
            .expect("shutdown request is accepted");
        assert_eq!(
            *control_bridge
                .priorities
                .lock()
                .expect("bridge priority list is healthy"),
            [WakePriority::Control]
        );
    }

    fn post_callback(
        dispatcher: &Dispatcher,
        callback: PostedCallback,
    ) -> Result<(), super::PostError> {
        dispatcher.post(PostedWork::callback(callback))
    }

    #[test]
    fn injected_ui_thread_context_overrides_component_creation_thread() {
        let ui_thread = UiThreadContext::capture();
        let dispatcher = thread::spawn(move || {
            Dispatcher::new_for_ui_thread(Arc::new(FakeBridge::default()), ui_thread)
        })
        .join()
        .expect("dispatcher construction completes");

        assert_eq!(dispatcher.dispatch_turn(), Ok(false));

        let worker_dispatcher = dispatcher.clone();
        let worker_result = thread::spawn(move || worker_dispatcher.dispatch_turn())
            .join()
            .expect("worker dispatch attempt completes");
        assert_eq!(worker_result, Err(DispatchError::WrongThread));
    }

    #[test]
    fn fifo_single_item_turns_are_not_reentrant() {
        let (dispatcher, bridge) = new_dispatcher();
        let events = Arc::new(Mutex::new(Vec::new()));
        let depth = Arc::new(AtomicUsize::new(0));
        let maximum_depth = Arc::new(AtomicUsize::new(0));
        let (first_posted, first_posted_rx) = mpsc::channel();

        let first_dispatcher = dispatcher.clone();
        let first_events = Arc::clone(&events);
        let first_depth = Arc::clone(&depth);
        let first_maximum_depth = Arc::clone(&maximum_depth);
        let nested_dispatcher = first_dispatcher.clone();
        let first_worker = thread::spawn(move || {
            post_callback(
                &first_dispatcher,
                Box::new(move || {
                    let current_depth = first_depth.fetch_add(1, Ordering::SeqCst) + 1;
                    first_maximum_depth.fetch_max(current_depth, Ordering::SeqCst);
                    first_events.lock().expect("events lock").push("first");

                    let nested_events = Arc::clone(&first_events);
                    let nested_depth = Arc::clone(&first_depth);
                    let nested_maximum_depth = Arc::clone(&first_maximum_depth);
                    post_callback(
                        &nested_dispatcher,
                        Box::new(move || {
                            let current_depth = nested_depth.fetch_add(1, Ordering::SeqCst) + 1;
                            nested_maximum_depth.fetch_max(current_depth, Ordering::SeqCst);
                            nested_events.lock().expect("events lock").push("nested");
                            nested_depth.fetch_sub(1, Ordering::SeqCst);
                        }),
                    )
                    .expect("callback post accepted");
                    assert_eq!(first_events.lock().expect("events lock").len(), 1);
                    first_depth.fetch_sub(1, Ordering::SeqCst);
                }),
            )
            .expect("first post accepted");
            first_posted.send(()).expect("notify second worker");
        });

        let second_dispatcher = dispatcher.clone();
        let second_events = Arc::clone(&events);
        let second_worker = thread::spawn(move || {
            first_posted_rx.recv().expect("first post completes");
            post_callback(
                &second_dispatcher,
                Box::new(move || {
                    second_events.lock().expect("events lock").push("second");
                }),
            )
            .expect("second post accepted");
        });
        first_worker.join().expect("first worker completes");
        second_worker.join().expect("second worker completes");

        assert_eq!(bridge.wake_count(), 1);
        assert!(dispatcher.dispatch_turn().expect("first UI turn"));
        assert_eq!(*events.lock().expect("events lock"), ["first"]);
        assert!(dispatcher.dispatch_turn().expect("second UI turn"));
        assert!(dispatcher.dispatch_turn().expect("nested UI turn"));
        assert_eq!(
            *events.lock().expect("events lock"),
            ["first", "second", "nested"]
        );
        assert_eq!(maximum_depth.load(Ordering::SeqCst), 1);
        assert_eq!(bridge.wake_count(), 3);
        assert!(!dispatcher.dispatch_turn().expect("empty UI turn"));
    }

    #[test]
    fn posts_at_wake_and_drain_boundaries_are_not_lost() {
        let (dispatcher, bridge) = new_dispatcher();
        let completed = Arc::new(AtomicUsize::new(0));
        for _ in 0..2 {
            let completed = Arc::clone(&completed);
            post_callback(
                &dispatcher,
                Box::new(move || {
                    completed.fetch_add(1, Ordering::SeqCst);
                }),
            )
            .expect("post accepted");
        }
        assert_eq!(bridge.wake_count(), 1);

        let barrier = Arc::new(Barrier::new(2));
        let (posted, posted_rx) = mpsc::channel();
        let racing_dispatcher = dispatcher.clone();
        let racing_barrier = Arc::clone(&barrier);
        let racing_worker = thread::spawn(move || {
            racing_barrier.wait();
            post_callback(&racing_dispatcher, Box::new(|| {})).expect("racing post accepted");
            posted.send(()).expect("notify completed post");
        });

        assert!(
            dispatcher
                .dispatch_turn_with(
                    || {
                        barrier.wait();
                        posted_rx.recv().expect("racing post completes");
                    },
                    || {}
                )
                .expect("dispatch turn")
        );
        racing_worker.join().expect("racing worker completes");
        assert_eq!(completed.load(Ordering::SeqCst), 1);
        assert_eq!(bridge.wake_count(), 2);

        while dispatcher.dispatch_turn().expect("drain next turn") {}
        assert_eq!(completed.load(Ordering::SeqCst), 2);
        assert_eq!(bridge.wake_count(), 3);

        let wrong_thread_dispatcher = dispatcher.clone();
        assert_eq!(
            thread::spawn(move || wrong_thread_dispatcher.dispatch_turn())
                .join()
                .expect("wrong-thread check completes"),
            Err(DispatchError::WrongThread)
        );
    }

    #[test]
    fn post_failure_preserves_work_and_shutdown_destroys_accepted_work_once() {
        let (dispatcher, _) = new_dispatcher();
        let invocations = Arc::new(AtomicUsize::new(0));
        let destroys = Arc::new(Mutex::new(Vec::new()));
        let ui_thread = thread::current().id();

        dispatcher.fail_next_reservation();
        let failed_invocations = Arc::clone(&invocations);
        let failed_destroys = Arc::clone(&destroys);
        let error = dispatcher
            .post(PostedWork::new(
                Box::new(move || {
                    failed_invocations.fetch_add(1, Ordering::SeqCst);
                }),
                Box::new(move || {
                    failed_destroys
                        .lock()
                        .expect("destroys lock")
                        .push(thread::current().id());
                }),
            ))
            .expect_err("injected allocation failure");
        assert_eq!(error.kind, PostErrorKind::OutOfMemory);
        assert_eq!(invocations.load(Ordering::SeqCst), 0);
        assert!(destroys.lock().expect("destroys lock").is_empty());
        dispatcher
            .post(error.into_work())
            .expect("caller-owned work can be retried");
        assert!(dispatcher.dispatch_turn().expect("invoke posted callback"));
        assert_eq!(invocations.load(Ordering::SeqCst), 1);
        assert_eq!(*destroys.lock().expect("destroys lock"), [ui_thread]);

        let cancelled_invocations = Arc::clone(&invocations);
        let cancelled_destroys = Arc::clone(&destroys);
        dispatcher
            .post(PostedWork::new(
                Box::new(move || {
                    cancelled_invocations.fetch_add(1, Ordering::SeqCst);
                }),
                Box::new(move || {
                    cancelled_destroys
                        .lock()
                        .expect("destroys lock")
                        .push(thread::current().id());
                }),
            ))
            .expect("accepted work");
        dispatcher
            .close_and_cancel()
            .expect("shutdown cancels queued work");
        assert_eq!(invocations.load(Ordering::SeqCst), 1);
        assert_eq!(
            *destroys.lock().expect("destroys lock"),
            [ui_thread, ui_thread]
        );
        assert!(
            !dispatcher
                .dispatch_turn()
                .expect("closed dispatcher is idle")
        );
        dispatcher
            .close_and_cancel()
            .expect("shutdown is idempotent");

        let rejected_invocations = Arc::clone(&invocations);
        let rejected_destroys = Arc::clone(&destroys);
        let error = dispatcher
            .post(PostedWork::new(
                Box::new(move || {
                    rejected_invocations.fetch_add(1, Ordering::SeqCst);
                }),
                Box::new(move || {
                    rejected_destroys
                        .lock()
                        .expect("destroys lock")
                        .push(thread::current().id());
                }),
            ))
            .expect_err("closed dispatcher rejects post");
        assert_eq!(error.kind, PostErrorKind::Closed);
        assert_eq!(invocations.load(Ordering::SeqCst), 1);
        assert_eq!(destroys.lock().expect("destroys lock").len(), 2);
        error.into_work().destroy_by_caller();
        assert_eq!(destroys.lock().expect("destroys lock").len(), 3);
    }

    #[test]
    fn shutdown_request_precedes_queued_work_and_cancels_it() {
        let (dispatcher, bridge) = new_dispatcher();
        let host = HostRegistration::new();
        let outer_scope = host.begin_event().expect("outer host scope");
        let nested_scope = host.begin_event().expect("nested host scope");
        let events = Arc::new(Mutex::new(Vec::new()));
        let ui_thread = thread::current().id();

        let active_dispatcher = dispatcher.clone();
        let active_events = Arc::clone(&events);
        post_callback(
            &dispatcher,
            Box::new(move || {
                active_events
                    .lock()
                    .expect("events lock")
                    .push("callback-started");
                active_dispatcher
                    .request_shutdown()
                    .expect("shutdown request accepted");
                active_dispatcher
                    .request_shutdown()
                    .expect("duplicate shutdown request coalesced");
                let late_events = Arc::clone(&active_events);
                active_dispatcher
                    .post(PostedWork::new(
                        Box::new({
                            let events = Arc::clone(&late_events);
                            move || {
                                events
                                    .lock()
                                    .expect("events lock")
                                    .push("late-callback-invoked");
                            }
                        }),
                        Box::new(move || {
                            late_events
                                .lock()
                                .expect("events lock")
                                .push("late-callback-destroyed");
                        }),
                    ))
                    .expect("post remains accepted until shutdown begins");
                active_events
                    .lock()
                    .expect("events lock")
                    .push("callback-finished");
            }),
        )
        .expect("active callback accepted");

        let queued_events = Arc::clone(&events);
        dispatcher
            .post(PostedWork::new(
                Box::new(move || {
                    queued_events
                        .lock()
                        .expect("events lock")
                        .push("queued-callback");
                }),
                {
                    let events = Arc::clone(&events);
                    Box::new(move || {
                        let mut events = events.lock().expect("events lock");
                        events.push("queued-destroy");
                        events.push(if thread::current().id() == ui_thread {
                            "destroyed-on-ui"
                        } else {
                            "destroyed-off-ui"
                        });
                    })
                },
            ))
            .expect("ordinary queued callback accepted");
        let notification_events = Arc::clone(&events);
        dispatcher
            .post(PostedWork::new(
                Box::new(move || {
                    notification_events
                        .lock()
                        .expect("events lock")
                        .push("notification-delivered");
                }),
                {
                    let events = Arc::clone(&events);
                    Box::new(move || {
                        let mut events = events.lock().expect("events lock");
                        events.push("notification-destroyed");
                        events.push(if thread::current().id() == ui_thread {
                            "notification-destroyed-on-ui"
                        } else {
                            "notification-destroyed-off-ui"
                        });
                    })
                },
            ))
            .expect("pending notification work accepted");

        assert!(
            !host
                .end_event_and_dispatch_shutdown(nested_scope, &dispatcher, || {})
                .expect("nested boundary does not dispatch shutdown")
        );
        assert_eq!(*events.lock().expect("events lock"), Vec::<&str>::new());

        assert!(
            dispatcher
                .dispatch_turn_with(|| {}, || {})
                .expect("active callback turn")
        );
        assert_eq!(bridge.wake_count(), 2);
        assert_eq!(
            *events.lock().expect("events lock"),
            ["callback-started", "callback-finished"]
        );

        let shutdown_events = Arc::clone(&events);
        assert!(
            host.end_event_and_dispatch_shutdown(outer_scope, &dispatcher, move || {
                shutdown_events
                    .lock()
                    .expect("events lock")
                    .push("shutdown-handler");
            })
            .expect("outer boundary services priority shutdown")
        );
        assert_eq!(
            *events.lock().expect("events lock"),
            [
                "callback-started",
                "callback-finished",
                "queued-destroy",
                "destroyed-on-ui",
                "notification-destroyed",
                "notification-destroyed-on-ui",
                "late-callback-destroyed",
                "shutdown-handler"
            ]
        );
        assert!(
            !dispatcher
                .dispatch_turn()
                .expect("closed dispatcher has no further turns")
        );
    }
}

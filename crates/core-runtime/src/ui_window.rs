use crate::handle_registry::registry::ObjectIdentity;
use crate::platform::frame::{FrameError, FramePhase, FrameTicket, FrameTracker};
use crate::platform::scene::{Damage, SceneSnapshot};
use std::collections::HashMap;
use std::sync::Arc;

/// Platform-facing peer contract exposed only through the private Core/C-ABI support surface.
///
/// A peer owns one Window's native surface resources. Implementations must keep failed `open`
/// attempts retryable and must not mutate Core UI state.
pub trait PlatformWindowPeer: Send + Sync {
    fn open(&self) -> Result<(), PlatformOperationError>;
    fn close(&self) -> Result<(), PlatformOperationError>;
    fn submit_frame(
        &self,
        ticket: FrameTicket,
        scene: Arc<SceneSnapshot>,
    ) -> Result<(), PlatformOperationError>;
}

/// Posts a completed or failed backend frame to the Runtime's UI dispatcher.
pub trait PlatformFrameCompletionSink: Send + Sync {
    fn complete_frame(&self, ticket: FrameTicket) -> Result<(), PlatformOperationError>;
    fn fail_frame(&self, ticket: FrameTicket) -> Result<(), PlatformOperationError>;
}

/// Creates lightweight Window peers for one Runtime's selected platform profile.
///
/// Factory construction must not open a DRM device or allocate display resources; that work is
/// deferred until the returned peer's `open` method is called.
pub trait PlatformRuntimeFactory: Send + Sync {
    fn create_window_peer(
        &self,
        completion_sink: Arc<dyn PlatformFrameCompletionSink>,
    ) -> Result<Arc<dyn PlatformWindowPeer>, PlatformOperationError>;
}

/// Opaque failure at the platform SPI boundary; detailed diagnostics remain backend-owned.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformOperationError;

pub(crate) trait WindowPeerAdapter: Send + Sync {
    fn open(&self, window: ObjectIdentity) -> Result<(), ()>;
    fn close(&self, window: ObjectIdentity) -> Result<(), ()>;
    fn submit_frame(&self, ticket: FrameTicket, scene: Arc<SceneSnapshot>) -> Result<(), ()>;
}

pub(crate) struct FactoryWindowPeer(pub(crate) Arc<dyn PlatformWindowPeer>);

impl WindowPeerAdapter for FactoryWindowPeer {
    fn open(&self, _: ObjectIdentity) -> Result<(), ()> {
        self.0.open().map_err(|_| ())
    }

    fn close(&self, _: ObjectIdentity) -> Result<(), ()> {
        self.0.close().map_err(|_| ())
    }

    fn submit_frame(&self, ticket: FrameTicket, scene: Arc<SceneSnapshot>) -> Result<(), ()> {
        self.0.submit_frame(ticket, scene).map_err(|_| ())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloseDecision {
    Accept,
    Decline,
    Defer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WindowLifecycleError {
    Unsupported,
    InvalidState,
    StaleRequest,
    IdentifierExhausted,
    OutOfMemory,
    PeerFailure,
    Frame(FrameError),
}

#[derive(Default)]
pub(crate) struct WindowLifecycle {
    peer: Option<Arc<dyn WindowPeerAdapter>>,
    generation: u64,
    next_request: u64,
    open: bool,
    opening: bool,
    closing: bool,
    in_flight: Option<(u64, u64)>,
    pending: HashMap<u64, u64>,
    frames: FrameTracker,
    force_full_damage: bool,
}

pub(crate) struct PreparedWindowFrame {
    pub(crate) ticket: FrameTicket,
    pub(crate) peer: Arc<dyn WindowPeerAdapter>,
    pub(crate) scene: Arc<SceneSnapshot>,
}

impl WindowLifecycle {
    pub(crate) fn set_peer(&mut self, peer: Arc<dyn WindowPeerAdapter>) {
        self.peer = Some(peer);
    }

    pub(crate) fn has_peer(&self) -> bool {
        self.peer.is_some()
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    #[cfg(test)]
    pub(crate) fn open(&mut self, identity: ObjectIdentity) -> Result<u64, WindowLifecycleError> {
        let (generation, peer) = self.prepare_open()?;
        if let Some(peer) = peer {
            self.finish_open(generation, peer.open(identity))?;
        }
        Ok(generation)
    }

    pub(crate) fn prepare_open(
        &mut self,
    ) -> Result<(u64, Option<Arc<dyn WindowPeerAdapter>>), WindowLifecycleError> {
        if self.open {
            return Ok((self.generation, None));
        }
        if self.opening || self.closing {
            return Err(WindowLifecycleError::InvalidState);
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(WindowLifecycleError::IdentifierExhausted)?;
        let peer = Arc::clone(
            self.peer
                .as_ref()
                .ok_or(WindowLifecycleError::Unsupported)?,
        );
        self.opening = true;
        Ok((generation, Some(peer)))
    }

    pub(crate) fn finish_open(
        &mut self,
        generation: u64,
        result: Result<(), ()>,
    ) -> Result<(), WindowLifecycleError> {
        if !self.opening || generation != self.generation.checked_add(1).unwrap_or(0) {
            return Err(WindowLifecycleError::StaleRequest);
        }
        self.opening = false;
        result.map_err(|()| WindowLifecycleError::PeerFailure)?;
        self.generation = generation;
        self.open = true;
        self.frames.reset();
        self.force_full_damage = true;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn close(&mut self, identity: ObjectIdentity) -> Result<(), WindowLifecycleError> {
        if !self.open {
            return Ok(());
        }
        let peer = self.prepare_close()?;
        let result = peer.close(identity);
        self.finish_close(self.generation, result)
    }

    pub(crate) fn prepare_close(
        &mut self,
    ) -> Result<Arc<dyn WindowPeerAdapter>, WindowLifecycleError> {
        if !self.open || self.closing || self.in_flight.is_some() {
            return Err(WindowLifecycleError::InvalidState);
        }
        let peer = Arc::clone(
            self.peer
                .as_ref()
                .ok_or(WindowLifecycleError::Unsupported)?,
        );
        self.closing = true;
        Ok(peer)
    }

    pub(crate) fn finish_close(
        &mut self,
        generation: u64,
        result: Result<(), ()>,
    ) -> Result<(), WindowLifecycleError> {
        if !self.open || !self.closing || generation != self.generation {
            return Err(WindowLifecycleError::StaleRequest);
        }
        self.closing = false;
        result.map_err(|()| WindowLifecycleError::PeerFailure)?;
        self.open = false;
        self.pending.clear();
        self.in_flight = None;
        self.frames.reset();
        self.force_full_damage = true;
        Ok(())
    }

    pub(crate) fn begin_frame(
        &mut self,
        scene: Arc<SceneSnapshot>,
    ) -> Result<PreparedWindowFrame, FrameError> {
        if !self.open {
            return Err(FrameError::InvalidTransition);
        }
        let peer = self
            .peer
            .as_ref()
            .cloned()
            .ok_or(FrameError::InvalidTransition)?;
        let scene = if self.force_full_damage {
            Arc::new(scene.with_damage(Damage::Full))
        } else {
            scene
        };
        let ticket = self.frames.begin(self.generation, Arc::clone(&scene))?;
        self.frames.mark_submitted(ticket)?;
        self.force_full_damage = false;
        Ok(PreparedWindowFrame {
            ticket,
            peer,
            scene,
        })
    }

    pub(crate) fn complete_frame(&mut self, ticket: FrameTicket) -> bool {
        self.frames.complete(ticket)
    }

    pub(crate) fn fail_frame(&mut self, ticket: FrameTicket) -> bool {
        let failed = self.frames.fail(ticket);
        if failed {
            self.force_full_damage = true;
        }
        failed
    }

    pub(crate) fn frame_phase(&self) -> FramePhase {
        self.frames.phase()
    }

    #[cfg(test)]
    pub(crate) fn close_intent(
        &mut self,
        identity: ObjectIdentity,
        callback: Option<Box<dyn FnOnce(u64, u64) -> CloseDecision>>,
    ) -> Result<Option<(u64, u64)>, WindowLifecycleError> {
        let request = self.begin_close_intent()?;
        let decision = callback.map_or(CloseDecision::Accept, |callback| {
            callback(request.0, request.1)
        });
        let peer = self.complete_close_intent(request.0, request.1, decision)?;
        if let Some(peer) = peer {
            let result = peer.close(identity);
            self.finish_close_intent(request.1, result)?;
            Ok(None)
        } else if decision == CloseDecision::Defer {
            Ok(Some(request))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn begin_close_intent(&mut self) -> Result<(u64, u64), WindowLifecycleError> {
        if !self.open || self.closing || self.in_flight.is_some() {
            return Err(WindowLifecycleError::InvalidState);
        }
        self.pending
            .try_reserve(1)
            .map_err(|_| WindowLifecycleError::OutOfMemory)?;
        let request = self
            .next_request
            .checked_add(1)
            .ok_or(WindowLifecycleError::IdentifierExhausted)?;
        let generation = self.generation;
        self.next_request = request;
        self.in_flight = Some((request, generation));
        Ok((request, generation))
    }

    pub(crate) fn complete_close_intent(
        &mut self,
        request: u64,
        generation: u64,
        decision: CloseDecision,
    ) -> Result<Option<Arc<dyn WindowPeerAdapter>>, WindowLifecycleError> {
        if !self.open
            || self.generation != generation
            || self.in_flight != Some((request, generation))
        {
            return Err(WindowLifecycleError::StaleRequest);
        }
        self.in_flight = None;
        match decision {
            CloseDecision::Decline => Ok(None),
            CloseDecision::Defer => {
                self.pending.insert(request, generation);
                Ok(None)
            }
            CloseDecision::Accept => {
                self.closing = true;
                Ok(Some(Arc::clone(
                    self.peer
                        .as_ref()
                        .ok_or(WindowLifecycleError::Unsupported)?,
                )))
            }
        }
    }

    pub(crate) fn finish_close_intent(
        &mut self,
        generation: u64,
        result: Result<(), ()>,
    ) -> Result<(), WindowLifecycleError> {
        if self.generation != generation || !self.open || !self.closing {
            return Err(WindowLifecycleError::StaleRequest);
        }
        self.closing = false;
        result.map_err(|()| WindowLifecycleError::PeerFailure)?;
        self.open = false;
        self.pending.clear();
        self.frames.reset();
        self.force_full_damage = true;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn resolve(
        &mut self,
        identity: ObjectIdentity,
        request: u64,
        generation: u64,
        decision: CloseDecision,
    ) -> Result<(), WindowLifecycleError> {
        if let Some(peer) = self.prepare_resolve(request, generation, decision)? {
            let result = peer.close(identity);
            self.finish_close_intent(generation, result)?;
        }
        Ok(())
    }

    pub(crate) fn prepare_resolve(
        &mut self,
        request: u64,
        generation: u64,
        decision: CloseDecision,
    ) -> Result<Option<Arc<dyn WindowPeerAdapter>>, WindowLifecycleError> {
        if decision == CloseDecision::Defer {
            return Err(WindowLifecycleError::InvalidState);
        }
        if !self.open
            || self.closing
            || generation != self.generation
            || self.pending.get(&request) != Some(&generation)
        {
            return Err(WindowLifecycleError::StaleRequest);
        }
        if decision == CloseDecision::Decline {
            self.pending.remove(&request);
            return Ok(None);
        }
        self.closing = true;
        Ok(Some(Arc::clone(
            self.peer
                .as_ref()
                .ok_or(WindowLifecycleError::Unsupported)?,
        )))
    }

    pub(crate) fn release_unresolved(
        &mut self,
        request: u64,
        generation: u64,
    ) -> Result<(), WindowLifecycleError> {
        if self.pending.get(&request) != Some(&generation) {
            return Err(WindowLifecycleError::StaleRequest);
        }
        self.pending.remove(&request);
        Ok(())
    }

    pub(crate) fn shutdown(&mut self, identity: ObjectIdentity) {
        self.frames.reset();
        self.force_full_damage = true;
        if self.open {
            if let Some(peer) = &self.peer {
                let _ = peer.close(identity);
            }
        }
        self.open = false;
        self.pending.clear();
        self.in_flight = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{CloseDecision, WindowLifecycle, WindowLifecycleError, WindowPeerAdapter};
    use crate::handle_registry::registry::ObjectIdentity;
    use crate::platform::frame::FrameTicket;
    use crate::platform::scene::SceneSnapshot;
    use std::num::NonZeroU64;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakePeer {
        opens: AtomicUsize,
        closes: AtomicUsize,
        fail_open: bool,
        fail_close: bool,
    }

    impl WindowPeerAdapter for FakePeer {
        fn open(&self, _: ObjectIdentity) -> Result<(), ()> {
            self.opens.fetch_add(1, Ordering::Relaxed);
            if self.fail_open { Err(()) } else { Ok(()) }
        }

        fn close(&self, _: ObjectIdentity) -> Result<(), ()> {
            self.closes.fetch_add(1, Ordering::Relaxed);
            if self.fail_close { Err(()) } else { Ok(()) }
        }

        fn submit_frame(&self, _: FrameTicket, _: Arc<SceneSnapshot>) -> Result<(), ()> {
            Ok(())
        }
    }

    fn identity() -> ObjectIdentity {
        ObjectIdentity::new(1, NonZeroU64::new(1).expect("non-zero generation"))
    }

    fn peer(fail_open: bool, fail_close: bool) -> Arc<FakePeer> {
        Arc::new(FakePeer {
            opens: AtomicUsize::new(0),
            closes: AtomicUsize::new(0),
            fail_open,
            fail_close,
        })
    }

    #[test]
    fn open_failure_is_retryable_and_generation_advances_only_after_success() {
        let mut lifecycle = WindowLifecycle::default();
        assert_eq!(
            lifecycle.open(identity()),
            Err(WindowLifecycleError::Unsupported)
        );
        let failed_peer = peer(true, false);
        lifecycle.set_peer(failed_peer.clone());
        assert_eq!(
            lifecycle.open(identity()),
            Err(WindowLifecycleError::PeerFailure)
        );
        assert!(!lifecycle.is_open());

        let working_peer = peer(false, false);
        lifecycle.set_peer(working_peer.clone());
        assert_eq!(lifecycle.open(identity()), Ok(1));
        assert_eq!(lifecycle.open(identity()), Ok(1));
        lifecycle.close(identity()).expect("close peer");
        assert!(!lifecycle.is_open());
        assert_eq!(lifecycle.open(identity()), Ok(2));
        assert_eq!(working_peer.opens.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn close_intent_default_decline_defer_release_and_stale_generation_follow_contract() {
        let mut lifecycle = WindowLifecycle::default();
        let fake = peer(false, false);
        lifecycle.set_peer(fake.clone());
        lifecycle.open(identity()).expect("open Window");

        assert_eq!(lifecycle.close_intent(identity(), None), Ok(None));
        assert!(!lifecycle.is_open());
        lifecycle.open(identity()).expect("reopen Window");
        assert_eq!(
            lifecycle.close_intent(identity(), Some(Box::new(|_, _| CloseDecision::Decline))),
            Ok(None)
        );
        assert!(lifecycle.is_open());

        let deferred = lifecycle
            .close_intent(identity(), Some(Box::new(|_, _| CloseDecision::Defer)))
            .expect("defer close request")
            .expect("owned request identity");
        lifecycle
            .release_unresolved(deferred.0, deferred.1)
            .expect("release unresolved request means Decline");
        assert!(lifecycle.is_open());

        let stale = lifecycle
            .close_intent(identity(), Some(Box::new(|_, _| CloseDecision::Defer)))
            .expect("defer another request")
            .expect("owned request identity");
        lifecycle.close(identity()).expect("owner closes peer");
        let next_generation = lifecycle.open(identity()).expect("open next generation");
        assert_eq!(next_generation, stale.1 + 1);
        assert_eq!(
            lifecycle.resolve(identity(), stale.0, stale.1, CloseDecision::Accept),
            Err(WindowLifecycleError::StaleRequest)
        );
        assert!(lifecycle.is_open());
        assert_eq!(fake.closes.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn accepted_close_failure_keeps_the_peer_open_and_shutdown_tears_it_down() {
        let mut lifecycle = WindowLifecycle::default();
        let fake = peer(false, true);
        lifecycle.set_peer(fake.clone());
        lifecycle.open(identity()).expect("open Window");
        assert_eq!(
            lifecycle.close(identity()),
            Err(WindowLifecycleError::PeerFailure)
        );
        assert!(lifecycle.is_open());
        lifecycle.shutdown(identity());
        assert!(!lifecycle.is_open());
        assert_eq!(fake.closes.load(Ordering::Relaxed), 2);
    }
}

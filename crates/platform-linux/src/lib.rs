//! Linux DRM/KMS + GBM/EGL adapter with a GLES renderer and asynchronous frame completion.

use crate::renderer::PlanError;
use freemvvm_core::abi_support::{
    FrameTicket, PlatformFrameCompletionSink, PlatformOperationError, PlatformRuntimeFactory,
    PlatformWindowPeer, SceneSnapshot, WindowClientArea,
};
use std::sync::{Arc, Mutex, mpsc};

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod renderer;

#[cfg(target_os = "linux")]
mod device;
#[cfg(not(target_os = "linux"))]
mod device {
    pub(super) struct NativeSurface;

    pub(super) fn open_surface() -> std::io::Result<NativeSurface> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Linux DRM adapter is unavailable on this target",
        ))
    }
}

/// Creates a lazy Linux platform factory. DRM/EGL resources are opened per Window.
pub fn factory() -> Arc<dyn PlatformRuntimeFactory> {
    Arc::new(LinuxFactory)
}

struct LinuxFactory;

impl PlatformRuntimeFactory for LinuxFactory {
    fn create_window_peer(
        &self,
        completion_sink: Arc<dyn PlatformFrameCompletionSink>,
    ) -> Result<Arc<dyn PlatformWindowPeer>, PlatformOperationError> {
        Ok(Arc::new(
            LinuxPeer::new(Arc::new(NativeSurfaceOpener), completion_sink)
                .map_err(|_| PlatformOperationError)?,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
enum SurfaceSubmission {
    Complete,
    Pending,
}

trait OpenedSurface {
    fn client_area(&self) -> WindowClientArea;
    fn submit(
        &mut self,
        ticket: FrameTicket,
        scene: &SceneSnapshot,
    ) -> Result<SurfaceSubmission, PlanError>;
    fn wait_present(&mut self) -> Result<Option<FrameTicket>, FrameTicket>;
    fn close(&mut self) -> Result<(), ()>;
}

#[cfg(target_os = "linux")]
impl OpenedSurface for device::NativeSurface {
    fn client_area(&self) -> WindowClientArea {
        device::NativeSurface::client_area(self)
    }

    fn submit(
        &mut self,
        ticket: FrameTicket,
        scene: &SceneSnapshot,
    ) -> Result<SurfaceSubmission, PlanError> {
        self.render(ticket, scene)
            .map(|submission| match submission {
                device::Submission::Complete => SurfaceSubmission::Complete,
                device::Submission::Pending => SurfaceSubmission::Pending,
            })
    }

    fn wait_present(&mut self) -> Result<Option<FrameTicket>, FrameTicket> {
        device::NativeSurface::wait_present(self)
    }

    fn close(&mut self) -> Result<(), ()> {
        self.close_surface()
    }
}

#[cfg(not(target_os = "linux"))]
impl OpenedSurface for device::NativeSurface {
    fn client_area(&self) -> WindowClientArea {
        WindowClientArea {
            width_dip: 0.0,
            height_dip: 0.0,
        }
    }

    fn submit(
        &mut self,
        _: FrameTicket,
        _: &SceneSnapshot,
    ) -> Result<SurfaceSubmission, PlanError> {
        Err(PlanError::NativeFailure)
    }

    fn wait_present(&mut self) -> Result<Option<FrameTicket>, FrameTicket> {
        Ok(None)
    }

    fn close(&mut self) -> Result<(), ()> {
        Ok(())
    }
}

trait SurfaceOpener: Send + Sync {
    fn open(&self) -> Result<Box<dyn OpenedSurface>, ()>;
}

struct NativeSurfaceOpener;

impl SurfaceOpener for NativeSurfaceOpener {
    fn open(&self) -> Result<Box<dyn OpenedSurface>, ()> {
        device::open_surface()
            .map(|surface| Box::new(surface) as Box<dyn OpenedSurface>)
            .map_err(|_| ())
    }
}

struct LinuxPeer {
    commands: mpsc::Sender<WorkerCommand>,
    worker: Mutex<Option<std::thread::JoinHandle<()>>>,
}

enum WorkerCommand {
    Open(u64, mpsc::SyncSender<Result<WindowClientArea, ()>>),
    Close(mpsc::SyncSender<Result<(), ()>>),
    Submit(
        FrameTicket,
        Arc<SceneSnapshot>,
        mpsc::SyncSender<Result<(), ()>>,
    ),
    Shutdown,
}

impl LinuxPeer {
    fn new(
        opener: Arc<dyn SurfaceOpener>,
        sink: Arc<dyn PlatformFrameCompletionSink>,
    ) -> Result<Self, ()> {
        let (commands, receiver) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("freemvvm-linux-platform".into())
            .spawn(move || surface_worker(receiver, opener, sink))
            .map_err(|_| ())?;
        Ok(Self {
            commands,
            worker: Mutex::new(Some(worker)),
        })
    }

    fn request(
        &self,
        make_command: impl FnOnce(mpsc::SyncSender<Result<(), ()>>) -> WorkerCommand,
    ) -> Result<(), PlatformOperationError> {
        let (reply, result) = mpsc::sync_channel(0);
        self.commands
            .send(make_command(reply))
            .map_err(|_| PlatformOperationError)?;
        result
            .recv()
            .map_err(|_| PlatformOperationError)?
            .map_err(|_| PlatformOperationError)
    }
}

impl Drop for LinuxPeer {
    fn drop(&mut self) {
        let _ = self.commands.send(WorkerCommand::Shutdown);
        if let Ok(worker) = self.worker.get_mut() {
            if let Some(worker) = worker.take() {
                let _ = worker.join();
            }
        }
    }
}

impl PlatformWindowPeer for LinuxPeer {
    fn open(&self, generation: u64) -> Result<WindowClientArea, PlatformOperationError> {
        let (reply, result) = mpsc::sync_channel(0);
        self.commands
            .send(WorkerCommand::Open(generation, reply))
            .map_err(|_| PlatformOperationError)?;
        result
            .recv()
            .map_err(|_| PlatformOperationError)?
            .map_err(|_| PlatformOperationError)
    }

    fn close(&self) -> Result<(), PlatformOperationError> {
        self.request(WorkerCommand::Close)
    }

    fn submit_frame(
        &self,
        ticket: FrameTicket,
        scene: Arc<SceneSnapshot>,
    ) -> Result<(), PlatformOperationError> {
        self.request(|reply| WorkerCommand::Submit(ticket, scene, reply))
    }
}

fn surface_worker(
    commands: mpsc::Receiver<WorkerCommand>,
    opener: Arc<dyn SurfaceOpener>,
    sink: Arc<dyn PlatformFrameCompletionSink>,
) {
    let mut surface: Option<Box<dyn OpenedSurface>> = None;
    let mut frame_pending = false;
    let mut deferred_closes = Vec::new();
    let mut shutting_down = false;
    loop {
        if frame_pending {
            let completion = surface
                .as_mut()
                .expect("pending frame owns an open surface")
                .wait_present();
            match completion {
                Ok(Some(completed)) => {
                    if sink.complete_frame(completed).is_err() {
                        let _ = sink.fail_frame(completed);
                    }
                    frame_pending = false;
                }
                Ok(None) => {}
                Err(failed) => {
                    surface = None;
                    frame_pending = false;
                    let _ = sink.fail_frame(failed);
                }
            }
            if frame_pending {
                match commands.recv_timeout(std::time::Duration::from_millis(8)) {
                    Ok(WorkerCommand::Close(reply)) => deferred_closes.push(reply),
                    Ok(WorkerCommand::Shutdown) => shutting_down = true,
                    Ok(WorkerCommand::Open(_, reply)) => {
                        let _ = reply.send(Err(()));
                    }
                    Ok(WorkerCommand::Submit(_, _, reply)) => {
                        let _ = reply.send(Err(()));
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => shutting_down = true,
                }
                continue;
            }
        }
        if !deferred_closes.is_empty() {
            let result = match surface.as_mut() {
                Some(opened) => opened.close(),
                None => Ok(()),
            };
            if result.is_ok() {
                surface = None;
            }
            for reply in deferred_closes.drain(..) {
                let _ = reply.send(result);
            }
        }
        if shutting_down {
            if let Some(mut opened) = surface.take() {
                let _ = opened.close();
            }
            break;
        }
        let command = match commands.recv_timeout(std::time::Duration::from_millis(8)) {
            Ok(command) => command,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        match command {
            WorkerCommand::Open(_generation, reply) => {
                let result = if surface.is_some() {
                    surface
                        .as_ref()
                        .map(|opened| opened.client_area())
                        .ok_or(())
                } else {
                    opener.open().map(|opened| {
                        let client_area = opened.client_area();
                        surface = Some(opened);
                        client_area
                    })
                };
                let _ = reply.send(result);
            }
            WorkerCommand::Close(reply) => {
                let result = match surface.as_mut() {
                    Some(opened) => opened.close(),
                    None => Ok(()),
                };
                if result.is_ok() {
                    surface = None;
                }
                let _ = reply.send(result);
            }
            WorkerCommand::Submit(ticket, scene, reply) => {
                let result = match surface.as_mut() {
                    Some(opened) => opened.submit(ticket, &scene),
                    None => Err(PlanError::NativeFailure),
                };
                match result {
                    Ok(SurfaceSubmission::Complete) => {
                        let _ = reply.send(Ok(()));
                        if sink.complete_frame(ticket).is_err() {
                            let _ = sink.fail_frame(ticket);
                        }
                    }
                    Ok(SurfaceSubmission::Pending) => {
                        let _ = reply.send(Ok(()));
                        frame_pending = true;
                    }
                    Err(error) => {
                        if error == PlanError::NativeFailure {
                            surface = None;
                        }
                        let _ = reply.send(Err(()));
                    }
                }
            }
            WorkerCommand::Shutdown => {
                shutting_down = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemvvm_core::abi_support::{Damage, DeviceScale, DrawOp, FrameTracker};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakeOpener {
        attempts: AtomicUsize,
        closes: Arc<AtomicUsize>,
        failures_remaining: AtomicUsize,
        pending: bool,
        fail_present: bool,
        reject_glyphs: bool,
        polls_before_present: usize,
    }

    struct FakeSurface {
        closes: Arc<AtomicUsize>,
        pending: bool,
        fail_present: bool,
        reject_glyphs: bool,
        ticket: Option<FrameTicket>,
        pending_polls: usize,
        polls_before_present: usize,
    }

    impl OpenedSurface for FakeSurface {
        fn client_area(&self) -> WindowClientArea {
            WindowClientArea {
                width_dip: 800.0,
                height_dip: 600.0,
            }
        }

        fn submit(
            &mut self,
            ticket: FrameTicket,
            scene: &SceneSnapshot,
        ) -> Result<SurfaceSubmission, PlanError> {
            if self.reject_glyphs
                && scene
                    .operations()
                    .iter()
                    .any(|operation| matches!(operation, DrawOp::GlyphRun(_)))
            {
                return Err(PlanError::UnsupportedOperation);
            }
            if self.pending {
                self.ticket = Some(ticket);
                Ok(SurfaceSubmission::Pending)
            } else {
                Ok(SurfaceSubmission::Complete)
            }
        }

        fn wait_present(&mut self) -> Result<Option<FrameTicket>, FrameTicket> {
            if self.pending_polls < self.polls_before_present {
                self.pending_polls += 1;
                return Ok(None);
            }
            let ticket = self.ticket.take();
            if self.fail_present {
                Err(ticket.expect("pending fake frame"))
            } else {
                Ok(ticket)
            }
        }

        fn close(&mut self) -> Result<(), ()> {
            Ok(())
        }
    }

    impl Drop for FakeSurface {
        fn drop(&mut self) {
            self.closes.fetch_add(1, Ordering::SeqCst);
        }
    }

    impl SurfaceOpener for FakeOpener {
        fn open(&self) -> Result<Box<dyn OpenedSurface>, ()> {
            self.attempts.fetch_add(1, Ordering::SeqCst);
            if self
                .failures_remaining
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                    remaining.checked_sub(1)
                })
                .is_ok()
            {
                return Err(());
            }
            Ok(Box::new(FakeSurface {
                closes: Arc::clone(&self.closes),
                pending: self.pending,
                fail_present: self.fail_present,
                reject_glyphs: self.reject_glyphs,
                ticket: None,
                pending_polls: 0,
                polls_before_present: self.polls_before_present,
            }))
        }
    }

    enum Completion {
        Complete(FrameTicket),
        Failed(FrameTicket),
    }

    struct ProbeSink(mpsc::Sender<Completion>);

    impl PlatformFrameCompletionSink for ProbeSink {
        fn complete_frame(&self, ticket: FrameTicket) -> Result<(), PlatformOperationError> {
            self.0
                .send(Completion::Complete(ticket))
                .map_err(|_| PlatformOperationError)?;
            Ok(())
        }

        fn fail_frame(&self, ticket: FrameTicket) -> Result<(), PlatformOperationError> {
            self.0
                .send(Completion::Failed(ticket))
                .map_err(|_| PlatformOperationError)?;
            Ok(())
        }
    }

    fn fake_opener(
        closes: Arc<AtomicUsize>,
        failures_remaining: usize,
        pending: bool,
        fail_present: bool,
        reject_glyphs: bool,
        polls_before_present: usize,
    ) -> Arc<FakeOpener> {
        Arc::new(FakeOpener {
            attempts: AtomicUsize::new(0),
            closes,
            failures_remaining: AtomicUsize::new(failures_remaining),
            pending,
            fail_present,
            reject_glyphs,
            polls_before_present,
        })
    }

    fn scene(revision: u64, operations: Vec<DrawOp>) -> Arc<SceneSnapshot> {
        Arc::new(
            SceneSnapshot::new(
                revision,
                DeviceScale::new(1.0, 1.0).expect("valid test scale"),
                [0.0, 0.0, 0.0],
                Arc::from(operations),
                Damage::Full,
            )
            .expect("valid test scene"),
        )
    }

    fn ticket(scene: Arc<SceneSnapshot>) -> FrameTicket {
        let mut tracker = FrameTracker::default();
        let ticket = tracker.begin(1, scene).expect("begin frame");
        tracker
            .mark_submitted(ticket)
            .expect("mark frame submitted");
        ticket
    }

    fn peer_and_completions(
        opener: Arc<dyn SurfaceOpener>,
    ) -> (LinuxPeer, mpsc::Receiver<Completion>) {
        let (sender, receiver) = mpsc::channel();
        let peer = LinuxPeer::new(opener, Arc::new(ProbeSink(sender))).expect("spawn worker");
        (peer, receiver)
    }

    #[test]
    fn platform_device_open_failure_can_retry_and_close_releases_surface_once() {
        let closes = Arc::new(AtomicUsize::new(0));
        let opener = fake_opener(Arc::clone(&closes), 1, false, false, false, 0);
        let (peer, _) = peer_and_completions(opener.clone());

        assert!(peer.open(1).is_err());
        assert_eq!(
            peer.open(1).expect("open after retry"),
            WindowClientArea {
                width_dip: 800.0,
                height_dip: 600.0,
            }
        );
        assert!(peer.open(1).is_ok());
        assert_eq!(opener.attempts.load(Ordering::SeqCst), 2);
        assert_eq!(closes.load(Ordering::SeqCst), 0);
        peer.close().expect("close surface");
        peer.close().expect("repeated close is harmless");
        assert_eq!(closes.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn platform_present_completes_only_after_worker_submission_result() {
        let closes = Arc::new(AtomicUsize::new(0));
        let opener = fake_opener(Arc::clone(&closes), 0, false, false, false, 0);
        let (peer, completions) = peer_and_completions(opener);
        peer.open(1).expect("open fake surface");
        let scene = scene(7, vec![]);
        let ticket = ticket(Arc::clone(&scene));
        peer.submit_frame(ticket, scene)
            .expect("submit immediate frame");
        match completions.recv().expect("completion") {
            Completion::Complete(completed) => assert_eq!(completed, ticket),
            Completion::Failed(_) => panic!("unexpected failure"),
        }
    }

    #[test]
    fn platform_loss_page_flip_failure_fails_ticket_and_releases_surface() {
        let closes = Arc::new(AtomicUsize::new(0));
        let opener = fake_opener(Arc::clone(&closes), 0, true, true, false, 0);
        let (peer, completions) = peer_and_completions(opener);
        peer.open(1).expect("open fake surface");
        let scene = scene(8, vec![]);
        let ticket = ticket(Arc::clone(&scene));
        peer.submit_frame(ticket, scene).expect("queue page flip");
        match completions.recv().expect("failure completion") {
            Completion::Failed(failed) => assert_eq!(failed, ticket),
            Completion::Complete(_) => panic!("unexpected completion"),
        }
        assert_eq!(closes.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn unsupported_draw_operation_fails_submission_without_discarding_surface() {
        let closes = Arc::new(AtomicUsize::new(0));
        let opener = fake_opener(Arc::clone(&closes), 0, false, false, true, 0);
        let (peer, completions) = peer_and_completions(opener);
        peer.open(1).expect("open fake surface");
        let glyph = freemvvm_core::abi_support::GlyphRunSnapshot::new(
            [0; 32],
            12.0,
            freemvvm_core::abi_support::Rect::new(0.0, 0.0, 4.0, 4.0).expect("glyph bounds"),
            Arc::from([]),
        )
        .expect("glyph run");
        let unsupported = scene(9, vec![DrawOp::GlyphRun(glyph)]);
        let unsupported_ticket = ticket(Arc::clone(&unsupported));
        assert!(peer.submit_frame(unsupported_ticket, unsupported).is_err());
        assert_eq!(closes.load(Ordering::SeqCst), 0);

        let supported = scene(10, vec![]);
        let supported_ticket = ticket(Arc::clone(&supported));
        peer.submit_frame(supported_ticket, supported)
            .expect("surface remains usable");
        assert!(matches!(
            completions.recv().expect("completion"),
            Completion::Complete(completed) if completed == supported_ticket
        ));
    }

    #[test]
    fn platform_loss_close_during_pending_flip_waits_for_completion_before_cleanup() {
        let closes = Arc::new(AtomicUsize::new(0));
        let opener = fake_opener(Arc::clone(&closes), 0, true, false, false, 4);
        let (peer, completions) = peer_and_completions(opener);
        peer.open(1).expect("open fake surface");
        let scene = scene(11, vec![]);
        let ticket = ticket(Arc::clone(&scene));
        peer.submit_frame(ticket, scene).expect("queue page flip");
        peer.close().expect("deferred close");
        assert!(matches!(
            completions.recv().expect("completion before close result"),
            Completion::Complete(completed) if completed == ticket
        ));
        assert_eq!(closes.load(Ordering::SeqCst), 1);
    }
}

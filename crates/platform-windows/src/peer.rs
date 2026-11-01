use freemvvm_core::abi_support::{
    FrameTicket, PlatformFrameCompletionSink, PlatformOperationError, PlatformWindowClientAreaSink,
    PlatformWindowPeer, SceneSnapshot, WindowClientArea, WindowOpenInfo,
};
use std::sync::{Arc, Mutex};

pub(crate) trait NativeWindowBackend: Send {
    fn open(
        &mut self,
        generation: u64,
        sink: Option<Arc<dyn PlatformWindowClientAreaSink>>,
    ) -> Result<WindowOpenInfo, PlatformOperationError>;

    fn close(&mut self) -> Result<(), PlatformOperationError>;

    fn submit_frame(
        &mut self,
        ticket: FrameTicket,
        scene: Arc<SceneSnapshot>,
    ) -> Result<(), PlatformOperationError>;
}

struct PeerState<B> {
    backend: B,
    generation: Option<u64>,
}

pub(crate) struct WindowsPeer<B: NativeWindowBackend> {
    state: Mutex<PeerState<B>>,
    client_area_sink: Mutex<Option<Arc<dyn PlatformWindowClientAreaSink>>>,
    completion_sink: Mutex<Option<Arc<dyn PlatformFrameCompletionSink>>>,
}

impl<B: NativeWindowBackend> WindowsPeer<B> {
    pub(crate) fn new(backend: B) -> Self {
        Self {
            state: Mutex::new(PeerState {
                backend,
                generation: None,
            }),
            client_area_sink: Mutex::new(None),
            completion_sink: Mutex::new(None),
        }
    }

    pub(crate) fn set_completion_sink(&self, sink: Arc<dyn PlatformFrameCompletionSink>) {
        if let Ok(mut current) = self.completion_sink.lock() {
            *current = Some(sink);
        }
    }
}

impl<B: NativeWindowBackend + 'static> PlatformWindowPeer for WindowsPeer<B> {
    fn open(&self, generation: u64) -> Result<WindowOpenInfo, PlatformOperationError> {
        if generation == 0 {
            return Err(PlatformOperationError);
        }

        let sink = self
            .client_area_sink
            .lock()
            .map_err(|_| PlatformOperationError)?
            .clone();
        let mut state = self.state.lock().map_err(|_| PlatformOperationError)?;
        if state.generation.is_some() {
            return Err(PlatformOperationError);
        }

        let info = state.backend.open(generation, sink)?;
        if !valid_client_area(info.client_area) {
            let _ = state.backend.close();
            return Err(PlatformOperationError);
        }
        state.generation = Some(generation);
        Ok(info)
    }

    fn set_client_area_sink(&self, sink: Arc<dyn PlatformWindowClientAreaSink>) {
        if let Ok(mut current) = self.client_area_sink.lock() {
            *current = Some(sink);
        }
    }

    fn close(&self) -> Result<(), PlatformOperationError> {
        let mut state = self.state.lock().map_err(|_| PlatformOperationError)?;
        if state.generation.is_none() {
            return Ok(());
        }
        state.backend.close()?;
        state.generation = None;
        Ok(())
    }

    fn submit_frame(
        &self,
        ticket: FrameTicket,
        scene: Arc<SceneSnapshot>,
    ) -> Result<(), PlatformOperationError> {
        let mut state = self.state.lock().map_err(|_| PlatformOperationError)?;
        if state.generation != Some(ticket.surface_generation()) {
            return Err(PlatformOperationError);
        }

        state.backend.submit_frame(ticket, scene)?;
        self.completion_sink
            .lock()
            .map_err(|_| PlatformOperationError)?
            .as_ref()
            .ok_or(PlatformOperationError)?
            .complete_frame(ticket)
            .map_err(|_| PlatformOperationError)
    }
}

fn valid_client_area(area: WindowClientArea) -> bool {
    area.width_dip.is_finite()
        && area.height_dip.is_finite()
        && area.width_dip >= 0.0
        && area.height_dip >= 0.0
}

#[cfg(test)]
mod tests {
    use super::{NativeWindowBackend, WindowsPeer};
    use freemvvm_core::abi_support::{
        Damage, DeviceScale, DrawOp, FrameTicket, FrameTracker, PlatformFrameCompletionSink,
        PlatformOperationError, PlatformWindowClientAreaSink, PlatformWindowPeer, SceneSnapshot,
        WindowClientArea, WindowOpenInfo,
    };
    use std::collections::VecDeque;
    use std::sync::Arc;

    struct FakeBackend {
        opens: VecDeque<Result<WindowOpenInfo, PlatformOperationError>>,
        close_results: VecDeque<Result<(), PlatformOperationError>>,
        open_generations: Vec<u64>,
        closes: usize,
        drop_probe: Option<Arc<std::sync::atomic::AtomicUsize>>,
    }

    impl NativeWindowBackend for FakeBackend {
        fn open(
            &mut self,
            generation: u64,
            sink: Option<Arc<dyn PlatformWindowClientAreaSink>>,
        ) -> Result<WindowOpenInfo, PlatformOperationError> {
            self.open_generations.push(generation);
            let area = self
                .opens
                .pop_front()
                .unwrap_or(Err(PlatformOperationError))?;
            if let Some(sink) = sink {
                let _ = sink.client_area_changed(generation, area.client_area);
            }
            Ok(area)
        }

        fn close(&mut self) -> Result<(), PlatformOperationError> {
            self.closes += 1;
            self.close_results.pop_front().unwrap_or(Ok(()))
        }

        fn submit_frame(
            &mut self,
            _ticket: FrameTicket,
            _scene: Arc<SceneSnapshot>,
        ) -> Result<(), PlatformOperationError> {
            Ok(())
        }
    }

    impl Drop for FakeBackend {
        fn drop(&mut self) {
            if let Some(probe) = &self.drop_probe {
                probe.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    fn area() -> WindowClientArea {
        WindowClientArea {
            width_dip: 640.0,
            height_dip: 480.0,
        }
    }

    fn open_info() -> WindowOpenInfo {
        WindowOpenInfo {
            client_area: area(),
            device_scale: DeviceScale::new(1.25, 1.5).unwrap(),
        }
    }

    struct RecordingSink(Arc<std::sync::Mutex<Vec<(u64, WindowClientArea)>>>);

    impl PlatformWindowClientAreaSink for RecordingSink {
        fn client_area_changed(
            &self,
            generation: u64,
            area: WindowClientArea,
        ) -> Result<(), PlatformOperationError> {
            self.0.lock().unwrap().push((generation, area));
            Ok(())
        }
    }

    struct RecordingCompletionSink(Arc<std::sync::Mutex<Vec<FrameTicket>>>);

    impl PlatformFrameCompletionSink for RecordingCompletionSink {
        fn complete_frame(&self, ticket: FrameTicket) -> Result<(), PlatformOperationError> {
            self.0.lock().unwrap().push(ticket);
            Ok(())
        }

        fn fail_frame(&self, _: FrameTicket) -> Result<(), PlatformOperationError> {
            Err(PlatformOperationError)
        }
    }

    fn scene() -> Arc<SceneSnapshot> {
        Arc::new(
            SceneSnapshot::new(
                3,
                DeviceScale::new(1.0, 1.0).unwrap(),
                [0.0, 0.0, 0.0],
                Arc::<[DrawOp]>::from([]),
                Damage::Full,
            )
            .unwrap(),
        )
    }

    #[test]
    fn platform_windows_peer_open_failure_can_be_retried() {
        let peer = WindowsPeer::new(FakeBackend {
            opens: VecDeque::from([Err(PlatformOperationError), Ok(open_info())]),
            close_results: VecDeque::new(),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: None,
        });

        assert!(peer.open(1).is_err());
        assert_eq!(peer.open(2), Ok(open_info()));
        peer.close().unwrap();

        let state = peer.state.lock().unwrap();
        assert_eq!(state.backend.open_generations, [1, 2]);
        assert_eq!(state.backend.closes, 1);
        assert_eq!(state.generation, None);
    }

    #[test]
    fn platform_windows_peer_rejects_invalid_area_and_closes_partial_native_state() {
        let peer = WindowsPeer::new(FakeBackend {
            opens: VecDeque::from([Ok(WindowOpenInfo {
                client_area: WindowClientArea {
                    width_dip: f64::NAN,
                    height_dip: 10.0,
                },
                device_scale: DeviceScale::new(1.0, 1.0).unwrap(),
            })]),
            close_results: VecDeque::new(),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: None,
        });

        assert!(peer.open(1).is_err());
        let state = peer.state.lock().unwrap();
        assert_eq!(state.backend.closes, 1);
        assert_eq!(state.generation, None);
    }

    #[test]
    fn platform_windows_peer_close_keeps_core_peer_reopenable() {
        let peer = WindowsPeer::new(FakeBackend {
            opens: VecDeque::from([Ok(open_info()), Ok(open_info())]),
            close_results: VecDeque::new(),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: None,
        });

        assert_eq!(peer.open(7), Ok(open_info()));
        peer.close().unwrap();
        assert_eq!(peer.open(8), Ok(open_info()));
        peer.close().unwrap();

        let state = peer.state.lock().unwrap();
        assert_eq!(state.backend.open_generations, [7, 8]);
        assert_eq!(state.backend.closes, 2);
    }

    #[test]
    fn platform_windows_peer_rejects_zero_and_duplicate_generations() {
        let peer = WindowsPeer::new(FakeBackend {
            opens: VecDeque::from([Ok(open_info())]),
            close_results: VecDeque::new(),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: None,
        });

        assert!(peer.open(0).is_err());
        assert_eq!(peer.open(9), Ok(open_info()));
        assert!(peer.open(10).is_err());
    }

    #[test]
    fn platform_windows_peer_forwards_client_area_with_active_generation() {
        let peer = WindowsPeer::new(FakeBackend {
            opens: VecDeque::from([Ok(open_info())]),
            close_results: VecDeque::new(),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: None,
        });
        let received = Arc::new(std::sync::Mutex::new(Vec::new()));
        peer.set_client_area_sink(Arc::new(RecordingSink(Arc::clone(&received))));

        assert_eq!(peer.open(12), Ok(open_info()));
        assert_eq!(*received.lock().unwrap(), [(12, area())]);
    }

    #[test]
    fn platform_windows_render_completes_only_the_active_generation_ticket() {
        let peer = WindowsPeer::new(FakeBackend {
            opens: VecDeque::from([Ok(open_info())]),
            close_results: VecDeque::new(),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: None,
        });
        let completed = Arc::new(std::sync::Mutex::new(Vec::new()));
        peer.set_completion_sink(Arc::new(RecordingCompletionSink(Arc::clone(&completed))));
        peer.open(4).unwrap();

        let mut frames = FrameTracker::default();
        let ticket = frames.begin(4, scene()).unwrap();
        peer.submit_frame(ticket, scene()).unwrap();
        assert_eq!(*completed.lock().unwrap(), [ticket]);

        let stale = frames.begin(5, scene()).unwrap();
        assert!(peer.submit_frame(stale, scene()).is_err());
        assert_eq!(*completed.lock().unwrap(), [ticket]);
    }

    #[test]
    fn platform_windows_peer_close_failure_preserves_open_generation_for_retry() {
        let peer = WindowsPeer::new(FakeBackend {
            opens: VecDeque::from([Ok(open_info())]),
            close_results: VecDeque::from([Err(PlatformOperationError), Ok(())]),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: None,
        });

        assert_eq!(peer.open(15), Ok(open_info()));
        assert!(peer.close().is_err());
        assert_eq!(peer.state.lock().unwrap().generation, Some(15));
        peer.close().unwrap();
        assert_eq!(peer.state.lock().unwrap().generation, None);
    }

    #[test]
    fn platform_windows_peer_drop_releases_native_backend_owner() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let drops = Arc::new(AtomicUsize::new(0));
        let peer = WindowsPeer::new(FakeBackend {
            opens: VecDeque::from([Ok(open_info())]),
            close_results: VecDeque::new(),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: Some(Arc::clone(&drops)),
        });
        assert_eq!(peer.open(18), Ok(open_info()));

        drop(peer);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}

use freemvvm_core::abi_support::{
    FrameTicket, PlatformOperationError, PlatformWindowClientAreaSink, PlatformWindowPeer,
    SceneSnapshot, WindowClientArea,
};
use std::sync::{Arc, Mutex};

pub(crate) trait NativeWindowBackend: Send {
    fn open(
        &mut self,
        generation: u64,
        sink: Option<Arc<dyn PlatformWindowClientAreaSink>>,
    ) -> Result<WindowClientArea, PlatformOperationError>;

    fn close(&mut self) -> Result<(), PlatformOperationError>;
}

struct PeerState<B> {
    backend: B,
    generation: Option<u64>,
}

pub(crate) struct WindowsPeer<B: NativeWindowBackend> {
    state: Mutex<PeerState<B>>,
    client_area_sink: Mutex<Option<Arc<dyn PlatformWindowClientAreaSink>>>,
}

impl<B: NativeWindowBackend> WindowsPeer<B> {
    pub(crate) fn new(backend: B) -> Self {
        Self {
            state: Mutex::new(PeerState {
                backend,
                generation: None,
            }),
            client_area_sink: Mutex::new(None),
        }
    }
}

impl<B: NativeWindowBackend + 'static> PlatformWindowPeer for WindowsPeer<B> {
    fn open(&self, generation: u64) -> Result<WindowClientArea, PlatformOperationError> {
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

        let area = state.backend.open(generation, sink)?;
        if !valid_client_area(area) {
            let _ = state.backend.close();
            return Err(PlatformOperationError);
        }
        state.generation = Some(generation);
        Ok(area)
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
        _scene: Arc<SceneSnapshot>,
    ) -> Result<(), PlatformOperationError> {
        let state = self.state.lock().map_err(|_| PlatformOperationError)?;
        if state.generation != Some(ticket.surface_generation()) {
            return Err(PlatformOperationError);
        }

        // Rendering is added by UNIT-PLATFORM-WINDOWS-002. Never accept a frame without a renderer.
        Err(PlatformOperationError)
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
        PlatformOperationError, PlatformWindowClientAreaSink, PlatformWindowPeer, WindowClientArea,
    };
    use std::collections::VecDeque;
    use std::sync::Arc;

    struct FakeBackend {
        opens: VecDeque<Result<WindowClientArea, PlatformOperationError>>,
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
        ) -> Result<WindowClientArea, PlatformOperationError> {
            self.open_generations.push(generation);
            let area = self
                .opens
                .pop_front()
                .unwrap_or(Err(PlatformOperationError))?;
            if let Some(sink) = sink {
                let _ = sink.client_area_changed(generation, area);
            }
            Ok(area)
        }

        fn close(&mut self) -> Result<(), PlatformOperationError> {
            self.closes += 1;
            self.close_results.pop_front().unwrap_or(Ok(()))
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

    #[test]
    fn platform_windows_peer_open_failure_can_be_retried() {
        let peer = WindowsPeer::new(FakeBackend {
            opens: VecDeque::from([Err(PlatformOperationError), Ok(area())]),
            close_results: VecDeque::new(),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: None,
        });

        assert!(peer.open(1).is_err());
        assert_eq!(peer.open(2), Ok(area()));
        peer.close().unwrap();

        let state = peer.state.lock().unwrap();
        assert_eq!(state.backend.open_generations, [1, 2]);
        assert_eq!(state.backend.closes, 1);
        assert_eq!(state.generation, None);
    }

    #[test]
    fn platform_windows_peer_rejects_invalid_area_and_closes_partial_native_state() {
        let peer = WindowsPeer::new(FakeBackend {
            opens: VecDeque::from([Ok(WindowClientArea {
                width_dip: f64::NAN,
                height_dip: 10.0,
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
            opens: VecDeque::from([Ok(area()), Ok(area())]),
            close_results: VecDeque::new(),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: None,
        });

        assert_eq!(peer.open(7), Ok(area()));
        peer.close().unwrap();
        assert_eq!(peer.open(8), Ok(area()));
        peer.close().unwrap();

        let state = peer.state.lock().unwrap();
        assert_eq!(state.backend.open_generations, [7, 8]);
        assert_eq!(state.backend.closes, 2);
    }

    #[test]
    fn platform_windows_peer_rejects_zero_and_duplicate_generations() {
        let peer = WindowsPeer::new(FakeBackend {
            opens: VecDeque::from([Ok(area())]),
            close_results: VecDeque::new(),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: None,
        });

        assert!(peer.open(0).is_err());
        assert_eq!(peer.open(9), Ok(area()));
        assert!(peer.open(10).is_err());
    }

    #[test]
    fn platform_windows_peer_forwards_client_area_with_active_generation() {
        let peer = WindowsPeer::new(FakeBackend {
            opens: VecDeque::from([Ok(area())]),
            close_results: VecDeque::new(),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: None,
        });
        let received = Arc::new(std::sync::Mutex::new(Vec::new()));
        peer.set_client_area_sink(Arc::new(RecordingSink(Arc::clone(&received))));

        assert_eq!(peer.open(12), Ok(area()));
        assert_eq!(*received.lock().unwrap(), [(12, area())]);
    }

    #[test]
    fn platform_windows_peer_close_failure_preserves_open_generation_for_retry() {
        let peer = WindowsPeer::new(FakeBackend {
            opens: VecDeque::from([Ok(area())]),
            close_results: VecDeque::from([Err(PlatformOperationError), Ok(())]),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: None,
        });

        assert_eq!(peer.open(15), Ok(area()));
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
            opens: VecDeque::from([Ok(area())]),
            close_results: VecDeque::new(),
            open_generations: Vec::new(),
            closes: 0,
            drop_probe: Some(Arc::clone(&drops)),
        });
        assert_eq!(peer.open(18), Ok(area()));

        drop(peer);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}

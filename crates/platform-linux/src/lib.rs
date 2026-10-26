//! Linux DRM/KMS + GBM/EGL adapter. Rendering and presentation are implemented separately.

use freemvvm_core::abi_support::{
    FrameTicket, PlatformFrameCompletionSink, PlatformOperationError, PlatformRuntimeFactory,
    PlatformWindowPeer, SceneSnapshot,
};
use std::sync::{Arc, Mutex};

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
        _: Arc<dyn PlatformFrameCompletionSink>,
    ) -> Result<Arc<dyn PlatformWindowPeer>, PlatformOperationError> {
        Ok(Arc::new(LinuxPeer::new(Arc::new(NativeSurfaceOpener))))
    }
}

trait OpenedSurface: Send {}

impl OpenedSurface for device::NativeSurface {}

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
    opener: Arc<dyn SurfaceOpener>,
    surface: Mutex<Option<Box<dyn OpenedSurface>>>,
}

impl LinuxPeer {
    fn new(opener: Arc<dyn SurfaceOpener>) -> Self {
        Self {
            opener,
            surface: Mutex::new(None),
        }
    }
}

impl PlatformWindowPeer for LinuxPeer {
    fn open(&self) -> Result<(), PlatformOperationError> {
        let mut surface = self.surface.lock().map_err(|_| PlatformOperationError)?;
        if surface.is_none() {
            *surface = Some(self.opener.open().map_err(|_| PlatformOperationError)?);
        }
        Ok(())
    }

    fn close(&self) -> Result<(), PlatformOperationError> {
        self.surface
            .lock()
            .map_err(|_| PlatformOperationError)?
            .take();
        Ok(())
    }

    fn submit_frame(
        &self,
        _: FrameTicket,
        _: Arc<SceneSnapshot>,
    ) -> Result<(), PlatformOperationError> {
        // A surface is not a presented frame. The GLES renderer is a later implementation unit.
        Err(PlatformOperationError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakeOpener {
        attempts: AtomicUsize,
        closes: Arc<AtomicUsize>,
        failures_remaining: AtomicUsize,
    }

    struct FakeSurface(Arc<AtomicUsize>);

    impl OpenedSurface for FakeSurface {}

    impl Drop for FakeSurface {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
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
            Ok(Box::new(FakeSurface(Arc::clone(&self.closes))))
        }
    }

    #[test]
    fn platform_device_open_failure_can_retry_and_close_releases_surface_once() {
        let closes = Arc::new(AtomicUsize::new(0));
        let opener = Arc::new(FakeOpener {
            attempts: AtomicUsize::new(0),
            closes: Arc::clone(&closes),
            failures_remaining: AtomicUsize::new(1),
        });
        let peer = LinuxPeer::new(opener.clone());

        assert!(peer.open().is_err());
        assert!(peer.open().is_ok());
        assert!(peer.open().is_ok());
        assert_eq!(opener.attempts.load(Ordering::SeqCst), 2);
        assert_eq!(closes.load(Ordering::SeqCst), 0);
        peer.close().expect("close surface");
        peer.close().expect("repeated close is harmless");
        assert_eq!(closes.load(Ordering::SeqCst), 1);
    }
}

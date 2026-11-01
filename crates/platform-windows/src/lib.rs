//! Windows Win32 window peer for the Core Platform SPI.

mod peer;

#[cfg(target_os = "windows")]
mod win32;
#[cfg(not(target_os = "windows"))]
mod win32 {
    use crate::peer::NativeWindowBackend;
    use freemvvm_core::abi_support::{PlatformOperationError, WindowClientArea};
    use std::sync::Arc;

    pub(super) struct Backend;

    impl Backend {
        pub(super) fn new() -> Self {
            Self
        }
    }

    impl NativeWindowBackend for Backend {
        fn open(
            &mut self,
            _generation: u64,
            _sink: Option<Arc<dyn freemvvm_core::abi_support::PlatformWindowClientAreaSink>>,
        ) -> Result<WindowClientArea, PlatformOperationError> {
            Err(PlatformOperationError)
        }

        fn close(&mut self) -> Result<(), PlatformOperationError> {
            Ok(())
        }
    }
}

use freemvvm_core::abi_support::{
    PlatformFrameCompletionSink, PlatformOperationError, PlatformRuntimeFactory, PlatformWindowPeer,
};
use std::sync::Arc;

/// Creates a lazy Windows factory. No native Window is created before `Window.open()`.
pub fn factory() -> Arc<dyn PlatformRuntimeFactory> {
    Arc::new(WindowsFactory)
}

struct WindowsFactory;

impl PlatformRuntimeFactory for WindowsFactory {
    fn create_window_peer(
        &self,
        _completion_sink: Arc<dyn PlatformFrameCompletionSink>,
    ) -> Result<Arc<dyn PlatformWindowPeer>, PlatformOperationError> {
        Ok(Arc::new(peer::WindowsPeer::new(win32::Backend::new())))
    }
}

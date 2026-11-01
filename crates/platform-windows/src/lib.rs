//! Windows Win32 window peer for the Core Platform SPI.

mod peer;
#[cfg(any(target_os = "windows", test))]
mod renderer;
mod text;

#[cfg(target_os = "windows")]
mod win32;
#[cfg(not(target_os = "windows"))]
mod win32 {
    use crate::peer::NativeWindowBackend;
    use freemvvm_core::abi_support::{
        PlatformOperationError, ResolvedTextSnapshot, TextRequest, TextServiceError, WindowOpenInfo,
    };
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
        ) -> Result<WindowOpenInfo, PlatformOperationError> {
            Err(PlatformOperationError)
        }

        fn close(&mut self) -> Result<(), PlatformOperationError> {
            Ok(())
        }

        fn submit_frame(
            &mut self,
            _ticket: freemvvm_core::abi_support::FrameTicket,
            _scene: Arc<freemvvm_core::abi_support::SceneSnapshot>,
        ) -> Result<(), PlatformOperationError> {
            Err(PlatformOperationError)
        }

        fn resolve_text(
            &mut self,
            _request: &TextRequest,
        ) -> Result<ResolvedTextSnapshot, TextServiceError> {
            Err(TextServiceError::Unavailable)
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
        completion_sink: Arc<dyn PlatformFrameCompletionSink>,
    ) -> Result<Arc<dyn PlatformWindowPeer>, PlatformOperationError> {
        let peer = peer::WindowsPeer::new(win32::Backend::new());
        peer.set_completion_sink(completion_sink);
        Ok(Arc::new(peer))
    }
}

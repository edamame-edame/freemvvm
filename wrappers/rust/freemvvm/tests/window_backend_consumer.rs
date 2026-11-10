use freemvvm::{UiRuntime, Window};

#[cfg(target_os = "windows")]
const ACTIVE_BACKEND: &str = "windows.win32-d2d";
#[cfg(target_os = "linux")]
const ACTIVE_BACKEND: &str = "linux.drm-gbm-egl-gles";

#[test]
fn public_window_creation_accepts_the_compiled_presentation_backend() {
    let mut runtime = UiRuntime::try_create().expect("native Runtime creation");
    {
        let ui = runtime.ui_context().expect("UI context");
        let mut default_window = Window::create(&ui).expect("default Window creation");
        let mut selected_window =
            Window::create_with_presentation_backend(&ui, Some(ACTIVE_BACKEND))
                .expect("selected Window creation");

        assert!(
            Window::create_with_presentation_backend(&ui, Some("unknown.backend")).is_err(),
            "unknown backend must be rejected"
        );

        selected_window.close().expect("release selected Window");
        default_window.close().expect("release default Window");
    }
    runtime.close().expect("Runtime shutdown");
}

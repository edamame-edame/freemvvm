use freemvvm::{RgbColor, UiRuntime, Window};

#[test]
fn public_window_background_api_round_trips_through_the_c_abi() {
    let mut runtime = UiRuntime::try_create().expect("native Runtime creation");
    {
        let ui = runtime.ui_context().expect("UI context");
        let mut window = Window::create(&ui).expect("Window creation");
        assert_eq!(
            window.background_rgb(&ui).expect("default background"),
            RgbColor {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
            }
        );

        let requested = RgbColor {
            red: 0.2,
            green: 0.4,
            blue: 0.8,
        };
        window
            .set_background_rgb(&ui, requested)
            .expect("set background");
        assert_eq!(
            window
                .background_rgb(&ui)
                .expect("read assigned background"),
            requested
        );

        window.reset_background_rgb(&ui).expect("reset background");
        assert_eq!(
            window.background_rgb(&ui).expect("read reset background"),
            RgbColor {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
            }
        );
        window.close().expect("release Window owner");
    }
    runtime.close().expect("Runtime shutdown");
}

use freemvvm::{
    Button, InputEvent, InputEventMask, StackPanel, Text, TextBox, UiError, UiRuntime, Widget,
    Window,
};

fn assert_public_event_api() {
    let _key = InputEventMask::KEY_DOWN | InputEventMask::KEY_UP;
    let _all = InputEventMask::ALL;
    let _callback_shape = |event: &mut InputEvent<'_>| match event {
        InputEvent::KeyDown { key, modifiers, .. } | InputEvent::KeyUp { key, modifiers, .. } => {
            let _physical_key = key.0;
            let _shift_down = modifiers.0 & freemvvm::Modifiers::SHIFT != 0;
        }
        InputEvent::TextCommit { text, .. } => {
            let _owned_copy = text.to_owned();
        }
        InputEvent::PrimaryPointerDown { position, .. }
        | InputEvent::PrimaryPointerUp { position, .. } => {
            let _position = (position.x, position.y);
        }
    };
}

fn subscribe_window(
    widget: &Window,
    ui: &freemvvm::UiContext<'_>,
) -> Result<freemvvm::Subscription, UiError> {
    widget.subscribe_input(ui, InputEventMask::KEY_DOWN, |_| {})
}

fn subscribe_panel(
    widget: &StackPanel,
    ui: &freemvvm::UiContext<'_>,
) -> Result<freemvvm::Subscription, UiError> {
    widget.subscribe_input(ui, InputEventMask::KEY_UP, |_| {})
}

fn subscribe_text_box(
    widget: &TextBox,
    ui: &freemvvm::UiContext<'_>,
) -> Result<freemvvm::Subscription, UiError> {
    widget.subscribe_input(ui, InputEventMask::TEXT_COMMIT, |_| {})
}

fn subscribe_text(
    widget: &Text,
    ui: &freemvvm::UiContext<'_>,
) -> Result<freemvvm::Subscription, UiError> {
    widget.subscribe_input(ui, InputEventMask::PRIMARY_POINTER_DOWN, |_| {})
}

fn subscribe_button(
    widget: &Button,
    ui: &freemvvm::UiContext<'_>,
) -> Result<freemvvm::Subscription, UiError> {
    widget.subscribe_input(ui, InputEventMask::PRIMARY_POINTER_UP, |_| {})
}

fn subscribe_widget(
    widget: &Widget,
    ui: &freemvvm::UiContext<'_>,
) -> Result<freemvvm::Subscription, UiError> {
    widget.subscribe_input(ui, InputEventMask::ALL, |_| {})
}

#[test]
fn public_input_subscriptions_register_and_cancel_through_the_c_abi() {
    assert_public_event_api();
    let mut runtime = UiRuntime::try_create().expect("native Runtime creation");
    {
        let ui = runtime.ui_context().expect("UI context");
        let mut window = Window::create(&ui).expect("Window creation");
        let mut panel = StackPanel::create(&ui).expect("StackPanel creation");
        let mut text_box = TextBox::create(&ui, "input").expect("TextBox creation");
        let mut text = Text::create(&ui, "text").expect("Text creation");
        let mut button = Button::create(&ui, "button").expect("Button creation");

        panel.insert(&ui, 0, &text).expect("insert Text child");
        window.set_content(&ui, &panel).expect("set Window content");
        let mut generic_child = panel
            .child_at(&ui, 0)
            .expect("query child")
            .expect("child exists");

        for mut subscription in [
            subscribe_window(&window, &ui).expect("Window subscription"),
            subscribe_panel(&panel, &ui).expect("StackPanel subscription"),
            subscribe_text_box(&text_box, &ui).expect("TextBox subscription"),
            subscribe_text(&text, &ui).expect("Text subscription"),
            subscribe_button(&button, &ui).expect("Button subscription"),
            subscribe_widget(&generic_child, &ui).expect("Widget subscription"),
        ] {
            subscription.cancel(&ui).expect("cancel subscription");
            subscription.close().expect("release Subscription owner");
        }

        generic_child.close().expect("release generic Widget");
        button.close().expect("release Button");
        text.close().expect("release Text");
        text_box.close().expect("release TextBox");
        panel.close().expect("release StackPanel");
        window.close().expect("release Window");
    }
    runtime.close().expect("Runtime shutdown");
}

use freemvvm::{
    AsyncCommand, BindingMode, Button, ChildAlign, Orientation, StackPanel, Text, TextBox,
    UiRuntime, WidgetRef, Window,
};

fn assert_widget_ref<T: WidgetRef>() {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert_widget_ref::<Window>();
    assert_widget_ref::<StackPanel>();
    assert_widget_ref::<Text>();
    assert_widget_ref::<TextBox>();
    assert_widget_ref::<Button>();

    let mut runtime = UiRuntime::try_create()?;
    {
        let ui = runtime.ui_context()?;
        let mut window = Window::create(&ui)?;
        let mut panel = StackPanel::create(&ui)?;
        let mut text_box = TextBox::create(&ui, "入力")?;
        let mut text = Text::create(&ui, "表示")?;
        let mut button = Button::create(&ui, "実行")?;

        panel.set_orientation(&ui, Orientation::Vertical)?;
        panel.insert(&ui, 0, &text_box)?;
        panel.insert(&ui, 1, &text)?;
        panel.insert(&ui, 2, &button)?;
        panel.set_child_align(&ui, &button, ChildAlign::Center)?;
        window.set_content(&ui, &panel)?;
        assert_eq!(panel.child_count(&ui)?, 3);
        let mut current_content = window.content(&ui)?.expect("window content");
        let mut first_child = panel.child_at(&ui, 0)?.expect("first child");

        let mut value = text_box.value_property(&ui)?;
        let mut content = text.content_property(&ui)?;
        let mut binding = value
            .bind_to(&ui, &content, BindingMode::OneWay)?
            .expect("persistent binding owner");
        assert_eq!(content.get(&ui)?, "入力");
        value.set(&ui, "更新".into())?;
        let mut title = window.title_property(&ui)?;
        let mut enabled = button.is_enabled_property(&ui)?;
        let mut accessible_name = button.accessible_name_property(&ui)?;
        title.set(&ui, "Rust controls consumer".into())?;
        assert!(enabled.get(&ui)?);
        accessible_name.set(&ui, "submit".into())?;

        let mut command = AsyncCommand::create(&ui, None, |_operation| {})?;
        button.set_command(&ui, Some(&command))?;
        let associated = button.command(&ui)?.expect("associated command");
        drop(associated);

        // Exercise safe Focus API type-checking without assuming a native peer exists.
        let _focus_api: fn(&Button, &freemvvm::UiContext<'_>) -> Result<(), freemvvm::UiError> =
            WidgetRef::request_focus;
        command.close(&ui)?;
        binding.close()?;
        accessible_name.close()?;
        enabled.close()?;
        title.close()?;
        content.close()?;
        value.close()?;
        first_child.close()?;
        current_content.close()?;
        button.close()?;
        text.close()?;
        text_box.close()?;
        panel.close()?;
        window.close()?;
    }
    runtime.close()?;
    Ok(())
}

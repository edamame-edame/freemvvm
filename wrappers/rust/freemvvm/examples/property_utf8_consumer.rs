use freemvvm::{Property, UiRuntime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = UiRuntime::try_create()?;
    {
        let ui = runtime.ui_context()?;
        let mut text = Property::try_new(&ui, String::from("初期🦀\0値"))?;
        assert_eq!(text.get(&ui)?, "初期🦀\0値");

        text.set(&ui, String::new())?;
        assert_eq!(text.get(&ui)?, "");

        let updated = String::from("更新した日本語🙂\0UTF-8値");
        text.set(&ui, updated.clone())?;
        assert_eq!(text.get(&ui)?, updated);

        text.close()?;
    }
    runtime.close()?;
    Ok(())
}

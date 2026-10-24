use freemvvm::{Property, UiRuntime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = UiRuntime::try_create()?;
    {
        let ui = runtime.ui_context()?;
        let mut enabled = Property::try_new(&ui, true)?;
        assert!(enabled.get(&ui)?);
        enabled.set(&ui, false)?;
        assert!(!enabled.get(&ui)?);

        let mut count = Property::try_new(&ui, i64::MIN)?;
        assert_eq!(count.get(&ui)?, i64::MIN);
        count.set(&ui, i64::MAX)?;
        assert_eq!(count.get(&ui)?, i64::MAX);

        enabled.close()?;
        count.close()?;
    }
    runtime.close()?;
    Ok(())
}

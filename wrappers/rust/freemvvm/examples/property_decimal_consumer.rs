use freemvvm::{Decimal64, Property, UiRuntime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = UiRuntime::try_create()?;
    {
        let ui = runtime.ui_context()?;

        let one_hundredth = Decimal64::try_from_parts(false, 2, 100)?;
        let mut decimal = Property::try_new(&ui, one_hundredth)?;
        let one_tenth = Decimal64::try_from_parts(false, 1, 10)?;
        decimal.set(&ui, one_tenth)?;
        let stored = decimal.get(&ui)?;
        assert!(!stored.is_negative());
        assert_eq!(stored.scale(), 2);
        assert_eq!(stored.coefficient(), 100);
        decimal.close()?;

        let negative_zero = Decimal64::try_from_parts(true, 7, 0)?;
        let mut zero = Property::try_new(&ui, negative_zero)?;
        zero.set(&ui, Decimal64::try_from_parts(false, 0, 0)?)?;
        let stored_zero = zero.get(&ui)?;
        assert!(stored_zero.is_negative());
        assert_eq!(stored_zero.scale(), 7);
        assert_eq!(stored_zero.coefficient(), 0);
        zero.close()?;

        let maximum = Decimal64::try_from_parts(false, 31, (1_u64 << 58) - 1)?;
        let mut boundary = Property::try_new(&ui, maximum)?;
        let stored_boundary = boundary.get(&ui)?;
        assert_eq!(stored_boundary.scale(), 31);
        assert_eq!(stored_boundary.coefficient(), (1_u64 << 58) - 1);
        boundary.close()?;
    }
    runtime.close()?;
    Ok(())
}

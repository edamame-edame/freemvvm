use freemvvm::{Property, UiRuntime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = UiRuntime::try_create()?;
    {
        let ui = runtime.ui_context()?;

        let mut finite = Property::try_new(&ui, -13.25_f64)?;
        assert_eq!(finite.get(&ui)?.to_bits(), (-13.25_f64).to_bits());
        finite.set(&ui, f64::INFINITY)?;
        assert_eq!(finite.get(&ui)?, f64::INFINITY);
        finite.set(&ui, f64::NEG_INFINITY)?;
        assert_eq!(finite.get(&ui)?, f64::NEG_INFINITY);
        finite.close()?;

        let initial_nan = f64::from_bits(0x7ff8_0000_0000_0011);
        let mut nan = Property::try_new(&ui, initial_nan)?;
        nan.set(&ui, f64::from_bits(0x7ff8_0000_0000_0022))?;
        assert_eq!(nan.get(&ui)?.to_bits(), initial_nan.to_bits());
        nan.close()?;

        let mut negative_zero = Property::try_new(&ui, -0.0_f64)?;
        negative_zero.set(&ui, 0.0_f64)?;
        assert_eq!(negative_zero.get(&ui)?.to_bits(), (-0.0_f64).to_bits());
        negative_zero.close()?;
    }
    runtime.close()?;
    Ok(())
}

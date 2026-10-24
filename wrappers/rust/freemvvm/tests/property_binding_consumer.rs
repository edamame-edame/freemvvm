use freemvvm::{
    Binding, BindingDirection, BindingError, BindingMode, Property, UiContext, UiError,
};

#[allow(dead_code)]
fn public_binding_api(
    source: &Property<i64>,
    target: &Property<i64>,
    ui: &UiContext<'_>,
) -> Result<Option<Binding<i64>>, UiError> {
    source.bind_to(ui, target, BindingMode::OneWay)
}

#[allow(dead_code)]
fn public_binding_operations(
    binding: &mut Binding<i64>,
    ui: &UiContext<'_>,
) -> Result<Option<BindingError<i64>>, UiError> {
    let snapshot = binding.error_snapshot(ui)?;
    if let Some(error) = &snapshot {
        let _ = error.error().raw_status();
        let _ = error.rejected_property();
        let _ = error.direction();
    }
    binding.subscribe_error(ui, || {})?;
    binding.cancel(ui)?;
    binding.close()?;
    Ok(snapshot)
}

#[test]
fn public_binding_modes_and_directions_are_exported() {
    assert_eq!(BindingMode::OneTime, BindingMode::OneTime);
    assert_eq!(BindingMode::OneWay, BindingMode::OneWay);
    assert_eq!(BindingMode::TwoWay, BindingMode::TwoWay);
    assert_eq!(
        BindingDirection::SourceToTarget,
        BindingDirection::SourceToTarget
    );
    assert_eq!(
        BindingDirection::TargetToSource,
        BindingDirection::TargetToSource
    );
}

use freemvvm::{
    AsyncCommand, CancellationToken, CommandOperation, CommandState, OperationId, Property,
    UiContext, UiError,
};

#[allow(dead_code)]
fn create_public_command(
    ui: &UiContext<'_>,
    eligibility: Option<&Property<bool>>,
) -> Result<AsyncCommand, UiError> {
    AsyncCommand::create(ui, eligibility, |operation| {
        let _operation: OperationId = operation.id();
        let _cancellation: &CancellationToken = operation.cancellation();
        let _sendable: CommandOperation = operation;
    })
}

#[allow(dead_code)]
fn use_public_command(command: &mut AsyncCommand, ui: &UiContext<'_>) -> Result<(), UiError> {
    let _operation = command.execute(ui)?;
    let _state = command.state_property(ui)?;
    let _can_execute = command.can_execute_property(ui)?;
    let _failure = command.failure_snapshot(ui)?;
    command.close(ui)
}

#[test]
fn public_command_state_preserves_unknown_values() {
    assert_eq!(CommandState::from_raw(0), CommandState::Idle);
    assert_eq!(CommandState::from_raw(5), CommandState::Cancelled);
    assert_eq!(
        CommandState::from_raw(0x1234),
        CommandState::Unknown(0x1234)
    );
}

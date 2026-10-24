//! Runtime-local state machine for one reusable asynchronous Command.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i64)]
pub(crate) enum CommandState {
    Idle = 0,
    Running = 1,
    Cancelling = 2,
    Succeeded = 3,
    Failed = 4,
    Cancelled = 5,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandFailure {
    pub(crate) domain: u32,
    pub(crate) code: u32,
    pub(crate) message: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CommandError {
    Closed,
    NotExecutable,
    InvalidOperation,
    OperationIdExhausted,
    OutOfMemory,
    SynchronizationPoisoned,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandSnapshot {
    pub(crate) state: CommandState,
    pub(crate) eligible: bool,
    pub(crate) can_execute: bool,
    pub(crate) current_operation: Option<u64>,
    pub(crate) failure: Option<Arc<CommandFailure>>,
}

#[derive(Clone, Debug)]
pub(crate) struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub(crate) fn is_requested(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    fn request(&self) {
        self.0.store(true, Ordering::Release);
    }
}

/// Non-reusing operation IDs shared by every Command in one Runtime.
pub(crate) struct OperationIdIssuer(std::sync::atomic::AtomicU64);

impl OperationIdIssuer {
    pub(crate) fn new() -> Self {
        Self(std::sync::atomic::AtomicU64::new(0))
    }

    pub(crate) fn issue(&self) -> Result<u64, CommandError> {
        self.0
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |last| {
                last.checked_add(1)
            })
            .map(|last| last + 1)
            .map_err(|_| CommandError::OperationIdExhausted)
    }
}

struct Operation {
    id: u64,
    cancel: CancelToken,
}

struct State {
    open: bool,
    eligible: bool,
    state: CommandState,
    current: Option<Operation>,
    failure: Option<Arc<CommandFailure>>,
}

/// State and operation identity for an asynchronous Command.
///
/// The owner calls state-changing methods on its UI thread. CancelToken only exposes an
/// atomic read to workers and does not grant access to Runtime state.
pub(crate) struct AsyncCommand {
    state: Mutex<State>,
}

impl AsyncCommand {
    pub(crate) fn new(eligible: bool) -> Self {
        Self {
            state: Mutex::new(State {
                open: true,
                eligible,
                state: CommandState::Idle,
                current: None,
                failure: None,
            }),
        }
    }

    pub(crate) fn snapshot(&self) -> Result<CommandSnapshot, CommandError> {
        let state = self
            .state
            .lock()
            .map_err(|_| CommandError::SynchronizationPoisoned)?;
        Ok(Self::snapshot_locked(&state))
    }

    pub(crate) fn ensure_open(&self) -> Result<(), CommandError> {
        let state = self
            .state
            .lock()
            .map_err(|_| CommandError::SynchronizationPoisoned)?;
        if state.open {
            Ok(())
        } else {
            Err(CommandError::Closed)
        }
    }

    pub(crate) fn can_execute(&self) -> Result<bool, CommandError> {
        let state = self
            .state
            .lock()
            .map_err(|_| CommandError::SynchronizationPoisoned)?;
        if !state.open {
            return Err(CommandError::Closed);
        }
        Ok(Self::can_execute_locked(&state))
    }

    pub(crate) fn set_eligible(&self, eligible: bool) -> Result<CommandSnapshot, CommandError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| CommandError::SynchronizationPoisoned)?;
        if !state.open {
            return Err(CommandError::Closed);
        }
        state.eligible = eligible;
        Ok(Self::snapshot_locked(&state))
    }

    pub(crate) fn execute_with_id(
        &self,
        id: u64,
    ) -> Result<(u64, CancelToken, CommandSnapshot), CommandError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| CommandError::SynchronizationPoisoned)?;
        if !state.open {
            return Err(CommandError::Closed);
        }
        if !Self::can_execute_locked(&state) {
            return Err(CommandError::NotExecutable);
        }
        if id == 0 {
            return Err(CommandError::InvalidOperation);
        }
        let cancel = CancelToken::new();
        state.state = CommandState::Running;
        state.current = Some(Operation {
            id,
            cancel: cancel.clone(),
        });
        state.failure = None;
        let snapshot = Self::snapshot_locked(&state);
        Ok((id, cancel, snapshot))
    }

    pub(crate) fn cancel(&self, id: u64) -> Result<CommandSnapshot, CommandError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| CommandError::SynchronizationPoisoned)?;
        if !state.open {
            return Err(CommandError::Closed);
        }
        let Some(operation) = state
            .current
            .as_ref()
            .filter(|operation| operation.id == id)
        else {
            return Err(CommandError::InvalidOperation);
        };
        match state.state {
            CommandState::Running => {
                operation.cancel.request();
                state.state = CommandState::Cancelling;
            }
            CommandState::Cancelling
            | CommandState::Succeeded
            | CommandState::Failed
            | CommandState::Cancelled => {}
            CommandState::Idle => return Err(CommandError::InvalidOperation),
        }
        Ok(Self::snapshot_locked(&state))
    }

    pub(crate) fn complete_success(&self, id: u64) -> Result<CommandSnapshot, CommandError> {
        self.complete(id, CommandState::Succeeded, None)
    }

    pub(crate) fn complete_cancelled(&self, id: u64) -> Result<CommandSnapshot, CommandError> {
        self.complete(id, CommandState::Cancelled, None)
    }

    pub(crate) fn complete_failure(
        &self,
        id: u64,
        domain: u32,
        code: u32,
        message: &[u8],
    ) -> Result<CommandSnapshot, CommandError> {
        // Validate and allocate the immutable result before committing the operation terminal.
        let mut copied_message = Vec::new();
        copied_message
            .try_reserve(message.len())
            .map_err(|_| CommandError::OutOfMemory)?;
        copied_message.extend_from_slice(message);
        let failure = Arc::new(CommandFailure {
            domain,
            code,
            message: copied_message,
        });
        self.complete(id, CommandState::Failed, Some(failure))
    }

    fn complete(
        &self,
        id: u64,
        terminal: CommandState,
        failure: Option<Arc<CommandFailure>>,
    ) -> Result<CommandSnapshot, CommandError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| CommandError::SynchronizationPoisoned)?;
        if !state.open {
            return Err(CommandError::Closed);
        }
        if state.current.as_ref().map(|operation| operation.id) != Some(id)
            || !matches!(
                state.state,
                CommandState::Running | CommandState::Cancelling
            )
        {
            return Err(CommandError::InvalidOperation);
        }
        state.state = terminal;
        state.failure = failure;
        Ok(Self::snapshot_locked(&state))
    }

    pub(crate) fn close(&self) -> Result<CommandSnapshot, CommandError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| CommandError::SynchronizationPoisoned)?;
        if state.open {
            state.open = false;
            if let Some(operation) = &state.current {
                if matches!(
                    state.state,
                    CommandState::Running | CommandState::Cancelling
                ) {
                    operation.cancel.request();
                }
            }
        }
        Ok(Self::snapshot_locked(&state))
    }

    fn can_execute_locked(state: &State) -> bool {
        state.open
            && state.eligible
            && !matches!(
                state.state,
                CommandState::Running | CommandState::Cancelling
            )
    }

    fn snapshot_locked(state: &State) -> CommandSnapshot {
        CommandSnapshot {
            state: state.state,
            eligible: state.eligible,
            can_execute: Self::can_execute_locked(state),
            current_operation: state.current.as_ref().map(|operation| operation.id),
            failure: state.failure.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AsyncCommand, CommandError, CommandState, OperationIdIssuer};
    use std::sync::atomic::Ordering;

    #[test]
    fn command_execute_commits_state_and_rejects_reentry() {
        let command = AsyncCommand::new(true);
        let (id, _, running) = command
            .execute_with_id(1)
            .expect("first operation is accepted");

        assert_eq!(id, 1);
        assert_eq!(running.state, CommandState::Running);
        assert!(!running.can_execute);
        assert_eq!(
            command
                .execute_with_id(2)
                .expect_err("running command rejects reentry"),
            CommandError::NotExecutable
        );
    }

    #[test]
    fn command_cancel_is_cooperative_and_terminal_cancel_is_idempotent() {
        let command = AsyncCommand::new(true);
        let (id, token, _) = command.execute_with_id(1).expect("operation is accepted");

        assert!(!token.is_requested());
        assert_eq!(
            command.cancel(id).expect("cancel request accepted").state,
            CommandState::Cancelling
        );
        assert!(token.is_requested());
        assert_eq!(
            command
                .cancel(id)
                .expect("duplicate cancel is idempotent")
                .state,
            CommandState::Cancelling
        );
        assert_eq!(
            command
                .complete_success(id)
                .expect("worker may finish successfully")
                .state,
            CommandState::Succeeded
        );
        assert_eq!(
            command
                .cancel(id)
                .expect("terminal cancel is idempotent")
                .state,
            CommandState::Succeeded
        );
    }

    #[test]
    fn command_failure_snapshot_is_immutable_and_cleared_by_next_execute() {
        let command = AsyncCommand::new(true);
        let (id, _, _) = command.execute_with_id(1).expect("operation is accepted");
        let failed = command
            .complete_failure(id, 7, 11, b"failed")
            .expect("failure is recorded");
        let snapshot = failed.failure.expect("failure snapshot is present");
        assert_eq!(snapshot.message, b"failed");

        let (_, _, next) = command
            .execute_with_id(2)
            .expect("terminal command can be reused");
        assert_eq!(next.state, CommandState::Running);
        assert!(next.failure.is_none());
        assert_eq!(snapshot.message, b"failed");
    }

    #[test]
    fn command_cancelled_result_is_terminal_and_does_not_infer_from_request() {
        let command = AsyncCommand::new(true);
        let (id, token, _) = command.execute_with_id(1).expect("operation is accepted");
        command.cancel(id).expect("request is accepted");
        assert!(token.is_requested());
        let terminal = command
            .complete_cancelled(id)
            .expect("worker reports cancellation");

        assert_eq!(terminal.state, CommandState::Cancelled);
        assert!(terminal.can_execute);
        assert!(terminal.failure.is_none());
    }

    #[test]
    fn command_rejects_stale_completion_and_close_stops_worker_without_joining() {
        let command = AsyncCommand::new(true);
        let (first, _, _) = command
            .execute_with_id(1)
            .expect("first operation is accepted");
        command
            .complete_success(first)
            .expect("first operation completes");
        let (second, token, _) = command.execute_with_id(2).expect("command can be reused");

        assert_eq!(second, first + 1);
        assert_eq!(
            command
                .complete_success(first)
                .expect_err("old ID is stale"),
            CommandError::InvalidOperation
        );
        let closed = command.close().expect("close is immediate");
        assert!(!closed.can_execute);
        assert!(token.is_requested());
        assert_eq!(command.complete_success(second), Err(CommandError::Closed));
        assert_eq!(
            command.close().expect("close is idempotent").state,
            CommandState::Running
        );
    }

    #[test]
    fn disabled_command_does_not_allocate_an_operation_id() {
        let command = AsyncCommand::new(false);
        assert_eq!(
            command
                .execute_with_id(1)
                .expect_err("disabled command rejects execution"),
            CommandError::NotExecutable
        );
        command.set_eligible(true).expect("eligibility is updated");
        assert_eq!(
            command
                .execute_with_id(1)
                .expect("first accepted operation")
                .0,
            1
        );
    }

    #[test]
    fn operation_id_issuer_is_shared_and_stops_before_wrapping() {
        let issuer = OperationIdIssuer::new();
        assert_eq!(issuer.issue().expect("first ID"), 1);
        assert_eq!(issuer.issue().expect("second ID"), 2);
        issuer.0.store(u64::MAX - 1, Ordering::Release);
        assert_eq!(issuer.issue().expect("maximum ID"), u64::MAX);
        assert_eq!(issuer.issue(), Err(CommandError::OperationIdExhausted));
    }
}

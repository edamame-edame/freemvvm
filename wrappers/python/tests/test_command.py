"""Fake-ABI tests for Python AsyncCommand ownership and dispatch."""

from __future__ import annotations

import pytest

from test_binding import FakeBindingApi, make_runtime

from freemvvm import AsyncCommand, CommandState, Property, UiError
from freemvvm import _native


class FakeCommandApi(FakeBindingApi):
    def __init__(self) -> None:
        super().__init__()
        self.start = None
        self.start_userdata = 0
        self.destroy = None
        self.command = 900
        self.operation_id = 77
        self.command_state = int(CommandState.IDLE)
        self.posts: list[tuple[object, int, object]] = []
        self.completions: list[tuple[object, ...]] = []
        self.post_status = _native.OK

    def handle_retain(self, borrowed: int) -> tuple[int, int]:
        return _native.OK, self._token()

    def command_create_async(
        self,
        runtime: int,
        eligibility: int,
        start: object,
        userdata: int,
        destroy: object,
    ) -> tuple[int, int, int]:
        self.start, self.start_userdata, self.destroy = (
            start,
            userdata,
            destroy,
        )
        return _native.OK, self.command, 0

    def command_execute(self, command: int) -> tuple[int, int, int]:
        self.command_state = int(CommandState.RUNNING)
        self.start(self.start_userdata, command, self.operation_id, 902)
        return _native.OK, self.operation_id, 0

    def command_cancel(
        self, command: int, operation_id: int
    ) -> tuple[int, int]:
        return _native.OK, 0

    def command_close(self, command: int) -> tuple[int, int]:
        self.destroy(self.start_userdata)
        return _native.OK, 0

    def command_state_property(self, command: int) -> tuple[int, int]:
        status, token, _ = self._create_property("i64", self.command_state)
        return status, token

    def command_can_execute_property(self, command: int) -> tuple[int, int]:
        status, token, _ = self._create_property("bool", True)
        return status, token

    def command_failure_snapshot(self, command: int) -> tuple[int, int]:
        return _native.OK, 0

    def command_cancel_token_requested(self, token: int) -> tuple[int, int]:
        return _native.OK, 0

    def command_complete_success(self, command: int, operation_id: int) -> int:
        self.command_state = int(CommandState.SUCCEEDED)
        self.completions.append(("success", command, operation_id))
        return _native.OK

    def command_complete_failure(
        self,
        command: int,
        operation_id: int,
        domain: int,
        code: int,
        message: bytes,
    ) -> int:
        self.command_state = int(CommandState.FAILED)
        self.completions.append(
            ("failure", command, operation_id, domain, code, message)
        )
        return _native.OK

    def command_complete_cancelled(
        self, command: int, operation_id: int
    ) -> int:
        self.command_state = int(CommandState.CANCELLED)
        self.completions.append(("cancelled", command, operation_id))
        return _native.OK

    def runtime_post(
        self, runtime: int, callback: object, userdata: int, destroy: object
    ) -> tuple[int, int]:
        if self.post_status != _native.OK:
            return self.post_status, 0
        self.posts.append((callback, userdata, destroy))
        return _native.OK, 0

    def dispatch(self) -> None:
        callback, userdata, destroy = self.posts.pop(0)
        callback(userdata)
        destroy(userdata)


def _create(api: FakeCommandApi, eligibility: Property | None = None):
    runtime, _ = make_runtime(api)
    command = AsyncCommand.create(runtime, lambda operation: None, eligibility)
    return runtime, command


def test_start_retains_worker_handles_and_reports_success_via_post() -> None:
    api = FakeCommandApi()
    runtime, _ = make_runtime(api)
    operations = []
    command = AsyncCommand.create(runtime, operations.append)

    operation_id = command.execute()
    operation = operations[0]
    assert operation_id == operation.operation_id == 77
    assert operation.is_cancel_requested() is False
    operation.report_success()
    operation.close()
    assert api.completions == []
    api.dispatch()
    assert api.completions == [("success", operation._command.token, 77)]
    assert command.state_property().get() == int(CommandState.SUCCEEDED)
    command.close()
    runtime.close()


def test_post_retains_callback_and_arguments_until_dispatch() -> None:
    api = FakeCommandApi()
    runtime, _ = make_runtime(api)
    operations = []
    command = AsyncCommand.create(runtime, operations.append)
    command.execute()
    observed: list[object] = []
    value = object()
    operations[0].post(observed.append, value)
    assert observed == []
    api.dispatch()
    assert observed == [value]
    operations[0].close()
    command.close()
    runtime.close()


def test_failure_validation_and_duplicate_terminal_are_rejected() -> None:
    api = FakeCommandApi()
    runtime, _ = make_runtime(api)
    operations = []
    command = AsyncCommand.create(runtime, operations.append)
    command.execute()
    operation = operations[0]
    with pytest.raises(ValueError):
        operation.report_failure(-1, 0, "bad")
    operation.report_failure(12, 34, "失敗")
    with pytest.raises(UiError):
        operation.report_success()
    api.dispatch()
    assert api.completions[0][-1] == "失敗".encode()
    operation.close()
    command.close()
    runtime.close()


def test_rejected_terminal_post_can_be_retried() -> None:
    api = FakeCommandApi()
    runtime, _ = make_runtime(api)
    operations = []
    command = AsyncCommand.create(runtime, operations.append)
    command.execute()
    operation = operations[0]
    api.post_status = _native.CLOSED
    with pytest.raises(UiError):
        operation.report_cancelled()
    api.post_status = _native.OK
    operation.report_cancelled()
    api.dispatch()
    assert api.completions[0][0] == "cancelled"
    operation.close()
    command.close()
    runtime.close()

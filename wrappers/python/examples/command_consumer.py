"""Exercise Python AsyncCommand against the built host C ABI library."""

from __future__ import annotations

import ctypes
import sys

import freemvvm
from freemvvm import AsyncCommand, CommandState, Property, Runtime
from freemvvm import _native


def _check(status: int, name: str) -> None:
    if status != _native.OK:
        raise RuntimeError(f"{name} failed with C ABI status {status}")


def _dispatch_event(
    library: ctypes.CDLL, runtime: int, action: object
) -> None:
    scope = ctypes.c_uint64()
    _check(
        library.ui_abi_1_runtime_host_begin_event(
            runtime, ctypes.byref(scope)
        ),
        "begin host event",
    )
    try:
        action()  # type: ignore[operator]
    finally:
        _check(
            library.ui_abi_1_runtime_host_end_event(runtime, scope.value),
            "end host event",
        )


def main() -> None:
    library = ctypes.CDLL(sys.argv[1])
    wake_type = ctypes.CFUNCTYPE(None, ctypes.c_void_p, ctypes.c_uint32)
    destroy_type = ctypes.CFUNCTYPE(None, ctypes.c_void_p)
    library.ui_abi_1_runtime_set_ui_loop_bridge.argtypes = [
        _native.Handle,
        wake_type,
        ctypes.c_void_p,
        destroy_type,
        ctypes.POINTER(_native.Handle),
    ]
    library.ui_abi_1_runtime_set_ui_loop_bridge.restype = _native.Status
    library.ui_abi_1_runtime_dispatch_turn.argtypes = [_native.Handle]
    library.ui_abi_1_runtime_dispatch_turn.restype = _native.Status
    library.ui_abi_1_runtime_host_begin_event.argtypes = [
        _native.Handle,
        ctypes.POINTER(ctypes.c_uint64),
    ]
    library.ui_abi_1_runtime_host_begin_event.restype = _native.Status
    library.ui_abi_1_runtime_host_end_event.argtypes = [
        _native.Handle,
        ctypes.c_uint64,
    ]
    library.ui_abi_1_runtime_host_end_event.restype = _native.Status
    freemvvm._load_native_api = lambda: _native.CtypesNativeApi(library)

    runtime = Runtime.create()
    token = runtime._cell.token
    wake = wake_type(lambda _userdata, _priority: None)
    no_destroy = ctypes.cast(ctypes.c_void_p(), destroy_type)
    error = _native.Handle()
    _check(
        library.ui_abi_1_runtime_set_ui_loop_bridge(
            token, wake, None, no_destroy, ctypes.byref(error)
        ),
        "set UI bridge",
    )
    if error.value:
        runtime._cell.api.handle_release(error.value)
        raise RuntimeError("registering the UI loop bridge returned an error")

    operations = []
    progress = Property.create(runtime, 0)
    command = AsyncCommand.create(runtime, operations.append)
    state = command.state_property()
    can_execute = command.can_execute_property()
    assert can_execute.get() is True
    try:
        for scenario in ("success", "failure", "cancel"):
            operation_id: list[int] = []
            _dispatch_event(
                library,
                token,
                lambda: operation_id.append(command.execute()),
            )
            operation = operations.pop()
            if scenario == "success":
                operation.post(progress.set, 42)
                operation.report_success()
            elif scenario == "failure":
                operation.report_failure(23, 41, "consumer failure")
            else:
                _dispatch_event(
                    library, token, lambda: command.cancel(operation_id[0])
                )
                operation.report_cancelled()

            _check(
                library.ui_abi_1_runtime_dispatch_turn(token),
                "dispatch command result",
            )
            if scenario == "success":
                _check(
                    library.ui_abi_1_runtime_dispatch_turn(token),
                    "dispatch progress callback",
                )
                assert progress.get() == 42
                assert state.get() == int(CommandState.SUCCEEDED)
            elif scenario == "failure":
                assert state.get() == int(CommandState.FAILED)
                snapshot = command.failure_snapshot()
                assert snapshot is not None
                assert (snapshot.domain, snapshot.code, snapshot.message) == (
                    23,
                    41,
                    "consumer failure",
                )
            else:
                assert state.get() == int(CommandState.CANCELLED)
            operation.close()
        command.close()
        runtime.shutdown()
    finally:
        command.try_close()
        state.try_close()
        can_execute.try_close()
        progress.try_close()
        runtime.try_close()
    print("Python AsyncCommand C ABI consumer: OK")


if __name__ == "__main__":
    main()

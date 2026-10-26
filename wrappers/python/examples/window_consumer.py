"""Exercise Python Window peer and close subscription against the host C ABI."""

from __future__ import annotations

import ctypes
import sys

import freemvvm
from freemvvm import Runtime, Text, Window
from freemvvm import _native


def _check(status: int, operation: str) -> None:
    if status != _native.OK:
        raise RuntimeError(f"{operation} failed with C ABI status {status}")


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
    freemvvm._load_native_api = lambda: _native.CtypesNativeApi(library)
    runtime = Runtime.create()
    wake = wake_type(lambda _userdata, _priority: None)
    bridge_error = _native.Handle()
    _check(
        library.ui_abi_1_runtime_set_ui_loop_bridge(
            runtime._cell.token,
            wake,
            None,
            ctypes.cast(ctypes.c_void_p(), destroy_type),
            ctypes.byref(bridge_error),
        ),
        "register UI loop bridge",
    )
    if bridge_error.value:
        runtime._cell.api.handle_release(bridge_error.value)
        raise RuntimeError("UI loop bridge registration returned an error")

    window = Window.create(runtime)
    content = Text.create(runtime, "preserve after unsupported peer open")
    window.set_content(content)
    subscription = None
    replacement_subscription = None
    try:
        subscription = window.on_close_request(lambda _request: None)
        try:
            window.on_close_request(lambda _request: None)
        except freemvvm.UiError as error:
            assert error.raw_status == _native.INVALID_ARGUMENT
        else:
            raise AssertionError("duplicate close subscription was accepted")

        try:
            window.open()
        except freemvvm.UiError as error:
            assert error.raw_status == _native.UNSUPPORTED
        else:
            raise AssertionError("host unexpectedly opened a native Window")

        current_content = window.content
        assert isinstance(current_content, freemvvm.Widget)
        current_content.close()

        subscription.cancel()
        subscription.close()
        replacement_subscription = window.on_close_request(
            lambda _request: None
        )
        replacement_subscription.cancel()
        replacement_subscription.close()
    finally:
        if replacement_subscription is not None:
            try:
                replacement_subscription.cancel()
            except freemvvm.UiError:
                pass
            try:
                replacement_subscription.close()
            except freemvvm.UiError:
                pass
        if subscription is not None:
            try:
                subscription.cancel()
            except freemvvm.UiError:
                pass
            try:
                subscription.close()
            except freemvvm.UiError:
                pass
        try:
            window.clear_content()
        except freemvvm.UiError:
            pass
        for owner in (content, window):
            try:
                owner.close()
            except freemvvm.UiError:
                pass
        try:
            runtime.close()
        except freemvvm.UiError:
            pass
    print("Python Window peer/close-request C ABI consumer: OK")


if __name__ == "__main__":
    main()

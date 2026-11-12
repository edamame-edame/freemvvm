"""Manually verify visible Linux Window frames and peer reopen."""

from __future__ import annotations

import ctypes
import sys
import time

import freemvvm
from freemvvm import Runtime, Window
from freemvvm import _native


def _check(status: int, operation: str) -> None:
    if status != _native.OK:
        raise RuntimeError(f"{operation} failed with C ABI status {status}")


def _show_color(library, runtime: Runtime, window: Window, name: str, rgb) -> None:
    print(f"Presenting {name} background for 4 seconds", flush=True)
    window.background_rgb = rgb
    deadline = time.monotonic() + 4.0
    while time.monotonic() < deadline:
        _check(
            library.ui_abi_1_runtime_dispatch_turn(runtime._cell.token),
            "runtime_dispatch_turn",
        )
        time.sleep(0.02)


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
    freemvvm._load_native_api = lambda: _native.CtypesNativeApi(library)

    runtime = Runtime.create()
    wake = wake_type(lambda _userdata, _priority: None)
    bridge_error = _native.Handle()
    status = library.ui_abi_1_runtime_set_ui_loop_bridge(
        runtime._cell.token,
        wake,
        None,
        ctypes.cast(ctypes.c_void_p(), destroy_type),
        ctypes.byref(bridge_error),
    )
    if status != _native.OK or bridge_error.value:
        if bridge_error.value:
            runtime._cell.api.handle_release(bridge_error.value)
        runtime.close()
        raise RuntimeError(
            f"register UI loop bridge failed with status {status}"
        )

    window = Window.create(runtime)
    opened = False
    try:
        print(
            "Observe solid red, blue, then green frames on the VM display.",
            flush=True,
        )
        time.sleep(2.0)
        window.background_rgb = (0.9, 0.05, 0.05)
        window.open()
        opened = True
        _show_color(
            library, runtime, window, "red", (0.9, 0.05, 0.05)
        )
        _show_color(
            library, runtime, window, "blue", (0.05, 0.1, 0.9)
        )
        window.close_peer()
        opened = False
        window.background_rgb = (0.05, 0.8, 0.1)
        window.open()
        opened = True
        _show_color(
            library, runtime, window, "green after reopen", (0.05, 0.8, 0.1)
        )
    finally:
        if opened:
            try:
                window.close_peer()
            except freemvvm.UiError:
                pass
        try:
            window.close()
        finally:
            runtime.close()
    print("Python Window visible present and reopen smoke: OK")


if __name__ == "__main__":
    main()

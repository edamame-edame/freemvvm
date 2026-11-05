"""Manually verify Window peer open/close on a configured Linux display."""

from __future__ import annotations

import ctypes
import sys

import freemvvm
from freemvvm import Runtime, Window
from freemvvm import _native


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
        window.open()
        opened = True
        window.close_peer()
        opened = False
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
    print("Python Window platform peer open/close smoke: OK")


if __name__ == "__main__":
    main()

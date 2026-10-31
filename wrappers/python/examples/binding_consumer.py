"""Consumer roundtrip for Python Property Bindings through the C ABI."""

from __future__ import annotations

import ctypes
import sys

import freemvvm
from freemvvm import Binding, BindingMode, Property, Runtime
from freemvvm import _native


def _check(status: int, operation: str) -> None:
    if status != _native.OK:
        raise RuntimeError(f"{operation} failed with C ABI status {status}")


def _dispatch_event(
    library: ctypes.CDLL, runtime: int, action: object
) -> None:
    scope = ctypes.c_uint64()
    _check(
        library.ui_abi_1_runtime_host_begin_event(
            runtime, ctypes.byref(scope)
        ),
        "runtime_host_begin_event",
    )
    try:
        action()  # type: ignore[operator]
    finally:
        _check(
            library.ui_abi_1_runtime_host_end_event(runtime, scope.value),
            "runtime_host_end_event",
        )
    _check(
        library.ui_abi_1_runtime_dispatch_turn(runtime),
        "runtime_dispatch_turn",
    )


def _bind_host_turn_functions(library: ctypes.CDLL) -> None:
    wake_callback_type = ctypes.CFUNCTYPE(
        None, ctypes.c_void_p, ctypes.c_uint32
    )
    destroy_callback_type = ctypes.CFUNCTYPE(None, ctypes.c_void_p)
    library.ui_abi_1_runtime_set_ui_loop_bridge.argtypes = [
        _native.Handle,
        wake_callback_type,
        ctypes.c_void_p,
        destroy_callback_type,
        ctypes.POINTER(_native.Handle),
    ]
    library.ui_abi_1_runtime_set_ui_loop_bridge.restype = _native.Status
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
    library.ui_abi_1_runtime_dispatch_turn.argtypes = [_native.Handle]
    library.ui_abi_1_runtime_dispatch_turn.restype = _native.Status


def main() -> None:
    library_path = sys.argv[1] if len(sys.argv) > 1 else "freemvvm_abi_1"
    library = ctypes.CDLL(library_path)
    _bind_host_turn_functions(library)
    freemvvm._load_native_api = lambda: _native.CtypesNativeApi(library)

    runtime = Runtime.create()
    runtime_token = runtime._cell.token
    wake_callback_type = ctypes.CFUNCTYPE(
        None, ctypes.c_void_p, ctypes.c_uint32
    )
    destroy_callback_type = ctypes.CFUNCTYPE(None, ctypes.c_void_p)
    wake_callback = wake_callback_type(lambda _userdata, _priority: None)
    no_destroy_callback = ctypes.cast(ctypes.c_void_p(), destroy_callback_type)
    bridge_error = _native.Handle()
    _check(
        library.ui_abi_1_runtime_set_ui_loop_bridge(
            runtime_token,
            wake_callback,
            None,
            no_destroy_callback,
            ctypes.byref(bridge_error),
        ),
        "runtime_set_ui_loop_bridge",
    )
    if bridge_error.value:
        runtime._cell.api.handle_release(bridge_error.value)
        raise RuntimeError("registering the UI loop bridge returned an error")
    properties: list[Property] = []
    bindings: list[Binding] = []
    try:
        source = Property.create(runtime, 1)
        target = Property.create(runtime, 0)
        properties.extend((source, target))
        assert Binding.create(source, target, BindingMode.ONE_TIME) is None
        assert target.get() == 1
        _dispatch_event(library, runtime_token, lambda: source.set(2))
        assert target.get() == 1

        source = Property.create(runtime, 10)
        target = Property.create(runtime, 0)
        properties.extend((source, target))
        one_way = Binding.create(source, target, BindingMode.ONE_WAY)
        assert one_way is not None
        bindings.append(one_way)
        assert target.get() == 10
        _dispatch_event(library, runtime_token, lambda: target.set(11))
        assert source.get() == 10
        _dispatch_event(library, runtime_token, lambda: source.set(12))
        assert target.get() == 12

        source = Property.create(runtime, "source")
        target = Property.create(runtime, "target")
        properties.extend((source, target))
        two_way = Binding.create(source, target, BindingMode.TWO_WAY)
        assert two_way is not None
        bindings.append(two_way)
        assert target.get() == "source"
        _dispatch_event(library, runtime_token, lambda: target.set("target"))
        assert source.get() == target.get() == "target"
        _dispatch_event(
            library,
            runtime_token,
            lambda: (source.set("first"), target.set("last")),
        )
        assert source.get() == target.get() == "last"

        source = Property.create(runtime, 20)
        target = Property.create(runtime, 0)
        properties.extend((source, target))
        released = Binding.create(source, target, BindingMode.ONE_WAY)
        assert released is not None
        released.close()
        _dispatch_event(library, runtime_token, lambda: source.set(21))
        assert target.get() == 21

        source = Property.create(runtime, 30)
        target = Property.create(runtime, 0)
        properties.extend((source, target))
        cancelled = Binding.create(source, target, BindingMode.ONE_WAY)
        assert cancelled is not None
        bindings.append(cancelled)
        cancelled.cancel()
        _dispatch_event(library, runtime_token, lambda: source.set(31))
        assert target.get() == 30

        for binding in bindings:
            binding.close()
        bindings.clear()
        for prop in properties:
            prop.close()
        properties.clear()
        runtime.shutdown()
    finally:
        for binding in bindings:
            binding.try_close()
        for prop in properties:
            prop.try_close()
        runtime.try_close()

    print("Python Binding C ABI consumer: OK")


if __name__ == "__main__":
    main()

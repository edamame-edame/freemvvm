"""Private ctypes declarations and native-library discovery."""

from __future__ import annotations

import ctypes
import ctypes.util
import os
import sys
from typing import Protocol


Handle = ctypes.c_uint64
Status = ctypes.c_uint32
Bool = ctypes.c_uint8

OK = 0
INVALID_ARGUMENT = 1
INVALID_HANDLE = 2
WRONG_TYPE = 3
WRONG_RUNTIME = 4
WRONG_THREAD = 5
CLOSED = 6
BUFFER_TOO_SMALL = 7
OUT_OF_MEMORY = 8
INTERNAL_ERROR = 9
UNSUPPORTED = 10
PENDING = 14

SHUTDOWN_ASYNC = 0
SHUTDOWN_SYNC = 1
SHUTDOWN_REQUEST_PENDING = 0
SHUTDOWN_REQUEST_COMPLETED = 1
SHUTDOWN_REQUEST_HOST_UNAVAILABLE = 2


class NativeApi(Protocol):
    """The narrow private boundary used by the Runtime facade and its tests."""

    def get_version(self) -> tuple[int, int, int, int]: ...

    def runtime_create(self) -> tuple[int, int, int]: ...

    def runtime_is_ui_thread(self, runtime: int) -> tuple[int, int]: ...

    def runtime_shutdown(self, runtime: int) -> tuple[int, int]: ...

    def runtime_request_shutdown(
        self, runtime: int, mode: int
    ) -> tuple[int, int, int]: ...

    def shutdown_request_query(
        self, request: int
    ) -> tuple[int, int, int, int]: ...

    def property_create_bool(
        self, runtime: int, initial: int
    ) -> tuple[int, int, int]: ...

    def property_create_i64(
        self, runtime: int, initial: int
    ) -> tuple[int, int, int]: ...

    def property_create_f64(
        self, runtime: int, initial: float
    ) -> tuple[int, int, int]: ...

    def property_create_utf8(
        self, runtime: int, initial: bytes
    ) -> tuple[int, int, int]: ...

    def property_get_bool(self, property: int) -> tuple[int, int]: ...

    def property_get_i64(self, property: int) -> tuple[int, int]: ...

    def property_get_f64(self, property: int) -> tuple[int, float]: ...

    def property_get_utf8(
        self, property: int, buffer: bytearray | None
    ) -> tuple[int, int]: ...

    def property_set_bool(
        self, property: int, value: int
    ) -> tuple[int, int]: ...

    def property_set_i64(
        self, property: int, value: int
    ) -> tuple[int, int]: ...

    def property_set_f64(
        self, property: int, value: float
    ) -> tuple[int, int]: ...

    def property_set_utf8(
        self, property: int, value: bytes
    ) -> tuple[int, int]: ...

    def handle_release(self, owned: int) -> int: ...

    def error_status(self, error: int) -> tuple[int, int]: ...

    def error_code(self, error: int) -> tuple[int, int, int]: ...

    def error_message_utf8(
        self, error: int, buffer: bytearray | None
    ) -> tuple[int, int]: ...


class CtypesNativeApi:
    """Private adapter for the versioned shared C ABI."""

    def __init__(self, library: ctypes.CDLL) -> None:
        self._library = library
        self._bind(
            "ui_abi_1_get_version", [ctypes.POINTER(ctypes.c_uint32)] * 3
        )
        self._bind(
            "ui_abi_1_runtime_create",
            [ctypes.POINTER(Handle), ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_runtime_is_ui_thread",
            [Handle, ctypes.POINTER(Bool)],
        )
        self._bind(
            "ui_abi_1_runtime_shutdown", [Handle, ctypes.POINTER(Handle)]
        )
        self._bind(
            "ui_abi_1_runtime_request_shutdown",
            [
                Handle,
                ctypes.c_uint32,
                ctypes.POINTER(Handle),
                ctypes.POINTER(Handle),
            ],
        )
        self._bind(
            "ui_abi_1_shutdown_request_query",
            [
                Handle,
                ctypes.POINTER(ctypes.c_uint32),
                ctypes.POINTER(ctypes.c_uint32),
                ctypes.POINTER(Handle),
            ],
        )
        self._bind(
            "ui_abi_1_property_create_bool",
            [Handle, Bool, ctypes.POINTER(Handle), ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_property_create_i64",
            [
                Handle,
                ctypes.c_int64,
                ctypes.POINTER(Handle),
                ctypes.POINTER(Handle),
            ],
        )
        self._bind(
            "ui_abi_1_property_create_f64",
            [
                Handle,
                ctypes.c_double,
                ctypes.POINTER(Handle),
                ctypes.POINTER(Handle),
            ],
        )
        self._bind(
            "ui_abi_1_property_create_utf8",
            [
                Handle,
                ctypes.POINTER(ctypes.c_uint8),
                ctypes.c_uint64,
                ctypes.POINTER(Handle),
                ctypes.POINTER(Handle),
            ],
        )
        self._bind(
            "ui_abi_1_property_get_bool", [Handle, ctypes.POINTER(Bool)]
        )
        self._bind(
            "ui_abi_1_property_get_i64",
            [Handle, ctypes.POINTER(ctypes.c_int64)],
        )
        self._bind(
            "ui_abi_1_property_get_f64",
            [Handle, ctypes.POINTER(ctypes.c_double)],
        )
        self._bind(
            "ui_abi_1_property_get_utf8",
            [
                Handle,
                ctypes.POINTER(ctypes.c_uint8),
                ctypes.c_uint64,
                ctypes.POINTER(ctypes.c_uint64),
            ],
        )
        self._bind(
            "ui_abi_1_property_set_bool",
            [Handle, Bool, ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_property_set_i64",
            [Handle, ctypes.c_int64, ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_property_set_f64",
            [Handle, ctypes.c_double, ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_property_set_utf8",
            [
                Handle,
                ctypes.POINTER(ctypes.c_uint8),
                ctypes.c_uint64,
                ctypes.POINTER(Handle),
            ],
        )
        self._bind("ui_abi_1_handle_release", [Handle])
        self._bind(
            "ui_abi_1_error_status",
            [Handle, ctypes.POINTER(ctypes.c_uint32)],
        )
        self._bind(
            "ui_abi_1_error_code",
            [
                Handle,
                ctypes.POINTER(ctypes.c_uint32),
                ctypes.POINTER(ctypes.c_uint32),
            ],
        )
        self._bind(
            "ui_abi_1_error_message_utf8",
            [
                Handle,
                ctypes.POINTER(ctypes.c_uint8),
                ctypes.c_uint64,
                ctypes.POINTER(ctypes.c_uint64),
            ],
        )

    @classmethod
    def load(cls) -> CtypesNativeApi:
        library_names = [ctypes.util.find_library("freemvvm_abi_1")]
        if os.name == "nt":
            library_names.extend(["freemvvm_abi_1.dll"])
        elif sys.platform == "darwin":
            library_names.extend(["libfreemvvm_abi_1.dylib"])
        else:
            library_names.extend(["libfreemvvm_abi_1.so"])

        errors: list[OSError] = []
        for name in library_names:
            if not name:
                continue
            try:
                return cls(ctypes.CDLL(name))
            except OSError as error:
                errors.append(error)

        detail = (
            str(errors[-1])
            if errors
            else "library was not found by the system loader"
        )
        raise OSError(f"FreeMvvm native runtime is unavailable: {detail}")

    def _bind(self, name: str, argtypes: list[object]) -> None:
        function = getattr(self._library, name)
        function.argtypes = argtypes
        function.restype = Status

    def get_version(self) -> tuple[int, int, int, int]:
        major = ctypes.c_uint32()
        minor = ctypes.c_uint32()
        patch = ctypes.c_uint32()
        status = self._library.ui_abi_1_get_version(
            ctypes.byref(major), ctypes.byref(minor), ctypes.byref(patch)
        )
        return status, major.value, minor.value, patch.value

    def runtime_create(self) -> tuple[int, int, int]:
        runtime = Handle()
        error = Handle()
        status = self._library.ui_abi_1_runtime_create(
            ctypes.byref(runtime), ctypes.byref(error)
        )
        return status, runtime.value, error.value

    def runtime_is_ui_thread(self, runtime: int) -> tuple[int, int]:
        is_ui_thread = Bool()
        status = self._library.ui_abi_1_runtime_is_ui_thread(
            runtime, ctypes.byref(is_ui_thread)
        )
        return status, is_ui_thread.value

    def runtime_shutdown(self, runtime: int) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_runtime_shutdown(
            runtime, ctypes.byref(error)
        )
        return status, error.value

    def runtime_request_shutdown(
        self, runtime: int, mode: int
    ) -> tuple[int, int, int]:
        request = Handle()
        error = Handle()
        status = self._library.ui_abi_1_runtime_request_shutdown(
            runtime, mode, ctypes.byref(request), ctypes.byref(error)
        )
        return status, request.value, error.value

    def shutdown_request_query(
        self, request: int
    ) -> tuple[int, int, int, int]:
        state = ctypes.c_uint32()
        reason = ctypes.c_uint32()
        error = Handle()
        status = self._library.ui_abi_1_shutdown_request_query(
            request,
            ctypes.byref(state),
            ctypes.byref(reason),
            ctypes.byref(error),
        )
        return status, state.value, reason.value, error.value

    def property_create_bool(
        self, runtime: int, initial: int
    ) -> tuple[int, int, int]:
        property_handle = Handle()
        error = Handle()
        status = self._library.ui_abi_1_property_create_bool(
            runtime,
            initial,
            ctypes.byref(property_handle),
            ctypes.byref(error),
        )
        return status, property_handle.value, error.value

    def property_create_i64(
        self, runtime: int, initial: int
    ) -> tuple[int, int, int]:
        property_handle = Handle()
        error = Handle()
        status = self._library.ui_abi_1_property_create_i64(
            runtime,
            initial,
            ctypes.byref(property_handle),
            ctypes.byref(error),
        )
        return status, property_handle.value, error.value

    def property_create_f64(
        self, runtime: int, initial: float
    ) -> tuple[int, int, int]:
        property_handle = Handle()
        error = Handle()
        status = self._library.ui_abi_1_property_create_f64(
            runtime,
            initial,
            ctypes.byref(property_handle),
            ctypes.byref(error),
        )
        return status, property_handle.value, error.value

    def property_create_utf8(
        self, runtime: int, initial: bytes
    ) -> tuple[int, int, int]:
        property_handle = Handle()
        error = Handle()
        storage = (ctypes.c_uint8 * len(initial)).from_buffer_copy(initial)
        pointer = ctypes.cast(storage, ctypes.POINTER(ctypes.c_uint8))
        if not initial:
            pointer = ctypes.POINTER(ctypes.c_uint8)()
        status = self._library.ui_abi_1_property_create_utf8(
            runtime,
            pointer,
            len(initial),
            ctypes.byref(property_handle),
            ctypes.byref(error),
        )
        return status, property_handle.value, error.value

    def property_get_bool(self, property: int) -> tuple[int, int]:
        value = Bool()
        status = self._library.ui_abi_1_property_get_bool(
            property, ctypes.byref(value)
        )
        return status, value.value

    def property_get_i64(self, property: int) -> tuple[int, int]:
        value = ctypes.c_int64()
        status = self._library.ui_abi_1_property_get_i64(
            property, ctypes.byref(value)
        )
        return status, value.value

    def property_get_f64(self, property: int) -> tuple[int, float]:
        value = ctypes.c_double()
        status = self._library.ui_abi_1_property_get_f64(
            property, ctypes.byref(value)
        )
        return status, value.value

    def property_get_utf8(
        self, property: int, buffer: bytearray | None
    ) -> tuple[int, int]:
        length = ctypes.c_uint64()
        if buffer is None or not buffer:
            pointer = ctypes.POINTER(ctypes.c_uint8)()
            capacity = 0 if buffer is None else len(buffer)
        else:
            storage = (ctypes.c_uint8 * len(buffer)).from_buffer(buffer)
            pointer = ctypes.cast(storage, ctypes.POINTER(ctypes.c_uint8))
            capacity = len(buffer)
        status = self._library.ui_abi_1_property_get_utf8(
            property, pointer, capacity, ctypes.byref(length)
        )
        return status, length.value

    def property_set_bool(self, property: int, value: int) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_property_set_bool(
            property, value, ctypes.byref(error)
        )
        return status, error.value

    def property_set_i64(self, property: int, value: int) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_property_set_i64(
            property, value, ctypes.byref(error)
        )
        return status, error.value

    def property_set_f64(self, property: int, value: float) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_property_set_f64(
            property, value, ctypes.byref(error)
        )
        return status, error.value

    def property_set_utf8(
        self, property: int, value: bytes
    ) -> tuple[int, int]:
        error = Handle()
        storage = (ctypes.c_uint8 * len(value)).from_buffer_copy(value)
        pointer = ctypes.cast(storage, ctypes.POINTER(ctypes.c_uint8))
        if not value:
            pointer = ctypes.POINTER(ctypes.c_uint8)()
        status = self._library.ui_abi_1_property_set_utf8(
            property, pointer, len(value), ctypes.byref(error)
        )
        return status, error.value

    def handle_release(self, owned: int) -> int:
        return self._library.ui_abi_1_handle_release(owned)

    def error_status(self, error: int) -> tuple[int, int]:
        status_value = ctypes.c_uint32()
        status = self._library.ui_abi_1_error_status(
            error, ctypes.byref(status_value)
        )
        return status, status_value.value

    def error_code(self, error: int) -> tuple[int, int, int]:
        domain = ctypes.c_uint32()
        code = ctypes.c_uint32()
        status = self._library.ui_abi_1_error_code(
            error, ctypes.byref(domain), ctypes.byref(code)
        )
        return status, domain.value, code.value

    def error_message_utf8(
        self, error: int, buffer: bytearray | None
    ) -> tuple[int, int]:
        length = ctypes.c_uint64()
        if buffer is None:
            pointer = ctypes.POINTER(ctypes.c_uint8)()
            capacity = 0
        else:
            storage = (ctypes.c_uint8 * len(buffer)).from_buffer(buffer)
            pointer = ctypes.cast(storage, ctypes.POINTER(ctypes.c_uint8))
            capacity = len(buffer)
        status = self._library.ui_abi_1_error_message_utf8(
            error, pointer, capacity, ctypes.byref(length)
        )
        return status, length.value

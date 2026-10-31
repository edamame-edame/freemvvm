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
VoidCallback = ctypes.CFUNCTYPE(None, ctypes.c_void_p)
CommandStartCallback = ctypes.CFUNCTYPE(
    Status, ctypes.c_void_p, Handle, ctypes.c_uint64, Handle
)
WindowCloseCallback = ctypes.CFUNCTYPE(None, ctypes.c_void_p, Handle)

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

ORIENTATION_VERTICAL = 1
ORIENTATION_HORIZONTAL = 2
ALIGN_START = 1
ALIGN_CENTER = 2
ALIGN_END = 3
ALIGN_STRETCH = 4
CLOSE_ACCEPT = 1
CLOSE_DECLINE = 2


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

    def property_create_decimal(
        self, runtime: int, initial_bits: int
    ) -> tuple[int, int, int]: ...

    def property_create_utf8(
        self, runtime: int, initial: bytes
    ) -> tuple[int, int, int]: ...

    def property_get_bool(self, property: int) -> tuple[int, int]: ...

    def property_get_i64(self, property: int) -> tuple[int, int]: ...

    def property_get_f64(self, property: int) -> tuple[int, float]: ...

    def property_get_decimal(self, property: int) -> tuple[int, int]: ...

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

    def property_set_decimal(
        self, property: int, bits: int
    ) -> tuple[int, int]: ...

    def property_set_utf8(
        self, property: int, value: bytes
    ) -> tuple[int, int]: ...

    def binding_create(
        self, source: int, target: int, mode: int
    ) -> tuple[int, int, int]: ...

    def binding_cancel(self, binding: int) -> tuple[int, int]: ...

    def binding_error_snapshot(
        self, binding: int
    ) -> tuple[int, int, int, int]: ...

    def window_create(self, runtime: int) -> tuple[int, int, int]: ...

    def window_open(self, window: int) -> tuple[int, int]: ...

    def window_close(self, window: int) -> tuple[int, int]: ...

    def window_close_request_subscribe(
        self,
        window: int,
        callback: WindowCloseCallback,
        userdata: int,
        destroy: VoidCallback,
    ) -> tuple[int, int, int]: ...

    def window_close_request_defer(
        self, borrowed_request: int
    ) -> tuple[int, int, int]: ...

    def window_close_request_resolve(
        self, request: int, decision: int
    ) -> tuple[int, int]: ...

    def subscription_cancel(self, subscription: int) -> tuple[int, int]: ...

    def stack_panel_create(self, runtime: int) -> tuple[int, int, int]: ...

    def text_create(
        self, runtime: int, initial: bytes
    ) -> tuple[int, int, int]: ...

    def text_box_create(
        self, runtime: int, initial: bytes
    ) -> tuple[int, int, int]: ...

    def button_create(
        self, runtime: int, label: bytes
    ) -> tuple[int, int, int]: ...

    def widget_is_enabled_property(self, widget: int) -> tuple[int, int]: ...

    def widget_accessible_name_property(
        self, widget: int
    ) -> tuple[int, int]: ...

    def widget_request_focus(self, widget: int) -> tuple[int, int]: ...

    def window_content(self, window: int) -> tuple[int, int, int]: ...

    def window_set_content(
        self, window: int, child: int
    ) -> tuple[int, int]: ...

    def window_clear_content(self, window: int) -> tuple[int, int]: ...

    def window_title_property(self, window: int) -> tuple[int, int]: ...

    def container_insert(
        self, container: int, child: int, index: int
    ) -> tuple[int, int]: ...

    def container_remove(
        self, container: int, child: int
    ) -> tuple[int, int]: ...

    def container_move(
        self, container: int, from_index: int, to_index: int
    ) -> tuple[int, int]: ...

    def container_child_count(self, container: int) -> tuple[int, int]: ...

    def container_child_at(
        self, container: int, index: int
    ) -> tuple[int, int, int]: ...

    def stack_panel_orientation(self, panel: int) -> tuple[int, int]: ...

    def stack_panel_set_orientation(
        self, panel: int, orientation: int
    ) -> tuple[int, int]: ...

    def stack_panel_set_child_align(
        self, panel: int, child: int, align: int
    ) -> tuple[int, int]: ...

    def text_content_property(self, text: int) -> tuple[int, int]: ...

    def text_box_value_property(self, text_box: int) -> tuple[int, int]: ...

    def button_label_property(self, button: int) -> tuple[int, int]: ...

    def button_set_command(
        self, button: int, command: int
    ) -> tuple[int, int]: ...

    def button_command(self, button: int) -> tuple[int, int, int]: ...

    def handle_retain(self, borrowed: int) -> tuple[int, int]: ...

    def runtime_post(
        self,
        runtime: int,
        callback: VoidCallback,
        userdata: int,
        destroy: VoidCallback,
    ) -> tuple[int, int]: ...

    def command_create_async(
        self,
        runtime: int,
        eligibility: int,
        start: CommandStartCallback,
        userdata: int,
        destroy: VoidCallback,
    ) -> tuple[int, int, int]: ...

    def command_execute(self, command: int) -> tuple[int, int, int]: ...

    def command_cancel(
        self, command: int, operation_id: int
    ) -> tuple[int, int]: ...

    def command_close(self, command: int) -> tuple[int, int]: ...

    def command_state_property(self, command: int) -> tuple[int, int]: ...

    def command_can_execute_property(
        self, command: int
    ) -> tuple[int, int]: ...

    def command_failure_snapshot(self, command: int) -> tuple[int, int]: ...

    def command_cancel_token_requested(
        self, token: int
    ) -> tuple[int, int]: ...

    def command_complete_success(
        self, command: int, operation_id: int
    ) -> int: ...

    def command_complete_failure(
        self,
        command: int,
        operation_id: int,
        domain: int,
        code: int,
        message: bytes,
    ) -> int: ...

    def command_complete_cancelled(
        self, command: int, operation_id: int
    ) -> int: ...

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
            "ui_abi_1_property_create_decimal",
            [
                Handle,
                ctypes.c_uint64,
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
            "ui_abi_1_property_get_decimal",
            [Handle, ctypes.POINTER(ctypes.c_uint64)],
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
            "ui_abi_1_property_set_decimal",
            [Handle, ctypes.c_uint64, ctypes.POINTER(Handle)],
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
        self._bind(
            "ui_abi_1_binding_create",
            [
                Handle,
                Handle,
                ctypes.c_uint32,
                ctypes.POINTER(Handle),
                ctypes.POINTER(Handle),
            ],
        )
        self._bind("ui_abi_1_binding_cancel", [Handle, ctypes.POINTER(Handle)])
        self._bind(
            "ui_abi_1_binding_error_snapshot",
            [
                Handle,
                ctypes.POINTER(Handle),
                ctypes.POINTER(Handle),
                ctypes.POINTER(ctypes.c_uint32),
            ],
        )
        self._bind(
            "ui_abi_1_window_create",
            [Handle, ctypes.POINTER(Handle), ctypes.POINTER(Handle)],
        )
        self._bind("ui_abi_1_window_open", [Handle, ctypes.POINTER(Handle)])
        self._bind("ui_abi_1_window_close", [Handle, ctypes.POINTER(Handle)])
        self._bind(
            "ui_abi_1_window_close_request_subscribe",
            [
                Handle,
                WindowCloseCallback,
                ctypes.c_void_p,
                VoidCallback,
                ctypes.POINTER(Handle),
                ctypes.POINTER(Handle),
            ],
        )
        self._bind(
            "ui_abi_1_window_close_request_defer",
            [Handle, ctypes.POINTER(Handle), ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_window_close_request_resolve",
            [Handle, ctypes.c_uint32, ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_subscription_cancel", [Handle, ctypes.POINTER(Handle)]
        )
        self._bind(
            "ui_abi_1_stack_panel_create",
            [Handle, ctypes.POINTER(Handle), ctypes.POINTER(Handle)],
        )
        for name in ("text", "text_box", "button"):
            self._bind(
                f"ui_abi_1_{name}_create",
                [
                    Handle,
                    ctypes.POINTER(ctypes.c_uint8),
                    ctypes.c_uint64,
                    ctypes.POINTER(Handle),
                    ctypes.POINTER(Handle),
                ],
            )
        for name in (
            "widget_is_enabled_property",
            "widget_accessible_name_property",
            "window_title_property",
            "text_content_property",
            "text_box_value_property",
            "button_label_property",
        ):
            self._bind(f"ui_abi_1_{name}", [Handle, ctypes.POINTER(Handle)])
        self._bind(
            "ui_abi_1_widget_request_focus", [Handle, ctypes.POINTER(Handle)]
        )
        self._bind(
            "ui_abi_1_window_content",
            [Handle, ctypes.POINTER(Handle), ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_window_set_content",
            [Handle, Handle, ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_window_clear_content", [Handle, ctypes.POINTER(Handle)]
        )
        self._bind(
            "ui_abi_1_container_insert",
            [Handle, Handle, ctypes.c_uint64, ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_container_remove",
            [Handle, Handle, ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_container_move",
            [Handle, ctypes.c_uint64, ctypes.c_uint64, ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_container_child_count",
            [Handle, ctypes.POINTER(ctypes.c_uint64)],
        )
        self._bind(
            "ui_abi_1_container_child_at",
            [
                Handle,
                ctypes.c_uint64,
                ctypes.POINTER(Handle),
                ctypes.POINTER(Handle),
            ],
        )
        self._bind(
            "ui_abi_1_stack_panel_orientation",
            [Handle, ctypes.POINTER(ctypes.c_uint32)],
        )
        self._bind(
            "ui_abi_1_stack_panel_set_orientation",
            [Handle, ctypes.c_uint32, ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_stack_panel_set_child_align",
            [Handle, Handle, ctypes.c_uint32, ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_button_set_command",
            [Handle, Handle, ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_button_command",
            [Handle, ctypes.POINTER(Handle), ctypes.POINTER(Handle)],
        )
        self._bind("ui_abi_1_handle_release", [Handle])
        self._bind("ui_abi_1_handle_retain", [Handle, ctypes.POINTER(Handle)])
        self._bind(
            "ui_abi_1_command_create_async",
            [
                Handle,
                Handle,
                CommandStartCallback,
                ctypes.c_void_p,
                VoidCallback,
                ctypes.POINTER(Handle),
                ctypes.POINTER(Handle),
            ],
        )
        self._bind(
            "ui_abi_1_command_execute",
            [Handle, ctypes.POINTER(ctypes.c_uint64), ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_command_cancel",
            [Handle, ctypes.c_uint64, ctypes.POINTER(Handle)],
        )
        self._bind("ui_abi_1_command_close", [Handle, ctypes.POINTER(Handle)])
        self._bind(
            "ui_abi_1_command_state_property", [Handle, ctypes.POINTER(Handle)]
        )
        self._bind(
            "ui_abi_1_command_can_execute_property",
            [Handle, ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_command_failure_snapshot",
            [Handle, ctypes.POINTER(Handle)],
        )
        self._bind(
            "ui_abi_1_command_cancel_token_requested",
            [Handle, ctypes.POINTER(Bool)],
        )
        self._bind(
            "ui_abi_1_command_complete_success", [Handle, ctypes.c_uint64]
        )
        self._bind(
            "ui_abi_1_command_complete_failure_utf8",
            [
                Handle,
                ctypes.c_uint64,
                ctypes.c_uint32,
                ctypes.c_uint32,
                ctypes.POINTER(ctypes.c_uint8),
                ctypes.c_uint64,
            ],
        )
        self._bind(
            "ui_abi_1_command_complete_cancelled", [Handle, ctypes.c_uint64]
        )
        self._bind(
            "ui_abi_1_runtime_post",
            [
                Handle,
                VoidCallback,
                ctypes.c_void_p,
                VoidCallback,
                ctypes.POINTER(Handle),
            ],
        )
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

    def property_create_decimal(
        self, runtime: int, initial_bits: int
    ) -> tuple[int, int, int]:
        property_handle = Handle()
        error = Handle()
        status = self._library.ui_abi_1_property_create_decimal(
            runtime,
            initial_bits,
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

    def property_get_decimal(self, property: int) -> tuple[int, int]:
        bits = ctypes.c_uint64()
        status = self._library.ui_abi_1_property_get_decimal(
            property, ctypes.byref(bits)
        )
        return status, bits.value

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

    def property_set_decimal(
        self, property: int, bits: int
    ) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_property_set_decimal(
            property, bits, ctypes.byref(error)
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

    def binding_create(
        self, source: int, target: int, mode: int
    ) -> tuple[int, int, int]:
        binding = Handle()
        error = Handle()
        status = self._library.ui_abi_1_binding_create(
            source,
            target,
            mode,
            ctypes.byref(binding),
            ctypes.byref(error),
        )
        return status, binding.value, error.value

    def binding_cancel(self, binding: int) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_binding_cancel(
            binding, ctypes.byref(error)
        )
        return status, error.value

    def binding_error_snapshot(
        self, binding: int
    ) -> tuple[int, int, int, int]:
        error = Handle()
        rejected_property = Handle()
        direction = ctypes.c_uint32()
        status = self._library.ui_abi_1_binding_error_snapshot(
            binding,
            ctypes.byref(error),
            ctypes.byref(rejected_property),
            ctypes.byref(direction),
        )
        return status, error.value, rejected_property.value, direction.value

    @staticmethod
    def _utf8_arguments(value: bytes) -> tuple[object, object, int]:
        storage = (ctypes.c_uint8 * len(value)).from_buffer_copy(value)
        pointer = (
            ctypes.cast(storage, ctypes.POINTER(ctypes.c_uint8))
            if value
            else ctypes.POINTER(ctypes.c_uint8)()
        )
        return storage, pointer, len(value)

    def _create_utf8_control(
        self, name: str, runtime: int, initial: bytes
    ) -> tuple[int, int, int]:
        _storage, pointer, length = self._utf8_arguments(initial)
        token = Handle()
        error = Handle()
        status = getattr(self._library, f"ui_abi_1_{name}_create")(
            runtime, pointer, length, ctypes.byref(token), ctypes.byref(error)
        )
        return status, token.value, error.value

    def window_create(self, runtime: int) -> tuple[int, int, int]:
        token = Handle()
        error = Handle()
        status = self._library.ui_abi_1_window_create(
            runtime, ctypes.byref(token), ctypes.byref(error)
        )
        return status, token.value, error.value

    def stack_panel_create(self, runtime: int) -> tuple[int, int, int]:
        token = Handle()
        error = Handle()
        status = self._library.ui_abi_1_stack_panel_create(
            runtime, ctypes.byref(token), ctypes.byref(error)
        )
        return status, token.value, error.value

    def text_create(
        self, runtime: int, initial: bytes
    ) -> tuple[int, int, int]:
        return self._create_utf8_control("text", runtime, initial)

    def text_box_create(
        self, runtime: int, initial: bytes
    ) -> tuple[int, int, int]:
        return self._create_utf8_control("text_box", runtime, initial)

    def button_create(
        self, runtime: int, label: bytes
    ) -> tuple[int, int, int]:
        return self._create_utf8_control("button", runtime, label)

    def _owned_property(self, name: str, handle: int) -> tuple[int, int]:
        token = Handle()
        status = getattr(self._library, f"ui_abi_1_{name}")(
            handle, ctypes.byref(token)
        )
        return status, token.value

    def widget_is_enabled_property(self, widget: int) -> tuple[int, int]:
        return self._owned_property("widget_is_enabled_property", widget)

    def widget_accessible_name_property(self, widget: int) -> tuple[int, int]:
        return self._owned_property("widget_accessible_name_property", widget)

    def widget_request_focus(self, widget: int) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_widget_request_focus(
            widget, ctypes.byref(error)
        )
        return status, error.value

    def window_content(self, window: int) -> tuple[int, int, int]:
        child = Handle()
        error = Handle()
        status = self._library.ui_abi_1_window_content(
            window, ctypes.byref(child), ctypes.byref(error)
        )
        return status, child.value, error.value

    def window_open(self, window: int) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_window_open(
            window, ctypes.byref(error)
        )
        return status, error.value

    def window_close(self, window: int) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_window_close(
            window, ctypes.byref(error)
        )
        return status, error.value

    def window_close_request_subscribe(
        self,
        window: int,
        callback: WindowCloseCallback,
        userdata: int,
        destroy: VoidCallback,
    ) -> tuple[int, int, int]:
        subscription = Handle()
        error = Handle()
        status = self._library.ui_abi_1_window_close_request_subscribe(
            window,
            callback,
            ctypes.c_void_p(userdata),
            destroy,
            ctypes.byref(subscription),
            ctypes.byref(error),
        )
        return status, subscription.value, error.value

    def window_close_request_defer(
        self, borrowed_request: int
    ) -> tuple[int, int, int]:
        request = Handle()
        error = Handle()
        status = self._library.ui_abi_1_window_close_request_defer(
            borrowed_request, ctypes.byref(request), ctypes.byref(error)
        )
        return status, request.value, error.value

    def window_close_request_resolve(
        self, request: int, decision: int
    ) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_window_close_request_resolve(
            request, decision, ctypes.byref(error)
        )
        return status, error.value

    def subscription_cancel(self, subscription: int) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_subscription_cancel(
            subscription, ctypes.byref(error)
        )
        return status, error.value

    def window_set_content(self, window: int, child: int) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_window_set_content(
            window, child, ctypes.byref(error)
        )
        return status, error.value

    def window_clear_content(self, window: int) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_window_clear_content(
            window, ctypes.byref(error)
        )
        return status, error.value

    def window_title_property(self, window: int) -> tuple[int, int]:
        return self._owned_property("window_title_property", window)

    def container_insert(
        self, container: int, child: int, index: int
    ) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_container_insert(
            container, child, index, ctypes.byref(error)
        )
        return status, error.value

    def container_remove(self, container: int, child: int) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_container_remove(
            container, child, ctypes.byref(error)
        )
        return status, error.value

    def container_move(
        self, container: int, from_index: int, to_index: int
    ) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_container_move(
            container, from_index, to_index, ctypes.byref(error)
        )
        return status, error.value

    def container_child_count(self, container: int) -> tuple[int, int]:
        count = ctypes.c_uint64()
        status = self._library.ui_abi_1_container_child_count(
            container, ctypes.byref(count)
        )
        return status, count.value

    def container_child_at(
        self, container: int, index: int
    ) -> tuple[int, int, int]:
        child = Handle()
        error = Handle()
        status = self._library.ui_abi_1_container_child_at(
            container, index, ctypes.byref(child), ctypes.byref(error)
        )
        return status, child.value, error.value

    def stack_panel_orientation(self, panel: int) -> tuple[int, int]:
        orientation = ctypes.c_uint32()
        status = self._library.ui_abi_1_stack_panel_orientation(
            panel, ctypes.byref(orientation)
        )
        return status, orientation.value

    def stack_panel_set_orientation(
        self, panel: int, orientation: int
    ) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_stack_panel_set_orientation(
            panel, orientation, ctypes.byref(error)
        )
        return status, error.value

    def stack_panel_set_child_align(
        self, panel: int, child: int, align: int
    ) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_stack_panel_set_child_align(
            panel, child, align, ctypes.byref(error)
        )
        return status, error.value

    def text_content_property(self, text: int) -> tuple[int, int]:
        return self._owned_property("text_content_property", text)

    def text_box_value_property(self, text_box: int) -> tuple[int, int]:
        return self._owned_property("text_box_value_property", text_box)

    def button_label_property(self, button: int) -> tuple[int, int]:
        return self._owned_property("button_label_property", button)

    def button_set_command(self, button: int, command: int) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_button_set_command(
            button, command, ctypes.byref(error)
        )
        return status, error.value

    def button_command(self, button: int) -> tuple[int, int, int]:
        command = Handle()
        error = Handle()
        status = self._library.ui_abi_1_button_command(
            button, ctypes.byref(command), ctypes.byref(error)
        )
        return status, command.value, error.value

    def handle_release(self, owned: int) -> int:
        return self._library.ui_abi_1_handle_release(owned)

    def handle_retain(self, borrowed: int) -> tuple[int, int]:
        owned = Handle()
        status = self._library.ui_abi_1_handle_retain(
            borrowed, ctypes.byref(owned)
        )
        return status, owned.value

    def command_create_async(
        self,
        runtime: int,
        eligibility: int,
        start: CommandStartCallback,
        userdata: int,
        destroy: VoidCallback,
    ) -> tuple[int, int, int]:
        command = Handle()
        error = Handle()
        status = self._library.ui_abi_1_command_create_async(
            runtime,
            eligibility,
            start,
            ctypes.c_void_p(userdata),
            destroy,
            ctypes.byref(command),
            ctypes.byref(error),
        )
        return status, command.value, error.value

    def command_execute(self, command: int) -> tuple[int, int, int]:
        operation_id = ctypes.c_uint64()
        error = Handle()
        status = self._library.ui_abi_1_command_execute(
            command, ctypes.byref(operation_id), ctypes.byref(error)
        )
        return status, operation_id.value, error.value

    def command_cancel(
        self, command: int, operation_id: int
    ) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_command_cancel(
            command, operation_id, ctypes.byref(error)
        )
        return status, error.value

    def command_close(self, command: int) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_command_close(
            command, ctypes.byref(error)
        )
        return status, error.value

    def command_state_property(self, command: int) -> tuple[int, int]:
        token = Handle()
        status = self._library.ui_abi_1_command_state_property(
            command, ctypes.byref(token)
        )
        return status, token.value

    def command_can_execute_property(self, command: int) -> tuple[int, int]:
        token = Handle()
        status = self._library.ui_abi_1_command_can_execute_property(
            command, ctypes.byref(token)
        )
        return status, token.value

    def command_failure_snapshot(self, command: int) -> tuple[int, int]:
        token = Handle()
        status = self._library.ui_abi_1_command_failure_snapshot(
            command, ctypes.byref(token)
        )
        return status, token.value

    def command_cancel_token_requested(self, token: int) -> tuple[int, int]:
        requested = Bool()
        status = self._library.ui_abi_1_command_cancel_token_requested(
            token, ctypes.byref(requested)
        )
        return status, requested.value

    def command_complete_success(self, command: int, operation_id: int) -> int:
        return self._library.ui_abi_1_command_complete_success(
            command, operation_id
        )

    def command_complete_failure(
        self,
        command: int,
        operation_id: int,
        domain: int,
        code: int,
        message: bytes,
    ) -> int:
        storage = (ctypes.c_uint8 * len(message)).from_buffer_copy(message)
        pointer = (
            ctypes.cast(storage, ctypes.POINTER(ctypes.c_uint8))
            if message
            else ctypes.POINTER(ctypes.c_uint8)()
        )
        return self._library.ui_abi_1_command_complete_failure_utf8(
            command, operation_id, domain, code, pointer, len(message)
        )

    def command_complete_cancelled(
        self, command: int, operation_id: int
    ) -> int:
        return self._library.ui_abi_1_command_complete_cancelled(
            command, operation_id
        )

    def runtime_post(
        self,
        runtime: int,
        callback: VoidCallback,
        userdata: int,
        destroy: VoidCallback,
    ) -> tuple[int, int]:
        error = Handle()
        status = self._library.ui_abi_1_runtime_post(
            runtime,
            callback,
            ctypes.c_void_p(userdata),
            destroy,
            ctypes.byref(error),
        )
        return status, error.value

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

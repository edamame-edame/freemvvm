"""Thin ctypes facade for FreeMVVM, including the WebView control plugin."""

from __future__ import annotations

from collections.abc import Callable
import ctypes
from dataclasses import dataclass
from enum import IntEnum
import threading
from typing import Any

UI_OK = 0
UI_INVALID_HANDLE = 1
UI_INVALID_ARGUMENT = 2
UI_WRONG_TYPE = 3
UI_WRONG_RUNTIME = 4
UI_WRONG_THREAD = 5
UI_CLOSED = 6
UI_REENTRANT_CALL = 7
UI_BUFFER_TOO_SMALL = 8
UI_OUT_OF_MEMORY = 9
UI_PENDING = 10
UI_VALIDATION_FAILED = 11
UI_INTERNAL_ERROR = 255

POST_CALLBACK_TYPE = ctypes.CFUNCTYPE(None, ctypes.c_void_p)
DESTROY_CALLBACK_TYPE = ctypes.CFUNCTYPE(None, ctypes.c_void_p)


class WebViewFormat(IntEnum):
    """Document format accepted by WebView."""

    HTML = 1
    MARKDOWN = 2


@dataclass(frozen=True)
class UiError(Exception):
    """Diagnostic error returned across the FreeMVVM C ABI boundary."""

    raw_status: int
    domain: int | None = None
    code: int | None = None


def bind_webview_abi(lib: ctypes.CDLL) -> ctypes.CDLL:
    """Bind fixed-width ctypes signatures for WebView C ABI entry points."""
    u64_p = ctypes.POINTER(ctypes.c_uint64)
    u32_p = ctypes.POINTER(ctypes.c_uint32)
    u8_p = ctypes.POINTER(ctypes.c_uint8)

    lib.ui_abi_1_handle_retain.argtypes = [ctypes.c_uint64, u64_p]
    lib.ui_abi_1_handle_retain.restype = ctypes.c_uint32

    lib.ui_abi_1_handle_release.argtypes = [ctypes.c_uint64]
    lib.ui_abi_1_handle_release.restype = ctypes.c_uint32

    lib.ui_abi_1_error_code.argtypes = [ctypes.c_uint64, u32_p, u32_p]
    lib.ui_abi_1_error_code.restype = ctypes.c_uint32

    lib.ui_abi_1_control_create_webview.argtypes = [
        ctypes.c_uint64,
        u64_p,
        u64_p,
    ]
    lib.ui_abi_1_control_create_webview.restype = ctypes.c_uint32

    lib.ui_abi_1_webview_set_source_utf8.argtypes = [
        ctypes.c_uint64,
        ctypes.c_uint32,
        u8_p,
        ctypes.c_uint64,
        u64_p,
    ]
    lib.ui_abi_1_webview_set_source_utf8.restype = ctypes.c_uint32

    lib.ui_abi_1_webview_set_stylesheet_utf8.argtypes = [
        ctypes.c_uint64,
        u8_p,
        ctypes.c_uint64,
        u64_p,
    ]
    lib.ui_abi_1_webview_set_stylesheet_utf8.restype = ctypes.c_uint32

    lib.ui_abi_1_webview_set_viewport.argtypes = [
        ctypes.c_uint64,
        ctypes.c_double,
        ctypes.c_double,
        u64_p,
    ]
    lib.ui_abi_1_webview_set_viewport.restype = ctypes.c_uint32

    lib.ui_abi_1_webview_bind_source_property.argtypes = [
        ctypes.c_uint64,
        ctypes.c_uint32,
        ctypes.c_uint64,
        u64_p,
        u64_p,
    ]
    lib.ui_abi_1_webview_bind_source_property.restype = ctypes.c_uint32

    lib.ui_abi_1_webview_expose_property_utf8.argtypes = [
        ctypes.c_uint64,
        u8_p,
        ctypes.c_uint64,
        ctypes.c_uint64,
        ctypes.c_uint8,
        u64_p,
    ]
    lib.ui_abi_1_webview_expose_property_utf8.restype = ctypes.c_uint32

    lib.ui_abi_1_webview_expose_command_utf8.argtypes = [
        ctypes.c_uint64,
        u8_p,
        ctypes.c_uint64,
        POST_CALLBACK_TYPE,
        ctypes.c_void_p,
        DESTROY_CALLBACK_TYPE,
        u64_p,
    ]
    lib.ui_abi_1_webview_expose_command_utf8.restype = ctypes.c_uint32

    lib.ui_abi_1_webview_eval_script_utf8.argtypes = [
        ctypes.c_uint64,
        u8_p,
        ctypes.c_uint64,
        u64_p,
    ]
    lib.ui_abi_1_webview_eval_script_utf8.restype = ctypes.c_uint32

    lib.ui_abi_1_webview_click_at.argtypes = [
        ctypes.c_uint64,
        ctypes.c_double,
        ctypes.c_double,
        u64_p,
    ]
    lib.ui_abi_1_webview_click_at.restype = ctypes.c_uint32

    lib.ui_abi_1_webview_text_content_utf8.argtypes = [
        ctypes.c_uint64,
        u8_p,
        ctypes.c_uint64,
        u64_p,
        u64_p,
    ]
    lib.ui_abi_1_webview_text_content_utf8.restype = ctypes.c_uint32

    lib.ui_abi_1_webview_render_rgba.argtypes = [
        ctypes.c_uint64,
        ctypes.c_uint32,
        ctypes.c_uint32,
        u8_p,
        ctypes.c_uint64,
        u64_p,
        u64_p,
    ]
    lib.ui_abi_1_webview_render_rgba.restype = ctypes.c_uint32
    return lib


class _CallbackRegistry:
    """Thread-safe strong root table for C ABI callback closures."""

    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._next_id = 1
        self._active: dict[int, Callable[[], None]] = {}
        self._release_queue: list[Callable[[], None]] = []

    def register(self, fn: Callable[[], None]) -> int:
        with self._lock:
            key = self._next_id
            self._next_id += 1
            self._active[key] = fn
            return key

    def discard(self, key: int) -> None:
        with self._lock:
            self._active.pop(key, None)

    def invoke(self, key: int) -> None:
        with self._lock:
            fn = self._active.get(key)
        if fn is None:
            return
        try:
            fn()
        except BaseException:
            # Never unwind across the C ABI boundary.
            return

    def schedule_destroy(self, key: int) -> None:
        with self._lock:
            fn = self._active.pop(key, None)
            if fn is not None:
                self._release_queue.append(fn)

    def drain_released(self) -> None:
        with self._lock:
            pending = list(self._release_queue)
            self._release_queue.clear()
        del pending


_GLOBAL_CALLBACKS = _CallbackRegistry()


@POST_CALLBACK_TYPE
def _c_command_invoke(userdata: int | None) -> None:
    if userdata:
        _GLOBAL_CALLBACKS.invoke(int(userdata))


@DESTROY_CALLBACK_TYPE
def _c_command_destroy(userdata: int | None) -> None:
    if userdata:
        _GLOBAL_CALLBACKS.schedule_destroy(int(userdata))


def _make_error(lib: Any, status: int, error_token: int) -> UiError:
    if error_token == 0:
        return UiError(raw_status=status)
    domain = ctypes.c_uint32(0)
    code = ctypes.c_uint32(0)
    code_status = int(
        lib.ui_abi_1_error_code(
            ctypes.c_uint64(error_token),
            ctypes.byref(domain),
            ctypes.byref(code),
        )
    )
    lib.ui_abi_1_handle_release(ctypes.c_uint64(error_token))
    if code_status == UI_OK:
        return UiError(
            raw_status=status,
            domain=int(domain.value),
            code=int(code.value),
        )
    return UiError(raw_status=status)


def _utf8_buf(text: str) -> tuple[Any, int]:
    raw = text.encode("utf-8")
    if not raw:
        return None, 0
    arr = (ctypes.c_uint8 * len(raw)).from_buffer_copy(raw)
    return ctypes.cast(arr, ctypes.POINTER(ctypes.c_uint8)), len(raw)


class WebViewBinding:
    """Owned handle for a OneWay source property binding on a WebView."""

    def __init__(self, token: int, lib: Any) -> None:
        self._lock = threading.Lock()
        self._token = token
        self._lib = lib

    def close(self) -> None:
        with self._lock:
            token = self._token
            if token == 0:
                return
        status = int(self._lib.ui_abi_1_handle_release(ctypes.c_uint64(token)))
        if status != UI_OK:
            raise UiError(raw_status=status)
        with self._lock:
            self._token = 0

    def __enter__(self) -> WebViewBinding:
        return self

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        self.close()


class WebView:
    """Safe Python facade for the FreeMVVM WebView control."""

    def __init__(self, token: int, lib: Any) -> None:
        self._lock = threading.Lock()
        self._token = token
        self._lib = lib
        self._ui_thread_id = threading.get_ident()
        self._deferred_tokens: list[int] = []

    @classmethod
    def create(cls, runtime_token: int, lib: Any) -> WebView:
        if runtime_token == 0:
            raise UiError(raw_status=UI_INVALID_ARGUMENT)
        out_webview = ctypes.c_uint64(0)
        out_error = ctypes.c_uint64(0)
        status = int(
            lib.ui_abi_1_control_create_webview(
                ctypes.c_uint64(runtime_token),
                ctypes.byref(out_webview),
                ctypes.byref(out_error),
            )
        )
        if status != UI_OK:
            raise _make_error(lib, status, int(out_error.value))
        if out_webview.value == 0:
            raise UiError(raw_status=UI_INTERNAL_ERROR)
        return cls(int(out_webview.value), lib)

    def _require_open_token(self) -> int:
        if threading.get_ident() != self._ui_thread_id:
            raise UiError(raw_status=UI_WRONG_THREAD)
        with self._lock:
            if self._token == 0:
                raise UiError(raw_status=UI_CLOSED)
            return self._token

    def set_source(self, fmt: WebViewFormat, source: str) -> None:
        token = self._require_open_token()
        ptr, length = _utf8_buf(source)
        out_error = ctypes.c_uint64(0)
        status = int(
            self._lib.ui_abi_1_webview_set_source_utf8(
                ctypes.c_uint64(token),
                ctypes.c_uint32(int(fmt)),
                ptr,
                ctypes.c_uint64(length),
                ctypes.byref(out_error),
            )
        )
        if status != UI_OK:
            raise _make_error(self._lib, status, int(out_error.value))

    def set_stylesheet(self, css: str) -> None:
        token = self._require_open_token()
        ptr, length = _utf8_buf(css)
        out_error = ctypes.c_uint64(0)
        status = int(
            self._lib.ui_abi_1_webview_set_stylesheet_utf8(
                ctypes.c_uint64(token),
                ptr,
                ctypes.c_uint64(length),
                ctypes.byref(out_error),
            )
        )
        if status != UI_OK:
            raise _make_error(self._lib, status, int(out_error.value))

    def set_viewport(self, width_dip: float, height_dip: float) -> None:
        token = self._require_open_token()
        out_error = ctypes.c_uint64(0)
        status = int(
            self._lib.ui_abi_1_webview_set_viewport(
                ctypes.c_uint64(token),
                ctypes.c_double(width_dip),
                ctypes.c_double(height_dip),
                ctypes.byref(out_error),
            )
        )
        if status != UI_OK:
            raise _make_error(self._lib, status, int(out_error.value))

    def bind_source_property(
        self, fmt: WebViewFormat, utf8_property_token: int
    ) -> WebViewBinding:
        token = self._require_open_token()
        out_binding = ctypes.c_uint64(0)
        out_error = ctypes.c_uint64(0)
        status = int(
            self._lib.ui_abi_1_webview_bind_source_property(
                ctypes.c_uint64(token),
                ctypes.c_uint32(int(fmt)),
                ctypes.c_uint64(utf8_property_token),
                ctypes.byref(out_binding),
                ctypes.byref(out_error),
            )
        )
        if status != UI_OK:
            raise _make_error(self._lib, status, int(out_error.value))
        return WebViewBinding(int(out_binding.value), self._lib)

    def expose_property(
        self, alias: str, property_token: int, allow_write: bool
    ) -> None:
        token = self._require_open_token()
        ptr, length = _utf8_buf(alias)
        out_error = ctypes.c_uint64(0)
        status = int(
            self._lib.ui_abi_1_webview_expose_property_utf8(
                ctypes.c_uint64(token),
                ptr,
                ctypes.c_uint64(length),
                ctypes.c_uint64(property_token),
                ctypes.c_uint8(1 if allow_write else 0),
                ctypes.byref(out_error),
            )
        )
        if status != UI_OK:
            raise _make_error(self._lib, status, int(out_error.value))

    def expose_command(self, alias: str, callback: Callable[[], None]) -> None:
        token = self._require_open_token()
        ptr, length = _utf8_buf(alias)
        key = _GLOBAL_CALLBACKS.register(callback)
        out_error = ctypes.c_uint64(0)
        status = int(
            self._lib.ui_abi_1_webview_expose_command_utf8(
                ctypes.c_uint64(token),
                ptr,
                ctypes.c_uint64(length),
                _c_command_invoke,
                ctypes.c_void_p(key),
                _c_command_destroy,
                ctypes.byref(out_error),
            )
        )
        if status != UI_OK:
            _GLOBAL_CALLBACKS.discard(key)
            raise _make_error(self._lib, status, int(out_error.value))

    def eval_script(self, script: str) -> None:
        token = self._require_open_token()
        ptr, length = _utf8_buf(script)
        out_error = ctypes.c_uint64(0)
        status = int(
            self._lib.ui_abi_1_webview_eval_script_utf8(
                ctypes.c_uint64(token),
                ptr,
                ctypes.c_uint64(length),
                ctypes.byref(out_error),
            )
        )
        _GLOBAL_CALLBACKS.drain_released()
        if status != UI_OK:
            raise _make_error(self._lib, status, int(out_error.value))

    def click_at(self, x_dip: float, y_dip: float) -> None:
        token = self._require_open_token()
        out_error = ctypes.c_uint64(0)
        status = int(
            self._lib.ui_abi_1_webview_click_at(
                ctypes.c_uint64(token),
                ctypes.c_double(x_dip),
                ctypes.c_double(y_dip),
                ctypes.byref(out_error),
            )
        )
        _GLOBAL_CALLBACKS.drain_released()
        if status != UI_OK:
            raise _make_error(self._lib, status, int(out_error.value))

    def text_content(self) -> str:
        token = self._require_open_token()
        required = ctypes.c_uint64(0)
        out_error = ctypes.c_uint64(0)
        status = int(
            self._lib.ui_abi_1_webview_text_content_utf8(
                ctypes.c_uint64(token),
                None,
                ctypes.c_uint64(0),
                ctypes.byref(required),
                ctypes.byref(out_error),
            )
        )
        if status != UI_OK:
            raise _make_error(self._lib, status, int(out_error.value))
        cap = int(required.value)
        if cap == 0:
            return ""
        buf = (ctypes.c_uint8 * cap)()
        written = ctypes.c_uint64(0)
        copy_status = int(
            self._lib.ui_abi_1_webview_text_content_utf8(
                ctypes.c_uint64(token),
                ctypes.cast(buf, ctypes.POINTER(ctypes.c_uint8)),
                ctypes.c_uint64(cap),
                ctypes.byref(written),
                ctypes.byref(out_error),
            )
        )
        if copy_status != UI_OK:
            raise _make_error(self._lib, copy_status, int(out_error.value))
        return bytes(buf[: int(written.value)]).decode("utf-8")

    def render_rgba(self, width_px: int, height_px: int) -> bytes:
        token = self._require_open_token()
        required = ctypes.c_uint64(0)
        out_error = ctypes.c_uint64(0)
        status = int(
            self._lib.ui_abi_1_webview_render_rgba(
                ctypes.c_uint64(token),
                ctypes.c_uint32(width_px),
                ctypes.c_uint32(height_px),
                None,
                ctypes.c_uint64(0),
                ctypes.byref(required),
                ctypes.byref(out_error),
            )
        )
        if status != UI_OK:
            raise _make_error(self._lib, status, int(out_error.value))
        cap = int(required.value)
        buf = (ctypes.c_uint8 * cap)()
        written = ctypes.c_uint64(0)
        copy_status = int(
            self._lib.ui_abi_1_webview_render_rgba(
                ctypes.c_uint64(token),
                ctypes.c_uint32(width_px),
                ctypes.c_uint32(height_px),
                ctypes.cast(buf, ctypes.POINTER(ctypes.c_uint8)),
                ctypes.c_uint64(cap),
                ctypes.byref(written),
                ctypes.byref(out_error),
            )
        )
        if copy_status != UI_OK:
            raise _make_error(self._lib, copy_status, int(out_error.value))
        return bytes(buf[: int(written.value)])

    def close(self) -> None:
        with self._lock:
            token = self._token
            if token == 0:
                return
        status = int(self._lib.ui_abi_1_handle_release(ctypes.c_uint64(token)))
        _GLOBAL_CALLBACKS.drain_released()
        if status != UI_OK:
            self._deferred_tokens.append(token)
            raise UiError(raw_status=status)
        with self._lock:
            self._token = 0

    def __enter__(self) -> WebView:
        return self

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        self.close()

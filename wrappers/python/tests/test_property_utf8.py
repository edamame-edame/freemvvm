"""Focused tests for the Python UTF-8 Property facade."""

from __future__ import annotations

import threading

import pytest

from freemvvm import ErrorSnapshot, Property, Runtime, UiError
from freemvvm import _native


class FakeUtf8Api:
    def __init__(self) -> None:
        self.calls: list[tuple[str, tuple[object, ...]]] = []
        self.releases: list[int] = []
        self.release_statuses: list[int] = []
        self.is_ui_thread = (_native.OK, 1)
        self.create_utf8 = (_native.OK, 201, 0)
        self.value = b""
        self.get_statuses: list[int] = []
        self.get_values: list[bytes] = []
        self.set_utf8 = (_native.OK, 0)
        self.error_status_value = (_native.OK, _native.INVALID_ARGUMENT)
        self.error_code_value = (_native.OK, 0x1234, 0x5678)
        self.error_message = "property rejected"
        self.get_entered: threading.Event | None = None
        self.get_continue: threading.Event | None = None

    def get_version(self) -> tuple[int, int, int, int]:
        return _native.OK, 1, 0, 0

    def runtime_create(self) -> tuple[int, int, int]:
        return _native.OK, 41, 0

    def runtime_is_ui_thread(self, runtime: int) -> tuple[int, int]:
        self.calls.append(("runtime_is_ui_thread", (runtime,)))
        return self.is_ui_thread

    def runtime_shutdown(self, runtime: int) -> tuple[int, int]:
        return _native.OK, 0

    def runtime_request_shutdown(
        self, runtime: int, mode: int
    ) -> tuple[int, int, int]:
        return _native.UNSUPPORTED, 0, 0

    def shutdown_request_query(
        self, request: int
    ) -> tuple[int, int, int, int]:
        return _native.UNSUPPORTED, 0, 0, 0

    def property_create_bool(
        self, runtime: int, initial: int
    ) -> tuple[int, int, int]:
        raise AssertionError("unexpected BOOL create")

    def property_create_i64(
        self, runtime: int, initial: int
    ) -> tuple[int, int, int]:
        raise AssertionError("unexpected I64 create")

    def property_create_utf8(
        self, runtime: int, initial: bytes
    ) -> tuple[int, int, int]:
        self.calls.append(("property_create_utf8", (runtime, initial)))
        self.value = initial
        return self.create_utf8

    def property_get_bool(self, property: int) -> tuple[int, int]:
        raise AssertionError("unexpected BOOL get")

    def property_get_i64(self, property: int) -> tuple[int, int]:
        raise AssertionError("unexpected I64 get")

    def property_get_utf8(
        self, property: int, buffer: bytearray | None
    ) -> tuple[int, int]:
        if self.get_entered is not None:
            self.get_entered.set()
        if self.get_continue is not None:
            self.get_continue.wait(timeout=2)
        self.calls.append(
            (
                "property_get_utf8",
                (property, None if buffer is None else len(buffer)),
            )
        )
        status = self.get_statuses.pop(0) if self.get_statuses else None
        value = self.get_values.pop(0) if self.get_values else self.value
        if buffer is None:
            return (
                status if status is not None else _native.BUFFER_TOO_SMALL
            ), len(value)
        if status is not None and status != _native.OK:
            return status, len(value)
        if len(value) > len(buffer):
            return _native.BUFFER_TOO_SMALL, len(value)
        buffer[: len(value)] = value
        return _native.OK, len(value)

    def property_set_bool(self, property: int, value: int) -> tuple[int, int]:
        raise AssertionError("unexpected BOOL set")

    def property_set_i64(self, property: int, value: int) -> tuple[int, int]:
        raise AssertionError("unexpected I64 set")

    def property_set_utf8(
        self, property: int, value: bytes
    ) -> tuple[int, int]:
        self.calls.append(("property_set_utf8", (property, value)))
        status, error = self.set_utf8
        if status == _native.OK:
            self.value = value
        return status, error

    def handle_release(self, owned: int) -> int:
        self.releases.append(owned)
        return (
            self.release_statuses.pop(0)
            if self.release_statuses
            else _native.OK
        )

    def error_status(self, error: int) -> tuple[int, int]:
        return self.error_status_value

    def error_code(self, error: int) -> tuple[int, int, int]:
        return self.error_code_value

    def error_message_utf8(
        self, error: int, buffer: bytearray | None
    ) -> tuple[int, int]:
        encoded = self.error_message.encode("utf-8")
        if buffer is None or len(buffer) < len(encoded):
            return _native.BUFFER_TOO_SMALL, len(encoded)
        buffer[: len(encoded)] = encoded
        return _native.OK, len(encoded)


def create_property(
    initial: str = "",
) -> tuple[Runtime, Property, FakeUtf8Api]:
    api = FakeUtf8Api()
    runtime = Runtime._create_with_api(api)
    return runtime, Property.create(runtime, initial), api


@pytest.mark.parametrize("value", ["", "ascii", "日本語", "🙂", "a\0b"])
def test_create_set_get_preserves_utf8_bytes(value: str) -> None:
    runtime, prop, api = create_property(value)
    encoded = value.encode("utf-8")
    assert api.calls[-1] == ("property_create_utf8", (41, encoded))
    assert prop.get() == value
    prop.set(value)
    assert api.calls[-1] == ("property_set_utf8", (201, encoded))
    assert prop.get() == value
    prop.close()
    runtime.close()


def test_create_rejects_non_builtin_str_and_surrogate_before_native_call() -> (
    None
):
    class StrSubclass(str):
        pass

    api = FakeUtf8Api()
    runtime = Runtime._create_with_api(api)
    with pytest.raises(TypeError):
        Property.create(runtime, StrSubclass("value"))  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        Property.create(runtime, b"bytes")  # type: ignore[arg-type]
    with pytest.raises(UnicodeEncodeError):
        Property.create(runtime, "\ud800")
    assert not any(name == "property_create_utf8" for name, _ in api.calls)
    runtime.close()


def test_set_rejects_wrong_type_and_surrogate_before_native_call() -> None:
    runtime, prop, api = create_property("initial")

    class StrSubclass(str):
        pass

    before = len(api.calls)
    with pytest.raises(TypeError):
        prop.set(1)  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        prop.set(StrSubclass("value"))  # type: ignore[arg-type]
    with pytest.raises(UnicodeEncodeError):
        prop.set("\udfff")
    assert len(api.calls) == before
    prop.close()
    runtime.close()


def test_get_requeries_and_retries_when_value_grows() -> None:
    runtime, prop, api = create_property("x")
    api.get_statuses = [
        _native.BUFFER_TOO_SMALL,
        _native.BUFFER_TOO_SMALL,
        _native.BUFFER_TOO_SMALL,
        _native.OK,
    ]
    api.get_values = [b"x", b"x", b"longer", b"longer"]
    assert prop.get() == "longer"
    assert [
        args[1] for name, args in api.calls if name == "property_get_utf8"
    ] == [None, 1, None, 6]
    prop.close()
    runtime.close()


@pytest.mark.parametrize(
    "payload",
    [b"\xff", b"\xc3"],
)
def test_get_rejects_invalid_or_truncated_utf8(payload: bytes) -> None:
    runtime, prop, api = create_property()
    api.value = payload
    with pytest.raises(UiError) as raised:
        prop.get()
    assert raised.value.snapshot == ErrorSnapshot(_native.INTERNAL_ERROR)
    prop.close()
    runtime.close()


def test_get_rejects_native_length_larger_than_buffer() -> None:
    runtime, prop, api = create_property("x")
    api.get_statuses = [_native.OK, _native.OK]
    api.get_values = [b"x", b"x"]
    original = api.property_get_utf8

    def contradictory_length(
        property: int, buffer: bytearray | None
    ) -> tuple[int, int]:
        status, length = original(property, buffer)
        return status, length + 1 if buffer is not None else length

    api.property_get_utf8 = contradictory_length  # type: ignore[method-assign]
    with pytest.raises(UiError) as raised:
        prop.get()
    assert raised.value.raw_status == _native.INTERNAL_ERROR
    prop.close()
    runtime.close()


def test_native_set_error_is_snapshotted_and_wrong_thread_is_rejected() -> (
    None
):
    runtime, prop, api = create_property("before")
    api.set_utf8 = (_native.INVALID_ARGUMENT, 902)
    with pytest.raises(UiError) as raised:
        prop.set("after")
    assert raised.value.snapshot == ErrorSnapshot(
        _native.INVALID_ARGUMENT, 0x1234, 0x5678, "property rejected"
    )
    assert 902 in api.releases
    api.is_ui_thread = (_native.OK, 0)
    with pytest.raises(UiError) as raised:
        prop.get()
    assert raised.value.raw_status == _native.WRONG_THREAD
    prop.close()
    runtime.close()


def test_failed_property_close_keeps_token_for_retry() -> None:
    runtime, prop, api = create_property("retry")
    api.release_statuses = [_native.INVALID_HANDLE, _native.OK]
    with pytest.raises(UiError) as raised:
        prop.close()
    assert raised.value.raw_status == _native.INVALID_HANDLE
    assert prop._cell.token == 201
    prop.close()
    prop.close()
    assert api.releases.count(201) == 2
    runtime.close()


def test_close_waits_for_inflight_utf8_get() -> None:
    runtime, prop, api = create_property("leased")
    api.get_entered = threading.Event()
    api.get_continue = threading.Event()
    getter = threading.Thread(target=prop.get)
    getter.start()
    assert api.get_entered.wait(timeout=2)

    closed = threading.Event()

    def close_property() -> None:
        prop.close()
        closed.set()

    closer = threading.Thread(target=close_property)
    closer.start()
    assert not closed.wait(timeout=0.05)
    assert 201 not in api.releases

    api.get_continue.set()
    getter.join(timeout=2)
    closer.join(timeout=2)
    assert not getter.is_alive()
    assert not closer.is_alive()
    assert api.releases.count(201) == 1
    runtime.close()

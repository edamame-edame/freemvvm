"""Focused tests for the Python F64 Property facade."""

from __future__ import annotations

import math
import threading

import pytest

from freemvvm import ErrorSnapshot, Property, Runtime, UiError
from freemvvm import _native


class FakeF64Api:
    def __init__(self) -> None:
        self.calls: list[tuple[str, tuple[object, ...]]] = []
        self.releases: list[int] = []
        self.release_statuses: list[int] = []
        self.is_ui_thread = (_native.OK, 1)
        self.create_f64 = (_native.OK, 301, 0)
        self.create_value = 0.0
        self.value = 0.0
        self.get_f64 = (_native.OK, 0.0)
        self.set_f64 = (_native.OK, 0)
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

    def property_create_f64(
        self, runtime: int, initial: float
    ) -> tuple[int, int, int]:
        self.calls.append(("property_create_f64", (runtime, initial)))
        self.create_value = initial
        if self.create_f64[0] == _native.OK:
            self.value = initial
        return self.create_f64

    def property_create_utf8(
        self, runtime: int, initial: bytes
    ) -> tuple[int, int, int]:
        raise AssertionError("unexpected UTF-8 create")

    def property_get_bool(self, property: int) -> tuple[int, int]:
        raise AssertionError("unexpected BOOL get")

    def property_get_i64(self, property: int) -> tuple[int, int]:
        raise AssertionError("unexpected I64 get")

    def property_get_f64(self, property: int) -> tuple[int, float]:
        if self.get_entered is not None:
            self.get_entered.set()
        if self.get_continue is not None:
            self.get_continue.wait(timeout=2)
        self.calls.append(("property_get_f64", (property,)))
        status, value = self.get_f64
        return status, self.value if status == _native.OK else value

    def property_get_utf8(
        self, property: int, buffer: bytearray | None
    ) -> tuple[int, int]:
        raise AssertionError("unexpected UTF-8 get")

    def property_set_bool(self, property: int, value: int) -> tuple[int, int]:
        raise AssertionError("unexpected BOOL set")

    def property_set_i64(self, property: int, value: int) -> tuple[int, int]:
        raise AssertionError("unexpected I64 set")

    def property_set_f64(self, property: int, value: float) -> tuple[int, int]:
        self.calls.append(("property_set_f64", (property, value)))
        status, error = self.set_f64
        if status == _native.OK:
            self.value = value
        return status, error

    def property_set_utf8(
        self, property: int, value: bytes
    ) -> tuple[int, int]:
        raise AssertionError("unexpected UTF-8 set")

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
    initial: float = 1.25,
) -> tuple[Runtime, Property, FakeF64Api]:
    api = FakeF64Api()
    runtime = Runtime._create_with_api(api)
    return runtime, Property.create(runtime, initial), api


def _assert_same_f64(left: float, right: float) -> None:
    if math.isnan(right):
        assert math.isnan(left)
    elif right == 0.0:
        assert left == 0.0
        assert math.copysign(1.0, left) == math.copysign(1.0, right)
    else:
        assert left == right


@pytest.mark.parametrize(
    "value",
    [1.25, -3.5, float("nan"), float("inf"), float("-inf"), 0.0, -0.0],
)
def test_create_get_set_preserves_f64_values(value: float) -> None:
    runtime, prop, api = create_property(value)
    assert api.calls[-1] == ("property_create_f64", (41, value))
    _assert_same_f64(prop.get(), value)

    prop.set(value)
    assert api.calls[-1] == ("property_set_f64", (301, value))
    _assert_same_f64(prop.get(), value)

    prop.set(7.75)
    assert prop.get() == 7.75
    prop.close()
    runtime.close()


def test_create_rejects_non_builtin_float_before_native_call() -> None:
    class FloatSubclass(float):
        pass

    api = FakeF64Api()
    runtime = Runtime._create_with_api(api)
    for value in (FloatSubclass(1.0), object()):
        with pytest.raises(TypeError):
            Property.create(runtime, value)  # type: ignore[arg-type]
    assert not any(name == "property_create_f64" for name, _ in api.calls)
    runtime.close()


def test_set_rejects_wrong_type_and_float_subclass_before_native_call() -> (
    None
):
    runtime, prop, api = create_property()

    class FloatSubclass(float):
        pass

    before = len(api.calls)
    for value in (True, 1, "1.0", FloatSubclass(1.0)):
        with pytest.raises(TypeError):
            prop.set(value)  # type: ignore[arg-type]
    assert len(api.calls) == before
    prop.close()
    runtime.close()


def test_failed_create_snapshots_error_and_releases_partial_property() -> None:
    api = FakeF64Api()
    api.create_f64 = (_native.INVALID_ARGUMENT, 302, 901)
    runtime = Runtime._create_with_api(api)

    with pytest.raises(UiError) as raised:
        Property.create(runtime, 3.5)

    assert raised.value.snapshot == ErrorSnapshot(
        _native.INVALID_ARGUMENT, 0x1234, 0x5678, "property rejected"
    )
    assert 302 in api.releases
    assert 901 in api.releases
    runtime.close()


def test_failed_set_snapshots_error_without_changing_property_value() -> None:
    runtime, prop, api = create_property(2.5)
    api.set_f64 = (_native.INVALID_ARGUMENT, 903)

    with pytest.raises(UiError) as raised:
        prop.set(8.5)

    assert raised.value.snapshot == ErrorSnapshot(
        _native.INVALID_ARGUMENT, 0x1234, 0x5678, "property rejected"
    )
    assert 903 in api.releases
    assert prop.get() == 2.5
    prop.close()
    runtime.close()


def test_get_failure_preserves_raw_status_without_error_snapshot() -> None:
    runtime, prop, api = create_property()
    api.get_f64 = (_native.INVALID_HANDLE, 99.0)

    with pytest.raises(UiError) as raised:
        prop.get()

    assert raised.value.snapshot == ErrorSnapshot(_native.INVALID_HANDLE)
    prop.close()
    runtime.close()


def test_wrong_thread_is_rejected_before_f64_native_operation() -> None:
    api = FakeF64Api()
    api.is_ui_thread = (_native.OK, 0)
    runtime = Runtime._create_with_api(api)
    with pytest.raises(UiError) as raised:
        Property.create(runtime, 1.0)
    assert raised.value.raw_status == _native.WRONG_THREAD
    assert not any(name == "property_create_f64" for name, _ in api.calls)
    runtime.close()


def test_closed_f64_property_rejects_get_and_set() -> None:
    runtime, prop, api = create_property()
    prop.close()
    before = sum(
        name.startswith("property_") for name, _ in api.calls
    )

    with pytest.raises(UiError) as raised:
        prop.get()
    assert raised.value.raw_status == _native.CLOSED
    with pytest.raises(UiError) as raised:
        prop.set(2.0)
    assert raised.value.raw_status == _native.CLOSED
    assert sum(name.startswith("property_") for name, _ in api.calls) == before
    runtime.close()


def test_f64_close_retains_token_after_failed_release_for_retry() -> None:
    runtime, prop, api = create_property()
    api.release_statuses = [_native.INVALID_HANDLE, _native.OK]

    with pytest.raises(UiError) as raised:
        prop.close()

    assert raised.value.raw_status == _native.INVALID_HANDLE
    assert prop._cell.token == 301
    prop.close()
    prop.close()
    assert api.releases.count(301) == 2
    runtime.close()


def test_close_waits_for_inflight_f64_get() -> None:
    runtime, prop, api = create_property(6.25)
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
    assert 301 not in api.releases

    api.get_continue.set()
    getter.join(timeout=2)
    closer.join(timeout=2)
    assert not getter.is_alive()
    assert not closer.is_alive()
    assert api.releases.count(301) == 1
    runtime.close()

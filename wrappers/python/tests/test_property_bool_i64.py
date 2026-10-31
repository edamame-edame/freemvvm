"""Focused tests for Python BOOL and signed-I64 Property values."""

from __future__ import annotations

import threading

import pytest

from freemvvm import ErrorSnapshot, Property, Runtime, UiError
from freemvvm import _native


class FakePropertyApi:
    def __init__(self) -> None:
        self.calls: list[tuple[str, tuple[int, ...]]] = []
        self.releases: list[int] = []
        self.release_statuses: list[int] = []
        self.is_ui_thread = (_native.OK, 1)
        self.create_bool = (_native.OK, 101, 0)
        self.create_i64 = (_native.OK, 102, 0)
        self.get_bool = (_native.OK, 0)
        self.get_i64 = (_native.OK, -(1 << 63))
        self.set_bool = (_native.OK, 0)
        self.set_i64 = (_native.OK, 0)
        self.error_status_value = (_native.OK, _native.INVALID_ARGUMENT)
        self.error_code_value = (_native.OK, 0x1234, 0x5678)
        self.error_message = "property rejected"
        self.get_entered: threading.Event | None = None
        self.get_continue: threading.Event | None = None

    def get_version(self) -> tuple[int, int, int, int]:
        self.calls.append(("version", ()))
        return _native.OK, 1, 0, 0

    def runtime_create(self) -> tuple[int, int, int]:
        self.calls.append(("runtime_create", ()))
        return _native.OK, 41, 0

    def runtime_is_ui_thread(self, runtime: int) -> tuple[int, int]:
        self.calls.append(("runtime_is_ui_thread", (runtime,)))
        return self.is_ui_thread

    def runtime_shutdown(self, runtime: int) -> tuple[int, int]:
        self.calls.append(("runtime_shutdown", (runtime,)))
        return _native.OK, 0

    def runtime_request_shutdown(
        self, runtime: int, mode: int
    ) -> tuple[int, int, int]:
        self.calls.append(("runtime_request_shutdown", (runtime, mode)))
        return _native.UNSUPPORTED, 0, 0

    def shutdown_request_query(
        self, request: int
    ) -> tuple[int, int, int, int]:
        self.calls.append(("shutdown_request_query", (request,)))
        return _native.UNSUPPORTED, 0, 0, 0

    def property_create_bool(
        self, runtime: int, initial: int
    ) -> tuple[int, int, int]:
        self.calls.append(("property_create_bool", (runtime, initial)))
        return self.create_bool

    def property_create_i64(
        self, runtime: int, initial: int
    ) -> tuple[int, int, int]:
        self.calls.append(("property_create_i64", (runtime, initial)))
        return self.create_i64

    def property_get_bool(self, property: int) -> tuple[int, int]:
        self.calls.append(("property_get_bool", (property,)))
        if self.get_entered is not None:
            self.get_entered.set()
        if self.get_continue is not None:
            self.get_continue.wait(timeout=2)
        return self.get_bool

    def property_get_i64(self, property: int) -> tuple[int, int]:
        self.calls.append(("property_get_i64", (property,)))
        if self.get_entered is not None:
            self.get_entered.set()
        if self.get_continue is not None:
            self.get_continue.wait(timeout=2)
        return self.get_i64

    def property_set_bool(self, property: int, value: int) -> tuple[int, int]:
        self.calls.append(("property_set_bool", (property, value)))
        return self.set_bool

    def property_set_i64(self, property: int, value: int) -> tuple[int, int]:
        self.calls.append(("property_set_i64", (property, value)))
        return self.set_i64

    def handle_release(self, owned: int) -> int:
        self.calls.append(("handle_release", (owned,)))
        self.releases.append(owned)
        return (
            self.release_statuses.pop(0)
            if self.release_statuses
            else _native.OK
        )

    def error_status(self, error: int) -> tuple[int, int]:
        self.calls.append(("error_status", (error,)))
        return self.error_status_value

    def error_code(self, error: int) -> tuple[int, int, int]:
        self.calls.append(("error_code", (error,)))
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
    initial: bool | int,
) -> tuple[Runtime, Property, FakePropertyApi]:
    api = FakePropertyApi()
    runtime = Runtime._create_with_api(api)
    return runtime, Property.create(runtime, initial), api


def close_all(runtime: Runtime, prop: Property) -> None:
    prop.close()
    runtime.close()


def test_factory_maps_builtin_bool_and_i64_exactly() -> None:
    runtime, bool_property, api = create_property(False)
    assert api.calls[-1] == ("property_create_bool", (41, 0))
    assert bool_property.get() is False
    bool_property.set(True)
    assert api.calls[-1] == ("property_set_bool", (101, 1))
    api.get_bool = (_native.OK, 1)
    assert bool_property.get() is True

    api.create_i64 = (_native.OK, 102, 0)
    i64_property = Property.create(runtime, -(1 << 63))
    assert api.calls[-1] == ("property_create_i64", (41, -(1 << 63)))
    api.get_i64 = (_native.OK, (1 << 63) - 1)
    assert i64_property.get() == (1 << 63) - 1
    i64_property.set((1 << 63) - 1)
    assert api.calls[-1] == ("property_set_i64", (102, (1 << 63) - 1))

    bool_property.close()
    i64_property.close()
    runtime.close()


def test_invalid_types_and_i64_overflow_are_rejected_before_native_calls() -> (
    None
):
    api = FakePropertyApi()
    runtime = Runtime._create_with_api(api)
    initial_call_count = len(api.calls)

    class IntSubclass(int):
        pass

    for value in (b"1", IntSubclass(1)):
        with pytest.raises(TypeError):
            Property.create(runtime, value)  # type: ignore[arg-type]
    for value in (-(1 << 63) - 1, 1 << 63):
        with pytest.raises(OverflowError):
            Property.create(runtime, value)

    assert len(api.calls) == initial_call_count
    assert not any(name.startswith("property_create") for name, _ in api.calls)
    runtime.close()


def test_set_rejects_wrong_kind_and_subclasses_without_native_call() -> None:
    runtime, bool_property, api = create_property(False)

    class IntSubclass(int):
        pass

    initial_call_count = len(api.calls)
    with pytest.raises(TypeError):
        bool_property.set(1)  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        bool_property.set(IntSubclass(1))  # type: ignore[arg-type]
    assert len(api.calls) == initial_call_count

    api.create_i64 = (_native.OK, 102, 0)
    i64_property = Property.create(runtime, 5)
    initial_call_count = len(api.calls)
    with pytest.raises(TypeError):
        i64_property.set(True)  # type: ignore[arg-type]
    with pytest.raises(OverflowError):
        i64_property.set(1 << 63)
    assert len(api.calls) == initial_call_count

    close_all(runtime, i64_property)
    bool_property.close()


def test_thread_runtime_and_native_bool_errors() -> None:
    api = FakePropertyApi()
    runtime = Runtime._create_with_api(api)
    api.is_ui_thread = (_native.OK, 0)
    with pytest.raises(UiError) as raised:
        Property.create(runtime, True)
    assert raised.value.raw_status == _native.WRONG_THREAD
    assert not any(name == "property_create_bool" for name, _ in api.calls)

    api.is_ui_thread = (_native.OK, 1)
    bool_property = Property.create(runtime, True)
    api.get_bool = (_native.OK, 2)
    with pytest.raises(UiError) as raised:
        bool_property.get()
    assert raised.value.raw_status == _native.INTERNAL_ERROR

    api.is_ui_thread = (_native.OK, 0)
    with pytest.raises(UiError) as raised:
        bool_property.set(False)
    assert raised.value.raw_status == _native.WRONG_THREAD
    assert not any(name == "property_set_bool" for name, _ in api.calls)

    api.is_ui_thread = (_native.OK, 1)
    runtime.close()
    with pytest.raises(UiError) as raised:
        bool_property.get()
    assert raised.value.raw_status == _native.CLOSED
    bool_property.close()


def test_native_errors_keep_snapshot_details_and_raw_get_status() -> None:
    api = FakePropertyApi()
    api.create_bool = (_native.INVALID_ARGUMENT, 0, 900)
    runtime = Runtime._create_with_api(api)
    with pytest.raises(UiError) as raised:
        Property.create(runtime, False)
    assert raised.value.snapshot == ErrorSnapshot(
        _native.INVALID_ARGUMENT, 0x1234, 0x5678, "property rejected"
    )
    assert 900 in api.releases

    api.create_bool = (_native.OK, 101, 0)
    bool_property = Property.create(runtime, False)
    api.get_bool = (_native.OUT_OF_MEMORY, 0)
    with pytest.raises(UiError) as raised:
        bool_property.get()
    assert raised.value.snapshot == ErrorSnapshot(_native.OUT_OF_MEMORY)

    api.set_bool = (_native.INVALID_ARGUMENT, 901)
    with pytest.raises(UiError) as raised:
        bool_property.set(True)
    assert raised.value.snapshot == ErrorSnapshot(
        _native.INVALID_ARGUMENT, 0x1234, 0x5678, "property rejected"
    )
    assert 901 in api.releases

    # There is no wrapper shadow value: a later get returns the native value.
    api.get_bool = (_native.OK, 0)
    assert bool_property.get() is False
    close_all(runtime, bool_property)


def test_failed_close_keeps_property_token_for_retry() -> None:
    runtime, bool_property, api = create_property(True)
    api.release_statuses = [_native.INVALID_HANDLE, _native.OK]

    with pytest.raises(UiError) as raised:
        bool_property.close()
    assert raised.value.raw_status == _native.INVALID_HANDLE
    assert bool_property._cell.token == 101

    bool_property.close()
    bool_property.close()
    assert api.releases.count(101) == 2
    runtime.close()


def test_property_close_waits_for_an_inflight_native_call() -> None:
    runtime, i64_property, api = create_property(7)
    api.get_entered = threading.Event()
    api.get_continue = threading.Event()
    getter_thread = threading.Thread(target=i64_property.get)
    getter_thread.start()
    assert api.get_entered.wait(timeout=2)

    closed = threading.Event()

    def close_property() -> None:
        i64_property.close()
        closed.set()

    close_thread = threading.Thread(target=close_property)
    close_thread.start()
    assert not closed.wait(timeout=0.05)
    assert 102 not in api.releases

    api.get_continue.set()
    getter_thread.join(timeout=2)
    close_thread.join(timeout=2)
    assert not getter_thread.is_alive()
    assert not close_thread.is_alive()
    assert api.releases.count(102) == 1
    runtime.close()

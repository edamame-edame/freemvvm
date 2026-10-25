"""Focused tests for Python DECIMAL Property mapping and ownership."""

from __future__ import annotations

from decimal import Decimal, Inexact, Rounded, localcontext
import threading

import pytest

from freemvvm import (
    ErrorSnapshot,
    Property,
    Runtime,
    UiError,
    _decimal_from_bits,
    _decimal_to_bits,
)
from freemvvm import _native


_COEFFICIENT_MAX = (1 << 58) - 1


def encode(sign: int, scale: int, coefficient: int) -> int:
    return (sign << 63) | (scale << 58) | coefficient


class FakeDecimalApi:
    def __init__(self) -> None:
        self.calls: list[tuple[str, tuple[object, ...]]] = []
        self.releases: list[int] = []
        self.release_statuses: list[int] = []
        self.is_ui_thread = (_native.OK, 1)
        self.create_decimal = (_native.OK, 301, 0)
        self.value = 0
        self.get_decimal_result: tuple[int, int] | None = None
        self.set_decimal_result: tuple[int, int] = (_native.OK, 0)
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
        self.calls.append(("property_create_bool", (runtime, initial)))
        return _native.OK, 310, 0

    def property_create_i64(
        self, runtime: int, initial: int
    ) -> tuple[int, int, int]:
        self.calls.append(("property_create_i64", (runtime, initial)))
        return _native.OK, 311, 0

    def property_create_f64(
        self, runtime: int, initial: float
    ) -> tuple[int, int, int]:
        self.calls.append(("property_create_f64", (runtime, initial)))
        return _native.OK, 312, 0

    def property_create_utf8(
        self, runtime: int, initial: bytes
    ) -> tuple[int, int, int]:
        self.calls.append(("property_create_utf8", (runtime, initial)))
        return _native.OK, 313, 0

    def property_create_decimal(
        self, runtime: int, initial_bits: int
    ) -> tuple[int, int, int]:
        self.calls.append(("property_create_decimal", (runtime, initial_bits)))
        status, token, error = self.create_decimal
        if status == _native.OK:
            self.value = initial_bits
        return status, token, error

    def property_get_bool(self, property: int) -> tuple[int, int]:
        raise AssertionError("unexpected BOOL get")

    def property_get_i64(self, property: int) -> tuple[int, int]:
        raise AssertionError("unexpected I64 get")

    def property_get_f64(self, property: int) -> tuple[int, float]:
        raise AssertionError("unexpected F64 get")

    def property_get_utf8(
        self, property: int, buffer: bytearray | None
    ) -> tuple[int, int]:
        raise AssertionError("unexpected UTF-8 get")

    def property_get_decimal(self, property: int) -> tuple[int, int]:
        self.calls.append(("property_get_decimal", (property,)))
        if self.get_entered is not None:
            self.get_entered.set()
            assert self.get_continue is not None
            assert self.get_continue.wait(timeout=2)
        if self.get_decimal_result is not None:
            return self.get_decimal_result
        return _native.OK, self.value

    def property_set_bool(self, property: int, value: int) -> tuple[int, int]:
        raise AssertionError("unexpected BOOL set")

    def property_set_i64(self, property: int, value: int) -> tuple[int, int]:
        raise AssertionError("unexpected I64 set")

    def property_set_f64(self, property: int, value: float) -> tuple[int, int]:
        raise AssertionError("unexpected F64 set")

    def property_set_utf8(
        self, property: int, value: bytes
    ) -> tuple[int, int]:
        raise AssertionError("unexpected UTF-8 set")

    def property_set_decimal(
        self, property: int, bits: int
    ) -> tuple[int, int]:
        self.calls.append(("property_set_decimal", (property, bits)))
        status, error = self.set_decimal_result
        if status == _native.OK:
            self.value = bits
        return status, error

    def handle_release(self, owned: int) -> int:
        self.releases.append(owned)
        if self.release_statuses:
            return self.release_statuses.pop(0)
        return _native.OK

    def error_status(self, error: int) -> tuple[int, int]:
        return self.error_status_value

    def error_code(self, error: int) -> tuple[int, int, int]:
        return self.error_code_value

    def error_message_utf8(
        self, error: int, buffer: bytearray | None
    ) -> tuple[int, int]:
        raw = self.error_message.encode("utf-8")
        if buffer is None:
            return _native.BUFFER_TOO_SMALL, len(raw)
        buffer[: len(raw)] = raw
        return _native.OK, len(raw)


def create_property(
    initial: Decimal = Decimal("1.00"),
) -> tuple[Runtime, Property, FakeDecimalApi]:
    api = FakeDecimalApi()
    runtime = Runtime._create_with_api(api)
    prop = Property.create(runtime, initial)
    return runtime, prop, api


@pytest.mark.parametrize(
    ("value", "expected"),
    [
        (Decimal("0"), encode(0, 0, 0)),
        (Decimal("-0.00"), encode(1, 2, 0)),
        (Decimal("1.00"), encode(0, 2, 100)),
        (Decimal("-1.25"), encode(1, 2, 125)),
        (Decimal("1E+2"), encode(0, 0, 100)),
        (Decimal("1E-31"), encode(0, 31, 1)),
        (Decimal("10E-32"), encode(0, 31, 1)),
        (Decimal("288230376151711743"), encode(0, 0, _COEFFICIENT_MAX)),
        (
            Decimal("288230376151711743.0"),
            encode(0, 0, _COEFFICIENT_MAX),
        ),
        (Decimal("0E-32"), encode(0, 0, 0)),
    ],
)
def test_decimal_to_bits_is_exact(value: Decimal, expected: int) -> None:
    assert _decimal_to_bits(value) == expected


@pytest.mark.parametrize(
    ("bits", "expected"),
    [
        (encode(0, 0, 0), Decimal("0")),
        (encode(1, 0, 0), Decimal("-0")),
        (encode(1, 2, 0), Decimal("-0.00")),
        (encode(0, 2, 100), Decimal("1.00")),
        (encode(1, 31, 1), Decimal("-1E-31")),
        (encode(0, 0, _COEFFICIENT_MAX), Decimal("288230376151711743")),
    ],
)
def test_decimal_from_bits_preserves_sign_scale_and_value(
    bits: int, expected: Decimal
) -> None:
    actual = _decimal_from_bits(bits)
    assert actual == expected
    assert actual.as_tuple() == expected.as_tuple()


@pytest.mark.parametrize(
    "value",
    [
        Decimal("1E-32"),
        Decimal("288230376151711744"),
        Decimal("288230376151711744.0"),
        Decimal("1E+1000000"),
    ],
)
def test_decimal_to_bits_rejects_unrepresentable_values(
    value: Decimal,
) -> None:
    with pytest.raises(OverflowError):
        _decimal_to_bits(value)


@pytest.mark.parametrize(
    "value",
    [
        Decimal("NaN"),
        Decimal("Infinity"),
        Decimal("-Infinity"),
        Decimal("sNaN"),
    ],
)
def test_decimal_to_bits_rejects_non_finite_values(value: Decimal) -> None:
    with pytest.raises(ValueError):
        _decimal_to_bits(value)


def test_decimal_to_bits_does_not_use_decimal_context() -> None:
    value = Decimal("123456789012.34500")
    with localcontext() as context:
        context.prec = 1
        context.traps[Inexact] = True
        context.traps[Rounded] = True
        assert _decimal_to_bits(value) == encode(0, 5, 12345678901234500)


def test_create_get_and_set_use_decimal_bits_without_value_cache() -> None:
    runtime, prop, api = create_property(Decimal("1.00"))
    assert api.calls[-1] == (
        "property_create_decimal",
        (41, encode(0, 2, 100)),
    )
    assert prop.get().as_tuple() == Decimal("1.00").as_tuple()

    prop.set(Decimal("-2.50"))
    assert api.calls[-1] == (
        "property_set_decimal",
        (301, encode(1, 2, 250)),
    )
    assert prop.get().as_tuple() == Decimal("-2.50").as_tuple()
    prop.close()
    runtime.close()


def test_create_dispatches_existing_kinds_and_rejects_decimal_subclass() -> (
    None
):
    class DecimalSubclass(Decimal):
        pass

    api = FakeDecimalApi()
    runtime = Runtime._create_with_api(api)
    properties = [
        Property.create(runtime, True),
        Property.create(runtime, 1),
        Property.create(runtime, 1.0),
        Property.create(runtime, "1.0"),
    ]
    assert [
        name for name, _ in api.calls if name.startswith("property_create_")
    ] == [
        "property_create_bool",
        "property_create_i64",
        "property_create_f64",
        "property_create_utf8",
    ]

    before = len(api.calls)
    for value in (object(), DecimalSubclass("1.0")):
        with pytest.raises(TypeError):
            Property.create(runtime, value)  # type: ignore[arg-type]
    assert not any(
        name.startswith("property_create_") for name, _ in api.calls[before:]
    )
    for prop in properties:
        prop.close()
    runtime.close()


def test_set_rejects_non_decimal_before_native_call() -> None:
    class DecimalSubclass(Decimal):
        pass

    runtime, prop, api = create_property()
    before = len(api.calls)
    for value in (True, 1, 1.0, "1.0", DecimalSubclass("2.0")):
        with pytest.raises(TypeError):
            prop.set(value)  # type: ignore[arg-type]
    assert len(api.calls) == before
    prop.close()
    runtime.close()


def test_non_finite_and_out_of_range_create_and_set_skip_native_calls() -> (
    None
):
    api = FakeDecimalApi()
    runtime = Runtime._create_with_api(api)
    before = len(api.calls)
    for value in (Decimal("NaN"), Decimal("Infinity")):
        with pytest.raises(ValueError):
            Property.create(runtime, value)
    with pytest.raises(OverflowError):
        Property.create(runtime, Decimal("1E-32"))
    assert not any(
        name.startswith("property_create_") for name, _ in api.calls[before:]
    )

    prop = Property.create(runtime, Decimal("1.00"))
    before = len(api.calls)
    for value in (Decimal("-Infinity"), Decimal("1E+1000000")):
        error = ValueError if not value.is_finite() else OverflowError
        with pytest.raises(error):
            prop.set(value)
    assert len(api.calls) == before
    prop.close()
    runtime.close()


def test_failed_create_snapshots_error_and_releases_partial_property() -> None:
    api = FakeDecimalApi()
    api.create_decimal = (_native.INVALID_ARGUMENT, 302, 901)
    runtime = Runtime._create_with_api(api)

    with pytest.raises(UiError) as raised:
        Property.create(runtime, Decimal("1.00"))

    assert raised.value.snapshot == ErrorSnapshot(
        _native.INVALID_ARGUMENT, 0x1234, 0x5678, "property rejected"
    )
    assert 302 in api.releases
    assert 901 in api.releases
    runtime.close()


def test_failed_set_snapshots_error_without_changing_property_value() -> None:
    runtime, prop, api = create_property(Decimal("1.00"))
    api.set_decimal_result = (_native.INVALID_ARGUMENT, 903)

    with pytest.raises(UiError) as raised:
        prop.set(Decimal("8.50"))

    assert raised.value.snapshot == ErrorSnapshot(
        _native.INVALID_ARGUMENT, 0x1234, 0x5678, "property rejected"
    )
    assert 903 in api.releases
    assert prop.get().as_tuple() == Decimal("1.00").as_tuple()
    prop.close()
    runtime.close()


def test_get_failure_preserves_raw_status_without_error_snapshot() -> None:
    runtime, prop, api = create_property()
    api.get_decimal_result = (_native.INVALID_HANDLE, 0)

    with pytest.raises(UiError) as raised:
        prop.get()

    assert raised.value.snapshot == ErrorSnapshot(_native.INVALID_HANDLE)
    prop.close()
    runtime.close()


def test_get_rejects_malformed_native_uint64_value() -> None:
    runtime, prop, api = create_property()
    api.get_decimal_result = (_native.OK, -1)

    with pytest.raises(UiError) as raised:
        prop.get()

    assert raised.value.raw_status == _native.INTERNAL_ERROR
    prop.close()
    runtime.close()


def test_wrong_thread_is_rejected_before_decimal_native_operation() -> None:
    api = FakeDecimalApi()
    api.is_ui_thread = (_native.OK, 0)
    runtime = Runtime._create_with_api(api)
    with pytest.raises(UiError) as raised:
        Property.create(runtime, Decimal("1.0"))
    assert raised.value.raw_status == _native.WRONG_THREAD
    assert not any(name == "property_create_decimal" for name, _ in api.calls)
    runtime.close()


def test_closed_decimal_property_rejects_get_and_set() -> None:
    runtime, prop, api = create_property()
    prop.close()
    before = sum(name.startswith("property_") for name, _ in api.calls)

    with pytest.raises(UiError) as raised:
        prop.get()
    assert raised.value.raw_status == _native.CLOSED
    with pytest.raises(UiError) as raised:
        prop.set(Decimal("2.0"))
    assert raised.value.raw_status == _native.CLOSED
    assert sum(name.startswith("property_") for name, _ in api.calls) == before
    runtime.close()


def test_close_retains_token_after_failed_release_for_retry() -> None:
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


def test_close_waits_for_inflight_decimal_get() -> None:
    runtime, prop, api = create_property(Decimal("6.25"))
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

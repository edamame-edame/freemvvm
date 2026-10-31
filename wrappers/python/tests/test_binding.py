"""Focused fake-ABI tests for Property Binding ownership and snapshots."""

from __future__ import annotations

import threading
from decimal import Decimal

import pytest

from freemvvm import (
    Binding,
    BindingDirection,
    BindingMode,
    ErrorSnapshot,
    Property,
    Runtime,
    UiError,
)
from freemvvm import _native


class FakeBindingApi:
    def __init__(self) -> None:
        self.calls: list[tuple[str, tuple[object, ...]]] = []
        self.releases: list[int] = []
        self.release_statuses: list[int] = []
        self.ui_thread = 1
        self.next_token = 100
        self.properties: dict[int, tuple[str, object]] = {}
        self.binding_create_result: tuple[int, int, int] | None = None
        self.binding_cancel_result = (_native.OK, 0)
        self.binding_snapshot_status = _native.OK
        self.binding_snapshot_result = (0, 0, 0)
        self.error_messages: dict[int, str] = {}
        self.snapshot_entered: threading.Event | None = None
        self.snapshot_continue: threading.Event | None = None

    def _token(self) -> int:
        self.next_token += 1
        return self.next_token

    def get_version(self) -> tuple[int, int, int, int]:
        return _native.OK, 1, 0, 0

    def runtime_create(self) -> tuple[int, int, int]:
        return _native.OK, 41, 0

    def runtime_is_ui_thread(self, runtime: int) -> tuple[int, int]:
        self.calls.append(("runtime_is_ui_thread", (runtime,)))
        return _native.OK, self.ui_thread

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

    def _create_property(
        self, kind: str, initial: object
    ) -> tuple[int, int, int]:
        token = self._token()
        self.properties[token] = (kind, initial)
        return _native.OK, token, 0

    def property_create_bool(
        self, runtime: int, initial: int
    ) -> tuple[int, int, int]:
        self.calls.append(("property_create_bool", (runtime, initial)))
        return self._create_property("bool", initial)

    def property_create_i64(
        self, runtime: int, initial: int
    ) -> tuple[int, int, int]:
        self.calls.append(("property_create_i64", (runtime, initial)))
        return self._create_property("i64", initial)

    def property_create_f64(
        self, runtime: int, initial: float
    ) -> tuple[int, int, int]:
        self.calls.append(("property_create_f64", (runtime, initial)))
        return self._create_property("f64", initial)

    def property_create_decimal(
        self, runtime: int, initial_bits: int
    ) -> tuple[int, int, int]:
        self.calls.append(("property_create_decimal", (runtime, initial_bits)))
        return self._create_property("decimal", initial_bits)

    def property_create_utf8(
        self, runtime: int, initial: bytes
    ) -> tuple[int, int, int]:
        self.calls.append(("property_create_utf8", (runtime, initial)))
        return self._create_property("utf8", initial)

    def _get(self, property: int, kind: str) -> tuple[int, object]:
        stored_kind, value = self.properties[property]
        assert stored_kind == kind
        return _native.OK, value

    def property_get_bool(self, property: int) -> tuple[int, int]:
        status, value = self._get(property, "bool")
        return status, int(value)

    def property_get_i64(self, property: int) -> tuple[int, int]:
        status, value = self._get(property, "i64")
        return status, int(value)

    def property_get_f64(self, property: int) -> tuple[int, float]:
        status, value = self._get(property, "f64")
        return status, float(value)

    def property_get_decimal(self, property: int) -> tuple[int, int]:
        status, value = self._get(property, "decimal")
        return status, int(value)

    def property_get_utf8(
        self, property: int, buffer: bytearray | None
    ) -> tuple[int, int]:
        status, value = self._get(property, "utf8")
        encoded = bytes(value)
        if buffer is None or len(buffer) < len(encoded):
            return _native.BUFFER_TOO_SMALL, len(encoded)
        buffer[: len(encoded)] = encoded
        return status, len(encoded)

    def _set(self, property: int, kind: str, value: object) -> tuple[int, int]:
        stored_kind, _ = self.properties[property]
        assert stored_kind == kind
        self.properties[property] = (kind, value)
        return _native.OK, 0

    def property_set_bool(self, property: int, value: int) -> tuple[int, int]:
        return self._set(property, "bool", value)

    def property_set_i64(self, property: int, value: int) -> tuple[int, int]:
        return self._set(property, "i64", value)

    def property_set_f64(self, property: int, value: float) -> tuple[int, int]:
        return self._set(property, "f64", value)

    def property_set_decimal(
        self, property: int, bits: int
    ) -> tuple[int, int]:
        return self._set(property, "decimal", bits)

    def property_set_utf8(
        self, property: int, value: bytes
    ) -> tuple[int, int]:
        return self._set(property, "utf8", value)

    def binding_create(
        self, source: int, target: int, mode: int
    ) -> tuple[int, int, int]:
        self.calls.append(("binding_create", (source, target, mode)))
        if self.binding_create_result is not None:
            return self.binding_create_result
        if mode == int(BindingMode.ONE_TIME):
            return _native.OK, 0, 0
        return _native.OK, self._token(), 0

    def binding_cancel(self, binding: int) -> tuple[int, int]:
        self.calls.append(("binding_cancel", (binding,)))
        return self.binding_cancel_result

    def binding_error_snapshot(
        self, binding: int
    ) -> tuple[int, int, int, int]:
        self.calls.append(("binding_error_snapshot", (binding,)))
        if self.snapshot_entered is not None:
            self.snapshot_entered.set()
            assert self.snapshot_continue is not None
            assert self.snapshot_continue.wait(timeout=2)
        return self.binding_snapshot_status, *self.binding_snapshot_result

    def handle_release(self, owned: int) -> int:
        self.releases.append(owned)
        if self.release_statuses:
            return self.release_statuses.pop(0)
        return _native.OK

    def error_status(self, error: int) -> tuple[int, int]:
        return _native.OK, _native.INVALID_ARGUMENT

    def error_code(self, error: int) -> tuple[int, int, int]:
        return _native.OK, 9, error

    def error_message_utf8(
        self, error: int, buffer: bytearray | None
    ) -> tuple[int, int]:
        encoded = self.error_messages.get(error, "binding rejected").encode()
        if buffer is None or len(buffer) < len(encoded):
            return _native.BUFFER_TOO_SMALL, len(encoded)
        buffer[: len(encoded)] = encoded
        return _native.OK, len(encoded)


def make_runtime(
    api: FakeBindingApi | None = None,
) -> tuple[Runtime, FakeBindingApi]:
    selected = api or FakeBindingApi()
    return Runtime._create_with_api(selected), selected


@pytest.mark.parametrize(
    ("initial", "kind"),
    [
        (True, "property_create_bool"),
        (3, "property_create_i64"),
        (3.5, "property_create_f64"),
        ("three", "property_create_utf8"),
        (Decimal("3.5"), "property_create_decimal"),
    ],
)
@pytest.mark.parametrize(
    ("mode", "returns_binding"),
    [
        (BindingMode.ONE_TIME, False),
        (BindingMode.ONE_WAY, True),
        (BindingMode.TWO_WAY, True),
    ],
)
def test_create_mode_mapping_for_each_property_kind(
    initial: object, kind: str, mode: BindingMode, returns_binding: bool
) -> None:
    runtime, api = make_runtime()
    source = Property.create(runtime, initial)  # type: ignore[arg-type]
    target = Property.create(runtime, initial)  # type: ignore[arg-type]

    binding = Binding.create(source, target, mode)

    create_calls = [
        name for name, _ in api.calls if name.startswith("property_create_")
    ]
    assert create_calls == [kind, kind]
    assert [call for call in api.calls if call[0] == "binding_create"][-1][1][
        2
    ] == int(mode)
    assert (binding is not None) is returns_binding
    if binding is not None:
        binding.close()
    source.close()
    target.close()
    runtime.close()


def test_create_rejects_invalid_arguments_and_incompatible_properties() -> (
    None
):
    runtime, api = make_runtime()
    source = Property.create(runtime, 1)
    target = Property.create(runtime, 2)
    text = Property.create(runtime, "2")

    with pytest.raises(TypeError):
        Binding.create(source, object(), BindingMode.ONE_WAY)  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        Binding.create(source, target, 2)  # type: ignore[arg-type]
    with pytest.raises(UiError) as raised:
        Binding.create(source, text, BindingMode.ONE_WAY)
    assert raised.value.raw_status == _native.WRONG_TYPE

    other_runtime, _ = make_runtime()
    other = Property.create(other_runtime, 3)
    with pytest.raises(UiError) as raised:
        Binding.create(source, other, BindingMode.ONE_WAY)
    assert raised.value.raw_status == _native.WRONG_RUNTIME
    assert not any(name == "binding_create" for name, _ in api.calls)

    api.ui_thread = 0
    with pytest.raises(UiError) as raised:
        Binding.create(source, target, BindingMode.ONE_WAY)
    assert raised.value.raw_status == _native.WRONG_THREAD
    assert not any(name == "binding_create" for name, _ in api.calls)
    api.ui_thread = 1

    source.close()
    target.close()
    text.close()
    other.close()
    runtime.close()
    other_runtime.close()


def test_create_failure_preserves_error_and_disposes_unexpected_binding() -> (
    None
):
    runtime, api = make_runtime()
    source = Property.create(runtime, 1)
    target = Property.create(runtime, 2)
    api.binding_create_result = (_native.INVALID_ARGUMENT, 700, 701)

    with pytest.raises(UiError) as raised:
        Binding.create(source, target, BindingMode.ONE_WAY)

    assert raised.value.snapshot == ErrorSnapshot(
        _native.INVALID_ARGUMENT, 9, 701, "binding rejected"
    )
    assert ("binding_cancel", (700,)) in api.calls
    assert 700 in api.releases and 701 in api.releases
    source.close()
    target.close()
    runtime.close()


@pytest.mark.parametrize(
    ("mode", "token"),
    [
        (BindingMode.ONE_TIME, 701),
        (BindingMode.ONE_WAY, 0),
    ],
)
def test_create_malformed_success_is_internal_error_and_cleans_token(
    mode: BindingMode, token: int
) -> None:
    runtime, api = make_runtime()
    source = Property.create(runtime, 1)
    target = Property.create(runtime, 2)
    api.binding_create_result = (_native.OK, token, 0)

    with pytest.raises(UiError) as raised:
        Binding.create(source, target, mode)

    assert raised.value.raw_status == _native.INTERNAL_ERROR
    if token:
        assert ("binding_cancel", (token,)) in api.calls
        assert token in api.releases
    source.close()
    target.close()
    runtime.close()


def test_cancel_and_close_have_distinct_effects_and_retry_failed_cancel() -> (
    None
):
    runtime, api = make_runtime()
    source = Property.create(runtime, 1)
    target = Property.create(runtime, 2)
    binding = Binding.create(source, target, BindingMode.ONE_WAY)
    assert binding is not None
    token = binding._cell.token

    api.binding_cancel_result = (_native.INVALID_ARGUMENT, 701)
    with pytest.raises(UiError):
        binding.cancel()
    assert binding._cell.token == token
    api.binding_cancel_result = (_native.OK, 0)
    binding.cancel()
    cancel_count = sum(name == "binding_cancel" for name, _ in api.calls)
    binding.close()
    assert (
        sum(name == "binding_cancel" for name, _ in api.calls) == cancel_count
    )
    assert api.releases.count(token) == 1

    source.close()
    target.close()
    runtime.close()


def test_close_retries_release_without_canceling_binding() -> None:
    runtime, api = make_runtime()
    source = Property.create(runtime, 1)
    target = Property.create(runtime, 2)
    binding = Binding.create(source, target, BindingMode.ONE_WAY)
    assert binding is not None
    token = binding._cell.token
    api.release_statuses = [_native.INVALID_HANDLE, _native.OK]

    error = binding.try_close()

    assert error is not None
    assert error.raw_status == _native.INVALID_HANDLE
    assert binding._cell.token == token
    assert not any(name == "binding_cancel" for name, _ in api.calls)
    binding.close()
    assert api.releases.count(token) == 2
    assert not any(name == "binding_cancel" for name, _ in api.calls)
    source.close()
    target.close()
    runtime.close()


@pytest.mark.parametrize(
    "direction",
    [BindingDirection.SOURCE_TO_TARGET, BindingDirection.TARGET_TO_SOURCE],
)
def test_error_snapshot_adopts_rejected_property_and_freezes_error(
    direction: BindingDirection,
) -> None:
    runtime, api = make_runtime()
    source = Property.create(runtime, 10)
    target = Property.create(runtime, 20)
    binding = Binding.create(source, target, BindingMode.TWO_WAY)
    assert binding is not None
    error_snapshot_token = api._token()
    rejected_property_token = api._token()
    api.error_messages[error_snapshot_token] = "target rejected value"
    api.properties[rejected_property_token] = ("i64", 20)
    api.binding_snapshot_result = (
        error_snapshot_token,
        rejected_property_token,
        int(direction),
    )

    snapshot = binding.error_snapshot()

    assert snapshot is not None
    assert snapshot.error == ErrorSnapshot(
        _native.INVALID_ARGUMENT,
        9,
        error_snapshot_token,
        "target rejected value",
    )
    assert snapshot.direction == direction
    assert snapshot.rejected_property.get() == 20
    assert error_snapshot_token in api.releases
    api.binding_snapshot_result = (0, 0, 0)
    assert binding.error_snapshot() is None
    assert snapshot.error.message == "target rejected value"

    snapshot.rejected_property.close()
    binding.close()
    source.close()
    target.close()
    runtime.close()


@pytest.mark.parametrize(
    "output",
    [
        (800, 0, int(BindingDirection.SOURCE_TO_TARGET)),
        (0, 801, int(BindingDirection.TARGET_TO_SOURCE)),
        (800, 801, 99),
        (0, 0, int(BindingDirection.SOURCE_TO_TARGET)),
    ],
)
def test_malformed_error_snapshot_releases_all_owned_outputs(
    output: tuple[int, int, int],
) -> None:
    runtime, api = make_runtime()
    source = Property.create(runtime, 1)
    target = Property.create(runtime, 2)
    binding = Binding.create(source, target, BindingMode.ONE_WAY)
    assert binding is not None
    api.binding_snapshot_result = output

    with pytest.raises(UiError) as raised:
        binding.error_snapshot()

    assert raised.value.raw_status == _native.INTERNAL_ERROR
    assert {token for token in output[:2] if token} <= set(api.releases)
    binding.close()
    source.close()
    target.close()
    runtime.close()


def test_failed_error_snapshot_releases_unexpected_owned_outputs() -> None:
    runtime, api = make_runtime()
    source = Property.create(runtime, 1)
    target = Property.create(runtime, 2)
    binding = Binding.create(source, target, BindingMode.ONE_WAY)
    assert binding is not None
    api.binding_snapshot_status = _native.INVALID_ARGUMENT
    api.binding_snapshot_result = (
        810,
        811,
        int(BindingDirection.SOURCE_TO_TARGET),
    )

    with pytest.raises(UiError) as raised:
        binding.error_snapshot()

    assert raised.value.raw_status == _native.INVALID_ARGUMENT
    assert {810, 811} <= set(api.releases)
    binding.close()
    source.close()
    target.close()
    runtime.close()


def test_error_snapshot_waits_for_inflight_call_before_close() -> None:
    runtime, api = make_runtime()
    source = Property.create(runtime, 1)
    target = Property.create(runtime, 2)
    binding = Binding.create(source, target, BindingMode.ONE_WAY)
    assert binding is not None
    token = binding._cell.token
    api.snapshot_entered = threading.Event()
    api.snapshot_continue = threading.Event()
    result: list[object] = []

    worker = threading.Thread(
        target=lambda: result.append(binding.error_snapshot())
    )
    worker.start()
    assert api.snapshot_entered.wait(timeout=2)
    closer = threading.Thread(target=binding.close)
    closer.start()
    api.snapshot_continue.set()
    worker.join(timeout=2)
    closer.join(timeout=2)

    assert not worker.is_alive()
    assert not closer.is_alive()
    assert result == [None]
    assert api.releases.count(token) == 1
    source.close()
    target.close()
    runtime.close()

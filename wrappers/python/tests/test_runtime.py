"""Focused tests for the public Runtime facade with a fake C ABI."""

from __future__ import annotations

import gc
import threading
import weakref

import pytest

import freemvvm
from freemvvm import (
    ErrorSnapshot,
    MissingRuntimeError,
    Runtime,
    ShutdownDisposition,
    ShutdownMode,
    ShutdownRequestState,
    UiError,
)
from freemvvm import _native


class FakeApi:
    def __init__(self) -> None:
        self.version = (0, 1, 0, 0)
        self.create = (0, 41, 0)
        self.is_ui_thread = (0, 1)
        self.shutdown = (0, 0)
        self.request = (0, 51, 0)
        self.query = (0, 0, 0, 0)
        self.releases: list[int] = []
        self.release_statuses: list[int] = []
        self.calls: list[tuple[str, tuple[int, ...]]] = []
        self.error_status_value = (0, 0)
        self.error_code_value = (0, 0, 0)
        self.error_message = ""
        self.query_entered: threading.Event | None = None
        self.query_continue: threading.Event | None = None

    def get_version(self) -> tuple[int, int, int, int]:
        self.calls.append(("version", ()))
        return self.version

    def runtime_create(self) -> tuple[int, int, int]:
        self.calls.append(("create", ()))
        return self.create

    def runtime_is_ui_thread(self, runtime: int) -> tuple[int, int]:
        self.calls.append(("is_ui_thread", (runtime,)))
        if self.query_entered is not None:
            self.query_entered.set()
        if self.query_continue is not None:
            self.query_continue.wait(timeout=2)
        return self.is_ui_thread

    def runtime_shutdown(self, runtime: int) -> tuple[int, int]:
        self.calls.append(("shutdown", (runtime,)))
        return self.shutdown

    def runtime_request_shutdown(self, runtime: int, mode: int) -> tuple[int, int, int]:
        self.calls.append(("request_shutdown", (runtime, mode)))
        return self.request

    def shutdown_request_query(self, request: int) -> tuple[int, int, int, int]:
        self.calls.append(("query_request", (request,)))
        return self.query

    def handle_release(self, owned: int) -> int:
        self.calls.append(("release", (owned,)))
        self.releases.append(owned)
        return self.release_statuses.pop(0) if self.release_statuses else _native.OK

    def error_status(self, error: int) -> tuple[int, int]:
        return self.error_status_value

    def error_code(self, error: int) -> tuple[int, int, int]:
        return self.error_code_value

    def error_message_utf8(self, error: int, buffer: bytearray | None) -> tuple[int, int]:
        encoded = self.error_message.encode("utf-8")
        if buffer is None or len(buffer) < len(encoded):
            return _native.BUFFER_TOO_SMALL, len(encoded)
        buffer[: len(encoded)] = encoded
        return _native.OK, len(encoded)


def create_runtime(api: FakeApi | None = None) -> tuple[Runtime, FakeApi]:
    fake = api or FakeApi()
    return Runtime._create_with_api(fake), fake


def test_create_checks_abi_major_and_preserves_version_failure() -> None:
    api = FakeApi()
    api.version = (_native.OK, 2, 0, 0)

    with pytest.raises(UiError) as raised:
        Runtime._create_with_api(api)

    assert raised.value.snapshot == ErrorSnapshot(_native.UNSUPPORTED)
    assert not any(name == "create" for name, _ in api.calls)

    api.version = (_native.INTERNAL_ERROR, 0, 0, 0)
    with pytest.raises(UiError) as raised:
        Runtime._create_with_api(api)
    assert raised.value.raw_status == _native.INTERNAL_ERROR

    api = FakeApi()
    api.create = (_native.OUT_OF_MEMORY, 0, 0)
    with pytest.raises(UiError) as raised:
        Runtime._create_with_api(api)
    assert raised.value.raw_status == _native.OUT_OF_MEMORY
    assert api.releases == []

    api = FakeApi()
    api.create = (_native.INTERNAL_ERROR, 42, 0)
    with pytest.raises(UiError) as raised:
        Runtime._create_with_api(api)
    assert raised.value.raw_status == _native.INTERNAL_ERROR
    assert api.releases == [42]

    with pytest.raises(TypeError, match="Runtime.create"):
        Runtime()


def test_public_factory_uses_private_provider_and_reports_missing_runtime(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    api = FakeApi()
    monkeypatch.setattr(freemvvm, "_load_native_api", lambda: api)
    runtime = Runtime.create()
    assert runtime.is_ui_thread()
    runtime.close()

    def fail_to_load() -> None:
        raise OSError("not installed")

    monkeypatch.setattr(freemvvm, "_load_native_api", fail_to_load)
    with pytest.raises(MissingRuntimeError, match="not installed"):
        Runtime.create()


def test_runtime_context_thread_query_and_closed_state() -> None:
    runtime, api = create_runtime()
    assert runtime.is_ui_thread() is True
    context = runtime.ui_context()
    context.check()

    api.is_ui_thread = (_native.OK, 0)
    with pytest.raises(UiError) as raised:
        context.check()
    assert raised.value.raw_status == _native.WRONG_THREAD

    api.is_ui_thread = (_native.OK, 2)
    with pytest.raises(UiError) as raised:
        runtime.is_ui_thread()
    assert raised.value.raw_status == _native.INTERNAL_ERROR

    api.is_ui_thread = (_native.OK, 1)
    runtime.close()
    with pytest.raises(UiError) as raised:
        runtime.is_ui_thread()
    assert raised.value.raw_status == _native.CLOSED


def test_close_is_separate_from_shutdown_and_failed_close_can_retry() -> None:
    runtime, api = create_runtime()
    api.release_statuses = [_native.INVALID_HANDLE, _native.OK]

    with pytest.raises(UiError) as raised:
        runtime.close()
    assert raised.value.raw_status == _native.INVALID_HANDLE
    assert not any(name == "shutdown" for name, _ in api.calls)

    runtime.close()
    runtime.close()
    assert api.releases == [41, 41]
    assert [name for name, _ in api.calls].count("shutdown") == 0


def test_shutdown_pending_is_accepted_and_error_details_are_retained() -> None:
    runtime, api = create_runtime()
    api.shutdown = (_native.PENDING, 0)
    assert runtime.shutdown() == ShutdownDisposition.ACCEPTED_PENDING

    api.shutdown = (_native.INTERNAL_ERROR, 61)
    api.error_status_value = (_native.OK, 37)
    api.error_code_value = (_native.OK, 0x1234, 0x5678)
    api.error_message = "shutdown pending"

    with pytest.raises(UiError) as raised:
        runtime.shutdown()
    assert raised.value.snapshot == ErrorSnapshot(37, 0x1234, 0x5678, "shutdown pending")
    assert api.releases[-1] == 61
    runtime.close()


def test_request_receipt_preserves_raw_state_and_release_does_not_cancel() -> None:
    runtime, api = create_runtime()
    with pytest.raises(ValueError, match="unsupported shutdown mode"):
        runtime.request_shutdown(True)
    api.request = (_native.PENDING, 51, 0)
    request = runtime.request_shutdown(ShutdownMode.ASYNC)
    assert api.calls[-1] == ("request_shutdown", (41, _native.SHUTDOWN_ASYNC))

    api.query = (_native.OK, _native.SHUTDOWN_REQUEST_HOST_UNAVAILABLE, 9, 0)
    snapshot = request.query()
    assert snapshot.known_state is ShutdownRequestState.HOST_UNAVAILABLE
    assert snapshot.reason == 9

    api.query = (_native.OK, 0xFFFFFFFF, 7, 0)
    snapshot = request.query()
    assert snapshot.known_state is None
    assert snapshot.raw_state == 0xFFFFFFFF

    request.close()
    assert api.releases.count(51) == 1
    assert [name for name, _ in api.calls].count("request_shutdown") == 1
    runtime.close()


def test_context_manager_preserves_body_error_when_close_also_fails() -> None:
    runtime, api = create_runtime()
    api.release_statuses = [_native.INVALID_HANDLE]

    with pytest.raises(ValueError, match="body") as raised:
        with runtime:
            raise ValueError("body")

    assert any("close also failed" in note for note in raised.value.__notes__)
    runtime.close()


def test_close_waits_for_an_inflight_native_call() -> None:
    api = FakeApi()
    api.query_entered = threading.Event()
    api.query_continue = threading.Event()
    runtime, _ = create_runtime(api)
    call_thread = threading.Thread(target=runtime.is_ui_thread)
    call_thread.start()
    assert api.query_entered.wait(timeout=2)

    closed = threading.Event()
    close_thread = threading.Thread(target=lambda: (runtime.close(), closed.set()))
    close_thread.start()
    assert not closed.wait(timeout=0.05)
    assert 41 not in api.releases

    api.query_continue.set()
    call_thread.join(timeout=2)
    close_thread.join(timeout=2)
    assert not call_thread.is_alive()
    assert not close_thread.is_alive()
    assert api.releases == [41]


def test_owner_finalizer_hands_off_release_for_next_safe_point() -> None:
    runtime, api = create_runtime()
    reference = weakref.ref(runtime)
    del runtime
    gc.collect()

    assert reference() is None
    assert api.releases == []
    runtime2, _ = create_runtime()
    runtime2.is_ui_thread()
    assert 41 in api.releases
    runtime2.close()

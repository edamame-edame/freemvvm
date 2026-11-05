"""Fake-ABI tests for Python Window peer and close-request ownership."""

from __future__ import annotations

import pytest

from test_binding import make_runtime
from test_controls import FakeControlsApi

from freemvvm import (
    CloseRequest,
    DeferredCloseRequest,
    UiError,
    Window,
    WindowCloseSubscription,
)
from freemvvm import _native
from freemvvm import _CleanupManager, _root_get


class FakeWindowApi(FakeControlsApi):
    def __init__(self) -> None:
        super().__init__()
        self.window_status: dict[str, tuple[int, int]] = {}
        self.background_rgb = (1.0, 1.0, 1.0)
        self.background_error = 0
        self.subscribe_result: tuple[int, int, int] | None = None
        self.subscription_callback = None
        self.subscription_userdata = 0
        self.subscription_destroy = None
        self.subscription_token = 8_000
        self.subscription_cancelled = False
        self.callback_active = False
        self.destroy_after_callback = False
        self.deferred_result: tuple[int, int, int] | None = None
        self.decisions: list[tuple[int, int]] = []

    def window_background_rgb(
        self, window: int
    ) -> tuple[int, float, float, float, int]:
        self.calls.append(("window_background_rgb", (window,)))
        status, error = self.window_status.get(
            "window_background_rgb", (_native.OK, 0)
        )
        return status, *self.background_rgb, error or self.background_error

    def window_set_background_rgb(
        self, window: int, red: float, green: float, blue: float
    ) -> tuple[int, int]:
        self.calls.append(
            ("window_set_background_rgb", (window, red, green, blue))
        )
        status, error = self.window_status.get(
            "window_set_background_rgb", (_native.OK, 0)
        )
        if status == _native.OK and not error:
            if not all(
                0.0 <= component <= 1.0 for component in (red, green, blue)
            ):
                return _native.INVALID_ARGUMENT, 0
            self.background_rgb = (red, green, blue)
        return status, error

    def window_reset_background_rgb(self, window: int) -> tuple[int, int]:
        self.calls.append(("window_reset_background_rgb", (window,)))
        status, error = self.window_status.get(
            "window_reset_background_rgb", (_native.OK, 0)
        )
        if status == _native.OK and not error:
            self.background_rgb = (1.0, 1.0, 1.0)
        return status, error

    def window_open(self, window: int) -> tuple[int, int]:
        self.calls.append(("window_open", (window,)))
        return self.window_status.get("window_open", (_native.OK, 0))

    def window_close(self, window: int) -> tuple[int, int]:
        self.calls.append(("window_close", (window,)))
        return self.window_status.get("window_close", (_native.OK, 0))

    def window_close_request_subscribe(
        self,
        window: int,
        callback: object,
        userdata: int,
        destroy: object,
    ) -> tuple[int, int, int]:
        self.calls.append(("window_close_request_subscribe", (window,)))
        if self.subscribe_result is not None:
            return self.subscribe_result
        self.subscription_callback = callback
        self.subscription_userdata = userdata
        self.subscription_destroy = destroy
        return _native.OK, self.subscription_token, 0

    def window_close_request_defer(
        self, borrowed_request: int
    ) -> tuple[int, int, int]:
        if self.deferred_result is not None:
            return self.deferred_result
        return _native.OK, 9_000 + borrowed_request, 0

    def window_close_request_resolve(
        self, request: int, decision: int
    ) -> tuple[int, int]:
        self.decisions.append((request, decision))
        return self.window_status.get("resolve", (_native.OK, 0))

    def subscription_cancel(self, subscription: int) -> tuple[int, int]:
        self.calls.append(("subscription_cancel", (subscription,)))
        status, error = self.window_status.get(
            "subscription_cancel", (_native.OK, 0)
        )
        if status == _native.OK and not error:
            self.subscription_cancelled = True
            if self.callback_active:
                self.destroy_after_callback = True
            else:
                self.destroy_subscription_root()
        return status, error

    def fire_close_request(self, token: int = 7_777) -> None:
        if self.subscription_cancelled:
            return
        assert self.subscription_callback is not None
        self.callback_active = True
        try:
            self.subscription_callback(self.subscription_userdata, token)
        finally:
            self.callback_active = False
            if self.destroy_after_callback:
                self.destroy_after_callback = False
                self.destroy_subscription_root()

    def destroy_subscription_root(self) -> None:
        if self.subscription_destroy is not None:
            destroy = self.subscription_destroy
            self.subscription_destroy = None
            destroy(self.subscription_userdata)
            self.subscription_callback = None


def _create_window(api: FakeWindowApi) -> tuple[object, Window]:
    runtime, _ = make_runtime(api)
    return runtime, Window.create(runtime)


def test_window_background_rgb_get_set_and_reset() -> None:
    api = FakeWindowApi()
    _runtime, window = _create_window(api)

    assert window.background_rgb == (1.0, 1.0, 1.0)
    window.background_rgb = (0.2, 0.4, 0.8)
    assert window.background_rgb == pytest.approx((0.2, 0.4, 0.8))
    window.reset_background_rgb()
    assert window.background_rgb == (1.0, 1.0, 1.0)
    assert [name for name, _args in api.calls if "background_rgb" in name] == [
        "window_background_rgb",
        "window_set_background_rgb",
        "window_background_rgb",
        "window_reset_background_rgb",
        "window_background_rgb",
    ]


def test_window_background_rgb_delegates_validation_and_maps_native_error() -> (
    None
):
    api = FakeWindowApi()
    _runtime, window = _create_window(api)

    with pytest.raises(UiError) as raised:
        window.background_rgb = (1.1, 0.0, 0.0)
    assert raised.value.raw_status == _native.INVALID_ARGUMENT
    assert api.calls[-1] == (
        "window_set_background_rgb",
        (window._cell.token, 1.1, 0.0, 0.0),
    )

    api.window_status["window_background_rgb"] = (_native.CLOSED, 0)
    with pytest.raises(UiError) as raised:
        _ = window.background_rgb
    assert raised.value.raw_status == _native.CLOSED


@pytest.mark.parametrize(
    "value", [(0.0, 0.0), (0.0, 0.0, 0.0, 0.0), [0.0, 0.0, 0.0]]
)
def test_window_background_rgb_requires_three_item_tuple(
    value: object,
) -> None:
    api = FakeWindowApi()
    _runtime, window = _create_window(api)

    with pytest.raises(TypeError):
        window.background_rgb = value  # type: ignore[assignment]
    assert not any(
        name == "window_set_background_rgb" for name, _ in api.calls
    )


def test_window_peer_close_is_distinct_from_owned_handle_release() -> None:
    api = FakeWindowApi()
    runtime, window = _create_window(api)

    window.open()
    window.close_peer()
    assert [
        name for name, _args in api.calls if name.startswith("window_")
    ] == [
        "window_create",
        "window_open",
        "window_close",
    ]
    token = window._cell.token
    api.window_status["window_close"] = (_native.CLOSED, 0)
    with pytest.raises(UiError) as raised:
        window.close_peer()
    assert raised.value.raw_status == _native.CLOSED
    assert window._cell.token == token
    api.window_status.pop("window_close")
    window.close()
    assert window._cell.token == 0
    runtime.close()


def test_window_peer_calls_check_thread_and_closed_owner_before_native_call() -> (
    None
):
    api = FakeWindowApi()
    runtime, window = _create_window(api)
    api.ui_thread = 0

    with pytest.raises(UiError) as wrong_thread:
        window.open()
    assert wrong_thread.value.raw_status == _native.WRONG_THREAD
    assert not any(name == "window_open" for name, _args in api.calls)

    api.ui_thread = 1
    window.close()
    before = len(api.calls)
    with pytest.raises(UiError) as closed:
        window.close_peer()
    assert closed.value.raw_status == _native.CLOSED
    assert not any(
        name == "window_close" for name, _args in api.calls[before:]
    )
    runtime.close()


def test_close_request_immediate_decisions_and_borrow_expiry() -> None:
    api = FakeWindowApi()
    runtime, window = _create_window(api)
    requests: list[CloseRequest] = []
    subscription = window.on_close_request(requests.append)

    api.fire_close_request(111)
    assert api.decisions == []
    assert len(requests) == 1
    with pytest.raises(UiError) as expired:
        requests[0].accept()
    assert expired.value.raw_status == _native.CLOSED

    window2 = Window.create(runtime)
    immediate = window2.on_close_request(lambda request: request.accept())
    api.subscription_cancelled = False
    api.fire_close_request(222)
    assert api.decisions[-1] == (222, _native.CLOSE_ACCEPT)

    immediate.cancel()
    immediate.close()
    subscription.cancel()
    subscription.close()
    window2.close()
    window.close()
    runtime.close()


def test_deferred_request_resolves_and_keeps_release_separate() -> None:
    api = FakeWindowApi()
    runtime, window = _create_window(api)
    retained: list[DeferredCloseRequest] = []
    subscription = window.on_close_request(
        lambda request: retained.append(request.defer())
    )

    api.fire_close_request(321)
    deferred = retained[0]
    assert isinstance(deferred, DeferredCloseRequest)
    assert api.decisions == []
    deferred.decline()
    assert api.decisions == [(9_321, _native.CLOSE_DECLINE)]
    with pytest.raises(UiError) as duplicate:
        deferred.accept()
    assert duplicate.value.raw_status == _native.CLOSED
    assert deferred._cell.token == 9_321
    deferred.close()
    assert deferred._cell.token == 0

    subscription.cancel()
    subscription.close()
    window.close()
    runtime.close()


def test_stale_resolution_failure_preserves_deferred_owner_for_retry() -> None:
    api = FakeWindowApi()
    runtime, window = _create_window(api)
    deferred_requests: list[DeferredCloseRequest] = []
    subscription = window.on_close_request(
        lambda request: deferred_requests.append(request.defer())
    )
    api.fire_close_request(432)
    deferred = deferred_requests[0]
    api.window_status["resolve"] = (_native.INVALID_HANDLE, 0)

    with pytest.raises(UiError) as stale:
        deferred.accept()

    assert stale.value.raw_status == _native.INVALID_HANDLE
    assert deferred._cell.token == 9_432
    api.window_status.pop("resolve")
    deferred.accept()
    assert api.decisions[-1] == (9_432, _native.CLOSE_ACCEPT)
    deferred.close()
    subscription.cancel()
    subscription.close()
    window.close()
    runtime.close()


def test_defer_failure_releases_unexpected_output_and_keeps_borrow_active(
    capsys: pytest.CaptureFixture[str],
) -> None:
    api = FakeWindowApi()
    runtime, window = _create_window(api)
    api.deferred_result = (_native.INVALID_HANDLE, 9_888, 0)
    observed: list[CloseRequest] = []

    def handle(request: CloseRequest) -> None:
        observed.append(request)
        with pytest.raises(UiError):
            request.defer()
        request.accept()

    subscription = window.on_close_request(handle)
    api.fire_close_request(888)

    assert api.releases == [9_888]
    assert api.decisions == [(888, _native.CLOSE_ACCEPT)]
    assert "" == capsys.readouterr().err
    with pytest.raises(UiError) as expired:
        observed[0].decline()
    assert expired.value.raw_status == _native.CLOSED

    subscription.cancel()
    subscription.close()
    window.close()
    runtime.close()


def test_callback_exception_is_reported_and_does_not_cross_abi(
    capsys: pytest.CaptureFixture[str],
) -> None:
    api = FakeWindowApi()
    runtime, window = _create_window(api)

    def fail(_request: CloseRequest) -> None:
        raise ValueError("callback failure")

    subscription = window.on_close_request(fail)
    api.fire_close_request()
    assert "callback failure" in capsys.readouterr().err
    assert api.decisions == []

    subscription.cancel()
    subscription.close()
    window.close()
    runtime.close()


def test_registration_failure_returns_root_and_unexpected_token() -> None:
    api = FakeWindowApi()
    runtime, window = _create_window(api)
    api.subscribe_result = (_native.INVALID_ARGUMENT, 8_111, 0)

    with pytest.raises(UiError) as raised:
        window.on_close_request(lambda _request: None)

    assert raised.value.raw_status == _native.INVALID_ARGUMENT
    assert api.releases == [8_111]
    assert api.subscription_callback is None
    window.close()
    runtime.close()


def test_cancel_during_callback_waits_for_inflight_root_destruction() -> None:
    api = FakeWindowApi()
    runtime, window = _create_window(api)
    holder: list[WindowCloseSubscription] = []
    callback_roots: list[int] = []

    def cancel_inside_callback(_request: CloseRequest) -> None:
        key = api.subscription_userdata
        callback_roots.append(key)
        holder[0].cancel()
        assert _root_get(key) is not None

    subscription = window.on_close_request(cancel_inside_callback)
    holder.append(subscription)
    api.fire_close_request()
    assert api.subscription_cancelled
    assert _root_get(callback_roots[0]) is None
    api.fire_close_request()
    assert len(callback_roots) == 1

    _CleanupManager.retry_pending()
    subscription.close()
    window.close()
    runtime.close()


def test_subscription_cancel_failure_keeps_owner_and_allows_retry() -> None:
    api = FakeWindowApi()
    runtime, window = _create_window(api)
    subscription = window.on_close_request(lambda _request: None)
    token = subscription._cell.token
    api.window_status["subscription_cancel"] = (_native.CLOSED, 0)

    with pytest.raises(UiError) as raised:
        subscription.cancel()

    assert raised.value.raw_status == _native.CLOSED
    assert subscription._cell.token == token
    assert _root_get(api.subscription_userdata) is not None
    api.window_status.pop("subscription_cancel")
    subscription.cancel()
    assert api.subscription_cancelled
    subscription.close()
    window.close()
    runtime.close()


def test_subscription_handle_release_does_not_cancel_registration() -> None:
    api = FakeWindowApi()
    runtime, window = _create_window(api)
    subscription = window.on_close_request(lambda _request: None)

    subscription.close()

    assert not api.subscription_cancelled
    assert _root_get(api.subscription_userdata) is not None
    api.fire_close_request()
    api.destroy_subscription_root()
    _CleanupManager.retry_pending()
    window.close()
    runtime.close()

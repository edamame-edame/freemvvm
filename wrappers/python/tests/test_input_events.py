"""Fake-ABI tests for Python Input event views and subscriptions."""

from __future__ import annotations

import gc
import threading
import weakref

import pytest

from test_binding import make_runtime
from test_controls import FakeControlsApi

from freemvvm import (
    Button,
    InputEventKind,
    InputEventMask,
    UiError,
    UiSubscription,
)
from freemvvm import _native
from freemvvm import _CleanupManager


class FakeInputApi(FakeControlsApi):
    def __init__(self) -> None:
        super().__init__()
        self.input_callback = None
        self.input_userdata = 0
        self.input_destroy = None
        self.input_subscription = 8_500
        self.input_registration_result: tuple[int, int, int] | None = None
        self.last_input_mask = 0
        self.input_cancel_status = _native.OK
        self.input_cancelled = False
        self.input_callback_active = False
        self.destroy_after_callback = False
        self.input_destroy_count = 0
        self.events: dict[int, dict[str, object]] = {}
        self.next_event = 9_000

    def widget_input_subscribe(
        self,
        widget: int,
        event_mask: int,
        callback: object,
        userdata: int,
        destroy: object,
    ) -> tuple[int, int, int]:
        self.calls.append(
            ("widget_input_subscribe", (widget, event_mask))
        )
        self.last_input_mask = event_mask
        if self.input_registration_result is not None:
            return self.input_registration_result
        self.input_callback = callback
        self.input_userdata = userdata
        self.input_destroy = destroy
        self.input_cancelled = False
        return _native.OK, self.input_subscription, 0

    def _event(
        self, token: int, kinds: tuple[int, ...]
    ) -> tuple[int, dict[str, object] | None]:
        event = self.events.get(token)
        if event is None:
            return _native.INVALID_HANDLE, None
        if int(event["kind"]) not in kinds:
            return _native.WRONG_TYPE, None
        return _native.OK, event

    def input_event_kind(self, borrowed_event: int) -> tuple[int, int]:
        event = self.events.get(borrowed_event)
        if event is None:
            return _native.INVALID_HANDLE, 0
        return _native.OK, int(event["kind"])

    def input_event_key(self, borrowed_event: int) -> tuple[int, int, int]:
        status, event = self._event(
            borrowed_event,
            (_native.INPUT_KEY_DOWN, _native.INPUT_KEY_UP),
        )
        if event is None:
            return status, 0, 0
        return status, int(event["key"]), int(event["modifiers"])

    def input_event_text(
        self, borrowed_event: int, buffer: bytearray | None
    ) -> tuple[int, int]:
        status, event = self._event(
            borrowed_event, (_native.INPUT_TEXT_COMMIT,)
        )
        if event is None:
            return status, 0
        encoded = str(event["text"]).encode("utf-8")
        if buffer is None:
            return (
                (_native.BUFFER_TOO_SMALL if encoded else _native.OK),
                len(encoded),
            )
        if len(buffer) < len(encoded):
            return _native.BUFFER_TOO_SMALL, len(encoded)
        buffer[: len(encoded)] = encoded
        return _native.OK, len(encoded)

    def input_event_pointer(
        self, borrowed_event: int
    ) -> tuple[int, float, float, int]:
        status, event = self._event(
            borrowed_event,
            (
                _native.INPUT_PRIMARY_POINTER_DOWN,
                _native.INPUT_PRIMARY_POINTER_UP,
            ),
        )
        if event is None:
            return status, 0.0, 0.0, 0
        return (
            status,
            float(event["x"]),
            float(event["y"]),
            int(event["button"]),
        )

    def input_event_set_handled(
        self, borrowed_event: int, handled: int
    ) -> tuple[int, int]:
        status, event = self._event(
            borrowed_event,
            (
                _native.INPUT_KEY_DOWN,
                _native.INPUT_KEY_UP,
                _native.INPUT_TEXT_COMMIT,
                _native.INPUT_PRIMARY_POINTER_DOWN,
                _native.INPUT_PRIMARY_POINTER_UP,
            ),
        )
        if event is None:
            return status, 0
        if handled != 1:
            return _native.INVALID_ARGUMENT, 0
        event["handled"] = True
        return _native.OK, 0

    def subscription_cancel(self, subscription: int) -> tuple[int, int]:
        self.calls.append(("subscription_cancel", (subscription,)))
        if self.input_cancel_status != _native.OK:
            return self.input_cancel_status, 0
        if not self.input_cancelled:
            self.input_cancelled = True
            if self.input_callback_active:
                self.destroy_after_callback = True
            else:
                self.destroy_input_root()
        return _native.OK, 0

    def destroy_input_root(self) -> None:
        if self.input_destroy is not None:
            destroy = self.input_destroy
            self.input_destroy = None
            self.input_destroy_count += 1
            destroy(self.input_userdata)

    def fire_input(
        self,
        kind: InputEventKind,
        *,
        key: int = 0x04,
        modifiers: int = 0,
        text: str = "",
        x: float = 10.5,
        y: float = 20.25,
        button: int = 1,
    ) -> dict[str, object] | None:
        if self.input_callback is None or self.input_cancelled:
            return None
        if not (int(self.input_mask) & (1 << (int(kind) - 1))):
            return None
        self.next_event += 1
        token = self.next_event
        event: dict[str, object] = {
            "kind": int(kind),
            "key": key,
            "modifiers": modifiers,
            "text": text,
            "x": x,
            "y": y,
            "button": button,
            "handled": False,
        }
        self.events[token] = event
        self.input_callback_active = True
        try:
            self.input_callback(
                self.input_userdata,
                1_001,
                1_001,
                token,
            )
        finally:
            self.input_callback_active = False
            self.events.pop(token, None)
            if self.destroy_after_callback:
                self.destroy_after_callback = False
                self.destroy_input_root()
        return event

    @property
    def input_mask(self) -> int:
        return self.last_input_mask


def _create(api: FakeInputApi):
    runtime, _ = make_runtime(api)
    widget = Button.create(runtime)
    return runtime, widget


def test_all_event_kinds_map_to_their_payloads_and_copy_text() -> None:
    api = FakeInputApi()
    runtime, widget = _create(api)
    received: list[tuple[object, ...]] = []

    def receive(event) -> None:
        if event.kind in (InputEventKind.KEY_DOWN, InputEventKind.KEY_UP):
            received.append((event.kind, event.key_identity, event.modifiers))
        elif event.kind is InputEventKind.TEXT_COMMIT:
            received.append((event.kind, event.text))
        else:
            received.append(
                (event.kind, event.position_dip, event.button)
            )

    subscription = widget.subscribe_input(InputEventMask.ALL, receive)
    assert isinstance(subscription, UiSubscription)
    text_event = "日本語 🙂"
    api.fire_input(InputEventKind.KEY_DOWN, modifiers=3)
    api.fire_input(InputEventKind.KEY_UP)
    api.fire_input(InputEventKind.TEXT_COMMIT, text=text_event)
    api.fire_input(InputEventKind.PRIMARY_POINTER_DOWN)
    api.fire_input(InputEventKind.PRIMARY_POINTER_UP)

    assert received == [
        (InputEventKind.KEY_DOWN, 0x04, 3),
        (InputEventKind.KEY_UP, 0x04, 0),
        (InputEventKind.TEXT_COMMIT, text_event),
        (InputEventKind.PRIMARY_POINTER_DOWN, (10.5, 20.25), 1),
        (InputEventKind.PRIMARY_POINTER_UP, (10.5, 20.25), 1),
    ]
    subscription.cancel()
    subscription.close()
    widget.close()
    runtime.close()


def test_event_view_expires_and_wrong_kind_getters_raise_native_status() -> None:
    api = FakeInputApi()
    runtime, widget = _create(api)
    views = []
    wrong_kind_status: list[int] = []

    def receive(event) -> None:
        views.append(event)
        with pytest.raises(UiError) as raised:
            _ = event.text
        wrong_kind_status.append(raised.value.raw_status)

    subscription = widget.subscribe_input(InputEventMask.KEY_DOWN, receive)
    api.fire_input(InputEventKind.KEY_DOWN)

    assert wrong_kind_status == [_native.WRONG_TYPE]
    with pytest.raises(RuntimeError, match="callback-scoped"):
        _ = views[0].kind

    subscription.cancel()
    subscription.close()
    widget.close()
    runtime.close()


def test_registration_validation_and_failure_do_not_keep_callable_root() -> None:
    api = FakeInputApi()
    runtime, widget = _create(api)
    before = len(api.calls)
    with pytest.raises(ValueError):
        widget.subscribe_input(InputEventMask(0), lambda _event: None)
    with pytest.raises(ValueError):
        widget.subscribe_input(InputEventMask(1 << 8), lambda _event: None)
    assert len(api.calls) == before

    called = []

    def callback(_event) -> None:
        called.append(True)

    callback_ref = weakref.ref(callback)
    api.input_registration_result = (_native.INVALID_ARGUMENT, 0, 0)
    with pytest.raises(UiError) as raised:
        widget.subscribe_input(InputEventMask.KEY_DOWN, callback)
    assert raised.value.raw_status == _native.INVALID_ARGUMENT
    del raised
    del callback
    gc.collect()
    _CleanupManager.retry_pending()
    assert callback_ref() is None
    assert called == []

    widget.close()
    runtime.close()


def test_callback_root_survives_gc_and_destroy_is_once_after_cancel() -> None:
    api = FakeInputApi()
    runtime, widget = _create(api)
    observed: list[InputEventKind] = []

    def callback(event) -> None:
        observed.append(event.kind)

    callback_ref = weakref.ref(callback)
    subscription = widget.subscribe_input(InputEventMask.KEY_DOWN, callback)
    del callback
    gc.collect()
    assert callback_ref() is not None

    api.fire_input(InputEventKind.KEY_DOWN)
    subscription.cancel()
    assert api.input_destroy_count == 1
    _CleanupManager.retry_pending()
    gc.collect()
    assert callback_ref() is None
    api.fire_input(InputEventKind.KEY_DOWN)
    assert observed == [InputEventKind.KEY_DOWN]

    subscription.close()
    widget.close()
    runtime.close()
    assert api.input_destroy_count == 1


def test_self_cancel_and_close_during_callback_do_not_deadlock() -> None:
    api = FakeInputApi()
    runtime, widget = _create(api)
    seen: list[str] = []
    owner: list[UiSubscription] = []

    def cancel_self(_event) -> None:
        seen.append("cancel")
        owner[0].cancel()

    owner.append(widget.subscribe_input(InputEventMask.KEY_DOWN, cancel_self))
    api.fire_input(InputEventKind.KEY_DOWN)
    api.fire_input(InputEventKind.KEY_DOWN)
    assert seen == ["cancel"]
    assert api.input_destroy_count == 1
    owner[0].close()

    def close_self(_event) -> None:
        seen.append("close")
        closing[0].close()

    closing: list[UiSubscription] = [
        widget.subscribe_input(InputEventMask.KEY_UP, close_self)
    ]
    api.fire_input(InputEventKind.KEY_UP)
    assert seen == ["cancel", "close"]
    assert not api.input_cancelled
    api.destroy_input_root()
    _CleanupManager.retry_pending()
    widget.close()
    runtime.close()


def test_callback_exception_is_contained_and_marks_event_handled(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    api = FakeInputApi()
    runtime, widget = _create(api)
    reported: list[BaseException] = []
    monkeypatch.setattr(
        "freemvvm._report_callback_exception",
        lambda error: reported.append(error),
    )
    expected = ValueError("callback failed")

    def fail(_event) -> None:
        raise expected

    subscription = widget.subscribe_input(InputEventMask.TEXT_COMMIT, fail)
    result = api.fire_input(InputEventKind.TEXT_COMMIT, text="x")

    assert result is not None and result["handled"] is True
    assert reported == [expected]
    subscription.cancel()
    subscription.close()
    widget.close()
    runtime.close()


def test_exception_reporter_failure_uses_interpreter_fallback(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    api = FakeInputApi()
    runtime, widget = _create(api)
    reported: list[BaseException] = []

    def fail_reporter(_error: BaseException) -> None:
        raise RuntimeError("reporter failed")

    monkeypatch.setattr(
        "freemvvm._report_callback_exception", fail_reporter
    )
    monkeypatch.setattr(
        "freemvvm.sys.__excepthook__",
        lambda _kind, error, _traceback: reported.append(error),
    )

    def fail_callback(_event) -> None:
        raise ValueError("callback failed")

    subscription = widget.subscribe_input(
        InputEventMask.TEXT_COMMIT, fail_callback
    )
    result = api.fire_input(InputEventKind.TEXT_COMMIT, text="x")

    assert result is not None and result["handled"] is True
    assert len(reported) == 1 and isinstance(reported[0], RuntimeError)
    assert str(reported[0]) == "reporter failed"
    subscription.cancel()
    subscription.close()
    widget.close()
    runtime.close()


def test_callback_can_be_entered_from_another_thread() -> None:
    api = FakeInputApi()
    runtime, widget = _create(api)
    received: list[InputEventKind] = []
    subscription = widget.subscribe_input(
        InputEventMask.KEY_DOWN, lambda event: received.append(event.kind)
    )
    worker = threading.Thread(target=api.fire_input, args=(InputEventKind.KEY_DOWN,))
    worker.start()
    worker.join(timeout=2)

    assert not worker.is_alive()
    assert received == [InputEventKind.KEY_DOWN]
    subscription.cancel()
    subscription.close()
    widget.close()
    runtime.close()

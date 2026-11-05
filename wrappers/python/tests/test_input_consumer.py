"""Public-package consumer test for the Python Input event API."""

from __future__ import annotations

from freemvvm import InputEventKind

from input_event_consumer import subscribe_to_all_input
from test_input_events import FakeInputApi, _create


def test_consumer_copies_text_and_cancels_through_public_types() -> None:
    api = FakeInputApi()
    runtime, widget = _create(api)
    observed: list[tuple[InputEventKind, str | None]] = []

    def receive(event) -> None:
        text = event.text if event.kind is InputEventKind.TEXT_COMMIT else None
        observed.append((event.kind, text))

    subscription = subscribe_to_all_input(widget, receive)
    api.fire_input(InputEventKind.KEY_DOWN)
    api.fire_input(InputEventKind.TEXT_COMMIT, text="入力🙂")
    copied_text = observed[1][1]
    api.fire_input(InputEventKind.TEXT_COMMIT, text="later")
    subscription.cancel()
    api.fire_input(InputEventKind.KEY_UP)

    assert copied_text == "入力🙂"
    assert observed == [
        (InputEventKind.KEY_DOWN, None),
        (InputEventKind.TEXT_COMMIT, "入力🙂"),
        (InputEventKind.TEXT_COMMIT, "later"),
    ]
    subscription.close()
    widget.close()
    runtime.close()

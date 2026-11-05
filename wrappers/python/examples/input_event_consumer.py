"""Example public API use for callback-scoped Input events."""

from __future__ import annotations

from collections.abc import Callable

from freemvvm import InputEventMask, InputEventView, UiSubscription, Widget


def subscribe_to_all_input(
    widget: Widget,
    callback: Callable[[InputEventView], None],
) -> UiSubscription:
    """Register one consumer callback for every currently supported kind."""
    return widget.subscribe_input(InputEventMask.ALL, callback)

"""Fake-ABI tests for Python UI Control facades and ownership mapping."""

from __future__ import annotations

import pytest
from collections.abc import Callable

from test_binding import make_runtime
from test_command import FakeCommandApi

from freemvvm import (
    Alignment,
    AsyncCommand,
    Button,
    Container,
    Orientation,
    StackPanel,
    Text,
    TextBox,
    UiError,
    Widget,
    Window,
)
from freemvvm import _native


class FakeControlsApi(FakeCommandApi):
    def __init__(self) -> None:
        super().__init__()
        self.next_control = 1_000
        self.controls: dict[int, str] = {}
        self.window_contents: dict[int, int] = {}
        self.button_commands: dict[int, int] = {}
        self.orientation = _native.ORIENTATION_VERTICAL
        self.control_create_results: dict[str, tuple[int, int, int]] = {}
        self.control_output_results: dict[str, tuple[int, int, int]] = {}
        self.status_results: dict[str, tuple[int, int]] = {}

    def _control(self, kind: str) -> tuple[int, int, int]:
        if kind in self.control_create_results:
            result = self.control_create_results[kind]
            if result[1]:
                self.controls[result[1]] = kind
            return result
        self.next_control += 1
        self.controls[self.next_control] = kind
        return _native.OK, self.next_control, 0

    def _property(
        self, operation: str, kind: str, initial: object
    ) -> tuple[int, int]:
        if operation in self.control_output_results:
            return self.control_output_results[operation][:2]
        status, token, _ = self._create_property(kind, initial)
        return status, token

    def _status(self, operation: str) -> tuple[int, int]:
        return self.status_results.get(operation, (_native.OK, 0))

    def window_create(self, runtime: int) -> tuple[int, int, int]:
        self.calls.append(("window_create", (runtime,)))
        return self._control("window")

    def stack_panel_create(self, runtime: int) -> tuple[int, int, int]:
        self.calls.append(("stack_panel_create", (runtime,)))
        return self._control("stack_panel")

    def text_create(
        self, runtime: int, initial: bytes
    ) -> tuple[int, int, int]:
        self.calls.append(("text_create", (runtime, initial)))
        return self._control("text")

    def text_box_create(
        self, runtime: int, initial: bytes
    ) -> tuple[int, int, int]:
        self.calls.append(("text_box_create", (runtime, initial)))
        return self._control("text_box")

    def button_create(
        self, runtime: int, label: bytes
    ) -> tuple[int, int, int]:
        self.calls.append(("button_create", (runtime, label)))
        return self._control("button")

    def widget_is_enabled_property(self, widget: int) -> tuple[int, int]:
        return self._property("widget_enabled", "bool", True)

    def widget_accessible_name_property(self, widget: int) -> tuple[int, int]:
        return self._property("widget_name", "utf8", b"")

    def widget_request_focus(self, widget: int) -> tuple[int, int]:
        return self._status("widget_request_focus")

    def window_content(self, window: int) -> tuple[int, int, int]:
        if "window_content" in self.control_output_results:
            return self.control_output_results["window_content"]
        child = self.window_contents.get(window, 0)
        return _native.OK, self._token() if child else 0, 0

    def window_set_content(self, window: int, child: int) -> tuple[int, int]:
        self.calls.append(("window_set_content", (window, child)))
        if self._status("window_set_content")[0] == _native.OK:
            self.window_contents[window] = child
        return self._status("window_set_content")

    def window_clear_content(self, window: int) -> tuple[int, int]:
        self.calls.append(("window_clear_content", (window,)))
        if self._status("window_clear_content")[0] == _native.OK:
            self.window_contents.pop(window, None)
        return self._status("window_clear_content")

    def window_title_property(self, window: int) -> tuple[int, int]:
        return self._property("window_title", "utf8", b"")

    def container_insert(
        self, container: int, child: int, index: int
    ) -> tuple[int, int]:
        self.calls.append(("container_insert", (container, child, index)))
        return self._status("container_insert")

    def container_remove(self, container: int, child: int) -> tuple[int, int]:
        self.calls.append(("container_remove", (container, child)))
        return self._status("container_remove")

    def container_move(
        self, container: int, from_index: int, to_index: int
    ) -> tuple[int, int]:
        self.calls.append(
            ("container_move", (container, from_index, to_index))
        )
        return self._status("container_move")

    def container_child_count(self, container: int) -> tuple[int, int]:
        self.calls.append(("container_child_count", (container,)))
        return self._status("container_child_count")[0], 2

    def container_child_at(
        self, container: int, index: int
    ) -> tuple[int, int, int]:
        if "container_child_at" in self.control_output_results:
            return self.control_output_results["container_child_at"]
        return _native.OK, self._token(), 0

    def stack_panel_orientation(self, panel: int) -> tuple[int, int]:
        if "stack_panel_orientation" in self.status_results:
            return self.status_results["stack_panel_orientation"]
        return _native.OK, self.orientation

    def stack_panel_set_orientation(
        self, panel: int, orientation: int
    ) -> tuple[int, int]:
        self.calls.append(
            ("stack_panel_set_orientation", (panel, orientation))
        )
        result = self._status("stack_panel_set_orientation")
        if result[0] == _native.OK:
            self.orientation = orientation
        return result

    def stack_panel_set_child_align(
        self, panel: int, child: int, align: int
    ) -> tuple[int, int]:
        self.calls.append(
            ("stack_panel_set_child_align", (panel, child, align))
        )
        return self._status("stack_panel_set_child_align")

    def text_content_property(self, text: int) -> tuple[int, int]:
        return self._property("text_content", "utf8", b"")

    def text_box_value_property(self, text_box: int) -> tuple[int, int]:
        return self._property("text_box_value", "utf8", b"")

    def button_label_property(self, button: int) -> tuple[int, int]:
        return self._property("button_label", "utf8", b"")

    def button_set_command(self, button: int, command: int) -> tuple[int, int]:
        self.calls.append(("button_set_command", (button, command)))
        result = self._status("button_set_command")
        if result[0] == _native.OK:
            self.button_commands[button] = command
        return result

    def button_command(self, button: int) -> tuple[int, int, int]:
        if "button_command" in self.control_output_results:
            return self.control_output_results["button_command"]
        command = self.button_commands.get(button, 0)
        return _native.OK, self._token() if command else 0, 0


def test_control_factories_and_properties_map_to_typed_native_owners() -> None:
    api = FakeControlsApi()
    runtime, _ = make_runtime(api)
    window = Window.create(runtime)
    panel = StackPanel.create(runtime)
    text = Text.create(runtime, "値")
    text_box = TextBox.create(runtime, "入力")
    button = Button.create(runtime, "実行")
    content = text.content
    value = text_box.value
    label = button.label
    title = window.title
    is_enabled = button.is_enabled
    accessible_name = button.accessible_name

    assert isinstance(panel, Container)
    assert not isinstance(window, Container)
    assert content.get() == ""
    assert value.get() == ""
    assert label.get() == ""
    assert title.get() == ""
    assert is_enabled.get() is True
    assert accessible_name.get() == ""
    assert [name for name, _ in api.calls if name.endswith("_create")] == [
        "window_create",
        "stack_panel_create",
        "text_create",
        "text_box_create",
        "button_create",
    ]
    assert [
        args[1]
        for name, args in api.calls
        if name in {"text_create", "text_box_create", "button_create"}
    ] == ["値".encode(), "入力".encode(), "実行".encode()]

    for prop in (content, value, label, title, is_enabled, accessible_name):
        prop.close()
    for owner in (button, text_box, text, panel, window):
        owner.close()
    runtime.close()


@pytest.mark.parametrize(
    ("factory", "kind"),
    [
        (Window.create, "window"),
        (StackPanel.create, "stack_panel"),
        (lambda runtime: Text.create(runtime, "x"), "text"),
        (lambda runtime: TextBox.create(runtime, "x"), "text_box"),
        (lambda runtime: Button.create(runtime, "x"), "button"),
    ],
)
def test_factory_releases_unexpected_owned_output(
    factory: Callable[[object], Widget], kind: str
) -> None:
    api = FakeControlsApi()
    runtime, _ = make_runtime(api)
    api.control_create_results[kind] = (_native.CLOSED, 777, 0)

    with pytest.raises(UiError) as raised:
        factory(runtime)

    assert raised.value.raw_status == _native.CLOSED
    assert api.releases == [777]
    runtime.close()


def test_utf8_factories_reject_non_str_and_surrogates_before_native_call() -> (
    None
):
    api = FakeControlsApi()
    runtime, _ = make_runtime(api)
    calls_before = list(api.calls)

    with pytest.raises(TypeError):
        Text.create(runtime, 3)  # type: ignore[arg-type]
    with pytest.raises(UnicodeEncodeError):
        TextBox.create(runtime, "\ud800")

    assert api.calls == calls_before
    runtime.close()


def test_window_content_is_an_owned_type_erased_widget_view() -> None:
    api = FakeControlsApi()
    runtime, _ = make_runtime(api)
    window = Window.create(runtime)
    text = Text.create(runtime, "title")

    assert window.content is None
    window.set_content(text)
    view = window.content
    assert type(view) is Widget
    assert view is not None
    view.close()
    window.clear_content()
    assert window.content is None

    window.close()
    text.close()
    runtime.close()


def test_panel_tree_ops_preserve_the_child_owner() -> None:
    api = FakeControlsApi()
    runtime, _ = make_runtime(api)
    panel = StackPanel.create(runtime)
    text = Text.create(runtime)

    assert panel.child_count == 2
    child = panel.child_at(0)
    assert type(child) is Widget
    child.close()
    panel.insert(text, 1)
    panel.remove(text)
    panel.move(0, 1)
    assert text._cell.token != 0
    assert [
        call[0] for call in api.calls if call[0].startswith("container_")
    ] == [
        "container_child_count",
        "container_insert",
        "container_remove",
        "container_move",
    ]

    before = len(api.calls)
    with pytest.raises(ValueError):
        panel.insert(text, -1)
    assert len(api.calls) == before
    panel.close()
    text.close()
    runtime.close()


def test_tree_rejects_other_runtime_before_native_call() -> None:
    api = FakeControlsApi()
    runtime, _ = make_runtime(api)
    other_runtime, _ = make_runtime(api)
    panel = StackPanel.create(runtime)
    text = Text.create(other_runtime)
    before = len(api.calls)

    with pytest.raises(UiError) as raised:
        panel.insert(text, 0)

    assert raised.value.raw_status == _native.WRONG_RUNTIME
    assert len(api.calls) == before
    panel.close()
    text.close()
    other_runtime.close()
    runtime.close()


def test_stack_panel_orientation_alignment_and_unknown_enum_mapping() -> None:
    api = FakeControlsApi()
    runtime, _ = make_runtime(api)
    panel = StackPanel.create(runtime)
    child = Button.create(runtime)

    assert panel.orientation is Orientation.VERTICAL
    panel.orientation = Orientation.HORIZONTAL
    panel.set_child_align(child, Alignment.CENTER)
    assert (
        "stack_panel_set_child_align",
        (panel._cell.token, child._cell.token, int(Alignment.CENTER)),
    ) in api.calls
    with pytest.raises(TypeError):
        panel.orientation = 2  # type: ignore[assignment]
    with pytest.raises(TypeError):
        panel.set_child_align(child, 2)  # type: ignore[arg-type]
    api.orientation = 99
    with pytest.raises(UiError) as raised:
        _ = panel.orientation
    assert raised.value.raw_status == _native.INTERNAL_ERROR

    child.close()
    panel.close()
    runtime.close()


def test_button_command_set_get_clear_and_runtime_validation() -> None:
    api = FakeControlsApi()
    runtime, _ = make_runtime(api)
    other_runtime, _ = make_runtime(api)
    button = Button.create(runtime)
    command = AsyncCommand.create(runtime, lambda _operation: None)
    other = AsyncCommand.create(other_runtime, lambda _operation: None)

    button.set_command(command)
    view = button.command
    assert isinstance(view, AsyncCommand)
    assert view._cell.token != command._cell.token
    button.set_command(None)
    assert button.command is None

    before = len(api.calls)
    with pytest.raises(UiError) as raised:
        button.set_command(other)
    assert raised.value.raw_status == _native.WRONG_RUNTIME
    assert len(api.calls) == before

    view.close()
    command.close()
    other.close()
    button.close()
    other_runtime.close()
    runtime.close()


def test_control_error_mapping_focus_and_close_retry() -> None:
    api = FakeControlsApi()
    runtime, _ = make_runtime(api)
    button = Button.create(runtime)
    api.status_results["widget_request_focus"] = (_native.INVALID_ARGUMENT, 0)

    with pytest.raises(UiError) as focus_error:
        button.request_focus()
    assert focus_error.value.raw_status == _native.INVALID_ARGUMENT

    api.release_statuses.append(_native.CLOSED)
    with pytest.raises(UiError) as close_error:
        button.close()
    assert close_error.value.raw_status == _native.CLOSED
    assert button._cell.token != 0
    button.close()
    runtime.close()


def test_child_getter_releases_unexpected_handle_on_failure() -> None:
    api = FakeControlsApi()
    runtime, _ = make_runtime(api)
    panel = StackPanel.create(runtime)
    api.control_output_results["container_child_at"] = (
        _native.INVALID_ARGUMENT,
        778,
        0,
    )

    with pytest.raises(UiError) as raised:
        panel.child_at(4)

    assert raised.value.raw_status == _native.INVALID_ARGUMENT
    assert api.releases == [778]
    panel.close()
    runtime.close()

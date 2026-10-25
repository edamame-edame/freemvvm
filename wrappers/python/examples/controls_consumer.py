"""Exercise Python UI Controls against the built host C ABI library."""

from __future__ import annotations

import ctypes
import sys

import freemvvm
from freemvvm import (
    Alignment,
    AsyncCommand,
    Binding,
    BindingMode,
    Button,
    Orientation,
    Runtime,
    StackPanel,
    Text,
    TextBox,
    Window,
)
from freemvvm import _native


def _check(status: int, operation: str) -> None:
    if status != _native.OK:
        raise RuntimeError(f"{operation} failed with C ABI status {status}")


def _dispatch_event(
    library: ctypes.CDLL, runtime: int, action: object
) -> None:
    scope = ctypes.c_uint64()
    _check(
        library.ui_abi_1_runtime_host_begin_event(
            runtime, ctypes.byref(scope)
        ),
        "begin host event",
    )
    try:
        action()  # type: ignore[operator]
    finally:
        _check(
            library.ui_abi_1_runtime_host_end_event(runtime, scope.value),
            "end host event",
        )


def main() -> None:
    library = ctypes.CDLL(sys.argv[1])
    wake_type = ctypes.CFUNCTYPE(None, ctypes.c_void_p, ctypes.c_uint32)
    destroy_type = ctypes.CFUNCTYPE(None, ctypes.c_void_p)
    library.ui_abi_1_runtime_set_ui_loop_bridge.argtypes = [
        _native.Handle,
        wake_type,
        ctypes.c_void_p,
        destroy_type,
        ctypes.POINTER(_native.Handle),
    ]
    library.ui_abi_1_runtime_set_ui_loop_bridge.restype = _native.Status
    library.ui_abi_1_runtime_host_begin_event.argtypes = [
        _native.Handle,
        ctypes.POINTER(ctypes.c_uint64),
    ]
    library.ui_abi_1_runtime_host_begin_event.restype = _native.Status
    library.ui_abi_1_runtime_host_end_event.argtypes = [
        _native.Handle,
        ctypes.c_uint64,
    ]
    library.ui_abi_1_runtime_host_end_event.restype = _native.Status
    freemvvm._load_native_api = lambda: _native.CtypesNativeApi(library)
    runtime = Runtime.create()
    wake = wake_type(lambda _userdata, _priority: None)
    no_destroy = ctypes.cast(ctypes.c_void_p(), destroy_type)
    bridge_error = _native.Handle()
    _check(
        library.ui_abi_1_runtime_set_ui_loop_bridge(
            runtime._cell.token,
            wake,
            None,
            no_destroy,
            ctypes.byref(bridge_error),
        ),
        "register UI loop bridge",
    )
    if bridge_error.value:
        runtime._cell.api.handle_release(bridge_error.value)
        raise RuntimeError("UI loop bridge registration returned an error")
    window = Window.create(runtime)
    panel = StackPanel.create(runtime)
    title = Text.create(runtime, "FreeMvvm")
    entry = TextBox.create(runtime, "入力")
    button = Button.create(runtime, "実行")
    command = AsyncCommand.create(runtime, lambda _operation: None)
    try:
        assert panel.orientation is Orientation.VERTICAL
        panel.orientation = Orientation.HORIZONTAL
        panel.insert(title, 0)
        panel.insert(entry, 1)
        panel.insert(button, 2)
        assert panel.child_count == 3
        panel.set_child_align(button, Alignment.CENTER)
        window.set_content(panel)
        content_view = window.content
        assert isinstance(content_view, freemvvm.Widget)
        content_view.close()
        first_child = panel.child_at(0)
        assert isinstance(first_child, freemvvm.Widget)
        first_child.close()
        panel.move(2, 1)
        assert panel.child_count == 3
        panel.remove(entry)
        assert panel.child_count == 2
        panel.insert(entry, 1)
        assert panel.child_count == 3

        window_title = window.title
        title_content = title.content
        entry_value = entry.value
        button_label = button.label
        enabled = button.is_enabled
        accessible_name = button.accessible_name
        binding = Binding.create(
            title_content, entry_value, BindingMode.ONE_WAY
        )
        assert binding is not None
        try:
            _dispatch_event(
                library,
                runtime._cell.token,
                lambda: (
                    window_title.set("サンプル"),
                    title_content.set("FreeMvvm controls"),
                ),
            )
            button_label.set("実行する")
            enabled.set(True)
            accessible_name.set("実行ボタン")
            assert window.title.get() == "サンプル"
            assert title.content.get() == "FreeMvvm controls"
            assert entry.value.get() == "FreeMvvm controls"
            assert button.label.get() == "実行する"

            button.set_command(command)
            command_view = button.command
            assert isinstance(command_view, AsyncCommand)
            command_view.close()
            button.set_command(None)
            assert button.command is None
        finally:
            binding.close()
            accessible_name.close()
            enabled.close()
            button_label.close()
            entry_value.close()
            title_content.close()
            window_title.close()

        window.clear_content()
        assert window.content is None
        # This ABI currently has no host-window activation path; only verify
        # the documented focus error mapping, not successful focus/rendering.
        focus_rejected = False
        try:
            button.request_focus()
        except freemvvm.UiError:
            focus_rejected = True
        assert focus_rejected

    finally:
        # Release the Button's strong Command reference before closing the
        # command.  The Control owners are then released child-first.
        for owner in (button, entry, title, panel, window):
            try:
                owner.close()
            except freemvvm.UiError:
                pass
        try:
            command.close()
        except freemvvm.UiError:
            pass
        try:
            runtime.close()
        except freemvvm.UiError:
            pass
    print("Python Controls C ABI consumer: OK")


if __name__ == "__main__":
    main()

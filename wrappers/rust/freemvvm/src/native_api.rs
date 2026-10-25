use std::sync::Arc;

pub(crate) const OK: u32 = freemvvm_sys::OK;
pub(crate) const INVALID_ARGUMENT: u32 = freemvvm_sys::INVALID_ARGUMENT;
pub(crate) const WRONG_RUNTIME: u32 = freemvvm_sys::WRONG_RUNTIME;
pub(crate) const WRONG_THREAD: u32 = freemvvm_sys::WRONG_THREAD;
pub(crate) const CLOSED: u32 = freemvvm_sys::CLOSED;
#[cfg(test)]
pub(crate) const BUFFER_TOO_SMALL: u32 = freemvvm_sys::BUFFER_TOO_SMALL;
pub(crate) const OUT_OF_MEMORY: u32 = freemvvm_sys::OUT_OF_MEMORY;
pub(crate) const INTERNAL_ERROR: u32 = freemvvm_sys::INTERNAL_ERROR;
pub(crate) const UNSUPPORTED: u32 = freemvvm_sys::UNSUPPORTED;
pub(crate) const PENDING: u32 = freemvvm_sys::PENDING;

#[derive(Clone, Copy)]
pub(crate) enum UiControlKind {
    Window,
    StackPanel,
    TextBox,
    Text,
    Button,
}

#[derive(Clone, Copy)]
pub(crate) enum UiPropertyKind {
    IsEnabled,
    AccessibleName,
    WindowTitle,
    TextContent,
    TextBoxValue,
    ButtonLabel,
}

#[derive(Clone, Copy)]
pub(crate) enum UiAction {
    RequestFocus,
    WindowOpen,
    WindowClose,
}

#[derive(Clone, Copy)]
pub(crate) enum UiTreeAction {
    WindowContent,
    WindowSetContent,
    WindowClearContent,
    Insert,
    Remove,
    Move,
    ChildCount,
    ChildAt,
}

#[derive(Clone, Copy)]
pub(crate) enum UiStackPanelAction {
    Orientation,
    SetOrientation,
    SetChildAlign,
}

#[derive(Clone, Copy)]
pub(crate) enum UiButtonAction {
    SetCommand,
    Command,
}

pub(crate) trait NativeApi: Send + Sync {
    fn get_version(&self) -> (u32, u32, u32, u32);
    fn runtime_create(&self) -> (u32, u64, u64);
    fn runtime_is_ui_thread(&self, runtime: u64) -> (u32, u8);
    fn runtime_shutdown(&self, runtime: u64) -> (u32, u64);
    fn runtime_request_shutdown(&self, runtime: u64, mode: u32) -> (u32, u64, u64);
    fn shutdown_request_query(&self, request: u64) -> (u32, u32, u32, u64);
    fn handle_retain(&self, borrowed: u64) -> (u32, u64);
    fn handle_release(&self, owned: u64) -> u32;
    fn error_status(&self, error: u64) -> (u32, u32);
    fn error_code(&self, error: u64) -> (u32, u32, u32);
    fn error_message_utf8(&self, error: u64, buffer: Option<&mut [u8]>) -> (u32, u64);

    fn ui_create(&self, _kind: UiControlKind, _runtime: u64, _bytes: &[u8]) -> (u32, u64, u64) {
        (UNSUPPORTED, 0, 0)
    }

    fn ui_property(&self, _kind: UiPropertyKind, _control: u64) -> (u32, u64) {
        (UNSUPPORTED, 0)
    }

    fn ui_action(&self, _action: UiAction, _control: u64) -> (u32, u64) {
        (UNSUPPORTED, 0)
    }

    fn ui_tree(
        &self,
        _action: UiTreeAction,
        _parent: u64,
        _child: u64,
        _index: u64,
        _to_index: u64,
    ) -> (u32, u64, u64) {
        (UNSUPPORTED, 0, 0)
    }

    fn ui_stack_panel(
        &self,
        _action: UiStackPanelAction,
        _panel: u64,
        _child: u64,
        _value: u32,
    ) -> (u32, u64, u64) {
        (UNSUPPORTED, 0, 0)
    }

    fn ui_button(&self, _action: UiButtonAction, _button: u64, _command: u64) -> (u32, u64, u64) {
        (UNSUPPORTED, 0, 0)
    }
}

pub(crate) type SharedApi = Arc<dyn NativeApi>;

pub(crate) struct LinkedApi;

impl NativeApi for LinkedApi {
    fn get_version(&self) -> (u32, u32, u32, u32) {
        let mut major = 0;
        let mut minor = 0;
        let mut patch = 0;
        // SAFETY: all out-pointers refer to initialized local storage for this call.
        let status =
            unsafe { freemvvm_sys::ui_abi_1_get_version(&mut major, &mut minor, &mut patch) };
        (status, major, minor, patch)
    }

    fn runtime_create(&self) -> (u32, u64, u64) {
        let mut runtime = 0;
        let mut error = 0;
        // SAFETY: the ABI writes only to the two valid out-pointers.
        let status = unsafe { freemvvm_sys::ui_abi_1_runtime_create(&mut runtime, &mut error) };
        (status, runtime, error)
    }

    fn runtime_is_ui_thread(&self, runtime: u64) -> (u32, u8) {
        let mut yes = 0;
        // SAFETY: `yes` is valid out-storage and `runtime` is borrowed by this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_runtime_is_ui_thread(runtime, &mut yes) };
        (status, yes)
    }

    fn runtime_shutdown(&self, runtime: u64) -> (u32, u64) {
        let mut error = 0;
        // SAFETY: `error` is valid out-storage and `runtime` is borrowed by this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_runtime_shutdown(runtime, &mut error) };
        (status, error)
    }

    fn runtime_request_shutdown(&self, runtime: u64, mode: u32) -> (u32, u64, u64) {
        let mut request = 0;
        let mut error = 0;
        // SAFETY: both output pointers are valid for the duration of the ABI call.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_runtime_request_shutdown(runtime, mode, &mut request, &mut error)
        };
        (status, request, error)
    }

    fn shutdown_request_query(&self, request: u64) -> (u32, u32, u32, u64) {
        let mut state = 0;
        let mut reason = 0;
        let mut error = 0;
        // SAFETY: all output pointers are valid for the duration of the ABI call.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_shutdown_request_query(
                request,
                &mut state,
                &mut reason,
                &mut error,
            )
        };
        (status, state, reason, error)
    }

    fn handle_retain(&self, borrowed: u64) -> (u32, u64) {
        let mut owned = 0;
        // SAFETY: `owned` is valid out-storage and the input is borrowed for this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_handle_retain(borrowed, &mut owned) };
        (status, owned)
    }

    fn handle_release(&self, owned: u64) -> u32 {
        // SAFETY: callers transfer one live owned token to this function.
        unsafe { freemvvm_sys::ui_abi_1_handle_release(owned) }
    }

    fn error_status(&self, error: u64) -> (u32, u32) {
        let mut status_value = 0;
        // SAFETY: `status_value` is valid out-storage and the error is borrowed for this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_error_status(error, &mut status_value) };
        (status, status_value)
    }

    fn error_code(&self, error: u64) -> (u32, u32, u32) {
        let mut domain = 0;
        let mut code = 0;
        // SAFETY: both outputs are valid and the error is borrowed for this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_error_code(error, &mut domain, &mut code) };
        (status, domain, code)
    }

    fn error_message_utf8(&self, error: u64, buffer: Option<&mut [u8]>) -> (u32, u64) {
        let mut length = 0;
        let (pointer, capacity) = match buffer {
            Some(bytes) => (bytes.as_mut_ptr(), bytes.len() as u64),
            None => (std::ptr::null_mut(), 0),
        };
        // SAFETY: a provided slice supplies writable storage of `capacity` bytes; None uses
        // the ABI's documented null/zero-capacity query form.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_error_message_utf8(error, pointer, capacity, &mut length)
        };
        (status, length)
    }

    fn ui_create(&self, kind: UiControlKind, runtime: u64, bytes: &[u8]) -> (u32, u64, u64) {
        let mut control = 0;
        let mut error = 0;
        let status = match kind {
            UiControlKind::Window => {
                // SAFETY: both outputs are valid for this call; runtime is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_window_create(runtime, &mut control, &mut error) }
            }
            UiControlKind::StackPanel => {
                // SAFETY: both outputs are valid for this call; runtime is borrowed.
                unsafe {
                    freemvvm_sys::ui_abi_1_stack_panel_create(runtime, &mut control, &mut error)
                }
            }
            UiControlKind::TextBox => {
                // SAFETY: the byte slice remains valid for the synchronous ABI call.
                unsafe {
                    freemvvm_sys::ui_abi_1_text_box_create(
                        runtime,
                        bytes.as_ptr(),
                        bytes.len() as u64,
                        &mut control,
                        &mut error,
                    )
                }
            }
            UiControlKind::Text => {
                // SAFETY: the byte slice remains valid for the synchronous ABI call.
                unsafe {
                    freemvvm_sys::ui_abi_1_text_create(
                        runtime,
                        bytes.as_ptr(),
                        bytes.len() as u64,
                        &mut control,
                        &mut error,
                    )
                }
            }
            UiControlKind::Button => {
                // SAFETY: the byte slice remains valid for the synchronous ABI call.
                unsafe {
                    freemvvm_sys::ui_abi_1_button_create(
                        runtime,
                        bytes.as_ptr(),
                        bytes.len() as u64,
                        &mut control,
                        &mut error,
                    )
                }
            }
        };
        (status, control, error)
    }

    fn ui_property(&self, kind: UiPropertyKind, control: u64) -> (u32, u64) {
        let mut property = 0;
        let status = match kind {
            UiPropertyKind::IsEnabled => {
                // SAFETY: property is valid output storage; control is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_widget_is_enabled_property(control, &mut property) }
            }
            UiPropertyKind::AccessibleName => {
                // SAFETY: property is valid output storage; control is borrowed.
                unsafe {
                    freemvvm_sys::ui_abi_1_widget_accessible_name_property(control, &mut property)
                }
            }
            UiPropertyKind::WindowTitle => {
                // SAFETY: property is valid output storage; control is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_window_title_property(control, &mut property) }
            }
            UiPropertyKind::TextContent => {
                // SAFETY: property is valid output storage; control is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_text_content_property(control, &mut property) }
            }
            UiPropertyKind::TextBoxValue => {
                // SAFETY: property is valid output storage; control is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_text_box_value_property(control, &mut property) }
            }
            UiPropertyKind::ButtonLabel => {
                // SAFETY: property is valid output storage; control is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_button_label_property(control, &mut property) }
            }
        };
        (status, property)
    }

    fn ui_action(&self, action: UiAction, control: u64) -> (u32, u64) {
        let mut error = 0;
        let status = match action {
            UiAction::RequestFocus => {
                // SAFETY: error is valid output storage; control is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_widget_request_focus(control, &mut error) }
            }
            UiAction::WindowOpen => {
                // SAFETY: error is valid output storage; Window is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_window_open(control, &mut error) }
            }
            UiAction::WindowClose => {
                // SAFETY: error is valid output storage; Window is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_window_close(control, &mut error) }
            }
        };
        (status, error)
    }

    fn ui_tree(
        &self,
        action: UiTreeAction,
        parent: u64,
        child: u64,
        index: u64,
        to_index: u64,
    ) -> (u32, u64, u64) {
        let mut value = 0;
        let mut error = 0;
        let status = match action {
            UiTreeAction::WindowContent => {
                // SAFETY: both outputs are valid; Window is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_window_content(parent, &mut value, &mut error) }
            }
            UiTreeAction::WindowSetContent => {
                // SAFETY: error is valid output storage; Window and child are borrowed.
                unsafe { freemvvm_sys::ui_abi_1_window_set_content(parent, child, &mut error) }
            }
            UiTreeAction::WindowClearContent => {
                // SAFETY: error is valid output storage; Window is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_window_clear_content(parent, &mut error) }
            }
            UiTreeAction::Insert => {
                // SAFETY: error is valid output storage; parent and child are borrowed.
                unsafe { freemvvm_sys::ui_abi_1_container_insert(parent, child, index, &mut error) }
            }
            UiTreeAction::Remove => {
                // SAFETY: error is valid output storage; parent and child are borrowed.
                unsafe { freemvvm_sys::ui_abi_1_container_remove(parent, child, &mut error) }
            }
            UiTreeAction::Move => {
                // SAFETY: error is valid output storage; parent is borrowed.
                unsafe {
                    freemvvm_sys::ui_abi_1_container_move(parent, index, to_index, &mut error)
                }
            }
            UiTreeAction::ChildCount => {
                // SAFETY: value is valid output storage; parent is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_container_child_count(parent, &mut value) }
            }
            UiTreeAction::ChildAt => {
                // SAFETY: both outputs are valid; parent is borrowed.
                unsafe {
                    freemvvm_sys::ui_abi_1_container_child_at(parent, index, &mut value, &mut error)
                }
            }
        };
        (status, value, error)
    }

    fn ui_stack_panel(
        &self,
        action: UiStackPanelAction,
        panel: u64,
        child: u64,
        value: u32,
    ) -> (u32, u64, u64) {
        let mut result = 0;
        let mut error = 0;
        let status = match action {
            UiStackPanelAction::Orientation => {
                // SAFETY: result is valid output storage; panel is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_stack_panel_orientation(panel, &mut result) }
            }
            UiStackPanelAction::SetOrientation => {
                // SAFETY: error is valid output storage; panel is borrowed.
                unsafe {
                    freemvvm_sys::ui_abi_1_stack_panel_set_orientation(panel, value, &mut error)
                }
            }
            UiStackPanelAction::SetChildAlign => {
                // SAFETY: error is valid output storage; panel and child are borrowed.
                unsafe {
                    freemvvm_sys::ui_abi_1_stack_panel_set_child_align(
                        panel, child, value, &mut error,
                    )
                }
            }
        };
        (status, u64::from(result), error)
    }

    fn ui_button(&self, action: UiButtonAction, button: u64, command: u64) -> (u32, u64, u64) {
        let mut value = 0;
        let mut error = 0;
        let status = match action {
            UiButtonAction::SetCommand => {
                // SAFETY: error is valid output storage; Button and optional Command are borrowed.
                unsafe { freemvvm_sys::ui_abi_1_button_set_command(button, command, &mut error) }
            }
            UiButtonAction::Command => {
                // SAFETY: both outputs are valid; Button is borrowed.
                unsafe { freemvvm_sys::ui_abi_1_button_command(button, &mut value, &mut error) }
            }
        };
        (status, value, error)
    }
}

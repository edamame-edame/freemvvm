//! Raw FFI boundary for the shared dynamic native runtime.
//!
//! This crate deliberately has no dependency on the internal Core crate. The
//! declarations mirror the public versioned C ABI header.

pub type Handle = u64;
pub type Status = u32;
pub type Bool = u8;
pub type WindowCloseCallback = unsafe extern "C" fn(*mut core::ffi::c_void, Handle);
pub type DestroyCallback = unsafe extern "C" fn(*mut core::ffi::c_void);

pub const OK: Status = 0;
pub const INVALID_ARGUMENT: Status = 1;
pub const INVALID_HANDLE: Status = 2;
pub const WRONG_TYPE: Status = 3;
pub const WRONG_RUNTIME: Status = 4;
pub const WRONG_THREAD: Status = 5;
pub const CLOSED: Status = 6;
pub const BUFFER_TOO_SMALL: Status = 7;
pub const OUT_OF_MEMORY: Status = 8;
pub const INTERNAL_ERROR: Status = 9;
pub const UNSUPPORTED: Status = 10;
pub const COMMAND_EXECUTION_FAILED: Status = 12;
pub const PENDING: Status = 14;

pub const COMMAND_STATE_IDLE: i64 = 0;
pub const COMMAND_STATE_RUNNING: i64 = 1;
pub const COMMAND_STATE_CANCELLING: i64 = 2;
pub const COMMAND_STATE_SUCCEEDED: i64 = 3;
pub const COMMAND_STATE_FAILED: i64 = 4;
pub const COMMAND_STATE_CANCELLED: i64 = 5;

pub const SHUTDOWN_ASYNC: u32 = 0;
pub const SHUTDOWN_SYNC: u32 = 1;
pub const SHUTDOWN_REQUEST_PENDING: u32 = 0;
pub const SHUTDOWN_REQUEST_COMPLETED: u32 = 1;
pub const SHUTDOWN_REQUEST_HOST_UNAVAILABLE: u32 = 2;

pub const UI_ORIENTATION_VERTICAL: u32 = 1;
pub const UI_ORIENTATION_HORIZONTAL: u32 = 2;
pub const UI_ALIGN_START: u32 = 1;
pub const UI_ALIGN_CENTER: u32 = 2;
pub const UI_ALIGN_END: u32 = 3;
pub const UI_ALIGN_STRETCH: u32 = 4;

#[cfg_attr(target_os = "windows", link(name = "freemvvm_abi_1.dll"))]
#[cfg_attr(not(target_os = "windows"), link(name = "freemvvm_abi_1"))]
unsafe extern "C" {
    pub fn ui_abi_1_get_version(
        out_major: *mut u32,
        out_minor: *mut u32,
        out_patch: *mut u32,
    ) -> Status;

    pub fn ui_abi_1_runtime_create(out_runtime: *mut Handle, out_error: *mut Handle) -> Status;
    pub fn ui_abi_1_runtime_is_ui_thread(runtime: Handle, out_yes: *mut Bool) -> Status;
    pub fn ui_abi_1_runtime_set_ui_loop_bridge(
        runtime: Handle,
        callback: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32)>,
        userdata: *mut core::ffi::c_void,
        destroy: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_runtime_host_begin_event(runtime: Handle, out_scope_id: *mut u64) -> Status;
    pub fn ui_abi_1_runtime_host_end_event(runtime: Handle, scope_id: u64) -> Status;
    pub fn ui_abi_1_runtime_dispatch_turn(runtime: Handle) -> Status;
    pub fn ui_abi_1_property_create_bool(
        runtime: Handle,
        initial: Bool,
        out_property: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_create_i64(
        runtime: Handle,
        initial: i64,
        out_property: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_create_utf8(
        runtime: Handle,
        bytes: *const u8,
        byte_length: u64,
        out_property: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_create_f64(
        runtime: Handle,
        initial: f64,
        out_property: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_create_decimal(
        runtime: Handle,
        initial_bits: u64,
        out_property: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_get_bool(property: Handle, out_value: *mut Bool) -> Status;
    pub fn ui_abi_1_property_get_i64(property: Handle, out_value: *mut i64) -> Status;
    pub fn ui_abi_1_property_get_utf8(
        property: Handle,
        buffer: *mut u8,
        capacity: u64,
        out_length: *mut u64,
    ) -> Status;
    pub fn ui_abi_1_property_get_f64(property: Handle, out_value: *mut f64) -> Status;
    pub fn ui_abi_1_property_get_decimal(property: Handle, out_bits: *mut u64) -> Status;
    pub fn ui_abi_1_property_set_bool(
        property: Handle,
        value: Bool,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_set_i64(
        property: Handle,
        value: i64,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_set_utf8(
        property: Handle,
        bytes: *const u8,
        byte_length: u64,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_set_f64(
        property: Handle,
        value: f64,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_set_decimal(
        property: Handle,
        value_bits: u64,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_subscribe(
        property: Handle,
        callback: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
        userdata: *mut core::ffi::c_void,
        destroy: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
        out_subscription: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_subscription_cancel(subscription: Handle, out_error: *mut Handle) -> Status;
    pub fn ui_abi_1_property_validator_register_i64(
        property: Handle,
        callback: Option<unsafe extern "C" fn(*mut core::ffi::c_void, i64, *mut Handle) -> Status>,
        userdata: *mut core::ffi::c_void,
        destroy: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
        out_registration: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_property_validator_cancel(
        registration: Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_binding_create(
        source: Handle,
        target: Handle,
        mode: u32,
        out_binding: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_binding_cancel(binding: Handle, out_error: *mut Handle) -> Status;
    pub fn ui_abi_1_binding_error_snapshot(
        binding: Handle,
        out_owned_error: *mut Handle,
        out_owned_rejected_property: *mut Handle,
        out_direction: *mut u32,
    ) -> Status;
    pub fn ui_abi_1_binding_subscribe_error(
        binding: Handle,
        callback: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
        userdata: *mut core::ffi::c_void,
        destroy: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
        out_subscription: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_command_create_async(
        runtime: Handle,
        eligibility_bool_or_zero: Handle,
        start: Option<unsafe extern "C" fn(*mut core::ffi::c_void, Handle, u64, Handle) -> Status>,
        userdata: *mut core::ffi::c_void,
        destroy: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
        out_command: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_command_execute(
        command: Handle,
        out_operation_id: *mut u64,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_command_cancel(
        command: Handle,
        operation_id: u64,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_command_close(command: Handle, out_error: *mut Handle) -> Status;
    pub fn ui_abi_1_command_state_property(
        command: Handle,
        out_owned_property: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_command_can_execute_property(
        command: Handle,
        out_owned_property: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_command_failure_snapshot(
        command: Handle,
        out_owned_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_command_cancel_token_requested(token: Handle, out_yes: *mut Bool) -> Status;
    pub fn ui_abi_1_command_complete_success(command: Handle, operation_id: u64) -> Status;
    pub fn ui_abi_1_command_complete_failure_utf8(
        command: Handle,
        operation_id: u64,
        domain: u32,
        code: u32,
        message: *const u8,
        message_byte_length: u64,
    ) -> Status;
    pub fn ui_abi_1_command_complete_cancelled(command: Handle, operation_id: u64) -> Status;
    pub fn ui_abi_1_runtime_post(
        runtime: Handle,
        callback: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
        userdata: *mut core::ffi::c_void,
        destroy: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_runtime_shutdown(runtime: Handle, out_error: *mut Handle) -> Status;
    pub fn ui_abi_1_runtime_request_shutdown(
        runtime: Handle,
        mode: u32,
        out_request: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_shutdown_request_query(
        request: Handle,
        out_state: *mut u32,
        out_reason: *mut u32,
        out_error: *mut Handle,
    ) -> Status;

    pub fn ui_abi_1_window_create(
        runtime: Handle,
        out_window: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_stack_panel_create(
        runtime: Handle,
        out_panel: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_text_box_create(
        runtime: Handle,
        bytes: *const u8,
        byte_length: u64,
        out_text_box: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_text_create(
        runtime: Handle,
        bytes: *const u8,
        byte_length: u64,
        out_text: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_button_create(
        runtime: Handle,
        bytes: *const u8,
        byte_length: u64,
        out_button: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_widget_is_enabled_property(widget: Handle, out_property: *mut Handle)
    -> Status;
    pub fn ui_abi_1_widget_accessible_name_property(
        widget: Handle,
        out_property: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_widget_request_focus(widget: Handle, out_error: *mut Handle) -> Status;
    pub fn ui_abi_1_window_content(
        window: Handle,
        out_child: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_window_set_content(
        window: Handle,
        borrowed_child: Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_window_clear_content(window: Handle, out_error: *mut Handle) -> Status;
    pub fn ui_abi_1_window_title_property(window: Handle, out_property: *mut Handle) -> Status;
    pub fn ui_abi_1_window_open(window: Handle, out_error: *mut Handle) -> Status;
    pub fn ui_abi_1_window_close(window: Handle, out_error: *mut Handle) -> Status;
    pub fn ui_abi_1_window_close_request_subscribe(
        window: Handle,
        callback: Option<WindowCloseCallback>,
        userdata: *mut core::ffi::c_void,
        destroy: Option<DestroyCallback>,
        out_subscription: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_window_close_request_defer(
        borrowed_request: Handle,
        out_owned_request: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_window_close_request_resolve(
        request: Handle,
        decision: u32,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_container_insert(
        container: Handle,
        borrowed_child: Handle,
        index: u64,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_container_remove(
        container: Handle,
        borrowed_child: Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_container_move(
        container: Handle,
        from_index: u64,
        to_index: u64,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_container_child_count(container: Handle, out_count: *mut u64) -> Status;
    pub fn ui_abi_1_container_child_at(
        container: Handle,
        index: u64,
        out_child: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_stack_panel_orientation(panel: Handle, out_value: *mut u32) -> Status;
    pub fn ui_abi_1_stack_panel_set_orientation(
        panel: Handle,
        orientation: u32,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_stack_panel_set_child_align(
        panel: Handle,
        borrowed_child: Handle,
        align: u32,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_text_content_property(text: Handle, out_property: *mut Handle) -> Status;
    pub fn ui_abi_1_text_box_value_property(text_box: Handle, out_property: *mut Handle) -> Status;
    pub fn ui_abi_1_button_label_property(button: Handle, out_property: *mut Handle) -> Status;
    pub fn ui_abi_1_button_set_command(
        button: Handle,
        borrowed_command_or_zero: Handle,
        out_error: *mut Handle,
    ) -> Status;
    pub fn ui_abi_1_button_command(
        button: Handle,
        out_command: *mut Handle,
        out_error: *mut Handle,
    ) -> Status;

    pub fn ui_abi_1_handle_retain(borrowed: Handle, out_owned: *mut Handle) -> Status;
    pub fn ui_abi_1_handle_release(owned: Handle) -> Status;
    pub fn ui_abi_1_error_status(error: Handle, out_status: *mut Status) -> Status;
    pub fn ui_abi_1_error_code(error: Handle, out_domain: *mut u32, out_code: *mut u32) -> Status;
    pub fn ui_abi_1_error_message_utf8(
        error: Handle,
        buffer: *mut u8,
        capacity: u64,
        out_length: *mut u64,
    ) -> Status;
}

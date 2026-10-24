//! Raw FFI boundary for the shared FreeMVVM C ABI (`ui_abi_1_*`).
//!
//! This crate deliberately has no dependency on the internal Core crate.

#![allow(non_camel_case_types)]

use std::ffi::c_void;

/// Opaque 64-bit handle token (`0` is always invalid).
pub type ui_handle = u64;
/// 32-bit status code returned by all C ABI entry points.
pub type ui_status = u32;
/// 8-bit boolean (`0` = false, `1` = true).
pub type ui_bool = u8;

pub const UI_OK: ui_status = 0;
pub const UI_INVALID_HANDLE: ui_status = 1;
pub const UI_INVALID_ARGUMENT: ui_status = 2;
pub const UI_WRONG_TYPE: ui_status = 3;
pub const UI_WRONG_RUNTIME: ui_status = 4;
pub const UI_WRONG_THREAD: ui_status = 5;
pub const UI_CLOSED: ui_status = 6;
pub const UI_REENTRANT_CALL: ui_status = 7;
pub const UI_BUFFER_TOO_SMALL: ui_status = 8;
pub const UI_OUT_OF_MEMORY: ui_status = 9;
pub const UI_PENDING: ui_status = 10;
pub const UI_VALIDATION_FAILED: ui_status = 11;
pub const UI_NOT_CONVERTIBLE: ui_status = 12;
pub const UI_PARSE_FAILED: ui_status = 13;
pub const UI_OVERFLOW: ui_status = 14;
pub const UI_DIVIDE_BY_ZERO: ui_status = 15;
pub const UI_NON_FINITE: ui_status = 16;
pub const UI_INTERNAL_ERROR: ui_status = 255;

pub const UI_WEBVIEW_FORMAT_HTML: u32 = 1;
pub const UI_WEBVIEW_FORMAT_MARKDOWN: u32 = 2;

/// Zero-argument callback invoked on the UI thread.
pub type ui_post_callback = Option<unsafe extern "C" fn(userdata: *mut c_void)>;
/// Finalizer callback invoked when native closure ownership is released.
pub type ui_destroy_callback = Option<unsafe extern "C" fn(userdata: *mut c_void)>;

/// Function-pointer table for the versioned `WebView` C ABI entry points.
///
/// Allows the safe `freemvvm` wrapper crate to operate against a dynamically
/// loaded `freemvvm_abi_1` shared library or a deterministic test double without
/// relinking the internal Rust core crate.
#[derive(Clone, Copy)]
pub struct WebViewAbiVtable {
    pub handle_retain:
        unsafe extern "C" fn(handle: ui_handle, out_handle: *mut ui_handle) -> ui_status,
    pub handle_release: unsafe extern "C" fn(handle: ui_handle) -> ui_status,
    pub error_status:
        unsafe extern "C" fn(error_handle: ui_handle, out_status: *mut u32) -> ui_status,
    pub error_code: unsafe extern "C" fn(
        error_handle: ui_handle,
        out_domain: *mut u32,
        out_code: *mut u32,
    ) -> ui_status,
    pub control_create_webview: unsafe extern "C" fn(
        runtime: ui_handle,
        out_webview: *mut ui_handle,
        out_error: *mut ui_handle,
    ) -> ui_status,
    pub webview_set_source_utf8: unsafe extern "C" fn(
        webview: ui_handle,
        format: u32,
        utf8_bytes: *const u8,
        byte_len: u64,
        out_error: *mut ui_handle,
    ) -> ui_status,
    pub webview_set_stylesheet_utf8: unsafe extern "C" fn(
        webview: ui_handle,
        utf8_bytes: *const u8,
        byte_len: u64,
        out_error: *mut ui_handle,
    ) -> ui_status,
    pub webview_set_viewport: unsafe extern "C" fn(
        webview: ui_handle,
        width_dip: f64,
        height_dip: f64,
        out_error: *mut ui_handle,
    ) -> ui_status,
    pub webview_bind_source_property: unsafe extern "C" fn(
        webview: ui_handle,
        format: u32,
        utf8_property: ui_handle,
        out_binding: *mut ui_handle,
        out_error: *mut ui_handle,
    ) -> ui_status,
    pub webview_expose_property_utf8: unsafe extern "C" fn(
        webview: ui_handle,
        alias_utf8: *const u8,
        alias_len: u64,
        property: ui_handle,
        allow_write: ui_bool,
        out_error: *mut ui_handle,
    ) -> ui_status,
    pub webview_expose_command_utf8: unsafe extern "C" fn(
        webview: ui_handle,
        alias_utf8: *const u8,
        alias_len: u64,
        callback: ui_post_callback,
        userdata: *mut c_void,
        destroy: ui_destroy_callback,
        out_error: *mut ui_handle,
    ) -> ui_status,
    pub webview_eval_script_utf8: unsafe extern "C" fn(
        webview: ui_handle,
        script_utf8: *const u8,
        script_len: u64,
        out_error: *mut ui_handle,
    ) -> ui_status,
    pub webview_click_at: unsafe extern "C" fn(
        webview: ui_handle,
        x_dip: f64,
        y_dip: f64,
        out_error: *mut ui_handle,
    ) -> ui_status,
    pub webview_text_content_utf8: unsafe extern "C" fn(
        webview: ui_handle,
        buffer: *mut u8,
        capacity: u64,
        out_byte_len: *mut u64,
        out_error: *mut ui_handle,
    ) -> ui_status,
    pub webview_render_rgba: unsafe extern "C" fn(
        webview: ui_handle,
        width_px: u32,
        height_px: u32,
        rgba_buffer: *mut u8,
        capacity: u64,
        out_byte_len: *mut u64,
        out_error: *mut ui_handle,
    ) -> ui_status,
}

unsafe extern "C" {
    pub fn ui_abi_1_handle_retain(handle: ui_handle, out_handle: *mut ui_handle) -> ui_status;
    pub fn ui_abi_1_handle_release(handle: ui_handle) -> ui_status;
    pub fn ui_abi_1_error_status(error_handle: ui_handle, out_status: *mut u32) -> ui_status;
    pub fn ui_abi_1_error_code(
        error_handle: ui_handle,
        out_domain: *mut u32,
        out_code: *mut u32,
    ) -> ui_status;
    pub fn ui_abi_1_control_create_webview(
        runtime: ui_handle,
        out_webview: *mut ui_handle,
        out_error: *mut ui_handle,
    ) -> ui_status;
    pub fn ui_abi_1_webview_set_source_utf8(
        webview: ui_handle,
        format: u32,
        utf8_bytes: *const u8,
        byte_len: u64,
        out_error: *mut ui_handle,
    ) -> ui_status;
    pub fn ui_abi_1_webview_set_stylesheet_utf8(
        webview: ui_handle,
        utf8_bytes: *const u8,
        byte_len: u64,
        out_error: *mut ui_handle,
    ) -> ui_status;
    pub fn ui_abi_1_webview_set_viewport(
        webview: ui_handle,
        width_dip: f64,
        height_dip: f64,
        out_error: *mut ui_handle,
    ) -> ui_status;
    pub fn ui_abi_1_webview_bind_source_property(
        webview: ui_handle,
        format: u32,
        utf8_property: ui_handle,
        out_binding: *mut ui_handle,
        out_error: *mut ui_handle,
    ) -> ui_status;
    pub fn ui_abi_1_webview_expose_property_utf8(
        webview: ui_handle,
        alias_utf8: *const u8,
        alias_len: u64,
        property: ui_handle,
        allow_write: ui_bool,
        out_error: *mut ui_handle,
    ) -> ui_status;
    pub fn ui_abi_1_webview_expose_command_utf8(
        webview: ui_handle,
        alias_utf8: *const u8,
        alias_len: u64,
        callback: ui_post_callback,
        userdata: *mut c_void,
        destroy: ui_destroy_callback,
        out_error: *mut ui_handle,
    ) -> ui_status;
    pub fn ui_abi_1_webview_eval_script_utf8(
        webview: ui_handle,
        script_utf8: *const u8,
        script_len: u64,
        out_error: *mut ui_handle,
    ) -> ui_status;
    pub fn ui_abi_1_webview_click_at(
        webview: ui_handle,
        x_dip: f64,
        y_dip: f64,
        out_error: *mut ui_handle,
    ) -> ui_status;
    pub fn ui_abi_1_webview_text_content_utf8(
        webview: ui_handle,
        buffer: *mut u8,
        capacity: u64,
        out_byte_len: *mut u64,
        out_error: *mut ui_handle,
    ) -> ui_status;
    pub fn ui_abi_1_webview_render_rgba(
        webview: ui_handle,
        width_px: u32,
        height_px: u32,
        rgba_buffer: *mut u8,
        capacity: u64,
        out_byte_len: *mut u64,
        out_error: *mut ui_handle,
    ) -> ui_status;
}

//! Safe Rust facade over `freemvvm-sys` (`UNIT-WEBVIEW-012`).
//!
//! Provides move-only owned `WebView` and `OwnedBinding` handles that enforce
//! UI-thread affinity (`!Send` and `!Sync`), panic-safe callback trampolines,
//! and preallocated cleanup tickets so `Drop` never panics or loses a native
//! token.

use freemvvm_sys::{
    UI_BUFFER_TOO_SMALL, UI_CLOSED, UI_INTERNAL_ERROR, UI_INVALID_ARGUMENT, UI_OK,
    UI_WEBVIEW_FORMAT_HTML, UI_WEBVIEW_FORMAT_MARKDOWN, WebViewAbiVtable, ui_handle, ui_status,
};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::sync::{Arc, Mutex};

/// Document format supported by `WebView`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum WebViewFormat {
    Html = UI_WEBVIEW_FORMAT_HTML,
    Markdown = UI_WEBVIEW_FORMAT_MARKDOWN,
}

/// Immutable diagnostic error snapshot returned across the C ABI boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiError {
    raw_status: ui_status,
    domain: Option<u32>,
    code: Option<u32>,
}

impl UiError {
    pub const fn new(raw_status: ui_status, domain: Option<u32>, code: Option<u32>) -> Self {
        Self {
            raw_status,
            domain,
            code,
        }
    }

    pub const fn raw_status(&self) -> ui_status {
        self.raw_status
    }

    pub const fn domain(&self) -> Option<u32> {
        self.domain
    }

    pub const fn code(&self) -> Option<u32> {
        self.code
    }
}

/// Preallocated cleanup state shared by handles belonging to a runtime.
#[derive(Default)]
pub struct CleanupRegistry {
    pending_tokens: Mutex<Vec<ui_handle>>,
}

impl CleanupRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            pending_tokens: Mutex::new(Vec::new()),
        })
    }

    /// Retries releasing any tokens that failed during `Drop`.
    pub fn drain_pending(&self, vtable: &WebViewAbiVtable) -> usize {
        let mut guard = match self.pending_tokens.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut remaining = Vec::new();
        let mut released = 0usize;
        for token in guard.drain(..) {
            // SAFETY: token is a deferred owned handle registered by Drop.
            let status = unsafe { (vtable.handle_release)(token) };
            if status == UI_OK {
                released += 1;
            } else {
                remaining.push(token);
            }
        }
        *guard = remaining;
        released
    }

    pub fn pending_count(&self) -> usize {
        match self.pending_tokens.lock() {
            Ok(g) => g.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }
}

fn make_ui_error(vtable: &WebViewAbiVtable, status: ui_status, error_handle: ui_handle) -> UiError {
    if error_handle == 0 {
        return UiError::new(status, None, None);
    }
    let mut domain = 0u32;
    let mut code = 0u32;
    // SAFETY: error_handle is an owned error snapshot returned by the C ABI.
    let code_status = unsafe { (vtable.error_code)(error_handle, &mut domain, &mut code) };
    // SAFETY: error_handle is released immediately after extracting numeric diagnostics.
    let _ = unsafe { (vtable.handle_release)(error_handle) };
    if code_status == UI_OK {
        UiError::new(status, Some(domain), Some(code))
    } else {
        UiError::new(status, None, None)
    }
}

/// Move-only owned binding handle returned by [`WebView::bind_source_property`].
pub struct OwnedBinding {
    token: ui_handle,
    vtable: Arc<WebViewAbiVtable>,
    cleanup: Arc<CleanupRegistry>,
    preallocated_slot: Vec<ui_handle>,
    _ui_affine: PhantomData<*const ()>,
}

impl OwnedBinding {
    pub fn is_valid(&self) -> bool {
        self.token != 0
    }

    pub fn try_close(&mut self) -> Result<(), UiError> {
        if self.token == 0 {
            return Ok(());
        }
        // SAFETY: self.token is an owned C ABI handle.
        let status = unsafe { (self.vtable.handle_release)(self.token) };
        if status == UI_OK {
            self.token = 0;
            Ok(())
        } else {
            Err(UiError::new(status, None, None))
        }
    }
}

impl Drop for OwnedBinding {
    fn drop(&mut self) {
        if self.token == 0 {
            return;
        }
        let token = self.token;
        self.token = 0;
        // SAFETY: token is owned by this handle and released at most once.
        let status = unsafe { (self.vtable.handle_release)(token) };
        if status != UI_OK {
            self.preallocated_slot.push(token);
            let mut guard = match self.cleanup.pending_tokens.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            guard.append(&mut self.preallocated_slot);
        }
    }
}

struct CommandBox {
    callback: Box<dyn FnMut()>,
}

unsafe extern "C" fn command_invoke_trampoline(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    // SAFETY: userdata points to a CommandBox allocated by expose_command and invoked serially on the UI thread.
    let cmd = unsafe { &mut *(userdata.cast::<CommandBox>()) };
    let _ = catch_unwind(AssertUnwindSafe(|| {
        (cmd.callback)();
    }));
}

unsafe extern "C" fn command_destroy_trampoline(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: userdata was allocated by Box::into_raw in expose_command and is destroyed exactly once.
        let _ = unsafe { Box::from_raw(userdata.cast::<CommandBox>()) };
    }));
}

/// Safe UI-affine `WebView` control handle (`!Send` and `!Sync`).
pub struct WebView {
    token: ui_handle,
    vtable: Arc<WebViewAbiVtable>,
    cleanup: Arc<CleanupRegistry>,
    preallocated_slot: Vec<ui_handle>,
    _ui_affine: PhantomData<*const ()>,
}

impl WebView {
    /// Creates a new `WebView` attached to `runtime_token`.
    pub fn create(
        runtime_token: ui_handle,
        vtable: Arc<WebViewAbiVtable>,
        cleanup: Arc<CleanupRegistry>,
    ) -> Result<Self, UiError> {
        if runtime_token == 0 {
            return Err(UiError::new(UI_INVALID_ARGUMENT, None, None));
        }
        let preallocated_slot = Vec::with_capacity(1);
        let mut out_webview: ui_handle = 0;
        let mut out_error: ui_handle = 0;
        // SAFETY: pointers are valid stack locals.
        let status = unsafe {
            (vtable.control_create_webview)(runtime_token, &mut out_webview, &mut out_error)
        };
        if status != UI_OK {
            return Err(make_ui_error(&vtable, status, out_error));
        }
        if out_webview == 0 {
            return Err(UiError::new(UI_INTERNAL_ERROR, None, None));
        }
        Ok(Self {
            token: out_webview,
            vtable,
            cleanup,
            preallocated_slot,
            _ui_affine: PhantomData,
        })
    }

    /// Acquires an independent retained `WebView` handle.
    pub fn try_retain(&self) -> Result<Self, UiError> {
        if self.token == 0 {
            return Err(UiError::new(UI_CLOSED, None, None));
        }
        let preallocated_slot = Vec::with_capacity(1);
        let mut out_handle: ui_handle = 0;
        // SAFETY: self.token is a valid owned handle and out_handle is a stack local.
        let status = unsafe { (self.vtable.handle_retain)(self.token, &mut out_handle) };
        if status != UI_OK {
            return Err(UiError::new(status, None, None));
        }
        Ok(Self {
            token: out_handle,
            vtable: Arc::clone(&self.vtable),
            cleanup: Arc::clone(&self.cleanup),
            preallocated_slot,
            _ui_affine: PhantomData,
        })
    }

    /// Explicitly releases the native `WebView` handle.
    pub fn try_close(&mut self) -> Result<(), UiError> {
        if self.token == 0 {
            return Ok(());
        }
        // SAFETY: self.token is a valid owned handle.
        let status = unsafe { (self.vtable.handle_release)(self.token) };
        if status == UI_OK {
            self.token = 0;
            Ok(())
        } else {
            Err(UiError::new(status, None, None))
        }
    }

    pub fn set_source(&self, format: WebViewFormat, source: &str) -> Result<(), UiError> {
        if self.token == 0 {
            return Err(UiError::new(UI_CLOSED, None, None));
        }
        let mut out_error: ui_handle = 0;
        let ptr = if source.is_empty() {
            ptr::null()
        } else {
            source.as_ptr()
        };
        // SAFETY: ptr is valid for source.len() bytes during the synchronous call.
        let status = unsafe {
            (self.vtable.webview_set_source_utf8)(
                self.token,
                format as u32,
                ptr,
                source.len() as u64,
                &mut out_error,
            )
        };
        if status == UI_OK {
            Ok(())
        } else {
            Err(make_ui_error(&self.vtable, status, out_error))
        }
    }

    pub fn set_stylesheet(&self, css: &str) -> Result<(), UiError> {
        if self.token == 0 {
            return Err(UiError::new(UI_CLOSED, None, None));
        }
        let mut out_error: ui_handle = 0;
        let ptr = if css.is_empty() {
            ptr::null()
        } else {
            css.as_ptr()
        };
        // SAFETY: ptr is valid for css.len() bytes during the synchronous call.
        let status = unsafe {
            (self.vtable.webview_set_stylesheet_utf8)(
                self.token,
                ptr,
                css.len() as u64,
                &mut out_error,
            )
        };
        if status == UI_OK {
            Ok(())
        } else {
            Err(make_ui_error(&self.vtable, status, out_error))
        }
    }

    pub fn set_viewport(&self, width_dip: f64, height_dip: f64) -> Result<(), UiError> {
        if self.token == 0 {
            return Err(UiError::new(UI_CLOSED, None, None));
        }
        let mut out_error: ui_handle = 0;
        // SAFETY: self.token is valid and out_error is a stack local.
        let status = unsafe {
            (self.vtable.webview_set_viewport)(self.token, width_dip, height_dip, &mut out_error)
        };
        if status == UI_OK {
            Ok(())
        } else {
            Err(make_ui_error(&self.vtable, status, out_error))
        }
    }

    pub fn bind_source_property(
        &self,
        format: WebViewFormat,
        utf8_property_token: ui_handle,
    ) -> Result<OwnedBinding, UiError> {
        if self.token == 0 {
            return Err(UiError::new(UI_CLOSED, None, None));
        }
        let preallocated_slot = Vec::with_capacity(1);
        let mut out_binding: ui_handle = 0;
        let mut out_error: ui_handle = 0;
        // SAFETY: handles and stack out pointers are valid.
        let status = unsafe {
            (self.vtable.webview_bind_source_property)(
                self.token,
                format as u32,
                utf8_property_token,
                &mut out_binding,
                &mut out_error,
            )
        };
        if status != UI_OK {
            return Err(make_ui_error(&self.vtable, status, out_error));
        }
        if out_binding == 0 {
            return Err(UiError::new(UI_INTERNAL_ERROR, None, None));
        }
        Ok(OwnedBinding {
            token: out_binding,
            vtable: Arc::clone(&self.vtable),
            cleanup: Arc::clone(&self.cleanup),
            preallocated_slot,
            _ui_affine: PhantomData,
        })
    }

    pub fn expose_property(
        &self,
        alias: &str,
        property_token: ui_handle,
        allow_write: bool,
    ) -> Result<(), UiError> {
        if self.token == 0 {
            return Err(UiError::new(UI_CLOSED, None, None));
        }
        let mut out_error: ui_handle = 0;
        let ptr = if alias.is_empty() {
            ptr::null()
        } else {
            alias.as_ptr()
        };
        // SAFETY: alias bytes are valid for the synchronous C ABI call.
        let status = unsafe {
            (self.vtable.webview_expose_property_utf8)(
                self.token,
                ptr,
                alias.len() as u64,
                property_token,
                u8::from(allow_write),
                &mut out_error,
            )
        };
        if status == UI_OK {
            Ok(())
        } else {
            Err(make_ui_error(&self.vtable, status, out_error))
        }
    }

    pub fn expose_command<F>(&self, alias: &str, callback: F) -> Result<(), UiError>
    where
        F: FnMut() + 'static,
    {
        if self.token == 0 {
            return Err(UiError::new(UI_CLOSED, None, None));
        }
        let boxed = Box::new(CommandBox {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(boxed).cast::<c_void>();
        let mut out_error: ui_handle = 0;
        let ptr = if alias.is_empty() {
            ptr::null()
        } else {
            alias.as_ptr()
        };
        // SAFETY: userdata ownership transfers to the C ABI only on UI_OK.
        let status = unsafe {
            (self.vtable.webview_expose_command_utf8)(
                self.token,
                ptr,
                alias.len() as u64,
                Some(command_invoke_trampoline),
                userdata,
                Some(command_destroy_trampoline),
                &mut out_error,
            )
        };
        if status == UI_OK {
            Ok(())
        } else {
            // SAFETY: registration failed, so Rust retains ownership and drops userdata immediately.
            let _ = unsafe { Box::from_raw(userdata.cast::<CommandBox>()) };
            Err(make_ui_error(&self.vtable, status, out_error))
        }
    }

    pub fn eval_script(&self, script: &str) -> Result<(), UiError> {
        if self.token == 0 {
            return Err(UiError::new(UI_CLOSED, None, None));
        }
        let mut out_error: ui_handle = 0;
        let ptr = if script.is_empty() {
            ptr::null()
        } else {
            script.as_ptr()
        };
        // SAFETY: script bytes are valid for the synchronous C ABI call.
        let status = unsafe {
            (self.vtable.webview_eval_script_utf8)(
                self.token,
                ptr,
                script.len() as u64,
                &mut out_error,
            )
        };
        if status == UI_OK {
            Ok(())
        } else {
            Err(make_ui_error(&self.vtable, status, out_error))
        }
    }

    pub fn click_at(&self, x_dip: f64, y_dip: f64) -> Result<(), UiError> {
        if self.token == 0 {
            return Err(UiError::new(UI_CLOSED, None, None));
        }
        let mut out_error: ui_handle = 0;
        // SAFETY: self.token is valid and out_error is a stack local.
        let status =
            unsafe { (self.vtable.webview_click_at)(self.token, x_dip, y_dip, &mut out_error) };
        if status == UI_OK {
            Ok(())
        } else {
            Err(make_ui_error(&self.vtable, status, out_error))
        }
    }

    pub fn text_content(&self) -> Result<String, UiError> {
        if self.token == 0 {
            return Err(UiError::new(UI_CLOSED, None, None));
        }
        let mut required: u64 = 0;
        let mut out_error: ui_handle = 0;
        // SAFETY: null buffer query for required UTF-8 byte length.
        let status = unsafe {
            (self.vtable.webview_text_content_utf8)(
                self.token,
                ptr::null_mut(),
                0,
                &mut required,
                &mut out_error,
            )
        };
        if status != UI_OK {
            return Err(make_ui_error(&self.vtable, status, out_error));
        }
        if required == 0 {
            return Ok(String::new());
        }
        let cap =
            usize::try_from(required).map_err(|_| UiError::new(UI_INVALID_ARGUMENT, None, None))?;
        let mut buf = vec![0u8; cap];
        let mut written: u64 = 0;
        // SAFETY: buf has `cap` writable bytes.
        let copy_status = unsafe {
            (self.vtable.webview_text_content_utf8)(
                self.token,
                buf.as_mut_ptr(),
                required,
                &mut written,
                &mut out_error,
            )
        };
        if copy_status != UI_OK {
            return Err(make_ui_error(&self.vtable, copy_status, out_error));
        }
        let actual =
            usize::try_from(written).map_err(|_| UiError::new(UI_INTERNAL_ERROR, None, None))?;
        if actual > buf.len() {
            return Err(UiError::new(UI_BUFFER_TOO_SMALL, None, None));
        }
        buf.truncate(actual);
        String::from_utf8(buf).map_err(|_| UiError::new(UI_INTERNAL_ERROR, None, None))
    }

    pub fn render_rgba(&self, width_px: u32, height_px: u32) -> Result<Vec<u8>, UiError> {
        if self.token == 0 {
            return Err(UiError::new(UI_CLOSED, None, None));
        }
        let mut required: u64 = 0;
        let mut out_error: ui_handle = 0;
        // SAFETY: null buffer query for required RGBA8 byte length.
        let status = unsafe {
            (self.vtable.webview_render_rgba)(
                self.token,
                width_px,
                height_px,
                ptr::null_mut(),
                0,
                &mut required,
                &mut out_error,
            )
        };
        if status != UI_OK {
            return Err(make_ui_error(&self.vtable, status, out_error));
        }
        let cap =
            usize::try_from(required).map_err(|_| UiError::new(UI_INVALID_ARGUMENT, None, None))?;
        let mut buf = vec![0u8; cap];
        let mut written: u64 = 0;
        // SAFETY: buf has `cap` writable bytes.
        let copy_status = unsafe {
            (self.vtable.webview_render_rgba)(
                self.token,
                width_px,
                height_px,
                buf.as_mut_ptr(),
                required,
                &mut written,
                &mut out_error,
            )
        };
        if copy_status != UI_OK {
            return Err(make_ui_error(&self.vtable, copy_status, out_error));
        }
        let actual =
            usize::try_from(written).map_err(|_| UiError::new(UI_INTERNAL_ERROR, None, None))?;
        if actual > buf.len() {
            return Err(UiError::new(UI_BUFFER_TOO_SMALL, None, None));
        }
        buf.truncate(actual);
        Ok(buf)
    }
}

impl Drop for WebView {
    fn drop(&mut self) {
        if self.token == 0 {
            return;
        }
        let token = self.token;
        self.token = 0;
        // SAFETY: token is owned by this WebView and released at most once.
        let status = unsafe { (self.vtable.handle_release)(token) };
        if status != UI_OK {
            self.preallocated_slot.push(token);
            let mut guard = match self.cleanup.pending_tokens.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            guard.append(&mut self.preallocated_slot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemvvm_sys::{UI_VALIDATION_FAILED, ui_bool, ui_destroy_callback, ui_post_callback};
    use std::cell::RefCell;

    struct FakeCommand {
        callback: ui_post_callback,
        userdata: usize,
        destroy: ui_destroy_callback,
    }

    #[derive(Default)]
    struct FakeState {
        source: String,
        css: String,
        fail_next_release: bool,
        released_tokens: Vec<ui_handle>,
        commands: Vec<(String, FakeCommand)>,
    }

    thread_local! {
        static FAKE_STATE: RefCell<FakeState> = RefCell::new(FakeState::default());
    }

    unsafe extern "C" fn fake_retain(handle: ui_handle, out_handle: *mut ui_handle) -> ui_status {
        if handle == 0 || out_handle.is_null() {
            return UI_INVALID_ARGUMENT;
        }
        // SAFETY: out_handle is non-null.
        unsafe { *out_handle = handle + 1000 };
        UI_OK
    }

    unsafe extern "C" fn fake_release(handle: ui_handle) -> ui_status {
        FAKE_STATE.with(|s| {
            let mut st = s.borrow_mut();
            if st.fail_next_release {
                st.fail_next_release = false;
                return UI_INTERNAL_ERROR;
            }
            st.released_tokens.push(handle);
            if handle == 10 {
                let cmds = std::mem::take(&mut st.commands);
                for (_, cmd) in cmds {
                    if let Some(d) = cmd.destroy {
                        // SAFETY: test invokes the registered destroy callback once on release.
                        unsafe { d(cmd.userdata as *mut c_void) };
                    }
                }
            }
            UI_OK
        })
    }

    unsafe extern "C" fn fake_error_status(_err: ui_handle, out_status: *mut u32) -> ui_status {
        // SAFETY: out_status is valid in tests.
        unsafe { *out_status = UI_VALIDATION_FAILED };
        UI_OK
    }

    unsafe extern "C" fn fake_error_code(
        _err: ui_handle,
        out_domain: *mut u32,
        out_code: *mut u32,
    ) -> ui_status {
        // SAFETY: out pointers are valid in tests.
        unsafe {
            *out_domain = 17;
            *out_code = 8;
        }
        UI_OK
    }

    unsafe extern "C" fn fake_create(
        runtime: ui_handle,
        out_webview: *mut ui_handle,
        _out_error: *mut ui_handle,
    ) -> ui_status {
        if runtime == 0 {
            return UI_INVALID_ARGUMENT;
        }
        // SAFETY: out_webview is valid in tests.
        unsafe { *out_webview = 10 };
        UI_OK
    }

    unsafe extern "C" fn fake_set_source(
        _webview: ui_handle,
        _format: u32,
        utf8_bytes: *const u8,
        byte_len: u64,
        _out_error: *mut ui_handle,
    ) -> ui_status {
        // SAFETY: slice is valid for byte_len bytes.
        let bytes = unsafe { std::slice::from_raw_parts(utf8_bytes, byte_len as usize) };
        FAKE_STATE.with(|s| {
            s.borrow_mut().source = String::from_utf8_lossy(bytes).into_owned();
        });
        UI_OK
    }

    unsafe extern "C" fn fake_set_css(
        _webview: ui_handle,
        utf8_bytes: *const u8,
        byte_len: u64,
        _out_error: *mut ui_handle,
    ) -> ui_status {
        // SAFETY: slice is valid for byte_len bytes.
        let bytes = unsafe { std::slice::from_raw_parts(utf8_bytes, byte_len as usize) };
        FAKE_STATE.with(|s| {
            s.borrow_mut().css = String::from_utf8_lossy(bytes).into_owned();
        });
        UI_OK
    }

    unsafe extern "C" fn fake_set_viewport(
        _webview: ui_handle,
        width_dip: f64,
        height_dip: f64,
        _out_error: *mut ui_handle,
    ) -> ui_status {
        if width_dip <= 0.0 || height_dip <= 0.0 {
            return UI_INVALID_ARGUMENT;
        }
        UI_OK
    }

    unsafe extern "C" fn fake_bind(
        _webview: ui_handle,
        _format: u32,
        prop: ui_handle,
        out_binding: *mut ui_handle,
        _out_error: *mut ui_handle,
    ) -> ui_status {
        if prop == 0 {
            return UI_INVALID_ARGUMENT;
        }
        // SAFETY: out_binding is valid in tests.
        unsafe { *out_binding = 20 };
        UI_OK
    }

    unsafe extern "C" fn fake_expose_prop(
        _webview: ui_handle,
        _alias: *const u8,
        _len: u64,
        prop: ui_handle,
        _allow_write: ui_bool,
        _out_error: *mut ui_handle,
    ) -> ui_status {
        if prop == 0 {
            return UI_INVALID_ARGUMENT;
        }
        UI_OK
    }

    unsafe extern "C" fn fake_expose_cmd(
        _webview: ui_handle,
        alias: *const u8,
        len: u64,
        callback: ui_post_callback,
        userdata: *mut c_void,
        destroy: ui_destroy_callback,
        _out_error: *mut ui_handle,
    ) -> ui_status {
        // SAFETY: alias is valid for len bytes.
        let name = unsafe {
            String::from_utf8_lossy(std::slice::from_raw_parts(alias, len as usize)).into_owned()
        };
        if name == "reject" {
            return UI_INVALID_ARGUMENT;
        }
        FAKE_STATE.with(|s| {
            s.borrow_mut().commands.push((
                name,
                FakeCommand {
                    callback,
                    userdata: userdata as usize,
                    destroy,
                },
            ));
        });
        UI_OK
    }

    unsafe extern "C" fn fake_eval(
        _webview: ui_handle,
        script: *const u8,
        len: u64,
        out_error: *mut ui_handle,
    ) -> ui_status {
        // SAFETY: script is valid for len bytes.
        let code = unsafe {
            String::from_utf8_lossy(std::slice::from_raw_parts(script, len as usize)).into_owned()
        };
        if code.contains("fetch") {
            // SAFETY: out_error is valid in tests.
            unsafe { *out_error = 99 };
            return UI_VALIDATION_FAILED;
        }
        let cb_pair = FAKE_STATE.with(|s| {
            s.borrow()
                .commands
                .first()
                .map(|(_, c)| (c.callback, c.userdata))
        });
        if let Some((Some(cb), userdata)) = cb_pair {
            // SAFETY: invoking registered callback in test.
            unsafe { cb(userdata as *mut c_void) };
        }
        UI_OK
    }

    unsafe extern "C" fn fake_click(
        _webview: ui_handle,
        _x: f64,
        _y: f64,
        _out_error: *mut ui_handle,
    ) -> ui_status {
        UI_OK
    }

    unsafe extern "C" fn fake_text(
        _webview: ui_handle,
        buffer: *mut u8,
        capacity: u64,
        out_byte_len: *mut u64,
        _out_error: *mut ui_handle,
    ) -> ui_status {
        FAKE_STATE.with(|s| {
            let st = s.borrow();
            let bytes = st.source.as_bytes();
            // SAFETY: out_byte_len is valid in tests.
            unsafe { *out_byte_len = bytes.len() as u64 };
            if buffer.is_null() {
                return UI_OK;
            }
            if (capacity as usize) < bytes.len() {
                return UI_BUFFER_TOO_SMALL;
            }
            // SAFETY: buffer has capacity >= bytes.len().
            unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, bytes.len()) };
            UI_OK
        })
    }

    unsafe extern "C" fn fake_rgba(
        _webview: ui_handle,
        width_px: u32,
        height_px: u32,
        rgba_buffer: *mut u8,
        capacity: u64,
        out_byte_len: *mut u64,
        _out_error: *mut ui_handle,
    ) -> ui_status {
        let total = (width_px as u64) * (height_px as u64) * 4;
        // SAFETY: out_byte_len is valid in tests.
        unsafe { *out_byte_len = total };
        if rgba_buffer.is_null() {
            return UI_OK;
        }
        if capacity < total {
            return UI_BUFFER_TOO_SMALL;
        }
        // SAFETY: rgba_buffer has capacity >= total.
        unsafe { ptr::write_bytes(rgba_buffer, 0xFF, total as usize) };
        UI_OK
    }

    fn test_vtable() -> Arc<WebViewAbiVtable> {
        Arc::new(WebViewAbiVtable {
            handle_retain: fake_retain,
            handle_release: fake_release,
            error_status: fake_error_status,
            error_code: fake_error_code,
            control_create_webview: fake_create,
            webview_set_source_utf8: fake_set_source,
            webview_set_stylesheet_utf8: fake_set_css,
            webview_set_viewport: fake_set_viewport,
            webview_bind_source_property: fake_bind,
            webview_expose_property_utf8: fake_expose_prop,
            webview_expose_command_utf8: fake_expose_cmd,
            webview_eval_script_utf8: fake_eval,
            webview_click_at: fake_click,
            webview_text_content_utf8: fake_text,
            webview_render_rgba: fake_rgba,
        })
    }

    #[test]
    fn rust_webview_wrapper_manages_ownership_callbacks_and_deferred_cleanup() {
        FAKE_STATE.with(|s| *s.borrow_mut() = FakeState::default());
        let vtable = test_vtable();
        let cleanup = CleanupRegistry::new();

        let webview =
            WebView::create(1, Arc::clone(&vtable), Arc::clone(&cleanup)).expect("create");
        webview.set_viewport(320.0, 200.0).expect("viewport");
        webview
            .set_source(WebViewFormat::Markdown, "# Rust Wrapper")
            .expect("source");
        webview
            .set_stylesheet("h1 { color: #112233; }")
            .expect("css");
        assert_eq!(webview.text_content().expect("text"), "# Rust Wrapper");

        let mut binding = webview
            .bind_source_property(WebViewFormat::Markdown, 50)
            .expect("bind");
        assert!(binding.is_valid());
        binding.try_close().expect("close binding");

        let invoked = Arc::new(Mutex::new(0usize));
        let invoked_clone = Arc::clone(&invoked);
        webview
            .expose_command("save", move || {
                *invoked_clone.lock().expect("lock") += 1;
            })
            .expect("expose command");

        // Failed registration must immediately drop closure without leaking.
        let err = webview.expose_command("reject", || {});
        assert!(err.is_err());

        // Panic inside a user command callback must be caught at the C ABI trampoline boundary.
        webview
            .expose_command("panic_cmd", || {
                panic!("user callback panic must not unwind across FFI");
            })
            .expect("expose panic command");

        webview
            .eval_script("freemvvm.invokeCommand('save');")
            .expect("eval");
        assert_eq!(*invoked.lock().expect("lock"), 1);

        let sandbox_err = webview
            .eval_script("fetch('https://example.invalid');")
            .expect_err("reject fetch");
        assert_eq!(sandbox_err.raw_status(), UI_VALIDATION_FAILED);
        assert_eq!(sandbox_err.domain(), Some(17));
        assert_eq!(sandbox_err.code(), Some(8));

        let rgba = webview.render_rgba(16, 8).expect("rgba");
        assert_eq!(rgba.len(), 16 * 8 * 4);

        // Simulate a release failure during Drop and verify transfer to CleanupRegistry.
        FAKE_STATE.with(|s| s.borrow_mut().fail_next_release = true);
        drop(webview);
        assert_eq!(cleanup.pending_count(), 1);
        assert_eq!(cleanup.drain_pending(&vtable), 1);
        assert_eq!(cleanup.pending_count(), 0);
    }
}

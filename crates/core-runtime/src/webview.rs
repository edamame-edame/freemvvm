//! Runtime-owned `WebView` control state and MVVM bridge (`UNIT-WEBVIEW-009`).

use crate::abi_support::{
    BIND_ONE_WAY, BindingApiError, DestroyCallback, PostCallback, PropertyApiError,
    ShutdownApiError, binding_cancel, binding_create, handle_release, property_create_utf8,
    property_get_bool, property_get_f64, property_get_i64, property_get_utf8, property_kind,
    property_set_bool, property_set_f64, property_set_i64, property_set_utf8,
};
use crate::binding::BindingConnectionId;
use crate::property::ValidationError;
use crate::runtime::RuntimeCore;
use freemvvm_webview::{
    ColorRgba, DocumentFormat, JsValue, ScriptHostBridge, SoftwareRasterizer, WebViewEngine,
    WebViewError,
};
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Kind stored in the internal handle registry for a `WebView` control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WebViewHandleKind {
    WebView,
}

/// Error categories returned by `WebView` operations in `abi_support`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WebViewApiError {
    InvalidHandle,
    InvalidArgument,
    WrongType,
    WrongRuntime,
    WrongThread,
    Closed,
    OutOfMemory,
    BufferTooSmall { required: usize },
    ValidationFailed(Option<Arc<ValidationError>>),
    Internal,
}

const WEBVIEW_ERROR_DOMAIN: u32 = 17;

pub(crate) fn map_webview_engine_error(err: WebViewError) -> WebViewApiError {
    let msg = err.to_string();
    let code = match err {
        WebViewError::InvalidLayoutConstraint | WebViewError::InvalidNodeId(_) => {
            return WebViewApiError::InvalidArgument;
        }
        WebViewError::SyntaxError(_) => 1,
        WebViewError::SourceBytesLimitExceeded { .. }
        | WebViewError::DomNodeLimitExceeded { .. }
        | WebViewError::DomDepthLimitExceeded { .. }
        | WebViewError::AttributeLimitExceeded { .. }
        | WebViewError::CssLimitExceeded { .. } => 2,
        WebViewError::DisallowedUriScheme(_) => 3,
        WebViewError::ScriptStepLimitExceeded { .. } => 5,
        WebViewError::ScriptStackOverflow { .. } => 6,
        WebViewError::ScriptHeapLimitExceeded { .. } => 7,
        WebViewError::ScriptRuntimeError(_)
        | WebViewError::UnexposedProperty(_)
        | WebViewError::ReadOnlyProperty(_)
        | WebViewError::UnexposedCommand(_) => 8,
    };
    WebViewApiError::ValidationFailed(Some(Arc::new(ValidationError {
        status: 11,
        domain: WEBVIEW_ERROR_DOMAIN,
        code,
        message: msg.into_bytes(),
    })))
}

#[derive(Clone)]
pub(crate) struct ExposedPropertyEntry {
    pub(crate) property_handle: u64,
    pub(crate) allow_write: bool,
}

pub(crate) struct ExposedCommandClosure {
    callback: PostCallback,
    userdata: usize,
    destroy: DestroyCallback,
    armed: AtomicBool,
}

impl ExposedCommandClosure {
    pub(crate) fn new(callback: PostCallback, userdata: usize, destroy: DestroyCallback) -> Self {
        Self {
            callback,
            userdata,
            destroy,
            armed: AtomicBool::new(true),
        }
    }

    pub(crate) fn invoke(&self) {
        if self.armed.load(Ordering::Acquire) {
            // SAFETY: callback and userdata are valid while armed on the UI thread.
            unsafe { (self.callback)(self.userdata as *mut c_void) };
        }
    }
}

impl Drop for ExposedCommandClosure {
    fn drop(&mut self) {
        if self.armed.swap(false, Ordering::AcqRel) {
            // SAFETY: ownership was transferred upon registration and is destroyed once.
            unsafe { (self.destroy)(self.userdata as *mut c_void) };
        }
    }
}

pub(crate) struct BoundSourceState {
    pub(crate) format: DocumentFormat,
    pub(crate) source_property: u64,
    pub(crate) mirror_property: u64,
    pub(crate) connection_id: BindingConnectionId,
    pub(crate) last_synced: String,
}

impl Drop for BoundSourceState {
    fn drop(&mut self) {
        if self.mirror_property != 0 {
            let _ = handle_release(self.mirror_property);
            self.mirror_property = 0;
        }
    }
}

pub(crate) struct WebViewControlInner {
    pub(crate) engine: WebViewEngine,
    pub(crate) exposed_properties: HashMap<String, ExposedPropertyEntry>,
    pub(crate) exposed_commands: HashMap<String, Arc<ExposedCommandClosure>>,
    pub(crate) bound_source: Option<BoundSourceState>,
}

/// Shared instance stored in `RuntimeCore::ui_objects` for one `WebView`.
pub(crate) struct WebViewInstance {
    pub(crate) runtime: Arc<RuntimeCore>,
    pub(crate) active_call: AtomicBool,
    pub(crate) inner: Mutex<WebViewControlInner>,
}

struct ActiveCallGuard<'a>(&'a AtomicBool);

impl Drop for ActiveCallGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

struct RuntimeMvvmBridge<'a> {
    exposed_properties: &'a HashMap<String, ExposedPropertyEntry>,
    exposed_commands: &'a HashMap<String, Arc<ExposedCommandClosure>>,
}

fn js_value_to_f64(value: &JsValue) -> f64 {
    match value {
        JsValue::Number(n) => *n,
        JsValue::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        JsValue::Str(s) => s.trim().parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    }
}

impl ScriptHostBridge for RuntimeMvvmBridge<'_> {
    fn get_property(&self, name: &str) -> Result<JsValue, WebViewError> {
        let entry = self
            .exposed_properties
            .get(name)
            .ok_or_else(|| WebViewError::UnexposedProperty(name.to_string()))?;
        let kind = property_kind(entry.property_handle)
            .map_err(|_| WebViewError::ScriptRuntimeError("invalid property handle".to_string()))?;
        match kind {
            1 => {
                let v = property_get_bool(entry.property_handle).map_err(|_| {
                    WebViewError::ScriptRuntimeError("property read failed".to_string())
                })?;
                Ok(JsValue::Bool(v))
            }
            2 => {
                let v = property_get_i64(entry.property_handle).map_err(|_| {
                    WebViewError::ScriptRuntimeError("property read failed".to_string())
                })?;
                Ok(JsValue::Number(v as f64))
            }
            3 => {
                let s = read_utf8_property_string(entry.property_handle).map_err(|_| {
                    WebViewError::ScriptRuntimeError("property read failed".to_string())
                })?;
                Ok(JsValue::Str(s))
            }
            5 => {
                let v = property_get_f64(entry.property_handle).map_err(|_| {
                    WebViewError::ScriptRuntimeError("property read failed".to_string())
                })?;
                Ok(JsValue::Number(v))
            }
            _ => Err(WebViewError::ScriptRuntimeError(
                "unsupported property kind in script bridge".to_string(),
            )),
        }
    }

    fn set_property(&mut self, name: &str, value: JsValue) -> Result<(), WebViewError> {
        let entry = self
            .exposed_properties
            .get(name)
            .ok_or_else(|| WebViewError::UnexposedProperty(name.to_string()))?;
        if !entry.allow_write {
            return Err(WebViewError::ReadOnlyProperty(name.to_string()));
        }
        let kind = property_kind(entry.property_handle)
            .map_err(|_| WebViewError::ScriptRuntimeError("invalid property handle".to_string()))?;
        match kind {
            1 => property_set_bool(entry.property_handle, value.is_truthy()).map_err(|_| {
                WebViewError::ScriptRuntimeError("property write rejected".to_string())
            }),
            2 => property_set_i64(entry.property_handle, js_value_to_f64(&value) as i64).map_err(
                |_| WebViewError::ScriptRuntimeError("property write rejected".to_string()),
            ),
            3 => {
                let text = value.to_js_string();
                property_set_utf8(entry.property_handle, text.as_bytes()).map_err(|_| {
                    WebViewError::ScriptRuntimeError("property write rejected".to_string())
                })
            }
            5 => property_set_f64(entry.property_handle, js_value_to_f64(&value)).map_err(|_| {
                WebViewError::ScriptRuntimeError("property write rejected".to_string())
            }),
            _ => Err(WebViewError::ScriptRuntimeError(
                "unsupported property kind in script bridge".to_string(),
            )),
        }
    }

    fn invoke_command(&mut self, name: &str) -> Result<bool, WebViewError> {
        let cmd = self
            .exposed_commands
            .get(name)
            .cloned()
            .ok_or_else(|| WebViewError::UnexposedCommand(name.to_string()))?;
        cmd.invoke();
        Ok(true)
    }
}

pub(crate) fn read_utf8_property_string(property_handle: u64) -> Result<String, PropertyApiError> {
    let required = property_get_utf8(property_handle, None)?;
    let mut buf = vec![0u8; required];
    let written = property_get_utf8(property_handle, Some(&mut buf))?;
    buf.truncate(written);
    String::from_utf8(buf).map_err(|_| PropertyApiError::Internal)
}

impl WebViewInstance {
    pub(crate) fn new(runtime: Arc<RuntimeCore>) -> Arc<Self> {
        Arc::new(Self {
            runtime,
            active_call: AtomicBool::new(false),
            inner: Mutex::new(WebViewControlInner {
                engine: WebViewEngine::default(),
                exposed_properties: HashMap::new(),
                exposed_commands: HashMap::new(),
                bound_source: None,
            }),
        })
    }

    fn enter_call(&self) -> Result<ActiveCallGuard<'_>, WebViewApiError> {
        if !self.runtime.is_ui_thread() {
            return Err(WebViewApiError::WrongThread);
        }
        self.active_call
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| WebViewApiError::InvalidArgument)?;
        Ok(ActiveCallGuard(&self.active_call))
    }

    fn sync_bound_source_locked(
        &self,
        inner: &mut WebViewControlInner,
    ) -> Result<(), WebViewApiError> {
        let Some((connection_id, source_property, format, last_synced)) =
            inner.bound_source.as_ref().map(|b| {
                (
                    b.connection_id,
                    b.source_property,
                    b.format,
                    b.last_synced.clone(),
                )
            })
        else {
            return Ok(());
        };
        if !self.runtime.is_binding_active(connection_id) {
            return Ok(());
        }
        let current = read_utf8_property_string(source_property).map_err(|e| match e {
            PropertyApiError::WrongThread => WebViewApiError::WrongThread,
            PropertyApiError::Closed => WebViewApiError::Closed,
            _ => WebViewApiError::Internal,
        })?;
        if current != last_synced {
            if let Some(bound) = inner.bound_source.as_mut() {
                bound.last_synced = current.clone();
            }
            let mut bridge = RuntimeMvvmBridge {
                exposed_properties: &inner.exposed_properties,
                exposed_commands: &inner.exposed_commands,
            };
            inner
                .engine
                .set_source(format, &current, &mut bridge)
                .map_err(map_webview_engine_error)?;
        }
        Ok(())
    }

    pub(crate) fn set_source_utf8(&self, format: u32, bytes: &[u8]) -> Result<(), WebViewApiError> {
        let _guard = self.enter_call()?;
        let doc_format = match format {
            1 => DocumentFormat::Html,
            2 => DocumentFormat::Markdown,
            _ => return Err(WebViewApiError::InvalidArgument),
        };
        let source = std::str::from_utf8(bytes).map_err(|_| WebViewApiError::InvalidArgument)?;
        let mut inner = self.inner.lock().map_err(|_| WebViewApiError::Internal)?;
        let inner_ref = &mut *inner;
        let mut bridge = RuntimeMvvmBridge {
            exposed_properties: &inner_ref.exposed_properties,
            exposed_commands: &inner_ref.exposed_commands,
        };
        inner_ref
            .engine
            .set_source(doc_format, source, &mut bridge)
            .map_err(map_webview_engine_error)
    }

    pub(crate) fn set_stylesheet_utf8(&self, bytes: &[u8]) -> Result<(), WebViewApiError> {
        let _guard = self.enter_call()?;
        let css = std::str::from_utf8(bytes).map_err(|_| WebViewApiError::InvalidArgument)?;
        let mut inner = self.inner.lock().map_err(|_| WebViewApiError::Internal)?;
        inner
            .engine
            .set_stylesheet(css)
            .map_err(map_webview_engine_error)
    }

    pub(crate) fn set_viewport(
        &self,
        width_dip: f64,
        height_dip: f64,
    ) -> Result<(), WebViewApiError> {
        let _guard = self.enter_call()?;
        if !width_dip.is_finite()
            || !height_dip.is_finite()
            || width_dip <= 0.0
            || height_dip <= 0.0
        {
            return Err(WebViewApiError::InvalidArgument);
        }
        let mut inner = self.inner.lock().map_err(|_| WebViewApiError::Internal)?;
        inner
            .engine
            .set_viewport(width_dip, height_dip)
            .map(|_| ())
            .map_err(map_webview_engine_error)
    }

    pub(crate) fn expose_property(
        &self,
        alias_bytes: &[u8],
        property_handle: u64,
        allow_write: bool,
    ) -> Result<(), WebViewApiError> {
        let _guard = self.enter_call()?;
        let alias =
            std::str::from_utf8(alias_bytes).map_err(|_| WebViewApiError::InvalidArgument)?;
        if alias.is_empty() {
            return Err(WebViewApiError::InvalidArgument);
        }
        let mut inner = self.inner.lock().map_err(|_| WebViewApiError::Internal)?;
        inner.exposed_properties.insert(
            alias.to_string(),
            ExposedPropertyEntry {
                property_handle,
                allow_write,
            },
        );
        Ok(())
    }

    pub(crate) fn expose_command(
        &self,
        alias_bytes: &[u8],
        callback: PostCallback,
        userdata: usize,
        destroy: DestroyCallback,
    ) -> Result<(), WebViewApiError> {
        let _guard = self.enter_call()?;
        let alias =
            std::str::from_utf8(alias_bytes).map_err(|_| WebViewApiError::InvalidArgument)?;
        if alias.is_empty() {
            return Err(WebViewApiError::InvalidArgument);
        }
        let mut inner = self.inner.lock().map_err(|_| WebViewApiError::Internal)?;
        inner
            .exposed_commands
            .try_reserve(1)
            .map_err(|_| WebViewApiError::OutOfMemory)?;
        let closure = Arc::new(ExposedCommandClosure::new(callback, userdata, destroy));
        inner.exposed_commands.insert(alias.to_string(), closure);
        Ok(())
    }

    pub(crate) fn bind_source_property(
        &self,
        runtime_handle: u64,
        format: u32,
        utf8_property: u64,
        resolve_connection_id: impl FnOnce(u64) -> Option<BindingConnectionId>,
    ) -> Result<u64, WebViewApiError> {
        let _guard = self.enter_call()?;
        let doc_format = match format {
            1 => DocumentFormat::Html,
            2 => DocumentFormat::Markdown,
            _ => return Err(WebViewApiError::InvalidArgument),
        };
        let initial = read_utf8_property_string(utf8_property).map_err(|e| match e {
            PropertyApiError::InvalidHandle => WebViewApiError::InvalidHandle,
            PropertyApiError::WrongType => WebViewApiError::WrongType,
            PropertyApiError::WrongThread => WebViewApiError::WrongThread,
            PropertyApiError::Closed => WebViewApiError::Closed,
            _ => WebViewApiError::Internal,
        })?;
        let mirror_property =
            property_create_utf8(runtime_handle, initial.as_bytes()).map_err(|e| match e {
                ShutdownApiError::OutOfMemory => WebViewApiError::OutOfMemory,
                ShutdownApiError::WrongThread => WebViewApiError::WrongThread,
                ShutdownApiError::Closed => WebViewApiError::Closed,
                _ => WebViewApiError::Internal,
            })?;
        let binding_handle = match binding_create(utf8_property, mirror_property, BIND_ONE_WAY) {
            Ok(b) => b,
            Err(err) => {
                let _ = handle_release(mirror_property);
                return Err(match err {
                    BindingApiError::InvalidHandle => WebViewApiError::InvalidHandle,
                    BindingApiError::WrongType => WebViewApiError::WrongType,
                    BindingApiError::WrongRuntime => WebViewApiError::WrongRuntime,
                    BindingApiError::WrongThread => WebViewApiError::WrongThread,
                    BindingApiError::Closed => WebViewApiError::Closed,
                    BindingApiError::OutOfMemory => WebViewApiError::OutOfMemory,
                    _ => WebViewApiError::Internal,
                });
            }
        };
        let Some(connection_id) = resolve_connection_id(binding_handle) else {
            let _ = binding_cancel(binding_handle);
            let _ = handle_release(binding_handle);
            let _ = handle_release(mirror_property);
            return Err(WebViewApiError::Internal);
        };

        let mut inner = self.inner.lock().map_err(|_| WebViewApiError::Internal)?;
        let inner_ref = &mut *inner;
        let mut bridge = RuntimeMvvmBridge {
            exposed_properties: &inner_ref.exposed_properties,
            exposed_commands: &inner_ref.exposed_commands,
        };
        if let Err(err) = inner_ref
            .engine
            .set_source(doc_format, &initial, &mut bridge)
        {
            drop(inner);
            let _ = binding_cancel(binding_handle);
            let _ = handle_release(binding_handle);
            let _ = handle_release(mirror_property);
            return Err(map_webview_engine_error(err));
        }

        inner_ref.bound_source = Some(BoundSourceState {
            format: doc_format,
            source_property: utf8_property,
            mirror_property,
            connection_id,
            last_synced: initial,
        });
        Ok(binding_handle)
    }

    pub(crate) fn eval_script_utf8(&self, script_bytes: &[u8]) -> Result<(), WebViewApiError> {
        let _guard = self.enter_call()?;
        let script =
            std::str::from_utf8(script_bytes).map_err(|_| WebViewApiError::InvalidArgument)?;
        let mut inner = self.inner.lock().map_err(|_| WebViewApiError::Internal)?;
        self.sync_bound_source_locked(&mut inner)?;
        let inner_ref = &mut *inner;
        let mut bridge = RuntimeMvvmBridge {
            exposed_properties: &inner_ref.exposed_properties,
            exposed_commands: &inner_ref.exposed_commands,
        };
        inner_ref
            .engine
            .eval_script(script, &mut bridge)
            .map(|_| ())
            .map_err(map_webview_engine_error)
    }

    pub(crate) fn click_at(&self, x_dip: f64, y_dip: f64) -> Result<(), WebViewApiError> {
        let _guard = self.enter_call()?;
        if !x_dip.is_finite() || !y_dip.is_finite() {
            return Err(WebViewApiError::InvalidArgument);
        }
        let mut inner = self.inner.lock().map_err(|_| WebViewApiError::Internal)?;
        self.sync_bound_source_locked(&mut inner)?;
        let inner_ref = &mut *inner;
        let mut bridge = RuntimeMvvmBridge {
            exposed_properties: &inner_ref.exposed_properties,
            exposed_commands: &inner_ref.exposed_commands,
        };
        inner_ref
            .engine
            .click_at(x_dip, y_dip, &mut bridge)
            .map(|_| ())
            .map_err(map_webview_engine_error)
    }

    pub(crate) fn text_content_utf8(
        &self,
        buffer: Option<&mut [u8]>,
    ) -> Result<usize, WebViewApiError> {
        let _guard = self.enter_call()?;
        let mut inner = self.inner.lock().map_err(|_| WebViewApiError::Internal)?;
        self.sync_bound_source_locked(&mut inner)?;
        let text = inner.engine.dom().text_content(inner.engine.dom().root());
        let bytes = text.as_bytes();
        let Some(buffer) = buffer else {
            return Ok(bytes.len());
        };
        if buffer.len() < bytes.len() {
            return Err(WebViewApiError::BufferTooSmall {
                required: bytes.len(),
            });
        }
        buffer[..bytes.len()].copy_from_slice(bytes);
        Ok(bytes.len())
    }

    pub(crate) fn render_rgba(
        &self,
        width_px: u32,
        height_px: u32,
        buffer: Option<&mut [u8]>,
    ) -> Result<usize, WebViewApiError> {
        let _guard = self.enter_call()?;
        if width_px == 0 || height_px == 0 || width_px > 4096 || height_px > 4096 {
            return Err(WebViewApiError::InvalidArgument);
        }
        let required = (width_px as usize)
            .checked_mul(height_px as usize)
            .and_then(|px| px.checked_mul(4))
            .ok_or(WebViewApiError::InvalidArgument)?;
        let Some(buffer) = buffer else {
            return Ok(required);
        };
        if buffer.len() < required {
            return Err(WebViewApiError::BufferTooSmall { required });
        }
        let mut inner = self.inner.lock().map_err(|_| WebViewApiError::Internal)?;
        self.sync_bound_source_locked(&mut inner)?;
        let scene = inner
            .engine
            .build_scene()
            .map_err(map_webview_engine_error)?;
        let pixels =
            SoftwareRasterizer::render_to_rgba(&scene, width_px, height_px, ColorRgba::WHITE);
        buffer[..required].copy_from_slice(&pixels[..required]);
        Ok(required)
    }
}

#[cfg(test)]
mod tests {
    use crate::abi_support::*;
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Default)]
    struct CommandProbe {
        calls: AtomicUsize,
        destroys: AtomicUsize,
    }

    unsafe extern "C" fn record_command(userdata: *mut c_void) {
        // SAFETY: test keeps CommandProbe alive until handle release completes.
        let probe = unsafe { &*(userdata.cast::<CommandProbe>()) };
        probe.calls.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn destroy_command(userdata: *mut c_void) {
        // SAFETY: test keeps CommandProbe alive until handle release completes.
        let probe = unsafe { &*(userdata.cast::<CommandProbe>()) };
        probe.destroys.fetch_add(1, Ordering::SeqCst);
    }

    #[test]
    fn webview_binds_source_property_exposes_mvvm_bridge_and_renders_rgba() {
        let runtime = runtime_create_handle().expect("runtime");
        let webview = control_create_webview(runtime).expect("webview");
        let source_prop =
            property_create_utf8(runtime, b"# Initial Heading\n\nBody text").expect("source prop");
        let count_prop = property_create_i64(runtime, 10).expect("count prop");
        let readonly_prop = property_create_i64(runtime, 99).expect("readonly prop");

        let binding =
            webview_bind_source_property(webview, 2, source_prop).expect("bind markdown source");
        let len = webview_text_content_utf8(webview, None).expect("text len");
        let mut buf = vec![0u8; len];
        webview_text_content_utf8(webview, Some(&mut buf)).expect("text copy");
        let rendered = String::from_utf8(buf).expect("utf8");
        assert!(rendered.contains("Initial Heading"));

        property_set_utf8(source_prop, b"# Updated Heading").expect("update source");
        let len2 = webview_text_content_utf8(webview, None).expect("text len 2");
        let mut buf2 = vec![0u8; len2];
        webview_text_content_utf8(webview, Some(&mut buf2)).expect("text copy 2");
        let rendered2 = String::from_utf8(buf2).expect("utf8");
        assert!(rendered2.contains("Updated Heading"));

        webview_expose_property_utf8(webview, b"count", count_prop, true).expect("expose count");
        webview_expose_property_utf8(webview, b"ro", readonly_prop, false)
            .expect("expose readonly");

        let probe = CommandProbe::default();
        webview_expose_command_utf8(
            webview,
            b"save",
            record_command,
            (&probe as *const CommandProbe) as usize,
            destroy_command,
        )
        .expect("expose command");

        webview_eval_script_utf8(
            webview,
            b"let v = freemvvm.getProperty('count'); freemvvm.setProperty('count', v + 5); freemvvm.invokeCommand('save');",
        )
        .expect("eval script");

        assert_eq!(property_get_i64(count_prop).expect("read count"), 15);
        assert_eq!(probe.calls.load(Ordering::SeqCst), 1);

        let ro_err = webview_eval_script_utf8(webview, b"freemvvm.setProperty('ro', 0);")
            .expect_err("read-only property must reject write");
        assert!(matches!(ro_err, WebViewApiError::ValidationFailed(Some(_))));
        assert_eq!(property_get_i64(readonly_prop).expect("read ro"), 99);

        let rgba_len = webview_render_rgba(webview, 64, 32, None).expect("rgba len");
        assert_eq!(rgba_len, 64 * 32 * 4);
        let mut pixels = vec![0u8; rgba_len];
        webview_render_rgba(webview, 64, 32, Some(&mut pixels)).expect("rgba render");

        let retained = runtime_retain_handle(webview).expect("retain webview");
        handle_release(retained).expect("release retained");
        binding_cancel(binding).expect("cancel binding");
        handle_release(binding).expect("release binding");
        handle_release(webview).expect("release webview");
        handle_release(readonly_prop).expect("release ro");
        handle_release(count_prop).expect("release count");
        handle_release(source_prop).expect("release source");
        handle_release(runtime).expect("release runtime");
    }
}

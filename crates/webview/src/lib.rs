//! Pure-Rust zero-dependency WebView engine plugin (`freemvvm-webview`).
//!
//! Implements the 6-layer WebView pipeline specified in:
//! - Requirements: `.docs/11_requirements/RQ-17_webview-plugin.md`
//! - Basic Design: `.docs/12_design/ui/webview-plugin.md`
//! - Detailed Design: `.docs/13_detail_design/ui/webview.md`
//! - Secure Design: `.docs/14_secure_design/08-webview-sandbox-parser-limits.md`

#![allow(clippy::collapsible_if)]

pub mod css;
pub mod dom;
pub mod html;
pub mod layout;
pub mod markdown;
pub mod paint;
pub mod script;

pub use css::{ColorRgba, ComputedStyle, DisplayMode, EdgeSizes, Stylesheet};
pub use dom::{DocumentFormat, DomArena, NodeId, NodeKind, ResourceBudget, WebViewError};
pub use layout::{
    DeterministicTextMeasurer, LayoutBox, RectDip, SizeDip, TextMeasurer, WebLayoutTree,
};
pub use paint::{
    AxAction, AxNode, AxRole, AxSnapshot, SceneCommand, SceneSnapshot, SoftwareRasterizer,
};
pub use script::{InMemoryHostBridge, JsValue, ScriptHostBridge, ScriptVm};

/// Self-contained WebView engine instance managing DOM, stylesheets, layout, script VM, and snapshots.
#[derive(Clone, Debug)]
pub struct WebViewEngine {
    budget: ResourceBudget,
    format: DocumentFormat,
    dom: DomArena,
    inline_stylesheets: Vec<Stylesheet>,
    external_stylesheet: Option<Stylesheet>,
    computed_styles: Vec<ComputedStyle>,
    layout_tree: WebLayoutTree,
    script_vm: ScriptVm,
    viewport: SizeDip,
    scroll_x: f64,
    scroll_y: f64,
    style_dirty: bool,
    layout_dirty: bool,
}

impl Default for WebViewEngine {
    fn default() -> Self {
        Self::new(ResourceBudget::default())
    }
}

impl WebViewEngine {
    /// Creates a new `WebViewEngine` with the specified `ResourceBudget`.
    pub fn new(budget: ResourceBudget) -> Self {
        let dom = DomArena::new(budget);
        let computed_styles = vec![ComputedStyle::default(); dom.node_count()];
        Self {
            budget,
            format: DocumentFormat::Html,
            dom,
            inline_stylesheets: Vec::new(),
            external_stylesheet: None,
            computed_styles,
            layout_tree: WebLayoutTree::default(),
            script_vm: ScriptVm::new(budget),
            viewport: SizeDip::new(800.0, 600.0),
            scroll_x: 0.0,
            scroll_y: 0.0,
            style_dirty: true,
            layout_dirty: true,
        }
    }

    pub const fn budget(&self) -> ResourceBudget {
        self.budget
    }

    pub const fn format(&self) -> DocumentFormat {
        self.format
    }

    pub fn dom(&self) -> &DomArena {
        &self.dom
    }

    pub fn layout_tree(&self) -> &WebLayoutTree {
        &self.layout_tree
    }

    pub const fn viewport(&self) -> SizeDip {
        self.viewport
    }

    pub const fn scroll_offset(&self) -> (f64, f64) {
        (self.scroll_x, self.scroll_y)
    }

    /// Loads a Markdown or HTML document transactionally (`RQ-17-003`, `RQ-17-004`, `RQ-17-009`).
    /// On failure, the previously committed DOM, stylesheets, and script VM state remain intact.
    pub fn set_source(
        &mut self,
        format: DocumentFormat,
        source: &str,
        host: &mut dyn ScriptHostBridge,
    ) -> Result<(), WebViewError> {
        match format {
            DocumentFormat::Markdown => {
                let new_dom = markdown::parse_markdown(source, self.budget)?;
                self.format = DocumentFormat::Markdown;
                self.dom = new_dom;
                self.inline_stylesheets.clear();
                self.script_vm = ScriptVm::new(self.budget);
                self.style_dirty = true;
                self.layout_dirty = true;
                Ok(())
            }
            DocumentFormat::Html => {
                let mut parsed = html::parse_html(source, self.budget)?;
                let mut sheets = Vec::with_capacity(parsed.stylesheets.len());
                for css_src in &parsed.stylesheets {
                    sheets.push(css::parse_stylesheet(css_src, self.budget)?);
                }
                let mut new_vm = ScriptVm::new(self.budget);
                for script_src in &parsed.scripts {
                    new_vm.eval_script(script_src, &mut parsed.dom, host)?;
                }
                self.format = DocumentFormat::Html;
                self.dom = parsed.dom;
                self.inline_stylesheets = sheets;
                self.script_vm = new_vm;
                self.style_dirty = true;
                self.layout_dirty = true;
                Ok(())
            }
        }
    }

    /// Sets an author CSS stylesheet (`RQ-17-004`, `RQ-17-006`).
    pub fn set_stylesheet(&mut self, css_source: &str) -> Result<(), WebViewError> {
        let sheet = css::parse_stylesheet(css_source, self.budget)?;
        self.external_stylesheet = Some(sheet);
        self.style_dirty = true;
        self.layout_dirty = true;
        Ok(())
    }

    /// Evaluates a JavaScript snippet against the current DOM and `host` bridge (`RQ-17-008`).
    pub fn eval_script(
        &mut self,
        script_source: &str,
        host: &mut dyn ScriptHostBridge,
    ) -> Result<JsValue, WebViewError> {
        let val = self
            .script_vm
            .eval_script(script_source, &mut self.dom, host)?;
        self.style_dirty = true;
        self.layout_dirty = true;
        Ok(val)
    }

    /// Updates the viewport size (`Measure` / `Arrange` in `f64` DIP, `RQ-17-005`).
    pub fn set_viewport(&mut self, width: f64, height: f64) -> Result<SizeDip, WebViewError> {
        if !width.is_finite() || !height.is_finite() || width < 0.0 || height < 0.0 {
            return Err(WebViewError::InvalidLayoutConstraint);
        }
        if (self.viewport.width - width).abs() > f64::EPSILON
            || (self.viewport.height - height).abs() > f64::EPSILON
        {
            self.viewport = SizeDip::new(width, height);
            self.layout_dirty = true;
        }
        self.ensure_layout(&DeterministicTextMeasurer)?;
        Ok(self.layout_tree.content_size)
    }

    /// Scrolls the viewport, clamping to valid content bounds.
    pub fn set_scroll_offset(&mut self, scroll_x: f64, scroll_y: f64) -> Result<(), WebViewError> {
        if !scroll_x.is_finite() || !scroll_y.is_finite() {
            return Err(WebViewError::InvalidLayoutConstraint);
        }
        self.ensure_layout(&DeterministicTextMeasurer)?;
        let max_x = (self.layout_tree.content_size.width - self.viewport.width).max(0.0);
        let max_y = (self.layout_tree.content_size.height - self.viewport.height).max(0.0);
        self.scroll_x = scroll_x.clamp(0.0, max_x);
        self.scroll_y = scroll_y.clamp(0.0, max_y);
        Ok(())
    }

    /// Ensures styles and layout are up to date.
    pub fn ensure_layout(&mut self, measurer: &dyn TextMeasurer) -> Result<(), WebViewError> {
        if self.style_dirty {
            let mut all_sheets = self.inline_stylesheets.clone();
            if let Some(ext) = &self.external_stylesheet {
                all_sheets.push(ext.clone());
            }
            self.computed_styles = css::compute_styles(&self.dom, &all_sheets);
            self.style_dirty = false;
            self.layout_dirty = true;
        }
        if self.layout_dirty {
            self.layout_tree =
                layout::compute_layout(&self.dom, &self.computed_styles, self.viewport, measurer)?;
            self.layout_dirty = false;
        }
        Ok(())
    }

    /// Produces an immutable `SceneSnapshot` for the current frame (`RQ-17-001`).
    pub fn build_scene(&mut self) -> Result<SceneSnapshot, WebViewError> {
        self.ensure_layout(&DeterministicTextMeasurer)?;
        Ok(paint::build_scene_snapshot(
            &self.layout_tree,
            0.0,
            0.0,
            self.scroll_x,
            self.scroll_y,
        ))
    }

    /// Produces an immutable `AxSnapshot` accessibility tree for the current DOM & layout (`RQ-17-012`).
    pub fn build_accessibility(&mut self) -> Result<AxSnapshot, WebViewError> {
        self.ensure_layout(&DeterministicTextMeasurer)?;
        Ok(paint::build_ax_snapshot(&self.dom, &self.layout_tree))
    }

    /// Simulates a pointer click at viewport coordinates `(vx, vy)`, dispatching script click handlers
    /// and returning any clicked link `href` or `data-command` attribute (`RQ-17-007`, `RQ-17-008`).
    pub fn click_at(
        &mut self,
        vx: f64,
        vy: f64,
        host: &mut dyn ScriptHostBridge,
    ) -> Result<Option<String>, WebViewError> {
        self.ensure_layout(&DeterministicTextMeasurer)?;
        let content_x = vx + self.scroll_x;
        let content_y = vy + self.scroll_y;
        let Some(hit_id) = self.layout_tree.hit_test(content_x, content_y) else {
            return Ok(None);
        };
        self.activate_node(hit_id, host)
    }

    /// Activates a specific DOM node by `NodeId` (used by pointer click or Accessibility `Invoke`).
    pub fn activate_node(
        &mut self,
        node_id: NodeId,
        host: &mut dyn ScriptHostBridge,
    ) -> Result<Option<String>, WebViewError> {
        // Walk up ancestors from `node_id` to find any click listeners, links, or buttons
        let mut cur_opt = Some(node_id);
        let mut action_target: Option<String> = None;
        while let Some(cur_id) = cur_opt {
            if let Some(node) = self.dom.get(cur_id) {
                if let NodeKind::Element(elem) = &node.kind {
                    if elem.is_disabled() {
                        return Ok(None);
                    }
                    if action_target.is_none() {
                        if let Some(cmd) = elem.get_attribute("data-command") {
                            let cmd_name = cmd.to_owned();
                            host.invoke_command(&cmd_name)?;
                            action_target = Some(format!("command:{cmd_name}"));
                        } else if let Some(href) = elem.get_attribute("href") {
                            let href_owned = href.to_owned();
                            if let Some(cmd_name) = href_owned.strip_prefix("command:") {
                                host.invoke_command(cmd_name)?;
                            }
                            action_target = Some(href_owned);
                        }
                    }
                }
            }
            self.script_vm
                .dispatch_dom_event(cur_id, "click", &mut self.dom, host)?;
            cur_opt = self.dom.get(cur_id).and_then(|n| n.parent);
        }
        self.style_dirty = true;
        self.layout_dirty = true;
        Ok(action_target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_pipeline_markdown_and_html_with_script_and_command_execution() {
        let mut engine = WebViewEngine::default();
        let mut host = InMemoryHostBridge::new();
        host.expose_property("title", JsValue::Str("Live Dashboard".to_owned()), true);
        host.expose_command("refresh", true);

        let html = r#"
            <html>
            <head>
                <style>
                    #header { color: #0055aa; font-size: 24px; }
                    .btn { padding: 8px; background-color: #eeeeee; }
                </style>
            </head>
            <body>
                <h1 id="header">Placeholder</h1>
                <button id="refresh-btn" class="btn" data-command="refresh">Refresh</button>
                <script>
                    let h = document.getElementById("header");
                    h.textContent = freemvvm.getProperty("title");
                </script>
            </body>
            </html>
        "#;

        engine
            .set_source(DocumentFormat::Html, html, &mut host)
            .unwrap();
        let scene = engine.build_scene().unwrap();
        assert!(
            scene.commands.iter().any(
                |c| matches!(c, SceneCommand::GlyphRun { text, .. } if text == "Live Dashboard")
            )
        );

        let btn_id = engine.dom().get_element_by_id("refresh-btn").unwrap();
        let triggered = engine.activate_node(btn_id, &mut host).unwrap();
        assert_eq!(triggered.as_deref(), Some("command:refresh"));
        assert_eq!(host.invoked_commands(), &["refresh"]);
    }
}

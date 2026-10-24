//! Web Layout Engine: BFC, IFC (text wrapping), Flexbox, and Table layout (`UNIT-WEBVIEW-005`).

use crate::css::{
    AlignItems, ColorRgba, ComputedStyle, DisplayMode, FlexDirection, JustifyContent, TextAlign,
    TextDecoration,
};
use crate::dom::{DomArena, NodeId, NodeKind, WebViewError};

/// Logical rectangle in device-independent pixels (`f64` DIP).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RectDip {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl RectDip {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

/// Logical size in device-independent pixels (`f64` DIP).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SizeDip {
    pub width: f64,
    pub height: f64,
}

impl SizeDip {
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

/// Trait for measuring text runs in logical DIP (`TextService` abstraction).
pub trait TextMeasurer {
    fn measure_text(&self, text: &str, font_size: f64, monospace: bool, bold: bool) -> SizeDip;
}

/// Deterministic built-in text measurer for OS-independent Core layout and unit tests.
#[derive(Clone, Copy, Debug, Default)]
pub struct DeterministicTextMeasurer;

impl TextMeasurer for DeterministicTextMeasurer {
    fn measure_text(&self, text: &str, font_size: f64, monospace: bool, bold: bool) -> SizeDip {
        let weight_factor = if bold { 1.06 } else { 1.0 };
        let mut width = 0.0;
        for ch in text.chars() {
            let char_factor = if monospace {
                0.60
            } else if ch.is_ascii() {
                0.55
            } else {
                1.00 // CJK / wide glyph
            };
            width += font_size * char_factor * weight_factor;
        }
        SizeDip::new(width, font_size)
    }
}

/// A positioned inline text fragment produced by the Inline Formatting Context (IFC).
#[derive(Clone, Debug, PartialEq)]
pub struct PositionedTextRun {
    pub node_id: NodeId,
    pub link_node_id: Option<NodeId>,
    pub text: String,
    pub rect: RectDip,
    pub color: ColorRgba,
    pub background_color: ColorRgba,
    pub font_size: f64,
    pub font_weight: u16,
    pub italic: bool,
    pub monospace: bool,
    pub decoration: TextDecoration,
}

/// A positioned layout box corresponding to a DOM node.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutBox {
    pub node_id: NodeId,
    pub border_rect: RectDip,
    pub content_rect: RectDip,
    pub style: ComputedStyle,
    pub text_runs: Vec<PositionedTextRun>,
    pub children: Vec<LayoutBox>,
}

/// Complete layout tree committed after `Measure` / `Arrange`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WebLayoutTree {
    pub viewport: SizeDip,
    pub content_size: SizeDip,
    pub root_box: Option<LayoutBox>,
}

impl WebLayoutTree {
    /// Performs hit-testing in content coordinates `(x, y)` (already adjusted for scroll offset)
    /// and returns the deepest matching DOM `NodeId`.
    pub fn hit_test(&self, x: f64, y: f64) -> Option<NodeId> {
        let root = self.root_box.as_ref()?;
        hit_test_box(root, x, y)
    }
}

fn hit_test_box(b: &LayoutBox, x: f64, y: f64) -> Option<NodeId> {
    if !b.border_rect.contains(x, y) {
        return None;
    }
    for run in b.text_runs.iter().rev() {
        if run.rect.contains(x, y) {
            return Some(run.link_node_id.unwrap_or(run.node_id));
        }
    }
    for child in b.children.iter().rev() {
        if let Some(hit) = hit_test_box(child, x, y) {
            return Some(hit);
        }
    }
    Some(b.node_id)
}

/// Computes the Web layout tree for `dom` and `styles` within `viewport`.
pub fn compute_layout(
    dom: &DomArena,
    styles: &[ComputedStyle],
    viewport: SizeDip,
    measurer: &dyn TextMeasurer,
) -> Result<WebLayoutTree, WebViewError> {
    if !viewport.width.is_finite()
        || !viewport.height.is_finite()
        || viewport.width < 0.0
        || viewport.height < 0.0
    {
        return Err(WebViewError::InvalidLayoutConstraint);
    }

    let root_id = dom.root();
    let root_box = layout_block_node(
        dom,
        styles,
        measurer,
        root_id,
        0.0,
        0.0,
        viewport.width,
        None,
    );
    let content_size = SizeDip::new(
        viewport.width.max(root_box.border_rect.width),
        root_box.border_rect.height,
    );

    Ok(WebLayoutTree {
        viewport,
        content_size,
        root_box: Some(root_box),
    })
}

#[allow(clippy::too_many_arguments)]
fn layout_block_node(
    dom: &DomArena,
    styles: &[ComputedStyle],
    measurer: &dyn TextMeasurer,
    node_id: NodeId,
    origin_x: f64,
    origin_y: f64,
    available_width: f64,
    list_index_prefix: Option<String>,
) -> LayoutBox {
    let style = styles.get(node_id.0).cloned().unwrap_or_default();
    let pad_border_h = style.padding.horizontal() + style.border_width.horizontal();
    let pad_border_v = style.padding.vertical() + style.border_width.vertical();

    let mut border_width = style
        .width
        .map(|w| w + pad_border_h)
        .unwrap_or_else(|| (available_width - style.margin.horizontal()).max(0.0));
    if let Some(min_w) = style.min_width {
        border_width = border_width.max(min_w + pad_border_h);
    }
    if let Some(max_w) = style.max_width {
        border_width = border_width.min(max_w + pad_border_h);
    }

    let border_x = origin_x + style.margin.left;
    let border_y = origin_y + style.margin.top;
    let content_x = border_x + style.border_width.left + style.padding.left;
    let content_y = border_y + style.border_width.top + style.padding.top;
    let content_width = (border_width - pad_border_h).max(0.0);

    let mut child_boxes = Vec::new();
    let mut text_runs = Vec::new();
    let mut cursor_y = content_y;

    let Some(node) = dom.get(node_id) else {
        return LayoutBox {
            node_id,
            border_rect: RectDip::new(border_x, border_y, border_width, 0.0),
            content_rect: RectDip::new(content_x, content_y, content_width, 0.0),
            style,
            text_runs,
            children: child_boxes,
        };
    };

    // Form control / leaf element intrinsic layout (`input`, `button`, `hr`)
    if let NodeKind::Element(elem) = &node.kind {
        if elem.tag_name == "input" {
            let input_type = elem.get_attribute("type").unwrap_or("text");
            if input_type == "checkbox" {
                let box_size = style.font_size;
                return LayoutBox {
                    node_id,
                    border_rect: RectDip::new(border_x, border_y, box_size, box_size),
                    content_rect: RectDip::new(
                        border_x + 2.0,
                        border_y + 2.0,
                        box_size - 4.0,
                        box_size - 4.0,
                    ),
                    style,
                    text_runs,
                    children: child_boxes,
                };
            }
            let val = elem
                .get_attribute("value")
                .or_else(|| elem.get_attribute("placeholder"))
                .unwrap_or("");
            let measured = measurer.measure_text(val, style.font_size, style.monospace, false);
            let w = style.width.unwrap_or(160.0) + pad_border_h;
            let h = style.height.unwrap_or(style.font_size * style.line_height) + pad_border_v;
            if !val.is_empty() {
                text_runs.push(PositionedTextRun {
                    node_id,
                    link_node_id: None,
                    text: val.to_owned(),
                    rect: RectDip::new(content_x, content_y, measured.width, measured.height),
                    color: style.color,
                    background_color: ColorRgba::TRANSPARENT,
                    font_size: style.font_size,
                    font_weight: style.font_weight,
                    italic: style.italic,
                    monospace: style.monospace,
                    decoration: style.text_decoration,
                });
            }
            return LayoutBox {
                node_id,
                border_rect: RectDip::new(border_x, border_y, w, h),
                content_rect: RectDip::new(
                    content_x,
                    content_y,
                    (w - pad_border_h).max(0.0),
                    (h - pad_border_v).max(0.0),
                ),
                style,
                text_runs,
                children: child_boxes,
            };
        }
        if elem.tag_name == "button" && style.width.is_none() {
            let label = dom.text_content(node_id);
            let measured = measurer.measure_text(
                &label,
                style.font_size,
                style.monospace,
                style.font_weight >= 600,
            );
            let w = measured.width + pad_border_h;
            let h = (style.font_size * style.line_height) + pad_border_v;
            if !label.is_empty() {
                text_runs.push(PositionedTextRun {
                    node_id,
                    link_node_id: None,
                    text: label,
                    rect: RectDip::new(content_x, content_y, measured.width, measured.height),
                    color: style.color,
                    background_color: ColorRgba::TRANSPARENT,
                    font_size: style.font_size,
                    font_weight: style.font_weight,
                    italic: style.italic,
                    monospace: style.monospace,
                    decoration: style.text_decoration,
                });
            }
            return LayoutBox {
                node_id,
                border_rect: RectDip::new(border_x, border_y, w, h),
                content_rect: RectDip::new(content_x, content_y, measured.width, measured.height),
                style,
                text_runs,
                children: child_boxes,
            };
        }
    }

    match style.display {
        DisplayMode::Flex => {
            layout_flex_children(
                dom,
                styles,
                measurer,
                node_id,
                &style,
                content_x,
                content_y,
                content_width,
                &mut child_boxes,
                &mut cursor_y,
            );
        }
        DisplayMode::Table => {
            layout_table_children(
                dom,
                styles,
                measurer,
                node_id,
                content_x,
                content_y,
                content_width,
                &mut child_boxes,
                &mut cursor_y,
            );
        }
        _ => {
            layout_bfc_or_ifc_children(
                dom,
                styles,
                measurer,
                node_id,
                &style,
                content_x,
                content_y,
                content_width,
                list_index_prefix,
                &mut child_boxes,
                &mut text_runs,
                &mut cursor_y,
            );
        }
    }

    let mut content_height = (cursor_y - content_y).max(0.0);
    if let Some(explicit_h) = style.height {
        content_height = explicit_h;
    }
    if let Some(min_h) = style.min_height {
        content_height = content_height.max(min_h);
    }
    let border_height = content_height + pad_border_v;

    LayoutBox {
        node_id,
        border_rect: RectDip::new(border_x, border_y, border_width, border_height),
        content_rect: RectDip::new(content_x, content_y, content_width, content_height),
        style,
        text_runs,
        children: child_boxes,
    }
}

#[derive(Clone, Debug)]
struct InlineSpan {
    node_id: NodeId,
    link_node_id: Option<NodeId>,
    text: String,
    style: ComputedStyle,
}

#[allow(clippy::too_many_arguments)]
fn layout_bfc_or_ifc_children(
    dom: &DomArena,
    styles: &[ComputedStyle],
    measurer: &dyn TextMeasurer,
    node_id: NodeId,
    style: &ComputedStyle,
    content_x: f64,
    _content_y: f64,
    content_width: f64,
    list_index_prefix: Option<String>,
    child_boxes: &mut Vec<LayoutBox>,
    text_runs: &mut Vec<PositionedTextRun>,
    cursor_y: &mut f64,
) {
    let Some(node) = dom.get(node_id) else {
        return;
    };
    let is_ordered_list = matches!(&node.kind, NodeKind::Element(e) if e.tag_name == "ol");
    let is_unordered_list = matches!(&node.kind, NodeKind::Element(e) if e.tag_name == "ul");
    let is_pre = matches!(&node.kind, NodeKind::Element(e) if e.tag_name == "pre");

    let mut pending_inlines: Vec<InlineSpan> = Vec::new();
    if let Some(prefix) = list_index_prefix {
        pending_inlines.push(InlineSpan {
            node_id,
            link_node_id: None,
            text: prefix,
            style: style.clone(),
        });
    }

    let mut prev_bottom_margin: f64 = 0.0;
    let mut has_prev_block = false;
    let mut li_counter = 1usize;

    for &child_id in &node.children {
        let child_style = styles.get(child_id.0).cloned().unwrap_or_default();
        if child_style.display == DisplayMode::None {
            continue;
        }
        let is_inline_child = matches!(child_style.display, DisplayMode::Inline);
        if is_inline_child {
            collect_inline_spans(dom, styles, child_id, None, &mut pending_inlines);
        } else {
            if !pending_inlines.is_empty() {
                flush_inline_formatting_context(
                    &pending_inlines,
                    style,
                    measurer,
                    content_x,
                    content_width,
                    is_pre,
                    cursor_y,
                    text_runs,
                );
                pending_inlines.clear();
                prev_bottom_margin = 0.0;
                has_prev_block = false;
            }

            // Margin collapsing between adjacent vertical block siblings
            let top_margin = child_style.margin.top;
            let collapsed_advance = if has_prev_block {
                prev_bottom_margin.max(top_margin) - prev_bottom_margin
            } else {
                top_margin
            };
            let child_origin_y = *cursor_y + collapsed_advance - child_style.margin.top;

            let li_prefix = if is_ordered_list {
                let p = format!("{li_counter}. ");
                li_counter += 1;
                Some(p)
            } else if is_unordered_list {
                Some("• ".to_owned())
            } else {
                None
            };

            let child_box = layout_block_node(
                dom,
                styles,
                measurer,
                child_id,
                content_x,
                child_origin_y,
                content_width,
                li_prefix,
            );
            *cursor_y =
                child_box.border_rect.y + child_box.border_rect.height + child_style.margin.bottom;
            prev_bottom_margin = child_style.margin.bottom;
            has_prev_block = true;
            child_boxes.push(child_box);
        }
    }

    if !pending_inlines.is_empty() {
        flush_inline_formatting_context(
            &pending_inlines,
            style,
            measurer,
            content_x,
            content_width,
            is_pre,
            cursor_y,
            text_runs,
        );
    }
}

fn collect_inline_spans(
    dom: &DomArena,
    styles: &[ComputedStyle],
    node_id: NodeId,
    active_link: Option<NodeId>,
    out: &mut Vec<InlineSpan>,
) {
    let Some(node) = dom.get(node_id) else {
        return;
    };
    let style = styles.get(node_id.0).cloned().unwrap_or_default();
    if style.display == DisplayMode::None {
        return;
    }

    match &node.kind {
        NodeKind::Text(t) => {
            if !t.is_empty() {
                out.push(InlineSpan {
                    node_id,
                    link_node_id: active_link,
                    text: t.clone(),
                    style,
                });
            }
        }
        NodeKind::Element(elem) => {
            if elem.tag_name == "br" {
                out.push(InlineSpan {
                    node_id,
                    link_node_id: active_link,
                    text: "\n".to_owned(),
                    style,
                });
                return;
            }
            let next_link = if elem.tag_name == "a" {
                Some(node_id)
            } else {
                active_link
            };
            for &child_id in &node.children {
                collect_inline_spans(dom, styles, child_id, next_link, out);
            }
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn flush_inline_formatting_context(
    spans: &[InlineSpan],
    container_style: &ComputedStyle,
    measurer: &dyn TextMeasurer,
    content_x: f64,
    content_width: f64,
    preserve_newlines: bool,
    cursor_y: &mut f64,
    out_runs: &mut Vec<PositionedTextRun>,
) {
    #[derive(Clone)]
    struct LineToken {
        node_id: NodeId,
        link_node_id: Option<NodeId>,
        text: String,
        width: f64,
        height: f64,
        style: ComputedStyle,
        is_newline: bool,
    }

    let mut tokens = Vec::new();
    for span in spans {
        let bold = span.style.font_weight >= 600;
        if preserve_newlines || span.text.contains('\n') {
            let parts: Vec<&str> = span.text.split('\n').collect();
            for (idx, part) in parts.iter().enumerate() {
                if !part.is_empty() {
                    let m = measurer.measure_text(
                        part,
                        span.style.font_size,
                        span.style.monospace,
                        bold,
                    );
                    tokens.push(LineToken {
                        node_id: span.node_id,
                        link_node_id: span.link_node_id,
                        text: (*part).to_owned(),
                        width: m.width,
                        height: span.style.font_size * span.style.line_height,
                        style: span.style.clone(),
                        is_newline: false,
                    });
                }
                if idx + 1 < parts.len() {
                    tokens.push(LineToken {
                        node_id: span.node_id,
                        link_node_id: span.link_node_id,
                        text: String::new(),
                        width: 0.0,
                        height: span.style.font_size * span.style.line_height,
                        style: span.style.clone(),
                        is_newline: true,
                    });
                }
            }
        } else {
            for word in split_keeping_trailing_space(&span.text) {
                let m =
                    measurer.measure_text(word, span.style.font_size, span.style.monospace, bold);
                tokens.push(LineToken {
                    node_id: span.node_id,
                    link_node_id: span.link_node_id,
                    text: word.to_owned(),
                    width: m.width,
                    height: span.style.font_size * span.style.line_height,
                    style: span.style.clone(),
                    is_newline: false,
                });
            }
        }
    }

    let max_line_w = if content_width > 0.0 {
        content_width
    } else {
        f64::MAX
    };

    let mut lines: Vec<Vec<LineToken>> = Vec::new();
    let mut current_line: Vec<LineToken> = Vec::new();
    let mut current_w = 0.0;

    for tok in tokens {
        if tok.is_newline {
            lines.push(std::mem::take(&mut current_line));
            current_w = 0.0;
            continue;
        }
        if !current_line.is_empty() && current_w + tok.width > max_line_w {
            lines.push(std::mem::take(&mut current_line));
            current_w = 0.0;
        }
        current_w += tok.width;
        current_line.push(tok);
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    for line in lines {
        let line_h = line.iter().map(|t| t.height).fold(
            container_style.font_size * container_style.line_height,
            f64::max,
        );
        let line_w: f64 = line.iter().map(|t| t.width).sum();
        let align_offset = match container_style.text_align {
            TextAlign::Left => 0.0,
            TextAlign::Center => ((content_width - line_w) * 0.5).max(0.0),
            TextAlign::Right => (content_width - line_w).max(0.0),
        };
        let mut x = content_x + align_offset;
        for tok in line {
            if !tok.text.is_empty() {
                let mut merged = false;
                if let Some(prev) = out_runs.last_mut() {
                    if prev.node_id == tok.node_id
                        && prev.link_node_id == tok.link_node_id
                        && (prev.rect.y - *cursor_y).abs() < 1e-6
                    {
                        prev.text.push_str(&tok.text);
                        prev.rect.width += tok.width;
                        merged = true;
                    }
                }
                if !merged {
                    out_runs.push(PositionedTextRun {
                        node_id: tok.node_id,
                        link_node_id: tok.link_node_id,
                        text: tok.text,
                        rect: RectDip::new(x, *cursor_y, tok.width, line_h),
                        color: tok.style.color,
                        background_color: tok.style.background_color,
                        font_size: tok.style.font_size,
                        font_weight: tok.style.font_weight,
                        italic: tok.style.italic,
                        monospace: tok.style.monospace,
                        decoration: tok.style.text_decoration,
                    });
                }
            }
            x += tok.width;
        }
        *cursor_y += line_h;
    }
}

fn split_keeping_trailing_space(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let bytes = s.as_bytes();
    let mut idx = 0;
    while idx < bytes.len() {
        if bytes[idx].is_ascii_whitespace() {
            while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
                idx += 1;
            }
            result.push(&s[start..idx]);
            start = idx;
        } else {
            let ch_len = s[idx..].chars().next().map_or(1, char::len_utf8);
            idx += ch_len;
        }
    }
    if start < bytes.len() {
        result.push(&s[start..]);
    }
    result
}

#[allow(clippy::too_many_arguments)]
fn layout_flex_children(
    dom: &DomArena,
    styles: &[ComputedStyle],
    measurer: &dyn TextMeasurer,
    node_id: NodeId,
    style: &ComputedStyle,
    content_x: f64,
    content_y: f64,
    content_width: f64,
    child_boxes: &mut Vec<LayoutBox>,
    cursor_y: &mut f64,
) {
    let Some(node) = dom.get(node_id) else {
        return;
    };
    let visible_children: Vec<NodeId> = node
        .children
        .iter()
        .copied()
        .filter(|&cid| {
            styles
                .get(cid.0)
                .is_none_or(|s| s.display != DisplayMode::None)
        })
        .collect();
    if visible_children.is_empty() {
        return;
    }

    match style.flex_direction {
        FlexDirection::Column => {
            let mut y = content_y;
            for (idx, cid) in visible_children.into_iter().enumerate() {
                if idx > 0 {
                    y += style.gap;
                }
                let cb = layout_block_node(
                    dom,
                    styles,
                    measurer,
                    cid,
                    content_x,
                    y,
                    content_width,
                    None,
                );
                y = cb.border_rect.y + cb.border_rect.height + cb.style.margin.bottom;
                child_boxes.push(cb);
            }
            *cursor_y = y;
        }
        FlexDirection::Row => {
            let count = visible_children.len();
            let total_gaps = style.gap * (count.saturating_sub(1) as f64);
            let total_grow: f64 = visible_children
                .iter()
                .map(|&cid| styles.get(cid.0).map_or(0.0, |s| s.flex_grow.max(0.0)))
                .sum();

            let mut provisional = Vec::with_capacity(count);
            let mut used_width = total_gaps;
            for &cid in &visible_children {
                let cstyle = styles.get(cid.0).cloned().unwrap_or_default();
                let basis = cstyle
                    .width
                    .unwrap_or((content_width - total_gaps) / (count as f64).max(1.0));
                let cb = layout_block_node(
                    dom, styles, measurer, cid, content_x, content_y, basis, None,
                );
                used_width += cb.border_rect.width + cb.style.margin.horizontal();
                provisional.push(cb);
            }

            let free_space = (content_width - used_width).max(0.0);
            let mut x = content_x
                + if total_grow == 0.0 {
                    match style.justify_content {
                        JustifyContent::FlexStart | JustifyContent::SpaceBetween => 0.0,
                        JustifyContent::Center => free_space * 0.5,
                        JustifyContent::FlexEnd => free_space,
                    }
                } else {
                    0.0
                };
            let extra_gap = if total_grow == 0.0
                && style.justify_content == JustifyContent::SpaceBetween
                && count > 1
            {
                free_space / ((count - 1) as f64)
            } else {
                0.0
            };

            let mut max_row_h = 0.0_f64;
            for (idx, cid) in visible_children.into_iter().enumerate() {
                if idx > 0 {
                    x += style.gap + extra_gap;
                }
                let grow = styles.get(cid.0).map_or(0.0, |s| s.flex_grow.max(0.0));
                let allocated_w = provisional[idx].border_rect.width
                    + if total_grow > 0.0 {
                        free_space * (grow / total_grow)
                    } else {
                        0.0
                    };
                let cb =
                    layout_block_node(dom, styles, measurer, cid, x, content_y, allocated_w, None);
                max_row_h = max_row_h.max(cb.border_rect.height + cb.style.margin.vertical());
                x += cb.border_rect.width + cb.style.margin.horizontal();
                child_boxes.push(cb);
            }

            if style.align_items == AlignItems::Stretch {
                for cb in child_boxes.iter_mut() {
                    cb.border_rect.height = max_row_h;
                }
            }
            *cursor_y = content_y + max_row_h;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn layout_table_children(
    dom: &DomArena,
    styles: &[ComputedStyle],
    measurer: &dyn TextMeasurer,
    table_id: NodeId,
    content_x: f64,
    content_y: f64,
    content_width: f64,
    child_boxes: &mut Vec<LayoutBox>,
    cursor_y: &mut f64,
) {
    let mut rows = Vec::new();
    collect_table_rows(dom, table_id, &mut rows);
    if rows.is_empty() {
        return;
    }

    let max_cols = rows
        .iter()
        .map(|&row_id| dom.get(row_id).map_or(0, |r| r.children.len()))
        .max()
        .unwrap_or(1)
        .max(1);

    let col_width = content_width / (max_cols as f64);
    let mut y = content_y;

    for row_id in rows {
        let Some(row_node) = dom.get(row_id) else {
            continue;
        };
        let row_style = styles.get(row_id.0).cloned().unwrap_or_default();
        let mut cell_boxes = Vec::new();
        let mut row_h = 0.0_f64;

        for (col_idx, &cell_id) in row_node.children.iter().enumerate() {
            let cell_x = content_x + (col_idx as f64) * col_width;
            let cell_box =
                layout_block_node(dom, styles, measurer, cell_id, cell_x, y, col_width, None);
            row_h = row_h.max(cell_box.border_rect.height);
            cell_boxes.push(cell_box);
        }

        for cb in &mut cell_boxes {
            cb.border_rect.height = row_h;
        }

        child_boxes.push(LayoutBox {
            node_id: row_id,
            border_rect: RectDip::new(content_x, y, content_width, row_h),
            content_rect: RectDip::new(content_x, y, content_width, row_h),
            style: row_style,
            text_runs: Vec::new(),
            children: cell_boxes,
        });
        y += row_h;
    }
    *cursor_y = y;
}

fn collect_table_rows(dom: &DomArena, node_id: NodeId, out: &mut Vec<NodeId>) {
    let Some(node) = dom.get(node_id) else {
        return;
    };
    for &child_id in &node.children {
        if let Some(child) = dom.get(child_id) {
            if let NodeKind::Element(elem) = &child.kind {
                if elem.tag_name == "tr" {
                    out.push(child_id);
                } else if matches!(elem.tag_name.as_str(), "thead" | "tbody" | "tfoot") {
                    collect_table_rows(dom, child_id, out);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::{compute_styles, parse_stylesheet};
    use crate::dom::ResourceBudget;
    use crate::html::parse_html;

    #[test]
    fn layouts_bfc_margin_collapse_ifc_wrapping_flexbox_and_table() {
        let html = r#"
            <div id="root" style="padding: 10px">
                <p id="p1" style="margin-bottom: 20px">Hello world from FreeMVVM WebView layout engine with wrapping text</p>
                <p id="p2" style="margin-top: 15px">Second paragraph after collapsed margin</p>
                <div id="flex" style="display: flex; gap: 10px">
                    <div id="f1" style="flex-grow: 1">Left</div>
                    <div id="f2" style="flex-grow: 1">Right</div>
                </div>
                <table>
                    <tr><th>Col A</th><th>Col B</th></tr>
                    <tr><td>Val 1</td><td>Val 2</td></tr>
                </table>
            </div>
        "#;
        let doc = parse_html(html, ResourceBudget::default()).unwrap();
        let sheet = parse_stylesheet("", ResourceBudget::default()).unwrap();
        let styles = compute_styles(&doc.dom, &[sheet]);

        let tree_wide = compute_layout(
            &doc.dom,
            &styles,
            SizeDip::new(600.0, 400.0),
            &DeterministicTextMeasurer,
        )
        .unwrap();
        let tree_narrow = compute_layout(
            &doc.dom,
            &styles,
            SizeDip::new(180.0, 400.0),
            &DeterministicTextMeasurer,
        )
        .unwrap();

        // Narrow viewport wraps inline text onto more lines, producing a taller layout
        assert!(tree_narrow.content_size.height > tree_wide.content_size.height);

        // Invalid layout constraint is rejected
        assert_eq!(
            compute_layout(
                &doc.dom,
                &styles,
                SizeDip::new(f64::NAN, 400.0),
                &DeterministicTextMeasurer,
            ),
            Err(WebViewError::InvalidLayoutConstraint)
        );
    }
}

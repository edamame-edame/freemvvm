//! `SceneSnapshot` display list generator, `AxSnapshot` accessibility generator,
//! and headless software RGBA rasterizer (`UNIT-WEBVIEW-006`).

use crate::css::{ColorRgba, EdgeSizes, TextDecoration};
use crate::dom::{DomArena, DomNode, NodeId, NodeKind};
use crate::layout::{LayoutBox, RectDip, WebLayoutTree};

/// Primitive drawing command emitted into a `SceneSnapshot`.
#[derive(Clone, Debug, PartialEq)]
pub enum SceneCommand {
    /// Pushes an axis-aligned clip rectangle in viewport DIP coordinates.
    PushClip { rect: RectDip },
    /// Pops the most recently pushed clip rectangle.
    PopClip,
    /// Fills a solid rectangle.
    FillRect {
        rect: RectDip,
        color: ColorRgba,
        border_radius: f64,
    },
    /// Strokes a rectangle border.
    StrokeRect {
        rect: RectDip,
        widths: EdgeSizes,
        color: ColorRgba,
    },
    /// Draws a horizontal or vertical line segment (underline, strikethrough, rule).
    DrawLine {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        thickness: f64,
        color: ColorRgba,
    },
    /// Draws a positioned run of glyphs.
    GlyphRun {
        node_id: NodeId,
        text: String,
        rect: RectDip,
        color: ColorRgba,
        font_size: f64,
        font_weight: u16,
        italic: bool,
        monospace: bool,
    },
}

/// Immutable display list snapshot for a single committed frame (`RQ-17-001`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneSnapshot {
    pub viewport: RectDip,
    pub commands: Vec<SceneCommand>,
}

/// Generates a `SceneSnapshot` from `layout` clipped to `viewport` and translated by `(scroll_x, scroll_y)`.
pub fn build_scene_snapshot(
    layout: &WebLayoutTree,
    viewport_origin_x: f64,
    viewport_origin_y: f64,
    scroll_x: f64,
    scroll_y: f64,
) -> SceneSnapshot {
    let viewport = RectDip::new(
        viewport_origin_x,
        viewport_origin_y,
        layout.viewport.width,
        layout.viewport.height,
    );
    let mut commands = Vec::new();
    commands.push(SceneCommand::PushClip { rect: viewport });

    if let Some(root) = &layout.root_box {
        emit_box_commands(
            root,
            viewport_origin_x - scroll_x,
            viewport_origin_y - scroll_y,
            &mut commands,
        );
    }

    commands.push(SceneCommand::PopClip);
    SceneSnapshot { viewport, commands }
}

fn emit_box_commands(b: &LayoutBox, offset_x: f64, offset_y: f64, out: &mut Vec<SceneCommand>) {
    let rect = RectDip::new(
        b.border_rect.x + offset_x,
        b.border_rect.y + offset_y,
        b.border_rect.width,
        b.border_rect.height,
    );

    if b.style.background_color.a > 0 && rect.width > 0.0 && rect.height > 0.0 {
        out.push(SceneCommand::FillRect {
            rect,
            color: b.style.background_color,
            border_radius: b.style.border_radius,
        });
    }

    if (b.style.border_width.top > 0.0
        || b.style.border_width.right > 0.0
        || b.style.border_width.bottom > 0.0
        || b.style.border_width.left > 0.0)
        && b.style.border_color.a > 0
    {
        out.push(SceneCommand::StrokeRect {
            rect,
            widths: b.style.border_width,
            color: b.style.border_color,
        });
    }

    for run in &b.text_runs {
        let run_rect = RectDip::new(
            run.rect.x + offset_x,
            run.rect.y + offset_y,
            run.rect.width,
            run.rect.height,
        );
        if run.background_color.a > 0 {
            out.push(SceneCommand::FillRect {
                rect: run_rect,
                color: run.background_color,
                border_radius: 2.0,
            });
        }
        out.push(SceneCommand::GlyphRun {
            node_id: run.node_id,
            text: run.text.clone(),
            rect: run_rect,
            color: run.color,
            font_size: run.font_size,
            font_weight: run.font_weight,
            italic: run.italic,
            monospace: run.monospace,
        });
        match run.decoration {
            TextDecoration::None => {}
            TextDecoration::Underline => {
                let y = run_rect.y + run_rect.height - 2.0;
                out.push(SceneCommand::DrawLine {
                    x1: run_rect.x,
                    y1: y,
                    x2: run_rect.x + run_rect.width,
                    y2: y,
                    thickness: 1.0,
                    color: run.color,
                });
            }
            TextDecoration::LineThrough => {
                let y = run_rect.y + run_rect.height * 0.5;
                out.push(SceneCommand::DrawLine {
                    x1: run_rect.x,
                    y1: y,
                    x2: run_rect.x + run_rect.width,
                    y2: y,
                    thickness: 1.0,
                    color: run.color,
                });
            }
        }
    }

    for child in &b.children {
        emit_box_commands(child, offset_x, offset_y, out);
    }
}

/// Platform-neutral semantic Accessibility role for WebView DOM nodes (`RQ-17-012`, `RQ-10-002`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxRole {
    WebDocument,
    Heading { level: u8 },
    Paragraph,
    Link,
    Button,
    EditableText,
    CheckBox,
    List,
    ListItem,
    Table,
    TableRow,
    TableCell,
    CodeBlock,
    BlockQuote,
    Group,
}

/// Supported Accessibility action on a semantic WebView node (`RQ-10-007`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxAction {
    Invoke,
    Focus,
    SetValue,
}

/// A single Accessibility node inside `AxSnapshot`.
#[derive(Clone, Debug, PartialEq)]
pub struct AxNode {
    pub node_id: NodeId,
    pub role: AxRole,
    pub name: String,
    pub value: Option<String>,
    pub enabled: bool,
    pub focused: bool,
    pub checked: Option<bool>,
    pub bounds: RectDip,
    pub actions: Vec<AxAction>,
    pub children: Vec<AxNode>,
}

/// Complete Accessibility snapshot projected from the WebView DOM and Layout (`RQ-17-012`).
#[derive(Clone, Debug, PartialEq)]
pub struct AxSnapshot {
    pub root: AxNode,
}

/// Builds an `AxSnapshot` from `dom` and `layout`.
pub fn build_ax_snapshot(dom: &DomArena, layout: &WebLayoutTree) -> AxSnapshot {
    let root_bounds = layout
        .root_box
        .as_ref()
        .map_or(RectDip::default(), |b| b.border_rect);
    let mut children = Vec::new();
    if let Some(root_box) = &layout.root_box {
        collect_ax_nodes(dom, root_box, &mut children);
    }
    AxSnapshot {
        root: AxNode {
            node_id: dom.root(),
            role: AxRole::WebDocument,
            name: String::new(),
            value: None,
            enabled: true,
            focused: false,
            checked: None,
            bounds: root_bounds,
            actions: Vec::new(),
            children,
        },
    }
}

fn collect_ax_nodes(dom: &DomArena, layout_box: &LayoutBox, out: &mut Vec<AxNode>) {
    let Some(dom_node) = dom.get(layout_box.node_id) else {
        return;
    };
    let NodeKind::Element(elem) = &dom_node.kind else {
        for child in &layout_box.children {
            collect_ax_nodes(dom, child, out);
        }
        return;
    };

    let role_opt = match elem.tag_name.as_str() {
        "h1" => Some(AxRole::Heading { level: 1 }),
        "h2" => Some(AxRole::Heading { level: 2 }),
        "h3" => Some(AxRole::Heading { level: 3 }),
        "h4" => Some(AxRole::Heading { level: 4 }),
        "h5" => Some(AxRole::Heading { level: 5 }),
        "h6" => Some(AxRole::Heading { level: 6 }),
        "p" => Some(AxRole::Paragraph),
        "a" => Some(AxRole::Link),
        "button" => Some(AxRole::Button),
        "input" => match elem.get_attribute("type").unwrap_or("text") {
            "checkbox" => Some(AxRole::CheckBox),
            "button" | "submit" => Some(AxRole::Button),
            _ => Some(AxRole::EditableText),
        },
        "textarea" => Some(AxRole::EditableText),
        "ul" | "ol" => Some(AxRole::List),
        "li" => Some(AxRole::ListItem),
        "table" => Some(AxRole::Table),
        "tr" => Some(AxRole::TableRow),
        "th" | "td" => Some(AxRole::TableCell),
        "pre" => Some(AxRole::CodeBlock),
        "blockquote" => Some(AxRole::BlockQuote),
        _ => None,
    };

    // Also inspect inline `<a>` links inside text runs that do not have separate block `LayoutBox`es
    let mut inline_link_nodes = Vec::new();
    for run in &layout_box.text_runs {
        if let Some(link_id) = run.link_node_id {
            if !inline_link_nodes
                .iter()
                .any(|n: &AxNode| n.node_id == link_id)
            {
                if let Some(DomNode {
                    kind: NodeKind::Element(link_elem),
                    ..
                }) = dom.get(link_id)
                {
                    let enabled = !link_elem.is_disabled();
                    inline_link_nodes.push(AxNode {
                        node_id: link_id,
                        role: AxRole::Link,
                        name: link_elem
                            .get_attribute("aria-label")
                            .map(str::to_owned)
                            .unwrap_or_else(|| dom.text_content(link_id)),
                        value: link_elem.get_attribute("href").map(str::to_owned),
                        enabled,
                        focused: link_elem.is_focused,
                        checked: None,
                        bounds: run.rect,
                        actions: if enabled {
                            vec![AxAction::Invoke, AxAction::Focus]
                        } else {
                            Vec::new()
                        },
                        children: Vec::new(),
                    });
                }
            }
        }
    }

    let mut child_ax = inline_link_nodes;
    for child_box in &layout_box.children {
        collect_ax_nodes(dom, child_box, &mut child_ax);
    }

    if let Some(role) = role_opt {
        let enabled = !elem.is_disabled();
        let name = elem
            .get_attribute("aria-label")
            .map(str::to_owned)
            .unwrap_or_else(|| dom.text_content(layout_box.node_id));
        let value = match role {
            AxRole::EditableText => Some(elem.get_attribute("value").unwrap_or("").to_owned()),
            AxRole::Link => elem.get_attribute("href").map(str::to_owned),
            AxRole::Paragraph | AxRole::TableCell | AxRole::CodeBlock => {
                Some(dom.text_content(layout_box.node_id))
            }
            _ => None,
        };
        let checked = if role == AxRole::CheckBox {
            Some(elem.is_checked())
        } else {
            None
        };
        let mut actions = Vec::new();
        if enabled {
            match role {
                AxRole::Link | AxRole::Button | AxRole::CheckBox => {
                    actions.push(AxAction::Invoke);
                    actions.push(AxAction::Focus);
                }
                AxRole::EditableText => {
                    actions.push(AxAction::Focus);
                    actions.push(AxAction::SetValue);
                }
                _ => {}
            }
        }
        out.push(AxNode {
            node_id: layout_box.node_id,
            role,
            name,
            value,
            enabled,
            focused: elem.is_focused,
            checked,
            bounds: layout_box.border_rect,
            actions,
            children: child_ax,
        });
    } else {
        out.extend(child_ax);
    }
}

/// Deterministic software RGBA framebuffer rasterizer for headless verification and embedded targets.
pub struct SoftwareRasterizer;

impl SoftwareRasterizer {
    /// Rasterizes `scene` into an RGBA8888 pixel buffer of `(width, height)`.
    pub fn render_to_rgba(
        scene: &SceneSnapshot,
        width: u32,
        height: u32,
        clear: ColorRgba,
    ) -> Vec<u8> {
        let pixel_count = (width as usize) * (height as usize);
        let mut buf = vec![0u8; pixel_count * 4];
        for px in buf.chunks_exact_mut(4) {
            px[0] = clear.r;
            px[1] = clear.g;
            px[2] = clear.b;
            px[3] = clear.a;
        }

        let mut clip_stack: Vec<RectDip> =
            vec![RectDip::new(0.0, 0.0, width as f64, height as f64)];

        for cmd in &scene.commands {
            let clip = clip_stack.last().copied().unwrap_or(RectDip::new(
                0.0,
                0.0,
                width as f64,
                height as f64,
            ));
            match cmd {
                SceneCommand::PushClip { rect } => {
                    let x0 = clip.x.max(rect.x);
                    let y0 = clip.y.max(rect.y);
                    let x1 = (clip.x + clip.width).min(rect.x + rect.width);
                    let y1 = (clip.y + clip.height).min(rect.y + rect.height);
                    clip_stack.push(RectDip::new(x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0)));
                }
                SceneCommand::PopClip => {
                    if clip_stack.len() > 1 {
                        clip_stack.pop();
                    }
                }
                SceneCommand::FillRect { rect, color, .. } => {
                    fill_clipped_rect(&mut buf, width, height, *rect, clip, *color);
                }
                SceneCommand::StrokeRect {
                    rect,
                    widths,
                    color,
                } => {
                    if widths.top > 0.0 {
                        fill_clipped_rect(
                            &mut buf,
                            width,
                            height,
                            RectDip::new(rect.x, rect.y, rect.width, widths.top),
                            clip,
                            *color,
                        );
                    }
                    if widths.bottom > 0.0 {
                        fill_clipped_rect(
                            &mut buf,
                            width,
                            height,
                            RectDip::new(
                                rect.x,
                                rect.y + rect.height - widths.bottom,
                                rect.width,
                                widths.bottom,
                            ),
                            clip,
                            *color,
                        );
                    }
                    if widths.left > 0.0 {
                        fill_clipped_rect(
                            &mut buf,
                            width,
                            height,
                            RectDip::new(rect.x, rect.y, widths.left, rect.height),
                            clip,
                            *color,
                        );
                    }
                    if widths.right > 0.0 {
                        fill_clipped_rect(
                            &mut buf,
                            width,
                            height,
                            RectDip::new(
                                rect.x + rect.width - widths.right,
                                rect.y,
                                widths.right,
                                rect.height,
                            ),
                            clip,
                            *color,
                        );
                    }
                }
                SceneCommand::DrawLine {
                    x1,
                    y1,
                    x2,
                    y2,
                    thickness,
                    color,
                } => {
                    let min_x = x1.min(*x2);
                    let min_y = y1.min(*y2);
                    let w = (x2 - x1).abs().max(*thickness);
                    let h = (y2 - y1).abs().max(*thickness);
                    fill_clipped_rect(
                        &mut buf,
                        width,
                        height,
                        RectDip::new(min_x, min_y, w, h),
                        clip,
                        *color,
                    );
                }
                SceneCommand::GlyphRun {
                    rect, color, text, ..
                } => {
                    // Render deterministic glyph blocks for non-whitespace characters
                    let char_count = text.chars().count().max(1);
                    let step = rect.width / (char_count as f64);
                    for (idx, ch) in text.chars().enumerate() {
                        if !ch.is_whitespace() {
                            let gx = rect.x + (idx as f64) * step + 1.0;
                            let gy = rect.y + rect.height * 0.2;
                            let gw = (step - 1.5).max(1.0);
                            let gh = (rect.height * 0.6).max(1.0);
                            fill_clipped_rect(
                                &mut buf,
                                width,
                                height,
                                RectDip::new(gx, gy, gw, gh),
                                clip,
                                *color,
                            );
                        }
                    }
                }
            }
        }
        buf
    }
}

fn fill_clipped_rect(
    buf: &mut [u8],
    width: u32,
    height: u32,
    rect: RectDip,
    clip: RectDip,
    color: ColorRgba,
) {
    if color.a == 0 {
        return;
    }
    let x0 = rect.x.max(clip.x).max(0.0).floor() as u32;
    let y0 = rect.y.max(clip.y).max(0.0).floor() as u32;
    let x1 = (rect.x + rect.width)
        .min(clip.x + clip.width)
        .min(width as f64)
        .ceil() as u32;
    let y1 = (rect.y + rect.height)
        .min(clip.y + clip.height)
        .min(height as f64)
        .ceil() as u32;

    for y in y0..y1.min(height) {
        for x in x0..x1.min(width) {
            let offset = ((y * width + x) as usize) * 4;
            buf[offset] = color.r;
            buf[offset + 1] = color.g;
            buf[offset + 2] = color.b;
            buf[offset + 3] = color.a;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::{compute_styles, parse_stylesheet};
    use crate::dom::ResourceBudget;
    use crate::layout::{DeterministicTextMeasurer, SizeDip, compute_layout};
    use crate::markdown::parse_markdown;

    #[test]
    fn generates_scene_snapshot_ax_snapshot_and_rasterizes_pixels() {
        let md = "# Heading 1\n\nVisit [Run Action](command:execute) now.";
        let dom = parse_markdown(md, ResourceBudget::default()).unwrap();
        let sheet = parse_stylesheet(
            "body { background-color: #ffffff; }",
            ResourceBudget::default(),
        )
        .unwrap();
        let styles = compute_styles(&dom, &[sheet]);
        let layout = compute_layout(
            &dom,
            &styles,
            SizeDip::new(320.0, 200.0),
            &DeterministicTextMeasurer,
        )
        .unwrap();

        let scene = build_scene_snapshot(&layout, 0.0, 0.0, 0.0, 0.0);
        assert!(
            scene.commands.iter().any(
                |c| matches!(c, SceneCommand::GlyphRun { text, .. } if text.contains("Heading"))
            )
        );
        assert!(
            scene
                .commands
                .iter()
                .any(|c| matches!(c, SceneCommand::DrawLine { .. }))
        );

        let ax = build_ax_snapshot(&dom, &layout);
        assert_eq!(ax.root.role, AxRole::WebDocument);
        assert!(
            ax.root
                .children
                .iter()
                .any(|n| n.role == AxRole::Heading { level: 1 })
        );

        let rgba = SoftwareRasterizer::render_to_rgba(&scene, 320, 200, ColorRgba::WHITE);
        assert_eq!(rgba.len(), 320 * 200 * 4);
        // Verify non-white foreground glyph pixels were drawn
        assert!(rgba.chunks_exact(4).any(|px| px != [255, 255, 255, 255]));
    }
}

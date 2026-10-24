//! CSS parser, selector matcher, specificity, User-Agent defaults, and Cascade (`UNIT-WEBVIEW-004`).

use crate::dom::{DomArena, ElementData, NodeId, NodeKind, ResourceBudget, WebViewError};

/// RGBA color value in sRGB `(r, g, b, a)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorRgba {
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// CSS `display` property values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    Block,
    Inline,
    InlineBlock,
    Flex,
    Table,
    TableRow,
    TableCell,
    None,
}

/// CSS `flex-direction` property values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
}

/// CSS `justify-content` property values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    Center,
    FlexEnd,
    SpaceBetween,
}

/// CSS `align-items` property values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignItems {
    Stretch,
    FlexStart,
    Center,
    FlexEnd,
}

/// CSS `text-align` property values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// CSS `text-decoration` property values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDecoration {
    None,
    Underline,
    LineThrough,
}

/// Edge sizes in logical DIP (`top`, `right`, `bottom`, `left`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeSizes {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl EdgeSizes {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    pub const fn all(v: f64) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }

    pub const fn symmetric(vertical: f64, horizontal: f64) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub fn horizontal(&self) -> f64 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f64 {
        self.top + self.bottom
    }
}

/// Resolved style for a single DOM node after cascading UA, author, and inline declarations.
#[derive(Clone, Debug, PartialEq)]
pub struct ComputedStyle {
    pub display: DisplayMode,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub min_width: Option<f64>,
    pub max_width: Option<f64>,
    pub min_height: Option<f64>,
    pub margin: EdgeSizes,
    pub padding: EdgeSizes,
    pub border_width: EdgeSizes,
    pub border_color: ColorRgba,
    pub border_radius: f64,
    pub color: ColorRgba,
    pub background_color: ColorRgba,
    pub font_size: f64,
    pub font_weight: u16,
    pub italic: bool,
    pub monospace: bool,
    pub line_height: f64,
    pub text_align: TextAlign,
    pub text_decoration: TextDecoration,
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub flex_grow: f64,
    pub gap: f64,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: DisplayMode::Block,
            width: None,
            height: None,
            min_width: None,
            max_width: None,
            min_height: None,
            margin: EdgeSizes::ZERO,
            padding: EdgeSizes::ZERO,
            border_width: EdgeSizes::ZERO,
            border_color: ColorRgba::new(208, 215, 222, 255),
            border_radius: 0.0,
            color: ColorRgba::new(31, 35, 40, 255),
            background_color: ColorRgba::TRANSPARENT,
            font_size: 16.0,
            font_weight: 400,
            italic: false,
            monospace: false,
            line_height: 1.4,
            text_align: TextAlign::Left,
            text_decoration: TextDecoration::None,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            flex_grow: 0.0,
            gap: 0.0,
        }
    }
}

impl ComputedStyle {
    /// Creates an initial style for a child inheriting inheritable properties from `parent`.
    pub fn inherit_from(parent: &Self, tag_name: &str) -> Self {
        let mut style = Self {
            color: parent.color,
            font_size: parent.font_size,
            font_weight: parent.font_weight,
            italic: parent.italic,
            monospace: parent.monospace,
            line_height: parent.line_height,
            text_align: parent.text_align,
            text_decoration: parent.text_decoration,
            ..Self::default()
        };
        apply_user_agent_defaults(&mut style, tag_name);
        style
    }
}

fn apply_user_agent_defaults(style: &mut ComputedStyle, tag_name: &str) {
    match tag_name {
        "head" | "style" | "script" | "meta" | "link" | "title" => {
            style.display = DisplayMode::None;
        }
        "span" | "a" | "strong" | "b" | "em" | "i" | "del" | "s" | "code" | "label" => {
            style.display = DisplayMode::Inline;
        }
        "button" | "input" | "img" => {
            style.display = DisplayMode::InlineBlock;
        }
        "table" => {
            style.display = DisplayMode::Table;
            style.margin = EdgeSizes::symmetric(8.0, 0.0);
        }
        "thead" | "tbody" | "tfoot" => {
            style.display = DisplayMode::Block;
        }
        "tr" => {
            style.display = DisplayMode::TableRow;
        }
        "th" | "td" => {
            style.display = DisplayMode::TableCell;
            style.padding = EdgeSizes::symmetric(6.0, 10.0);
            style.border_width = EdgeSizes::all(1.0);
            if tag_name == "th" {
                style.font_weight = 700;
                style.background_color = ColorRgba::new(246, 248, 250, 255);
            }
        }
        "h1" => {
            style.font_size = 28.0;
            style.font_weight = 700;
            style.margin = EdgeSizes::symmetric(12.0, 0.0);
        }
        "h2" => {
            style.font_size = 22.0;
            style.font_weight = 700;
            style.margin = EdgeSizes::symmetric(10.0, 0.0);
        }
        "h3" => {
            style.font_size = 18.0;
            style.font_weight = 700;
            style.margin = EdgeSizes::symmetric(8.0, 0.0);
        }
        "h4" | "h5" | "h6" => {
            style.font_size = 16.0;
            style.font_weight = 700;
            style.margin = EdgeSizes::symmetric(6.0, 0.0);
        }
        "p" | "ul" | "ol" => {
            style.margin = EdgeSizes::symmetric(8.0, 0.0);
            if tag_name == "ul" || tag_name == "ol" {
                style.padding.left = 20.0;
            }
        }
        "li" => {
            style.margin = EdgeSizes::symmetric(2.0, 0.0);
        }
        "blockquote" => {
            style.margin = EdgeSizes::symmetric(8.0, 0.0);
            style.padding = EdgeSizes {
                top: 4.0,
                right: 8.0,
                bottom: 4.0,
                left: 12.0,
            };
            style.border_width.left = 4.0;
            style.border_color = ColorRgba::new(208, 215, 222, 255);
            style.color = ColorRgba::new(87, 96, 106, 255);
        }
        "pre" => {
            style.margin = EdgeSizes::symmetric(8.0, 0.0);
            style.padding = EdgeSizes::all(10.0);
            style.background_color = ColorRgba::new(246, 248, 250, 255);
            style.border_width = EdgeSizes::all(1.0);
            style.monospace = true;
        }
        "hr" => {
            style.height = Some(1.0);
            style.margin = EdgeSizes::symmetric(12.0, 0.0);
            style.background_color = ColorRgba::new(208, 215, 222, 255);
        }
        "body" => {
            style.padding = EdgeSizes::all(8.0);
        }
        _ => {}
    }

    match tag_name {
        "a" => {
            style.color = ColorRgba::new(9, 105, 218, 255);
            style.text_decoration = TextDecoration::Underline;
        }
        "strong" | "b" => {
            style.font_weight = 700;
        }
        "em" | "i" => {
            style.italic = true;
        }
        "del" | "s" => {
            style.text_decoration = TextDecoration::LineThrough;
        }
        "code" => {
            style.monospace = true;
            style.background_color = ColorRgba::new(243, 244, 246, 255);
        }
        "button" | "input" => {
            style.padding = EdgeSizes::symmetric(4.0, 8.0);
            style.border_width = EdgeSizes::all(1.0);
            style.background_color = ColorRgba::new(246, 248, 250, 255);
        }
        _ => {}
    }
}

/// CSS selector specificity `(id_count, class_or_pseudo_count, tag_count)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity(pub u16, pub u16, pub u16);

/// Combinator between compound selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Combinator {
    /// Space (`' '`) — any ancestor.
    Descendant,
    /// `>` — direct parent.
    Child,
}

/// Pseudo-class condition on a compound selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PseudoClass {
    Hover,
    Focus,
    Active,
    Disabled,
    Checked,
}

/// A single compound selector (`tag#id.class1.class2:hover`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompoundSelector {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub pseudos: Vec<PseudoClass>,
}

/// A complex selector consisting of a root compound selector and `(Combinator, CompoundSelector)` chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selector {
    pub head: CompoundSelector,
    pub chain: Vec<(Combinator, CompoundSelector)>,
    pub specificity: Specificity,
}

/// A single CSS declaration (`property: value`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
    pub property: String,
    pub value: String,
}

/// A parsed CSS rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CssRule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

/// A parsed CSS stylesheet.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stylesheet {
    pub rules: Vec<CssRule>,
}

/// Parses a CSS stylesheet string while enforcing `budget` (`SD-WV-03`).
pub fn parse_stylesheet(css: &str, budget: ResourceBudget) -> Result<Stylesheet, WebViewError> {
    if css.len() > budget.max_source_bytes {
        return Err(WebViewError::SourceBytesLimitExceeded {
            actual: css.len(),
            max: budget.max_source_bytes,
        });
    }
    let clean = strip_css_comments(css);
    let mut rules = Vec::new();
    let mut rest = clean.as_str();

    while let Some(open_brace) = rest.find('{') {
        let selector_prelude = rest[..open_brace].trim();
        let after_open = &rest[open_brace + 1..];
        let Some(close_brace) = after_open.find('}') else {
            break;
        };
        let decl_block = &after_open[..close_brace];
        rest = &after_open[close_brace + 1..];

        if selector_prelude.is_empty() || selector_prelude.starts_with('@') {
            continue;
        }

        if rules.len() >= budget.max_css_rules {
            return Err(WebViewError::CssLimitExceeded {
                reason: "max_css_rules exceeded",
            });
        }

        let mut selectors = Vec::new();
        for raw_sel in selector_prelude.split(',') {
            let trimmed = raw_sel.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(sel) = parse_selector(trimmed, budget.max_selector_combinators)? {
                selectors.push(sel);
            }
        }

        let declarations = parse_declarations(decl_block);
        if !selectors.is_empty() && !declarations.is_empty() {
            rules.push(CssRule {
                selectors,
                declarations,
            });
        }
    }

    Ok(Stylesheet { rules })
}

fn strip_css_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];
        if let Some(end) = after_start.find("*/") {
            rest = &after_start[end + 2..];
        } else {
            return out;
        }
    }
    out.push_str(rest);
    out
}

fn parse_selector(raw: &str, max_combinators: usize) -> Result<Option<Selector>, WebViewError> {
    let normalized = raw.replace('>', " > ");
    let tokens: Vec<&str> = normalized.split_ascii_whitespace().collect();
    if tokens.is_empty() {
        return Ok(None);
    }

    let mut idx = 0;
    let Some(head) = parse_compound_selector(tokens[idx]) else {
        return Ok(None);
    };
    idx += 1;

    let mut chain = Vec::new();
    while idx < tokens.len() {
        let combinator = if tokens[idx] == ">" {
            idx += 1;
            if idx >= tokens.len() {
                return Ok(None);
            }
            Combinator::Child
        } else {
            Combinator::Descendant
        };
        if chain.len() >= max_combinators {
            return Err(WebViewError::CssLimitExceeded {
                reason: "max_selector_combinators exceeded",
            });
        }
        let Some(compound) = parse_compound_selector(tokens[idx]) else {
            return Ok(None);
        };
        chain.push((combinator, compound));
        idx += 1;
    }

    let mut spec = specificity_of(&head);
    for (_, comp) in &chain {
        let s = specificity_of(comp);
        spec.0 = spec.0.saturating_add(s.0);
        spec.1 = spec.1.saturating_add(s.1);
        spec.2 = spec.2.saturating_add(s.2);
    }

    Ok(Some(Selector {
        head,
        chain,
        specificity: spec,
    }))
}

fn specificity_of(comp: &CompoundSelector) -> Specificity {
    let ids = u16::from(comp.id.is_some());
    let classes = (comp.classes.len() + comp.pseudos.len()) as u16;
    let tags = u16::from(comp.tag.is_some());
    Specificity(ids, classes, tags)
}

fn parse_compound_selector(token: &str) -> Option<CompoundSelector> {
    if token.is_empty() {
        return None;
    }
    let mut comp = CompoundSelector::default();
    let bytes = token.as_bytes();
    let mut idx = 0;

    while idx < bytes.len() {
        match bytes[idx] {
            b'*' => {
                idx += 1;
            }
            b'#' => {
                idx += 1;
                let start = idx;
                while idx < bytes.len() && is_ident_char(bytes[idx]) {
                    idx += 1;
                }
                if idx == start {
                    return None;
                }
                comp.id = Some(token[start..idx].to_owned());
            }
            b'.' => {
                idx += 1;
                let start = idx;
                while idx < bytes.len() && is_ident_char(bytes[idx]) {
                    idx += 1;
                }
                if idx == start {
                    return None;
                }
                comp.classes.push(token[start..idx].to_owned());
            }
            b':' => {
                idx += 1;
                let start = idx;
                while idx < bytes.len() && is_ident_char(bytes[idx]) {
                    idx += 1;
                }
                let pseudo_name = token[start..idx].to_ascii_lowercase();
                let pseudo = match pseudo_name.as_str() {
                    "hover" => PseudoClass::Hover,
                    "focus" => PseudoClass::Focus,
                    "active" => PseudoClass::Active,
                    "disabled" => PseudoClass::Disabled,
                    "checked" => PseudoClass::Checked,
                    _ => return None,
                };
                comp.pseudos.push(pseudo);
            }
            _ if is_ident_char(bytes[idx]) => {
                let start = idx;
                while idx < bytes.len() && is_ident_char(bytes[idx]) {
                    idx += 1;
                }
                comp.tag = Some(token[start..idx].to_ascii_lowercase());
            }
            _ => return None,
        }
    }
    Some(comp)
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn parse_declarations(block: &str) -> Vec<Declaration> {
    let mut decls = Vec::new();
    for part in block.split(';') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((prop, val)) = trimmed.split_once(':') else {
            continue;
        };
        let prop_name = prop.trim().to_ascii_lowercase();
        let prop_val = val.trim().trim_end_matches("!important").trim().to_owned();
        if !prop_name.is_empty() && !prop_val.is_empty() {
            decls.push(Declaration {
                property: prop_name,
                value: prop_val,
            });
        }
    }
    decls
}

/// Checks if `selector` matches `node_id` in `dom`.
pub fn selector_matches(dom: &DomArena, node_id: NodeId, selector: &Selector) -> bool {
    let all_compounds: Vec<&CompoundSelector> = std::iter::once(&selector.head)
        .chain(selector.chain.iter().map(|(_, c)| c))
        .collect();
    let Some(&last_compound) = all_compounds.last() else {
        return false;
    };
    if !compound_matches_node(dom, node_id, last_compound) {
        return false;
    }

    let mut current_id = node_id;
    for step_idx in (0..selector.chain.len()).rev() {
        let (combinator, _) = selector.chain[step_idx];
        let target_compound = all_compounds[step_idx];
        match combinator {
            Combinator::Child => {
                let Some(parent_id) = dom.get(current_id).and_then(|n| n.parent) else {
                    return false;
                };
                if !compound_matches_node(dom, parent_id, target_compound) {
                    return false;
                }
                current_id = parent_id;
            }
            Combinator::Descendant => {
                let mut ancestor_opt = dom.get(current_id).and_then(|n| n.parent);
                let mut found = None;
                while let Some(anc_id) = ancestor_opt {
                    if compound_matches_node(dom, anc_id, target_compound) {
                        found = Some(anc_id);
                        break;
                    }
                    ancestor_opt = dom.get(anc_id).and_then(|n| n.parent);
                }
                let Some(matched_anc) = found else {
                    return false;
                };
                current_id = matched_anc;
            }
        }
    }
    true
}

fn compound_matches_node(dom: &DomArena, node_id: NodeId, comp: &CompoundSelector) -> bool {
    let Some(node) = dom.get(node_id) else {
        return false;
    };
    let NodeKind::Element(elem) = &node.kind else {
        return false;
    };
    compound_matches_element(elem, comp)
}

fn compound_matches_element(elem: &ElementData, comp: &CompoundSelector) -> bool {
    if let Some(tag) = &comp.tag {
        if &elem.tag_name != tag {
            return false;
        }
    }
    if let Some(id) = &comp.id {
        if elem.id() != Some(id.as_str()) {
            return false;
        }
    }
    for cls in &comp.classes {
        if !elem.has_class(cls) {
            return false;
        }
    }
    for pseudo in &comp.pseudos {
        let ok = match pseudo {
            PseudoClass::Hover => elem.is_hovered,
            PseudoClass::Focus => elem.is_focused,
            PseudoClass::Active => elem.is_active,
            PseudoClass::Disabled => elem.is_disabled(),
            PseudoClass::Checked => elem.is_checked(),
        };
        if !ok {
            return false;
        }
    }
    true
}

/// Computes `ComputedStyle` for every node in `dom` (indexed by `NodeId.0`).
pub fn compute_styles(dom: &DomArena, stylesheets: &[Stylesheet]) -> Vec<ComputedStyle> {
    let mut styles = vec![ComputedStyle::default(); dom.node_count()];
    compute_node_style_recursive(
        dom,
        dom.root(),
        &ComputedStyle::default(),
        stylesheets,
        &mut styles,
    );
    styles
}

fn compute_node_style_recursive(
    dom: &DomArena,
    node_id: NodeId,
    parent_style: &ComputedStyle,
    stylesheets: &[Stylesheet],
    out: &mut [ComputedStyle],
) {
    let Some(node) = dom.get(node_id) else {
        return;
    };

    let mut style = match &node.kind {
        NodeKind::Document => ComputedStyle::default(),
        NodeKind::Text(_) | NodeKind::Comment(_) => {
            let mut s = ComputedStyle::inherit_from(parent_style, "#text");
            s.display = DisplayMode::Inline;
            s
        }
        NodeKind::Element(elem) => {
            let mut s = ComputedStyle::inherit_from(parent_style, &elem.tag_name);
            let mut matched_rules: Vec<(Specificity, usize, &[Declaration])> = Vec::new();
            let mut order_counter = 0usize;

            for sheet in stylesheets {
                for rule in &sheet.rules {
                    for sel in &rule.selectors {
                        order_counter += 1;
                        if selector_matches(dom, node_id, sel) {
                            matched_rules.push((
                                sel.specificity,
                                order_counter,
                                &rule.declarations,
                            ));
                        }
                    }
                }
            }

            matched_rules.sort_by_key(|&(spec, ord, _)| (spec, ord));
            for (_, _, decls) in matched_rules {
                for decl in decls {
                    apply_declaration(&mut s, &decl.property, &decl.value);
                }
            }

            if let Some(inline_css) = elem.get_attribute("style") {
                for decl in parse_declarations(inline_css) {
                    apply_declaration(&mut s, &decl.property, &decl.value);
                }
            }
            s
        }
    };

    if !style.font_size.is_finite() || style.font_size <= 0.0 {
        style.font_size = 16.0;
    }

    out[node_id.0] = style.clone();
    for &child_id in &node.children {
        compute_node_style_recursive(dom, child_id, &style, stylesheets, out);
    }
}

fn apply_declaration(style: &mut ComputedStyle, prop: &str, val: &str) {
    let v = val.trim();
    match prop {
        "display" => match v.to_ascii_lowercase().as_str() {
            "block" => style.display = DisplayMode::Block,
            "inline" => style.display = DisplayMode::Inline,
            "inline-block" => style.display = DisplayMode::InlineBlock,
            "flex" => style.display = DisplayMode::Flex,
            "table" => style.display = DisplayMode::Table,
            "table-row" => style.display = DisplayMode::TableRow,
            "table-cell" => style.display = DisplayMode::TableCell,
            "none" => style.display = DisplayMode::None,
            _ => {}
        },
        "width" => style.width = parse_length_px(v, style.font_size),
        "height" => style.height = parse_length_px(v, style.font_size),
        "min-width" => style.min_width = parse_length_px(v, style.font_size),
        "max-width" => style.max_width = parse_length_px(v, style.font_size),
        "min-height" => style.min_height = parse_length_px(v, style.font_size),
        "margin" => {
            if let Some(edges) = parse_edge_shorthand(v, style.font_size) {
                style.margin = edges;
            }
        }
        "margin-top" => {
            if let Some(px) = parse_length_px(v, style.font_size) {
                style.margin.top = px;
            }
        }
        "margin-right" => {
            if let Some(px) = parse_length_px(v, style.font_size) {
                style.margin.right = px;
            }
        }
        "margin-bottom" => {
            if let Some(px) = parse_length_px(v, style.font_size) {
                style.margin.bottom = px;
            }
        }
        "margin-left" => {
            if let Some(px) = parse_length_px(v, style.font_size) {
                style.margin.left = px;
            }
        }
        "padding" => {
            if let Some(edges) = parse_edge_shorthand(v, style.font_size) {
                style.padding = edges;
            }
        }
        "padding-top" => {
            if let Some(px) = parse_length_px(v, style.font_size) {
                style.padding.top = px.max(0.0);
            }
        }
        "padding-right" => {
            if let Some(px) = parse_length_px(v, style.font_size) {
                style.padding.right = px.max(0.0);
            }
        }
        "padding-bottom" => {
            if let Some(px) = parse_length_px(v, style.font_size) {
                style.padding.bottom = px.max(0.0);
            }
        }
        "padding-left" => {
            if let Some(px) = parse_length_px(v, style.font_size) {
                style.padding.left = px.max(0.0);
            }
        }
        "border-width" => {
            if let Some(edges) = parse_edge_shorthand(v, style.font_size) {
                style.border_width = edges;
            }
        }
        "border" => {
            for token in v.split_ascii_whitespace() {
                if let Some(px) = parse_length_px(token, style.font_size) {
                    style.border_width = EdgeSizes::all(px.max(0.0));
                } else if let Some(c) = parse_color(token) {
                    style.border_color = c;
                }
            }
        }
        "border-color" => {
            if let Some(c) = parse_color(v) {
                style.border_color = c;
            }
        }
        "border-radius" => {
            if let Some(px) = parse_length_px(v, style.font_size) {
                style.border_radius = px.max(0.0);
            }
        }
        "color" => {
            if let Some(c) = parse_color(v) {
                style.color = c;
            }
        }
        "background-color" | "background" => {
            if let Some(c) = parse_color(v) {
                style.background_color = c;
            }
        }
        "font-size" => {
            if let Some(px) = parse_length_px(v, style.font_size) {
                if px > 0.0 {
                    style.font_size = px;
                }
            }
        }
        "font-weight" => match v.to_ascii_lowercase().as_str() {
            "normal" => style.font_weight = 400,
            "bold" => style.font_weight = 700,
            other => {
                if let Ok(w) = other.parse::<u16>() {
                    if (100..=900).contains(&w) {
                        style.font_weight = w;
                    }
                }
            }
        },
        "font-style" => match v.to_ascii_lowercase().as_str() {
            "italic" | "oblique" => style.italic = true,
            "normal" => style.italic = false,
            _ => {}
        },
        "font-family" => {
            if v.to_ascii_lowercase().contains("monospace") {
                style.monospace = true;
            }
        }
        "text-align" => match v.to_ascii_lowercase().as_str() {
            "left" => style.text_align = TextAlign::Left,
            "center" => style.text_align = TextAlign::Center,
            "right" => style.text_align = TextAlign::Right,
            _ => {}
        },
        "text-decoration" => match v.to_ascii_lowercase().as_str() {
            "none" => style.text_decoration = TextDecoration::None,
            "underline" => style.text_decoration = TextDecoration::Underline,
            "line-through" => style.text_decoration = TextDecoration::LineThrough,
            _ => {}
        },
        "line-height" => {
            if let Ok(mult) = v.parse::<f64>() {
                if mult.is_finite() && mult > 0.0 {
                    style.line_height = mult;
                }
            } else if let Some(px) = parse_length_px(v, style.font_size) {
                if px > 0.0 && style.font_size > 0.0 {
                    style.line_height = px / style.font_size;
                }
            }
        }
        "flex-direction" => match v.to_ascii_lowercase().as_str() {
            "row" => style.flex_direction = FlexDirection::Row,
            "column" => style.flex_direction = FlexDirection::Column,
            _ => {}
        },
        "justify-content" => match v.to_ascii_lowercase().as_str() {
            "flex-start" | "start" => style.justify_content = JustifyContent::FlexStart,
            "center" => style.justify_content = JustifyContent::Center,
            "flex-end" | "end" => style.justify_content = JustifyContent::FlexEnd,
            "space-between" => style.justify_content = JustifyContent::SpaceBetween,
            _ => {}
        },
        "align-items" => match v.to_ascii_lowercase().as_str() {
            "stretch" => style.align_items = AlignItems::Stretch,
            "flex-start" | "start" => style.align_items = AlignItems::FlexStart,
            "center" => style.align_items = AlignItems::Center,
            "flex-end" | "end" => style.align_items = AlignItems::FlexEnd,
            _ => {}
        },
        "flex-grow" | "flex" => {
            if let Some(first) = v.split_ascii_whitespace().next() {
                if let Ok(grow) = first.parse::<f64>() {
                    if grow.is_finite() && grow >= 0.0 {
                        style.flex_grow = grow;
                    }
                }
            }
        }
        "gap" => {
            if let Some(px) = parse_length_px(v, style.font_size) {
                style.gap = px.max(0.0);
            }
        }
        _ => {}
    }
}

fn parse_length_px(raw: &str, base_font_size: f64) -> Option<f64> {
    let s = raw.trim().to_ascii_lowercase();
    if s == "0" {
        return Some(0.0);
    }
    let val = if let Some(num) = s.strip_suffix("px") {
        num.trim().parse::<f64>().ok()?
    } else if let Some(num) = s.strip_suffix("rem") {
        num.trim().parse::<f64>().ok()? * 16.0
    } else if let Some(num) = s.strip_suffix("em") {
        num.trim().parse::<f64>().ok()? * base_font_size
    } else {
        s.parse::<f64>().ok()?
    };
    if val.is_finite() { Some(val) } else { None }
}

fn parse_edge_shorthand(raw: &str, base_font_size: f64) -> Option<EdgeSizes> {
    let mut vals = Vec::new();
    for part in raw.split_ascii_whitespace() {
        vals.push(parse_length_px(part, base_font_size)?);
    }
    match vals.as_slice() {
        [all] => Some(EdgeSizes::all(*all)),
        [v, h] => Some(EdgeSizes::symmetric(*v, *h)),
        [t, h, b] => Some(EdgeSizes {
            top: *t,
            right: *h,
            bottom: *b,
            left: *h,
        }),
        [t, r, b, l, ..] => Some(EdgeSizes {
            top: *t,
            right: *r,
            bottom: *b,
            left: *l,
        }),
        [] => None,
    }
}

/// Parses CSS hex (`#rgb`, `#rrggbb`, `#rrggbbaa`), `rgb(...)`, `rgba(...)`, or named colors.
pub fn parse_color(raw: &str) -> Option<ColorRgba> {
    let s = raw.trim().to_ascii_lowercase();
    if let Some(hex) = s.strip_prefix('#') {
        return match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some(ColorRgba::new(r, g, b, 255))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(ColorRgba::new(r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(ColorRgba::new(r, g, b, a))
            }
            _ => None,
        };
    }

    if let Some(inner) = s
        .strip_prefix("rgb(")
        .or_else(|| s.strip_prefix("rgba("))
        .and_then(|v| v.strip_suffix(')'))
    {
        let comps: Vec<&str> = inner.split(',').map(str::trim).collect();
        if comps.len() >= 3 {
            let r = comps[0].parse::<u8>().ok()?;
            let g = comps[1].parse::<u8>().ok()?;
            let b = comps[2].parse::<u8>().ok()?;
            let a = if comps.len() >= 4 {
                if let Ok(af) = comps[3].parse::<f64>() {
                    if af <= 1.0 {
                        (af.clamp(0.0, 1.0) * 255.0).round() as u8
                    } else {
                        af.clamp(0.0, 255.0).round() as u8
                    }
                } else {
                    255
                }
            } else {
                255
            };
            return Some(ColorRgba::new(r, g, b, a));
        }
    }

    match s.as_str() {
        "transparent" => Some(ColorRgba::TRANSPARENT),
        "black" => Some(ColorRgba::BLACK),
        "white" => Some(ColorRgba::WHITE),
        "red" => Some(ColorRgba::new(255, 0, 0, 255)),
        "green" => Some(ColorRgba::new(0, 128, 0, 255)),
        "blue" => Some(ColorRgba::new(0, 0, 255, 255)),
        "gray" | "grey" => Some(ColorRgba::new(128, 128, 128, 255)),
        "yellow" => Some(ColorRgba::new(255, 255, 0, 255)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::DomNode;
    use crate::html::parse_html;

    #[test]
    fn cascades_specificity_combinators_pseudo_classes_and_inline_styles() {
        let html = r#"
            <div id="container" class="wrapper">
                <p id="target" class="highlight" style="padding: 12px">Text</p>
            </div>
        "#;
        let css = r#"
            p { color: #111111; font-size: 14px; }
            .wrapper > p.highlight { color: #222222; font-size: 18px; }
            #target { color: #333333; }
            #target:hover { color: #ff0000; }
        "#;
        let mut doc = parse_html(html, ResourceBudget::default()).unwrap();
        let sheet = parse_stylesheet(css, ResourceBudget::default()).unwrap();
        let target_id = doc.dom.get_element_by_id("target").unwrap();

        let styles = compute_styles(&doc.dom, std::slice::from_ref(&sheet));
        assert_eq!(
            styles[target_id.0].color,
            ColorRgba::new(0x33, 0x33, 0x33, 255)
        );
        assert_eq!(styles[target_id.0].font_size, 18.0);
        assert_eq!(styles[target_id.0].padding, EdgeSizes::all(12.0));

        if let Some(DomNode {
            kind: NodeKind::Element(elem),
            ..
        }) = doc.dom.get_mut(target_id)
        {
            elem.is_hovered = true;
        }
        let hovered_styles = compute_styles(&doc.dom, &[sheet]);
        assert_eq!(
            hovered_styles[target_id.0].color,
            ColorRgba::new(255, 0, 0, 255)
        );
    }
}

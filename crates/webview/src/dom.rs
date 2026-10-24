//! DOM Arena, Node model, URI policy, and Resource Budget guard (`UNIT-WEBVIEW-001`).

use std::fmt;

/// Deterministic resource budget enforced across parsers, DOM arena, CSS cascade, and Script VM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceBudget {
    /// Maximum UTF-8 byte length of a single document or stylesheet source (default: 1 MiB).
    pub max_source_bytes: usize,
    /// Maximum number of DOM nodes in a single `DomArena` (default: 16,384).
    pub max_nodes: usize,
    /// Maximum tree nesting depth in a `DomArena` (default: 128).
    pub max_depth: usize,
    /// Maximum number of attributes on a single element node (default: 64).
    pub max_attributes_per_element: usize,
    /// Maximum number of CSS rules across author stylesheets (default: 4,096).
    pub max_css_rules: usize,
    /// Maximum number of combinator segments in a single CSS selector (default: 16).
    pub max_selector_combinators: usize,
    /// Maximum bytecode instruction steps per script execution (default: 100,000).
    pub max_script_steps: usize,
    /// Maximum call stack depth in the Script VM (default: 64).
    pub max_call_depth: usize,
    /// Maximum heap allocation budget in bytes for the Script VM (default: 4 MiB).
    pub max_heap_bytes: usize,
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self {
            max_source_bytes: 1024 * 1024,
            max_nodes: 16_384,
            max_depth: 128,
            max_attributes_per_element: 64,
            max_css_rules: 4_096,
            max_selector_combinators: 16,
            max_script_steps: 100_000,
            max_call_depth: 64,
            max_heap_bytes: 4 * 1024 * 1024,
        }
    }
}

/// Errors produced by the WebView engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebViewError {
    /// Input bytes exceeded `max_source_bytes`.
    SourceBytesLimitExceeded { actual: usize, max: usize },
    /// DOM node count exceeded `max_nodes`.
    DomNodeLimitExceeded { max: usize },
    /// DOM nesting depth exceeded `max_depth`.
    DomDepthLimitExceeded { max: usize },
    /// Element attribute count exceeded `max_attributes_per_element`.
    AttributeLimitExceeded { max: usize },
    /// CSS rule count or selector combinator depth exceeded budget.
    CssLimitExceeded { reason: &'static str },
    /// Invalid node identifier or detached/cross-arena operation.
    InvalidNodeId(NodeId),
    /// Disallowed or external URI scheme under offline-by-default policy (`SD-WV-06`).
    DisallowedUriScheme(String),
    /// Syntax error during CSS or Script parsing.
    SyntaxError(String),
    /// Script VM exceeded its per-execution instruction fuel (`SD-WV-05`).
    ScriptStepLimitExceeded { max_steps: usize },
    /// Script VM exceeded its call-stack depth (`SD-WV-05`).
    ScriptStackOverflow { max_depth: usize },
    /// Script VM exceeded its heap memory budget (`SD-WV-05`).
    ScriptHeapLimitExceeded { max_bytes: usize },
    /// Runtime error inside the Script VM.
    ScriptRuntimeError(String),
    /// Attempted to access a ViewModel Property not exposed on the WebView whitelist (`SD-WV-07`).
    UnexposedProperty(String),
    /// Attempted to write to a read-only exposed ViewModel Property.
    ReadOnlyProperty(String),
    /// Attempted to invoke a Command not exposed on the WebView whitelist (`SD-WV-07`).
    UnexposedCommand(String),
    /// Invalid or non-finite layout constraint (`RQ-09-067`, `RQ-17-005`).
    InvalidLayoutConstraint,
}

impl fmt::Display for WebViewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceBytesLimitExceeded { actual, max } => {
                write!(f, "source byte length {actual} exceeds limit {max}")
            }
            Self::DomNodeLimitExceeded { max } => {
                write!(f, "DOM node count exceeds limit {max}")
            }
            Self::DomDepthLimitExceeded { max } => {
                write!(f, "DOM tree depth exceeds limit {max}")
            }
            Self::AttributeLimitExceeded { max } => {
                write!(f, "element attribute count exceeds limit {max}")
            }
            Self::CssLimitExceeded { reason } => {
                write!(f, "CSS resource limit exceeded: {reason}")
            }
            Self::InvalidNodeId(id) => write!(f, "invalid DOM node id {}", id.0),
            Self::DisallowedUriScheme(uri) => {
                write!(f, "disallowed URI under offline-by-default policy: {uri}")
            }
            Self::SyntaxError(msg) => write!(f, "syntax error: {msg}"),
            Self::ScriptStepLimitExceeded { max_steps } => {
                write!(f, "script step limit ({max_steps}) exceeded")
            }
            Self::ScriptStackOverflow { max_depth } => {
                write!(f, "script call stack depth limit ({max_depth}) exceeded")
            }
            Self::ScriptHeapLimitExceeded { max_bytes } => {
                write!(f, "script heap memory limit ({max_bytes} bytes) exceeded")
            }
            Self::ScriptRuntimeError(msg) => write!(f, "script runtime error: {msg}"),
            Self::UnexposedProperty(name) => {
                write!(f, "property '{name}' is not exposed to WebView")
            }
            Self::ReadOnlyProperty(name) => {
                write!(f, "property '{name}' is exposed as read-only")
            }
            Self::UnexposedCommand(name) => {
                write!(f, "command '{name}' is not exposed to WebView")
            }
            Self::InvalidLayoutConstraint => write!(f, "invalid or non-finite layout constraint"),
        }
    }
}

impl std::error::Error for WebViewError {}

/// Document source format supported by the WebView engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentFormat {
    Html,
    Markdown,
}

/// Stable index of a node inside a `DomArena`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Kind of DOM node stored in `DomArena`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
    /// Document root node (always `NodeId(0)`).
    Document,
    /// HTML/Markdown element node (`h1`, `p`, `div`, `a`, `input`, etc.).
    Element(ElementData),
    /// Text leaf node.
    Text(String),
    /// Comment node.
    Comment(String),
}

/// Attribute key-value pair on an element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

/// Element-specific payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementData {
    /// Lowercase normalized tag name (`div`, `p`, `h1`, `a`, `input`, `table`, etc.).
    pub tag_name: String,
    /// Ordered list of element attributes.
    pub attributes: Vec<Attribute>,
    /// Interactive state flags used by pseudo-class matching (`:hover`, `:focus`, `:active`, `:disabled`, `:checked`).
    pub is_hovered: bool,
    pub is_focused: bool,
    pub is_active: bool,
}

impl ElementData {
    pub fn new(tag_name: impl Into<String>) -> Self {
        Self {
            tag_name: tag_name.into().to_ascii_lowercase(),
            attributes: Vec::new(),
            is_hovered: false,
            is_focused: false,
            is_active: false,
        }
    }

    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        let lower = name.to_ascii_lowercase();
        self.attributes
            .iter()
            .find(|attr| attr.name == lower)
            .map(|attr| attr.value.as_str())
    }

    pub fn id(&self) -> Option<&str> {
        self.get_attribute("id")
    }

    pub fn classes(&self) -> impl Iterator<Item = &str> {
        self.get_attribute("class")
            .unwrap_or("")
            .split_ascii_whitespace()
    }

    pub fn has_class(&self, class_name: &str) -> bool {
        self.classes().any(|c| c == class_name)
    }

    pub fn is_disabled(&self) -> bool {
        self.get_attribute("disabled").is_some()
    }

    pub fn is_checked(&self) -> bool {
        self.get_attribute("checked").is_some()
    }
}

/// A single node inside `DomArena`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub depth: usize,
    pub kind: NodeKind,
}

/// Flat arena-allocated DOM tree with deterministic resource budget enforcement (`SD-WV-02`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomArena {
    nodes: Vec<DomNode>,
    budget: ResourceBudget,
}

impl DomArena {
    /// Creates a new `DomArena` containing the root `Document` node at `NodeId(0)`.
    pub fn new(budget: ResourceBudget) -> Self {
        let root = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            depth: 0,
            kind: NodeKind::Document,
        };
        Self {
            nodes: vec![root],
            budget,
        }
    }

    /// Returns the root `Document` node ID (`NodeId(0)`).
    pub const fn root(&self) -> NodeId {
        NodeId(0)
    }

    /// Returns the active resource budget.
    pub const fn budget(&self) -> ResourceBudget {
        self.budget
    }

    /// Returns the total number of allocated nodes (including the Document root).
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Validates that source UTF-8 length does not exceed `budget.max_source_bytes` (`SD-WV-01`).
    pub fn check_source_bytes(&self, source: &str) -> Result<(), WebViewError> {
        if source.len() > self.budget.max_source_bytes {
            return Err(WebViewError::SourceBytesLimitExceeded {
                actual: source.len(),
                max: self.budget.max_source_bytes,
            });
        }
        Ok(())
    }

    /// Looks up a node by `NodeId`.
    pub fn get(&self, id: NodeId) -> Option<&DomNode> {
        self.nodes.get(id.0)
    }

    /// Looks up a mutable node by `NodeId`.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut DomNode> {
        self.nodes.get_mut(id.0)
    }

    /// Returns all nodes in arena allocation order.
    pub fn nodes(&self) -> &[DomNode] {
        &self.nodes
    }

    /// Allocates and appends an element node as a child of `parent`.
    pub fn append_element(
        &mut self,
        parent: NodeId,
        tag_name: impl Into<String>,
    ) -> Result<NodeId, WebViewError> {
        self.append_node(parent, NodeKind::Element(ElementData::new(tag_name)))
    }

    /// Allocates and appends a text node as a child of `parent`.
    pub fn append_text(
        &mut self,
        parent: NodeId,
        text: impl Into<String>,
    ) -> Result<NodeId, WebViewError> {
        self.append_node(parent, NodeKind::Text(text.into()))
    }

    /// Allocates and appends a comment node as a child of `parent`.
    pub fn append_comment(
        &mut self,
        parent: NodeId,
        comment: impl Into<String>,
    ) -> Result<NodeId, WebViewError> {
        self.append_node(parent, NodeKind::Comment(comment.into()))
    }

    fn append_node(&mut self, parent: NodeId, kind: NodeKind) -> Result<NodeId, WebViewError> {
        if self.nodes.len() >= self.budget.max_nodes {
            return Err(WebViewError::DomNodeLimitExceeded {
                max: self.budget.max_nodes,
            });
        }
        let parent_depth = self
            .nodes
            .get(parent.0)
            .ok_or(WebViewError::InvalidNodeId(parent))?
            .depth;
        let child_depth = parent_depth + 1;
        if child_depth > self.budget.max_depth {
            return Err(WebViewError::DomDepthLimitExceeded {
                max: self.budget.max_depth,
            });
        }
        let new_id = NodeId(self.nodes.len());
        self.nodes.push(DomNode {
            id: new_id,
            parent: Some(parent),
            children: Vec::new(),
            depth: child_depth,
            kind,
        });
        self.nodes[parent.0].children.push(new_id);
        Ok(new_id)
    }

    /// Sets an attribute on an element node, enforcing `max_attributes_per_element` and URI policy on `href`/`src`.
    pub fn set_attribute(
        &mut self,
        node_id: NodeId,
        name: impl AsRef<str>,
        value: impl Into<String>,
    ) -> Result<(), WebViewError> {
        let attr_name = name.as_ref().to_ascii_lowercase();
        let attr_value = value.into();

        if matches!(attr_name.as_str(), "href" | "src" | "action") {
            validate_uri_scheme(&attr_value)?;
        }

        let max_attrs = self.budget.max_attributes_per_element;
        let node = self
            .nodes
            .get_mut(node_id.0)
            .ok_or(WebViewError::InvalidNodeId(node_id))?;
        let NodeKind::Element(elem) = &mut node.kind else {
            return Err(WebViewError::InvalidNodeId(node_id));
        };

        if let Some(existing) = elem.attributes.iter_mut().find(|a| a.name == attr_name) {
            existing.value = attr_value;
            return Ok(());
        }
        if elem.attributes.len() >= max_attrs {
            return Err(WebViewError::AttributeLimitExceeded { max: max_attrs });
        }
        elem.attributes.push(Attribute {
            name: attr_name,
            value: attr_value,
        });
        Ok(())
    }

    /// Reads an attribute from an element node.
    pub fn get_attribute(&self, node_id: NodeId, name: &str) -> Option<&str> {
        let node = self.nodes.get(node_id.0)?;
        if let NodeKind::Element(elem) = &node.kind {
            elem.get_attribute(name)
        } else {
            None
        }
    }

    /// Adds a CSS class to an element node.
    pub fn add_class(&mut self, node_id: NodeId, class_name: &str) -> Result<(), WebViewError> {
        let trimmed = class_name.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        let existing = self
            .get_attribute(node_id, "class")
            .unwrap_or("")
            .to_owned();
        if existing.split_ascii_whitespace().any(|c| c == trimmed) {
            return Ok(());
        }
        let updated = if existing.is_empty() {
            trimmed.to_owned()
        } else {
            format!("{existing} {trimmed}")
        };
        self.set_attribute(node_id, "class", updated)
    }

    /// Removes a CSS class from an element node.
    pub fn remove_class(&mut self, node_id: NodeId, class_name: &str) -> Result<(), WebViewError> {
        let trimmed = class_name.trim();
        let existing = self
            .get_attribute(node_id, "class")
            .unwrap_or("")
            .to_owned();
        let filtered: Vec<&str> = existing
            .split_ascii_whitespace()
            .filter(|&c| c != trimmed)
            .collect();
        self.set_attribute(node_id, "class", filtered.join(" "))
    }

    /// Finds the first element node with the given `id` attribute in document pre-order.
    pub fn get_element_by_id(&self, target_id: &str) -> Option<NodeId> {
        self.find_in_subtree(self.root(), &|node| {
            if let NodeKind::Element(elem) = &node.kind {
                elem.id() == Some(target_id)
            } else {
                false
            }
        })
    }

    /// Finds the first node in pre-order matching `predicate`.
    pub fn find_in_subtree(
        &self,
        start: NodeId,
        predicate: &dyn Fn(&DomNode) -> bool,
    ) -> Option<NodeId> {
        let node = self.nodes.get(start.0)?;
        if predicate(node) {
            return Some(start);
        }
        for &child in &node.children {
            if let Some(found) = self.find_in_subtree(child, predicate) {
                return Some(found);
            }
        }
        None
    }

    /// Concatenates all descendant text node contents of `node_id`.
    pub fn text_content(&self, node_id: NodeId) -> String {
        let mut out = String::new();
        self.collect_text(node_id, &mut out);
        out
    }

    fn collect_text(&self, node_id: NodeId, out: &mut String) {
        let Some(node) = self.nodes.get(node_id.0) else {
            return;
        };
        match &node.kind {
            NodeKind::Text(t) => out.push_str(t),
            NodeKind::Document | NodeKind::Element(_) => {
                for &child in &node.children {
                    self.collect_text(child, out);
                }
            }
            NodeKind::Comment(_) => {}
        }
    }

    /// Replaces the children of `node_id` with a single text node containing `new_text`.
    pub fn set_text_content(
        &mut self,
        node_id: NodeId,
        new_text: impl Into<String>,
    ) -> Result<(), WebViewError> {
        let text = new_text.into();
        let existing_children = self
            .nodes
            .get(node_id.0)
            .ok_or(WebViewError::InvalidNodeId(node_id))?
            .children
            .clone();

        if existing_children.len() == 1 {
            let child_id = existing_children[0];
            if let Some(DomNode {
                kind: NodeKind::Text(slot),
                ..
            }) = self.nodes.get_mut(child_id.0)
            {
                *slot = text;
                return Ok(());
            }
        }

        for &child_id in &existing_children {
            if let Some(child) = self.nodes.get_mut(child_id.0) {
                child.parent = None;
            }
        }
        self.nodes[node_id.0].children.clear();
        if !text.is_empty() {
            self.append_text(node_id, text)?;
        }
        Ok(())
    }
}

/// Validates a URI against the offline-by-default policy (`SD-WV-06`, `RQ-17-010`).
///
/// Allowed:
/// - Fragment identifiers (`#section`)
/// - Relative paths (`page.html`, `/docs/intro.md`, `./asset.png`) without `..` traversal
/// - Named FreeMVVM command/resource URIs (`command:save`, `mvvm:action`, `asset:logo`)
///
/// Disallowed:
/// - External network schemes (`http:`, `https:`, `ws:`, `wss:`, `ftp:`)
/// - Host filesystem or code execution schemes (`file:`, `javascript:`, `vbscript:`, `data:`)
pub fn validate_uri_scheme(uri: &str) -> Result<(), WebViewError> {
    let trimmed = uri.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(());
    }
    if let Some((scheme, rest)) = trimmed.split_once(':') {
        let scheme_lower = scheme.to_ascii_lowercase();
        if scheme_lower
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
        {
            return match scheme_lower.as_str() {
                "command" | "mvvm" | "asset" => {
                    if rest.contains("..") {
                        Err(WebViewError::DisallowedUriScheme(trimmed.to_owned()))
                    } else {
                        Ok(())
                    }
                }
                _ => Err(WebViewError::DisallowedUriScheme(trimmed.to_owned())),
            };
        }
    }
    if trimmed.starts_with("//") || trimmed.split('/').any(|seg| seg == "..") {
        return Err(WebViewError::DisallowedUriScheme(trimmed.to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_dom_tree_and_queries_elements_by_id_and_text_content() {
        let mut dom = DomArena::new(ResourceBudget::default());
        let div = dom.append_element(dom.root(), "DIV").unwrap();
        dom.set_attribute(div, "id", "main-box").unwrap();
        dom.add_class(div, "card").unwrap();
        dom.add_class(div, "highlighted").unwrap();

        let h1 = dom.append_element(div, "h1").unwrap();
        dom.append_text(h1, "Hello ").unwrap();
        let span = dom.append_element(h1, "span").unwrap();
        dom.append_text(span, "FreeMVVM").unwrap();

        assert_eq!(dom.get_element_by_id("main-box"), Some(div));
        assert_eq!(dom.text_content(div), "Hello FreeMVVM");

        dom.remove_class(div, "card").unwrap();
        assert_eq!(dom.get_attribute(div, "class"), Some("highlighted"));

        dom.set_text_content(h1, "Updated Heading").unwrap();
        assert_eq!(dom.text_content(div), "Updated Heading");
    }

    #[test]
    fn enforces_node_depth_and_attribute_budgets_without_corrupting_arena() {
        let budget = ResourceBudget {
            max_source_bytes: 32,
            max_nodes: 4,
            max_depth: 2,
            max_attributes_per_element: 2,
            ..ResourceBudget::default()
        };
        let mut dom = DomArena::new(budget);

        assert!(matches!(
            dom.check_source_bytes("0123456789abcdefghijklmnopqrstuvwxyz"),
            Err(WebViewError::SourceBytesLimitExceeded { .. })
        ));

        let d1 = dom.append_element(dom.root(), "div").unwrap();
        let d2 = dom.append_element(d1, "p").unwrap();
        assert_eq!(
            dom.append_element(d2, "span"),
            Err(WebViewError::DomDepthLimitExceeded { max: 2 })
        );

        dom.set_attribute(d1, "id", "a").unwrap();
        dom.set_attribute(d1, "class", "b").unwrap();
        assert_eq!(
            dom.set_attribute(d1, "title", "c"),
            Err(WebViewError::AttributeLimitExceeded { max: 2 })
        );

        let _d3 = dom.append_text(d1, "ok").unwrap();
        assert_eq!(
            dom.append_text(d1, "overflow"),
            Err(WebViewError::DomNodeLimitExceeded { max: 4 })
        );
        assert_eq!(dom.node_count(), 4);
    }

    #[test]
    fn enforces_offline_by_default_uri_policy() {
        let mut dom = DomArena::new(ResourceBudget::default());
        let a = dom.append_element(dom.root(), "a").unwrap();

        assert!(dom.set_attribute(a, "href", "#section-1").is_ok());
        assert!(dom.set_attribute(a, "href", "command:submit").is_ok());
        assert!(dom.set_attribute(a, "href", "docs/guide.md").is_ok());

        for forbidden in [
            "http://example.com",
            "https://example.com/script.js",
            "javascript:alert(1)",
            "file:///etc/passwd",
            "data:text/html,<script>",
            "//remote.example.com/x",
            "../secret.txt",
        ] {
            assert!(
                matches!(
                    dom.set_attribute(a, "href", forbidden),
                    Err(WebViewError::DisallowedUriScheme(_))
                ),
                "URI should be rejected: {forbidden}"
            );
        }
    }
}

//! HTML tokenizer and tree builder into `DomArena` (`UNIT-WEBVIEW-003`).

use crate::dom::{DomArena, DomNode, NodeId, NodeKind, ResourceBudget, WebViewError};
use crate::markdown::decode_entities;

/// Result of parsing an HTML document, including the DOM tree and extracted `<style>` and `<script>` blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedHtmlDocument {
    pub dom: DomArena,
    pub stylesheets: Vec<String>,
    pub scripts: Vec<String>,
}

/// Parses an HTML document into a `ParsedHtmlDocument` while enforcing `budget`.
pub fn parse_html(input: &str, budget: ResourceBudget) -> Result<ParsedHtmlDocument, WebViewError> {
    let mut dom = DomArena::new(budget);
    dom.check_source_bytes(input)?;

    let mut stylesheets = Vec::new();
    let mut scripts = Vec::new();
    let mut stack: Vec<NodeId> = vec![dom.root()];

    let mut cursor = 0usize;
    let bytes = input.as_bytes();

    while cursor < bytes.len() {
        let rest = &input[cursor..];
        if let Some(lt_rel) = rest.find('<') {
            if lt_rel > 0 {
                let raw_text = &rest[..lt_rel];
                append_normalized_text(&mut dom, current_parent(&stack), raw_text)?;
            }
            cursor += lt_rel;
            let tag_rest = &input[cursor..];

            // HTML Comment <!-- ... -->
            if let Some(after_open) = tag_rest.strip_prefix("<!--") {
                if let Some(end_rel) = after_open.find("-->") {
                    let comment_body = &after_open[..end_rel];
                    dom.append_comment(current_parent(&stack), comment_body)?;
                    cursor += 4 + end_rel + 3;
                } else {
                    dom.append_comment(current_parent(&stack), after_open)?;
                    break;
                }
                continue;
            }

            // DOCTYPE or declaration <!...>
            if tag_rest.starts_with("<!") || tag_rest.starts_with("<?") {
                if let Some(gt_rel) = tag_rest.find('>') {
                    cursor += gt_rel + 1;
                } else {
                    break;
                }
                continue;
            }

            // Closing tag </tag>
            if let Some(after_slash) = tag_rest.strip_prefix("</") {
                if let Some(gt_rel) = after_slash.find('>') {
                    let close_name = after_slash[..gt_rel].trim().to_ascii_lowercase();
                    pop_matching_tag(&dom, &mut stack, &close_name);
                    cursor += 2 + gt_rel + 1;
                } else {
                    break;
                }
                continue;
            }

            // Start tag <tag attr="val" ...>
            if let Some((tag_name, attrs, self_closing, consumed)) = parse_start_tag(tag_rest) {
                apply_implicit_close(&dom, &mut stack, &tag_name);
                let parent = current_parent(&stack);
                let elem_id = dom.append_element(parent, &tag_name)?;
                for (k, v) in attrs {
                    dom.set_attribute(elem_id, k, decode_entities(&v))?;
                }
                cursor += consumed;

                // Raw text elements: <style> and <script>
                if (tag_name == "style" || tag_name == "script") && !self_closing {
                    let close_marker = if tag_name == "style" {
                        "</style>"
                    } else {
                        "</script>"
                    };
                    let after_tag = &input[cursor..];
                    let raw_end = find_ascii_case_insensitive(after_tag, close_marker);
                    let (raw_body, advance) = match raw_end {
                        Some(pos) => (&after_tag[..pos], pos + close_marker.len()),
                        None => (after_tag, after_tag.len()),
                    };
                    if !raw_body.trim().is_empty() {
                        dom.append_text(elem_id, raw_body)?;
                        if tag_name == "style" {
                            stylesheets.push(raw_body.to_owned());
                        } else {
                            scripts.push(raw_body.to_owned());
                        }
                    }
                    cursor += advance;
                    continue;
                }

                if !self_closing && !is_void_element(&tag_name) {
                    stack.push(elem_id);
                }
                continue;
            }

            // Fallback: literal '<'
            append_normalized_text(&mut dom, current_parent(&stack), "<")?;
            cursor += 1;
        } else {
            append_normalized_text(&mut dom, current_parent(&stack), rest)?;
            break;
        }
    }

    Ok(ParsedHtmlDocument {
        dom,
        stylesheets,
        scripts,
    })
}

fn current_parent(stack: &[NodeId]) -> NodeId {
    stack.last().copied().unwrap_or(NodeId(0))
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn apply_implicit_close(dom: &DomArena, stack: &mut Vec<NodeId>, new_tag: &str) {
    let Some(&top_id) = stack.last() else {
        return;
    };
    let Some(DomNode {
        kind: NodeKind::Element(top_elem),
        ..
    }) = dom.get(top_id)
    else {
        return;
    };
    let open = top_elem.tag_name.as_str();
    let should_close = matches!(
        (open, new_tag),
        (
            "p",
            "p" | "div"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "ul"
                | "ol"
                | "table"
                | "blockquote"
                | "pre"
                | "hr"
        ) | ("li", "li")
            | ("td" | "th", "td" | "th" | "tr")
            | ("tr", "tr")
    );
    if should_close && stack.len() > 1 {
        stack.pop();
    }
}

fn pop_matching_tag(dom: &DomArena, stack: &mut Vec<NodeId>, close_name: &str) {
    for idx in (1..stack.len()).rev() {
        if let Some(DomNode {
            kind: NodeKind::Element(elem),
            ..
        }) = dom.get(stack[idx])
        {
            if elem.tag_name == close_name {
                stack.truncate(idx);
                return;
            }
        }
    }
}

fn append_normalized_text(
    dom: &mut DomArena,
    parent: NodeId,
    raw_text: &str,
) -> Result<(), WebViewError> {
    let preserve_ws = is_inside_pre_or_code(dom, parent);
    let normalized = if preserve_ws {
        decode_entities(raw_text)
    } else {
        let collapsed = collapse_ascii_whitespace(raw_text);
        if collapsed.trim().is_empty() {
            return Ok(());
        }
        decode_entities(&collapsed)
    };
    if !normalized.is_empty() {
        dom.append_text(parent, normalized)?;
    }
    Ok(())
}

fn is_inside_pre_or_code(dom: &DomArena, mut node_id: NodeId) -> bool {
    while let Some(node) = dom.get(node_id) {
        if let NodeKind::Element(elem) = &node.kind {
            if matches!(elem.tag_name.as_str(), "pre" | "code" | "textarea") {
                return true;
            }
        }
        let Some(parent) = node.parent else {
            break;
        };
        node_id = parent;
    }
    false
}

fn collapse_ascii_whitespace(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_ws = false;
    for ch in input.chars() {
        if ch.is_ascii_whitespace() {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
        } else {
            out.push(ch);
            prev_ws = false;
        }
    }
    out
}

type ParsedStartTag = (String, Vec<(String, String)>, bool, usize);

fn parse_start_tag(tag_rest: &str) -> Option<ParsedStartTag> {
    if !tag_rest.starts_with('<') {
        return None;
    }
    let bytes = tag_rest.as_bytes();
    let mut idx = 1;
    let name_start = idx;
    while idx < bytes.len()
        && (bytes[idx].is_ascii_alphanumeric() || bytes[idx] == b'-' || bytes[idx] == b'_')
    {
        idx += 1;
    }
    if idx == name_start {
        return None;
    }
    let tag_name = tag_rest[name_start..idx].to_ascii_lowercase();
    let mut attrs = Vec::new();
    let mut self_closing = false;

    while idx < bytes.len() {
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() {
            return None;
        }
        if bytes[idx] == b'>' {
            return Some((tag_name, attrs, self_closing, idx + 1));
        }
        if tag_rest[idx..].starts_with("/>") {
            self_closing = true;
            return Some((tag_name, attrs, self_closing, idx + 2));
        }

        let attr_start = idx;
        while idx < bytes.len()
            && !bytes[idx].is_ascii_whitespace()
            && bytes[idx] != b'='
            && bytes[idx] != b'>'
            && !tag_rest[idx..].starts_with("/>")
        {
            idx += 1;
        }
        if idx == attr_start {
            idx += 1;
            continue;
        }
        let attr_name = tag_rest[attr_start..idx].to_ascii_lowercase();
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        let mut attr_val = String::new();
        if idx < bytes.len() && bytes[idx] == b'=' {
            idx += 1;
            while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
                idx += 1;
            }
            if idx < bytes.len() && (bytes[idx] == b'"' || bytes[idx] == b'\'') {
                let quote = bytes[idx];
                idx += 1;
                let val_start = idx;
                while idx < bytes.len() && bytes[idx] != quote {
                    idx += 1;
                }
                attr_val = tag_rest[val_start..idx].to_owned();
                if idx < bytes.len() && bytes[idx] == quote {
                    idx += 1;
                }
            } else {
                let val_start = idx;
                while idx < bytes.len() && !bytes[idx].is_ascii_whitespace() && bytes[idx] != b'>' {
                    idx += 1;
                }
                attr_val = tag_rest[val_start..idx].to_owned();
            }
        }
        attrs.push((attr_name, attr_val));
    }
    None
}

fn find_ascii_case_insensitive(haystack: &str, needle_lower: &str) -> Option<usize> {
    let h_bytes = haystack.as_bytes();
    let n_bytes = needle_lower.as_bytes();
    if n_bytes.is_empty() || h_bytes.len() < n_bytes.len() {
        return None;
    }
    (0..=(h_bytes.len() - n_bytes.len())).find(|&i| {
        h_bytes[i..i + n_bytes.len()]
            .iter()
            .zip(n_bytes.iter())
            .all(|(&hb, &nb)| hb.to_ascii_lowercase() == nb)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_html_structure_void_tags_entities_and_extracts_style_and_script() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <style>.title { color: #ff0000; }</style>
            </head>
            <body>
                <!-- main header -->
                <h1 id="hdr" class="title">Welcome &amp; Enjoy</h1>
                <p>First paragraph<br>Second line
                <p>Implicitly closed previous paragraph</p>
                <input type="text" value="initial" disabled />
                <script>let x = 42;</script>
            </body>
            </html>
        "#;
        let parsed = parse_html(html, ResourceBudget::default()).unwrap();
        assert_eq!(parsed.stylesheets, vec![".title { color: #ff0000; }"]);
        assert_eq!(parsed.scripts, vec!["let x = 42;"]);

        let hdr = parsed.dom.get_element_by_id("hdr").unwrap();
        assert_eq!(parsed.dom.text_content(hdr), "Welcome & Enjoy");
    }

    #[test]
    fn rejects_external_script_or_anchor_uri_in_html() {
        let html = r#"<a href="javascript:evil()">Click</a>"#;
        assert!(matches!(
            parse_html(html, ResourceBudget::default()),
            Err(WebViewError::DisallowedUriScheme(_))
        ));
    }
}

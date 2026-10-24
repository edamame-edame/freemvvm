//! CommonMark + GFM Markdown parser into `DomArena` (`UNIT-WEBVIEW-002`).

use crate::dom::{DomArena, NodeId, ResourceBudget, WebViewError};

/// Parses a CommonMark / GitHub Flavored Markdown (GFM) document into a `DomArena`.
pub fn parse_markdown(input: &str, budget: ResourceBudget) -> Result<DomArena, WebViewError> {
    let mut dom = DomArena::new(budget);
    dom.check_source_bytes(input)?;

    let body = dom.append_element(dom.root(), "body")?;
    let lines: Vec<&str> = input.lines().collect();
    parse_blocks(&mut dom, body, &lines)?;
    Ok(dom)
}

fn parse_blocks(dom: &mut DomArena, parent: NodeId, lines: &[&str]) -> Result<(), WebViewError> {
    let mut idx = 0;
    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            idx += 1;
            continue;
        }

        // Fenced code block (``` or ~~~)
        if let Some(fence_char) = detect_code_fence(trimmed) {
            let lang = trimmed
                .trim_start_matches(fence_char)
                .trim()
                .split_ascii_whitespace()
                .next()
                .unwrap_or("");
            idx += 1;
            let mut code_buf = String::new();
            while idx < lines.len() {
                let inner = lines[idx];
                if is_closing_code_fence(inner.trim(), fence_char) {
                    idx += 1;
                    break;
                }
                if !code_buf.is_empty() {
                    code_buf.push('\n');
                }
                code_buf.push_str(inner);
                idx += 1;
            }
            let pre = dom.append_element(parent, "pre")?;
            let code = dom.append_element(pre, "code")?;
            if !lang.is_empty() {
                dom.set_attribute(code, "class", format!("language-{lang}"))?;
            }
            dom.append_text(code, code_buf)?;
            continue;
        }

        // ATX Heading (# .. ######)
        if let Some((level, heading_text)) = parse_atx_heading(trimmed) {
            let tag = match level {
                1 => "h1",
                2 => "h2",
                3 => "h3",
                4 => "h4",
                5 => "h5",
                _ => "h6",
            };
            let h = dom.append_element(parent, tag)?;
            parse_inlines(dom, h, heading_text)?;
            idx += 1;
            continue;
        }

        // Horizontal rule (---, ***, ___)
        if is_horizontal_rule(trimmed) {
            dom.append_element(parent, "hr")?;
            idx += 1;
            continue;
        }

        // Blockquote (> ...)
        if trimmed.starts_with('>') {
            let mut quote_lines: Vec<String> = Vec::new();
            while idx < lines.len() {
                let cur = lines[idx].trim_start();
                if let Some(rest) = cur.strip_prefix('>') {
                    let stripped = rest.strip_prefix(' ').unwrap_or(rest);
                    quote_lines.push(stripped.to_owned());
                    idx += 1;
                } else {
                    break;
                }
            }
            let bq = dom.append_element(parent, "blockquote")?;
            let refs: Vec<&str> = quote_lines.iter().map(String::as_str).collect();
            parse_blocks(dom, bq, &refs)?;
            continue;
        }

        // GFM Table (header row + separator row `| --- | :---: | ---: |`)
        if idx + 1 < lines.len() && is_table_row(trimmed) {
            if let Some(alignments) = parse_table_separator(lines[idx + 1].trim()) {
                let table = dom.append_element(parent, "table")?;
                let thead = dom.append_element(table, "thead")?;
                let head_tr = dom.append_element(thead, "tr")?;
                for (col_idx, cell_text) in split_table_cells(trimmed).into_iter().enumerate() {
                    let th = dom.append_element(head_tr, "th")?;
                    if let Some(Some(align)) = alignments.get(col_idx) {
                        dom.set_attribute(th, "style", format!("text-align: {align}"))?;
                    }
                    parse_inlines(dom, th, cell_text)?;
                }
                idx += 2;
                let tbody = dom.append_element(table, "tbody")?;
                while idx < lines.len() {
                    let row_line = lines[idx].trim();
                    if row_line.is_empty() || !is_table_row(row_line) {
                        break;
                    }
                    let tr = dom.append_element(tbody, "tr")?;
                    for (col_idx, cell_text) in split_table_cells(row_line).into_iter().enumerate()
                    {
                        let td = dom.append_element(tr, "td")?;
                        if let Some(Some(align)) = alignments.get(col_idx) {
                            dom.set_attribute(td, "style", format!("text-align: {align}"))?;
                        }
                        parse_inlines(dom, td, cell_text)?;
                    }
                    idx += 1;
                }
                continue;
            }
        }

        // Unordered or Ordered List
        if let Some((is_ordered, _first_item)) = parse_list_marker(trimmed) {
            let list_tag = if is_ordered { "ol" } else { "ul" };
            let list_el = dom.append_element(parent, list_tag)?;
            while idx < lines.len() {
                let cur = lines[idx].trim();
                let Some((cur_ordered, item_text)) = parse_list_marker(cur) else {
                    break;
                };
                if cur_ordered != is_ordered {
                    break;
                }
                let li = dom.append_element(list_el, "li")?;
                if let Some((checked, task_rest)) = parse_task_list_prefix(item_text) {
                    let checkbox = dom.append_element(li, "input")?;
                    dom.set_attribute(checkbox, "type", "checkbox")?;
                    dom.set_attribute(checkbox, "disabled", "disabled")?;
                    if checked {
                        dom.set_attribute(checkbox, "checked", "checked")?;
                    }
                    parse_inlines(dom, li, task_rest)?;
                } else {
                    parse_inlines(dom, li, item_text)?;
                }
                idx += 1;
            }
            continue;
        }

        // Paragraph (accumulate consecutive non-blank, non-special lines)
        let mut para_buf = String::new();
        while idx < lines.len() {
            let cur = lines[idx].trim();
            if cur.is_empty()
                || detect_code_fence(cur).is_some()
                || parse_atx_heading(cur).is_some()
                || is_horizontal_rule(cur)
                || cur.starts_with('>')
                || parse_list_marker(cur).is_some()
                || (idx + 1 < lines.len()
                    && is_table_row(cur)
                    && parse_table_separator(lines[idx + 1].trim()).is_some())
            {
                break;
            }
            if !para_buf.is_empty() {
                para_buf.push(' ');
            }
            para_buf.push_str(cur);
            idx += 1;
        }
        if !para_buf.is_empty() {
            let p = dom.append_element(parent, "p")?;
            parse_inlines(dom, p, &para_buf)?;
        }
    }
    Ok(())
}

fn detect_code_fence(trimmed: &str) -> Option<char> {
    if trimmed.starts_with("```") {
        Some('`')
    } else if trimmed.starts_with("~~~") {
        Some('~')
    } else {
        None
    }
}

fn is_closing_code_fence(trimmed: &str, fence_char: char) -> bool {
    let min_fence = if fence_char == '`' { "```" } else { "~~~" };
    trimmed.starts_with(min_fence) && trimmed.chars().all(|c| c == fence_char)
}

fn parse_atx_heading(trimmed: &str) -> Option<(usize, &str)> {
    let hashes = trimmed.bytes().take_while(|&b| b == b'#').count();
    if (1..=6).contains(&hashes) {
        let rest = &trimmed[hashes..];
        if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
            let text = rest.trim().trim_end_matches('#').trim();
            return Some((hashes, text));
        }
    }
    None
}

fn is_horizontal_rule(trimmed: &str) -> bool {
    for marker in ['-', '*', '_'] {
        let mut count = 0usize;
        let mut valid = true;
        for ch in trimmed.chars() {
            if ch == marker {
                count += 1;
            } else if !ch.is_ascii_whitespace() {
                valid = false;
                break;
            }
        }
        if valid && count >= 3 {
            return true;
        }
    }
    false
}

fn is_table_row(trimmed: &str) -> bool {
    trimmed.contains('|')
}

fn split_table_cells(trimmed: &str) -> Vec<&str> {
    let inner = trimmed
        .strip_prefix('|')
        .unwrap_or(trimmed)
        .strip_suffix('|')
        .unwrap_or(trimmed);
    inner.split('|').map(str::trim).collect()
}

fn parse_table_separator(trimmed: &str) -> Option<Vec<Option<&'static str>>> {
    if !trimmed.contains('|') || !trimmed.contains('-') {
        return None;
    }
    let cells = split_table_cells(trimmed);
    if cells.is_empty() {
        return None;
    }
    let mut alignments = Vec::with_capacity(cells.len());
    for cell in cells {
        if cell.is_empty() {
            return None;
        }
        let left_colon = cell.starts_with(':');
        let right_colon = cell.ends_with(':');
        let core = cell.trim_start_matches(':').trim_end_matches(':');
        if core.len() < 3 || !core.chars().all(|c| c == '-') {
            return None;
        }
        let align = match (left_colon, right_colon) {
            (true, true) => Some("center"),
            (false, true) => Some("right"),
            (true, false) => Some("left"),
            (false, false) => None,
        };
        alignments.push(align);
    }
    Some(alignments)
}

fn parse_list_marker(trimmed: &str) -> Option<(bool, &str)> {
    for prefix in ["- ", "* ", "+ "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return Some((false, rest.trim()));
        }
    }
    let digits = trimmed.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digits > 0 {
        let after_digits = &trimmed[digits..];
        if let Some(rest) = after_digits
            .strip_prefix(". ")
            .or_else(|| after_digits.strip_prefix(") "))
        {
            return Some((true, rest.trim()));
        }
    }
    None
}

fn parse_task_list_prefix(item_text: &str) -> Option<(bool, &str)> {
    if let Some(rest) = item_text.strip_prefix("[ ] ") {
        Some((false, rest))
    } else if let Some(rest) = item_text
        .strip_prefix("[x] ")
        .or_else(|| item_text.strip_prefix("[X] "))
    {
        Some((true, rest))
    } else {
        None
    }
}

/// Parses inline CommonMark/GFM syntax (`**bold**`, `*em*`, `~~del~~`, `` `code` ``, `[label](url)`, `![alt](src)`).
pub fn parse_inlines(dom: &mut DomArena, parent: NodeId, text: &str) -> Result<(), WebViewError> {
    let mut plain = String::new();
    let mut idx = 0;
    let bytes = text.as_bytes();

    let flush_plain =
        |dom: &mut DomArena, parent: NodeId, plain: &mut String| -> Result<(), WebViewError> {
            if !plain.is_empty() {
                let decoded = decode_entities(plain);
                plain.clear();
                dom.append_text(parent, decoded)?;
            }
            Ok(())
        };

    while idx < bytes.len() {
        let rest = &text[idx..];

        // Escape sequence `\*`, `\[`, etc.
        if bytes[idx] == b'\\' && idx + 1 < bytes.len() && bytes[idx + 1].is_ascii_punctuation() {
            plain.push(bytes[idx + 1] as char);
            idx += 2;
            continue;
        }

        // Inline code span `...`
        if let Some(after_tick) = rest.strip_prefix('`') {
            if let Some(end_rel) = after_tick.find('`') {
                flush_plain(dom, parent, &mut plain)?;
                let code_content = &after_tick[..end_rel];
                let code_node = dom.append_element(parent, "code")?;
                dom.append_text(code_node, code_content)?;
                idx += 2 + end_rel;
                continue;
            }
        }

        // Image `![alt](src)`
        if rest.starts_with("![") {
            if let Some((alt, src, consumed)) = parse_bracket_paren(&rest[1..]) {
                flush_plain(dom, parent, &mut plain)?;
                let img = dom.append_element(parent, "img")?;
                dom.set_attribute(img, "alt", decode_entities(alt))?;
                dom.set_attribute(img, "src", src.trim())?;
                idx += 1 + consumed;
                continue;
            }
        }

        // Link `[label](href)`
        if rest.starts_with('[') {
            if let Some((label, href, consumed)) = parse_bracket_paren(rest) {
                flush_plain(dom, parent, &mut plain)?;
                let a = dom.append_element(parent, "a")?;
                dom.set_attribute(a, "href", href.trim())?;
                parse_inlines(dom, a, label)?;
                idx += consumed;
                continue;
            }
        }

        // Bold `**...**` or `__...__`
        if let Some((delim, tag)) = [("**", "strong"), ("__", "strong"), ("~~", "del")]
            .into_iter()
            .find(|(d, _)| rest.starts_with(d))
        {
            if let Some(end_rel) = rest[delim.len()..].find(delim) {
                if end_rel > 0 {
                    flush_plain(dom, parent, &mut plain)?;
                    let inner = &rest[delim.len()..delim.len() + end_rel];
                    let el = dom.append_element(parent, tag)?;
                    parse_inlines(dom, el, inner)?;
                    idx += delim.len() * 2 + end_rel;
                    continue;
                }
            }
        }

        // Italic `*...*` or `_..._`
        if let Some(delim) = ['*', '_'].into_iter().find(|&d| rest.starts_with(d)) {
            if let Some(end_rel) = rest[1..].find(delim) {
                if end_rel > 0 {
                    flush_plain(dom, parent, &mut plain)?;
                    let inner = &rest[1..1 + end_rel];
                    let el = dom.append_element(parent, "em")?;
                    parse_inlines(dom, el, inner)?;
                    idx += 2 + end_rel;
                    continue;
                }
            }
        }

        let Some(ch) = rest.chars().next() else {
            break;
        };
        plain.push(ch);
        idx += ch.len_utf8();
    }

    flush_plain(dom, parent, &mut plain)?;
    Ok(())
}

fn parse_bracket_paren(input: &str) -> Option<(&str, &str, usize)> {
    if !input.starts_with('[') {
        return None;
    }
    let close_bracket = input[1..].find(']')? + 1;
    let after_bracket = &input[close_bracket + 1..];
    if !after_bracket.starts_with('(') {
        return None;
    }
    let close_paren = after_bracket[1..].find(')')? + 1;
    let label = &input[1..close_bracket];
    let target = &after_bracket[1..close_paren];
    let consumed = close_bracket + 1 + close_paren + 1;
    Some((label, target, consumed))
}

/// Decodes standard HTML character entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#39;`, `&nbsp;`).
pub fn decode_entities(input: &str) -> String {
    if !input.contains('&') {
        return input.to_owned();
    }
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(amp_idx) = rest.find('&') {
        out.push_str(&rest[..amp_idx]);
        let after_amp = &rest[amp_idx + 1..];
        if let Some(semi_idx) = after_amp.find(';') {
            let entity = &after_amp[..semi_idx];
            let replacement = match entity {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" | "#39" => Some('\''),
                "nbsp" => Some('\u{00A0}'),
                _ if entity.starts_with("#x") || entity.starts_with("#X") => {
                    u32::from_str_radix(&entity[2..], 16)
                        .ok()
                        .and_then(char::from_u32)
                }
                _ if entity.starts_with('#') => {
                    entity[1..].parse::<u32>().ok().and_then(char::from_u32)
                }
                _ => None,
            };
            if let Some(ch) = replacement {
                out.push(ch);
                rest = &after_amp[semi_idx + 1..];
                continue;
            }
        }
        out.push('&');
        rest = after_amp;
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::NodeKind;

    #[test]
    fn parses_headings_paragraphs_lists_tables_code_and_inlines() {
        let md = "\
# Title *Emphasis*

Paragraph with **bold**, ~~strike~~, `inline_code`, and [Action](command:run).

- [x] Done item
- [ ] Todo item

| Name | Score |
| :--- | ---: |
| Alice | 95 |

```rust
fn main() {}
```

> Quoted text
";
        let dom = parse_markdown(md, ResourceBudget::default()).unwrap();
        let full_text = dom.text_content(dom.root());
        assert!(full_text.contains("Title Emphasis"));
        assert!(full_text.contains("Paragraph with bold, strike, inline_code, and Action."));
        assert!(full_text.contains("Alice"));
        assert!(full_text.contains("fn main() {}"));
        assert!(full_text.contains("Quoted text"));

        let has_tag = |tag: &str| {
            dom.nodes().iter().any(|n| match &n.kind {
                NodeKind::Element(e) => e.tag_name == tag,
                _ => false,
            })
        };
        for expected in [
            "h1",
            "em",
            "p",
            "strong",
            "del",
            "code",
            "a",
            "ul",
            "li",
            "input",
            "table",
            "thead",
            "tbody",
            "th",
            "td",
            "pre",
            "blockquote",
        ] {
            assert!(has_tag(expected), "missing expected tag <{expected}>");
        }
    }

    #[test]
    fn rejects_external_links_in_markdown_under_offline_policy() {
        let md = "Click [evil](https://malicious.example.com) now.";
        assert!(matches!(
            parse_markdown(md, ResourceBudget::default()),
            Err(WebViewError::DisallowedUriScheme(_))
        ));
    }
}

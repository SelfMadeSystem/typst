use std::fmt::Write;

use typst_library::html::{HtmlDocument, HtmlElement, HtmlNode, HtmlTag};

/// Encodes an HTML document into a string.
pub fn html(document: &HtmlDocument) -> String {
    let mut w = Writer { buf: String::new() };
    write_element(&mut w, &document.root);
    w.buf
}

// TODO:
// - Proper whitespace handling and better escaping.
// - Pretty printing.
// - Ensure that void nodes have no children earlier.
// - Probably a lot more.

struct Writer {
    buf: String,
}

/// Encode an HTML node into the writer.
fn write_node(w: &mut Writer, node: &HtmlNode) {
    match node {
        HtmlNode::Text(text) => write_text(w, text),
        HtmlNode::Element(element) => write_element(w, element),
    }
}

/// Encode plain text into the writer.
fn write_text(w: &mut Writer, text: &str) {
    for c in text.chars() {
        match c {
            '<' => w.buf.push_str("&lt;"),
            '>' => w.buf.push_str("&gt;"),
            '&' => w.buf.push_str("&amp;"),
            '\'' => w.buf.push_str("&#39;"),
            '"' => w.buf.push_str("&quot;"),
            _ => w.buf.push(c),
        }
    }
}

/// Encode one element into the write.
fn write_element(w: &mut Writer, element: &HtmlElement) {
    w.buf.push('<');
    w.buf.push_str(element.tag.as_str());

    for (key, value) in &element.attrs.0 {
        write!(w.buf, " {key}={value:?}").unwrap();
    }

    w.buf.push('>');

    if is_void(&element.tag) {
        return;
    }

    for node in &element.children {
        write_node(w, node);
    }

    w.buf.push_str("</");
    w.buf.push_str(element.tag.as_str());
    w.buf.push('>');
}

/// Whether this is a void tag whose associated element may not have a children.
fn is_void(tag: &HtmlTag) -> bool {
    matches!(
        tag.as_str(),
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

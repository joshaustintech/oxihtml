use crate::dom::{Attr, Doctype, Namespace, Node, NodeData, NodeId, QualName};

fn namespace_prefix(ns: &Namespace) -> &'_ str {
    match ns {
        Namespace::Html => "",
        Namespace::Svg => "svg ",
        Namespace::MathMl => "math ",
        Namespace::Other(s) => {
            if s == "xlink" {
                "xlink "
            } else if s == "xml" {
                "xml "
            } else if s == "xmlns" {
                "xmlns "
            } else {
                ""
            }
        }
    }
}

fn qualified_name(name: &QualName) -> String {
    let prefix = namespace_prefix(&name.ns);
    if prefix.is_empty() {
        name.local.clone()
    } else {
        format!("{prefix}{}", name.local)
    }
}

fn is_template_html_ns(node: &Node) -> bool {
    match &node.data {
        NodeData::Element { name, .. } => matches!(name.ns, Namespace::Html) && name.local == "template",
        _ => false,
    }
}

fn utf16_sort_key(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

fn sort_attrs_for_test_output(attrs: &[Attr]) -> Vec<(Vec<u16>, String, String)> {
    let mut out = Vec::with_capacity(attrs.len());
    for attr in attrs {
        let display = qualified_name(&attr.name);
        let key = utf16_sort_key(&display);
        out.push((key, display, attr.value.clone()));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn doctype_to_test_format(doctype: &Doctype) -> String {
    if doctype.public_id.is_empty() && doctype.system_id.is_empty() {
        return format!("<!DOCTYPE {}>", doctype.name);
    }
    format!(
        "<!DOCTYPE {} \"{}\" \"{}\">",
        doctype.name, doctype.public_id, doctype.system_id
    )
}

fn node_to_test_lines(arena: &[Node], node_id: NodeId, indent: usize, out: &mut Vec<String>) {
    let node = &arena[node_id];
    match &node.data {
        NodeData::Document | NodeData::DocumentFragment => {
            for &child in &node.children {
                node_to_test_lines(arena, child, indent, out);
            }
        }
        NodeData::Doctype(dt) => {
            out.push(format!("| {}{}", " ".repeat(indent), doctype_to_test_format(dt)));
        }
        NodeData::Comment(data) => {
            out.push(format!("| {}<!-- {} -->", " ".repeat(indent), data));
        }
        NodeData::Text(data) => {
            out.push(format!("| {}\"{}\"", " ".repeat(indent), data));
        }
        NodeData::Element {
            name,
            attrs,
            template_contents,
        } => {
            out.push(format!("| {}<{}>", " ".repeat(indent), qualified_name(name)));
            for (_key, display, value) in sort_attrs_for_test_output(attrs) {
                out.push(format!("| {}{}=\"{}\"", " ".repeat(indent + 2), display, value));
            }

            if is_template_html_ns(node) {
                if let Some(contents) = *template_contents {
                    out.push(format!("| {}content", " ".repeat(indent + 2)));
                    for &child in &arena[contents].children {
                        node_to_test_lines(arena, child, indent + 4, out);
                    }
                    return;
                }
            }

            for &child in &node.children {
                node_to_test_lines(arena, child, indent + 2, out);
            }
        }
    }
}

pub fn to_test_format(arena: &[Node], root: NodeId) -> String {
    let mut lines = Vec::new();
    node_to_test_lines(arena, root, 0, &mut lines);
    lines.join("\n")
}

pub fn normalize_tree_text(text: &str) -> String {
    let trimmed = text.trim();
    trimmed
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

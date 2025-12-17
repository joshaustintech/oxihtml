use oxihtml::dom::{
    append_child, ensure_template_contents, set_attr, Attr, Doctype, Namespace, NodeData, QualName,
};
use oxihtml::serialize::to_test_format;

fn qname(ns: Namespace, local: &str) -> QualName {
    QualName {
        ns,
        local: local.to_string(),
    }
}

#[test]
fn test_format_serializes_doctype() {
    let mut doc = oxihtml::dom::Document::new_empty();
    let dt = doc.create_doctype(Doctype {
        name: "html".to_string(),
        public_id: String::new(),
        system_id: String::new(),
    });
    append_child(&mut doc.arena, doc.root, dt);
    assert_eq!(to_test_format(&doc.arena, doc.root), "| <!DOCTYPE html>");
}

#[test]
fn test_format_serializes_doctype_with_public_and_system_ids() {
    let mut doc = oxihtml::dom::Document::new_empty();
    let dt = doc.create_doctype(Doctype {
        name: "html".to_string(),
        public_id: "pub".to_string(),
        system_id: "sys".to_string(),
    });
    append_child(&mut doc.arena, doc.root, dt);
    assert_eq!(
        to_test_format(&doc.arena, doc.root),
        "| <!DOCTYPE html \"pub\" \"sys\">"
    );
}

#[test]
fn test_format_serializes_template_contents() {
    let mut doc = oxihtml::dom::Document::new_empty();
    let template = doc.create_element(qname(Namespace::Html, "template"));
    append_child(&mut doc.arena, doc.root, template);

    let contents = ensure_template_contents(&mut doc.arena, template);
    assert!(matches!(doc.arena[contents].data, NodeData::DocumentFragment));

    let p = doc.create_element(qname(Namespace::Html, "p"));
    let text = doc.create_text("hi");
    append_child(&mut doc.arena, p, text);
    append_child(&mut doc.arena, contents, p);

    assert_eq!(
        to_test_format(&doc.arena, doc.root),
        "| <template>\n|   content\n|     <p>\n|       \"hi\""
    );
}

#[test]
fn test_format_sorts_attributes_deterministically() {
    let mut doc = oxihtml::dom::Document::new_empty();
    let div = doc.create_element(qname(Namespace::Html, "div"));
    append_child(&mut doc.arena, doc.root, div);

    set_attr(
        &mut doc.arena,
        div,
        Attr {
            name: qname(Namespace::Html, "b"),
            value: "2".to_string(),
        },
    );
    set_attr(
        &mut doc.arena,
        div,
        Attr {
            name: qname(Namespace::Html, "a"),
            value: "1".to_string(),
        },
    );

    assert_eq!(
        to_test_format(&doc.arena, doc.root),
        "| <div>\n|   a=\"1\"\n|   b=\"2\""
    );
}


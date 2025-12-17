use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use oxihtml::html5lib::{parse_json, parse_tree_construction_dat, Json, ScriptDirective};

fn temp_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("oxihtml-test-{}-{}-{}", std::process::id(), t, name));
    p
}

#[test]
fn json_parser_supports_core_types_and_escapes() {
    let input = br#"{
      "null": null,
      "bools": [true, false],
      "num": -12,
      "str": "a\nb\tc\\\"",
      "u": "\u0041\uD83D\uDE00"
    }"#;
    let json = parse_json(input).expect("parse_json ok");
    let Json::Object(obj) = json else {
        panic!("expected object");
    };
    let get = |k: &str| obj.iter().find_map(|(kk, vv)| (kk == k).then_some(vv)).unwrap();
    assert_eq!(get("null"), &Json::Null);
    assert_eq!(get("num"), &Json::Number(-12));
    assert_eq!(
        get("bools"),
        &Json::Array(vec![Json::Bool(true), Json::Bool(false)])
    );
    assert_eq!(get("str"), &Json::String("a\nb\tc\\\"".to_string()));
    assert_eq!(get("u"), &Json::String("A😀".to_string()));
}

#[test]
fn tree_construction_dat_parses_cases_and_directives() {
    let dat = r#"#data
<p>Hello
#errors
1:1: some-error
#document
| <html>
|   <head>
|   <body>
|     <p>
|       "Hello"

#data
<svg><title>X</title></svg>
#errors
#document-fragment
svg svg
#script-on
#document
| <svg svg>
|   <svg svg>
|     <svg title>
|       "X"
"#;

    let path = temp_path("tc.dat");
    fs::write(&path, dat).unwrap();
    let cases = parse_tree_construction_dat(&path).unwrap();
    fs::remove_file(&path).ok();

    assert_eq!(cases.len(), 2);
    assert_eq!(cases[0].data.trim(), "<p>Hello");
    assert_eq!(cases[0].error_count, 1);
    assert_eq!(cases[0].script_directive, ScriptDirective::Both);
    assert!(cases[0].fragment_context.is_none());
    assert!(cases[0].expected.contains("| <html>"));

    assert_eq!(cases[1].script_directive, ScriptDirective::On);
    let ctx = cases[1].fragment_context.clone().unwrap();
    assert_eq!(ctx.namespace.as_deref(), Some("svg"));
    assert_eq!(ctx.tag_name, "svg");
}


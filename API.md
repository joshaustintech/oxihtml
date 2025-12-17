# OxiHTML public Rust API (draft)

Goal: a pure-Rust (std-only) HTML5 parser, tokenizer, and serializer that can run the full `html5lib-tests` suite.

Design constraints:
- Prefer enums for data modeling.
- Prefer arena indices (`NodeId`) over `Box`, `Arc`, or trait objects.
- Keep modules independent: input → tokenizer → treebuilder → dom → serializer.

## Crate layout

```text
oxihtml/
  src/
    lib.rs
    input.rs
    tokenizer.rs
    treebuilder.rs
    dom.rs
    serialize.rs
    html5lib.rs        # test-format serialization + fixtures parsing helpers (std-only)
  src/bin/
    html5lib-runner.rs
```

## Top-level API

```rust
pub struct Options {
    pub scripting_enabled: bool,
    pub iframe_srcdoc: bool,
    pub collect_errors: bool,
}

pub struct Parser {
    opts: Options,
}

impl Parser {
    pub fn new(opts: Options) -> Self;
    pub fn parse_document(&mut self, input: &str) -> Parsed<Document>;
    pub fn parse_fragment(&mut self, ctx: FragmentContext<'_>, input: &str) -> Parsed<DocumentFragment>;
}

pub struct Parsed<T> {
    pub value: T,
    pub errors: Vec<ParseError>,
}
```

## DOM model (arena-based, enum-first)

```rust
pub type NodeId = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Namespace {
    Html,
    Svg,
    MathMl,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualName {
    pub ns: Namespace,
    pub local: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attr {
    pub name: QualName,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeData {
    Document,
    DocumentFragment,
    Element { name: QualName, attrs: Vec<Attr>, template_contents: Option<NodeId> },
    Text(String),
    Comment(String),
    Doctype(Doctype),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub data: NodeData,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

pub struct Document {
    pub arena: Vec<Node>,
    pub root: NodeId, // NodeData::Document
}

pub struct DocumentFragment {
    pub arena: Vec<Node>,
    pub root: NodeId, // NodeData::DocumentFragment
}
```

## Errors and locations

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub line: u32,
    pub col: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub code: ErrorCode,
    pub location: Location,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    // Stored as exact html5lib/JustHTML codes to avoid mapping bugs.
    Code(String),
}
```

## html5lib conformance helpers

- `serialize::to_html(...)` (human HTML)
- `serialize::to_test_format(...)` (exact html5lib “| <tag>” format)
- `bin/html5lib-runner`:
  - loads fixtures from `~/html5lib-tests`
  - runs tokenizer and tree-construction tests
  - prints stable failure summaries and returns non-zero on failure


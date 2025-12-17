pub mod dom;
pub mod html5lib;
pub mod serialize;

#[derive(Clone, Debug)]
pub struct Options {
    pub scripting_enabled: bool,
    pub iframe_srcdoc: bool,
    pub collect_errors: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            scripting_enabled: false,
            iframe_srcdoc: false,
            collect_errors: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub line: u32,
    pub col: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    Code(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub code: ErrorCode,
    pub location: Location,
}

#[derive(Clone, Debug)]
pub struct Parsed<T> {
    pub value: T,
    pub errors: Vec<ParseError>,
}

#[derive(Clone, Debug)]
pub struct FragmentContext {
    pub namespace: Option<String>,
    pub tag_name: String,
}

pub struct Parser {
    opts: Options,
}

impl Parser {
    pub fn new(opts: Options) -> Self {
        Self { opts }
    }

    pub fn parse_document(&mut self, _input: &str) -> Parsed<dom::Document> {
        let doc = dom::Document::new_empty();
        Parsed {
            value: doc,
            errors: if self.opts.collect_errors {
                vec![ParseError {
                    code: ErrorCode::Code("unimplemented".to_string()),
                    location: Location { line: 1, col: 1 },
                }]
            } else {
                Vec::new()
            },
        }
    }

    pub fn parse_fragment(&mut self, _ctx: FragmentContext, _input: &str) -> Parsed<dom::DocumentFragment> {
        let frag = dom::DocumentFragment::new_empty();
        Parsed {
            value: frag,
            errors: if self.opts.collect_errors {
                vec![ParseError {
                    code: ErrorCode::Code("unimplemented".to_string()),
                    location: Location { line: 1, col: 1 },
                }]
            } else {
                Vec::new()
            },
        }
    }
}

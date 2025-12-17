use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(i64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

#[derive(Clone, Debug)]
pub struct JsonParseError {
    pub message: String,
    pub offset: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScriptDirective {
    On,
    Off,
    Both,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FragmentContextSpec {
    pub namespace: Option<String>,
    pub tag_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeConstructionCase {
    pub data: String,
    pub error_count: usize,
    pub fragment_context: Option<FragmentContextSpec>,
    pub script_directive: ScriptDirective,
    pub expected: String,
}

fn is_header_line(line: &str) -> bool {
    matches!(
        line,
        "#data"
            | "#errors"
            | "#new-errors"
            | "#document-fragment"
            | "#script-on"
            | "#script-off"
            | "#document"
    )
}

fn parse_fragment_context_line(line: &str) -> FragmentContextSpec {
    let s = line.trim();
    if let Some(rest) = s.strip_prefix("svg ") {
        return FragmentContextSpec {
            namespace: Some("svg".to_string()),
            tag_name: rest.to_string(),
        };
    }
    if let Some(rest) = s.strip_prefix("math ") {
        return FragmentContextSpec {
            namespace: Some("math".to_string()),
            tag_name: rest.to_string(),
        };
    }
    FragmentContextSpec {
        namespace: None,
        tag_name: s.to_string(),
    }
}

pub fn discover_tree_construction_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let tc_root = root.join("tree-construction");
    if !tc_root.is_dir() {
        return Ok(out);
    }

    let mut stack = vec![tc_root];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == "dat")
            {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

pub fn discover_tokenizer_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    discover_test_json_files_in(root.join("tokenizer"))
}

pub fn discover_serializer_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    discover_test_json_files_in(root.join("serializer"))
}

fn discover_test_json_files_in(dir: PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }

    let mut stack = vec![dir];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == "test")
            {
                out.push(path);
            }
        }
    }

    out.sort();
    Ok(out)
}

pub fn parse_json_file(path: &Path) -> io::Result<Result<Json, JsonParseError>> {
    let bytes = fs::read(path)?;
    Ok(parse_json(&bytes))
}

pub fn parse_json(input: &[u8]) -> Result<Json, JsonParseError> {
    let mut p = JsonParser { input, i: 0 };
    p.skip_ws();
    let value = p.parse_value()?;
    p.skip_ws();
    if p.i != p.input.len() {
        return Err(p.err("trailing characters"));
    }
    Ok(value)
}

struct JsonParser<'a> {
    input: &'a [u8],
    i: usize,
}

impl<'a> JsonParser<'a> {
    fn err(&self, message: &str) -> JsonParseError {
        JsonParseError {
            message: message.to_string(),
            offset: self.i,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.i).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.i += 1;
        Some(b)
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if matches!(b, b' ' | b'\n' | b'\r' | b'\t') {
                self.i += 1;
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Result<Json, JsonParseError> {
        self.skip_ws();
        match self.peek().ok_or_else(|| self.err("unexpected EOF"))? {
            b'n' => self.parse_null(),
            b't' | b'f' => self.parse_bool(),
            b'-' | b'0'..=b'9' => self.parse_number(),
            b'"' => self.parse_string().map(Json::String),
            b'[' => self.parse_array(),
            b'{' => self.parse_object(),
            _ => Err(self.err("unexpected character")),
        }
    }

    fn parse_null(&mut self) -> Result<Json, JsonParseError> {
        if self.input.get(self.i..self.i + 4) == Some(b"null") {
            self.i += 4;
            Ok(Json::Null)
        } else {
            Err(self.err("expected null"))
        }
    }

    fn parse_bool(&mut self) -> Result<Json, JsonParseError> {
        if self.input.get(self.i..self.i + 4) == Some(b"true") {
            self.i += 4;
            Ok(Json::Bool(true))
        } else if self.input.get(self.i..self.i + 5) == Some(b"false") {
            self.i += 5;
            Ok(Json::Bool(false))
        } else {
            Err(self.err("expected boolean"))
        }
    }

    fn parse_number(&mut self) -> Result<Json, JsonParseError> {
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        let mut saw_digit = false;
        while let Some(b'0'..=b'9') = self.peek() {
            saw_digit = true;
            self.i += 1;
        }
        if !saw_digit {
            return Err(self.err("expected digits"));
        }
        if self.peek() == Some(b'.') || self.peek() == Some(b'e') || self.peek() == Some(b'E') {
            return Err(self.err("non-integer numbers not supported"));
        }
        let s = std::str::from_utf8(&self.input[start..self.i]).map_err(|_| self.err("invalid utf-8"))?;
        let n = s.parse::<i64>().map_err(|_| self.err("invalid number"))?;
        Ok(Json::Number(n))
    }

    fn parse_string(&mut self) -> Result<String, JsonParseError> {
        if self.bump() != Some(b'"') {
            return Err(self.err("expected '\"'"));
        }
        let mut out: Vec<u8> = Vec::new();
        while let Some(b) = self.bump() {
            match b {
                b'"' => return String::from_utf8(out).map_err(|_| self.err("invalid utf-8")),
                b'\\' => {
                    let esc = self.bump().ok_or_else(|| self.err("unexpected EOF in escape"))?;
                    match esc {
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        b'/' => out.push(b'/'),
                        b'b' => out.push(0x08),
                        b'f' => out.push(0x0C),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'u' => {
                            let code = self.parse_hex_u16()?;
                            if (0xD800..=0xDBFF).contains(&code) {
                                // surrogate pair
                                if self.bump() != Some(b'\\') || self.bump() != Some(b'u') {
                                    return Err(self.err("expected low surrogate"));
                                }
                                let low = self.parse_hex_u16()?;
                                if !(0xDC00..=0xDFFF).contains(&low) {
                                    return Err(self.err("invalid low surrogate"));
                                }
                                let hi = (code - 0xD800) as u32;
                                let lo = (low - 0xDC00) as u32;
                                let cp = 0x10000 + ((hi << 10) | lo);
                                let ch = char::from_u32(cp).ok_or_else(|| self.err("invalid codepoint"))?;
                                let mut buf = [0u8; 4];
                                let encoded = ch.encode_utf8(&mut buf);
                                out.extend_from_slice(encoded.as_bytes());
                            } else {
                                let ch = char::from_u32(code as u32).ok_or_else(|| self.err("invalid codepoint"))?;
                                let mut buf = [0u8; 4];
                                let encoded = ch.encode_utf8(&mut buf);
                                out.extend_from_slice(encoded.as_bytes());
                            }
                        }
                        _ => return Err(self.err("unknown escape")),
                    }
                }
                _ => {
                    if b < 0x20 {
                        return Err(self.err("control character in string"));
                    }
                    out.push(b);
                }
            }
        }
        Err(self.err("unexpected EOF in string"))
    }

    fn parse_hex_u16(&mut self) -> Result<u16, JsonParseError> {
        let mut v: u16 = 0;
        for _ in 0..4 {
            let b = self.bump().ok_or_else(|| self.err("unexpected EOF in \\u"))?;
            let n = match b {
                b'0'..=b'9' => (b - b'0') as u16,
                b'a'..=b'f' => (b - b'a' + 10) as u16,
                b'A'..=b'F' => (b - b'A' + 10) as u16,
                _ => return Err(self.err("invalid hex digit")),
            };
            v = (v << 4) | n;
        }
        Ok(v)
    }

    fn parse_array(&mut self) -> Result<Json, JsonParseError> {
        if self.bump() != Some(b'[') {
            return Err(self.err("expected '['"));
        }
        self.skip_ws();
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Ok(Json::Array(items));
        }
        loop {
            let v = self.parse_value()?;
            items.push(v);
            self.skip_ws();
            match self.bump().ok_or_else(|| self.err("unexpected EOF in array"))? {
                b',' => {
                    self.skip_ws();
                    continue;
                }
                b']' => return Ok(Json::Array(items)),
                _ => return Err(self.err("expected ',' or ']'")),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Json, JsonParseError> {
        if self.bump() != Some(b'{') {
            return Err(self.err("expected '{'"));
        }
        self.skip_ws();
        let mut items = Vec::new();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Ok(Json::Object(items));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            if self.bump() != Some(b':') {
                return Err(self.err("expected ':'"));
            }
            let value = self.parse_value()?;
            items.push((key, value));
            self.skip_ws();
            match self.bump().ok_or_else(|| self.err("unexpected EOF in object"))? {
                b',' => {
                    self.skip_ws();
                    continue;
                }
                b'}' => return Ok(Json::Object(items)),
                _ => return Err(self.err("expected ',' or '}'")),
            }
        }
    }
}

pub fn parse_tree_construction_dat(path: &Path) -> io::Result<Vec<TreeConstructionCase>> {
    let content = fs::read_to_string(path)?;
    let mut lines = content.split('\n').peekable();
    let mut cases = Vec::new();

    while let Some(line) = lines.next() {
        if line != "#data" {
            continue;
        }

        let mut data_lines = Vec::new();
        while let Some(&next) = lines.peek() {
            if next == "#errors" {
                break;
            }
            data_lines.push(lines.next().unwrap());
        }
        if lines.next() != Some("#errors") {
            continue;
        }
        let data = if data_lines.is_empty() {
            String::new()
        } else {
            data_lines.join("\n")
        };

        let mut error_count = 0usize;
        while let Some(&next) = lines.peek() {
            if is_header_line(next) {
                break;
            }
            let err_line = lines.next().unwrap();
            if !err_line.trim().is_empty() {
                error_count += 1;
            }
        }

        if lines.peek().copied() == Some("#new-errors") {
            lines.next();
            while let Some(&next) = lines.peek() {
                if is_header_line(next) {
                    break;
                }
                let err_line = lines.next().unwrap();
                if !err_line.trim().is_empty() {
                    error_count += 1;
                }
            }
        }

        let mut fragment_context: Option<FragmentContextSpec> = None;
        if lines.peek().copied() == Some("#document-fragment") {
            lines.next();
            let ctx_line = lines.next().unwrap_or_default();
            fragment_context = Some(parse_fragment_context_line(ctx_line));
        }

        let mut script_directive = ScriptDirective::Both;
        if let Some(&next) = lines.peek() {
            if next == "#script-on" {
                script_directive = ScriptDirective::On;
                lines.next();
            } else if next == "#script-off" {
                script_directive = ScriptDirective::Off;
                lines.next();
            }
        }

        if lines.next() != Some("#document") {
            continue;
        }

        let mut expected_lines = Vec::new();
        while let Some(&next) = lines.peek() {
            if next == "#data" {
                break;
            }
            expected_lines.push(lines.next().unwrap());
        }

        let expected = expected_lines
            .join("\n")
            .trim_matches('\n')
            .to_string();

        cases.push(TreeConstructionCase {
            data,
            error_count,
            fragment_context,
            script_directive,
            expected,
        });
    }

    Ok(cases)
}

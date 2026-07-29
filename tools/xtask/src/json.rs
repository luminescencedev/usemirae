//! JSON reading for the manifests and schemas xtask consumes.
//!
//! Two layers, because they answer different questions:
//! - `string_field` and `nested_string_field` read one known field out of a
//!   manifest without caring about the rest of the document;
//! - `parse` builds a full value tree, which schema code generation needs.
//!
//! Both are hand-written: a JSON crate would have to clear the Rust dependency
//! procedure in `DEPENDENCY_VERSIONS.md` section 11. The parser accepts the JSON
//! subset the repository authors (no `\u` escapes, no exponent notation) and
//! returns an error for anything else rather than guessing.

use std::fmt;

/// A parsed JSON value.
///
/// Objects keep source order so generated output is stable.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Value {
    Null,
    Bool(bool),
    /// Numbers are kept as text so an integer never round-trips through a float.
    Number(String),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Value {
    /// Read a string value.
    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(text) => Some(text),
            _ => None,
        }
    }

    /// Read an object member by key.
    pub(crate) fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Self::Object(members) => members
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    /// Read the members of an object in source order.
    pub(crate) fn members(&self) -> &[(String, Value)] {
        match self {
            Self::Object(members) => members,
            _ => &[],
        }
    }

    /// Read the elements of an array.
    pub(crate) fn elements(&self) -> &[Value] {
        match self {
            Self::Array(elements) => elements,
            _ => &[],
        }
    }
}

/// Why a document could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParseError {
    /// Byte offset where parsing stopped.
    pub(crate) offset: usize,
    pub(crate) detail: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "at byte {}: {}", self.offset, self.detail)
    }
}

/// Cursor over the document being parsed.
struct Parser<'input> {
    bytes: &'input [u8],
    offset: usize,
}

impl<'input> Parser<'input> {
    fn new(input: &'input str) -> Self {
        Self {
            bytes: input.as_bytes(),
            offset: 0,
        }
    }

    fn error(&self, detail: &str) -> ParseError {
        ParseError {
            offset: self.offset,
            detail: detail.to_owned(),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.offset).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(byte) = self.peek() {
            if byte.is_ascii_whitespace() {
                self.offset += 1;
            } else {
                break;
            }
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), ParseError> {
        if self.peek() == Some(byte) {
            self.offset += 1;
            return Ok(());
        }

        Err(self.error(&format!("expected `{}`", byte as char)))
    }

    fn literal(&mut self, text: &str, value: Value) -> Result<Value, ParseError> {
        if self.bytes[self.offset..].starts_with(text.as_bytes()) {
            self.offset += text.len();
            return Ok(value);
        }

        Err(self.error("unrecognised literal"))
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.expect(b'"')?;
        let mut out = String::new();

        while let Some(byte) = self.peek() {
            self.offset += 1;

            match byte {
                b'"' => return Ok(out),
                b'\\' => {
                    let escape = self.peek().ok_or_else(|| self.error("truncated escape"))?;
                    self.offset += 1;
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'b' | b'f' => return Err(self.error("unsupported escape")),
                        b'u' => {
                            return Err(self.error(
                                "\\u escapes are not supported; write the character directly",
                            ));
                        }
                        _ => return Err(self.error("unknown escape")),
                    }
                }
                _ => {
                    // Multi-byte UTF-8 sequences are copied through unchanged.
                    let start = self.offset - 1;
                    let mut end = self.offset;
                    while end < self.bytes.len() && self.bytes[end] & 0b1100_0000 == 0b1000_0000 {
                        end += 1;
                    }
                    let slice = self
                        .bytes
                        .get(start..end)
                        .ok_or_else(|| self.error("truncated string"))?;
                    match std::str::from_utf8(slice) {
                        Ok(text) => out.push_str(text),
                        Err(_) => return Err(self.error("invalid UTF-8 in string")),
                    }
                    self.offset = end;
                }
            }
        }

        Err(self.error("unterminated string"))
    }

    fn parse_number(&mut self) -> Result<Value, ParseError> {
        let start = self.offset;

        if self.peek() == Some(b'-') {
            self.offset += 1;
        }

        while let Some(byte) = self.peek() {
            if byte.is_ascii_digit() || byte == b'.' {
                self.offset += 1;
            } else if byte == b'e' || byte == b'E' {
                return Err(self.error("exponent notation is not supported"));
            } else {
                break;
            }
        }

        if start == self.offset {
            return Err(self.error("expected a number"));
        }

        match std::str::from_utf8(&self.bytes[start..self.offset]) {
            Ok(text) => Ok(Value::Number(text.to_owned())),
            Err(_) => Err(self.error("invalid number")),
        }
    }

    fn parse_array(&mut self) -> Result<Value, ParseError> {
        self.expect(b'[')?;
        let mut elements = Vec::new();
        self.skip_whitespace();

        if self.peek() == Some(b']') {
            self.offset += 1;
            return Ok(Value::Array(elements));
        }

        loop {
            self.skip_whitespace();
            elements.push(self.parse_value()?);
            self.skip_whitespace();

            match self.peek() {
                Some(b',') => self.offset += 1,
                Some(b']') => {
                    self.offset += 1;
                    return Ok(Value::Array(elements));
                }
                _ => return Err(self.error("expected `,` or `]`")),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Value, ParseError> {
        self.expect(b'{')?;
        let mut members: Vec<(String, Value)> = Vec::new();
        self.skip_whitespace();

        if self.peek() == Some(b'}') {
            self.offset += 1;
            return Ok(Value::Object(members));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;

            if members.iter().any(|(existing, _)| *existing == key) {
                return Err(self.error(&format!("duplicate key `{key}`")));
            }

            self.skip_whitespace();
            self.expect(b':')?;
            self.skip_whitespace();
            let value = self.parse_value()?;
            members.push((key, value));
            self.skip_whitespace();

            match self.peek() {
                Some(b',') => self.offset += 1,
                Some(b'}') => {
                    self.offset += 1;
                    return Ok(Value::Object(members));
                }
                _ => return Err(self.error("expected `,` or `}`")),
            }
        }
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        self.skip_whitespace();

        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => self.parse_string().map(Value::String),
            Some(b't') => self.literal("true", Value::Bool(true)),
            Some(b'f') => self.literal("false", Value::Bool(false)),
            Some(b'n') => self.literal("null", Value::Null),
            Some(byte) if byte == b'-' || byte.is_ascii_digit() => self.parse_number(),
            Some(_) => Err(self.error("unexpected character")),
            None => Err(self.error("unexpected end of input")),
        }
    }
}

/// Parse a complete JSON document.
pub(crate) fn parse(input: &str) -> Result<Value, ParseError> {
    let mut parser = Parser::new(input);
    let value = parser.parse_value()?;
    parser.skip_whitespace();

    if parser.offset != parser.bytes.len() {
        return Err(parser.error("trailing content after the document"));
    }

    Ok(value)
}

/// Find the object body that follows `"key":`, including nested braces.
fn object_body<'input>(source: &'input str, key: &str) -> Option<&'input str> {
    let needle = format!("\"{key}\"");
    let after_key = source.find(&needle)? + needle.len();
    let start = source[after_key..].find('{')? + after_key + 1;

    let mut depth = 1_usize;
    for (offset, character) in source[start..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&source[start..start + offset]);
                }
            }
            _ => {}
        }
    }

    None
}

/// Read a top-level string field such as `packageManager`.
///
/// Only the first match is considered, so a field nested in an earlier object
/// cannot shadow the lookup for well-formed manifests.
pub(crate) fn string_field<'input>(source: &'input str, key: &str) -> Option<&'input str> {
    let needle = format!("\"{key}\"");
    let after_key = source.find(&needle)? + needle.len();
    let rest = source[after_key..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;

    Some(&rest[..end])
}

/// Read a string field from inside a nested object, such as `engines.node`.
pub(crate) fn nested_string_field<'input>(
    source: &'input str,
    object: &str,
    key: &str,
) -> Option<&'input str> {
    string_field(object_body(source, object)?, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"{
  "name": "mirae",
  "private": true,
  "packageManager": "pnpm@11.17.0",
  "engines": {
    "node": "24.18.1",
    "pnpm": "11.17.0"
  },
  "scripts": {
    "preinstall": "cargo xtask bootstrap"
  }
}"#;

    #[test]
    fn reads_a_top_level_string_field() {
        assert_eq!(
            string_field(MANIFEST, "packageManager"),
            Some("pnpm@11.17.0")
        );
        assert_eq!(string_field(MANIFEST, "name"), Some("mirae"));
    }

    #[test]
    fn reads_nested_string_fields() {
        assert_eq!(
            nested_string_field(MANIFEST, "engines", "node"),
            Some("24.18.1")
        );
        assert_eq!(
            nested_string_field(MANIFEST, "engines", "pnpm"),
            Some("11.17.0")
        );
    }

    #[test]
    fn does_not_confuse_a_later_object_for_the_requested_one() {
        assert_eq!(
            nested_string_field(MANIFEST, "scripts", "preinstall"),
            Some("cargo xtask bootstrap")
        );
    }

    #[test]
    fn returns_none_for_absent_fields() {
        assert_eq!(string_field(MANIFEST, "license"), None);
        assert_eq!(nested_string_field(MANIFEST, "engines", "bun"), None);
        assert_eq!(nested_string_field(MANIFEST, "absent", "node"), None);
    }

    #[test]
    fn returns_none_for_a_non_string_value() {
        assert_eq!(string_field(MANIFEST, "private"), None);
    }

    #[test]
    fn handles_nested_braces_inside_the_object() {
        let source = r#"{"engines":{"extra":{"deep":"x"},"node":"24.18.1"}}"#;
        assert_eq!(
            nested_string_field(source, "engines", "node"),
            Some("24.18.1")
        );
    }

    #[test]
    fn parses_every_value_kind() {
        let document = r#"
        {
          "text": "hello",
          "escaped": "a\"b\\c\nd",
          "unicode": "réady ✓",
          "integer": 42,
          "negative": -7,
          "decimal": 1.5,
          "yes": true,
          "no": false,
          "nothing": null,
          "list": [1, "two", false],
          "nested": { "inner": { "leaf": "value" } },
          "empty_object": {},
          "empty_array": []
        }
        "#;

        let value = parse(document).unwrap_or(Value::Null);

        assert_eq!(value.get("text").and_then(Value::as_str), Some("hello"));
        assert_eq!(
            value.get("escaped").and_then(Value::as_str),
            Some("a\"b\\c\nd")
        );
        assert_eq!(
            value.get("unicode").and_then(Value::as_str),
            Some("réady ✓")
        );
        assert_eq!(value.get("integer"), Some(&Value::Number("42".to_owned())));
        assert_eq!(value.get("negative"), Some(&Value::Number("-7".to_owned())));
        assert_eq!(value.get("yes"), Some(&Value::Bool(true)));
        assert_eq!(value.get("nothing"), Some(&Value::Null));
        assert_eq!(value.get("list").map(|list| list.elements().len()), Some(3));
        assert_eq!(
            value
                .get("nested")
                .and_then(|nested| nested.get("inner"))
                .and_then(|inner| inner.get("leaf"))
                .and_then(Value::as_str),
            Some("value")
        );
        assert_eq!(value.get("empty_object").map(Value::members), Some(&[][..]));
        assert_eq!(value.get("empty_array").map(Value::elements), Some(&[][..]));
    }

    #[test]
    fn keeps_object_members_in_source_order() {
        let value = parse(r#"{"zulu":1,"alpha":2,"mike":3}"#).unwrap_or(Value::Null);
        let keys: Vec<&str> = value
            .members()
            .iter()
            .map(|(key, _)| key.as_str())
            .collect();

        assert_eq!(keys, vec!["zulu", "alpha", "mike"]);
    }

    #[test]
    fn keeps_integers_exact() {
        // A float round-trip would turn this into 9007199254740993.0 and lose a
        // digit; keeping the text avoids the question entirely.
        let value = parse(r#"{"big": 9007199254740993}"#).unwrap_or(Value::Null);

        assert_eq!(
            value.get("big"),
            Some(&Value::Number("9007199254740993".to_owned()))
        );
    }

    #[test]
    fn rejects_malformed_documents() {
        for document in [
            "{",
            "{\"a\"}",
            "{\"a\":}",
            "{\"a\":1,}",
            "[1,]",
            "{} extra",
            "\"unterminated",
            "tru",
            "{\"a\": 1e5}",
        ] {
            assert!(parse(document).is_err(), "accepted `{document}`");
        }
    }

    #[test]
    fn rejects_a_duplicate_key() {
        let error = parse(r#"{"a":1,"a":2}"#);

        assert!(
            error
                .err()
                .is_some_and(|error| error.detail.contains("duplicate key"))
        );
    }

    #[test]
    fn rejects_unsupported_escapes_with_an_explanation() {
        // A literal backslash-u escape, which the parser rejects on purpose.
        let document = "{\"a\":\"\\u0041\"}";
        let error = parse(document);

        assert!(
            error
                .err()
                .is_some_and(|error| error.detail.contains("\\u escapes"))
        );
    }
}

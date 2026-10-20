//! The canonical parser.
//!
//! The parser is the boundary at which bytes from outside become a document
//! the decision path can act on. It is written to be boring in the way that
//! matters: no recursion without a depth bound, no allocation before the
//! corresponding limit is checked, and no input shape that produces a panic.
//!
//! Everything it rejects, it rejects with a named reason. There is no
//! "something went wrong" path out of here.

use std::collections::BTreeMap;
use std::string::String;
use std::vec::Vec;

use crate::error::CanonicalError;
use crate::value::Value;

/// The largest document the profile accepts, in bytes.
pub const MAX_DOCUMENT_BYTES: usize = crate::limits::MAX_DOCUMENT_BYTES;
/// The deepest nesting the profile accepts.
pub const MAX_DEPTH: usize = crate::limits::MAX_DEPTH;
/// The most members an object may have.
pub const MAX_OBJECT_MEMBERS: usize = crate::limits::MAX_OBJECT_MEMBERS;
/// The most elements an array may have.
pub const MAX_ARRAY_ELEMENTS: usize = crate::limits::MAX_ARRAY_ELEMENTS;
/// The longest a string may be, in bytes.
pub const MAX_STRING_BYTES: usize = crate::limits::MAX_STRING_BYTES;

/// Parses a document under the canonical profile.
///
/// The input must be UTF-8. Use [`parse_bytes`] when the input has not been
/// checked for UTF-8 yet.
pub fn parse(input: &str) -> Result<Value, CanonicalError> {
    Parser::new(input).parse_document()
}

/// Parses a document under the canonical profile, checking UTF-8 first.
///
/// This is the entry point for anything that arrived as bytes.
pub fn parse_bytes(input: &[u8]) -> Result<Value, CanonicalError> {
    let text = std::str::from_utf8(input)
        .map_err(|error| CanonicalError::InvalidUtf8 { valid_up_to: error.valid_up_to() })?;
    parse(text)
}

/// Parses a document and re-encodes it in canonical form.
///
/// The result is the one byte sequence that stands for this document. Two
/// documents with the same canonical form are the same document; two
/// documents with different canonical forms are not.
pub fn canonicalize(input: &str) -> Result<String, CanonicalError> {
    Ok(crate::encode::encode_to_string(&parse(input)?))
}

/// Returns whether the input is already in canonical form.
///
/// Canonical means: parsing it and re-encoding it reproduces the input byte
/// for byte. This is the strongest available statement that a document has
/// exactly one representation, and it is the definition the digest bindings in
/// the rest of the project rely on.
pub fn is_canonical(input: &str) -> bool {
    match parse(input) {
        Ok(value) => crate::encode::encode_to_string(&value) == input,
        Err(_) => false,
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { bytes: input.as_bytes(), pos: 0 }
    }

    fn parse_document(mut self) -> Result<Value, CanonicalError> {
        if self.bytes.len() > MAX_DOCUMENT_BYTES {
            return Err(CanonicalError::DocumentLimitExceeded { limit: MAX_DOCUMENT_BYTES });
        }
        self.skip_whitespace();
        let value = self.parse_value(0)?;
        self.skip_whitespace();
        if self.pos != self.bytes.len() {
            return Err(CanonicalError::TrailingContent { offset: self.pos });
        }
        Ok(value)
    }

    fn parse_value(&mut self, depth: usize) -> Result<Value, CanonicalError> {
        // The depth check happens before the value is looked at, so a document
        // that is only deep enough to blow the stack is rejected without
        // recursing into it.
        if depth > MAX_DEPTH {
            return Err(CanonicalError::DepthLimitExceeded { limit: MAX_DEPTH, offset: self.pos });
        }
        let byte = self.peek()?;
        match byte {
            b'n' => self.parse_literal(b"null", Value::Null),
            b't' => self.parse_literal(b"true", Value::Bool(true)),
            b'f' => self.parse_literal(b"false", Value::Bool(false)),
            b'"' => Ok(Value::String(self.parse_string()?)),
            b'[' => self.parse_array(depth),
            b'{' => self.parse_object(depth),
            b'-' | b'+' | b'0'..=b'9' => Ok(Value::Int(self.parse_number()?)),
            other => Err(CanonicalError::UnexpectedByte { byte: other, offset: self.pos }),
        }
    }

    fn parse_literal(&mut self, literal: &[u8], value: Value) -> Result<Value, CanonicalError> {
        let end = self.pos + literal.len();
        if end <= self.bytes.len() && &self.bytes[self.pos..end] == literal {
            self.pos = end;
            Ok(value)
        } else {
            Err(CanonicalError::UnexpectedByte {
                byte: self.bytes.get(self.pos).copied().unwrap_or(b' '),
                offset: self.pos,
            })
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<Value, CanonicalError> {
        self.pos += 1; // consume '['
        let mut items: Vec<Value> = Vec::new();
        self.skip_whitespace();
        if self.peek()? == b']' {
            self.pos += 1;
            return Ok(Value::Array(items));
        }
        loop {
            self.skip_whitespace();
            let item = self.parse_value(depth + 1)?;
            if items.len() >= MAX_ARRAY_ELEMENTS {
                return Err(CanonicalError::ArrayLimitExceeded {
                    limit: MAX_ARRAY_ELEMENTS,
                    offset: self.pos,
                });
            }
            items.push(item);
            self.skip_whitespace();
            match self.peek()? {
                b',' => self.pos += 1,
                b']' => {
                    self.pos += 1;
                    return Ok(Value::Array(items));
                }
                other => {
                    return Err(CanonicalError::UnexpectedByte { byte: other, offset: self.pos });
                }
            }
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<Value, CanonicalError> {
        self.pos += 1; // consume '{'
        let mut map: BTreeMap<String, Value> = BTreeMap::new();
        self.skip_whitespace();
        if self.peek()? == b'}' {
            self.pos += 1;
            return Ok(Value::Object(map));
        }
        loop {
            self.skip_whitespace();
            if self.peek()? != b'"' {
                return Err(CanonicalError::UnexpectedByte {
                    byte: self.peek()?,
                    offset: self.pos,
                });
            }
            let key_offset = self.pos;
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.peek()? != b':' {
                return Err(CanonicalError::UnexpectedByte {
                    byte: self.peek()?,
                    offset: self.pos,
                });
            }
            self.pos += 1;
            self.skip_whitespace();
            let value = self.parse_value(depth + 1)?;
            if map.len() >= MAX_OBJECT_MEMBERS {
                return Err(CanonicalError::MemberLimitExceeded {
                    limit: MAX_OBJECT_MEMBERS,
                    offset: self.pos,
                });
            }
            if map.insert(key.clone(), value).is_some() {
                return Err(CanonicalError::DuplicateKey { key, offset: key_offset });
            }
            self.skip_whitespace();
            match self.peek()? {
                b',' => self.pos += 1,
                b'}' => {
                    self.pos += 1;
                    return Ok(Value::Object(map));
                }
                other => {
                    return Err(CanonicalError::UnexpectedByte { byte: other, offset: self.pos });
                }
            }
        }
    }

    /// Parses a string, returning its decoded contents.
    ///
    /// The returned string has already been length-checked against the
    /// profile's string limit, so no caller has to.
    fn parse_string(&mut self) -> Result<String, CanonicalError> {
        self.pos += 1; // consume '"'
        let mut out = String::new();
        loop {
            let byte = self.peek()?;
            match byte {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    let escape_offset = self.pos;
                    self.pos += 1;
                    let escape = self.peek()?;
                    self.pos += 1;
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let ch = self.parse_unicode_escape(escape_offset)?;
                            out.push(ch);
                        }
                        _ => {
                            return Err(CanonicalError::InvalidEscape { offset: escape_offset });
                        }
                    }
                }
                // A raw control character is not legal inside a JSON string.
                0x00..=0x1F => {
                    return Err(CanonicalError::UnexpectedByte { byte, offset: self.pos });
                }
                _ => {
                    // The input is `&str`, so the continuation bytes of a
                    // multi-byte character are already known to be well-formed.
                    let ch = self.next_char()?;
                    out.push(ch);
                }
            }
            if out.len() > MAX_STRING_BYTES {
                return Err(CanonicalError::StringLimitExceeded {
                    limit: MAX_STRING_BYTES,
                    offset: self.pos,
                });
            }
        }
    }

    /// Parses the four hex digits of a `\u` escape, and any low surrogate that
    /// must follow a high one.
    fn parse_unicode_escape(&mut self, escape_offset: usize) -> Result<char, CanonicalError> {
        let first = self.parse_hex4(escape_offset)?;
        if (0xD800..0xDC00).contains(&first) {
            // High surrogate: a low surrogate must follow, or the pair does not
            // name a character at all.
            if self.peek()? != b'\\' {
                return Err(CanonicalError::LoneSurrogate { offset: escape_offset });
            }
            self.pos += 1;
            if self.peek()? != b'u' {
                return Err(CanonicalError::LoneSurrogate { offset: escape_offset });
            }
            self.pos += 1;
            let second = self.parse_hex4(escape_offset)?;
            if !(0xDC00..0xE000).contains(&second) {
                return Err(CanonicalError::LoneSurrogate { offset: escape_offset });
            }
            let combined = 0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00);
            return char::from_u32(combined)
                .ok_or(CanonicalError::InvalidUnicodeEscape { offset: escape_offset });
        }
        if (0xDC00..0xE000).contains(&first) {
            return Err(CanonicalError::LoneSurrogate { offset: escape_offset });
        }
        char::from_u32(first).ok_or(CanonicalError::InvalidUnicodeEscape { offset: escape_offset })
    }

    fn parse_hex4(&mut self, escape_offset: usize) -> Result<u32, CanonicalError> {
        let mut value = 0u32;
        for _ in 0..4 {
            let byte = self
                .peek()
                .map_err(|_| CanonicalError::InvalidUnicodeEscape { offset: escape_offset })?;
            let digit = match byte {
                b'0'..=b'9' => u32::from(byte - b'0'),
                b'a'..=b'f' => u32::from(byte - b'a') + 10,
                b'A'..=b'F' => u32::from(byte - b'A') + 10,
                _ => {
                    return Err(CanonicalError::InvalidUnicodeEscape { offset: escape_offset });
                }
            };
            value = value * 16 + digit;
            self.pos += 1;
        }
        Ok(value)
    }

    /// Parses a canonical integer.
    ///
    /// The canonical profile has exactly one number form, so everything else
    /// — a fraction, an exponent, a leading `+`, a leading zero, a negative
    /// zero — is rejected here rather than normalised. Normalising would mean
    /// the canonical form of a document is not its input, and every comparison
    /// between two documents would have to re-derive it.
    fn parse_number(&mut self) -> Result<i64, CanonicalError> {
        let start = self.pos;
        // A leading `+` is a number with a second spelling, so it is
        // reported as a non-canonical number rather than as an unexpected
        // byte: the caller needs to know that the sign is the problem.
        if self.peek()? == b'+' {
            self.pos += 1;
            while self.peek().is_ok_and(|byte| byte.is_ascii_digit()) {
                self.pos += 1;
            }
            self.consume_number_tail();
            return Err(CanonicalError::NonCanonicalNumber {
                text: self.text_from(start),
                offset: start,
            });
        }
        let negative = self.peek()? == b'-';
        if negative {
            self.pos += 1;
        }
        match self.peek()? {
            b'0' => {
                self.pos += 1;
                if self.peek().is_ok_and(|byte| byte.is_ascii_digit()) {
                    return Err(CanonicalError::NonCanonicalNumber {
                        text: self.text_from(start),
                        offset: start,
                    });
                }
            }
            b'1'..=b'9' => {
                while self.peek().is_ok_and(|byte| byte.is_ascii_digit()) {
                    self.pos += 1;
                }
            }
            other => {
                return Err(CanonicalError::UnexpectedByte { byte: other, offset: self.pos });
            }
        }
        if negative && self.bytes[start + 1] == b'0' {
            return Err(CanonicalError::NonCanonicalNumber {
                text: self.text_from(start),
                offset: start,
            });
        }
        // A fraction or an exponent is a valid JSON number but not a
        // canonical one.
        if self.peek().is_ok_and(|byte| byte == b'.' || byte == b'e' || byte == b'E') {
            self.consume_number_tail();
            return Err(CanonicalError::NonCanonicalNumber {
                text: self.text_from(start),
                offset: start,
            });
        }
        let text = self.text_from(start);
        text.parse::<i64>().map_err(|_| CanonicalError::NumberOutOfRange { text, offset: start })
    }

    /// Consumes the fraction and exponent of a non-canonical number, so the
    /// error message shows the whole offending token rather than its head.
    fn consume_number_tail(&mut self) {
        if self.peek().is_ok_and(|byte| byte == b'.') {
            self.pos += 1;
            while self.peek().is_ok_and(|byte| byte.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if self.peek().is_ok_and(|byte| byte == b'e' || byte == b'E') {
            self.pos += 1;
            if self.peek().is_ok_and(|byte| byte == b'+' || byte == b'-') {
                self.pos += 1;
            }
            while self.peek().is_ok_and(|byte| byte.is_ascii_digit()) {
                self.pos += 1;
            }
        }
    }

    fn text_from(&self, start: usize) -> String {
        String::from_utf8_lossy(&self.bytes[start..self.pos]).into_owned()
    }

    fn next_char(&mut self) -> Result<char, CanonicalError> {
        let rest = core::str::from_utf8(&self.bytes[self.pos..])
            .map_err(|_| CanonicalError::InvalidStringEncoding { offset: self.pos })?;
        let ch = rest.chars().next().ok_or(CanonicalError::UnexpectedEnd { offset: self.pos })?;
        self.pos += ch.len_utf8();
        Ok(ch)
    }

    fn peek(&self) -> Result<u8, CanonicalError> {
        self.bytes.get(self.pos).copied().ok_or(CanonicalError::UnexpectedEnd { offset: self.pos })
    }

    fn skip_whitespace(&mut self) {
        while let Some(byte) = self.bytes.get(self.pos) {
            match byte {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }
}

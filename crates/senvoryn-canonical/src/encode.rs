//! The canonical encoder.
//!
//! Encoding cannot fail: every [`Value`] the model can represent has exactly
//! one canonical byte sequence. That is the point of the model — there is no
//! invalid state here to report.

use std::string::String;
use std::vec::Vec;

use crate::value::Value;

/// Encodes a value in canonical form.
pub fn encode_to_string(value: &Value) -> String {
    let mut out = String::new();
    encode_into(value, &mut out);
    out
}

/// Encodes a value in canonical form, as bytes.
pub fn encode_to_bytes(value: &Value) -> Vec<u8> {
    encode_to_string(value).into_bytes()
}

fn encode_into(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Int(number) => {
            out.push_str(&number.to_string());
        }
        Value::String(text) => encode_string(text, out),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                encode_into(item, out);
            }
            out.push(']');
        }
        Value::Object(members) => {
            out.push('{');
            // `BTreeMap` iterates in ascending key order, and `str`'s ordering
            // is byte-wise over UTF-8 — which is the profile's key order.
            for (index, (key, member)) in members.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                encode_string(key, out);
                out.push(':');
                encode_into(member, out);
            }
            out.push('}');
        }
    }
}

/// Escapes a string into the profile's fixed escape set.
///
/// Only the two characters that must be escaped are escaped, plus the C0
/// controls. Everything else — including `/` and every non-ASCII character —
/// is written through as UTF-8. `\uXXXX` has two spellings for the same text
/// and UTF-8 has one, so a canonical document never contains one for a
/// character that UTF-8 can already represent.
fn encode_string(text: &str, out: &mut String) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

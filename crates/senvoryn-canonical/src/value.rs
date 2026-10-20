//! The canonical document model.

use std::collections::BTreeMap;

/// A value in the canonical document model.
///
/// The type system is the enforcement: there is no float variant, so a
/// fractional number cannot be represented, and there is no `RawJson` variant
/// that would let unvalidated bytes reach the decision path.
///
/// Objects are a [`BTreeMap`], which gives two properties the profile
/// requires for free: keys are unique, and iteration order is ascending by
/// UTF-8 byte sequence — which is the profile's key order, and is not the same
/// thing as Rust's `String` ordering only by accident. `str`'s ordering is
/// byte-wise over UTF-8, so the two agree exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// JSON `null`.
    Null,
    /// JSON `true` / `false`.
    Bool(bool),
    /// A JSON integer. The profile has no fractional or exponent form.
    Int(i64),
    /// A JSON string, already decoded to Rust `String` and therefore valid UTF-8.
    String(String),
    /// A JSON array.
    Array(Vec<Value>),
    /// A JSON object with unique keys in canonical order.
    Object(BTreeMap<String, Value>),
}

impl Value {
    /// Builds an object from key/value pairs.
    ///
    /// Returns `None` if two keys are equal, because a duplicate key has no
    /// canonical meaning and silently keeping one of them would be exactly the
    /// ambiguity the profile exists to remove.
    pub fn object<const N: usize>(entries: [(&str, Value); N]) -> Option<Value> {
        let mut map = BTreeMap::new();
        for (key, value) in entries {
            if map.insert(key.to_owned(), value).is_some() {
                return None;
            }
        }
        Some(Value::Object(map))
    }

    /// The name of this value's type, for messages that do not need the value.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "boolean",
            Self::Int(_) => "integer",
            Self::String(_) => "string",
            Self::Array(_) => "array",
            Self::Object(_) => "object",
        }
    }

    /// The value as an object, if it is one.
    pub fn as_object(&self) -> Option<&BTreeMap<String, Value>> {
        match self {
            Self::Object(map) => Some(map),
            _ => None,
        }
    }

    /// The value as an array, if it is one.
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }

    /// The value as a string slice, if it is a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(text) => Some(text),
            _ => None,
        }
    }

    /// The value as an integer, if it is one.
    ///
    /// A boolean is not an integer here even though some languages allow the
    /// conversion. `true` and `1` are different claims.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(number) => Some(*number),
            _ => None,
        }
    }

    /// The value as a boolean, if it is one.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(flag) => Some(*flag),
            _ => None,
        }
    }

    /// The member of an object, if this value is an object that has it.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.as_object().and_then(|map| map.get(key))
    }
}

impl From<&str> for Value {
    fn from(text: &str) -> Self {
        Self::String(text.to_owned())
    }
}

impl From<String> for Value {
    fn from(text: String) -> Self {
        Self::String(text)
    }
}

impl From<i64> for Value {
    fn from(number: i64) -> Self {
        Self::Int(number)
    }
}

impl From<bool> for Value {
    fn from(flag: bool) -> Self {
        Self::Bool(flag)
    }
}

//! Rejection reasons for the canonical profile.
//!
//! Every variant names one specific thing the input did wrong. That is the
//! difference between a refusal a caller can act on — "a duplicate JSON key is
//! not canonical and would let two parsers disagree about the same bytes" —
//! and "invalid request", which tells the caller nothing and gets worked
//! around.

use core::fmt;

/// Why a document was rejected by the canonical profile.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CanonicalError {
    /// The input was not valid UTF-8.
    InvalidUtf8 {
        /// Byte offset of the first invalid byte.
        valid_up_to: usize,
    },

    /// The input ended in the middle of a value.
    UnexpectedEnd {
        /// Byte offset at which the input ran out.
        offset: usize,
    },

    /// A byte appeared where the profile does not allow it.
    UnexpectedByte {
        /// The byte that was found.
        byte: u8,
        /// Byte offset at which it was found.
        offset: usize,
    },

    /// The same object key appeared twice.
    ///
    /// The profile rejects this rather than resolving it, because "last one
    /// wins" and "first one wins" are both defensible readings and two
    /// implementations may pick different ones — which is precisely how the
    /// same bytes come to mean two different things.
    DuplicateKey {
        /// The key that appeared twice.
        key: String,
        /// Byte offset of the second occurrence.
        offset: usize,
    },

    /// A number was not an integer in canonical form.
    ///
    /// Canonical numbers are integers only, so `1.0`, `1e0`, `+1`, `01` and
    /// `-0` are all rejected rather than normalised. Normalising them would
    /// mean the canonical form of a document differs from its input in a way
    /// that has to be recomputed on every comparison.
    NonCanonicalNumber {
        /// The offending token, as it appeared.
        text: String,
        /// Byte offset at which the token started.
        offset: usize,
    },

    /// An integer did not fit in [`i64`].
    NumberOutOfRange {
        /// The offending token, as it appeared.
        text: String,
        /// Byte offset at which the token started.
        offset: usize,
    },

    /// A string contained an escape sequence the profile does not define.
    InvalidEscape {
        /// Byte offset of the backslash.
        offset: usize,
    },

    /// A `\uXXXX` escape did not encode a Unicode scalar value.
    InvalidUnicodeEscape {
        /// Byte offset of the backslash.
        offset: usize,
    },

    /// A `\uXXXX` escape encoded a high surrogate with no low surrogate after
    /// it.
    LoneSurrogate {
        /// Byte offset of the backslash.
        offset: usize,
    },

    /// The document nested deeper than the profile allows.
    DepthLimitExceeded {
        /// The profile's nesting limit.
        limit: usize,
        /// Byte offset at which the excess nesting was found.
        offset: usize,
    },

    /// An object had more members than the profile allows.
    MemberLimitExceeded {
        /// The profile's member limit.
        limit: usize,
        /// Byte offset at which the excess member was found.
        offset: usize,
    },

    /// An array had more elements than the profile allows.
    ArrayLimitExceeded {
        /// The profile's element limit.
        limit: usize,
        /// Byte offset at which the excess element was found.
        offset: usize,
    },

    /// A string was longer than the profile allows.
    StringLimitExceeded {
        /// The profile's string limit, in bytes.
        limit: usize,
        /// Byte offset at which the excess was found.
        offset: usize,
    },

    /// The document was longer than the profile allows.
    DocumentLimitExceeded {
        /// The profile's document limit, in bytes.
        limit: usize,
    },

    /// Content followed the top-level value.
    TrailingContent {
        /// Byte offset at which the trailing content started.
        offset: usize,
    },

    /// A string contained a byte that is not valid UTF-8.
    ///
    /// The parser works on `&str`, so this can only be reached through a
    /// conversion done by the caller. It exists so that the failure has a name
    /// rather than being a panic at the conversion site.
    InvalidStringEncoding {
        /// Byte offset at which the invalid byte was found.
        offset: usize,
    },
}

impl CanonicalError {
    /// The byte offset into the input at which the problem was detected.
    ///
    /// For [`CanonicalError::InvalidUtf8`] this is the offset of the first
    /// invalid byte, which is `valid_up_to`. For
    /// [`CanonicalError::DocumentLimitExceeded`] the document was rejected
    /// before it was looked at, so the offset is zero.
    pub fn offset(&self) -> usize {
        match self {
            Self::InvalidUtf8 { valid_up_to } => *valid_up_to,
            Self::UnexpectedEnd { offset }
            | Self::UnexpectedByte { offset, .. }
            | Self::DuplicateKey { offset, .. }
            | Self::NonCanonicalNumber { offset, .. }
            | Self::NumberOutOfRange { offset, .. }
            | Self::InvalidEscape { offset }
            | Self::InvalidUnicodeEscape { offset }
            | Self::LoneSurrogate { offset }
            | Self::DepthLimitExceeded { offset, .. }
            | Self::MemberLimitExceeded { offset, .. }
            | Self::ArrayLimitExceeded { offset, .. }
            | Self::StringLimitExceeded { offset, .. }
            | Self::TrailingContent { offset }
            | Self::InvalidStringEncoding { offset } => *offset,
            Self::DocumentLimitExceeded { .. } => 0,
        }
    }
}

impl fmt::Display for CanonicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 { valid_up_to } => write!(
                f,
                "input is not valid UTF-8 at byte {valid_up_to}; \
                 the canonical profile accepts UTF-8 only"
            ),
            Self::UnexpectedEnd { offset } => {
                write!(f, "input ended at byte {offset} in the middle of a value")
            }
            Self::UnexpectedByte { byte, offset } => write!(
                f,
                "unexpected byte 0x{byte:02x} at byte {offset}; \
                 the canonical profile accepts only the JSON grammar with \
                 integer-only numbers"
            ),
            Self::DuplicateKey { key, offset } => write!(
                f,
                "duplicate object key {key:?} at byte {offset}; \
                 the canonical profile rejects duplicate keys because two \
                 parsers could otherwise disagree about the same bytes"
            ),
            Self::NonCanonicalNumber { text, offset } => write!(
                f,
                "number {text:?} at byte {offset} is not a canonical integer; \
                 the canonical profile accepts only -?(0|[1-9][0-9]*)"
            ),
            Self::NumberOutOfRange { text, offset } => write!(
                f,
                "number {text:?} at byte {offset} does not fit in a 64-bit signed integer"
            ),
            Self::InvalidEscape { offset } => {
                write!(f, "invalid escape sequence at byte {offset}")
            }
            Self::InvalidUnicodeEscape { offset } => write!(
                f,
                "invalid \\u escape at byte {offset}; \
                 the escape must encode a Unicode scalar value"
            ),
            Self::LoneSurrogate { offset } => write!(
                f,
                "lone surrogate at byte {offset}; \
                 a high surrogate must be followed by a low surrogate"
            ),
            Self::DepthLimitExceeded { limit, offset } => write!(
                f,
                "nesting deeper than {limit} at byte {offset}; \
                 the limit is part of the profile, not of this implementation"
            ),
            Self::MemberLimitExceeded { limit, .. } => {
                write!(f, "object has more than {limit} members")
            }
            Self::ArrayLimitExceeded { limit, .. } => {
                write!(f, "array has more than {limit} elements")
            }
            Self::StringLimitExceeded { limit, .. } => {
                write!(f, "string is longer than {limit} bytes")
            }
            Self::DocumentLimitExceeded { limit } => {
                write!(f, "document is longer than {limit} bytes")
            }
            Self::TrailingContent { offset } => write!(
                f,
                "content follows the top-level value at byte {offset}; \
                 a canonical document contains exactly one value"
            ),
            Self::InvalidStringEncoding { offset } => {
                write!(f, "string at byte {offset} is not valid UTF-8")
            }
        }
    }
}

impl std::error::Error for CanonicalError {}

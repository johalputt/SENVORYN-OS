//! Senvoryn Canonical JSON Profile v1 (`SCP-CJSON-0001`).
//!
//! A canonical form is a byte encoding with exactly one representation per
//! document. Two implementations that agree on this profile agree on what a
//! document *is*; two that do not can compute different digests over the same
//! request and reach different decisions from it. This crate is where that
//! agreement is made concrete.
//!
//! # The profile
//!
//! | Rule | Why |
//! |---|---|
//! | UTF-8 only, no byte-order mark | A byte-order mark is a different byte sequence for the same document |
//! | No insignificant whitespace | Removes a second representation of the same value |
//! | Object keys in ascending UTF-8 byte order | One key order, so two encoders agree |
//! | Duplicate keys rejected | Two parsers would otherwise disagree about the same bytes |
//! | Integers only — no fraction, no exponent | Removes "ambiguous number formats" at the root |
//! | No leading `+`, no leading zeros, no `-0` | One spelling per integer |
//! | No `NaN`, no `Infinity` | Not JSON, and not representable |
//! | Fixed escape set; non-ASCII written as UTF-8 | One spelling per character |
//! | Exactly one top-level value | A second value would be invisible to a streaming reader |
//!
//! The limits — document size, nesting depth, object members, array elements,
//! string length — are part of the profile for the same reason the rest of it
//! is: two conforming implementations must reject the same documents.
//!
//! # What this crate is not
//!
//! It is **not** a general JSON library, and it is **not** a validator. It
//! produces and consumes canonical documents and nothing else. It does not
//! sign, hash, or verify anything: ADR-002 requires publicly reviewed
//! cryptographic primitives, and a hand-written digest inside the trusted core
//! would contradict that. The bytes are here; what is done with them is the
//! caller's decision.
//!
//! It is **specified, not independently reviewed**. Nothing in this project is
//! signed, so nothing is blocked by that — but the profile has not been
//! through a second pair of eyes, and the evidence ledger says so.

#![deny(missing_docs)]

mod encode;
mod error;
mod limits;
mod parse;
mod value;

pub use encode::{encode_to_bytes, encode_to_string};
pub use error::CanonicalError;
pub use parse::{
    MAX_ARRAY_ELEMENTS, MAX_DEPTH, MAX_DOCUMENT_BYTES, MAX_OBJECT_MEMBERS, MAX_STRING_BYTES,
    canonicalize, is_canonical, parse, parse_bytes,
};
pub use value::Value;

/// The identifier of the canonical profile this crate implements.
pub const PROFILE_ID: &str = "SCP-CJSON-0001";

/// The version of the canonical profile this crate implements.
pub const PROFILE_VERSION: u32 = 1;

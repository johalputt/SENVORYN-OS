//! The limits of the canonical profile.
//!
//! These are part of the profile, not settings of this implementation. Two
//! conforming implementations must reject the same documents, so the numbers
//! are named here and quoted in the specification, and a change to any of them
//! is a change to the profile.
//!
//! Each limit exists because without it there is a way to make the parser
//! spend an unbounded amount of memory or stack on an input that is under the
//! caller's control.

/// The largest document the profile accepts, in bytes.
///
/// A request envelope in this project is a few hundred bytes. A megabyte is
/// three orders of magnitude above that, which leaves room for a payload the
/// SCP must carry but not for a document designed to exhaust memory.
pub const MAX_DOCUMENT_BYTES: usize = 1_048_576;

/// The deepest nesting the profile accepts.
///
/// Every level of nesting is one more stack frame in the parser. The limit
/// keeps a document from exhausting the stack, and is checked before the
/// value is looked at, so an over-deep document is rejected without recursing.
pub const MAX_DEPTH: usize = 32;

/// The most members an object may have.
///
/// Bounds the work of the duplicate-key check and the memory of the map.
pub const MAX_OBJECT_MEMBERS: usize = 1024;

/// The most elements an array may have.
///
/// Bounds the memory of the vector.
pub const MAX_ARRAY_ELEMENTS: usize = 4096;

/// The longest a string may be, in bytes.
///
/// Bounds the memory of the decoded string, and therefore the size of the
/// document being built.
pub const MAX_STRING_BYTES: usize = 65_536;

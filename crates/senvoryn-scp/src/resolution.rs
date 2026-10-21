//! What a name resolves to.
//!
//! The SCP does not resolve resources: it has no filesystem, no resolver and
//! no trust store, and it is not the component that knows what objects exist.
//! The environment does. What arrives here is the result of that resolution —
//! the canonical identity of an object, as the environment knows it.
//!
//! The distinction the record draws is between the *name* a caller used and
//! the *identity* the name means. A caller may address an object by an alias,
//! and the alias is legitimate; what the decision path must compare against is
//! the canonical identity, because that is what a grant is over and what the
//! registry's rules are keyed by.

/// The canonical identity of a resource, as the environment resolved it.
///
/// The three fields are the ones the decision path needs and no more: the
/// identifier the rest of the system knows the object by, the type the
/// object has, and the version the object was at when it was resolved. The
/// wider identity the specification states — owner, sensitivity, lifecycle
/// state, supported operations — belongs to the resource record, and which of
/// it the decision path will need is still open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedResource {
    /// The identifier the rest of the system knows the object by.
    pub canonical_id: String,
    /// The type the object has, as the environment knows it.
    pub resource_type: String,
    /// The version of the object at the moment of resolution.
    pub version: String,
}

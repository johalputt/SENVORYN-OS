//! The capability model, and the attenuation rule.

use std::collections::BTreeSet;

/// How the issuer's authority was established.
///
/// Not a boolean. A boolean that defaults to `true` lets a capability that
/// nobody verified be treated as verified by omission, and that is the
/// single most common way a control is neutralised. A capability cannot be
/// constructed without saying which of these it has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// The issuer's authority was verified against the trust store.
    Verified,
    /// The issuer could not be checked. Not the same as checked and failed.
    Unchecked,
    /// Verification was attempted and the proof did not hold.
    Invalid,
}

impl Provenance {
    /// The provenance as it appears in a decision record.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "VERIFIED",
            Self::Unchecked => "UNCHECKED",
            Self::Invalid => "INVALID",
        }
    }

    /// Reads a provenance from its serialized form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "VERIFIED" => Some(Self::Verified),
            "UNCHECKED" => Some(Self::Unchecked),
            "INVALID" => Some(Self::Invalid),
            _ => None,
        }
    }
}

/// A capability grant.
///
/// A capability binds a subject to a resource, a set of operations, a set of
/// purposes, a validity window, a policy epoch, and its provenance. Anything
/// the holder of the capability is not bound by is not theirs to exercise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    /// The capability's own identifier.
    pub capability_id: String,
    /// The principal the capability was granted to.
    pub subject_id: String,
    /// The resource the capability covers.
    pub resource_id: String,
    /// The kind of resource the capability covers.
    pub resource_type: String,
    /// The operations the capability permits.
    pub operations: BTreeSet<String>,
    /// The purposes the capability permits.
    pub purposes: BTreeSet<String>,
    /// When the capability becomes usable, in monotonic milliseconds.
    pub not_before: u64,
    /// When the capability stops being usable, in monotonic milliseconds.
    pub expires_at: u64,
    /// The policy generation the capability was granted under.
    pub policy_epoch: u64,
    /// How the issuer's authority was established.
    pub provenance: Provenance,
    /// How much delegation budget the capability carries.
    pub delegation_depth: u32,
    /// The most times the capability may be used, if it is limited.
    pub max_uses: Option<u64>,
}

/// Returns whether `child` is equal to or narrower than `parent`.
///
/// Specification §4.2. A capability may only ever be narrowed on the way
/// down a delegation chain; a child that is broader in any dimension is not
/// a valid attenuation of its parent.
///
/// The delegation-depth rule follows the specification's wording literally —
/// "child delegation depth must decrease" — even though the capability
/// schema's `delegation_depth` also reads as a chain depth, which would make
/// it increase. The two readings cannot both hold, the specification is the
/// normative text, and the ambiguity is recorded as an open decision rather
/// than resolved silently here.
pub fn attenuate(parent: &Capability, child: &Capability) -> bool {
    child.subject_id == parent.subject_id
        && child.resource_id == parent.resource_id
        && child.resource_type == parent.resource_type
        && child.operations.is_subset(&parent.operations)
        && child.purposes.is_subset(&parent.purposes)
        && child.not_before >= parent.not_before
        && child.expires_at <= parent.expires_at
        && child.policy_epoch == parent.policy_epoch
        && child.delegation_depth < parent.delegation_depth
        && match parent.max_uses {
            None => true,
            Some(limit) => child.max_uses.is_some_and(|uses| uses <= limit),
        }
}

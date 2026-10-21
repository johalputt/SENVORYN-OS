//! Reason codes, and the denial-code registry that is their authority.
//!
//! A reason code is a `&'static str` in a newtype rather than an enum
//! because the registry grows: a new code must not require a change to every
//! match in the crate, and it must not be possible to add a code that the
//! registry does not contain.
//!
//! The registry is embedded at compile time and is the single authority on
//! which codes exist. `tests/reason_code_registry.rs` checks this crate's
//! codes against it in both directions, so a code added on one side and not
//! the other fails the build rather than surviving to a release.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use senvoryn_canonical::{Value, parse};

use crate::decision::DecisionStatus;

/// The denial-code registry, embedded at compile time.
const REGISTRY_JSON: &str = include_str!("../../../errors/denial-codes.json");

/// A reason code.
///
/// Every code the evaluator can produce exists in the denial-code registry,
/// with one exception: the permitting reason, which is not a denial and so is
/// not in a denial registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReasonCode(&'static str);

impl ReasonCode {
    /// The reason a permitted decision carries.
    pub const ALL_POLICY_GATES_PASSED: Self = Self("ALL_POLICY_GATES_PASSED");

    /// The envelope is malformed, or does not identify a principal.
    pub const MALFORMED_OR_MISSING_IDENTITY: Self = Self("MALFORMED_OR_MISSING_IDENTITY");
    /// The request is addressed to another node or service.
    pub const WRONG_RECIPIENT: Self = Self("WRONG_RECIPIENT");
    /// The principal is not known to this node.
    pub const UNKNOWN_PRINCIPAL: Self = Self("UNKNOWN_PRINCIPAL");
    /// The request has expired, or is not yet valid.
    pub const REQUEST_NOT_FRESH: Self = Self("REQUEST_NOT_FRESH");
    /// The nonce has been seen before, or has been revoked.
    pub const REPLAY_OR_REVOKED_NONCE: Self = Self("REPLAY_OR_REVOKED_NONCE");
    /// The resource does not resolve to an object.
    pub const RESOURCE_UNRESOLVABLE: Self = Self("RESOURCE_UNRESOLVABLE");
    /// The operation is unknown, or is not defined for this resource type.
    pub const UNKNOWN_OPERATION_RISK: Self = Self("UNKNOWN_OPERATION_RISK");
    /// The caller claimed less risk than the registry assigns.
    pub const RISK_CLAIM_MISMATCH: Self = Self("RISK_CLAIM_MISMATCH");
    /// The request was made under a policy epoch this node does not accept.
    pub const POLICY_EPOCH_MISMATCH: Self = Self("POLICY_EPOCH_MISMATCH");
    /// The operation registry in force is not the registry for the epoch this
    /// node is running.
    pub const REGISTRY_EPOCH_MISMATCH: Self = Self("REGISTRY_EPOCH_MISMATCH");
    /// No capability was presented.
    pub const MISSING_CAPABILITY: Self = Self("MISSING_CAPABILITY");
    /// The capability's issuer could not be verified.
    pub const UNVERIFIED_CAPABILITY_ISSUER: Self = Self("UNVERIFIED_CAPABILITY_ISSUER");
    /// The capability has been revoked.
    pub const CAPABILITY_REVOKED: Self = Self("CAPABILITY_REVOKED");
    /// The capability belongs to another principal.
    pub const CAPABILITY_SUBJECT_MISMATCH: Self = Self("CAPABILITY_SUBJECT_MISMATCH");
    /// The capability does not cover the requested resource.
    pub const CAPABILITY_RESOURCE_MISMATCH: Self = Self("CAPABILITY_RESOURCE_MISMATCH");
    /// The capability does not cover the requested operation.
    pub const CAPABILITY_OPERATION_MISMATCH: Self = Self("CAPABILITY_OPERATION_MISMATCH");
    /// The capability does not cover the requested purpose.
    pub const CAPABILITY_PURPOSE_MISMATCH: Self = Self("CAPABILITY_PURPOSE_MISMATCH");
    /// The capability is outside its validity window.
    pub const CAPABILITY_NOT_CURRENT: Self = Self("CAPABILITY_NOT_CURRENT");
    /// The capability was granted under a policy epoch this node does not
    /// accept.
    pub const CAPABILITY_POLICY_EPOCH_MISMATCH: Self = Self("CAPABILITY_POLICY_EPOCH_MISMATCH");
    /// The current system mode does not permit the operation.
    pub const OPERATION_BLOCKED_BY_SYSTEM_MODE: Self = Self("OPERATION_BLOCKED_BY_SYSTEM_MODE");
    /// The action needs an approval bound to this exact action.
    pub const FRESH_ACTION_BOUND_APPROVAL_REQUIRED: Self =
        Self("FRESH_ACTION_BOUND_APPROVAL_REQUIRED");
    /// The action needs a strong approval that has not been given.
    pub const STRONG_APPROVAL_REQUIRED: Self = Self("STRONG_APPROVAL_REQUIRED");
    /// The audit sink cannot record the required audit class.
    pub const AUDIT_UNAVAILABLE_FOR_SENSITIVE_ACTION: Self =
        Self("AUDIT_UNAVAILABLE_FOR_SENSITIVE_ACTION");
    /// The request asks for more than the policy allows.
    pub const RESOURCE_BUDGET_EXCEEDED: Self = Self("RESOURCE_BUDGET_EXCEEDED");
    /// A precondition the operation requires does not hold.
    pub const PRECONDITION_FAILED: Self = Self("PRECONDITION_FAILED");
    /// The document uses a schema this node does not support.
    pub const UNSUPPORTED_CRITICAL_SCHEMA: Self = Self("UNSUPPORTED_CRITICAL_SCHEMA");
    /// A signature did not verify.
    pub const SIGNATURE_INVALID: Self = Self("SIGNATURE_INVALID");
    /// The request attempts to roll the policy back.
    pub const ANTI_ROLLBACK_VIOLATION: Self = Self("ANTI_ROLLBACK_VIOLATION");

    /// Every code this crate can produce, including the permitting reason.
    pub const ALL: &'static [ReasonCode] = &[
        Self::ALL_POLICY_GATES_PASSED,
        Self::MALFORMED_OR_MISSING_IDENTITY,
        Self::WRONG_RECIPIENT,
        Self::UNKNOWN_PRINCIPAL,
        Self::REQUEST_NOT_FRESH,
        Self::REPLAY_OR_REVOKED_NONCE,
        Self::RESOURCE_UNRESOLVABLE,
        Self::UNKNOWN_OPERATION_RISK,
        Self::RISK_CLAIM_MISMATCH,
        Self::POLICY_EPOCH_MISMATCH,
        Self::REGISTRY_EPOCH_MISMATCH,
        Self::MISSING_CAPABILITY,
        Self::UNVERIFIED_CAPABILITY_ISSUER,
        Self::CAPABILITY_REVOKED,
        Self::CAPABILITY_SUBJECT_MISMATCH,
        Self::CAPABILITY_RESOURCE_MISMATCH,
        Self::CAPABILITY_OPERATION_MISMATCH,
        Self::CAPABILITY_PURPOSE_MISMATCH,
        Self::CAPABILITY_NOT_CURRENT,
        Self::CAPABILITY_POLICY_EPOCH_MISMATCH,
        Self::OPERATION_BLOCKED_BY_SYSTEM_MODE,
        Self::FRESH_ACTION_BOUND_APPROVAL_REQUIRED,
        Self::STRONG_APPROVAL_REQUIRED,
        Self::AUDIT_UNAVAILABLE_FOR_SENSITIVE_ACTION,
        Self::RESOURCE_BUDGET_EXCEEDED,
        Self::PRECONDITION_FAILED,
        Self::UNSUPPORTED_CRITICAL_SCHEMA,
        Self::SIGNATURE_INVALID,
        Self::ANTI_ROLLBACK_VIOLATION,
    ];

    /// The code as it appears in a decision record.
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl core::fmt::Display for ReasonCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.0)
    }
}

/// One entry of the denial-code registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeEntry {
    /// The code itself.
    pub code: &'static str,
    /// The class the code belongs to.
    pub class: &'static str,
    /// The decision status the code produces by default.
    pub default_status: DecisionStatus,
}

/// The denial-code registry, parsed from the embedded artifact.
///
/// The strings are leaked once for the life of the process. The registry is
/// small, fixed at compile time, and read on every decision, so this is the
/// one place where a deliberate leak is the right trade: it lets the codes be
/// `&'static str` without copying them on every lookup.
fn registry() -> &'static BTreeMap<&'static str, CodeEntry> {
    static REGISTRY: OnceLock<BTreeMap<&'static str, CodeEntry>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        parse_registry().unwrap_or_else(|error| {
            // The registry is a compile-time constant under this project's
            // control, and a test asserts that it parses. If it somehow does
            // not, the registry is empty and every default-status lookup
            // falls back to DENY — the direction that fails closed.
            eprintln!("the embedded denial-code registry could not be parsed: {error}");
            BTreeMap::new()
        })
    })
}

fn parse_registry() -> Result<BTreeMap<&'static str, CodeEntry>, String> {
    let document = parse(REGISTRY_JSON).map_err(|error| error.to_string())?;
    let codes = document
        .get("codes")
        .and_then(Value::as_array)
        .ok_or_else(|| "the registry has no \"codes\" array".to_owned())?;
    let mut map = BTreeMap::new();
    for entry in codes {
        let code = entry
            .get("code")
            .and_then(Value::as_str)
            .ok_or_else(|| "a registry entry has no \"code\"".to_owned())?;
        let class = entry
            .get("class")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("registry entry {code} has no \"class\""))?;
        let default = entry
            .get("default")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("registry entry {code} has no \"default\""))?;
        let default_status = match default {
            "DENY" => DecisionStatus::Deny,
            "CHALLENGE" => DecisionStatus::Challenge,
            "DEFER" => DecisionStatus::Defer,
            "QUARANTINE" => DecisionStatus::Quarantine,
            "PERMIT" => DecisionStatus::Permit,
            other => return Err(format!("registry entry {code} has an unknown default {other:?}")),
        };
        let code: &'static str = Box::leak(code.to_owned().into_boxed_str());
        let class: &'static str = Box::leak(class.to_owned().into_boxed_str());
        map.insert(code, CodeEntry { code, class, default_status });
    }
    Ok(map)
}

/// The codes in the denial-code registry.
pub fn registry_codes() -> &'static BTreeSet<&'static str> {
    static CODES: OnceLock<BTreeSet<&'static str>> = OnceLock::new();
    CODES.get_or_init(|| registry().keys().copied().collect())
}

/// The registry's entry for a code, if it has one.
pub fn registry_entry(code: &str) -> Option<&'static CodeEntry> {
    registry().get(code)
}

/// The decision status the registry assigns to a code by default.
///
/// A code the registry does not contain has no default, and the caller must
/// decide. This returns `None` rather than guessing.
pub fn registry_default_status(code: &str) -> Option<DecisionStatus> {
    registry_entry(code).map(|entry| entry.default_status)
}

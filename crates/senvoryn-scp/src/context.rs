//! The state the policy environment provides to the evaluator.

use std::collections::{BTreeMap, BTreeSet};

use crate::registry::OperationRegistry;
use crate::resolution::ResolvedResource;

/// The system state the node is in.
///
/// A mode is not a label on a screen: it is enforced on every privileged
/// request, and it is checked against the registry's declared mode set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemMode {
    /// Everything works as intended.
    Normal,
    /// Some capability is unavailable.
    Degraded,
    /// Only non-mutating operations are permitted.
    ReadOnly,
    /// The node is recovering.
    Recovery,
    /// The node is isolated.
    Quarantined,
    /// The node is under maintenance.
    Maintenance,
    /// The node is disconnected.
    Offline,
}

impl SystemMode {
    /// The mode as it appears in a registry entry.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Degraded => "DEGRADED",
            Self::ReadOnly => "READ_ONLY",
            Self::Recovery => "RECOVERY",
            Self::Quarantined => "QUARANTINED",
            Self::Maintenance => "MAINTENANCE",
            Self::Offline => "OFFLINE",
        }
    }

    /// Reads a mode from its serialized form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "NORMAL" => Some(Self::Normal),
            "DEGRADED" => Some(Self::Degraded),
            "READ_ONLY" => Some(Self::ReadOnly),
            "RECOVERY" => Some(Self::Recovery),
            "QUARANTINED" => Some(Self::Quarantined),
            "MAINTENANCE" => Some(Self::Maintenance),
            "OFFLINE" => Some(Self::Offline),
            _ => None,
        }
    }
}

/// Whether the audit sink can record the audit class an operation needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditState {
    /// The sink is up and can record the required class.
    Ready,
    /// The sink cannot record what this action needs.
    Unavailable,
}

impl AuditState {
    /// The state as it appears in a serialized context.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Unavailable => "UNAVAILABLE",
        }
    }

    /// Reads a state from its serialized form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "READY" => Some(Self::Ready),
            "UNAVAILABLE" => Some(Self::Unavailable),
            _ => None,
        }
    }
}

/// Whether the wall clock may be believed.
///
/// A wall clock that has moved backwards is refused rather than believed: it
/// would make an expired document look current, which is the one way a clock
/// can grant authority. The environment states its own trust here, the same
/// way it states whether the audit sink is up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockState {
    /// The wall clock has not moved backwards since the last reading.
    Trusted,
    /// The wall clock has moved backwards, or cannot be read.
    Untrusted,
}

impl ClockState {
    /// The state as it appears in a serialized context.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trusted => "TRUSTED",
            Self::Untrusted => "UNTRUSTED",
        }
    }

    /// Reads a state from its serialized form.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "TRUSTED" => Some(Self::Trusted),
            "UNTRUSTED" => Some(Self::Untrusted),
            _ => None,
        }
    }
}

/// Everything the policy environment knows.
///
/// Every field here is something the environment *knows*, never something the
/// caller asserts. `audit_state` in particular has no default: a context that
/// never says whether the audit sink is up must not be able to imply that it
/// is. The same is true of `resolved_resources` and `known_principals` — a
/// caller cannot be its own authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyContext {
    /// The node or service this context belongs to.
    pub local_node_id: String,
    /// The policy generation this node currently accepts.
    pub current_policy_epoch: u64,
    /// The current time, in monotonic milliseconds since boot.
    pub now_monotonic: u64,
    /// The current wall-clock time, in milliseconds since the Unix epoch.
    ///
    /// The monotonic clock is the session's authority and is what the
    /// request's freshness and the capability's window are measured with. The
    /// wall clock is read only to judge a published document's validity
    /// window, and only while `clock_state` says it may be believed.
    pub now_wall_ms: i64,
    /// Whether the wall clock may be believed.
    pub clock_state: ClockState,
    /// The system state the node is in.
    pub mode: SystemMode,
    /// The principals this node knows.
    pub known_principals: BTreeSet<String>,
    /// The resources this node can resolve, and what each name resolves to.
    ///
    /// The key is the name a caller may use — which may be an alias — and
    /// the value is the canonical identity of the object that name means.
    /// A caller cannot be its own authority here either: the table is the
    /// environment's, and the decision path reads the canonical identity out
    /// of it rather than out of the request.
    pub resolved_resources: BTreeMap<String, ResolvedResource>,
    /// The trusted Operation Registry.
    pub operations: OperationRegistry,
    /// Whether the audit sink can record what an action needs.
    pub audit_state: AuditState,
    /// The digest the approval service has bound to the current action.
    pub action_digest: Option<String>,
    /// Nonces that have been seen and must not be replayed.
    pub revoked_nonces: BTreeSet<String>,
    /// Capabilities that have been revoked.
    pub revoked_capabilities: BTreeSet<String>,
    /// Whether a strong approval has been granted for the current action.
    pub strong_approval_granted: bool,
    /// The preconditions the environment reports as currently holding.
    pub satisfied_preconditions: BTreeSet<String>,
}

//! The audit-event builder.
//!
//! Every decision leaves a record. The builder is deterministic: the same
//! decision and the same inputs produce the same event, byte for byte, so
//! that a record can be checked against a digest and a replay can be
//! detected. Nothing here reads a clock or generates a random identifier —
//! the sequence and the event identifier are the audit sink's to assign, and
//! the sink is the component that knows what has already been written.

use std::collections::BTreeMap;

use senvoryn_canonical::Value;

use crate::decision::Decision;
use crate::request::Request;

/// How the time in an audit event was established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustedTimeStatus {
    /// Both a trusted wall clock and a monotonic clock were available.
    TrustedWallAndMonotonic,
    /// Only the monotonic clock was available.
    MonotonicOnly,
    /// The time was not trusted.
    Untrusted,
    /// No time was available.
    Unavailable,
}

impl TrustedTimeStatus {
    /// The status as it appears in an audit event.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TrustedWallAndMonotonic => "TRUSTED_WALL_AND_MONOTONIC",
            Self::MonotonicOnly => "MONOTONIC_ONLY",
            Self::Untrusted => "UNTRUSTED",
            Self::Unavailable => "UNAVAILABLE",
        }
    }
}

/// The classification of the data an action touched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataClassification {
    /// Publicly releasable.
    Public,
    /// Internal to the installation.
    Internal,
    /// Personal data.
    Personal,
    /// Sensitive data.
    Sensitive,
    /// Secret data.
    Secret,
    /// Security evidence.
    SecurityEvidence,
}

impl DataClassification {
    /// The classification as it appears in an audit event.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "PUBLIC",
            Self::Internal => "INTERNAL",
            Self::Personal => "PERSONAL",
            Self::Sensitive => "SENSITIVE",
            Self::Secret => "SECRET",
            Self::SecurityEvidence => "SECURITY_EVIDENCE",
        }
    }
}

/// Builds the audit event for a decision.
///
/// The result is a document shaped like `senvoryn.audit-event.v1`. The
/// `audit_ref` a decision record carries is the `event_id` of the event built
/// here, which is how a decision is tied to its record.
///
/// The event names the canonical object the decision was about, because the
/// event is the system's own record of what it acted on and not a copy of
/// what the caller asked for. A decision refused before the resource was
/// resolved names the identifier the request used.
#[allow(clippy::too_many_arguments)]
pub fn build_audit_event(
    decision: &Decision,
    request: &Request,
    event_id: &str,
    sequence: u64,
    trusted_time_status: TrustedTimeStatus,
    data_classification: DataClassification,
    previous_event_digest: Option<&str>,
) -> Value {
    let mut event = BTreeMap::new();
    event.insert("schema".to_owned(), Value::from("senvoryn.audit-event.v1"));
    event.insert("event_id".to_owned(), Value::from(event_id));
    event.insert("sequence".to_owned(), Value::Int(i64::try_from(sequence).unwrap_or(i64::MAX)));
    event.insert("event_type".to_owned(), Value::from("scp.decision"));
    event.insert("actor_id".to_owned(), Value::from(request.principal_id.clone()));
    event.insert(
        "policy_epoch".to_owned(),
        Value::Int(i64::try_from(decision.policy_epoch()).unwrap_or(i64::MAX)),
    );
    event.insert("outcome".to_owned(), Value::from(decision.status().as_str()));
    event.insert("reason_code".to_owned(), Value::from(decision.reason().as_str()));
    event.insert("trusted_time_status".to_owned(), Value::from(trusted_time_status.as_str()));
    event.insert("data_classification".to_owned(), Value::from(data_classification.as_str()));
    event.insert(
        "previous_event_digest".to_owned(),
        match previous_event_digest {
            Some(digest) => Value::from(digest),
            None => Value::Null,
        },
    );
    event.insert(
        "resource_id".to_owned(),
        match decision.resolved() {
            Some(resolved) => Value::from(resolved.canonical_id.clone()),
            None => Value::from(request.resource_id.clone()),
        },
    );
    event.insert("request_id".to_owned(), Value::from(request.request_id.clone()));
    event.insert(
        "capability_id".to_owned(),
        match decision.capability_id() {
            Some(capability_id) => Value::from(capability_id),
            None => Value::Null,
        },
    );
    Value::Object(event)
}

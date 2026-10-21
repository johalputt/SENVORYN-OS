//! The decision vocabulary and the decision itself.

use std::collections::BTreeMap;

use senvoryn_canonical::Value;

use crate::reason::ReasonCode;
use crate::resolution::ResolvedResource;

/// The decision vocabulary of specification §3.3.
///
/// A closed enum, so that a new status is a compile error everywhere it has
/// to be handled rather than a string that some match arms forget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DecisionStatus {
    /// Every mandatory check passed.
    Permit,
    /// A mandatory check failed.
    Deny,
    /// More evidence or explicit approval is required. No effect occurs.
    Challenge,
    /// The operation is non-destructive and waits for a dependency or window.
    Defer,
    /// The request is isolated by policy.
    Quarantine,
}

impl DecisionStatus {
    /// The status as it appears in a decision record.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Permit => "PERMIT",
            Self::Deny => "DENY",
            Self::Challenge => "CHALLENGE",
            Self::Defer => "DEFER",
            Self::Quarantine => "QUARANTINE",
        }
    }

    /// Whether this status grants any authority.
    ///
    /// `CHALLENGE` and `DEFER` grant no partial execution authority, which is
    /// specification §3.3 in as many words.
    pub fn grants_authority(&self) -> bool {
        matches!(self, Self::Permit)
    }

    /// The status a decision record names, if it names one this node knows.
    ///
    /// The vocabulary is closed, so an unknown status is a refusal to read
    /// the record rather than a status to be treated as `DENY`: a record
    /// that says something this node does not understand is a record this
    /// node cannot act on.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "PERMIT" => Some(Self::Permit),
            "DENY" => Some(Self::Deny),
            "CHALLENGE" => Some(Self::Challenge),
            "DEFER" => Some(Self::Defer),
            "QUARANTINE" => Some(Self::Quarantine),
            _ => None,
        }
    }
}

impl core::fmt::Display for DecisionStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What a deferred decision is waiting for.
///
/// A deferral is a decision that says "not yet", so it has to say what it is
/// waiting for. A deferral that waits for nothing is a decision that never
/// arrives, and the record would publish a next step that names no event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Deferral {
    /// The policy epoch in force must advance to this epoch.
    ///
    /// The request named a policy epoch this node has not reached yet, so the
    /// policy that will govern it does not exist yet.
    PolicyEpoch(u64),
}

impl Deferral {
    /// The next step a caller must take, as the decision record publishes it.
    ///
    /// The wording belongs to this implementation; the thing it names does
    /// not. The epoch is the deferral's substance and it is in the decision
    /// itself, where a gate cannot disagree with it.
    pub fn required_next_step(&self) -> String {
        match self {
            Self::PolicyEpoch(epoch) => {
                format!("wait for policy epoch {epoch} to come into force")
            }
        }
    }
}

/// The result of evaluating one request.
///
/// The decision grants nothing by itself. Effect separation is the sixteenth
/// of the sixteen gates, and it is enforced by this type having no field in
/// which an effect could be recorded: there is nowhere to put one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    status: DecisionStatus,
    reason: ReasonCode,
    capability_id: Option<String>,
    policy_epoch: u64,
    resolved: Option<ResolvedResource>,
    deferral: Option<Deferral>,
}

impl Decision {
    /// A decision that refuses the request.
    pub fn deny(reason: ReasonCode, policy_epoch: u64) -> Self {
        Self {
            status: DecisionStatus::Deny,
            reason,
            capability_id: None,
            policy_epoch,
            resolved: None,
            deferral: None,
        }
    }

    /// A decision that asks for more evidence or approval.
    pub fn challenge(reason: ReasonCode, policy_epoch: u64) -> Self {
        Self {
            status: DecisionStatus::Challenge,
            reason,
            capability_id: None,
            policy_epoch,
            resolved: None,
            deferral: None,
        }
    }

    /// A decision that waits for a policy window to open.
    ///
    /// The deferral names what it waits for, because a deferral that waits for
    /// nothing is a decision that never arrives. It grants no authority: the
    /// capability scope stays empty, and `grants_authority` is false for
    /// every status but `PERMIT`.
    pub fn defer(reason: ReasonCode, deferral: Deferral, policy_epoch: u64) -> Self {
        Self {
            status: DecisionStatus::Defer,
            reason,
            capability_id: None,
            policy_epoch,
            resolved: None,
            deferral: Some(deferral),
        }
    }

    /// A decision that permits the request, scoped by a capability.
    pub fn permit(reason: ReasonCode, capability_id: String, policy_epoch: u64) -> Self {
        Self {
            status: DecisionStatus::Permit,
            reason,
            capability_id: Some(capability_id),
            policy_epoch,
            resolved: None,
            deferral: None,
        }
    }

    /// The decision status.
    pub fn status(&self) -> DecisionStatus {
        self.status
    }

    /// The reason the decision was reached.
    pub fn reason(&self) -> ReasonCode {
        self.reason
    }

    /// The capability the authority is scoped to, if any.
    pub fn capability_id(&self) -> Option<&str> {
        self.capability_id.as_deref()
    }

    /// The policy epoch the decision was made under.
    pub fn policy_epoch(&self) -> u64 {
        self.policy_epoch
    }

    /// The canonical identity the decision was made about, if the request
    /// resolved to one.
    ///
    /// A decision that names nothing was refused before the resource was
    /// resolved, and the record for such a decision names the identifier the
    /// request used instead.
    pub fn resolved(&self) -> Option<&ResolvedResource> {
        self.resolved.as_ref()
    }

    /// Attaches the canonical identity the decision was made about.
    pub fn with_resolved(mut self, resolved: ResolvedResource) -> Self {
        self.resolved = Some(resolved);
        self
    }

    /// What the decision waits for, if it waits for anything.
    ///
    /// `None` for every decision that is not a deferral. It is not a
    /// wildcard: a decision that waits for nothing is not waiting.
    pub fn deferral(&self) -> Option<&Deferral> {
        self.deferral.as_ref()
    }

    /// Whether the decision grants any authority.
    pub fn grants_authority(&self) -> bool {
        self.status.grants_authority() && self.capability_id.is_some()
    }

    /// Renders the decision as a `senvoryn.decision-record.v1` document.
    ///
    /// The record's `effect_performed` is always false. The schema says
    /// `const: false`, and that is not decoration: effect separation is the
    /// sixteenth of the sixteen gates, and the evaluator has no way to
    /// perform one.
    ///
    /// The record names the canonical object the decision was about, and the
    /// version of it the decision was made against. The identifier the
    /// request used is the fallback for a decision that was refused before
    /// the resource was resolved.
    ///
    /// The record's `required_next_step` comes from the decision itself, not
    /// from the caller of this method. A deferral is the one status that
    /// publishes a next step, and what it waits for is the deferral's own
    /// substance — a value the caller could supply would be a value the
    /// caller could disagree with, and ADR-021's rule applies to it as much
    /// as to the plan: the authority has to arrive as the thing the
    /// authority wrote.
    pub fn to_record(&self, details: &RecordDetails<'_>) -> Value {
        let mut record = BTreeMap::new();
        record.insert("schema".to_owned(), Value::from("senvoryn.decision-record.v1"));
        record.insert("decision_id".to_owned(), Value::from(details.decision_id));
        record.insert("request_id".to_owned(), Value::from(details.request_id));
        record.insert("status".to_owned(), Value::from(self.status.as_str()));
        record.insert(
            "reason_codes".to_owned(),
            Value::Array(vec![Value::from(self.reason.as_str())]),
        );
        record.insert(
            "policy_epoch".to_owned(),
            Value::Int(i64::try_from(self.policy_epoch).unwrap_or(i64::MAX)),
        );
        record.insert("evaluated_at".to_owned(), Value::from(details.evaluated_at));
        record.insert(
            "target_resource_id".to_owned(),
            match &self.resolved {
                Some(resolved) => Value::from(resolved.canonical_id.clone()),
                None => Value::from(details.presented_resource_id),
            },
        );
        record.insert(
            "target_resource_version".to_owned(),
            match &self.resolved {
                Some(resolved) => Value::from(resolved.version.clone()),
                None => Value::Null,
            },
        );
        record.insert(
            "permitted_payload_digest".to_owned(),
            match details.permitted_payload_digest {
                Some(digest) => Value::from(digest),
                None => Value::Null,
            },
        );
        record.insert("effect_performed".to_owned(), Value::Bool(false));
        record.insert("audit_ref".to_owned(), Value::from(details.audit_ref));
        record.insert(
            "capability_scope_ref".to_owned(),
            match &self.capability_id {
                Some(capability_id) => Value::from(capability_id.clone()),
                None => Value::Null,
            },
        );
        record.insert(
            "valid_until".to_owned(),
            match details.valid_until {
                Some(until) => Value::from(until),
                None => Value::Null,
            },
        );
        record.insert(
            "required_next_step".to_owned(),
            match &self.deferral {
                Some(deferral) => Value::from(deferral.required_next_step()),
                None => Value::Null,
            },
        );
        Value::Object(record)
    }
}

/// The details a decision record needs that the decision does not carry.
///
/// The decision says what was decided; the record says which decision it was,
/// when it was made, and which audit event it produced.
#[derive(Debug, Clone, Copy)]
pub struct RecordDetails<'a> {
    /// The decision's own identifier.
    pub decision_id: &'a str,
    /// The request the decision was made for.
    pub request_id: &'a str,
    /// When the decision was made, as a date-time.
    pub evaluated_at: &'a str,
    /// The identifier the request used.
    ///
    /// The record names the canonical object when the request resolved to
    /// one; this is the fallback for a decision that was refused before the
    /// resource was resolved.
    pub presented_resource_id: &'a str,
    /// The digest of the plan the request named.
    ///
    /// The record carries the request's claim about the plan it asked to
    /// run, and the executor compares it with the plan it was handed. A
    /// request that named no plan leaves it null, and a null permits no
    /// plan: it is a decision about an operation, not about a plan.
    pub permitted_payload_digest: Option<&'a str>,
    /// The audit event that records this decision.
    pub audit_ref: &'a str,
    /// How long the granted authority lasts.
    pub valid_until: Option<&'a str>,
}

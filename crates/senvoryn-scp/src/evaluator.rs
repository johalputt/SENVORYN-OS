//! The decision oracle: the sixteen validation gates of specification §3.2.
//!
//! The evaluator is a pure function. It reads no clock, performs no I/O,
//! mutates nothing, and returns a decision that grants nothing by itself.
//! Time arrives as a value in the context, which is the only way a decision
//! about expiry can be tested at all.

use crate::capability::{Capability, Provenance};
use crate::context::{AuditState, PolicyContext};
use crate::decision::Decision;
use crate::reason::ReasonCode;
use crate::registry::{AuditClass, OperationEntry, RiskTier};
use crate::request::Request;

/// Evaluates one request against the policy context.
///
/// The gates are evaluated in the order specification §3.2 gives them, and the
/// first one that fails is the reason reported. The order is not cosmetic:
/// the recipient gate comes before the identity gate so that a request
/// addressed elsewhere does not learn whether the principal exists, and the
/// mode gate comes before the epoch gate so that a node in read-only mode
/// does not reveal whether the caller's policy epoch is current.
pub fn evaluate(
    request: &Request,
    capability: Option<&Capability>,
    context: &PolicyContext,
) -> Decision {
    let epoch = context.current_policy_epoch;

    // Gate 2 — canonical parse. The structural half: every field a later
    // gate relies on must be present before any of them is read.
    if request.request_id.is_empty()
        || request.principal_id.is_empty()
        || request.recipient_id.is_empty()
        || request.resource_id.is_empty()
        || request.resource_type.is_empty()
        || request.operation.is_empty()
        || request.purpose.is_empty()
        || request.nonce.is_empty()
    {
        return Decision::deny(ReasonCode::MALFORMED_OR_MISSING_IDENTITY, epoch);
    }

    // Gate 3 — recipient.
    if request.recipient_id != context.local_node_id {
        return Decision::deny(ReasonCode::WRONG_RECIPIENT, epoch);
    }

    // Gate 4 — identity.
    if !context.known_principals.contains(&request.principal_id) {
        return Decision::deny(ReasonCode::UNKNOWN_PRINCIPAL, epoch);
    }

    // Gate 5 — freshness: nonce, replay cache, expiry.
    if context.revoked_nonces.contains(&request.nonce) {
        return Decision::deny(ReasonCode::REPLAY_OR_REVOKED_NONCE, epoch);
    }
    if request.issued_at > context.now_monotonic || request.expires_at <= context.now_monotonic {
        return Decision::deny(ReasonCode::REQUEST_NOT_FRESH, epoch);
    }

    // Gate 6 — object resolution. The resource must resolve to a canonical
    // object; an identifier that resolves to nothing is denied rather than
    // dereferenced.
    if !context.resolved_resources.contains_key(&request.resource_id) {
        return Decision::deny(ReasonCode::RESOURCE_UNRESOLVABLE, epoch);
    }

    // Gate 7 — revocation. Principal revocation is expressed by removing the
    // principal from `known_principals`; capability revocation is listed
    // here.
    if let Some(capability) = capability {
        if context.revoked_capabilities.contains(&capability.capability_id) {
            return Decision::deny(ReasonCode::CAPABILITY_REVOKED, epoch);
        }
    }

    // Gate 8 — mode. The registry declares the modes each operation is
    // designed for, and the registry is the signed policy, so it is the only
    // authority consulted. An operation the registry does not know is left to
    // gate 11, which is where the unknown operation is reported.
    let entry = context.operations.effective(&request.operation);
    if let Some(entry) = &entry {
        if !entry.allowed_modes.contains(context.mode.as_str()) {
            return Decision::deny(ReasonCode::OPERATION_BLOCKED_BY_SYSTEM_MODE, epoch);
        }
    }

    // Gate 9 — policy epoch. The request's epoch, the node's epoch, and
    // the epoch of the registry that will decide it must be the same
    // generation. A decision made under a registry from another generation
    // is a decision made under policy that is not the policy in force, and
    // the request's own epoch check cannot see it: the request and the node
    // agree about the epoch while the registry disagrees with both.
    if request.policy_epoch != context.current_policy_epoch {
        return Decision::deny(ReasonCode::POLICY_EPOCH_MISMATCH, epoch);
    }
    if context.operations.epoch() != context.current_policy_epoch {
        return Decision::deny(ReasonCode::REGISTRY_EPOCH_MISMATCH, epoch);
    }

    // Gate 10 — capability.
    let Some(capability) = capability else {
        return Decision::deny(ReasonCode::MISSING_CAPABILITY, epoch);
    };
    match capability.provenance {
        Provenance::Invalid => return Decision::deny(ReasonCode::SIGNATURE_INVALID, epoch),
        Provenance::Unchecked => {
            return Decision::deny(ReasonCode::UNVERIFIED_CAPABILITY_ISSUER, epoch);
        }
        Provenance::Verified => {}
    }
    if capability.subject_id != request.principal_id {
        return Decision::deny(ReasonCode::CAPABILITY_SUBJECT_MISMATCH, epoch);
    }
    if capability.resource_id != request.resource_id
        || capability.resource_type != request.resource_type
    {
        return Decision::deny(ReasonCode::CAPABILITY_RESOURCE_MISMATCH, epoch);
    }
    if !capability.operations.contains(&request.operation) {
        return Decision::deny(ReasonCode::CAPABILITY_OPERATION_MISMATCH, epoch);
    }
    if !capability.purposes.contains(&request.purpose) {
        return Decision::deny(ReasonCode::CAPABILITY_PURPOSE_MISMATCH, epoch);
    }
    if context.now_monotonic < capability.not_before
        || context.now_monotonic >= capability.expires_at
    {
        return Decision::deny(ReasonCode::CAPABILITY_NOT_CURRENT, epoch);
    }
    if capability.policy_epoch != context.current_policy_epoch {
        return Decision::deny(ReasonCode::CAPABILITY_POLICY_EPOCH_MISMATCH, epoch);
    }

    // Gate 11 — risk and consent. The tier comes from the registry. The
    // caller's claim is never authoritative: a lower claim is a mismatch, and
    // a higher claim is ignored in favour of the registry's tier.
    let Some(entry) = entry else {
        return Decision::deny(ReasonCode::UNKNOWN_OPERATION_RISK, epoch);
    };
    if !entry.resource_types.contains(&request.resource_type) {
        return Decision::deny(ReasonCode::UNKNOWN_OPERATION_RISK, epoch);
    }
    if request.risk_claim < entry.risk_tier {
        return Decision::deny(ReasonCode::RISK_CLAIM_MISMATCH, epoch);
    }
    if requires_approval(&entry) {
        let bound = request
            .approval_digest
            .as_deref()
            .is_some_and(|digest| Some(digest) == context.action_digest.as_deref());
        if !bound {
            return Decision::challenge(ReasonCode::FRESH_ACTION_BOUND_APPROVAL_REQUIRED, epoch);
        }
        if entry.approval_requirements.contains("STRONG_AUTH") && !context.strong_approval_granted {
            return Decision::challenge(ReasonCode::STRONG_APPROVAL_REQUIRED, epoch);
        }
    }

    // Gate 12 — budget. The registry states the limits; where it states
    // none, there is no policy to check against, and the gate does not invent
    // one.
    if let Some(budget) = &request.budget {
        if let (Some(requested), Some(allowed)) = (budget.duration_ms, entry.max_duration_ms)
            && requested > allowed
        {
            return Decision::deny(ReasonCode::RESOURCE_BUDGET_EXCEEDED, epoch);
        }
        if let (Some(requested), Some(allowed)) =
            (budget.operation_count, entry.max_operation_count)
            && requested > allowed
        {
            return Decision::deny(ReasonCode::RESOURCE_BUDGET_EXCEEDED, epoch);
        }
    }

    // Gate 13 — precondition. The registry states the preconditions the
    // operation needs; the environment reports which currently hold.
    if entry
        .required_preconditions
        .iter()
        .any(|precondition| !context.satisfied_preconditions.contains(precondition))
    {
        return Decision::deny(ReasonCode::PRECONDITION_FAILED, epoch);
    }

    // Gate 14 — audit readiness. Absent audit storage is not absent risk.
    if entry.audit_class >= AuditClass::Sensitive && context.audit_state != AuditState::Ready {
        return Decision::deny(ReasonCode::AUDIT_UNAVAILABLE_FOR_SENSITIVE_ACTION, epoch);
    }

    // Gate 15 — decision construction. Gate 16 — effect separation: nothing
    // here performs an effect, and the decision has nowhere to record one.
    Decision::permit(ReasonCode::ALL_POLICY_GATES_PASSED, capability.capability_id.clone(), epoch)
}

/// Whether an operation needs an approval bound to the exact action.
fn requires_approval(entry: &OperationEntry) -> bool {
    !entry.approval_requirements.is_empty()
        && !entry.approval_requirements.iter().all(|requirement| requirement == "NONE")
}

/// The tier the registry assigns to an operation, if it assigns one.
pub fn registry_risk_tier(context: &PolicyContext, operation_id: &str) -> Option<RiskTier> {
    context.operations.effective(operation_id).map(|entry| entry.risk_tier)
}

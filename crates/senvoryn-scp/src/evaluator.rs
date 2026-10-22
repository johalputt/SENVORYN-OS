//! The decision oracle: the sixteen validation gates of specification §3.2.
//!
//! The evaluator is a pure function. It reads no clock, performs no I/O,
//! mutates nothing, and returns a decision that grants nothing by itself.
//! Time arrives as a value in the context, which is the only way a decision
//! about expiry can be tested at all.

use crate::capability::{Capability, Provenance};
use crate::context::{AuditState, ClockState, PolicyContext};
use crate::decision::{Decision, Deferral, Isolation};
use crate::reason::ReasonCode;
use crate::registry::{AuditClass, OperationEntry, Reversibility, RiskTier};
use crate::request::Request;
use crate::resolution::ResolvedResource;

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

    // The identity gate, continued — the quarantine of a principal.
    //
    // A quarantined principal is *known*: it is held apart, not let go, and
    // the node says so, so that the subject can ask for the isolation to be
    // lifted. The check is here, beside the known-principal check, because
    // both are facts about the subject — and here, before the resource is
    // resolved and before the capability is examined, so that a quarantined
    // principal learns that it is quarantined and nothing else.
    //
    // A quarantine is a state of the subject and not a penalty for an error:
    // no other failure falls back to it. ADR-023.
    if context.quarantined_principals.contains(&request.principal_id) {
        return Decision::quarantine(
            ReasonCode::PRINCIPAL_QUARANTINED,
            Isolation::Principal(request.principal_id.clone()),
            epoch,
        );
    }

    // Gate 5 — freshness: nonce, replay cache, expiry.
    if context.revoked_nonces.contains(&request.nonce) {
        return Decision::deny(ReasonCode::REPLAY_OR_REVOKED_NONCE, epoch);
    }
    if request.issued_at > context.now_monotonic || request.expires_at <= context.now_monotonic {
        return Decision::deny(ReasonCode::REQUEST_NOT_FRESH, epoch);
    }

    // Gate 6 — object resolution. The name the caller used is resolved to the
    // canonical identity of an object: the identifier the rest of the system
    // knows it by, its type, and its version. A name that resolves to nothing
    // is denied rather than dereferenced. A name that resolves to an object
    // whose type is not the type the request claimed is denied rather than
    // corrected: the type in the envelope is the caller's claim about its own
    // target, and a caller that misdescribes its target is either confused or
    // hostile, and the boundary does not decide which.
    let resolved = context.resolved_resources.get(&request.resource_id).cloned();
    let Some(resolved) = resolved else {
        return Decision::deny(ReasonCode::RESOURCE_UNRESOLVABLE, epoch);
    };
    if resolved.resource_type != request.resource_type {
        return Decision::deny(ReasonCode::RESOURCE_TYPE_MISMATCH, epoch).with_resolved(resolved);
    }

    // Everything from here on is judged against the canonical identity rather
    // than against what the caller wrote, and the decision carries that
    // identity so that the record and the audit name the object the effect
    // will be applied to.
    decide(request, capability, context, &resolved, epoch).with_resolved(resolved)
}

/// The gates that follow the resolution.
///
/// These are the gates that need the canonical identity, and they are a
/// separate function so that the identity reaches all of them from one place:
/// a gate that could read the caller's strings instead would be one line away
/// from doing so.
fn decide(
    request: &Request,
    capability: Option<&Capability>,
    context: &PolicyContext,
    resolved: &ResolvedResource,
    epoch: u64,
) -> Decision {
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
    //
    // The registry's own consistency is settled first. Specification §3.2
    // says no decision may be made under a registry whose window has not
    // opened or has closed, and a deferral is a decision — it produces a
    // record and an audit event — so a node whose policy in force is not in
    // force decides nothing at all, not even a postponement.
    if context.operations.epoch() != context.current_policy_epoch {
        return Decision::deny(ReasonCode::REGISTRY_EPOCH_MISMATCH, epoch);
    }
    // The registry's own validity window. A registry with no window is a
    // policy that never expires, so nothing would bound how old the policy
    // in force is; and a wall clock that has moved backwards would make an
    // expired registry look current, which is the one way a clock grants
    // authority. So the window is judged against the wall clock, and a clock
    // the environment will not vouch for is refused before it is used.
    if context.clock_state == ClockState::Untrusted {
        return Decision::deny(ReasonCode::CLOCK_UNTRUSTED, epoch);
    }
    if context.now_wall_ms < context.operations.valid_from_ms() {
        return Decision::deny(ReasonCode::REGISTRY_NOT_YET_VALID, epoch);
    }
    if context.now_wall_ms >= context.operations.valid_until_ms() {
        return Decision::deny(ReasonCode::REGISTRY_EXPIRED, epoch);
    }

    // The request's own epoch. A request under a **future** epoch names a
    // policy that does not exist yet, so the answer is neither yes nor no:
    // it is "not yet", and the node defers it. A request under a **stale**
    // epoch names a policy that is gone and will not come back, so the node
    // denies it. ADR-022.
    //
    // The deferral is bounded by the request itself. Gate 5 has already
    // refused a request that has expired, so a deferred request becomes a
    // denial once it expires: a deferral cannot outlive the request it
    // defers, and no new state is needed to say so.
    if request.policy_epoch != context.current_policy_epoch {
        if request.policy_epoch > context.current_policy_epoch && is_non_destructive(entry.as_ref())
        {
            return Decision::defer(
                ReasonCode::POLICY_EPOCH_NOT_YET_IN_FORCE,
                Deferral::PolicyEpoch(request.policy_epoch),
                epoch,
            );
        }
        return Decision::deny(ReasonCode::POLICY_EPOCH_MISMATCH, epoch);
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
    // The grant is over an object, so it is compared with the canonical
    // identity of the object the name resolved to and with nothing the
    // caller wrote. A grant for one object cannot be aimed at another by
    // addressing it differently, and an alias still matches the grant that
    // names the object the alias means.
    if capability.resource_id != resolved.canonical_id
        || capability.resource_type != resolved.resource_type
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
    // a higher claim is ignored in favour of the registry's tier. The
    // registry's rules are keyed by canonical resource type, so the type that
    // is looked up is the one the environment resolved.
    let Some(entry) = entry else {
        return Decision::deny(ReasonCode::UNKNOWN_OPERATION_RISK, epoch);
    };
    if !entry.resource_types.contains(&resolved.resource_type) {
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

/// Whether an operation may be deferred rather than denied.
///
/// Specification §3.3 qualifies `DEFER` with "operation is non-destructive",
/// and a deferral is a promise of a later answer — so it is a promise this
/// project may only make about an operation whose effect can be undone.
///
/// The registry in force is the only policy this node holds, so the registry
/// in force is what answers. The registry for the future epoch does not
/// exist — that is why the request is deferred at all — and the one it does
/// hold is the only one it may consult. An operation the registry does not
/// know, or one whose reversibility the registry does not state, is not
/// shown to be non-destructive, and `UNKNOWN` is the worst case by
/// construction.
///
/// An external effect settles it the other way: an operation that reaches
/// outside the node is not undone by restoring local state, whatever the
/// registry says about reversibility.
fn is_non_destructive(entry: Option<&OperationEntry>) -> bool {
    match entry {
        // An operation the registry does not know is not shown to be
        // non-destructive. Absence is never protection: the registry's
        // `UNKNOWN` reversibility is the worst case by construction, and an
        // operation it does not mention is worse than one it describes
        // badly.
        None => false,
        Some(entry) => {
            entry.reversibility == Reversibility::Reversible && entry.external_effects.is_empty()
        }
    }
}

/// The tier the registry assigns to an operation, if it assigns one.
pub fn registry_risk_tier(context: &PolicyContext, operation_id: &str) -> Option<RiskTier> {
    context.operations.effective(operation_id).map(|entry| entry.risk_tier)
}

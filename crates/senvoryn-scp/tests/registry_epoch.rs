//! The registry in force must be the registry for the epoch in force.
//!
//! A registry carries the policy generation it was published for. If the
//! node is running a different generation, then the risk tiers it consults
//! come from policy that is not the policy the request was bound to, and
//! the request's own epoch check cannot see it: the request and the node
//! agree about the epoch while the registry disagrees with both.

use std::collections::BTreeSet;

use senvoryn_scp::{
    AuditState, Capability, DecisionStatus, OperationRegistry, PolicyContext, Provenance, Request,
    RiskTier, SystemMode, evaluate,
};

/// The policy epoch the node is running.
const EPOCH: u64 = 7;

fn request() -> Request {
    Request {
        request_id: "req:epoch:0001".to_owned(),
        principal_id: "user:alice".to_owned(),
        recipient_id: "node:local".to_owned(),
        resource_id: "file:report".to_owned(),
        resource_type: "file_object".to_owned(),
        operation: "read".to_owned(),
        purpose: "user-request".to_owned(),
        policy_epoch: EPOCH,
        issued_at: 0,
        expires_at: 1000,
        nonce: "nonce:epoch:0001".to_owned(),
        risk_claim: RiskTier::R0,
        approval_digest: None,
        budget: None,
    }
}

fn capability() -> Capability {
    Capability {
        capability_id: "cap:epoch:0001".to_owned(),
        subject_id: "user:alice".to_owned(),
        resource_id: "file:report".to_owned(),
        resource_type: "file_object".to_owned(),
        operations: ["read".to_owned()].into_iter().collect(),
        purposes: ["user-request".to_owned()].into_iter().collect(),
        not_before: 0,
        expires_at: 1000,
        policy_epoch: EPOCH,
        provenance: Provenance::Verified,
        delegation_depth: 1,
        max_uses: None,
    }
}

fn context(operations: OperationRegistry) -> PolicyContext {
    PolicyContext {
        local_node_id: "node:local".to_owned(),
        current_policy_epoch: EPOCH,
        now_monotonic: 500,
        mode: SystemMode::Normal,
        known_principals: ["user:alice".to_owned()].into_iter().collect(),
        resolved_resources: [("file:report".to_owned(), "v1".to_owned())].into_iter().collect(),
        operations,
        audit_state: AuditState::Ready,
        action_digest: None,
        revoked_nonces: BTreeSet::new(),
        revoked_capabilities: BTreeSet::new(),
        strong_approval_granted: false,
        satisfied_preconditions: [
            "resource_exists".to_owned(),
            "capability_scope_matches".to_owned(),
        ]
        .into_iter()
        .collect(),
    }
}

/// The sample registry's rules, republished for a given epoch.
///
/// The rules are taken from the embedded sample and republished, so that the
/// only thing that differs from the sample is the epoch.
fn registry_at(epoch: u64) -> OperationRegistry {
    let sample = OperationRegistry::sample().expect("the embedded sample registry parses");
    OperationRegistry::from_entries(sample.entries().to_vec(), epoch)
}

#[test]
fn a_registry_from_another_epoch_is_not_the_policy_in_force() {
    let decision = evaluate(&request(), Some(&capability()), &context(registry_at(EPOCH - 1)));
    assert_eq!(
        decision.reason().as_str(),
        "REGISTRY_EPOCH_MISMATCH",
        "a registry from another epoch is not the policy in force, and the reason must say so"
    );
    assert_eq!(
        decision.status(),
        DecisionStatus::Deny,
        "a registry from another epoch must not permit anything"
    );
}

#[test]
fn the_registry_for_the_epoch_in_force_is_the_policy_in_force() {
    // The other half of the rule, and the one that keeps the check honest: a
    // registry that belongs to the epoch in force decides normally. The
    // sample is read as a document here, so that the epoch it declares is
    // the epoch it is held to.
    let sample = OperationRegistry::sample().expect("the embedded sample registry parses");
    let epoch = sample.epoch();
    let decision = evaluate(&request(), Some(&capability()), &context(sample));
    assert_eq!(
        decision.status(),
        DecisionStatus::Permit,
        "the registry for the epoch in force must decide the request: {}",
        decision.reason().as_str()
    );
    assert_eq!(epoch, EPOCH, "the sample registry must be the registry for the epoch in force");
}

//! The registry's merge rule.
//!
//! The registry is a list rather than a map because it may hold more than one
//! rule for the same operation, and the strictest rule is the one that
//! applies. A caller must not be able to escape a strict rule by also
//! matching a lax one — so the merge takes the most restrictive value on
//! every axis, and a test that does not check every axis is a test that
//! leaves one of them wrong.
//!
//! The Python reference model has the same merge and the same test. The
//! reversibility axis was once merged with the operator the wrong way round,
//! which kept the *more* reversible of two rules; neither implementation
//! noticed, because the sample registry holds one rule per operation and
//! the merge never ran.

use std::collections::BTreeSet;

use senvoryn_scp::{AuditClass, OperationEntry, OperationRegistry, Reversibility, RiskTier};

/// The lax rule: a low risk, easily undone, broadly allowed operation.
fn lax() -> OperationEntry {
    OperationEntry {
        operation_id: "write".to_owned(),
        risk_tier: RiskTier::R0,
        approval_requirements: BTreeSet::from(["NONE".to_owned()]),
        allowed_modes: BTreeSet::from(["NORMAL".to_owned(), "MAINTENANCE".to_owned()]),
        resource_types: BTreeSet::from(["file_object".to_owned()]),
        max_duration_ms: Some(60_000),
        max_operation_count: Some(5),
        required_preconditions: BTreeSet::new(),
        required_postconditions: BTreeSet::new(),
        reversibility: Reversibility::Reversible,
        external_effects: BTreeSet::new(),
        idempotent: true,
        audit_class: AuditClass::Standard,
    }
}

/// The strict rule for the same operation.
fn strict() -> OperationEntry {
    OperationEntry {
        operation_id: "write".to_owned(),
        risk_tier: RiskTier::R3,
        approval_requirements: BTreeSet::from(["STRONG_AUTH".to_owned()]),
        allowed_modes: BTreeSet::from(["NORMAL".to_owned()]),
        resource_types: BTreeSet::from(["file_object".to_owned(), "data_object".to_owned()]),
        max_duration_ms: Some(1_000),
        max_operation_count: Some(1),
        required_preconditions: BTreeSet::new(),
        required_postconditions: BTreeSet::new(),
        reversibility: Reversibility::NonReversible,
        external_effects: BTreeSet::new(),
        idempotent: false,
        audit_class: AuditClass::SecurityCritical,
    }
}

#[test]
fn the_merge_takes_the_most_restrictive_rule_on_every_axis() {
    let registry = OperationRegistry::from_entries(vec![lax(), strict()]);
    let merged = registry.effective("write").expect("the operation has a rule");

    assert_eq!(merged.risk_tier, RiskTier::R3, "the highest risk tier wins");
    assert_eq!(
        merged.reversibility,
        Reversibility::NonReversible,
        "the least reversible rule wins"
    );
    assert_eq!(merged.allowed_modes, BTreeSet::from(["NORMAL".to_owned()]));
    assert_eq!(
        merged.approval_requirements,
        BTreeSet::from(["NONE".to_owned(), "STRONG_AUTH".to_owned()])
    );
    assert_eq!(
        merged.resource_types,
        BTreeSet::from(["file_object".to_owned(), "data_object".to_owned()])
    );
    assert_eq!(merged.max_duration_ms, Some(1_000));
    assert_eq!(merged.max_operation_count, Some(1));
    assert_eq!(merged.audit_class, AuditClass::SecurityCritical);
    assert!(!merged.idempotent);
}

#[test]
fn an_operation_the_registry_does_not_contain_has_no_rule() {
    let registry = OperationRegistry::from_entries(vec![lax()]);
    assert!(registry.effective("teleport").is_none());
}

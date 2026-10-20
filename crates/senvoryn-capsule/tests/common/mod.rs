//! Shared fixtures for the Change Capsule tests.
//!
//! Every test that needs a capsule needs the same one: a well-formed capsule
//! for a write operation, against a simulated file target, with one step and
//! one mandatory postcondition. Building it in one place keeps a test from
//! passing because it accidentally changed the fixture rather than the rule
//! it was written to check.

// Each test binary compiles this module on its own, so a fixture used by only
// one of them reads as unused in the others. The fixtures are used; the
// warning is an artefact of how the test harness is built.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use senvoryn_canonical::parse;
use senvoryn_capsule::{Approval, Authorization, ChangeCapsule, Environment, Executor};
use senvoryn_scp::OperationRegistry;

const CAPSULE_JSON: &str = include_str!("../../../../examples/capsule.sample.json");

/// The sample capsule's JSON text with one field replaced.
///
/// The replacement must actually happen: a test that quietly changed nothing
/// would pass for the wrong reason, which is the failure mode the mutation
/// gate exists to catch.
pub fn capsule_text_with(anchor: &str, replacement: &str) -> String {
    assert!(CAPSULE_JSON.contains(anchor), "the sample capsule no longer contains {anchor:?}");
    CAPSULE_JSON.replacen(anchor, replacement, 1)
}

/// The idempotency key of the capsule's first step.
pub const WRITE_KEY: &str = "idem:capsule:example:0001:step.write";

/// The idempotency key of the capsule's second step.
pub const VERIFY_KEY: &str = "idem:capsule:example:0001:step.verify";

/// The digest the sample capsule carries.
pub const DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000002";

/// The sample capsule.
pub fn capsule() -> ChangeCapsule {
    ChangeCapsule::from_value(&parse(CAPSULE_JSON).expect("the sample capsule is canonical"))
        .expect("the sample capsule is well formed")
}

/// The sample registry.
pub fn registry() -> OperationRegistry {
    OperationRegistry::sample().expect("the sample registry is well formed")
}

/// An environment in which the sample capsule may run.
pub fn environment() -> Environment {
    let mut satisfied_preconditions = BTreeSet::new();
    satisfied_preconditions.insert("resource_version_matches".to_owned());
    let mut observed_versions = BTreeMap::new();
    observed_versions.insert("resource_version_matches".to_owned(), "7".to_owned());
    let mut satisfied_postconditions = BTreeSet::new();
    satisfied_postconditions.insert("write_accepted".to_owned());
    satisfied_postconditions.insert("written_version_verified".to_owned());

    Environment {
        current_version: "7".to_owned(),
        policy_epoch: 7,
        now_ms: 1_767_225_600_000,
        satisfied_preconditions,
        observed_versions,
        satisfied_postconditions,
        staged: false,
        applied_steps: Vec::new(),
        compensated_steps: Vec::new(),
        outstanding_effects: BTreeSet::new(),
        recovery_material_present: true,
        audit_sink_durable: true,
    }
}

/// An authorization that permits the sample capsule.
pub fn authorization() -> Authorization {
    Authorization {
        permitted_digest: DIGEST.to_owned(),
        approvals: vec![Approval {
            approval_type: "USER_CONFIRMATION".to_owned(),
            binds_digest: DIGEST.to_owned(),
        }],
    }
}

/// An executor for the sample capsule, in an environment where it may run.
pub fn executor() -> Executor {
    Executor::new(capsule(), registry(), environment()).expect("the executor builds")
}

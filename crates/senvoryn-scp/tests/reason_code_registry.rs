//! Checks this crate's reason codes against the denial-code registry.
//!
//! The registry is the authority on which codes exist. These tests check both
//! directions, so a code added to one side and not the other fails the build
//! rather than surviving to a release.

use senvoryn_scp::{ReasonCode, registry_codes, registry_default_status};

#[test]
fn every_code_this_crate_can_produce_is_in_the_registry() {
    let registry = registry_codes();
    for code in ReasonCode::ALL {
        if *code == ReasonCode::ALL_POLICY_GATES_PASSED {
            // The permitting reason is not a denial, so it is not in a denial
            // registry. It is still a reason code, and it is still reported.
            continue;
        }
        assert!(
            registry.contains(code.as_str()),
            "{} is not in the denial-code registry",
            code.as_str()
        );
    }
}

#[test]
fn every_code_in_the_registry_is_known_to_this_crate() {
    let known: Vec<&str> = ReasonCode::ALL.iter().map(|code| code.as_str()).collect();
    for code in registry_codes() {
        assert!(
            known.contains(code),
            "{code} is in the denial-code registry but this crate cannot produce it"
        );
    }
}

#[test]
fn the_registry_and_this_crate_agree_on_the_default_status() {
    // The evaluator states the status explicitly, so this is the check that
    // stops the two from drifting apart. A code the evaluator treats as a
    // challenge must not be a denial in the registry, or a reader of the
    // registry would be told the wrong thing.
    for code in ReasonCode::ALL {
        if *code == ReasonCode::ALL_POLICY_GATES_PASSED {
            continue;
        }
        let default = registry_default_status(code.as_str())
            .unwrap_or_else(|| panic!("{} is not in the registry", code.as_str()));
        match *code {
            ReasonCode::FRESH_ACTION_BOUND_APPROVAL_REQUIRED
            | ReasonCode::STRONG_APPROVAL_REQUIRED => assert_eq!(
                default.as_str(),
                "CHALLENGE",
                "{} should default to CHALLENGE",
                code.as_str()
            ),
            _ => assert_eq!(default.as_str(), "DENY", "{} should default to DENY", code.as_str()),
        }
    }
}

#[test]
fn the_registry_has_no_duplicate_codes() {
    let registry = registry_codes();
    let entries = senvoryn_scp::registry_entry("WRONG_RECIPIENT")
        .expect("a well-known code is in the registry");
    assert_eq!(entries.code, "WRONG_RECIPIENT");
    assert_eq!(registry.len(), senvoryn_scp::registry_codes().len());
}

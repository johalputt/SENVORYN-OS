//! The exit criterion for this gate: **an interrupted operation never falsely
//! reports `COMMITTED`**.
//!
//! The test walks the whole fault space — every injection point, every fault
//! — and asserts that no combination of them ends the transaction in
//! `COMMITTED`. It is the property the crate exists to establish, so it is
//! written as a property over the space rather than as a list of examples.

mod common;

use common::{DIGEST, authorization, capsule, environment, executor, registry};
use senvoryn_capsule::{Fault, FaultInjector, FaultPoint, State};

/// Every point at which a fault can be injected.
fn fault_points() -> Vec<FaultPoint> {
    let mut points = vec![
        FaultPoint::BeforeValidation,
        FaultPoint::BeforeAuthorization,
        FaultPoint::BeforeStaging,
        FaultPoint::AfterStaging,
        FaultPoint::BeforePrecheck,
        FaultPoint::BeforeVerification,
        FaultPoint::DuringVerification,
    ];
    for index in 0..2 {
        points.push(FaultPoint::BeforeStep { index });
        points.push(FaultPoint::DuringStep { index });
        points.push(FaultPoint::AfterStep { index });
    }
    points
}

/// Every fault that can be injected.
fn faults() -> Vec<Fault> {
    vec![
        Fault::Io,
        Fault::Crash,
        Fault::StalePrecondition,
        Fault::PolicyEpochChanged,
        Fault::AuditSinkLost,
        Fault::DiskFull,
        Fault::ClockRollback,
        Fault::CorruptedRecoveryArtifact,
    ]
}

#[test]
fn no_interrupted_operation_ever_reports_committed() {
    let mut checked = 0usize;
    for point in fault_points() {
        for fault in faults() {
            let mut executor = executor().with_faults(FaultInjector::at(point, fault));
            let outcome = executor.run(&authorization());
            assert_ne!(
                outcome.final_state,
                State::Committed,
                "a fault at {point:?} ({fault:?}) was reported as COMMITTED: {reason}",
                reason = outcome.reason
            );
            // A fault injected at a point the transaction never reaches is not
            // a fault the transaction survived. Without this check, a point
            // the capsule never visits would prove nothing and pass quietly.
            assert!(
                executor.faults_fired().contains(&point),
                "the fault at {point:?} never fired, so it proved nothing"
            );
            checked += 1;
        }
    }
    assert!(checked >= 100, "the fault space is smaller than expected: {checked}");
}

#[test]
fn a_run_with_no_fault_reports_committed() {
    // The property above is only meaningful if the un-faulted run commits.
    // Without this, a simulator that never commits anything would satisfy it.
    let mut executor = executor();
    let outcome = executor.run(&authorization());
    assert_eq!(outcome.final_state, State::Committed, "{}", outcome.reason);
    assert_eq!(outcome.steps_executed, vec!["step.write".to_owned(), "step.verify".to_owned()]);
    assert!(outcome.steps_reused.is_empty());
    assert!(executor.faults_fired().is_empty());
}

#[test]
fn a_crash_during_a_step_leaves_the_effect_unknown_and_escalates() {
    // A crash is the case where the executor cannot know whether the effect
    // landed. It must not repeat the step, and it must not report success.
    let mut executor = executor()
        .with_faults(FaultInjector::at(FaultPoint::DuringStep { index: 0 }, Fault::Crash));
    let outcome = executor.run(&authorization());
    assert_ne!(outcome.final_state, State::Committed);
    assert_eq!(outcome.final_state, State::NeedsHumanRecovery, "{}", outcome.reason);
}

#[test]
fn an_io_error_during_a_step_is_compensated() {
    // The effect is known to have landed, so the executor knows what to undo.
    let mut executor =
        executor().with_faults(FaultInjector::at(FaultPoint::DuringStep { index: 0 }, Fault::Io));
    let outcome = executor.run(&authorization());
    assert_ne!(outcome.final_state, State::Committed);
    assert_eq!(outcome.final_state, State::RolledBack, "{}", outcome.reason);
    assert!(
        executor.environment().outstanding_effects.is_empty(),
        "the compensation left an effect outstanding"
    );
}

#[test]
fn an_interrupted_run_records_what_it_knows_about_the_step() {
    let mut executor =
        executor().with_faults(FaultInjector::at(FaultPoint::BeforeStep { index: 0 }, Fault::Io));
    executor.run(&authorization());
    let recorded =
        executor.ledger().lookup("capsule:example:0001", "idem:capsule:example:0001:step.write");
    assert!(recorded.is_some(), "the interrupted step was not recorded");
    assert!(matches!(recorded, Some(senvoryn_capsule::StepOutcome::Interrupted { .. })));
}

#[test]
fn a_capsule_the_registry_does_not_know_is_rejected_before_anything_runs() {
    // An operation the registry does not contain has no risk tier, so it has
    // no place in a capsule. The registry is the authority on that.
    let text = common::capsule_text_with("\"operation\": \"write\"", "\"operation\": \"teleport\"");
    let document = senvoryn_canonical::parse(&text).expect("the capsule is canonical");
    let capsule =
        senvoryn_capsule::ChangeCapsule::from_value(&document).expect("the capsule parses");
    let mut executor = senvoryn_capsule::Executor::new(capsule, registry(), environment())
        .expect("the executor builds");
    let outcome = executor.run(&authorization());
    assert_eq!(outcome.final_state, State::Rejected, "{}", outcome.reason);
    assert!(executor.environment().applied_steps.is_empty());
}

#[test]
fn a_capsule_that_claims_less_risk_than_the_registry_is_rejected() {
    // The registry is the authority on risk, in the capsule executor exactly
    // as it is in the SCP.
    let text = common::capsule_text_with(
        "\"risk_tier\": \"R2\",\n  \"reversibility\"",
        "\"risk_tier\": \"R0\",\n  \"reversibility\"",
    );
    let document = senvoryn_canonical::parse(&text).expect("the capsule is canonical");
    let capsule =
        senvoryn_capsule::ChangeCapsule::from_value(&document).expect("the capsule parses");
    let mut executor = senvoryn_capsule::Executor::new(capsule, registry(), environment())
        .expect("the executor builds");
    let outcome = executor.run(&authorization());
    assert_eq!(outcome.final_state, State::Rejected, "{}", outcome.reason);
}

#[test]
fn a_capsule_for_another_plan_is_rejected() {
    // The decision permits a digest. If the digest is not this capsule's, the
    // decision is about a different plan and permits nothing here.
    let mut executor = executor();
    let other = senvoryn_capsule::Authorization {
        permitted_digest: DIGEST.replace('2', "3"),
        approvals: authorization().approvals,
    };
    let outcome = executor.run(&other);
    assert_eq!(outcome.final_state, State::Rejected, "{}", outcome.reason);
}

#[test]
fn a_missing_approval_is_rejected() {
    let mut executor = executor();
    let bare = senvoryn_capsule::Authorization {
        permitted_digest: DIGEST.to_owned(),
        approvals: Vec::new(),
    };
    let outcome = executor.run(&bare);
    assert_eq!(outcome.final_state, State::Rejected, "{}", outcome.reason);
}

#[test]
fn an_expired_capsule_expires() {
    let mut environment = environment();
    environment.now_ms = 1_767_225_600_000 + 3_600_000;
    let mut executor = senvoryn_capsule::Executor::new(capsule(), registry(), environment)
        .expect("the executor builds");
    let outcome = executor.run(&authorization());
    assert_eq!(outcome.final_state, State::Expired, "{}", outcome.reason);
}

#[test]
fn a_stale_target_version_is_rejected() {
    let mut environment = environment();
    environment.current_version = "8".to_owned();
    let mut executor = senvoryn_capsule::Executor::new(capsule(), registry(), environment)
        .expect("the executor builds");
    let outcome = executor.run(&authorization());
    assert_eq!(outcome.final_state, State::Rejected, "{}", outcome.reason);
}

#[test]
fn a_stale_policy_epoch_is_rejected() {
    let mut environment = environment();
    environment.policy_epoch = 8;
    let mut executor = senvoryn_capsule::Executor::new(capsule(), registry(), environment)
        .expect("the executor builds");
    let outcome = executor.run(&authorization());
    assert_eq!(outcome.final_state, State::Rejected, "{}", outcome.reason);
}

#[test]
fn a_full_disk_leaves_the_effect_unknown() {
    // A write that could not complete is not a write that did not happen: it
    // may have been partially applied. "Partially applied" is not a state the
    // ledger can describe, so the effect is unknown, and an effect that is
    // not known cannot be undone automatically.
    let mut executor = executor()
        .with_faults(FaultInjector::at(FaultPoint::DuringStep { index: 0 }, Fault::DiskFull));
    let outcome = executor.run(&authorization());
    assert!(
        outcome.reason.contains("effect is not known"),
        "a full disk must leave the effect unknown, and the reason must say so: {reason}",
        reason = outcome.reason
    );
    assert_eq!(outcome.final_state, State::NeedsHumanRecovery, "{}", outcome.reason);
    let recorded =
        executor.ledger().lookup("capsule:example:0001", "idem:capsule:example:0001:step.write");
    assert!(
        matches!(
            recorded,
            Some(senvoryn_capsule::StepOutcome::Interrupted {
                effect: senvoryn_capsule::EffectState::Unknown
            })
        ),
        "a full disk was recorded as something other than an unknown effect: {recorded:?}"
    );
}

#[test]
fn a_clock_that_moves_backwards_is_not_trusted() {
    let mut executor =
        executor().with_faults(FaultInjector::at(FaultPoint::BeforePrecheck, Fault::ClockRollback));
    let outcome = executor.run(&authorization());
    assert!(
        outcome.reason.contains("clock moved backwards"),
        "a clock that moves backwards must not be trusted, and the reason must say so: {reason}",
        reason = outcome.reason
    );
    assert_eq!(outcome.final_state, State::Failed, "{}", outcome.reason);
}

#[test]
fn a_capsule_cannot_be_executed_before_it_was_issued() {
    // The only check that catches a clock that moved backwards before the
    // transaction had observed it at all: a capsule did not exist before it
    // was issued.
    let mut environment = environment();
    environment.now_ms = 1_767_225_600_000 - 1_000;
    let mut executor = senvoryn_capsule::Executor::new(capsule(), registry(), environment)
        .expect("the executor builds");
    let outcome = executor.run(&authorization());
    assert!(
        outcome.reason.contains("not been issued yet"),
        "a capsule cannot be executed before it was issued, and the reason must say so: {reason}",
        reason = outcome.reason
    );
    assert_eq!(outcome.final_state, State::Rejected, "{}", outcome.reason);
}

#[test]
fn a_recovery_artifact_that_is_present_but_corrupt_is_not_recovery_material() {
    // Presence is not integrity. Material that is present but corrupt is
    // worse than material that is missing, because a missing artefact stops
    // the transaction and a corrupt one is used.
    let mut environment = environment();
    environment.recovery_material_intact = false;
    let mut executor = senvoryn_capsule::Executor::new(capsule(), registry(), environment)
        .expect("the executor builds");
    let outcome = executor.run(&authorization());
    assert!(
        outcome.reason.contains("recovery artifact is corrupt"),
        "a corrupt recovery artifact is not recovery material, and the reason must say so: {reason}",
        reason = outcome.reason
    );
    assert_eq!(outcome.final_state, State::Failed, "{}", outcome.reason);
    assert!(executor.environment().applied_steps.is_empty());
}

#[test]
fn recovery_material_that_becomes_corrupt_cannot_undo_the_effect() {
    // The material was intact at the precheck and is not now. A compensation
    // that claims to have undone an effect with material that cannot be
    // trusted is a false claim, so the transaction escalates instead.
    let mut executor = executor().with_faults(FaultInjector::at(
        FaultPoint::DuringStep { index: 0 },
        Fault::CorruptedRecoveryArtifact,
    ));
    let outcome = executor.run(&authorization());
    assert!(
        outcome.reason.contains("automatic recovery cannot safely complete"),
        "a compensation needs intact recovery material, and the reason must say so: {reason}",
        reason = outcome.reason
    );
    assert_eq!(outcome.final_state, State::NeedsHumanRecovery, "{}", outcome.reason);
}

#[test]
fn a_capsule_whose_window_closes_during_the_plan_does_not_land_the_next_effect() {
    // The window is checked before every step, not only at the boundaries. A
    // step that lands an effect after the window closed is the case the
    // window exists to prevent.
    let text = common::capsule_text_with(
        "\"expires_at\": \"2026-01-01T01:00:00Z\"",
        "\"expires_at\": \"2026-01-01T00:00:30Z\"",
    );
    let document = senvoryn_canonical::parse(&text).expect("the capsule is canonical");
    let capsule =
        senvoryn_capsule::ChangeCapsule::from_value(&document).expect("the capsule parses");
    let mut executor = senvoryn_capsule::Executor::new(capsule, registry(), environment())
        .expect("the executor builds");
    let outcome = executor.run(&authorization());
    assert!(
        outcome.reason.contains("validity window closed while the plan was running"),
        "a capsule whose window closes during the plan must not land the next effect, \
         and the reason must say so: {reason}",
        reason = outcome.reason
    );
    assert_eq!(outcome.final_state, State::RolledBack, "{}", outcome.reason);
    assert_eq!(
        outcome.steps_executed,
        vec!["step.write".to_owned()],
        "the second step ran after the window closed"
    );
}

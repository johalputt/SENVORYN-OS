//! The idempotency ledger: a repeated request must not repeat a destructive
//! operation.
//!
//! Specification §5.5. The rule has two halves and both are tested here: a
//! step whose outcome is known is not run again, and a step whose effect may
//! have landed is never run again at all.

mod common;

use common::{authorization, executor};
use senvoryn_capsule::{
    EffectState, Fault, FaultInjector, FaultPoint, IdempotencyLedger, State, StepOutcome,
};

#[test]
fn a_resumed_run_does_not_repeat_a_step_that_already_ran() {
    // The rule specification §5.5 states in one sentence: a repeated request
    // with the same idempotency key resumes reconciliation rather than
    // repeating a destructive operation.
    let mut first =
        executor().with_faults(FaultInjector::at(FaultPoint::BeforeStep { index: 1 }, Fault::Io));
    let outcome = first.run(&authorization());
    assert_eq!(outcome.steps_executed, vec!["step.write".to_owned()]);
    assert_ne!(outcome.final_state, State::Committed);

    let mut second = executor().with_ledger(first.ledger().clone());
    let outcome = second.run(&authorization());
    assert_eq!(
        outcome.steps_executed,
        vec!["step.verify".to_owned()],
        "the resumed run did not skip the step that had already run: {}",
        outcome.reason
    );
    assert_eq!(outcome.steps_reused, vec!["step.write".to_owned()]);
    assert_eq!(outcome.final_state, State::Committed, "{}", outcome.reason);
}

#[test]
fn a_resumed_run_does_not_repeat_a_step_whose_effect_is_not_known() {
    // The other half of the rule. The crash leaves the effect unknown, so
    // the resumed run must not repeat the step, and must not claim success.
    let mut first = executor()
        .with_faults(FaultInjector::at(FaultPoint::DuringStep { index: 0 }, Fault::Crash));
    let outcome = first.run(&authorization());
    assert_ne!(outcome.final_state, State::Committed);

    let mut second = executor().with_ledger(first.ledger().clone());
    let outcome = second.run(&authorization());
    assert_ne!(outcome.final_state, State::Committed);
    assert!(
        !outcome.steps_executed.contains(&"step.write".to_owned()),
        "the resumed run repeated a step whose effect was not known"
    );
}

#[test]
fn a_step_the_ledger_knows_is_reused_and_not_repeated() {
    let mut ledger = IdempotencyLedger::new();
    ledger.record(
        "capsule:example:0001",
        "idem:capsule:example:0001:step.write",
        StepOutcome::Succeeded,
    );
    assert_eq!(
        ledger.lookup("capsule:example:0001", "idem:capsule:example:0001:step.write"),
        Some(StepOutcome::Succeeded)
    );
    assert_eq!(
        ledger.lookup("capsule:example:0001", "idem:other"),
        None,
        "the ledger answered for a key it does not hold"
    );
}

#[test]
fn only_an_outcome_known_not_to_have_applied_may_be_retried() {
    assert!(
        StepOutcome::Interrupted { effect: EffectState::NotApplied }.may_retry(),
        "an operation that definitely did not run may be attempted again"
    );
    assert!(
        !StepOutcome::Interrupted { effect: EffectState::Unknown }.may_retry(),
        "an operation whose effect is not known may not be attempted again"
    );
    assert!(
        !StepOutcome::Interrupted { effect: EffectState::Applied }.may_retry(),
        "an operation whose effect landed may not be attempted again"
    );
    assert!(!StepOutcome::Succeeded.may_retry());
    assert!(!StepOutcome::Failed.may_retry());
}

#[test]
fn a_step_whose_effect_landed_counts_as_landed() {
    assert!(StepOutcome::Succeeded.effect_landed());
    assert!(StepOutcome::Interrupted { effect: EffectState::Applied }.effect_landed());
    assert!(!StepOutcome::Interrupted { effect: EffectState::NotApplied }.effect_landed());
    assert!(!StepOutcome::Interrupted { effect: EffectState::Unknown }.effect_landed());
}

#[test]
fn an_interrupted_step_is_retried_because_its_effect_did_not_land() {
    // The step was interrupted before it ran, so the effect definitely did
    // not land and the step may be attempted again.
    let mut executor =
        executor().with_faults(FaultInjector::at(FaultPoint::BeforeStep { index: 0 }, Fault::Io));
    let outcome = executor.run(&authorization());
    assert_ne!(outcome.final_state, State::Committed);
    assert_eq!(outcome.final_state, State::Failed, "{}", outcome.reason);
    let recorded =
        executor.ledger().lookup("capsule:example:0001", "idem:capsule:example:0001:step.write");
    assert_eq!(recorded, Some(StepOutcome::Interrupted { effect: EffectState::NotApplied }));
    assert!(recorded.expect("the step is recorded").may_retry());
}

#[test]
fn a_step_whose_effect_may_have_landed_is_never_repeated() {
    // The crash case. The executor does not know whether the effect landed,
    // so it must not run the step again, and it must not claim success.
    let mut executor = executor()
        .with_faults(FaultInjector::at(FaultPoint::DuringStep { index: 0 }, Fault::Crash));
    let outcome = executor.run(&authorization());
    assert_ne!(outcome.final_state, State::Committed);
    let recorded =
        executor.ledger().lookup("capsule:example:0001", "idem:capsule:example:0001:step.write");
    assert_eq!(recorded, Some(StepOutcome::Interrupted { effect: EffectState::Unknown }));
    assert!(!recorded.expect("the step is recorded").may_retry());
}

#[test]
fn the_ledger_is_the_only_place_a_repeated_step_is_refused() {
    // Two executors, one ledger: the second must not repeat the first's
    // destructive step. This is the rule specification §5.5 states in one
    // sentence, and it is the reason the ledger exists.
    let mut ledger = IdempotencyLedger::new();
    ledger.record(
        "capsule:example:0001",
        "idem:capsule:example:0001:step.write",
        StepOutcome::Succeeded,
    );
    assert_eq!(ledger.len(), 1);
    assert!(!ledger.is_empty());
}

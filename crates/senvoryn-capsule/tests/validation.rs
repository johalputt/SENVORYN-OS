//! The executor's validation controls: what a capsule must satisfy before
//! anything is staged.
//!
//! The shared vectors say what the outcome is. These say which rule produced
//! it, so a change to one rule cannot be absorbed by a different one — which
//! is the failure the mutation gate exists to catch.

mod common;

use common::{authorization, capsule_text_with, capsule_text_with_all, environment, registry};
use senvoryn_capsule::{ChangeCapsule, Executor, State};

/// Builds an executor for a capsule whose JSON text is given.
fn executor_for(text: &str) -> Executor {
    let document = senvoryn_canonical::parse(text).expect("the capsule is canonical");
    let capsule = ChangeCapsule::from_value(&document).expect("the capsule is well formed");
    Executor::new(capsule, registry(), environment()).expect("the executor builds")
}

#[test]
fn a_capsule_whose_budget_outlasts_its_window_is_refused() {
    // The capsule's own claims have to agree with one another. It says the
    // plan takes at most `budget.duration_ms`, and it says it is only valid
    // until `expires_at`. A plan whose own bound outlasts its own window
    // cannot finish inside the authority it was issued under, so the capsule
    // contradicts itself — and a capsule that contradicts itself is refused
    // before anything is staged.
    let text = capsule_text_with("\"duration_ms\": 60000", "\"duration_ms\": 7200000");
    let outcome = executor_for(&text).run(&authorization());
    assert!(
        outcome.reason.contains("does not fit inside its validity window"),
        "a capsule whose budget outlasts its window must be refused, and the reason must say so: {reason}",
        reason = outcome.reason
    );
    assert_eq!(outcome.final_state, State::Rejected, "{}", outcome.reason);
}

#[test]
fn a_capsule_whose_budget_exactly_fills_its_window_is_admitted() {
    // The boundary of the check above: a budget that exactly fills the
    // window is not a budget that outlasts it. A capsule that asks for every
    // millisecond it was granted has not asked for too much.
    //
    // The window is shortened to match the registry's combined allowance, so
    // that the budget is limited by the window rather than by the registry.
    let text = capsule_text_with_all(&[
        ("\"expires_at\": \"2026-01-01T01:00:00Z\"", "\"expires_at\": \"2026-01-01T00:01:30Z\""),
        ("\"duration_ms\": 60000", "\"duration_ms\": 90000"),
    ]);
    let outcome = executor_for(&text).run(&authorization());
    assert_eq!(
        outcome.final_state,
        State::Committed,
        "a capsule whose budget exactly fills its window is admitted, and the reason must not refuse it: {reason}",
        reason = outcome.reason
    );
}

#[test]
fn a_capsule_whose_budget_does_not_cover_its_steps_is_refused() {
    // The budget is the plan's own bound. A plan that asks for more time
    // than its budget allows is a plan that does not fit inside the
    // authority it asked for.
    let text = capsule_text_with("\"duration_ms\": 60000", "\"duration_ms\": 20000");
    let outcome = executor_for(&text).run(&authorization());
    assert!(
        outcome.reason.contains("does not cover its steps"),
        "a capsule whose budget does not cover its steps must be refused, and the reason must say so: {reason}",
        reason = outcome.reason
    );
    assert_eq!(outcome.final_state, State::Rejected, "{}", outcome.reason);
}

#[test]
fn a_capsule_whose_budget_exceeds_the_registry_allowance_is_refused() {
    // The registry bounds how long the operations a plan performs may run,
    // and the plan's budget cannot outlast that. The allowance is the sum of
    // the per-operation bounds the plan actually uses.
    let text = capsule_text_with("\"duration_ms\": 60000", "\"duration_ms\": 100000");
    let outcome = executor_for(&text).run(&authorization());
    assert!(
        outcome.reason.contains("exceeds the 90000ms its operations are allowed"),
        "a capsule whose budget exceeds the registry's allowance must be refused, and the reason must say so: {reason}",
        reason = outcome.reason
    );
    assert_eq!(outcome.final_state, State::Rejected, "{}", outcome.reason);
}

#[test]
fn a_step_may_not_run_longer_than_the_registry_allows() {
    // The registry's `max_duration_ms` bounds one invocation of one
    // operation. It is not the capsule's budget, and it is not the plan's
    // total.
    let text = capsule_text_with("\"timeout_ms\": 30000", "\"timeout_ms\": 90000");
    let outcome = executor_for(&text).run(&authorization());
    assert!(
        outcome.reason.contains("but the registry allows 60000ms"),
        "a step longer than the registry allows must be refused, and the reason must say so: {reason}",
        reason = outcome.reason
    );
    assert_eq!(outcome.final_state, State::Rejected, "{}", outcome.reason);
}

#[test]
fn the_registry_counts_each_operation_and_the_capsule_counts_the_plan() {
    // The registry's `max_operation_count` says how many times *one*
    // operation may be performed. The capsule's `budget.operation_count` says
    // how many operations the *plan* may perform. Neither number means the
    // other one's thing, so the two are not compared to each other.
    let text = capsule_text_with_all(&[
        ("\"operation\": \"read\"", "\"operation\": \"write\""),
        ("\"risk_tier\": \"R0\"", "\"risk_tier\": \"R2\""),
    ]);
    let outcome = executor_for(&text).run(&authorization());
    assert!(
        outcome.reason.contains("but the registry allows 1"),
        "an operation performed more often than the registry allows must be refused, and the reason must say so: {reason}",
        reason = outcome.reason
    );
    assert_eq!(outcome.final_state, State::Rejected, "{}", outcome.reason);

    let text = capsule_text_with("\"operation_count\": 2", "\"operation_count\": 1");
    let outcome = executor_for(&text).run(&authorization());
    assert!(
        outcome
            .reason
            .contains("the capsule's budget covers 1 operation(s) but the plan has 2 step(s)"),
        "a plan with more steps than the capsule's budget covers must be refused, and the reason must say so: {reason}",
        reason = outcome.reason
    );
    assert_eq!(outcome.final_state, State::Rejected, "{}", outcome.reason);
}

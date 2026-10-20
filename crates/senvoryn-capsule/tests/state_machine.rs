//! The state table is the authority on which transitions are legal, and the
//! executor may only take a transition the table allows.
//!
//! These tests check the table itself — that it is well formed, that it obeys
//! the invariants it states, and that it can express every outcome the
//! specification requires — and that the executor refuses to invent a
//! transition that is not in it.

mod common;

use common::{authorization, capsule, environment, executor, registry};
use senvoryn_capsule::{Executor, State, TransitionTable};

fn table() -> TransitionTable {
    Executor::new(capsule(), registry(), environment())
        .expect("the executor builds")
        .table()
        .clone()
}

#[test]
fn the_table_is_read_from_the_repositorys_own_state_machine() {
    let table = table();
    assert_eq!(table.initial, State::Draft);
    assert!(table.terminal.contains(&State::Committed));
    assert!(table.terminal.contains(&State::NeedsHumanRecovery));
}

#[test]
fn no_transition_skips_validation_or_authorization() {
    // The table's own invariant: "No transition skips VALIDATED and
    // AUTHORIZED." A table that cannot be checked against its own invariant
    // is a table that is not enforcing anything.
    let table = table();
    for transition in table.transitions() {
        let reaches_validated = matches!(
            transition.to,
            State::Validated | State::Rejected | State::Expired | State::Failed
        );
        assert!(
            reaches_validated || transition.from != State::Draft,
            "{} → {} leaves DRAFT without passing through VALIDATED",
            transition.from.as_str(),
            transition.to.as_str()
        );
        let bypasses_authorization = matches!(
            transition.to,
            State::Staged
                | State::Prechecked
                | State::Applying
                | State::Verifying
                | State::Committed
        );
        assert!(
            !bypasses_authorization
                || matches!(
                    transition.from,
                    State::Authorized
                        | State::Staged
                        | State::Prechecked
                        | State::Applying
                        | State::Verifying
                ),
            "{} → {} reaches a post-authorization state without passing through AUTHORIZED",
            transition.from.as_str(),
            transition.to.as_str()
        );
    }
}

#[test]
fn every_state_is_reachable() {
    // A state no transition enters is a state the specification defines but
    // the machine cannot produce. The table used to have one: FAILED was
    // listed as a state and no transition reached it, which is the same
    // defect as a decision status that no rule ever returns.
    let table = table();
    let mut reached = std::collections::BTreeSet::new();
    reached.insert(table.initial);
    for transition in table.transitions() {
        reached.insert(transition.to);
    }
    for state in [
        State::Draft,
        State::Validated,
        State::Authorized,
        State::Staged,
        State::Prechecked,
        State::Applying,
        State::Verifying,
        State::Committed,
        State::Rejected,
        State::Expired,
        State::Cancelled,
        State::Failed,
        State::Compensating,
        State::RolledBack,
        State::Partial,
        State::NeedsHumanRecovery,
    ] {
        assert!(
            reached.contains(&state),
            "{} is a state of the machine that no transition reaches",
            state.as_str()
        );
    }
}

#[test]
fn every_terminal_state_is_reachable() {
    let table = table();
    for state in &table.terminal {
        let reachable = table.transitions().any(|transition| transition.to == *state);
        assert!(reachable, "{} is terminal but no transition reaches it", state.as_str());
    }
}

#[test]
fn committed_can_only_be_reached_from_verifying() {
    let table = table();
    for transition in table.transitions() {
        if transition.to == State::Committed {
            assert_eq!(
                transition.from,
                State::Verifying,
                "COMMITTED must be reached from VERIFYING"
            );
        }
    }
}

#[test]
fn the_table_refuses_a_transition_it_does_not_contain() {
    // The table is the authority, so it has to be able to say no. A test that
    // only ever asks about transitions the table allows proves nothing about
    // the refusals.
    let table = table();
    assert!(table.permits(State::Draft, State::Validated));
    assert!(
        !table.permits(State::Draft, State::Committed),
        "the table allowed DRAFT → COMMITTED, which skips validation"
    );
    assert!(
        !table.permits(State::Committed, State::Draft),
        "the table allowed a terminal state to be left"
    );
    assert!(
        !table.permits(State::Applying, State::Committed),
        "the table allowed a commit without a verification"
    );
}

#[test]
fn the_executor_only_ever_enters_states_the_table_allows() {
    let mut executor = executor();
    let outcome = executor.run(&authorization());
    let table = executor.table();
    // The history starts with the state the transaction began in, so the
    // first entry is the starting point rather than a transition.
    let mut previous = table.initial;
    for state in outcome.history.iter().skip(1) {
        assert!(
            table.permits(previous, *state),
            "the executor entered {} from {}, which the table does not allow",
            state.as_str(),
            previous.as_str()
        );
        previous = *state;
    }
    assert_eq!(outcome.final_state, State::Committed);
}

#[test]
fn the_happy_path_walks_the_states_the_specification_names() {
    let mut executor = executor();
    let outcome = executor.run(&authorization());
    let walked: Vec<&str> = outcome.history.iter().map(|state| state.as_str()).collect();
    assert_eq!(
        walked,
        vec![
            "DRAFT",
            "VALIDATED",
            "AUTHORIZED",
            "STAGED",
            "PRECHECKED",
            "APPLYING",
            "VERIFYING",
            "COMMITTED",
        ]
    );
}

//! The idempotency ledger: what the executor already did, and what it still
//! does not know.
//!
//! Specification §5.5: a repeated request with the same idempotency key
//! returns the recorded result or resumes reconciliation; it must not blindly
//! repeat a destructive operation. The ledger is where that rule is kept, and
//! the distinction it has to preserve is between an operation that **did not
//! happen** and one whose outcome is **not known**. The first may be retried;
//! the second may not, because repeating it could apply a destructive effect
//! twice.

use std::collections::BTreeMap;

/// Whether an operation's effect landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectState {
    /// The operation did not run. It may be attempted again.
    NotApplied,
    /// The operation ran and its effect landed.
    Applied,
    /// The operation may or may not have run. Its effect is not known.
    Unknown,
}

/// What happened to a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    /// The step ran and its success predicates held.
    Succeeded,
    /// The step ran and failed. Its effect was undone, or it had none.
    Failed,
    /// The step was interrupted.
    Interrupted {
        /// What is known about its effect.
        effect: EffectState,
    },
}

impl StepOutcome {
    /// Whether the step may be attempted again.
    ///
    /// Only an outcome that is known not to have applied the effect may be
    /// retried. An unknown effect may not: the effect might have landed, and
    /// repeating a non-idempotent operation would apply it twice.
    pub fn may_retry(&self) -> bool {
        matches!(self, Self::Interrupted { effect: EffectState::NotApplied })
    }

    /// Whether the step's effect is known to have landed.
    pub fn effect_landed(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Interrupted { effect: EffectState::Applied })
    }
}

/// The record of what has already been attempted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IdempotencyLedger {
    entries: BTreeMap<(String, String), StepOutcome>,
}

impl IdempotencyLedger {
    /// A ledger with nothing in it.
    pub fn new() -> Self {
        Self::default()
    }

    /// What is known about a step, if anything.
    pub fn lookup(&self, capsule_id: &str, idempotency_key: &str) -> Option<StepOutcome> {
        self.entries.get(&(capsule_id.to_owned(), idempotency_key.to_owned())).copied()
    }

    /// Records what happened to a step.
    pub fn record(&mut self, capsule_id: &str, idempotency_key: &str, outcome: StepOutcome) {
        self.entries.insert((capsule_id.to_owned(), idempotency_key.to_owned()), outcome);
    }

    /// The number of recorded steps.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing has been recorded.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

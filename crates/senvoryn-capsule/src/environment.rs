//! The simulated system a capsule acts on, and the faults that can interrupt it.
//!
//! The simulator exists to answer one question honestly: **when an operation
//! is interrupted, does the transaction still report success?** To answer it,
//! the world the capsule acts on has to be able to change underneath the
//! transaction, and the transaction has to be able to find out.

use std::collections::{BTreeMap, BTreeSet};

/// The simulated system.
///
/// Every field is public because a test has to be able to move the world: a
/// precondition to stop holding, a version to change, an effect to appear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    /// The version the target is currently at.
    pub current_version: String,
    /// The policy epoch in force right now.
    pub policy_epoch: u64,
    /// The current time, in milliseconds since the Unix epoch.
    pub now_ms: i64,
    /// The preconditions that currently hold.
    pub satisfied_preconditions: BTreeSet<String>,
    /// The version each precondition observed, where it observed one.
    pub observed_versions: BTreeMap<String, String>,
    /// The postconditions that currently hold.
    pub satisfied_postconditions: BTreeSet<String>,
    /// Whether the artifacts are staged.
    pub staged: bool,
    /// The steps whose effect has landed.
    pub applied_steps: Vec<String>,
    /// The steps whose effect has been undone.
    pub compensated_steps: Vec<String>,
    /// The effects that are still outstanding.
    pub outstanding_effects: BTreeSet<String>,
    /// Whether the recovery material is present.
    pub recovery_material_present: bool,
    /// Whether the recovery material is intact.
    ///
    /// Presence is not integrity. Material that is present but corrupt is
    /// worse than material that is missing, because a missing artefact stops
    /// the transaction and a corrupt one is used.
    pub recovery_material_intact: bool,
    /// Whether the audit sink can record durably.
    pub audit_sink_durable: bool,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            current_version: String::new(),
            policy_epoch: 0,
            now_ms: 0,
            satisfied_preconditions: BTreeSet::new(),
            observed_versions: BTreeMap::new(),
            satisfied_postconditions: BTreeSet::new(),
            staged: false,
            applied_steps: Vec::new(),
            compensated_steps: Vec::new(),
            outstanding_effects: BTreeSet::new(),
            recovery_material_present: true,
            recovery_material_intact: true,
            audit_sink_durable: true,
        }
    }
}

impl Environment {
    /// Whether a precondition currently holds.
    pub fn precondition_holds(&self, predicate_id: &str) -> bool {
        self.satisfied_preconditions.contains(predicate_id)
    }

    /// The version a precondition observed, if it observed one.
    pub fn observed_version(&self, predicate_id: &str) -> Option<&str> {
        self.observed_versions.get(predicate_id).map(String::as_str)
    }

    /// Whether a postcondition currently holds.
    pub fn postcondition_holds(&self, predicate_id: &str) -> bool {
        self.satisfied_postconditions.contains(predicate_id)
    }

    /// The effect of a step, as the simulated world records it.
    pub fn apply_step(&mut self, step_id: &str) {
        self.applied_steps.push(step_id.to_owned());
        self.outstanding_effects.insert(step_id.to_owned());
    }

    /// The undo of a step's effect.
    pub fn compensate_step(&mut self, step_id: &str) {
        self.compensated_steps.push(step_id.to_owned());
        self.outstanding_effects.remove(step_id);
    }
}

/// Where in the transaction a fault can be injected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaultPoint {
    /// Before the capsule is validated.
    BeforeValidation,
    /// Before the capsule is authorized.
    BeforeAuthorization,
    /// Before the artifacts are staged.
    BeforeStaging,
    /// After the artifacts are staged, before the precheck.
    AfterStaging,
    /// Before the target state is rechecked.
    BeforePrecheck,
    /// Before a step runs.
    BeforeStep {
        /// The step's index.
        index: usize,
    },
    /// While a step runs, after its effect has landed.
    DuringStep {
        /// The step's index.
        index: usize,
    },
    /// After a step has run.
    AfterStep {
        /// The step's index.
        index: usize,
    },
    /// Before the outcomes are verified.
    BeforeVerification,
    /// While the outcomes are verified, before the verdict is recorded.
    DuringVerification,
}

/// What the fault does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// A read or write failed. The world is unchanged.
    Io,
    /// The machine stopped. The world is in whatever state it had reached.
    Crash,
    /// The target changed underneath the transaction.
    StalePrecondition,
    /// The policy epoch moved on underneath the transaction.
    PolicyEpochChanged,
    /// The audit sink stopped accepting durable records.
    AuditSinkLost,
    /// The storage could not accept a write.
    ///
    /// A write that fails is not a write that did not happen: it may have
    /// been partially applied. "Partially applied" is not a state the
    /// idempotency ledger can describe, so the effect of a step interrupted
    /// this way is **not known**, exactly as it is after a crash.
    DiskFull,
    /// The clock moved backwards.
    ///
    /// A clock that goes backwards makes a validity window meaningless: a
    /// capsule that looked expired looks unexpired again. The transaction
    /// records the highest time it has seen and refuses to act on a clock
    /// that has gone behind it.
    ClockRollback,
    /// The recovery material turned out to be corrupt.
    ///
    /// The material is present, and using it would undo the effect with
    /// material that cannot be trusted. Presence is not integrity.
    CorruptedRecoveryArtifact,
}

impl Fault {
    /// Whether the fault stops the transaction where it happened.
    ///
    /// An interruption is a moment the transaction does not continue past. A
    /// change to the world is not: the clock moved backwards, or the recovery
    /// material turned out to be corrupt, and the transaction carries on and
    /// finds the change for itself.
    pub fn interrupts(self) -> bool {
        !matches!(self, Fault::ClockRollback | Fault::CorruptedRecoveryArtifact)
    }
}

/// The faults to inject, by point.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FaultInjector {
    faults: BTreeMap<FaultPoint, Fault>,
}

impl FaultInjector {
    /// A injector that injects nothing.
    pub fn none() -> Self {
        Self::default()
    }

    /// An injector that fails at one point.
    pub fn at(point: FaultPoint, fault: Fault) -> Self {
        let mut faults = BTreeMap::new();
        faults.insert(point, fault);
        Self { faults }
    }

    /// The fault to inject at a point, if there is one.
    pub fn fault_at(&self, point: FaultPoint) -> Option<Fault> {
        self.faults.get(&point).copied()
    }
}

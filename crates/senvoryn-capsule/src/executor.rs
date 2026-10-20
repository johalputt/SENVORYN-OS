//! The executor: the Change Capsule state machine, driven.
//!
//! The executor's job is narrow and it is the whole of it: **move the
//! transaction from one state to the next only when the table allows it and
//! the guard holds.** It does not decide policy — the System Control Plane
//! does that, and this crate takes the decision as an input. It does not
//! decide whether a capsule is acceptable — the registry does that. What it
//! does is refuse to report a result it cannot prove.
//!
//! The exit criterion for this gate is the property the whole crate exists
//! for: **an interrupted operation never falsely reports `COMMITTED`.** Every
//! path to `COMMITTED` requires that every step has an outcome, that every
//! mandatory postcondition holds, and that the evidence is durable. A fault
//! at any point breaks one of those, so a fault at any point ends the
//! transaction somewhere other than `COMMITTED`.

use senvoryn_scp::OperationRegistry;

use crate::authorization::{self, Authorization};
use crate::capsule::ChangeCapsule;
use crate::environment::{Environment, Fault, FaultInjector, FaultPoint};
use crate::ledger::{EffectState, IdempotencyLedger, StepOutcome};
use crate::machine::{State, TransitionTable};

/// The state table, embedded at compile time.
///
/// The table in `state-machines/` is the authority on which transitions are
/// legal. Embedding it means the executor and the repository's contract pack
/// cannot disagree without the checksum manifest noticing.
const STATE_TABLE_JSON: &str =
    include_str!("../../../state-machines/change-capsule-transitions.json");

/// How far the clock jumps back when it is rolled back.
///
/// An hour is enough to reopen a validity window that had just closed, and
/// small enough that the transaction's own budget does not notice.
const CLOCK_ROLLBACK_MS: i64 = 3_600_000;

/// What the executor did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    /// The state the transaction ended in.
    pub final_state: State,
    /// Why it ended there.
    pub reason: String,
    /// The steps that ran.
    pub steps_executed: Vec<String>,
    /// The steps that were not run because the ledger already knew their outcome.
    pub steps_reused: Vec<String>,
    /// Every state the transaction entered, in order.
    pub history: Vec<State>,
}

/// Why the executor could not start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutorError {
    /// The state table could not be read.
    Machine(crate::machine::MachineError),
    /// The capsule could not be read.
    Capsule(crate::capsule::CapsuleError),
}

impl core::fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Machine(error) => write!(f, "{error}"),
            Self::Capsule(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ExecutorError {}

/// The Change Capsule executor.
#[derive(Debug, Clone)]
pub struct Executor {
    capsule: ChangeCapsule,
    table: TransitionTable,
    registry: OperationRegistry,
    environment: Environment,
    ledger: IdempotencyLedger,
    faults: FaultInjector,
    state: State,
    history: Vec<State>,
    /// The fault points that were actually reached.
    ///
    /// A fault injected at a point the transaction never reaches is not a
    /// fault the transaction survived. Recording what fired lets a test say
    /// so, instead of quietly proving nothing.
    fired: Vec<FaultPoint>,
    /// The highest time the transaction has observed.
    ///
    /// A clock that moves backwards makes the capsule's validity window
    /// meaningless: a capsule that looked expired looks unexpired again, and
    /// a budget that looked spent looks unspent. The transaction records the
    /// highest time it has seen and refuses to act on a clock that has gone
    /// behind it.
    highest_now_ms: i64,
}

impl Executor {
    /// Builds an executor for a capsule.
    pub fn new(
        capsule: ChangeCapsule,
        registry: OperationRegistry,
        environment: Environment,
    ) -> Result<Self, ExecutorError> {
        let table = TransitionTable::from_value(
            &senvoryn_canonical::parse(STATE_TABLE_JSON).map_err(|error| {
                ExecutorError::Machine(crate::machine::MachineError::Malformed {
                    field: "state table".to_owned(),
                    detail: error.to_string(),
                })
            })?,
        )
        .map_err(ExecutorError::Machine)?;
        let state = table.initial;
        Ok(Self {
            capsule,
            table,
            registry,
            environment,
            ledger: IdempotencyLedger::new(),
            faults: FaultInjector::none(),
            state,
            history: vec![state],
            fired: Vec::new(),
            highest_now_ms: i64::MIN,
        })
    }

    /// The state table the executor follows.
    pub fn table(&self) -> &TransitionTable {
        &self.table
    }

    /// The state the transaction is in.
    pub fn state(&self) -> State {
        self.state
    }

    /// The simulated system.
    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    /// What the executor knows about the steps it has attempted.
    pub fn ledger(&self) -> &IdempotencyLedger {
        &self.ledger
    }

    /// The fault points the transaction actually reached.
    ///
    /// A fault injected at a point the transaction never reaches is not a
    /// fault the transaction survived, and a test that counted it as one would
    /// be proving nothing.
    pub fn faults_fired(&self) -> &[FaultPoint] {
        &self.fired
    }

    /// The fault at a point, recording that the transaction reached it, and
    /// letting the fault change the world.
    fn fault_at(&mut self, point: FaultPoint) -> Option<Fault> {
        let fault = self.faults.fault_at(point);
        if let Some(fault) = fault {
            self.fired.push(point);
            self.apply_fault(fault);
        }
        fault
    }

    /// Lets a fault change the simulated world.
    ///
    /// A fault is not only an event the transaction observes. Some of them are
    /// changes to the world the transaction is acting on, and a change has to
    /// be found by the transaction's own checks rather than announced to it:
    /// a clock that moves backwards does not announce itself, and neither
    /// does a recovery artefact that turns out to be corrupt.
    fn apply_fault(&mut self, fault: Fault) {
        match fault {
            Fault::ClockRollback => self.environment.now_ms -= CLOCK_ROLLBACK_MS,
            Fault::CorruptedRecoveryArtifact => self.environment.recovery_material_intact = false,
            _ => {}
        }
    }

    /// The current time, if the clock can be trusted.
    ///
    /// Every read of the clock goes through here. A transaction that read the
    /// clock once and trusted it again later is a transaction whose expiry
    /// check can be defeated by moving the clock back.
    fn now(&mut self) -> Result<i64, Stop> {
        let now = self.environment.now_ms;
        if now < self.highest_now_ms {
            return Err(self.untrusted_clock("the clock moved backwards under the capsule"));
        }
        self.highest_now_ms = now;
        Ok(now)
    }

    /// The transaction cannot trust the clock, so it stops.
    ///
    /// Nothing has landed, so the transaction failed. Something has landed,
    /// so the transaction has to undo it before it can report anything.
    fn untrusted_clock(&mut self, reason: &str) -> Stop {
        if self.environment.applied_steps.is_empty() {
            self.fail(reason)
        } else {
            self.compensate(reason)
        }
    }

    /// Injects a fault.
    pub fn with_faults(mut self, faults: FaultInjector) -> Self {
        self.faults = faults;
        self
    }

    /// Resumes a transaction with the ledger of an earlier attempt.
    ///
    /// A capsule that was interrupted may be run again. The ledger is what
    /// stops the second attempt from repeating a step that already ran, and
    /// what stops it from repeating one whose effect is not known.
    pub fn with_ledger(mut self, ledger: IdempotencyLedger) -> Self {
        self.ledger = ledger;
        self
    }

    /// Runs the transaction.
    ///
    /// The authorization is the System Control Plane's decision. It is checked
    /// against the capsule before the capsule is allowed to act, and the check
    /// is that the decision and every approval bind the capsule's own digest.
    pub fn run(&mut self, authorization: &Authorization) -> ExecutionOutcome {
        let mut executed: Vec<String> = Vec::new();
        let mut reused: Vec<String> = Vec::new();

        if let Some(stop) = self.validate() {
            return self.outcome(stop, executed, reused);
        }
        if let Some(stop) = self.authorize(authorization) {
            return self.outcome(stop, executed, reused);
        }
        if let Some(stop) = self.stage() {
            return self.outcome(stop, executed, reused);
        }
        if let Some(stop) = self.precheck() {
            return self.outcome(stop, executed, reused);
        }
        if let Some(stop) = self.apply(&mut executed, &mut reused) {
            return self.outcome(stop, executed, reused);
        }
        if let Some(stop) = self.verify() {
            return self.outcome(stop, executed, reused);
        }
        self.outcome(
            Stop {
                state: self.state,
                reason: "every mandatory postcondition held and the evidence is durable".to_owned(),
            },
            executed,
            reused,
        )
    }

    /// `DRAFT → VALIDATED`: the capsule is a plan the registry accepts.
    fn validate(&mut self) -> Option<Stop> {
        if let Some(fault) = self.fault_at(FaultPoint::BeforeValidation) {
            if fault.interrupts() {
                return Some(self.fail(&format!(
                    "the capsule was interrupted before it was validated ({fault:?})"
                )));
            }
        }
        let now = match self.now() {
            Ok(now) => now,
            Err(stop) => return Some(stop),
        };
        // A capsule cannot be executed before it was issued, so a clock that
        // reads earlier than the issue time is a clock that cannot be
        // trusted. This is the only check that catches a clock that moved
        // backwards before the transaction had observed it at all.
        if now < self.capsule.issued_at_ms {
            return Some(self.reject("the capsule has not been issued yet"));
        }
        if self.capsule.expires_at_ms <= now {
            return Some(self.expire("the capsule's validity window has closed"));
        }
        // The capsule's own claims have to agree with one another. It says
        // the plan takes at most `budget.duration_ms`, and it says it is only
        // valid until `expires_at`. A plan whose own bound outlasts its own
        // window cannot finish inside the authority it was issued under, so
        // the capsule contradicts itself. This is a consistency check, not a
        // policy one: it needs no clock, and it is checked at validation so
        // the operator learns before anything is staged.
        if self.capsule.issued_at_ms + self.capsule.budget.duration_ms as i64
            > self.capsule.expires_at_ms
        {
            return Some(self.reject(&format!(
                "the capsule's budget of {}ms does not fit inside its validity window of {}ms",
                self.capsule.budget.duration_ms,
                self.capsule.expires_at_ms - self.capsule.issued_at_ms
            )));
        }
        for step in &self.capsule.steps {
            let Some(entry) = self.registry.effective(&step.operation) else {
                return Some(self.reject(&format!(
                    "the operation {:?} is not in the operation registry",
                    step.operation
                )));
            };
            // The registry is the authority on risk. A capsule that claims
            // less risk than the registry assigns is refused; a capsule that
            // claims more is not punished for it, exactly as in the SCP.
            if step.risk_tier < entry.risk_tier {
                return Some(self.reject(&format!(
                    "the step {:?} claims {:?} but the registry assigns {:?}",
                    step.step_id, step.risk_tier, entry.risk_tier
                )));
            }
            // The capsule may not ask for less authority than the registry
            // demands of the operation it performs. The System Control Plane
            // is supposed to have caught this already, but the executor is
            // the last gate before the effect, and a plan that understates
            // the authority it needs must not run.
            for required in &entry.approval_requirements {
                if required == "NONE" {
                    continue;
                }
                if !self
                    .capsule
                    .approval_requirements
                    .iter()
                    .any(|requirement| &requirement.approval_type == required)
                {
                    return Some(self.reject(&format!(
                        "the capsule does not require the {required} approval the operation {:?} demands",
                        step.operation
                    )));
                }
            }
            if let Some(maximum) = entry.max_duration_ms {
                if step.timeout_ms > maximum {
                    return Some(self.reject(&format!(
                        "the step {:?} may run for {}ms but the registry allows {maximum}ms",
                        step.step_id, step.timeout_ms
                    )));
                }
            }
            if let Some(maximum) = entry.max_operation_count {
                // The registry's count is per operation: how many times that
                // operation may be performed. The capsule's own count is a
                // total across the plan, so the two are not compared to each
                // other.
                let uses = self
                    .capsule
                    .steps
                    .iter()
                    .filter(|other| other.operation == step.operation)
                    .count() as u64;
                if uses > maximum {
                    return Some(self.reject(&format!(
                        "the capsule performs the operation {:?} {uses} time(s) but the registry allows {maximum}",
                        step.operation
                    )));
                }
            }
        }
        if self.capsule.risk_tier < self.capsule.claimed_risk_tier() {
            return Some(self.reject("the capsule's own risk tier is below the risk of its steps"));
        }
        if self.capsule.budget.operation_count < self.capsule.steps.len() as u64 {
            return Some(self.reject(&format!(
                "the capsule's budget covers {} operation(s) but the plan has {} step(s)",
                self.capsule.budget.operation_count,
                self.capsule.steps.len()
            )));
        }
        let planned: u64 = self.capsule.steps.iter().map(|step| step.timeout_ms).sum();
        if self.capsule.budget.duration_ms < planned {
            return Some(self.reject(&format!(
                "the capsule's budget of {}ms does not cover its steps, which ask for {planned}ms",
                self.capsule.budget.duration_ms
            )));
        }
        // The registry bounds how long each operation may run. A transaction
        // cannot outlast the operations it performs, so its budget is bounded
        // by their combined allowance.
        let allowance: u64 = self
            .capsule
            .steps
            .iter()
            .filter_map(|step| self.registry.effective(&step.operation))
            .filter_map(|entry| entry.max_duration_ms)
            .sum();
        if self.capsule.budget.duration_ms > allowance {
            return Some(self.reject(&format!(
                "the capsule's budget of {}ms exceeds the {allowance}ms its operations are allowed",
                self.capsule.budget.duration_ms
            )));
        }
        if self.capsule.policy_epoch != self.environment.policy_epoch {
            return Some(self.reject("the capsule was made under a stale policy epoch"));
        }
        if self.capsule.target.expected_version != self.environment.current_version {
            return Some(self.reject("the target is not at the version the capsule expects"));
        }
        for precondition in &self.capsule.preconditions {
            if !self.environment.precondition_holds(&precondition.predicate_id) {
                return Some(self.reject(&format!(
                    "the precondition {:?} does not hold",
                    precondition.predicate_id
                )));
            }
        }
        self.transition(State::Validated)
    }

    /// `VALIDATED → AUTHORIZED`: the decision permits this exact plan.
    fn authorize(&mut self, authorization: &Authorization) -> Option<Stop> {
        if let Some(fault) = self.fault_at(FaultPoint::BeforeAuthorization) {
            if fault.interrupts() {
                return Some(self.fail(&format!(
                    "the capsule was interrupted before it was authorized ({fault:?})"
                )));
            }
        }
        match authorization::check(&self.capsule, authorization) {
            Ok(()) => self.transition(State::Authorized),
            Err(error) => {
                Some(self.reject(&format!("the decision does not permit the capsule: {error}")))
            }
        }
    }

    /// `AUTHORIZED → STAGED`: the artifacts are staged outside the active storage.
    fn stage(&mut self) -> Option<Stop> {
        if let Some(fault) = self.fault_at(FaultPoint::BeforeStaging) {
            if fault.interrupts() {
                return Some(
                    self.fail(&format!("the capsule was interrupted before staging ({fault:?})")),
                );
            }
        }
        self.environment.staged = true;
        if let Some(fault) = self.fault_at(FaultPoint::AfterStaging) {
            if fault.interrupts() {
                return Some(
                    self.fail(&format!("the capsule was interrupted after staging ({fault:?})")),
                );
            }
        }
        self.transition(State::Staged)
    }

    /// `STAGED → PRECHECKED`: the world is rechecked, because it moved.
    fn precheck(&mut self) -> Option<Stop> {
        if let Some(fault) = self.fault_at(FaultPoint::BeforePrecheck) {
            if fault.interrupts() {
                return Some(self.fail(&format!(
                    "the capsule was interrupted before the precheck ({fault:?})"
                )));
            }
        }
        // The clock is read again, and a clock that has gone backwards is
        // not a clock the transaction can act on. The validity window is
        // *not* rechecked here: the simulator cannot move the clock forward
        // between the validation and the precheck, so a check here could
        // never fire, and a control that has never been seen to fail has
        // proved nothing. The window is checked before every step, which is
        // where the clock does move.
        match self.now() {
            Ok(_) => {}
            Err(stop) => return Some(stop),
        }
        if self.capsule.target.expected_version != self.environment.current_version {
            return Some(self.fail("the target version changed under the capsule"));
        }
        if self.capsule.policy_epoch != self.environment.policy_epoch {
            return Some(self.fail("the policy epoch moved on under the capsule"));
        }
        for precondition in &self.capsule.preconditions {
            if !self.environment.precondition_holds(&precondition.predicate_id) {
                return Some(self.fail(&format!(
                    "the precondition {:?} no longer holds",
                    precondition.predicate_id
                )));
            }
        }
        if self.capsule.recovery_plan.recovery_artifact_ref.is_some() {
            if !self.environment.recovery_material_present {
                return Some(self.fail("the recovery artifact is missing"));
            }
            // Presence is not integrity. Material that is present but corrupt
            // would be used to undo the effect, and using it would undo the
            // effect with material that cannot be trusted.
            if !self.environment.recovery_material_intact {
                return Some(self.fail("the recovery artifact is corrupt"));
            }
        }
        self.transition(State::Prechecked)
    }

    /// `PRECHECKED → APPLYING → VERIFYING`: the steps run.
    ///
    /// The ledger is consulted before every step. A step whose outcome is
    /// already known is not run again, and a step whose effect may have
    /// landed is never run again at all.
    fn apply(&mut self, executed: &mut Vec<String>, reused: &mut Vec<String>) -> Option<Stop> {
        // The `?` operator cannot be used here: `transition` returns `None` to
        // mean "the transition happened", and `?` would read that as an early
        // return. The steps would never run and the transaction would look
        // like it had failed for want of an outcome.
        if let Some(stop) = self.transition(State::Applying) {
            return Some(stop);
        }

        for index in 0..self.capsule.steps.len() {
            let step = self.capsule.steps[index].clone();
            let capsule_id = self.capsule.capsule_id.clone();

            if let Some(outcome) = self.ledger.lookup(&capsule_id, &step.idempotency_key) {
                match outcome {
                    StepOutcome::Succeeded => {
                        reused.push(step.step_id.clone());
                        continue;
                    }
                    StepOutcome::Failed => {
                        reused.push(step.step_id.clone());
                        return Some(self.fail(&format!(
                            "the step {:?} failed on an earlier attempt",
                            step.step_id
                        )));
                    }
                    StepOutcome::Interrupted { effect } => {
                        if effect != EffectState::NotApplied {
                            // The effect may have landed. Repeating a
                            // non-idempotent operation would apply it twice.
                            return Some(self.compensate(&format!(
                                "the step {:?} was interrupted and its effect is not known",
                                step.step_id
                            )));
                        }
                        // The effect definitely did not land, so the step may
                        // be attempted again.
                    }
                }
            }

            if let Some(fault) = self.fault_at(FaultPoint::BeforeStep { index }) {
                if fault.interrupts() {
                    self.ledger.record(
                        &capsule_id,
                        &step.idempotency_key,
                        StepOutcome::Interrupted { effect: EffectState::NotApplied },
                    );
                    return Some(self.fail(&format!(
                        "the step {:?} was interrupted before it ran ({fault:?})",
                        step.step_id
                    )));
                }
            }

            // The validity window is checked before every step, and not only
            // at the boundaries. A step that lands an effect after the
            // window closed is the case the window exists to prevent, and a
            // transaction that admits a capsule and then applies it an hour
            // later has defeated its own check.
            let now = match self.now() {
                Ok(now) => now,
                Err(stop) => return Some(stop),
            };
            if self.capsule.expires_at_ms <= now {
                let reason = "the capsule's validity window closed while the plan was running";
                // Nothing has landed yet, so the transaction failed. Something
                // has landed, so the transaction has to undo it before it can
                // report anything.
                return Some(if self.environment.applied_steps.is_empty() {
                    self.fail(reason)
                } else {
                    self.compensate(reason)
                });
            }

            // The step runs. A crash or a full disk leaves it unknown whether
            // the effect landed; every other fault leaves it known.
            let fault = self.fault_at(FaultPoint::DuringStep { index });
            let effect = match fault {
                // A machine that stopped, and a write that could not
                // complete, both leave the effect unknown. A write that fails
                // may have been partially applied, and "partially applied" is
                // not a state the ledger can describe.
                Some(Fault::Crash) | Some(Fault::DiskFull) => EffectState::Unknown,
                Some(_) => {
                    self.environment.apply_step(&step.step_id);
                    EffectState::Applied
                }
                None => {
                    self.environment.apply_step(&step.step_id);
                    EffectState::Applied
                }
            };

            if effect == EffectState::Unknown {
                self.ledger.record(
                    &capsule_id,
                    &step.idempotency_key,
                    StepOutcome::Interrupted { effect },
                );
                return Some(self.compensate(&format!(
                    "the step {:?} was interrupted and its effect is not known",
                    step.step_id
                )));
            }

            if let Some(fault) = fault {
                self.ledger.record(
                    &capsule_id,
                    &step.idempotency_key,
                    StepOutcome::Interrupted { effect },
                );
                return Some(self.compensate(&format!(
                    "the step {:?} was interrupted after its effect landed ({fault:?})",
                    step.step_id
                )));
            }

            let succeeded = step
                .success_predicates
                .iter()
                .all(|predicate| self.environment.postcondition_holds(predicate));
            if !succeeded {
                self.ledger.record(&capsule_id, &step.idempotency_key, StepOutcome::Failed);
                return Some(self.compensate(&format!(
                    "the step {:?} did not meet its success predicates",
                    step.step_id
                )));
            }

            self.ledger.record(&capsule_id, &step.idempotency_key, StepOutcome::Succeeded);
            executed.push(step.step_id.clone());

            // The step took the time it declared. The clock moves with the
            // work, which is what makes a validity window close *during* a
            // plan rather than only before it.
            self.environment.now_ms += step.timeout_ms as i64;

            if let Some(fault) = self.fault_at(FaultPoint::AfterStep { index }) {
                if fault.interrupts() {
                    return Some(self.compensate(&format!(
                        "the capsule was interrupted after the step {:?} ({fault:?})",
                        step.step_id
                    )));
                }
            }
        }
        self.transition(State::Verifying)
    }

    /// `VERIFYING → COMMITTED`: the outcome is proven, or it is not reported.
    fn verify(&mut self) -> Option<Stop> {
        if let Some(fault) = self.fault_at(FaultPoint::BeforeVerification) {
            if fault.interrupts() {
                return Some(self.compensate(&format!(
                    "the capsule was interrupted before verification ({fault:?})"
                )));
            }
        }
        for step in &self.capsule.steps {
            if self.ledger.lookup(&self.capsule.capsule_id, &step.idempotency_key).is_none() {
                return Some(
                    self.compensate(&format!("the step {:?} has no outcome", step.step_id)),
                );
            }
        }
        if let Some(fault) = self.fault_at(FaultPoint::DuringVerification) {
            if fault.interrupts() {
                return Some(
                    self.compensate(&format!("the verification was interrupted ({fault:?})")),
                );
            }
        }
        let mandatory = self.capsule.mandatory_postconditions();
        let unmet: Vec<&String> = mandatory
            .iter()
            .filter(|predicate| !self.environment.postcondition_holds(predicate))
            .collect();
        if !unmet.is_empty() {
            return Some(
                self.compensate(&format!("the mandatory postcondition {unmet:?} did not hold")),
            );
        }
        if !self.environment.audit_sink_durable {
            return Some(self.compensate("the audit sink cannot record the evidence durably"));
        }
        // The last read of the clock, and the last look at the validity
        // window. A commit is the moment the operator stops watching, and it
        // is the last moment at which the transaction can refuse.
        let now = match self.now() {
            Ok(now) => now,
            Err(stop) => return Some(stop),
        };
        if self.capsule.expires_at_ms <= now {
            return Some(self.compensate("the capsule's validity window closed before the commit"));
        }
        if self.capsule.recovery_plan.recovery_artifact_ref.is_some()
            && !self.environment.recovery_material_intact
        {
            return Some(self.compensate("the recovery artifact was corrupted before the commit"));
        }
        self.transition(State::Committed)
    }

    /// The effects that landed are undone, or the transaction escalates.
    fn compensate(&mut self, reason: &str) -> Stop {
        if !self.table.permits(self.state, State::Compensating) {
            // The table does not allow compensation from here, so the
            // transaction stays where it is rather than claiming a result it
            // cannot prove.
            return Stop { state: self.state, reason: reason.to_owned() };
        }
        self.state = State::Compensating;
        self.history.push(State::Compensating);

        if !self.capsule.recovery_plan.is_automatically_recoverable() {
            return self.escalate(reason);
        }
        // A step whose effect is not known cannot be undone: the executor does
        // not know whether there is anything to undo, and undoing the wrong
        // thing is worse than stopping.
        if self.has_unknown_effect() {
            return self.escalate(reason);
        }
        // The recovery material is checked **here**, and not only at the
        // precheck. Material that was present before the steps ran may be
        // gone or corrupt now, and a compensation that claims to have undone
        // an effect without material to undo it with is a false claim.
        if self.capsule.recovery_plan.recovery_artifact_ref.is_some() {
            if !self.environment.recovery_material_present {
                return self.escalate(reason);
            }
            if !self.environment.recovery_material_intact {
                return self.escalate(reason);
            }
        }
        for step in &self.environment.applied_steps {
            let Some(entry) = self.registry.effective(&step_operation(self, step)) else {
                return self.escalate(reason);
            };
            if !entry.reversibility.is_compensatable() {
                return self.escalate(reason);
            }
        }
        let applied = self.environment.applied_steps.clone();
        for step_id in applied.iter().rev() {
            self.environment.compensate_step(step_id);
        }
        if self.environment.outstanding_effects.is_empty() {
            self.state = State::RolledBack;
            self.history.push(State::RolledBack);
            Stop { state: State::RolledBack, reason: reason.to_owned() }
        } else {
            self.state = State::Partial;
            self.history.push(State::Partial);
            Stop { state: State::Partial, reason: reason.to_owned() }
        }
    }

    /// Whether any step's effect is not known.
    fn has_unknown_effect(&self) -> bool {
        self.capsule.steps.iter().any(|step| {
            matches!(
                self.ledger.lookup(&self.capsule.capsule_id, &step.idempotency_key),
                Some(StepOutcome::Interrupted { effect: EffectState::Unknown })
            )
        })
    }

    /// Automatic recovery cannot safely complete.
    fn escalate(&mut self, reason: &str) -> Stop {
        if self.table.permits(self.state, State::NeedsHumanRecovery) {
            self.state = State::NeedsHumanRecovery;
            self.history.push(State::NeedsHumanRecovery);
        }
        Stop {
            state: self.state,
            reason: format!("{reason}; automatic recovery cannot safely complete"),
        }
    }

    /// The capsule was refused before it could act.
    fn reject(&mut self, reason: &str) -> Stop {
        if self.table.permits(self.state, State::Rejected) {
            self.state = State::Rejected;
            self.history.push(State::Rejected);
        }
        Stop { state: self.state, reason: reason.to_owned() }
    }

    /// The capsule's validity window closed.
    fn expire(&mut self, reason: &str) -> Stop {
        if self.table.permits(self.state, State::Expired) {
            self.state = State::Expired;
            self.history.push(State::Expired);
        }
        Stop { state: self.state, reason: reason.to_owned() }
    }

    /// The capsule failed before it mutated anything.
    fn fail(&mut self, reason: &str) -> Stop {
        if self.table.permits(self.state, State::Failed) {
            self.state = State::Failed;
            self.history.push(State::Failed);
        }
        Stop { state: self.state, reason: reason.to_owned() }
    }

    /// Moves to a state the table allows, or stops where it is.
    fn transition(&mut self, to: State) -> Option<Stop> {
        if !self.table.permits(self.state, to) {
            return Some(Stop {
                state: self.state,
                reason: format!(
                    "the state table does not allow {} → {}",
                    self.state.as_str(),
                    to.as_str()
                ),
            });
        }
        self.state = to;
        self.history.push(to);
        None
    }

    fn outcome(&self, stop: Stop, executed: Vec<String>, reused: Vec<String>) -> ExecutionOutcome {
        ExecutionOutcome {
            final_state: stop.state,
            reason: stop.reason,
            steps_executed: executed,
            steps_reused: reused,
            history: self.history.clone(),
        }
    }
}

/// The operation an applied step performed, recovered from its identifier.
///
/// The simulated environment records step identifiers; the registry is keyed
/// by operation. The step identifier is the operation plus a suffix in the
/// simulator's own fixtures, so the operation is recovered by taking the part
/// before the first `.`.
fn step_operation(executor: &Executor, step_id: &str) -> String {
    executor
        .capsule
        .steps
        .iter()
        .find(|step| step.step_id == step_id)
        .map(|step| step.operation.clone())
        .unwrap_or_else(|| step_id.split('.').next().unwrap_or(step_id).to_owned())
}

/// A stop: where the transaction ended and why.
struct Stop {
    state: State,
    reason: String,
}

//! Senvoryn Change Capsule executor simulator.
//!
//! This crate implements the Change Capsule transaction of Core Contract
//! Specification §5 as a **simulator**: it drives the state machine, keeps the
//! idempotency ledger, rechecks the world before each privileged step, and
//! simulates compensation and failure. It acts on a simulated environment, not
//! on a real system, and it performs no effect of its own.
//!
//! # What it is for
//!
//! The gate this crate serves has one exit criterion: **an interrupted
//! operation never falsely reports `COMMITTED`.** The crate is built so that
//! criterion is checkable rather than asserted — faults are injected at named
//! points, every path to `COMMITTED` is guarded, and the property is tested
//! over the whole fault space.
//!
//! # What it is not
//!
//! It is not an executor. It verifies no signature, resolves no resource
//! against a filesystem, and writes no audit record. It does not decide
//! policy: the System Control Plane does that, and this crate takes the
//! decision as an input and checks that it binds the capsule's own digest.
//! Two authorities on the same question is the failure mode the shared
//! conformance vectors exist to prevent.
//!
//! # The parts
//!
//! * [`capsule::ChangeCapsule`] — the plan, read from the repository's schema.
//! * [`machine::TransitionTable`] — the allowed transitions, read from the
//!   repository's own state table.
//! * [`executor::Executor`] — the driver.
//! * [`ledger::IdempotencyLedger`] — what has already been attempted, and
//!   what is still not known.
//! * [`environment::FaultInjector`] — where the world breaks.

#![deny(missing_docs)]

pub mod authorization;
pub mod capsule;
pub mod environment;
pub mod executor;
pub mod ledger;
pub mod machine;

pub use authorization::{Approval, Authorization, AuthorizationError};
pub use capsule::{CapsuleError, ChangeCapsule};
pub use environment::{Environment, Fault, FaultInjector, FaultPoint};
pub use executor::{ExecutionOutcome, Executor, ExecutorError};
pub use ledger::{EffectState, IdempotencyLedger, StepOutcome};
pub use machine::{MachineError, State, Transition, TransitionTable};

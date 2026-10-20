//! Senvoryn System Control Plane — the decision oracle.
//!
//! This crate implements the sixteen validation gates of Core Contract
//! Specification §3.2 as a **pure** evaluator: it reads no clock, performs no
//! I/O, mutates nothing, and returns a decision that grants nothing by
//! itself. Effect separation is the sixteenth of the sixteen gates, and it is
//! enforced by the decision type having no field in which an effect could be
//! recorded.
//!
//! # What it is not
//!
//! It is **not** an SCP. It is the decision function an SCP would call, and
//! it deliberately has no cryptographic boundary: it does not verify
//! signatures, it does not resolve resources against a filesystem, and it
//! does not talk to an audit sink. It is the part that decides, tested
//! against the part that has to be true for the decision to be right.
//!
//! It is **specified, not independently reviewed**.
//!
//! # The parts
//!
//! * [`evaluator::evaluate`] — the sixteen gates, in the order the
//!   specification gives them.
//! * [`capability::attenuate`] — the attenuation rule of specification §4.2.
//! * [`registry::OperationRegistry`] — the trusted Operation Registry, and
//!   the "most restrictive rule wins" merge of specification §3.2.
//! * [`reason`] — the reason codes, checked against the denial-code registry
//!   that is their authority.
//! * [`audit`] — the audit-event builder.
//!
//! # Keeping in step with the prototype
//!
//! The Python reference model in `reference/scp_reference.py` implements the
//! same gates. The shared conformance vectors in `conformance/vectors/` are
//! executed by both, and CI fails if the two disagree about any decision or
//! reason code. That is the mechanism that stops them becoming two
//! authorities on the same question.

#![deny(missing_docs)]

mod audit;
mod capability;
mod context;
mod decision;
mod evaluator;
mod reason;
mod registry;
mod request;

pub use audit::{DataClassification, TrustedTimeStatus, build_audit_event};
pub use capability::{Capability, Provenance, attenuate};
pub use context::{AuditState, PolicyContext, SystemMode};
pub use decision::{Decision, DecisionStatus, RecordDetails};
pub use evaluator::{evaluate, registry_risk_tier};
pub use reason::{CodeEntry, ReasonCode, registry_codes, registry_default_status, registry_entry};
pub use registry::{AuditClass, OperationEntry, OperationRegistry, RegistryError, RiskTier};
pub use request::{Budget, EnvelopeError, Request};

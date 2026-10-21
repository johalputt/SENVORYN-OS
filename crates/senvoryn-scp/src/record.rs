//! The decision record, read back.
//!
//! The policy layer writes a decision record; something else has to read it.
//! The capsule executor is that something else: it is handed a decision and a
//! plan, and it has to know that the decision was made about that plan. Until
//! now the executor was handed a value the host assembled, and a value the
//! host assembles is a value the host can assemble wrongly.
//!
//! This module reads the record the policy layer produces. The reader is
//! **closed** for the same reason the policy request reader is (ADR-019): a
//! record that carries a field this reader does not define is refused rather
//! than read, because a field the reader does not account for may be a field
//! a newer writer expects this reader to enforce. The reader also refuses a
//! record that claims an effect was performed: the record's own schema says
//! `const: false`, and a record that says otherwise was not written by the
//! policy layer.
//!
//! Reading a record grants nothing. The reader reports what the record says;
//! whether what it says permits the plan in hand is the executor's question.

use senvoryn_canonical::Value;

use crate::decision::DecisionStatus;

/// The fields a decision record defines.
const FIELDS: &[&str] = &[
    "schema",
    "decision_id",
    "request_id",
    "status",
    "reason_codes",
    "policy_epoch",
    "evaluated_at",
    "target_resource_id",
    "target_resource_version",
    "permitted_payload_digest",
    "capability_scope_ref",
    "valid_until",
    "required_next_step",
    "effect_performed",
    "audit_ref",
];

/// Why a document could not be read as a decision record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordError {
    /// A required field was missing, of the wrong type, or out of range.
    Malformed {
        /// The field at fault.
        field: &'static str,
        /// What was wrong with it.
        detail: String,
    },
    /// The document carried a field the decision record does not define.
    UnknownField {
        /// The field that is not in the record.
        field: String,
    },
    /// The document declared a schema this node does not support.
    UnsupportedSchema {
        /// The schema the document declared.
        declared: String,
    },
    /// The document claimed that an effect was performed.
    ///
    /// The record's schema fixes `effect_performed` to false, and effect
    /// separation is the sixteenth of the sixteen gates: the policy layer
    /// cannot have performed one. A record that says it did was not written
    /// by the policy layer, and a reader that accepted it would be reading
    /// an authority it cannot check.
    EffectClaimed,
}

impl core::fmt::Display for RecordError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Malformed { field, detail } => {
                write!(f, "the decision record's {field:?} is {detail}")
            }
            Self::UnknownField { field } => {
                write!(f, "the decision record carries a field it does not define: {field:?}")
            }
            Self::UnsupportedSchema { declared } => {
                write!(f, "the decision record declares an unsupported schema: {declared:?}")
            }
            Self::EffectClaimed => {
                write!(f, "the decision record claims an effect was performed")
            }
        }
    }
}

impl std::error::Error for RecordError {}

/// A decision record, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionRecord {
    /// The decision's own identifier.
    pub decision_id: String,
    /// The request the decision was made for.
    pub request_id: String,
    /// What was decided.
    pub status: DecisionStatus,
    /// The reason codes the decision carries.
    pub reason_codes: Vec<String>,
    /// The policy epoch the decision was made under.
    pub policy_epoch: u64,
    /// When the decision was made, as a date-time.
    pub evaluated_at: String,
    /// The canonical object the decision was about.
    pub target_resource_id: String,
    /// The version of that object the decision was made against.
    ///
    /// `None` when the decision was refused before the resource was
    /// resolved, or when the request named no plan and so there was no
    /// version to name. A decision that names no version was made about a
    /// name rather than an object, and it authorises no effect on an object.
    pub target_resource_version: Option<String>,
    /// The digest of the plan the request named.
    ///
    /// The request's claim about the plan it asked to run, carried into the
    /// record so that the executor can compare it with the plan it was
    /// handed. The policy layer does not compute the digest (ADR-002) and
    /// does not judge it; it carries it.
    pub permitted_payload_digest: Option<String>,
    /// The capability the authority was scoped to, if any.
    pub capability_scope_ref: Option<String>,
    /// How long the granted authority lasts, if it is bounded.
    pub valid_until: Option<String>,
    /// What the caller must do next, if anything.
    pub required_next_step: Option<String>,
    /// The audit event that records this decision.
    pub audit_ref: String,
}

impl DecisionRecord {
    /// Reads a document as a decision record.
    ///
    /// The reader enforces the form and nothing else: it reads the fields,
    /// it refuses a field it does not define, and it refuses a record that
    /// claims an effect. It does not judge the decision.
    pub fn from_value(document: &Value) -> Result<Self, RecordError> {
        let object = match document.as_object() {
            Some(object) => object,
            None => {
                return Err(RecordError::Malformed {
                    field: "document",
                    detail: format!("expected an object, found {}", document.type_name()),
                });
            }
        };
        for key in object.keys() {
            if !FIELDS.contains(&key.as_str()) {
                return Err(RecordError::UnknownField { field: key.clone() });
            }
        }

        let schema = required_string(document, "schema")?;
        if schema != "senvoryn.decision-record.v1" {
            return Err(RecordError::UnsupportedSchema { declared: schema });
        }

        let status_text = required_string(document, "status")?;
        let status = DecisionStatus::parse(&status_text).ok_or_else(|| RecordError::Malformed {
            field: "status",
            detail: format!("{status_text:?} is not a decision status"),
        })?;

        let effect_performed = match document.get("effect_performed") {
            Some(Value::Bool(value)) => *value,
            _ => {
                return Err(RecordError::Malformed {
                    field: "effect_performed",
                    detail: "absent or not a boolean".to_owned(),
                });
            }
        };
        if effect_performed {
            return Err(RecordError::EffectClaimed);
        }

        let reason_codes = match document.get("reason_codes") {
            Some(Value::Array(items)) => {
                let mut codes = Vec::with_capacity(items.len());
                for item in items {
                    let code = item.as_str().ok_or_else(|| RecordError::Malformed {
                        field: "reason_codes",
                        detail: "carries an entry that is not a string".to_owned(),
                    })?;
                    codes.push(code.to_owned());
                }
                codes
            }
            _ => {
                return Err(RecordError::Malformed {
                    field: "reason_codes",
                    detail: "absent or not an array".to_owned(),
                });
            }
        };

        let policy_epoch = match document.get("policy_epoch").and_then(Value::as_int) {
            Some(epoch) if epoch >= 0 => u64::try_from(epoch).unwrap_or(u64::MAX),
            _ => {
                return Err(RecordError::Malformed {
                    field: "policy_epoch",
                    detail: "absent, not an integer or negative".to_owned(),
                });
            }
        };

        Ok(Self {
            decision_id: required_string(document, "decision_id")?,
            request_id: required_string(document, "request_id")?,
            status,
            reason_codes,
            policy_epoch,
            evaluated_at: required_string(document, "evaluated_at")?,
            target_resource_id: required_string(document, "target_resource_id")?,
            target_resource_version: optional_string(document, "target_resource_version")?,
            permitted_payload_digest: optional_string(document, "permitted_payload_digest")?,
            capability_scope_ref: optional_string(document, "capability_scope_ref")?,
            valid_until: optional_string(document, "valid_until")?,
            required_next_step: optional_string(document, "required_next_step")?,
            audit_ref: required_string(document, "audit_ref")?,
        })
    }
}

fn required_string(document: &Value, field: &'static str) -> Result<String, RecordError> {
    let text = document.get(field).and_then(Value::as_str).ok_or_else(|| {
        RecordError::Malformed { field, detail: "absent or not a string".to_owned() }
    })?;
    if text.is_empty() {
        return Err(RecordError::Malformed { field, detail: "present but empty".to_owned() });
    }
    Ok(text.to_owned())
}

fn optional_string(document: &Value, field: &'static str) -> Result<Option<String>, RecordError> {
    match document.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => {
            if text.is_empty() {
                return Err(RecordError::Malformed {
                    field,
                    detail: "present but empty".to_owned(),
                });
            }
            Ok(Some(text.clone()))
        }
        Some(_) => Err(RecordError::Malformed { field, detail: "not a string".to_owned() }),
    }
}

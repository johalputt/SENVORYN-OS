//! The request, as the policy layer sees it.
//!
//! Two documents are called a request, and they are not the same one. The
//! *wire envelope* (`schemas/request-envelope.schema.json`) is what travels:
//! it carries date-times, a replay key, a target node and the caller's
//! classification and effect claims. The *policy request* is what the
//! transport layer hands to the policy layer: the same intent with the times
//! already expressed in monotonic milliseconds and the fields the policy
//! layer does not read removed.
//!
//! This module reads the policy request. The transport layer that produces it
//! is not built yet, so the wire envelope has no reader in this repository
//! and the two forms have drifted apart: the policy request has no replay key
//! distinct from its nonce, no target node, no classification claim and no
//! effect claim. Those gaps are recorded in ADR-019 rather than papered over,
//! because a reader that pretends to be the wire envelope's reader is a
//! reader whose contract nobody can state.

use senvoryn_canonical::Value;

use crate::registry::RiskTier;

/// The budget a request asks for.
///
/// An absent field is unbounded *by the request*. Whether an unbounded
/// request is allowed is the budget gate's business, and it answers that from
/// the registry rather than from the request.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Budget {
    /// The longest the operation may run, in milliseconds.
    pub duration_ms: Option<u64>,
    /// The most operations the request may perform.
    pub operation_count: Option<u64>,
}

/// One request envelope.
///
/// Times are monotonic milliseconds since boot. The wire format carries
/// date-times, and converting them to monotonic values is the transport
/// layer's job: the policy layer compares against a monotonic clock so that
/// a wall-clock change cannot extend or cut short a capability's life, and a
/// wall clock is a fallible instrument. The conversion is the reason the
/// freshness gate also checks the boot epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The request's own identifier.
    pub request_id: String,
    /// The principal making the request.
    pub principal_id: String,
    /// The node or service the request is addressed to.
    pub recipient_id: String,
    /// The resource the request names.
    pub resource_id: String,
    /// The kind of resource the request names.
    pub resource_type: String,
    /// The operation requested.
    pub operation: String,
    /// The reason the operation is wanted.
    pub purpose: String,
    /// The policy generation the request was made under.
    pub policy_epoch: u64,
    /// When the request became valid, in monotonic milliseconds.
    pub issued_at: u64,
    /// When the request stops being valid, in monotonic milliseconds.
    pub expires_at: u64,
    /// The value that makes this request unique.
    pub nonce: String,
    /// The risk the caller claims for the operation.
    pub risk_claim: RiskTier,
    /// The digest of the action the approval is bound to.
    pub approval_digest: Option<String>,
    /// The digest of the plan the request asks to run.
    ///
    /// A request that names a plan says which plan it is about; a request
    /// that names none asks for a single operation. The digest is the
    /// caller's claim about the plan — the policy layer does not compute it
    /// (ADR-002) and does not judge it — but it is the thread the decision
    /// record carries, so that the executor can compare the plan it was
    /// handed with the plan the decision was made about.
    pub payload_digest: Option<String>,
    /// The budget the request asks for.
    pub budget: Option<Budget>,
}

/// The fields the policy request form defines.
///
/// The form is closed: a document carrying a field this list does not name is
/// refused rather than read. The specification requires an unknown critical
/// field to be rejected, and the form marks no field as non-critical, so a
/// field this node does not understand may be a field a newer sender expects
/// this node to enforce. Ignoring it would be a decision to enforce a
/// contract nobody wrote down.
const FIELDS: &[&str] = &[
    "schema",
    "request_id",
    "principal_id",
    "recipient_id",
    "resource_id",
    "resource_type",
    "operation",
    "purpose",
    "policy_epoch",
    "issued_at",
    "expires_at",
    "nonce",
    "risk_claim",
    "approval_digest",
    "payload_digest",
    "budget",
];

/// The fields the policy request form's budget defines.
const BUDGET_FIELDS: &[&str] = &["duration_ms", "operation_count"];

/// Why a document could not be read as a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeError {
    /// A required field was missing, of the wrong type, or out of range.
    Malformed {
        /// The field at fault.
        field: &'static str,
        /// What was wrong with it.
        detail: String,
    },
    /// The document carried a field the policy request form does not define.
    UnknownField {
        /// The field that is not in the form.
        field: String,
    },
    /// The document declared a schema this node does not support.
    UnsupportedSchema {
        /// The schema that was declared.
        declared: String,
    },
}

impl core::fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Malformed { field, detail } => {
                write!(f, "field {field:?} is malformed: {detail}")
            }
            Self::UnknownField { field } => {
                write!(f, "field {field:?} is not part of the policy request form")
            }
            Self::UnsupportedSchema { declared } => {
                write!(f, "unsupported request schema {declared:?}")
            }
        }
    }
}

impl std::error::Error for EnvelopeError {}

impl Request {
    /// Reads a request from a canonical document.
    ///
    /// The document is the policy request form: the transport layer's output,
    /// with the times already expressed in monotonic milliseconds. The reader
    /// enforces the form — the field set is closed, the required fields are
    /// present and non-empty, the numbers are non-negative integers and the
    /// risk claim is a tier this node knows — and nothing else. It does not
    /// judge the request: an operation this node has never heard of, a
    /// resource it cannot resolve and a principal it does not know are all
    /// read successfully and refused by the gates that follow.
    pub fn from_value(document: &Value) -> Result<Self, EnvelopeError> {
        reject_unknown_fields(document, "document", FIELDS)?;

        let schema =
            document.get("schema").and_then(Value::as_str).ok_or(EnvelopeError::Malformed {
                field: "schema",
                detail: "absent or not a string".to_owned(),
            })?;
        if schema != "senvoryn.request-envelope.v1" {
            return Err(EnvelopeError::UnsupportedSchema { declared: schema.to_owned() });
        }

        let request_id = required_string(document, "request_id")?;
        let principal_id = required_string(document, "principal_id")?;
        let recipient_id = required_string(document, "recipient_id")?;
        let resource_id = required_string(document, "resource_id")?;
        let resource_type = required_string(document, "resource_type")?;
        let operation = required_string(document, "operation")?;
        let purpose = required_string(document, "purpose")?;
        let nonce = required_string(document, "nonce")?;

        let policy_epoch = required_u64(document, "policy_epoch")?;
        let issued_at = required_u64(document, "issued_at")?;
        let expires_at = required_u64(document, "expires_at")?;

        let risk_claim_text = required_string(document, "risk_claim")?;
        let risk_claim =
            RiskTier::parse(&risk_claim_text).ok_or_else(|| EnvelopeError::Malformed {
                field: "risk_claim",
                detail: format!("{risk_claim_text:?} is not a risk tier"),
            })?;

        let approval_digest = optional_string(document, "approval_digest")?;
        let payload_digest = optional_string(document, "payload_digest")?;
        let budget = match document.get("budget") {
            None | Some(Value::Null) => None,
            Some(value) => {
                reject_unknown_fields(value, "budget", BUDGET_FIELDS)?;
                Some(Budget {
                    duration_ms: optional_u64(value, "duration_ms")?,
                    operation_count: optional_u64(value, "operation_count")?,
                })
            }
        };

        Ok(Self {
            request_id,
            principal_id,
            recipient_id,
            resource_id,
            resource_type,
            operation,
            purpose,
            policy_epoch,
            issued_at,
            expires_at,
            nonce,
            risk_claim,
            approval_digest,
            payload_digest,
            budget,
        })
    }
}

/// Refuses a document that is not an object, or that carries a field the
/// form does not define.
///
/// The check is on the way in rather than on each field, so a document with
/// an extra field is refused for the extra field and not for the first
/// required field that happens to be missing.
fn reject_unknown_fields(
    document: &Value,
    container: &'static str,
    fields: &[&str],
) -> Result<(), EnvelopeError> {
    let object = match document.as_object() {
        Some(object) => object,
        None => {
            return Err(EnvelopeError::Malformed {
                field: container,
                detail: format!("expected an object, found {}", document.type_name()),
            });
        }
    };
    for key in object.keys() {
        if !fields.contains(&key.as_str()) {
            return Err(EnvelopeError::UnknownField { field: key.clone() });
        }
    }
    Ok(())
}

fn required_string(document: &Value, field: &'static str) -> Result<String, EnvelopeError> {
    let text = document.get(field).and_then(Value::as_str).ok_or_else(|| {
        EnvelopeError::Malformed { field, detail: "absent or not a string".to_owned() }
    })?;
    if text.is_empty() {
        return Err(EnvelopeError::Malformed { field, detail: "present but empty".to_owned() });
    }
    Ok(text.to_owned())
}

fn optional_string(document: &Value, field: &'static str) -> Result<Option<String>, EnvelopeError> {
    match document.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(other) => Err(EnvelopeError::Malformed {
            field,
            detail: format!("expected a string, found {}", other.type_name()),
        }),
    }
}

fn required_u64(document: &Value, field: &'static str) -> Result<u64, EnvelopeError> {
    let number = document.get(field).and_then(Value::as_int).ok_or_else(|| {
        EnvelopeError::Malformed { field, detail: "absent or not an integer".to_owned() }
    })?;
    u64::try_from(number)
        .map_err(|_| EnvelopeError::Malformed { field, detail: format!("{number} is negative") })
}

fn optional_u64(document: &Value, field: &'static str) -> Result<Option<u64>, EnvelopeError> {
    match document.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => {
            let number = value.as_int().ok_or_else(|| EnvelopeError::Malformed {
                field,
                detail: format!("expected an integer, found {}", value.type_name()),
            })?;
            u64::try_from(number).map(Some).map_err(|_| EnvelopeError::Malformed {
                field,
                detail: format!("{number} is negative"),
            })
        }
    }
}

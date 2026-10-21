//! The policy request form: what the reader enforces, and what it refuses.
//!
//! These are the reader's own controls. The gates that judge a request are
//! tested by the shared conformance vectors; what is tested here is the
//! boundary before them — the place where a document becomes a `Request`, or
//! does not.

use senvoryn_canonical::{Value, parse};
use senvoryn_scp::{EnvelopeError, Request, RiskTier};

/// A policy request the reader accepts.
const VALID: &str = r#"{
  "schema": "senvoryn.request-envelope.v1",
  "request_id": "req:example:0001",
  "principal_id": "user:alice",
  "recipient_id": "node:local",
  "resource_id": "file:report",
  "resource_type": "file_object",
  "operation": "read",
  "purpose": "user-request",
  "policy_epoch": 7,
  "issued_at": 100,
  "expires_at": 900,
  "nonce": "nonce-example-0001",
  "risk_claim": "R1",
  "approval_digest": "sha256:approval",
  "budget": { "duration_ms": 500, "operation_count": 3 }
}"#;

fn document(text: &str) -> Value {
    parse(text).expect("the test document is canonical")
}

fn read(text: &str) -> Result<Request, EnvelopeError> {
    Request::from_value(&document(text))
}

#[test]
fn a_policy_request_is_read_as_it_was_written() {
    let request = read(VALID).expect("the request is well formed");
    assert_eq!(request.request_id, "req:example:0001");
    assert_eq!(request.principal_id, "user:alice");
    assert_eq!(request.recipient_id, "node:local");
    assert_eq!(request.resource_id, "file:report");
    assert_eq!(request.resource_type, "file_object");
    assert_eq!(request.operation, "read");
    assert_eq!(request.purpose, "user-request");
    assert_eq!(request.policy_epoch, 7);
    assert_eq!(request.issued_at, 100);
    assert_eq!(request.expires_at, 900);
    assert_eq!(request.nonce, "nonce-example-0001");
    assert_eq!(request.risk_claim, RiskTier::R1);
    assert_eq!(request.approval_digest.as_deref(), Some("sha256:approval"));
    let budget = request.budget.expect("the request carries a budget");
    assert_eq!(budget.duration_ms, Some(500));
    assert_eq!(budget.operation_count, Some(3));
}

#[test]
fn a_document_with_an_unknown_field_is_refused() {
    let text = VALID.replace(
        "\"risk_claim\": \"R1\"",
        "\"risk_claim\": \"R1\", \"unexpected_privilege\": \"root\"",
    );
    assert_eq!(
        read(&text),
        Err(EnvelopeError::UnknownField { field: "unexpected_privilege".to_owned() })
    );
}

#[test]
fn a_document_with_an_unknown_budget_field_is_refused() {
    let text = VALID.replace(
        "\"budget\": { \"duration_ms\": 500, \"operation_count\": 3 }",
        "\"budget\": { \"duration_ms\": 500, \"cpu_ms\": 12 }",
    );
    assert_eq!(read(&text), Err(EnvelopeError::UnknownField { field: "cpu_ms".to_owned() }));
}

#[test]
fn a_document_that_is_not_an_object_is_refused() {
    assert_eq!(
        read("[1, 2, 3]"),
        Err(EnvelopeError::Malformed {
            field: "document",
            detail: "expected an object, found array".to_owned(),
        })
    );
}

#[test]
fn a_document_with_an_unsupported_schema_is_refused() {
    let text = VALID.replace("senvoryn.request-envelope.v1", "senvoryn.request-envelope.v2");
    assert_eq!(
        read(&text),
        Err(EnvelopeError::UnsupportedSchema {
            declared: "senvoryn.request-envelope.v2".to_owned()
        })
    );
}

#[test]
fn a_document_without_a_required_field_is_refused() {
    let text = VALID.replace("\"nonce\": \"nonce-example-0001\",", "");
    assert!(
        matches!(read(&text), Err(EnvelopeError::Malformed { field: "nonce", .. })),
        "a request without a nonce must be refused for the nonce"
    );
}

#[test]
fn a_document_with_an_empty_required_field_is_refused() {
    let text = VALID.replace("\"purpose\": \"user-request\"", "\"purpose\": \"\"");
    assert!(
        matches!(read(&text), Err(EnvelopeError::Malformed { field: "purpose", .. })),
        "a request with an empty purpose must be refused for the purpose"
    );
}

#[test]
fn a_document_with_a_negative_number_is_refused() {
    let text = VALID.replace("\"issued_at\": 100", "\"issued_at\": -1");
    assert!(
        matches!(read(&text), Err(EnvelopeError::Malformed { field: "issued_at", .. })),
        "a request with a negative issued_at must be refused for the issued_at"
    );
}

#[test]
fn a_document_with_a_number_that_is_not_an_integer_is_refused() {
    let text = VALID.replace("\"policy_epoch\": 7", "\"policy_epoch\": \"7\"");
    assert!(
        matches!(read(&text), Err(EnvelopeError::Malformed { field: "policy_epoch", .. })),
        "a request whose policy_epoch is a string must be refused for the policy_epoch"
    );
}

#[test]
fn a_document_with_a_malformed_risk_claim_is_refused() {
    let text = VALID.replace("\"risk_claim\": \"R1\"", "\"risk_claim\": \"R9\"");
    assert!(
        matches!(read(&text), Err(EnvelopeError::Malformed { field: "risk_claim", .. })),
        "a request with an unknown risk tier must be refused for the risk_claim"
    );
}

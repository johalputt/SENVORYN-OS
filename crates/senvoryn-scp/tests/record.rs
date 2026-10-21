//! The decision record, read back.
//!
//! The policy layer writes a decision record and the capsule executor reads
//! it. The two ends of that boundary are tested here together, because the
//! failure mode they have to avoid is a record whose writer and reader have
//! drifted apart: the executor would then be checking the plan and the
//! version against fields the decision never carried.

use senvoryn_canonical::Value;
use senvoryn_scp::{
    Decision, DecisionRecord, DecisionStatus, ReasonCode, RecordDetails, ResolvedResource,
};

/// A decision that permitted a plan against a version of the target.
fn decision() -> Decision {
    Decision::permit(ReasonCode::ALL_POLICY_GATES_PASSED, "capability-0001".to_owned(), 7)
        .with_resolved(ResolvedResource {
            canonical_id: "file:///srv/data/report.csv".to_owned(),
            resource_type: "file_object".to_owned(),
            version: "7".to_owned(),
        })
}

/// The record a decision makes, carrying the plan the request named.
fn record(permitted_payload_digest: Option<&str>) -> Value {
    decision().to_record(&RecordDetails {
        decision_id: "decision:record:0001",
        request_id: "request:record:0001",
        evaluated_at: "2026-01-01T00:00:00Z",
        presented_resource_id: "file:///srv/data/report.csv",
        permitted_payload_digest,
        audit_ref: "audit:record:0001",
        valid_until: None,
        required_next_step: None,
    })
}

#[test]
fn a_record_the_policy_layer_wrote_is_read_back() {
    let document = record(Some("0000000000000000000000000000000000000000000000000000000000000002"));
    let read = DecisionRecord::from_value(&document)
        .unwrap_or_else(|error| panic!("the record the policy layer wrote was refused: {error}"));
    assert_eq!(read.status, DecisionStatus::Permit);
    assert_eq!(read.request_id, "request:record:0001");
    assert_eq!(read.policy_epoch, 7);
    assert_eq!(read.target_resource_id, "file:///srv/data/report.csv");
    assert_eq!(read.target_resource_version.as_deref(), Some("7"));
    assert_eq!(
        read.permitted_payload_digest.as_deref(),
        Some("0000000000000000000000000000000000000000000000000000000000000002")
    );
    assert_eq!(read.audit_ref, "audit:record:0001");
}

#[test]
fn a_record_carries_the_plan_the_request_named() {
    // The digest is the request's claim about the plan it asked to run, and
    // the record is what the executor compares the plan it was handed with.
    // A record that does not carry it authorises no plan.
    let document = record(Some("0000000000000000000000000000000000000000000000000000000000000002"));
    assert_eq!(
        document.get("permitted_payload_digest").and_then(Value::as_str),
        Some("0000000000000000000000000000000000000000000000000000000000000002"),
        "the record does not name the plan the request asked to run"
    );
    let without = record(None);
    assert_eq!(
        without.get("permitted_payload_digest"),
        Some(&Value::Null),
        "a request that named no plan must leave the record's digest null"
    );
}

#[test]
fn a_record_with_a_field_the_record_does_not_define_is_refused() {
    // The form is closed: a field the reader does not account for may be a
    // field a newer writer expects the reader to enforce.
    let mut document =
        record(Some("0000000000000000000000000000000000000000000000000000000000000002"));
    if let Value::Object(map) = &mut document {
        map.insert("effect_confirmed".to_owned(), Value::Bool(true));
    }
    let error =
        DecisionRecord::from_value(&document).expect_err("a record with an unknown field was read");
    assert!(
        matches!(error, senvoryn_scp::RecordError::UnknownField { .. }),
        "the record was refused for the wrong reason: {error}"
    );
}

#[test]
fn a_record_that_claims_an_effect_was_performed_is_refused() {
    // The record's schema fixes `effect_performed` to false, and the policy
    // layer cannot have performed one. A record that says it did was not
    // written by the policy layer.
    let mut document =
        record(Some("0000000000000000000000000000000000000000000000000000000000000002"));
    if let Value::Object(map) = &mut document {
        map.insert("effect_performed".to_owned(), Value::Bool(true));
    }
    let error =
        DecisionRecord::from_value(&document).expect_err("a record claiming an effect was read");
    assert!(
        matches!(error, senvoryn_scp::RecordError::EffectClaimed),
        "the record was refused for the wrong reason: {error}"
    );
}

#[test]
fn a_record_with_a_status_this_node_does_not_know_is_refused() {
    // The decision vocabulary is closed. A record that says something this
    // node does not understand is a record this node cannot act on, and
    // treating an unknown status as anything at all would be a guess.
    let mut document =
        record(Some("0000000000000000000000000000000000000000000000000000000000000002"));
    if let Value::Object(map) = &mut document {
        map.insert("status".to_owned(), Value::from("MAYBE"));
    }
    let error = DecisionRecord::from_value(&document)
        .expect_err("a record with an unknown status was read");
    assert!(
        matches!(error, senvoryn_scp::RecordError::Malformed { field: "status", .. }),
        "the record was refused for the wrong reason: {error}"
    );
}

#[test]
fn a_record_with_another_schema_is_refused() {
    let mut document =
        record(Some("0000000000000000000000000000000000000000000000000000000000000002"));
    if let Value::Object(map) = &mut document {
        map.insert("schema".to_owned(), Value::from("senvoryn.decision-record.v2"));
    }
    let error =
        DecisionRecord::from_value(&document).expect_err("a record with another schema was read");
    assert!(
        matches!(error, senvoryn_scp::RecordError::UnsupportedSchema { .. }),
        "the record was refused for the wrong reason: {error}"
    );
}

//! Executes the shared conformance vectors against the Rust implementation.
//!
//! The Python reference model in `reference/scp_reference.py` executes the
//! same file. A disagreement between the two about any decision or reason
//! code fails both suites, and CI runs both. That is the mechanism that keeps
//! the prototype and the implementation from drifting into two authorities on
//! the same question.

use std::collections::{BTreeMap, BTreeSet};

use senvoryn_canonical::{Value, parse};
use senvoryn_scp::{
    AuditState, Budget, Capability, DecisionStatus, OperationEntry, OperationRegistry,
    PolicyContext, Provenance, ReasonCode, Request, RiskTier, SystemMode, attenuate, evaluate,
};

const VECTORS_JSON: &str = include_str!("../../../conformance/vectors/vectors.json");

fn string_field(value: &Value, field: &str) -> String {
    value
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("field {field:?} is missing or not a string"))
        .to_owned()
}

fn optional_string(value: &Value, field: &str) -> Option<String> {
    match value.get(field) {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => Some(text.clone()),
        Some(other) => panic!("field {field:?} is not a string but {}", other.type_name()),
    }
}

fn u64_field(value: &Value, field: &str) -> u64 {
    let number = value
        .get(field)
        .and_then(Value::as_int)
        .unwrap_or_else(|| panic!("field {field:?} is missing or not an integer"));
    u64::try_from(number).unwrap_or_else(|_| panic!("field {field:?} is negative"))
}

fn optional_u64(value: &Value, field: &str) -> Option<u64> {
    match value.get(field) {
        None | Some(Value::Null) => None,
        Some(inner) => {
            let number =
                inner.as_int().unwrap_or_else(|| panic!("field {field:?} is not an integer"));
            Some(u64::try_from(number).unwrap_or_else(|_| panic!("field {field:?} is negative")))
        }
    }
}

fn string_set(value: &Value, field: &str) -> BTreeSet<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).map(str::to_owned).collect())
        .unwrap_or_default()
}

fn operation_entry(raw: &Value) -> OperationEntry {
    OperationEntry {
        operation_id: string_field(raw, "operation_id"),
        risk_tier: RiskTier::parse(&string_field(raw, "risk_tier"))
            .unwrap_or_else(|| panic!("bad risk tier in the vector file")),
        approval_requirements: string_set(raw, "approval_requirements"),
        allowed_modes: string_set(raw, "allowed_modes"),
        resource_types: string_set(raw, "resource_types"),
        max_duration_ms: optional_u64(raw, "max_duration_ms"),
        max_operation_count: optional_u64(raw, "max_operation_count"),
        required_preconditions: string_set(raw, "required_preconditions"),
        audit_class: raw
            .get("audit_class")
            .and_then(Value::as_str)
            .and_then(senvoryn_scp::AuditClass::parse)
            .unwrap_or(senvoryn_scp::AuditClass::Standard),
    }
}

fn operation_registry(entries: &[Value]) -> OperationRegistry {
    OperationRegistry::from_entries(entries.iter().map(operation_entry).collect())
}

fn capability_from(raw: &Value) -> Capability {
    Capability {
        capability_id: string_field(raw, "capability_id"),
        subject_id: string_field(raw, "subject_id"),
        resource_id: string_field(raw, "resource_id"),
        resource_type: string_field(raw, "resource_type"),
        operations: string_set(raw, "operations"),
        purposes: string_set(raw, "purposes"),
        not_before: u64_field(raw, "not_before"),
        expires_at: u64_field(raw, "expires_at"),
        policy_epoch: u64_field(raw, "policy_epoch"),
        provenance: Provenance::parse(&string_field(raw, "provenance"))
            .unwrap_or_else(|| panic!("bad provenance in the vector file")),
        delegation_depth: raw
            .get("delegation_depth")
            .and_then(Value::as_int)
            .and_then(|number| u32::try_from(number).ok())
            .unwrap_or(0),
        max_uses: optional_u64(raw, "max_uses"),
    }
}

fn request_from(raw: &Value) -> Request {
    Request {
        request_id: string_field(raw, "request_id"),
        principal_id: string_field(raw, "principal_id"),
        recipient_id: string_field(raw, "recipient_id"),
        resource_id: string_field(raw, "resource_id"),
        resource_type: string_field(raw, "resource_type"),
        operation: string_field(raw, "operation"),
        purpose: string_field(raw, "purpose"),
        policy_epoch: u64_field(raw, "policy_epoch"),
        issued_at: u64_field(raw, "issued_at"),
        expires_at: u64_field(raw, "expires_at"),
        nonce: string_field(raw, "nonce"),
        risk_claim: RiskTier::parse(&string_field(raw, "risk_claim"))
            .unwrap_or_else(|| panic!("bad risk claim in the vector file")),
        approval_digest: optional_string(raw, "approval_digest"),
        budget: match raw.get("budget") {
            None | Some(Value::Null) => None,
            Some(budget) => Some(Budget {
                duration_ms: optional_u64(budget, "duration_ms"),
                operation_count: optional_u64(budget, "operation_count"),
            }),
        },
    }
}

fn context_from(raw: &Value, registry: &OperationRegistry) -> PolicyContext {
    PolicyContext {
        local_node_id: string_field(raw, "local_node_id"),
        current_policy_epoch: u64_field(raw, "current_policy_epoch"),
        now_monotonic: u64_field(raw, "now_monotonic"),
        mode: SystemMode::parse(&string_field(raw, "mode"))
            .unwrap_or_else(|| panic!("bad mode in the vector file")),
        known_principals: string_set(raw, "known_principals"),
        resolved_resources: raw
            .get("resolved_resources")
            .and_then(Value::as_object)
            .map(|map| {
                map.iter()
                    .map(|(key, version)| {
                        (key.clone(), version.as_str().map(str::to_owned).unwrap_or_default())
                    })
                    .collect()
            })
            .unwrap_or_default(),
        operations: registry.clone(),
        audit_state: AuditState::parse(&string_field(raw, "audit_state"))
            .unwrap_or_else(|| panic!("bad audit state in the vector file")),
        action_digest: optional_string(raw, "action_digest"),
        revoked_nonces: string_set(raw, "revoked_nonces"),
        revoked_capabilities: string_set(raw, "revoked_capabilities"),
        strong_approval_granted: raw
            .get("strong_approval_granted")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        satisfied_preconditions: string_set(raw, "satisfied_preconditions"),
    }
}

/// Merge the named fields of `overrides` over `base`.
///
/// Each named field replaces the base value wholesale. There is no nesting
/// and no deep merge, so there is no merge rule for this implementation and
/// the Python one to disagree about.
fn merged(base: &Value, overrides: Option<&Value>) -> Value {
    let mut fields: BTreeMap<String, Value> = match base {
        Value::Object(map) => map.clone(),
        _ => panic!("the vector file's base values must be objects"),
    };
    if let Some(Value::Object(overrides)) = overrides {
        for (key, value) in overrides {
            fields.insert(key.clone(), value.clone());
        }
    }
    Value::Object(fields)
}

fn vectors() -> Value {
    parse(VECTORS_JSON).expect("the vector file is valid canonical JSON")
}

#[test]
fn every_case_matches_the_expected_decision() {
    let document = vectors();
    let base_request = document.get("request").expect("the vector file has a base request");
    let base_capability =
        document.get("capability").expect("the vector file has a base capability");
    let base_context = document.get("context").expect("the vector file has a base context");
    let base_registry =
        document.get("registry").and_then(Value::as_array).expect("the vector file has a registry");

    let cases = document.get("cases").and_then(Value::as_array).expect("the vector file has cases");

    for case in cases {
        let id = string_field(case, "id");
        let registry = match case.get("registry") {
            None => operation_registry(base_registry),
            Some(Value::Array(entries)) => operation_registry(entries),
            Some(_) => panic!("{id}: the registry override must be an array"),
        };

        let request = request_from(&merged(base_request, case.get("request")));
        let capability = match case.get("capability") {
            None => Some(capability_from(base_capability)),
            Some(Value::Null) => None,
            Some(overrides) => Some(capability_from(&merged(base_capability, Some(overrides)))),
        };
        let context = context_from(&merged(base_context, case.get("context")), &registry);

        let decision = evaluate(&request, capability.as_ref(), &context);

        let expected = case.get("expected").expect("the case has an expectation");
        let expected_status = string_field(expected, "status");
        let expected_reason = string_field(expected, "reason");

        assert_eq!(
            decision.status().as_str(),
            expected_status,
            "{id} produced {} / {}",
            decision.status(),
            decision.reason()
        );
        assert_eq!(decision.reason().as_str(), expected_reason, "{id} produced the wrong reason");
        assert!(
            !decision.grants_authority() || expected_status == "PERMIT",
            "{id} granted authority it should not have"
        );
    }
}

#[test]
fn every_attenuation_case_matches() {
    let document = vectors();
    let base_capability =
        document.get("capability").expect("the vector file has a base capability");
    let cases = document
        .get("attenuation_cases")
        .and_then(Value::as_array)
        .expect("the vector file has attenuation cases");

    for case in cases {
        let id = string_field(case, "id");
        let parent = capability_from(&merged(base_capability, case.get("parent")));
        let child = capability_from(&merged(base_capability, case.get("child")));
        let expected = case
            .get("expected")
            .and_then(Value::as_bool)
            .expect("the attenuation case has an expectation");
        assert_eq!(attenuate(&parent, &child), expected, "{id} produced the wrong result");
    }
}

#[test]
fn the_vector_file_covers_the_specification() {
    // The file must carry at least one case per conformance requirement, and
    // every case must have its own identifier. The count is checked here and
    // in the Python suite. That duplication is deliberate: the file is
    // shared, so a case removed from it must fail in both implementations
    // rather than in neither.
    let document = vectors();
    let cases = document.get("cases").and_then(Value::as_array).expect("cases");
    let attenuation =
        document.get("attenuation_cases").and_then(Value::as_array).expect("attenuation cases");
    assert!(cases.len() >= 30, "the vector file has only {} decision cases", cases.len());
    assert!(
        attenuation.len() >= 11,
        "the vector file has only {} attenuation cases",
        attenuation.len()
    );

    let mut identifiers: Vec<String> = cases.iter().map(|case| string_field(case, "id")).collect();
    identifiers.extend(attenuation.iter().map(|case| string_field(case, "id")));
    let mut sorted = identifiers.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), identifiers.len(), "a case identifier repeats");
}

#[test]
fn every_case_names_the_requirement_it_covers() {
    // A case cites either a constitutional invariant (INV-nn) or the
    // conformance test identifier it exercises (SCP-nnn and the rest).
    let document = vectors();
    let cases = document.get("cases").and_then(Value::as_array).expect("cases");
    for case in cases {
        let id = string_field(case, "id");
        let requirement = string_field(case, "requirement");
        let prefix = requirement.split('-').next().unwrap_or_default();
        assert!(
            ["INV", "SCP", "CAP", "CC", "OC", "AI", "INS", "UPD", "PRIV", "FED", "EVID"]
                .contains(&prefix),
            "{id} names a requirement that is not a known identifier: {requirement}"
        );
        let gate = case
            .get("gate")
            .and_then(Value::as_int)
            .unwrap_or_else(|| panic!("{id} names no gate"));
        assert!((1..=16).contains(&gate), "{id} names a gate outside the sixteen");
        let expected = case.get("expected").expect("the case has an expectation");
        let status = string_field(expected, "status");
        assert!(
            ["PERMIT", "DENY", "CHALLENGE", "DEFER", "QUARANTINE"].contains(&status.as_str()),
            "{id} expects a status that is not in the decision vocabulary"
        );
        let reason = string_field(expected, "reason");
        assert!(
            ReasonCode::ALL.iter().any(|code| code.as_str() == reason),
            "{id} expects a reason code that does not exist: {reason}"
        );
    }
}

#[test]
fn a_permitted_decision_is_scoped_to_its_capability() {
    let document = vectors();
    let base_request = document.get("request").expect("the vector file has a base request");
    let base_capability =
        document.get("capability").expect("the vector file has a base capability");
    let base_context = document.get("context").expect("the vector file has a base context");
    let base_registry =
        document.get("registry").and_then(Value::as_array).expect("the vector file has a registry");
    let registry = operation_registry(base_registry);
    let request = request_from(base_request);
    let capability = capability_from(base_capability);
    let context = context_from(base_context, &registry);

    let decision = evaluate(&request, Some(&capability), &context);
    assert_eq!(decision.status(), DecisionStatus::Permit);
    assert_eq!(decision.capability_id(), Some("cap:example:0001"));
    assert!(decision.grants_authority());
}

#[test]
fn a_denied_decision_grants_nothing() {
    let document = vectors();
    let base_request = document.get("request").expect("the vector file has a base request");
    let base_capability =
        document.get("capability").expect("the vector file has a base capability");
    let base_context = document.get("context").expect("the vector file has a base context");
    let base_registry =
        document.get("registry").and_then(Value::as_array).expect("the vector file has a registry");
    let registry = operation_registry(base_registry);
    let mut request = request_from(base_request);
    request.principal_id = "user:mallory".to_owned();
    let capability = capability_from(base_capability);
    let context = context_from(base_context, &registry);

    let decision = evaluate(&request, Some(&capability), &context);
    assert_eq!(decision.status(), DecisionStatus::Deny);
    assert_eq!(decision.reason(), ReasonCode::UNKNOWN_PRINCIPAL);
    assert_eq!(decision.capability_id(), None);
    assert!(!decision.grants_authority());
}

#[test]
fn a_challenge_grants_no_partial_authority() {
    let document = vectors();
    let base_request = document.get("request").expect("the vector file has a base request");
    let base_capability =
        document.get("capability").expect("the vector file has a base capability");
    let base_context = document.get("context").expect("the vector file has a base context");
    let base_registry =
        document.get("registry").and_then(Value::as_array).expect("the vector file has a registry");
    let registry = operation_registry(base_registry);
    let mut request = request_from(base_request);
    request.operation = "delete".to_owned();
    request.risk_claim = RiskTier::R3;
    request.approval_digest = Some("digest-aaaa".to_owned());
    let mut capability = capability_from(base_capability);
    capability.operations = ["delete".to_owned()].into_iter().collect();
    let mut context = context_from(base_context, &registry);
    context.action_digest = Some("digest-aaaa".to_owned());

    let decision = evaluate(&request, Some(&capability), &context);
    assert_eq!(decision.status(), DecisionStatus::Challenge);
    assert_eq!(decision.reason(), ReasonCode::STRONG_APPROVAL_REQUIRED);
    assert_eq!(decision.capability_id(), None);
    assert!(!decision.grants_authority());
}

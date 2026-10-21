//! The capsule conformance vectors.
//!
//! The vector file is the contract: it says what the executor must do, in
//! data, and the test below checks the executor against it. A change to the
//! executor that changes an outcome has to change the file as well, which is
//! what makes the change reviewable.
//!
//! The vectors are agreed by two implementations: this one and the Python
//! reference model in `reference/capsule_reference.py`. Both read the same
//! file and both must reach the state it names, so the file is a contract
//! rather than a transcript. CI runs the two suites together and fails on a
//! disagreement.

use std::collections::{BTreeMap, BTreeSet};

use senvoryn_canonical::{Value, parse};
use senvoryn_capsule::{
    Approval, Authorization, ChangeCapsule, Environment, Executor, Fault, FaultInjector,
    FaultPoint, State,
};
use senvoryn_scp::{
    Decision, DecisionRecord, DecisionStatus, Deferral, OperationRegistry, ReasonCode,
    RecordDetails, ResolvedResource,
};

const VECTORS_JSON: &str = include_str!("../../../conformance/capsule-vectors.json");
const BASE_CAPSULE_JSON: &str = include_str!("../../../examples/capsule.sample.json");

/// The digest the base capsule carries.
const BASE_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000002";

/// A digest that is not the base capsule's.
const OTHER_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000009";

/// The version the base capsule expects its target to be at.
const BASE_VERSION: &str = "7";

/// A version the base capsule does not expect.
const OTHER_VERSION: &str = "6";

/// Sets a value at a dotted path, where a numeric part is an array index.
fn set_path(root: &mut Value, path: &str, new_value: Value) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = root;
    for part in &parts[..parts.len() - 1] {
        current = match current {
            Value::Object(map) => {
                map.get_mut(*part).unwrap_or_else(|| panic!("the capsule has no {part:?}"))
            }
            Value::Array(items) => {
                let index: usize =
                    part.parse().unwrap_or_else(|_| panic!("{part:?} is not an array index"));
                &mut items[index]
            }
            other => panic!("cannot descend into a {}", other.type_name()),
        };
    }
    let last = parts[parts.len() - 1];
    match current {
        Value::Object(map) => {
            map.insert(last.to_owned(), new_value);
        }
        Value::Array(items) => {
            let index: usize =
                last.parse().unwrap_or_else(|_| panic!("{last:?} is not an array index"));
            items[index] = new_value;
        }
        other => panic!("cannot set {last:?} in a {}", other.type_name()),
    }
}

/// Reads a fault point's name.
fn fault_point(name: &str, index: Option<i64>) -> FaultPoint {
    match name {
        "BEFORE_VALIDATION" => FaultPoint::BeforeValidation,
        "BEFORE_AUTHORIZATION" => FaultPoint::BeforeAuthorization,
        "BEFORE_STAGING" => FaultPoint::BeforeStaging,
        "AFTER_STAGING" => FaultPoint::AfterStaging,
        "BEFORE_PRECHECK" => FaultPoint::BeforePrecheck,
        "BEFORE_STEP" => FaultPoint::BeforeStep { index: index.unwrap_or(0) as usize },
        "DURING_STEP" => FaultPoint::DuringStep { index: index.unwrap_or(0) as usize },
        "AFTER_STEP" => FaultPoint::AfterStep { index: index.unwrap_or(0) as usize },
        "BEFORE_VERIFICATION" => FaultPoint::BeforeVerification,
        "DURING_VERIFICATION" => FaultPoint::DuringVerification,
        other => panic!("{other:?} is not a fault point"),
    }
}

/// Reads a fault's name.
fn fault(name: &str) -> Fault {
    match name {
        "IO" => Fault::Io,
        "CRASH" => Fault::Crash,
        "STALE_PRECONDITION" => Fault::StalePrecondition,
        "POLICY_EPOCH_CHANGED" => Fault::PolicyEpochChanged,
        "AUDIT_SINK_LOST" => Fault::AuditSinkLost,
        "DISK_FULL" => Fault::DiskFull,
        "CLOCK_ROLLBACK" => Fault::ClockRollback,
        "CORRUPTED_RECOVERY_ARTIFACT" => Fault::CorruptedRecoveryArtifact,
        other => panic!("{other:?} is not a fault"),
    }
}

/// The environment the base capsule runs in.
fn base_environment() -> Environment {
    let mut satisfied_preconditions = BTreeSet::new();
    satisfied_preconditions.insert("resource_version_matches".to_owned());
    let mut observed_versions = BTreeMap::new();
    observed_versions.insert("resource_version_matches".to_owned(), "7".to_owned());
    let mut satisfied_postconditions = BTreeSet::new();
    satisfied_postconditions.insert("write_accepted".to_owned());
    satisfied_postconditions.insert("written_version_verified".to_owned());

    Environment {
        current_version: "7".to_owned(),
        policy_epoch: 7,
        now_ms: 1_767_225_600_000,
        satisfied_preconditions,
        observed_versions,
        satisfied_postconditions,
        staged: false,
        applied_steps: Vec::new(),
        compensated_steps: Vec::new(),
        outstanding_effects: BTreeSet::new(),
        recovery_material_present: true,
        recovery_material_intact: true,
        audit_sink_durable: true,
    }
}

/// Applies the environment overrides of a case.
fn apply_environment(environment: &mut Environment, overrides: Option<&Value>) {
    let Some(overrides) = overrides.and_then(Value::as_object) else {
        return;
    };
    for (key, value) in overrides {
        match key.as_str() {
            "current_version" => {
                environment.current_version =
                    value.as_str().expect("current_version is a string").to_owned();
            }
            "policy_epoch" => {
                environment.policy_epoch =
                    value.as_int().expect("policy_epoch is an integer") as u64;
            }
            "now_ms" => {
                environment.now_ms = value.as_int().expect("now_ms is an integer");
            }
            "recovery_material_present" => {
                environment.recovery_material_present =
                    value.as_bool().expect("recovery_material_present is a boolean");
            }
            "recovery_material_intact" => {
                environment.recovery_material_intact =
                    value.as_bool().expect("recovery_material_intact is a boolean");
            }
            "audit_sink_durable" => {
                environment.audit_sink_durable =
                    value.as_bool().expect("audit_sink_durable is a boolean");
            }
            "satisfied_preconditions" => {
                environment.satisfied_preconditions = value
                    .as_array()
                    .expect("satisfied_preconditions is an array")
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect();
            }
            "satisfied_postconditions" => {
                environment.satisfied_postconditions = value
                    .as_array()
                    .expect("satisfied_postconditions is an array")
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect();
            }
            other => panic!("{other:?} is not an environment field"),
        }
    }
}

/// Builds the decision record a case is decided under.
///
/// The record is written by the policy layer's own writer and read back
/// through its own reader, so every case exercises both ends of the boundary
/// and the two cannot drift apart: a field the writer stops writing is a
/// field the reader stops finding, and the case that expects a commit fails.
///
/// The executor is handed the authorization it reads out of that record —
/// not a value the harness assembles — which is the point: the plan and the
/// version the executor checks come from the decision.
fn build_decision_record(specification: Option<&Value>) -> DecisionRecord {
    let permitted_digest = match specification
        .and_then(|specification| specification.get("digest"))
        .and_then(Value::as_str)
    {
        Some("matching") => Some(BASE_DIGEST.to_owned()),
        Some("other") => Some(OTHER_DIGEST.to_owned()),
        Some("none") => None,
        None => Some(BASE_DIGEST.to_owned()),
        other => panic!("{other:?} is not a digest selector"),
    };
    let permitted_resource_version = match specification
        .and_then(|specification| specification.get("version"))
        .and_then(Value::as_str)
    {
        Some("matching") => Some(BASE_VERSION.to_owned()),
        Some("other") => Some(OTHER_VERSION.to_owned()),
        Some("none") => None,
        None => Some(BASE_VERSION.to_owned()),
        other => panic!("{other:?} is not a version selector"),
    };
    let status = match specification
        .and_then(|specification| specification.get("status"))
        .and_then(Value::as_str)
    {
        Some("PERMIT") => DecisionStatus::Permit,
        Some("DENY") => DecisionStatus::Deny,
        Some("CHALLENGE") => DecisionStatus::Challenge,
        Some("DEFER") => DecisionStatus::Defer,
        Some("QUARANTINE") => DecisionStatus::Quarantine,
        None => DecisionStatus::Permit,
        other => panic!("{other:?} is not a decision status"),
    };

    let decision = match status {
        DecisionStatus::Permit => {
            Decision::permit(ReasonCode::ALL_POLICY_GATES_PASSED, "capability-0001".to_owned(), 7)
        }
        DecisionStatus::Deny => Decision::deny(ReasonCode::UNKNOWN_OPERATION_RISK, 7),
        DecisionStatus::Challenge => Decision::challenge(ReasonCode::UNKNOWN_OPERATION_RISK, 7),
        // A deferral is a real status, and the record has to carry it. A
        // harness that wrote a denial here would make a case about a
        // deferred decision pass for the wrong reason — the executor would
        // refuse a DENY, and the case would say nothing about DEFER.
        DecisionStatus::Defer => {
            Decision::defer(ReasonCode::POLICY_EPOCH_NOT_YET_IN_FORCE, Deferral::PolicyEpoch(8), 7)
        }
        // No gate produces a quarantine, so there is no decision to write.
        // The harness refuses rather than writing a denial under this name:
        // a case that asked for a quarantine and silently received a denial
        // would prove nothing about either.
        DecisionStatus::Quarantine => {
            panic!("no decision produces a quarantine, so none can be written")
        }
    };
    let decision = match permitted_resource_version {
        Some(version) => decision.with_resolved(ResolvedResource {
            canonical_id: "file:///srv/data/report.csv".to_owned(),
            resource_type: "file_object".to_owned(),
            version,
        }),
        None => decision,
    };

    let record = decision.to_record(&RecordDetails {
        decision_id: "decision:vector:0001",
        request_id: "request:vector:0001",
        evaluated_at: "2026-01-01T00:00:00Z",
        presented_resource_id: "file:///srv/data/report.csv",
        permitted_payload_digest: permitted_digest.as_deref(),
        audit_ref: "audit:vector:0001",
        valid_until: None,
    });
    DecisionRecord::from_value(&record)
        .unwrap_or_else(|error| panic!("the decision record is malformed: {error}"))
}

/// Builds the authorization a case is executed under.
///
/// A case that says nothing about the decision is judged under the decision
/// the sample capsule needs: the plan, the version and the one approval the
/// sample capsule requires. A case that says something about the decision
/// says everything about it, because a case that changed the approval while
/// testing the digest would pass for the wrong reason.
fn build_authorization(specification: Option<&Value>) -> Authorization {
    let record = build_decision_record(specification);
    let approvals = match specification
        .and_then(|specification| specification.get("approvals"))
        .and_then(Value::as_array)
    {
        Some(entries) => entries
            .iter()
            .map(|entry| Approval {
                approval_type: entry
                    .get("type")
                    .and_then(Value::as_str)
                    .expect("an approval has a type")
                    .to_owned(),
                binds_digest: match entry.get("binds").and_then(Value::as_str) {
                    Some("capsule") => BASE_DIGEST.to_owned(),
                    Some("other") => OTHER_DIGEST.to_owned(),
                    other => panic!("{other:?} is not a binding selector"),
                },
            })
            .collect(),
        None => vec![Approval {
            approval_type: "USER_CONFIRMATION".to_owned(),
            binds_digest: BASE_DIGEST.to_owned(),
        }],
    };
    Authorization::from_record(&record, approvals)
}

#[test]
fn every_vector_reaches_the_state_it_names() {
    let document = parse(VECTORS_JSON).expect("the vector file is canonical");
    let cases = document.get("cases").and_then(Value::as_array).expect("the vector file has cases");
    assert!(cases.len() >= 20, "the vector file is smaller than expected");

    let registry = OperationRegistry::sample().expect("the sample registry is well formed");
    let mut identifiers: Vec<&str> = Vec::new();

    for case in cases {
        let identifier = case.get("id").and_then(Value::as_str).expect("every case has an id");
        identifiers.push(identifier);

        let mut capsule_document = parse(BASE_CAPSULE_JSON).expect("the base capsule is canonical");
        if let Some(overrides) = case.get("capsule_overrides").and_then(Value::as_object) {
            for (path, value) in overrides {
                set_path(&mut capsule_document, path, value.clone());
            }
        }
        let capsule = ChangeCapsule::from_value(&capsule_document)
            .unwrap_or_else(|error| panic!("{identifier}: the capsule is malformed: {error}"));

        let mut environment = base_environment();
        apply_environment(&mut environment, case.get("environment_overrides"));

        let authorization = build_authorization(case.get("decision"));

        let mut executor = Executor::new(capsule, registry.clone(), environment)
            .unwrap_or_else(|error| panic!("{identifier}: the executor does not build: {error}"));
        if let Some(specification) = case.get("fault").and_then(Value::as_object) {
            let point = fault_point(
                specification.get("point").and_then(Value::as_str).expect("a fault has a point"),
                specification.get("index").and_then(Value::as_int),
            );
            let kind = fault(
                specification.get("fault").and_then(Value::as_str).expect("a fault has a kind"),
            );
            executor = executor.with_faults(FaultInjector::at(point, kind));
        }

        let outcome = executor.run(&authorization);
        let expected_state =
            case.get("expected_state").and_then(Value::as_str).expect("every case names a state");
        assert_eq!(
            outcome.final_state.as_str(),
            expected_state,
            "{identifier}: {} (the case says: {})",
            outcome.reason,
            case.get("control").and_then(Value::as_str).unwrap_or_default()
        );
        if let Some(expected_reason) = case.get("expected_reason_contains").and_then(Value::as_str)
        {
            assert!(
                outcome.reason.contains(expected_reason),
                "{identifier}: the reason {:?} does not mention {expected_reason:?}",
                outcome.reason
            );
        }
    }

    let mut unique = identifiers.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), identifiers.len(), "two vectors share an id");
}

#[test]
fn no_vector_reports_a_commit_it_did_not_earn() {
    // The property the whole gate exists for, stated once more over the
    // vectors: only the case that expects a commit may report one.
    let document = parse(VECTORS_JSON).expect("the vector file is canonical");
    let cases = document.get("cases").and_then(Value::as_array).expect("the vector file has cases");
    let registry = OperationRegistry::sample().expect("the sample registry is well formed");

    for case in cases {
        let identifier = case.get("id").and_then(Value::as_str).expect("every case has an id");
        let expected_state =
            case.get("expected_state").and_then(Value::as_str).expect("every case names a state");
        let mut capsule_document = parse(BASE_CAPSULE_JSON).expect("the base capsule is canonical");
        if let Some(overrides) = case.get("capsule_overrides").and_then(Value::as_object) {
            for (path, value) in overrides {
                set_path(&mut capsule_document, path, value.clone());
            }
        }
        let capsule =
            ChangeCapsule::from_value(&capsule_document).expect("the capsule is well formed");
        let mut environment = base_environment();
        apply_environment(&mut environment, case.get("environment_overrides"));
        let authorization = build_authorization(case.get("decision"));
        let mut executor =
            Executor::new(capsule, registry.clone(), environment).expect("the executor builds");
        if let Some(specification) = case.get("fault").and_then(Value::as_object) {
            let point = fault_point(
                specification.get("point").and_then(Value::as_str).expect("a fault has a point"),
                specification.get("index").and_then(Value::as_int),
            );
            let kind = fault(
                specification.get("fault").and_then(Value::as_str).expect("a fault has a kind"),
            );
            executor = executor.with_faults(FaultInjector::at(point, kind));
        }
        let outcome = executor.run(&authorization);
        if expected_state != "COMMITTED" {
            assert_ne!(
                outcome.final_state,
                State::Committed,
                "{identifier}: the case does not expect a commit and the transaction reported one"
            );
        }
    }
}

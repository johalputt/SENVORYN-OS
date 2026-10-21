//! A development tool for exercising the SCP decision oracle.
//!
//! **This is not an SCP and must not be deployed as one.** It holds no
//! trust store, verifies no signature, resolves no resource against a
//! filesystem, and writes no audit record. It exists so that the decision
//! function can be driven by hand, and so that the conformance vectors can
//! be replayed by someone who does not want to read the test harness.
//!
//! It reads three documents, evaluates one request against one context, and
//! writes the decision record to standard output as canonical JSON.

use std::collections::BTreeSet;
use std::process::ExitCode;

use senvoryn_canonical::{Value, encode_to_string, parse};
use senvoryn_instant::parse_utc_millis;
use senvoryn_scp::{
    AuditState, Capability, ClockState, OperationEntry, OperationRegistry, PolicyContext,
    Provenance, RecordDetails, Request, SystemMode, evaluate,
};

const USAGE: &str = "\
senvoryn-scp — evaluate one request against one policy context

NOT AN SCP. This is a development tool for the decision oracle in the
senvoryn-scp crate. It verifies no signature, resolves no resource against a
filesystem, and writes no audit record. It must not be deployed as a System
Control Plane.

USAGE:
    senvoryn-scp --request <file> --context <file> [--capability <file>]

OPTIONS:
    --request <file>      The request envelope to evaluate.
    --context <file>      The policy context, including the operation registry.
    --capability <file>   The capability being presented. Optional.
    --help                Print this message.

The input is parsed with the Senvoryn Canonical JSON Profile's parser, so the
grammar is strict even though the layout is not: whitespace and key order
are accepted on the way in and normalised on the way out. The decision record
is written to standard output in canonical form.
";

fn main() -> ExitCode {
    match run() {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("senvoryn-scp: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.iter().any(|argument| argument == "--help" || argument == "-h") {
        return Ok(USAGE.trim_end().to_owned());
    }
    if arguments.is_empty() {
        return Err(format!("no arguments given\n\n{USAGE}"));
    }

    let mut request_path: Option<String> = None;
    let mut context_path: Option<String> = None;
    let mut capability_path: Option<String> = None;
    let mut index = 0;
    while index < arguments.len() {
        let flag = arguments[index].as_str();
        let value = arguments.get(index + 1).ok_or_else(|| format!("{flag} needs a file name"))?;
        match flag {
            "--request" => request_path = Some(value.clone()),
            "--context" => context_path = Some(value.clone()),
            "--capability" => capability_path = Some(value.clone()),
            other => return Err(format!("unknown argument {other:?}\n\n{USAGE}")),
        }
        index += 2;
    }

    let request_path = request_path.ok_or_else(|| format!("--request is required\n\n{USAGE}"))?;
    let context_path = context_path.ok_or_else(|| format!("--context is required\n\n{USAGE}"))?;

    let request = Request::from_value(&read_document(&request_path)?)
        .map_err(|error| format!("the request envelope is not usable: {error}"))?;
    let context = read_context(&read_document(&context_path)?)
        .map_err(|error| format!("the policy context is not usable: {error}"))?;
    let capability = match capability_path {
        Some(path) => Some(
            read_capability(&read_document(&path)?)
                .map_err(|error| format!("the capability is not usable: {error}"))?,
        ),
        None => None,
    };

    let decision = evaluate(&request, capability.as_ref(), &context);
    let record = decision.to_record(&RecordDetails {
        decision_id: "decision:cli:0001",
        request_id: &request.request_id,
        evaluated_at: "1970-01-01T00:00:00Z",
        target_resource_id: &request.resource_id,
        audit_ref: "audit:cli:0001",
        valid_until: None,
        required_next_step: None,
    });
    Ok(encode_to_string(&record))
}

fn read_document(path: &str) -> Result<Value, String> {
    let text =
        std::fs::read_to_string(path).map_err(|error| format!("could not read {path}: {error}"))?;
    parse(&text)
        .map_err(|error| format!("{path} is not valid under the canonical profile: {error}"))
}

fn read_context(document: &Value) -> Result<PolicyContext, String> {
    Ok(PolicyContext {
        local_node_id: required_string(document, "local_node_id")?,
        current_policy_epoch: required_u64(document, "current_policy_epoch")?,
        now_monotonic: required_u64(document, "now_monotonic")?,
        now_wall_ms: required_i64(document, "now_wall_ms")?,
        clock_state: ClockState::parse(&required_string(document, "clock_state")?)
            .ok_or_else(|| "the context names a clock state that does not exist".to_owned())?,
        mode: SystemMode::parse(&required_string(document, "mode")?)
            .ok_or_else(|| "the context names a system mode that does not exist".to_owned())?,
        known_principals: string_set(document, "known_principals"),
        resolved_resources: document
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
        operations: read_registry(document)?,
        audit_state: AuditState::parse(&required_string(document, "audit_state")?)
            .ok_or_else(|| "the context names an audit state that does not exist".to_owned())?,
        action_digest: optional_string(document, "action_digest"),
        revoked_nonces: string_set(document, "revoked_nonces"),
        revoked_capabilities: string_set(document, "revoked_capabilities"),
        strong_approval_granted: document
            .get("strong_approval_granted")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        satisfied_preconditions: string_set(document, "satisfied_preconditions"),
    })
}

fn read_registry(document: &Value) -> Result<OperationRegistry, String> {
    let epoch = required_u64(document, "registry_epoch")?;
    let valid_from_ms = required_instant(document, "issued_at")?;
    let valid_until_ms = required_instant(document, "expires_at")?;
    let operations = document
        .get("operations")
        .and_then(Value::as_array)
        .ok_or_else(|| "the context has no \"operations\" array".to_owned())?;
    let mut entries = Vec::with_capacity(operations.len());
    for operation in operations {
        entries.push(OperationEntry {
            operation_id: required_string(operation, "operation_id")?,
            risk_tier: senvoryn_scp::RiskTier::parse(&required_string(operation, "risk_tier")?)
                .ok_or_else(|| {
                    "an operation entry names a risk tier that does not exist".to_owned()
                })?,
            approval_requirements: string_set(operation, "approval_requirements"),
            allowed_modes: string_set(operation, "allowed_modes"),
            resource_types: string_set(operation, "resource_types"),
            max_duration_ms: optional_u64(operation, "max_duration_ms"),
            max_operation_count: optional_u64(operation, "max_operation_count"),
            required_preconditions: string_set(operation, "required_preconditions"),
            required_postconditions: string_set(operation, "required_postconditions"),
            reversibility: operation
                .get("reversibility")
                .and_then(Value::as_str)
                .and_then(senvoryn_scp::Reversibility::parse)
                .unwrap_or(senvoryn_scp::Reversibility::Unknown),
            external_effects: string_set(operation, "external_effects"),
            idempotent: operation.get("idempotent").and_then(Value::as_bool).unwrap_or(false),
            audit_class: operation
                .get("audit_class")
                .and_then(Value::as_str)
                .and_then(senvoryn_scp::AuditClass::parse)
                .unwrap_or(senvoryn_scp::AuditClass::Standard),
        });
    }
    Ok(OperationRegistry::from_entries(entries, epoch, valid_from_ms, valid_until_ms))
}

fn read_capability(document: &Value) -> Result<Capability, String> {
    Ok(Capability {
        capability_id: required_string(document, "capability_id")?,
        subject_id: required_string(document, "subject_id")?,
        resource_id: required_string(document, "resource_id")?,
        resource_type: required_string(document, "resource_type")?,
        operations: string_set(document, "operations"),
        purposes: string_set(document, "purposes"),
        not_before: required_u64(document, "not_before")?,
        expires_at: required_u64(document, "expires_at")?,
        policy_epoch: required_u64(document, "policy_epoch")?,
        provenance: Provenance::parse(&required_string(document, "provenance")?)
            .ok_or_else(|| "the capability names a provenance that does not exist".to_owned())?,
        delegation_depth: document
            .get("delegation_depth")
            .and_then(Value::as_int)
            .and_then(|number| u32::try_from(number).ok())
            .unwrap_or(0),
        max_uses: optional_u64(document, "max_uses"),
    })
}

fn required_string(document: &Value, field: &str) -> Result<String, String> {
    document
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("the field {field:?} is missing or is not a string"))
}

fn optional_string(document: &Value, field: &str) -> Option<String> {
    match document.get(field) {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => Some(text.clone()),
        Some(_) => None,
    }
}

fn required_instant(document: &Value, field: &str) -> Result<i64, String> {
    let text = required_string(document, field)?;
    parse_utc_millis(&text)
        .map_err(|detail| format!("the field {field:?} is not a strict UTC timestamp: {detail}"))
}

fn required_i64(document: &Value, field: &str) -> Result<i64, String> {
    document
        .get(field)
        .and_then(Value::as_int)
        .ok_or_else(|| format!("the field {field:?} is missing or is not an integer"))
}

fn required_u64(document: &Value, field: &str) -> Result<u64, String> {
    let number = document
        .get(field)
        .and_then(Value::as_int)
        .ok_or_else(|| format!("the field {field:?} is missing or is not an integer"))?;
    u64::try_from(number).map_err(|_| format!("the field {field:?} is negative"))
}

fn optional_u64(document: &Value, field: &str) -> Option<u64> {
    match document.get(field) {
        None | Some(Value::Null) => None,
        Some(value) => value.as_int().and_then(|number| u64::try_from(number).ok()),
    }
}

fn string_set(document: &Value, field: &str) -> BTreeSet<String> {
    document
        .get(field)
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).map(str::to_owned).collect())
        .unwrap_or_default()
}

//! A deterministic fuzz harness for the policy request reader.
//!
//! The reader is the boundary where a document becomes a request the gates
//! will judge, and a gate cannot judge a request that was read wrongly. This
//! harness mutates a corpus of policy requests in a reproducible way and
//! asserts three things: the reader never panics, a request that was read
//! says exactly what the document said, and the same document is read the
//! same way twice.
//!
//! It is a plain test rather than `cargo-fuzz` for the reason the canonical
//! parser's harness gives: the project has no third-party dependencies and no
//! nightly toolchain, and a fuzz target that cannot run in CI is a fuzz
//! target nobody runs.

use std::collections::BTreeMap;

use senvoryn_canonical::{Value, parse};
use senvoryn_scp::{Budget, Request, RiskTier};

/// The corpus. Every entry is a shape the reader has to survive.
const SEEDS: &[&str] = &[
    // The smallest request the form allows.
    r#"{"schema":"senvoryn.request-envelope.v1","request_id":"req:example:0001","principal_id":"user:alice","recipient_id":"node:local","resource_id":"file:report","resource_type":"file_object","operation":"read","purpose":"user-request","policy_epoch":7,"issued_at":100,"expires_at":900,"nonce":"nonce-example-0001","risk_claim":"R0"}"#,
    // A request with every optional field present.
    r#"{"schema":"senvoryn.request-envelope.v1","request_id":"req:example:0002","principal_id":"service:indexer","recipient_id":"node:local","resource_id":"device:disk0","resource_type":"device","operation":"read.sectors","purpose":"indexing","policy_epoch":7,"issued_at":100,"expires_at":900,"nonce":"nonce-example-0002","risk_claim":"R2","approval_digest":"sha256:approval","budget":{"duration_ms":500,"operation_count":3}}"#,
    // A request whose optional fields are present but empty.
    r#"{"schema":"senvoryn.request-envelope.v1","request_id":"req:example:0003","principal_id":"user:alice","recipient_id":"node:local","resource_id":"file:report","resource_type":"file_object","operation":"read","purpose":"user-request","policy_epoch":7,"issued_at":100,"expires_at":900,"nonce":"nonce-example-0003","risk_claim":"R1","approval_digest":null,"budget":null}"#,
    // A request with a field the form does not define.
    r#"{"schema":"senvoryn.request-envelope.v1","request_id":"req:example:0004","principal_id":"user:alice","recipient_id":"node:local","resource_id":"file:report","resource_type":"file_object","operation":"read","purpose":"user-request","policy_epoch":7,"issued_at":100,"expires_at":900,"nonce":"nonce-example-0004","risk_claim":"R1","unexpected_privilege":"root"}"#,
    // A request that declares a schema this node does not support.
    r#"{"schema":"senvoryn.request-envelope.v2","request_id":"req:example:0005","principal_id":"user:alice","recipient_id":"node:local","resource_id":"file:report","resource_type":"file_object","operation":"read","purpose":"user-request","policy_epoch":7,"issued_at":100,"expires_at":900,"nonce":"nonce-example-0005","risk_claim":"R1"}"#,
    // A document that is not a request at all.
    r#"[{"schema":"senvoryn.request-envelope.v1"}]"#,
];

/// A small deterministic generator, seeded differently per case so that the
/// whole corpus is covered.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }
}

/// Applies one deterministic mutation.
fn mutate(rng: &mut Rng, value: &Value) -> Value {
    let object = value.as_object();
    let field = match object {
        Some(object) if !object.is_empty() => {
            let index = rng.below(object.len());
            object.keys().nth(index).cloned()
        }
        _ => None,
    };
    match rng.below(8) {
        // Replace the whole document with something that is not an object.
        0 => match rng.below(5) {
            0 => Value::Null,
            1 => Value::Bool(rng.below(2) == 1),
            2 => Value::Int(rng.next_u64() as i64),
            3 => Value::String("not a request".to_owned()),
            _ => Value::Array(vec![Value::Int(1), Value::Null]),
        },
        // Drop a field.
        1 => match (object, field) {
            (Some(object), Some(field)) => {
                let mut map = object.clone();
                map.remove(&field);
                Value::Object(map)
            }
            _ => value.clone(),
        },
        // Give a field a value of the wrong type.
        2 => match (object, field) {
            (Some(object), Some(field)) => {
                let mut map = object.clone();
                let replacement = match object.get(&field) {
                    Some(Value::String(_)) => Value::Int(1),
                    Some(Value::Int(_)) => Value::String("7".to_owned()),
                    Some(Value::Bool(_)) => Value::Null,
                    _ => Value::String(String::new()),
                };
                map.insert(field, replacement);
                Value::Object(map)
            }
            _ => value.clone(),
        },
        // Null a field.
        3 => match (object, field) {
            (Some(object), Some(field)) => {
                let mut map = object.clone();
                map.insert(field, Value::Null);
                Value::Object(map)
            }
            _ => value.clone(),
        },
        // Empty a string field.
        4 => match (object, field) {
            (Some(object), Some(field)) => {
                let mut map = object.clone();
                if matches!(object.get(&field), Some(Value::String(_))) {
                    map.insert(field, Value::String(String::new()));
                }
                Value::Object(map)
            }
            _ => value.clone(),
        },
        // Negate a number field.
        5 => match (object, field) {
            (Some(object), Some(field)) => {
                let mut map = object.clone();
                if let Some(Value::Int(number)) = object.get(&field) {
                    map.insert(field, Value::Int(number.checked_neg().unwrap_or(i64::MAX)));
                }
                Value::Object(map)
            }
            _ => value.clone(),
        },
        // Add a field the form does not define.
        6 => match object {
            Some(object) => {
                let mut map = object.clone();
                map.insert(format!("unexpected_{}", rng.below(4)), Value::Int(1));
                Value::Object(map)
            }
            None => value.clone(),
        },
        // Change the budget, which is the one nested object in the form.
        _ => match object.and_then(|object| object.get("budget")).cloned() {
            Some(Value::Object(inner)) => {
                let mut map = object.expect("the budget lives in an object").clone();
                let mut budget = inner.clone();
                if !budget.is_empty() && rng.below(2) == 0 {
                    let index = rng.below(budget.len());
                    let key = budget.keys().nth(index).cloned().expect("the key exists");
                    budget.remove(&key);
                } else {
                    budget.insert(format!("cpu_{}", rng.below(4)), Value::Int(1));
                }
                map.insert("budget".to_owned(), Value::Object(budget));
                Value::Object(map)
            }
            _ => value.clone(),
        },
    }
}

#[test]
fn the_reader_never_panics_and_never_lies() {
    let mut cases = 0u32;
    for (seed_index, seed) in SEEDS.iter().enumerate() {
        let document = parse(seed).expect("a seed is canonical");
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ ((seed_index as u64 + 1) * 0x100_0000_01B3));
        for _ in 0..4_000 {
            let mut mutated = document.clone();
            for _ in 0..rng.below(4) {
                mutated = mutate(&mut rng, &mutated);
            }
            cases += 1;
            check(&mutated);
        }
    }
    assert!(cases >= 20_000, "the harness ran only {cases} cases");
}

fn check(document: &Value) {
    let result = Request::from_value(document);
    // The reader is a pure function: the same document is read the same way.
    assert_eq!(result, Request::from_value(document), "the reader read the same document two ways");
    let Ok(request) = result else {
        return;
    };
    // A request that was read must say what the document said. The reader is
    // the place where a document becomes a request, and a reader that
    // changes a value while reading it is a reader whose decision is not the
    // caller's decision.
    let object =
        document.as_object().expect("the reader accepted a document that is not an object");
    assert_eq!(request.request_id, string(object, "request_id"));
    assert_eq!(request.principal_id, string(object, "principal_id"));
    assert_eq!(request.recipient_id, string(object, "recipient_id"));
    assert_eq!(request.resource_id, string(object, "resource_id"));
    assert_eq!(request.resource_type, string(object, "resource_type"));
    assert_eq!(request.operation, string(object, "operation"));
    assert_eq!(request.purpose, string(object, "purpose"));
    assert_eq!(request.nonce, string(object, "nonce"));
    assert_eq!(request.policy_epoch, number(object, "policy_epoch"));
    assert_eq!(request.issued_at, number(object, "issued_at"));
    assert_eq!(request.expires_at, number(object, "expires_at"));
    assert_eq!(request.risk_claim, tier(object));
    assert_eq!(request.approval_digest, optional_string(object, "approval_digest"));
    assert_eq!(request.budget, optional_budget(object));
}

fn string(object: &BTreeMap<String, Value>, field: &str) -> String {
    object
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("the reader accepted a document whose {field:?} is not a string"))
        .to_owned()
}

fn number(object: &BTreeMap<String, Value>, field: &str) -> u64 {
    let value = object.get(field).and_then(Value::as_int).unwrap_or_else(|| {
        panic!("the reader accepted a document whose {field:?} is not an integer")
    });
    u64::try_from(value).unwrap_or_else(|_| panic!("the reader accepted a negative {field:?}"))
}

fn optional_string(object: &BTreeMap<String, Value>, field: &str) -> Option<String> {
    match object.get(field) {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => Some(text.clone()),
        Some(other) => {
            panic!("the reader accepted a document whose {field:?} is {}", other.type_name())
        }
    }
}

fn optional_budget(object: &BTreeMap<String, Value>) -> Option<Budget> {
    match object.get("budget") {
        None | Some(Value::Null) => None,
        Some(Value::Object(inner)) => Some(Budget {
            duration_ms: optional_number(inner, "duration_ms"),
            operation_count: optional_number(inner, "operation_count"),
        }),
        Some(other) => panic!("the reader accepted a budget that is {}", other.type_name()),
    }
}

fn optional_number(object: &BTreeMap<String, Value>, field: &str) -> Option<u64> {
    match object.get(field) {
        None | Some(Value::Null) => None,
        Some(Value::Int(value)) => Some(
            u64::try_from(*value)
                .unwrap_or_else(|_| panic!("the reader accepted a negative budget {field:?}")),
        ),
        Some(other) => {
            panic!("the reader accepted a budget {field:?} that is {}", other.type_name())
        }
    }
}

fn tier(object: &BTreeMap<String, Value>) -> RiskTier {
    match object.get("risk_claim").and_then(Value::as_str) {
        Some("R0") => RiskTier::R0,
        Some("R1") => RiskTier::R1,
        Some("R2") => RiskTier::R2,
        Some("R3") => RiskTier::R3,
        other => panic!("the reader accepted a risk claim of {other:?}"),
    }
}

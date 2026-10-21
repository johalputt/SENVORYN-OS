//! Property tests for the decision path.
//!
//! The generator is a seeded xorshift rather than a random source, so a
//! failure names the seed it happened at and can be replayed exactly. A
//! property test that cannot be replayed is a property test nobody can fix.
//!
//! Every seed is used on its own, so that one seed tripping several rules
//! cannot mask a rule that is never the deciding one.

use std::collections::BTreeSet;

use senvoryn_canonical::{Value, canonicalize, encode_to_string, parse};
use senvoryn_scp::{
    AuditState, Capability, OperationRegistry, PolicyContext, Provenance, Request,
    ResolvedResource, RiskTier, SystemMode, attenuate, evaluate,
};

const SEEDS: [u64; 8] = [
    0x243F_6A88_85A3_08D3,
    0x1319_8A2E_0370_7344,
    0xA409_3822_299F_31D0,
    0x082E_FA98_EC4E_6C89,
    0x4528_21E6_38D0_1377,
    0xBE54_66CF_34E9_0C6C,
    0xC0AC_29B7_C97C_50DD,
    0x3F84_D5B5_B547_0917,
];

/// A small deterministic generator.
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

    fn below(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
}

const OPERATIONS: [&str; 5] = ["read", "write", "delete", "inspect", "status"];
const PURPOSES: [&str; 3] = ["user-request", "model-training", "backup"];

fn random_capability(rng: &mut Rng, id: &str) -> Capability {
    let operations: BTreeSet<String> = OPERATIONS
        .iter()
        .filter(|_| rng.chance(50))
        .map(|operation| (*operation).to_owned())
        .collect();
    let purposes: BTreeSet<String> =
        PURPOSES.iter().filter(|_| rng.chance(50)).map(|purpose| (*purpose).to_owned()).collect();
    Capability {
        capability_id: id.to_owned(),
        subject_id: "user:alice".to_owned(),
        resource_id: "file:report".to_owned(),
        resource_type: "file_object".to_owned(),
        operations,
        purposes,
        not_before: 0,
        expires_at: 1000,
        policy_epoch: 7,
        provenance: Provenance::Verified,
        delegation_depth: (rng.below(8) as u32) + 1,
        max_uses: if rng.chance(50) { Some(rng.below(10) + 1) } else { None },
    }
}

fn random_request(rng: &mut Rng) -> Request {
    Request {
        request_id: "req:property:0001".to_owned(),
        principal_id: "user:alice".to_owned(),
        recipient_id: "node:local".to_owned(),
        resource_id: "file:report".to_owned(),
        resource_type: "file_object".to_owned(),
        operation: OPERATIONS[rng.below(OPERATIONS.len() as u64) as usize].to_owned(),
        purpose: PURPOSES[rng.below(PURPOSES.len() as u64) as usize].to_owned(),
        policy_epoch: 7,
        issued_at: 100,
        expires_at: 900,
        nonce: "nonce-property-0001".to_owned(),
        risk_claim: RiskTier::R0,
        approval_digest: None,
        payload_digest: None,
        budget: None,
    }
}

fn context(registry: OperationRegistry) -> PolicyContext {
    PolicyContext {
        local_node_id: "node:local".to_owned(),
        current_policy_epoch: 7,
        now_monotonic: 500,
        now_wall_ms: 1_767_225_600_000,
        clock_state: senvoryn_scp::ClockState::Trusted,
        mode: SystemMode::Normal,
        known_principals: ["user:alice".to_owned()].into_iter().collect(),
        resolved_resources: [(
            "file:report".to_owned(),
            ResolvedResource {
                canonical_id: "file:report".to_owned(),
                resource_type: "file_object".to_owned(),
                version: "v1".to_owned(),
            },
        )]
        .into_iter()
        .collect(),
        operations: registry,
        audit_state: AuditState::Ready,
        action_digest: None,
        revoked_nonces: BTreeSet::new(),
        revoked_capabilities: BTreeSet::new(),
        strong_approval_granted: false,
        satisfied_preconditions: BTreeSet::new(),
    }
}

fn registry() -> OperationRegistry {
    senvoryn_scp::OperationRegistry::sample().expect("the embedded sample registry parses")
}

#[test]
fn attenuation_never_broadens_a_capability() {
    for seed in SEEDS {
        let mut rng = Rng(seed);
        for _ in 0..200 {
            let parent = random_capability(&mut rng, "cap:parent");
            let mut child = random_capability(&mut rng, "cap:child");
            // The child is a delegation of the parent, so it starts from the
            // parent's shape and then varies.
            child.subject_id = parent.subject_id.clone();
            child.resource_id = parent.resource_id.clone();
            child.resource_type = parent.resource_type.clone();

            if !attenuate(&parent, &child) {
                continue;
            }
            assert!(
                child.operations.is_subset(&parent.operations),
                "seed {seed:#x}: an attenuated child granted operations the parent does not have"
            );
            assert!(
                child.purposes.is_subset(&parent.purposes),
                "seed {seed:#x}: an attenuated child granted purposes the parent does not have"
            );
            assert!(
                child.not_before >= parent.not_before,
                "seed {seed:#x}: an attenuated child became valid earlier"
            );
            assert!(
                child.expires_at <= parent.expires_at,
                "seed {seed:#x}: an attenuated child outlived its parent"
            );
            assert_eq!(
                child.policy_epoch, parent.policy_epoch,
                "seed {seed:#x}: an attenuated child changed the policy epoch"
            );
            if let Some(limit) = parent.max_uses {
                assert!(
                    child.max_uses.is_some_and(|uses| uses <= limit),
                    "seed {seed:#x}: an attenuated child raised the use limit"
                );
            }
        }
    }
}

#[test]
fn attenuation_is_reflexive_on_the_depth_rule() {
    // A capability is not an attenuation of itself: the delegation depth must
    // decrease, and a child at the same depth is not a delegation.
    for seed in SEEDS {
        let mut rng = Rng(seed);
        let capability = random_capability(&mut rng, "cap:same");
        assert!(
            !attenuate(&capability, &capability),
            "seed {seed:#x}: a capability attenuated to itself"
        );
    }
}

#[test]
fn the_evaluator_is_deterministic() {
    for seed in SEEDS {
        let mut rng = Rng(seed);
        let context = context(registry());
        for _ in 0..200 {
            let request = random_request(&mut rng);
            let capability = random_capability(&mut rng, "cap:property");
            let first = evaluate(&request, Some(&capability), &context);
            let second = evaluate(&request, Some(&capability), &context);
            assert_eq!(
                first, second,
                "seed {seed:#x}: the same inputs produced two different decisions"
            );
        }
    }
}

#[test]
fn the_evaluator_does_not_mutate_its_inputs() {
    for seed in SEEDS {
        let mut rng = Rng(seed);
        let context = context(registry());
        for _ in 0..200 {
            let request = random_request(&mut rng);
            let capability = random_capability(&mut rng, "cap:purity");
            let request_before = request.clone();
            let capability_before = capability.clone();
            let context_before = context.clone();

            evaluate(&request, Some(&capability), &context);

            assert_eq!(request, request_before, "seed {seed:#x}: the request was mutated");
            assert_eq!(capability, capability_before, "seed {seed:#x}: the capability was mutated");
            assert_eq!(context, context_before, "seed {seed:#x}: the context was mutated");
        }
    }
}

#[test]
fn a_decision_that_does_not_permit_grants_nothing() {
    for seed in SEEDS {
        let mut rng = Rng(seed);
        let context = context(registry());
        for _ in 0..200 {
            let request = random_request(&mut rng);
            let capability = random_capability(&mut rng, "cap:authority");
            let decision = evaluate(&request, Some(&capability), &context);
            if decision.status() != senvoryn_scp::DecisionStatus::Permit {
                assert!(
                    !decision.grants_authority(),
                    "seed {seed:#x}: a {} decision granted authority",
                    decision.status()
                );
                assert_eq!(
                    decision.capability_id(),
                    None,
                    "seed {seed:#x}: a {} decision named a capability",
                    decision.status()
                );
            }
        }
    }
}

#[test]
fn a_permitted_decision_always_names_its_capability() {
    for seed in SEEDS {
        let mut rng = Rng(seed);
        let context = context(registry());
        for _ in 0..200 {
            let request = random_request(&mut rng);
            let capability = random_capability(&mut rng, "cap:scope");
            let decision = evaluate(&request, Some(&capability), &context);
            if decision.status() == senvoryn_scp::DecisionStatus::Permit {
                assert!(
                    decision.grants_authority(),
                    "seed {seed:#x}: a permitted decision granted no authority"
                );
                assert_eq!(
                    decision.capability_id(),
                    Some(capability.capability_id.as_str()),
                    "seed {seed:#x}: a permitted decision named the wrong capability"
                );
            }
        }
    }
}

#[test]
fn canonical_encoding_is_stable() {
    let seeds: [&str; 6] =
        ["{}", "[]", "null", "true", "0", r#"{"a":[1,2,3],"b":{"c":null},"d":"x"}"#];
    for seed in SEEDS {
        let mut rng = Rng(seed);
        for _ in 0..100 {
            let document = random_document(&mut rng);
            let Ok(parsed) = parse(&document) else { continue };
            let encoded = encode_to_string(&parsed);
            let reparsed = parse(&encoded).expect("an encoding always parses");
            assert_eq!(
                parsed, reparsed,
                "seed {seed:#x}: encoding and re-parsing changed the document"
            );
            assert_eq!(
                encode_to_string(&reparsed),
                encoded,
                "seed {seed:#x}: the encoding is not idempotent"
            );
        }
    }
    for document in seeds {
        let canonical = canonicalize(document).expect("canonicalizes");
        assert_eq!(canonical, document, "{document} was not already canonical");
    }
}

/// Builds a random document that the canonical parser will accept.
fn random_document(rng: &mut Rng) -> String {
    fn value(rng: &mut Rng, depth: usize) -> String {
        match rng.below(if depth > 3 { 4 } else { 6 }) {
            0 => "null".to_owned(),
            1 => if rng.chance(50) { "true" } else { "false" }.to_owned(),
            2 => rng.below(1000).to_string(),
            3 => format!("\"s{}\"", rng.below(1000)),
            4 => {
                let items: Vec<String> = (0..rng.below(4)).map(|_| value(rng, depth + 1)).collect();
                format!("[{}]", items.join(","))
            }
            _ => {
                let mut members: Vec<String> = Vec::new();
                for index in 0..rng.below(4) {
                    members.push(format!("\"k{index}\":{}", value(rng, depth + 1)));
                }
                format!("{{{}}}", members.join(","))
            }
        }
    }
    value(rng, 0)
}

#[test]
fn a_canonical_document_encodes_to_itself() {
    let document = parse(r#"{"b":1,"a":2}"#).expect("parses");
    let encoded = encode_to_string(&document);
    assert_eq!(encoded, r#"{"a":2,"b":1}"#);
    assert_ne!(encoded, r#"{"b":1,"a":2}"#);
    assert!(canonicalize(&encoded).expect("canonicalizes") == encoded);
}

#[test]
fn the_value_model_cannot_represent_a_fraction() {
    // The absence of a float variant is the enforcement: there is nowhere in
    // the model to put a fractional number, so it cannot reach a decision.
    let document = parse("1").expect("parses");
    assert!(matches!(document, Value::Int(1)));
}

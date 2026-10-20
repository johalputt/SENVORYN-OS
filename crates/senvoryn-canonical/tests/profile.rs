//! Tests for the canonical profile.
//!
//! Every rejection the profile makes is tested here by name. A profile whose
//! rules are not individually tested is a profile that will be loosened by a
//! well-meaning refactor without anyone noticing.

use senvoryn_canonical::{
    CanonicalError, MAX_ARRAY_ELEMENTS, MAX_DEPTH, MAX_DOCUMENT_BYTES, MAX_OBJECT_MEMBERS,
    MAX_STRING_BYTES, Value, canonicalize, is_canonical, parse, parse_bytes,
};

/// A document that is already canonical, used as the base for most tests.
const CANONICAL: &str = r#"{"a":1,"b":[true,false,null],"c":"x"}"#;

#[test]
fn parses_a_canonical_document() {
    let value = parse(CANONICAL).expect("canonical document parses");
    assert_eq!(value.get("a").and_then(Value::as_int), Some(1));
    assert_eq!(value.get("b").and_then(Value::as_array).map(<[Value]>::len), Some(3));
    assert_eq!(value.get("c").and_then(Value::as_str), Some("x"));
}

#[test]
fn encoding_is_stable_for_canonical_input() {
    assert_eq!(canonicalize(CANONICAL).expect("canonicalizes"), CANONICAL);
    assert!(is_canonical(CANONICAL));
}

#[test]
fn canonical_form_sorts_object_keys() {
    // The input is not canonical because the keys are out of order; the
    // canonical form puts them in ascending UTF-8 byte order.
    let canonical = canonicalize(r#"{"b":1,"a":2}"#).expect("canonicalizes");
    assert_eq!(canonical, r#"{"a":2,"b":1}"#);
    assert!(!is_canonical(r#"{"b":1,"a":2}"#));
}

#[test]
fn canonical_form_removes_insignificant_whitespace() {
    let canonical = canonicalize("{ \"a\" : [ 1 , 2 ] }").expect("canonicalizes");
    assert_eq!(canonical, r#"{"a":[1,2]}"#);
}

#[test]
fn rejects_duplicate_object_keys() {
    // The central hazard: two parsers that resolve duplicates differently
    // reach different decisions from the same bytes.
    let error = parse(r#"{"a":1,"a":2}"#).expect_err("duplicate keys are rejected");
    assert_eq!(error, CanonicalError::DuplicateKey { key: "a".to_owned(), offset: 7 });
}

#[test]
fn rejects_duplicate_keys_in_nested_objects() {
    let error = parse(r#"{"outer":{"a":1,"a":2}}"#).expect_err("duplicate keys are rejected");
    assert!(matches!(error, CanonicalError::DuplicateKey { .. }));
}

#[test]
fn accepts_a_repeated_key_in_separate_objects() {
    // The same key in two different objects is not a duplicate.
    let value = parse(r#"{"a":{"x":1},"b":{"x":2}}"#).expect("sibling objects are fine");
    assert_eq!(value.get("a").and_then(|v| v.get("x")).and_then(Value::as_int), Some(1));
}

#[test]
fn rejects_non_canonical_numbers() {
    for text in ["1.0", "1e0", "1E0", "1.5", "1e10", "-1.0", "0.1"] {
        let error = parse(text).expect_err("fractional and exponent numbers are rejected");
        assert!(
            matches!(error, CanonicalError::NonCanonicalNumber { .. }),
            "{text} was rejected as {error:?}"
        );
    }
}

#[test]
fn rejects_numbers_with_more_than_one_spelling() {
    for text in ["+1", "01", "-01", "00"] {
        let error = parse(text).expect_err("ambiguous integer spellings are rejected");
        assert!(
            matches!(error, CanonicalError::NonCanonicalNumber { .. }),
            "{text} was rejected as {error:?}"
        );
    }
}

#[test]
fn rejects_negative_zero() {
    let error = parse("-0").expect_err("negative zero is rejected");
    assert_eq!(error, CanonicalError::NonCanonicalNumber { text: "-0".to_owned(), offset: 0 });
}

#[test]
fn accepts_canonical_integers() {
    for (text, expected) in [("0", 0i64), ("-1", -1), ("42", 42), ("9223372036854775807", i64::MAX)]
    {
        let value = parse(text).expect("canonical integer parses");
        assert_eq!(value.as_int(), Some(expected), "{text}");
    }
}

#[test]
fn accepts_the_minimum_integer() {
    let value = parse("-9223372036854775808").expect("i64 minimum parses");
    assert_eq!(value.as_int(), Some(i64::MIN));
}

#[test]
fn rejects_an_integer_outside_the_signed_64_bit_range() {
    let error = parse("9223372036854775808").expect_err("out of range is rejected");
    assert!(matches!(error, CanonicalError::NumberOutOfRange { .. }));

    let error = parse("-9223372036854775809").expect_err("out of range is rejected");
    assert!(matches!(error, CanonicalError::NumberOutOfRange { .. }));
}

#[test]
fn rejects_content_after_the_top_level_value() {
    let error = parse("{} {}").expect_err("trailing content is rejected");
    assert_eq!(error, CanonicalError::TrailingContent { offset: 3 });

    let error = parse("1 2").expect_err("trailing content is rejected");
    assert!(matches!(error, CanonicalError::TrailingContent { .. }));
}

#[test]
fn rejects_invalid_utf8_bytes() {
    let error = parse_bytes(&[b'{', 0xff, b'}']).expect_err("invalid UTF-8 is rejected");
    assert_eq!(error, CanonicalError::InvalidUtf8 { valid_up_to: 1 });
}

#[test]
fn rejects_a_lone_surrogate_escape() {
    let error = parse(r#""\ud800""#).expect_err("a lone high surrogate is rejected");
    assert!(matches!(error, CanonicalError::LoneSurrogate { .. }));

    let error = parse(r#""\udc00""#).expect_err("a lone low surrogate is rejected");
    assert!(matches!(error, CanonicalError::LoneSurrogate { .. }));
}

#[test]
fn rejects_a_high_surrogate_without_its_low_surrogate() {
    let error = parse(r#""\ud800A""#).expect_err("an unpaired high surrogate is rejected");
    assert!(matches!(error, CanonicalError::LoneSurrogate { .. }));
}

#[test]
fn decodes_a_surrogate_pair() {
    let value = parse(r#""😀""#).expect("a surrogate pair decodes");
    assert_eq!(value.as_str(), Some("\u{1f600}"));
}

#[test]
fn writes_non_ascii_as_utf8_rather_than_as_an_escape() {
    // The escaped form and the UTF-8 form decode to the same character, so
    // only one of them can be canonical. The profile picks UTF-8, because
    // `\uXXXX` has two spellings and UTF-8 has one.
    let canonical = canonicalize(r#""\u00e9""#).expect("canonicalizes");
    assert_eq!(canonical, "\"\u{e9}\"");
    assert!(!is_canonical(r#""\u00e9""#));
    assert!(is_canonical("\"\u{e9}\""));
}

#[test]
fn accepts_a_solidus_escape_but_does_not_emit_one() {
    let value = parse(r#""a\/b""#).expect("the solidus escape is legal JSON");
    assert_eq!(value.as_str(), Some("a/b"));
    assert_eq!(canonicalize(r#""a\/b""#).expect("canonicalizes"), r#""a/b""#);
}

#[test]
fn escapes_control_characters_with_lowercase_hex() {
    // A control character with no short form is written as `\u00xx` with
    // lowercase hex. Lowercase and uppercase both decode to the same
    // character, so the profile picks one.
    let canonical = canonicalize(r#""\u0000""#).expect("canonicalizes");
    assert_eq!(canonical, r#""\u0000""#);

    let canonical = canonicalize(r#""\u001f""#).expect("canonicalizes");
    assert_eq!(canonical, r#""\u001f""#);
}

#[test]
fn uses_the_short_escape_forms_where_the_profile_defines_them() {
    for (input, expected) in [
        (r#"\u0008"#, r#"\b"#),
        (r#"\u0009"#, r#"\t"#),
        (r#"\u000a"#, r#"\n"#),
        (r#"\u000c"#, r#"\f"#),
        (r#"\u000d"#, r#"\r"#),
    ] {
        let canonical = canonicalize(&format!("\"{input}\"")).expect("canonicalizes");
        assert_eq!(canonical, format!("\"{expected}\""));
    }
}

#[test]
fn rejects_a_raw_control_character_in_a_string() {
    let error = parse("\"a\nb\"").expect_err("a raw newline in a string is rejected");
    assert!(matches!(error, CanonicalError::UnexpectedByte { .. }));
}

#[test]
fn rejects_an_undefined_escape() {
    let error = parse(r#""\x""#).expect_err("an undefined escape is rejected");
    assert!(matches!(error, CanonicalError::InvalidEscape { .. }));
}

#[test]
fn rejects_nesting_deeper_than_the_limit() {
    let too_deep = format!("{}1{}", "[".repeat(MAX_DEPTH + 1), "]".repeat(MAX_DEPTH + 1));
    let error = parse(&too_deep).expect_err("over-deep nesting is rejected");
    assert!(matches!(error, CanonicalError::DepthLimitExceeded { .. }));
}

#[test]
fn accepts_nesting_at_the_limit() {
    let at_limit = format!("{}1{}", "[".repeat(MAX_DEPTH), "]".repeat(MAX_DEPTH));
    parse(&at_limit).expect("nesting at the limit is accepted");
}

#[test]
fn rejects_an_object_with_too_many_members() {
    let members = (0..=MAX_OBJECT_MEMBERS)
        .map(|index| format!("\"k{index}\":1"))
        .collect::<Vec<_>>()
        .join(",");
    let document = format!("{{{members}}}");
    let error = parse(&document).expect_err("too many members is rejected");
    assert!(matches!(error, CanonicalError::MemberLimitExceeded { .. }));
}

#[test]
fn accepts_an_object_with_the_limit_number_of_members() {
    let members = (0..MAX_OBJECT_MEMBERS)
        .map(|index| format!("\"k{index}\":1"))
        .collect::<Vec<_>>()
        .join(",");
    let document = format!("{{{members}}}");
    parse(&document).expect("the limit number of members is accepted");
}

#[test]
fn rejects_an_array_with_too_many_elements() {
    let elements = vec!["1"; MAX_ARRAY_ELEMENTS + 1].join(",");
    let document = format!("[{elements}]");
    let error = parse(&document).expect_err("too many elements is rejected");
    assert!(matches!(error, CanonicalError::ArrayLimitExceeded { .. }));
}

#[test]
fn rejects_a_string_longer_than_the_limit() {
    let long = "a".repeat(MAX_STRING_BYTES + 1);
    let document = format!("\"{long}\"");
    let error = parse(&document).expect_err("an over-long string is rejected");
    assert!(matches!(error, CanonicalError::StringLimitExceeded { .. }));
}

#[test]
fn rejects_a_document_longer_than_the_limit() {
    let document = format!("\"{}\"", "a".repeat(MAX_DOCUMENT_BYTES));
    let error = parse(&document).expect_err("an over-long document is rejected");
    assert!(matches!(error, CanonicalError::DocumentLimitExceeded { .. }));
}

#[test]
fn rejects_a_truncated_document() {
    let error = parse("{\"a\":").expect_err("a truncated document is rejected");
    assert!(matches!(error, CanonicalError::UnexpectedEnd { .. }));
}

#[test]
fn rejects_a_trailing_comma_in_an_object() {
    let error = parse(r#"{"a":1,}"#).expect_err("a trailing comma is rejected");
    assert!(matches!(error, CanonicalError::UnexpectedByte { .. }));
}

#[test]
fn rejects_an_unquoted_object_key() {
    let error = parse(r#"{a:1}"#).expect_err("an unquoted key is rejected");
    assert!(matches!(error, CanonicalError::UnexpectedByte { .. }));
}

#[test]
fn parses_an_empty_object_and_array() {
    assert_eq!(parse("{}").expect("empty object"), Value::Object(Default::default()));
    assert_eq!(parse("[]").expect("empty array"), Value::Array(Vec::new()));
}

#[test]
fn boolean_is_not_an_integer() {
    // `true` and `1` are different claims; a parser that converts between
    // them has lost information.
    assert_eq!(parse("true").expect("parses").as_int(), None);
    assert_eq!(parse("1").expect("parses").as_bool(), None);
}

#[test]
fn value_object_rejects_duplicate_keys_at_construction() {
    let value = Value::object([("a", Value::Int(1)), ("a", Value::Int(2))]);
    assert!(value.is_none(), "a duplicate key has no canonical meaning");
}

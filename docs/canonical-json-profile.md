# Senvoryn Canonical JSON Profile — `SCP-CJSON-0001`

**Profile identifier:** `SCP-CJSON-0001`
**Version:** 1
**Status:** Prototype. Named and specified; the production encoding remains subject to independent review under [ADR-003](../decisions/ADR-003-protocol-encoding.md) and [ADR-008](../decisions/ADR-008-canonical-json-profile.md).

---

## 1. What this profile is for

A protocol object that is signed, hashed, or compared byte-for-byte needs exactly one
byte sequence. This profile defines that byte sequence.

The constraint that makes the obvious solution wrong: a JSON document has many correct
spellings of the same value. `{"a":1,"b":2}` and `{"b":2,"a":1}` are the same document.
`1`, `1.0`, `1e0`, `+1`, and `01` are the same number. A serializer's default is not a
protocol specification, and two implementations that disagree about it disagree about
what was signed.

The profile therefore fixes three things together: a **grammar** that is narrower than
JSON, a **canonical form** that is a function of the value, and **limits** that bound
what an attacker can make the parser allocate.

## 2. Conformance

A document is **canonical** under this profile if and only if all of the following hold.

### 2.1 Encoding

1. The document is UTF-8. A byte-order mark is not permitted.
2. Exactly one top-level value appears in the document. Content after that value is an
   error, not whitespace to be skipped.
3. No insignificant whitespace appears anywhere: no space, tab, carriage return, or
   line feed between tokens.

### 2.2 Objects

4. Object member names are JSON strings.
5. Member names are unique within their object. A repeated name is an error.
6. Members appear in ascending order of the UTF-8 byte sequence of their names.

### 2.3 Arrays

7. Array elements appear in their order. The profile does not sort them.

### 2.4 Numbers

8. A number is an integer and is written without a fraction, an exponent, a leading
   `+`, a leading zero, or a redundant minus sign.
9. Integers are in the signed 64-bit range. A value outside it is an error, not a
   truncation.
10. `-0` is not a number under this profile. The value zero is written `0`.

### 2.5 Strings

11. The escape set is fixed: `\"`, `\\`, `\b`, `\f`, `\n`, `\r`, `\t`, and `\u00XX`
    for the remaining C0 control characters. The hex digits of a `\u00XX` escape are
    lower case.
12. A `\uXXXX` escape that denotes a code point in the surrogate range D800–DFFF is an
    error, unless it is the first half of a surrogate pair, in which case the pair is
    decoded to the code point it denotes.
13. Every other character — including every non-ASCII character — appears literally as
    UTF-8. A non-ASCII character is never written as an escape.
14. A control character below U+0020 that is not one of the six that have a short
    escape appears as a `\u00XX` escape.

### 2.6 The value model

15. The profile's values are: null, true, false, an integer, a string, an array, and an
    object. There is no fractional number, and there is no representation of one.

## 3. Limits

The limits are checked **before** the corresponding allocation is made. A parser that
allocates first and checks later can be made to allocate by the very input that was
meant to be rejected.

| Limit | Value |
|---|---|
| Document size | 1 MiB (1 048 576 bytes) |
| Nesting depth | 32 |
| Object members | 1024 per object |
| Array elements | 4096 per array |
| String length | 65 536 bytes |

## 4. The reference implementation

`crates/senvoryn-canonical` implements this profile with **no third-party dependencies**.

| Item | Rust |
|---|---|
| Parse a document | `senvoryn_canonical::parse(&str)`, `parse_bytes(&[u8])` |
| Canonicalise a document | `senvoryn_canonical::canonicalize(&str) -> Result<String, CanonicalError>` |
| Test whether a document is canonical | `senvoryn_canonical::is_canonical(&str) -> bool` |
| Encode a value | `senvoryn_canonical::encode_to_string(&Value) -> String` |
| Profile identifier | `senvoryn_canonical::PROFILE_ID`, `PROFILE_VERSION` |

The parser is **lenient about layout and strict about grammar**. It accepts a
document whose members are unsorted and whose whitespace is irregular, and returns the
value; `encode_to_string` then produces the canonical bytes. It rejects a document
whose *grammar* is outside the profile — a duplicate key, `1.0`, `01`, a lone
surrogate — because those are ambiguities rather than layout.

`canonicalize` is the strict form: it fails unless the input is already canonical.

## 5. What this profile does not do

- **It does not specify a digest.** The canonical bytes are produced; the digest over
  them is the caller's. [ADR-002](../decisions/ADR-002-crypto-boundary.md) requires
  publicly reviewed primitives, and inventing one here would put unverified
  cryptography in the trusted core.
- **It does not specify how a document is validated against a schema.** Schema
  validation and signature verification are separate gates, per ADR-003.
- **It does not cover the wire format.** The request envelope carries date-times; the
  policy layer works in monotonic time. The conversion is the transport layer's job.

## 6. Conformance testing

The profile's own test suite is `crates/senvoryn-canonical/tests/profile.rs`. It covers
every clause above, including the limits at and one past the boundary.

`crates/senvoryn-canonical/tests/fuzz.rs` mutates a seed corpus deterministically and
asserts that the parser never panics, that a document which parses re-parses to the
same value, and that an encoding is idempotent. It is a plain test rather than
`cargo-fuzz` because the project has no third-party dependencies and no nightly
toolchain, and a fuzz target that cannot run in continuous integration is a fuzz target
nobody runs.

## 7. References

- [ADR-003 — Canonical Protocol Encoding](../decisions/ADR-003-protocol-encoding.md)
- [ADR-008 — Canonical JSON Profile SCP-CJSON-0001](../decisions/ADR-008-canonical-json-profile.md)
- [ADR-002 — Cryptographic Boundary](../decisions/ADR-002-crypto-boundary.md)
- `crates/senvoryn-canonical/` — the implementation

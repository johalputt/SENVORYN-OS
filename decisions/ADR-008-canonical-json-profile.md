# ADR-008 — Canonical JSON Profile `SCP-CJSON-0001`

**Status:** Proposed
**Date:** 2026-10-20
**Deciders:** Project maintainers
**Affected components:** Core Contract Specification §3, protocol encoding, `crates/senvoryn-canonical`

---

## Context

- **Problem:** ADR-003 requires that a specific canonical byte encoding be named,
  specified, and tested before any protocol object is signed. Its status says the
  prototype may use deterministic JSON, with production pending review. Nothing was
  named, so nothing could be tested against, and two implementations were free to
  disagree about what was signed.
- **Constraint:** The encoding has to be narrow enough to have exactly one spelling per
  value, and the parser has to be checkable without a network, a package registry, or a
  nightly toolchain. The project's trusted core is to have no third-party dependencies.
- **Assumptions:** The set of security-relevant JSON documents is small and known. A
  profile can be specified for the protocol's own documents without attempting to
  profile JSON as a whole.

## Decision

We will specify the Senvoryn Canonical JSON Profile, identifier `SCP-CJSON-0001`, as
the normative encoding for every protocol object whose bytes are signed, hashed, or
compared. The profile is documented in
[`docs/canonical-json-profile.md`](../docs/canonical-json-profile.md) and implemented
in `crates/senvoryn-canonical` with no third-party dependencies.

The profile fixes a grammar, a canonical form, and limits. The grammar rejects
duplicate keys, non-integer numbers, redundant signs, leading zeros, lone surrogates,
and trailing content. The canonical form sorts object members by the UTF-8 byte order
of their names and removes all insignificant whitespace. The limits are checked before
the corresponding allocation.

## Rationale

A canonical form is a function of the value. Two parties that hold the same value
therefore hold the same bytes, and a digest over those bytes is meaningful. The
alternative — leaving the encoding to each implementation — makes every digest
ambiguous, which in practice means that signatures are either not verified or verified
against a form the signer did not produce.

Integers only is the narrowest choice that removes the whole class of
alternate-spelling ambiguities at once. A JSON number may be written as a fraction, an
exponent, a leading plus, a leading zero, or a negative zero, and each of those is a
distinct byte sequence for a value that a reader will consider equal. Excluding them
removes five ambiguities with one rule, at the cost of forbidding fractional
quantities in protocol documents — which the contract does not need.

The limits are stated in the profile rather than left to the implementation because an
unbounded parser is a denial-of-service primitive. Checking a limit before the
allocation it bounds is the difference between rejecting a document and being made to
hold it.

## Consequences

**What this makes easy:**
- A digest over a protocol object is unambiguous, and two implementations can be
  compared byte for byte.
- The parser is small enough to read, and has no third-party code in it to audit.
- The limits are part of the specification, so a conformance claim about them is
  checkable.

**What this makes hard:**
- Every protocol document that is signed must be canonicalised before it is signed.
- Fractional numbers cannot appear in protocol objects. A document that needs them
  needs a different encoding, and that would be a new profile version.

**What this costs:**
- The profile is now a compatibility commitment. Changing any rule in §2 is a breaking
  change and requires a new profile identifier.

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Deterministic JSON as RFC 8785 describes it | RFC 8785 sorts members by UTF-16 code units and serialises numbers by ECMAScript rules. Both were chosen for interoperation with a JavaScript ecosystem this project does not have, and RFC 8785's number rules permit fractions and exponents, which is the ambiguity this profile exists to remove. |
| A dependency such as `serde_json` with sorted maps | It would put a third-party parser in the trusted core, and the project has committed to a zero-dependency trusted core. |
| Leave the encoding to each implementation, as ADR-003's status permits | That is what the last milestone did, and it is why there was nothing to test against. The status is the thing being changed. |
| Restrict the profile to integers only, and forbid strings | Over-tightening. Strings carry the identifiers the protocol is made of. |

## Open items

- [ ] Independent review of the profile before it is used to verify anything — an
      unsigned profile is a design artefact; a signed one needs a second pair of eyes.
- [ ] Whether a future profile version should permit fractional numbers for a
      measurement-bearing document type — what evidence would close it: a protocol
      object that cannot be expressed as integers.
- [ ] Whether the profile needs a canonical form for *streams* of documents — what
      evidence would close it: a protocol message that carries more than one object.

## References

- [ADR-003 — Canonical Protocol Encoding](ADR-003-protocol-encoding.md)
- [ADR-002 — Cryptographic Boundary](ADR-002-crypto-boundary.md)
- [`docs/canonical-json-profile.md`](../docs/canonical-json-profile.md)
- [`crates/senvoryn-canonical/`](../crates/senvoryn-canonical/) — the implementation
- Core Contract Specification §3

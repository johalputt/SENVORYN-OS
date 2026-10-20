# ADR-003 — Canonical Protocol Encoding

**Status:** Prototype may use deterministic JSON; production pending review

## Decision
A specific canonical byte encoding must be named, specified, and tested before signing any protocol object. Reject duplicate keys, ambiguous numbers, invalid encodings, unknown critical fields, and alternative representations of the same security-sensitive value.

## Consequences
Schema validation and signature verification are separate gates. A serializer default is not a protocol specification.

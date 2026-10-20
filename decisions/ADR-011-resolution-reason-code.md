# ADR-011 — Adding a Reason Code for the Object-Resolution Gate

**Status:** Proposed
**Date:** 2026-10-20
**Deciders:** Project maintainers
**Affected components:** `errors/denial-codes.json`, Core Contract Specification §3.2 gate 6

---

## Context

- **Problem:** The specification's validation order has sixteen gates. Gate 6 is
  **object resolution**: the request's resource must resolve to a canonical object.
  The denial-code registry, which is the authority on the reason codes a decision may
  carry, had no code for a resource that does not resolve.

  The gap was found by the conformance vectors, not by reading: the vector that
  exercises gate 6 had to produce a reason code, and there was none to produce. The
  Python reference model had previously emitted `UNKNOWN_RISK_TIER` for that situation,
  which was not in the registry at all.
- **Constraint:** A decision record's reason codes are drawn from the registry, and the
  registry is what a reader consults to learn what a decision means. A gate that
  denies for a reason the registry does not describe produces a decision no reader can
  interpret, and it makes the registry incomplete in a way that is hard to detect,
  because nothing checks for it until a gate is exercised.
- **Assumptions:** The registry can be extended without a version break. Its consumers
  are the decision record's `reason_codes` array and the code that maps a code to a
  default status.

## Decision

We will add `RESOURCE_UNRESOLVABLE` to the denial-code registry, with class
`resolution` and default `DENY`.

Gate 6 denies with this code when the request's resource identifier does not resolve to
an object. The denial is a plain `DENY`: an unresolvable resource is not a challenge,
because there is no evidence the caller could supply that would make an identifier
resolve.

The registry now carries 27 codes. Every code in it is produced by at least one
implementation, and every code an implementation can produce is in it; both directions
are checked by tests on both sides.

## Rationale

The registry's purpose is to be complete. A gate in the specification with no
corresponding code is a hole in the vocabulary, and holes in a vocabulary are filled
ad hoc — which is how `UNKNOWN_RISK_TIER` came to exist, and why it was wrong. The code
that was emitted did not exist; the code that describes the situation did.

The class `resolution` is new, and that is deliberate. The classes in the registry
describe the *kind* of failure, not the gate, and a resource that cannot be resolved
fails in a way none of the existing classes describes: it is not a protocol failure,
not a freshness failure, and not a risk failure.

The default is `DENY` rather than `CHALLENGE`, because the gate that uses this code
has already resolved everything it can. A request naming a resource that does not
exist is refused. There is no further evidence that would help.

## Consequences

**What this makes easy:**
- Gate 6 can deny with a code a reader can look up.
- The registry's completeness is now a tested property rather than an aspiration.

**What this makes hard:**
- Every implementation has to know the code, so the shared conformance vectors now
  carry it and both implementations are checked against it.
- The registry has 27 codes and each new one has to be added to both implementations,
  which the registry-sync tests enforce.

**What this costs:**
- A change to the contract pack, and a regeneration of the checksum manifest.

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Reuse an existing code, such as `MALFORMED_OR_MISSING_IDENTITY` | The request is not malformed; it is well-formed and names something that does not exist. A reader following the registry would be sent to look for a parse failure, which is not what happened. |
| Reuse `UNKNOWN_RISK_TIER`, as the Python prototype did | The code was not in the registry, and it describes the wrong thing: the risk tier was not unknown, the resource was unresolvable. |
| Make gate 6 a `CHALLENGE` | There is nothing for the caller to do differently. A challenge is a request for evidence, and no evidence makes an identifier resolve. |
| Leave gate 6 unimplemented until the resolver exists | The gate is normative and the denial is well-defined without a resolver. Leaving it out would leave a specification gate with no behaviour at all. |

## Open items

- [ ] Whether a resource that resolves but whose *version* does not exist should carry
      this code or a separate one — what evidence would close it: the specification
      stating whether version resolution is part of gate 6 or a separate check.
- [ ] Whether resolution failures are distinguishable to the caller, since telling a
      caller that a resource does not exist is itself an information disclosure — what
      evidence would close it: a rule in the specification about what a refusal may
      reveal.

## References

- Core Contract Specification §3.2, gate 6
- [`errors/denial-codes.json`](../errors/denial-codes.json)
- [`crates/senvoryn-scp/tests/reason_code_registry.rs`](../crates/senvoryn-scp/tests/reason_code_registry.rs)
- [`tests/test_reason_code_registry.py`](../tests/test_reason_code_registry.py)

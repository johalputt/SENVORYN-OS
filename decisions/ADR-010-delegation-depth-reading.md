# ADR-010 — Delegation Depth: The Literal Reading, and the Contradiction

**Status:** Proposed
**Date:** 2026-10-20
**Deciders:** Project maintainers
**Affected components:** Core Contract Specification §4.2, capability model, `crates/senvoryn-scp`, `reference/scp_reference.py`

---

## Context

- **Problem:** Two documents in this repository describe the same number and disagree
  about its direction.

  Core Contract Specification §4.2 states that a child capability's **delegation depth
  must decrease** relative to its parent's.

  The capability schema's `delegation_depth` field is documented as the depth of the
  delegation chain — the number of delegations from the root — with the field
  `parent_capability_id` linking a capability to the one that granted it. Under that
  reading, a child sits **one deeper** than its parent, and the depth **increases**.

  Both readings cannot be correct. Under the literal reading of §4.2, a root
  capability at depth 0 can never be delegated at all, because no value is less than
  zero — which would make the root of every chain the one capability that cannot be
  delegated.
- **Constraint:** The attenuation rule is the mechanism by which authority is
  narrowed on the way down a delegation chain. If the rule is wrong in either
  direction, the effect is that a child capability is either wrongly refused (an
  availability failure) or wrongly accepted (an authority failure). The second of
  those is the worse one, so a rule that refuses more is the safer failure.
- **Assumptions:** The specification text is normative and the schema's prose
  description is not. Where a normative text and an explanatory annotation disagree,
  the normative text governs until the specification says otherwise.

## Decision

We will implement the attenuation rule as the specification's §4.2 states it:
**a child's delegation depth must be strictly less than its parent's.**

We will record the contradiction rather than resolve it silently. The rule is stated in
one place — `attenuate` in `crates/senvoryn-scp` and `attenuate` in
`reference/scp_reference.py` — and both implementations are held to it by the shared
conformance vectors.

This ADR does **not** amend the specification. It records the implementation's reading
of it and asks the specification to be reconciled.

## Rationale

Where a normative text and an annotation disagree, an implementation that follows the
annotation is making an editorial decision that is invisible to a reviewer reading the
specification. An implementation that follows the normative text, and says so, is
making a decision the reviewer can check.

The rule also fails in the safer direction. Under the literal reading, a delegation
chain has a bounded length and a depth-0 root is not delegable. That is inconvenient,
but inconvenient is recoverable and dangerous is not: the failure mode of the literal
reading is a refused delegation, and the failure mode of the other reading is a
delegation chain that never ends, in which every capability in the chain looks
equally deep and no chain can be shown to be too long.

The decision is reversible without cost. If the specification is amended, the rule
changes in one function and the conformance vectors change with it.

## Consequences

**What this makes easy:**
- An implementer can check the rule against the specification text directly.
- The conformance vectors pin the behaviour, so the two implementations cannot drift.

**What this makes hard:**
- A root capability cannot currently be delegated, which no deployment has needed yet
  because no deployment exists.
- The capability schema's prose may need amending to match, or the specification's
  §4.2 needs amending to match the schema.

**What this costs:**
- Nothing yet. The cost appears if the specification is amended the other way and
  implementations have shipped against the literal reading.

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Implement the depth rule as "the child may not increase the chain depth", matching the schema | It contradicts the normative text, and a reviewer reading the specification would conclude the implementation is wrong without being able to see why it is right. |
| Reject any child whose depth is not exactly one more than its parent's | A stricter rule than either reading, and it is not in either document. |
| Leave the depth out of the attenuation rule entirely | That removes a check the specification requires, in the direction of permitting more. |
| Resolve the contradiction by amending the specification | That is the correct end state, but it is the specification's to change, through the specification's own process, and not this ADR's to do unilaterally. |

## Open items

- [ ] Which reading is correct — what evidence would close it: an amendment to Core
      Contract Specification §4.2 or to the capability schema's `delegation_depth`
      description, stating the direction explicitly.
- [ ] Whether a root capability should be delegable at all — what evidence would
      close it: the same amendment.
- [ ] What the maximum delegation chain length is, and whether exceeding it is a
      denial or a quarantine — what evidence would close it: a rule in the
      specification.

## References

- Core Contract Specification §4.2
- [`schemas/capability.schema.json`](../schemas/capability.schema.json)
- [`crates/senvoryn-scp/src/capability.rs`](../crates/senvoryn-scp/src/capability.rs)
- [`reference/scp_reference.py`](../reference/scp_reference.py)
- [`conformance/vectors/vectors.json`](../conformance/vectors/vectors.json)

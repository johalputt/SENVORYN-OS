# ADR-014 — The Change Capsule Executor Has Two Implementations

**Status:** Proposed
**Date:** 2026-10-21
**Deciders:** Project maintainers
**Affected components:** `reference/capsule_reference.py`, `crates/senvoryn-capsule`, `conformance/capsule-vectors.json`

---

## Context

- **Problem:** The Gate 1 work left the SCP with two implementations — a
  Python reference model and a Rust implementation — held to one another by a
  shared vector file. The Gate 2 work initially did not: the Change Capsule
  executor was written once, in Rust, and checked against a vector file that
  only that one implementation read.

  A vector file read by one implementation is not an agreement. It is a
  transcript. If the executor's behaviour changes and the file is updated to
  match, nothing fails — the file has simply been rewritten to say what the
  code now does. The vectors catch drift between the code and the contract
  only insofar as somebody remembers to update the contract, and they catch
  **nothing** between two implementations, which is the failure the shared SCP
  vectors exist to prevent.

  This is not a hypothetical. The registry's merge rule had the operator
  backwards — it kept the *more* reversible of two conflicting rules instead
  of the less reversible one — and the sample registry holds one rule per
  operation, so the merge never ran in any test. Writing the same rule a
  second time, from the contract rather than from the first implementation,
  is what found it.

- **Constraint:** The trusted core carries no third-party dependencies
  (ADR-009), so the two implementations share nothing but the contract
  artifacts: the state table, the schema, the sample registry, the sample
  capsule, and the vector file. Neither implementation is allowed to read
  the other's data structures.

  The Rust implementation is the one that would ship. The Python one is a
  reference model, in the same sense that `reference/scp_reference.py` is:
  a second opinion about what the contract means, written by a different
  route.

- **Assumptions:** The specification's §5.3 transaction contract, the state
  table, and the vector file are enough to write a second implementation
  without consulting the first. If that turns out to be false, the
  specification is underspecified, and that is worth knowing too.

## Decision

We will implement the Change Capsule executor **twice** — once in Rust, once
in Python — and hold them to one another with the shared vector file
`conformance/capsule-vectors.json`, exactly as the SCP's two implementations
are held to `conformance/vectors/vectors.json`.

The two implementations are not a port of one another. The Python model
raises where the Rust model returns an option, parses timestamps with the
standard library where the Rust model computes days from the civil calendar,
and merges registry rules with a different function. They agree because they
both implement the same contract, not because one was copied from the other.

The vector file is the shared contract. Every case names a state the
transaction must reach and a fragment of the reason it must give, and **both**
implementations must produce it. A change to either implementation that
changes an outcome has to change the file, which changes what the other
implementation must produce — so the file cannot be quietly rewritten to
match a divergence.

The vector file also carries the reason fragment, not only the state. The
wording of a reason is part of the contract, so a control cannot be satisfied
by a different reason that happens to produce the same state.

## Rationale

An independent implementation is the only thing that can tell the difference
between "the code does what the contract says" and "the code does what I
thought the contract said". Every other check — a unit test, a coverage
number, a property test — compares the code with the author's understanding
of the contract. Two implementations written from the contract compare it
with the contract.

The cost is real and it is worth stating: the second implementation is
several hundred lines that exist only to disagree usefully, and it will fall
behind whenever the contract changes. The mitigation is that the vectors are
the contract, so a contract change is a vector change, and the second
implementation failing to follow it is the *point*.

The reason fragment is carried in the vector file for the same reason the
state is. A state alone is too coarse: `REJECTED` is produced by at least
eight different controls, and a vector that only said "REJECTED" would be
satisfied by any of them. The fragment names which control fired, so a
control that is quietly replaced by another one shows up as a disagreement
rather than as a pass.

## Consequences

**What this makes easy:**
- The capsule vectors are an agreement rather than a transcript, and the
  same mechanism the SCP uses applies to the capsule.
- A defect that only one implementation can have is found, because the other
  one does not have it. The registry's reversibility merge was found this
  way.
- The Python model is a readable statement of what the transaction does, for
  a reader who does not read Rust.

**What this makes hard:**
- Two implementations to keep in step, and a vector file that has to be
  updated when either changes.
- The mutation gate now has to cover both, and a Python mutation needs the
  gate to run a Python test rather than a Rust one. That is implemented, and
  it is the reason the gate now knows about two languages.

**What this costs:**
- About six hundred lines of Python, and the run time of the gate.

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Keep one implementation and a vector file | The vector file then records what the code does. It catches a change nobody recorded, which is worth something, but it cannot catch a change the author believes is correct — which is the change that matters. |
| Write the second implementation in Rust as a port of the first | A port shares the first implementation's mistakes, because it shares its author's reading of the contract. The point is a different route to the same place, which means a different language and a different set of standard-library tools. |
| Write the second implementation in Python and treat it as subordinate | Subordination is what made the SCP's Python model a prototype in the first place. A reference model that is known to be subordinate is not an independent opinion, and the defects it finds are exactly the ones a subordinate model is likely to share. |
| Compare the two implementations' outputs directly, case by case, rather than through a shared file | The direct comparison catches a divergence the file does not cover, which is a real gain. The shared file catches it too, and additionally pins the contract itself. Doing both would need a report file written by each implementation and a comparison step, which is more machinery for a smaller gain. The file's approach is the one the SCP already uses, and consistency has value of its own. |

## Open items

- [ ] Whether the reason fragments in the vector file should be the *only*
      wording, or whether the two implementations may word a reason
      differently and the file should carry a reason *code* instead — what
      evidence would close it: a decision on whether reasons are part of the
      contract or part of the presentation.
- [ ] Whether the envelope→request mapping should get the same treatment as
      the capsule — what evidence would close it: a second implementation of
      the mapping, or a decision that the mapping is simple enough that one
      implementation and a fuzz harness are enough.
- [ ] Whether the two implementations should be checked against each other
      on cases *outside* the vector file, such as a randomly generated plan —
      what evidence would close it: a generator both implementations read.

## References

- [`conformance/capsule-vectors.json`](../conformance/capsule-vectors.json)
- [`reference/capsule_reference.py`](../reference/capsule_reference.py)
- [`crates/senvoryn-capsule`](../crates/senvoryn-capsule)
- [`scripts/mutations.json`](../scripts/mutations.json)
- [ADR-009 — Zero-dependency trusted core](ADR-009-zero-dependency-trusted-core.md)
- Core Contract Specification §5.3, Change Capsule transaction contract

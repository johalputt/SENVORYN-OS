# ADR-013 — The Pre-Effect Failure Transitions the Capsule State Table Was Missing

**Status:** Proposed
**Date:** 2026-10-21
**Deciders:** Project maintainers
**Affected components:** `state-machines/change-capsule-transitions.json`, Core Contract Specification §5.3

---

## Context

- **Problem:** The specification says of the Change Capsule transaction: *"Any
  pre-effect failure goes to `FAILED` or `REJECTED` without mutation."*

  The machine-readable state table did not implement that. `FAILED` was listed as
  a state of the machine, and **no transition in the table entered it**. The table
  could express a refusal (`REJECTED`) and an expiry (`EXPIRED`) from `DRAFT`, but
  from every state after `DRAFT` it could express only a *success* path, a
  cancellation, or a compensation. There was no way to say "this failed before it
  changed anything" once the capsule had been validated.

  The gap was found by building the executor, not by reading the table. The
  executor's pre-effect failure paths needed a destination, and the table did not
  have one. The same shape of defect as the denial-code registry's missing
  resolution code (ADR-011): a state the specification defines and the machine
  cannot produce.

- **Constraint:** The state table is a contract artifact, hashed into the checksum
  manifest and read by the executor at build time. Changing it changes what the
  machine can do, so the change has to be justified against the specification
  rather than made to fit the code.

- **Assumptions:** The specification's sentence is normative, and the table is
  meant to implement it rather than to narrow it. A lower-level artifact may
  narrow a capability or add checks, but it may not remove a failure path the
  specification requires.

## Decision

We will amend the Change Capsule transition table so that every state from which a
pre-effect failure can occur can reach `FAILED`.

The added transitions are:

| From | Guard |
|---|---|
| `DRAFT` | `interrupted_during_validation` |
| `VALIDATED` | `interrupted_before_authorization` |
| `AUTHORIZED` | `interrupted_during_staging` |
| `STAGED` | `interrupted_during_precheck` |
| `PRECHECKED` | `interrupted_before_any_effect` |
| `APPLYING` | `step_failed_before_its_effect_landed` |

`REJECTED` remains the destination for a refusal — a capsule the policy, the
registry or the decision will not accept — and `FAILED` for an operational
failure: the machine broke, the storage was unreachable, the artifact was
missing, the run was interrupted. The two are distinct because a refusal is a
decision and a failure is an accident, and the evidence a reader wants differs.

The `APPLYING` transition deserves its own note. `APPLYING → COMPENSATING` already
existed and is the path for a failure *after* an effect landed. The new
`APPLYING → FAILED` is the path for a step that failed *before* its effect landed,
where there is nothing to compensate.

The amendment is a **narrowing of what the machine permits**, not a broadening of
authority: it adds failure paths, and it adds no path that reaches `COMMITTED`
that did not already exist.

## Rationale

A state that no transition reaches is a hole in the machine. The specification
names `FAILED` as the destination for a pre-effect failure, and a reader of the
table who asked "what happens if the storage fails while staging?" would find no
answer. The only alternatives were to leave the state unreachable — which makes
the table's own list of states a description of a machine that cannot fail — or to
route such failures to `CANCELLED`, which is wrong: a cancellation is a decision
to stop, and an I/O error is not a decision.

The distinction between `REJECTED` and `FAILED` matters for the evidence trail. A
rejected capsule never ran; a failed one was in flight. A reader asking "did
anything happen?" gets different answers, and the state is how they get it.

The `APPLYING → FAILED` transition is the one the specification's sentence
implies most directly and the one the table most conspicuously lacked: a step can
fail before its effect lands, and in that case there is nothing to compensate.

## Consequences

**What this makes easy:**
- The table can express every pre-effect failure the specification requires.
- The executor's failure paths have a destination that says what kind of failure
  it was.
- A test can assert that every state in the table is reachable, which is now
  possible because there is no unreachable state left.

**What this makes hard:**
- The table changed, so the checksum manifest changed with it.
- `FAILED` is now reachable from six states, and the executor's `fail` helper
  consults the table rather than assigning the state, so the destinations cannot
  drift from the table.

**What this costs:**
- Six transitions in a contract artifact, and the regeneration of the checksum
  manifest.

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Leave the table unchanged and route pre-effect failures to `REJECTED` | `REJECTED` means the capsule was refused. An I/O error during staging is not a refusal, and a reader of the evidence would be told the wrong thing about a transaction that was in flight. |
| Route pre-effect failures to `CANCELLED` | A cancellation is a decision to stop before mutating. An interrupted run that failed for reasons nobody chose is not a cancellation. |
| Remove `FAILED` from the table's list of states | The specification names it. Removing it from the table would make the machine disagree with the document it implements, which is the same defect in the other direction. |
| Add a single `* → FAILED` wildcard rather than six named transitions | The table's contract is that every transition has a named guard. A wildcard would remove the ability to say *why* the transaction failed, and would make the table no longer a specification of the machine. |

## Open items

- [ ] Whether a failure during `VERIFYING` should reach `FAILED` when no effect is
      outstanding, or always go to `COMPENSATING` — what evidence would close it:
      the specification stating whether a verification failure implies an effect
      that has to be undone.
- [ ] Whether `CANCELLED` needs its own pre-effect failure distinction — what
      evidence would close it: a decision on whether cancellation and failure need
      different evidence records.

## References

- [`state-machines/change-capsule-transitions.json`](../state-machines/change-capsule-transitions.json)
- [`crates/senvoryn-capsule/src/machine.rs`](../crates/senvoryn-capsule/src/machine.rs)
- [`crates/senvoryn-capsule/tests/state_machine.rs`](../crates/senvoryn-capsule/tests/state_machine.rs)
- Core Contract Specification §5.3, Change Capsule transaction contract
- [ADR-011 — Adding a reason code for the object-resolution gate](ADR-011-resolution-reason-code.md)

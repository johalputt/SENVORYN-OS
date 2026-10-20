# ADR-016 — The Operation Registry's Limits, Its Epoch, and Its Freshness

**Status:** Proposed
**Date:** 2026-10-21
**Deciders:** Project maintainers
**Affected components:** `crates/senvoryn-scp`, `crates/senvoryn-capsule`, `reference/scp_reference.py`, `reference/capsule_reference.py`, `schemas/operation-registry.schema.json`, `examples/operation-registry.sample.json`

---

## Context

- **Problem:** The Operation Registry is the authority on risk, and it is the
  one policy artifact the specification calls *security-critical*. Two things
  about it are undecided, and one of them is a hole:
  1. **Its limits and the capsule's budget are two different numbers that
     mean different things.** The registry's `max_duration_ms` and
     `max_operation_count` sit on an *operation entry*, so they read as
     per-operation limits. The capsule's `budget.duration_ms` and
     `budget.operation_count` sit on the *capsule*, so they read as a total
     for the plan. The executor reconciled the two — the registry bound is
     checked per operation and the capsule's budget covers the plan — and the
     reconciliation was written down in a comment rather than decided
     anywhere. A comment is not a contract.
  2. **The registry carries an epoch that nothing reads.** `registry_epoch` is
     in the schema and in the sample registry, and no implementation in this
     repository parses it. The request's `policy_epoch` is checked against
     the system's epoch at gate 9, and the *registry that assigns the risk*
     is never checked against anything. An operator who advances the system's
     policy epoch while the registry on disk is still the previous one gets
     risk tiers from the previous policy, and every epoch check still passes.
  3. **The registry has no validity window.** It has a version and a
     `signature_ref`, and nothing says when it stops being authoritative. A
     registry with no window is a registry that is authoritative forever, which
     makes the review cadence the specification asks for unenforceable: no
     clock can show that the policy in force was reviewed.

- **Constraint:** The specification's 16-gate order is normative and fixed.
  Gate 5 is *freshness* and it is about the request's own credentials. The
  registry's freshness is a property of the **decision-maker**, not of the
  request, so it does not obviously belong in the request's gate order at
  all. A gate the request cannot influence cannot be one of the request's
  gates.

- **Constraint:** ADR-002 keeps cryptography out of the trusted core, and the
  registry is still unsigned. A signed registry is not this ADR's work; what
  this ADR settles is what a signed registry will have to say.

- **Assumptions:** The registry's `registry_epoch` and the request's
  `policy_epoch` count the same thing — the generation of policy in force.
  The specification uses "policy epoch" for the generation a request is bound
  to, and the registry *is* the policy for operations, so a registry and the
  epoch it belongs to are the same generation. **This is recorded as an
  assumption because the specification does not say it.**

## Decision

**The registry's limits are per operation, and the capsule's budget is a total
for the plan.** Neither number means the other's thing, and the two are never
compared to each other. The reconciliation is four rules, and all four are
implemented and tested in both implementations:

| Rule | Whose number | What it bounds |
|---|---|---|
| `entry.max_duration_ms` | the registry | one invocation of one operation |
| `entry.max_operation_count` | the registry | how many times one operation may appear in one plan |
| `budget.duration_ms` | the capsule | the plan's total, which must cover the sum of the steps' timeouts and must not exceed the sum of the per-invocation allowances of the operations the plan uses |
| `budget.operation_count` | the capsule | the plan's total number of operations |

**The registry in force must be the registry for the epoch in force.** A
decision made under a registry from a different epoch is a decision made
under policy that is not the policy the request was bound to. The SCP will
refuse rather than decide under a mismatched registry, and the refusal will
carry its own reason code, distinct from the request's own epoch mismatch:
the two failures are different and an operator needs to tell them apart.

**The registry must carry a validity window.** `issued_at` and `expires_at`,
in the same strict UTC form the capsules use, and signed with the rest of the
document — a window that can be edited without the signature covering it is
not a window. A registry whose window has closed is not authoritative, and
**the system does not fall back to the last registry it had.** "The last
registry we happened to have" is not a policy; a system that keeps deciding
with a policy it knows is out of date is the failure this whole project
exists to prevent.

**The maximum staleness is a published number.** The registry's window may
not be wider than the system's own maximum, so the claim "this system acts on
policy no older than *N*" is a checkable statement rather than an intention.
The *number* is a deployment parameter, because it depends on the review
cadence, which is an organisational fact rather than a technical one. The
*rule* — the window may not exceed the maximum — is the contract.

**A staged rollout is a property of the fleet, not of a machine.** A machine
has one registry in force, and a request is bound to one epoch. What a staged
rollout must not produce is a machine deciding under two registry documents
at once. The most-restrictive merge that applies to several rules *inside* one
registry does not extend to two registry *documents*, because a request is
bound to exactly one epoch and the merge would let a request be permitted by
the older document and by the newer one at the same time.

## Rationale

**A limit on a per-operation field means the operation.** The registry's
entries describe one operation each: its risk, its reversibility, how long
one invocation may take, how often it may be performed, which modes it is
allowed in. Reading `max_operation_count` as a plan-wide total would mean
that adding a second `read` to a plan changes what the *read* entry allows,
which is not what a per-entry field can mean. The capsule's budget is
different in kind: it is the plan's own claim about itself, and it is checked
against the plan.

**A policy that is not the policy in force is not a policy.** The epoch
mechanism exists so that a request is decided under the policy generation it
was bound to. The risk tier comes from a document, and if that document is
from another generation, the request is bound to one policy and decided under
another. The gate-9 check cannot see this, because the request and the
system agree about the epoch while the registry disagrees with both.

**A registry with no window cannot be reviewed.** The specification requires
the registry to have "provenance, review, staged rollout, and rollback". Three
of those four are properties of a process. The fourth — knowing that the
policy in force is *current* — is a property of a document, and it needs a
date on the document. A registry that never expires makes the review cadence
an unobservable convention, and the project's own ledger already records the
consequence: the maximum stale-authority window is unpublished, so the
local-autonomy claim is `PROPOSED` and cannot be anything else.

**Falling back is the same failure as never expiring.** A system that keeps
using a registry it knows has expired has not fixed the problem; it has moved
the problem from "the policy is old" to "the policy is old and the system
knows it and is using it anyway". The failure mode of a fail-closed system is
that it stops; the failure mode of a fail-open one is that it keeps deciding,
and the second is worse, because it is silent.

**The maximum staleness is the checkable version of the review cadence.** The
cadence itself is a human process. What a system can enforce is the bound:
no registry the system will accept may be older than the maximum, and the
maximum is published so the claim can be read rather than believed.

## Consequences

**What this makes easy:**
- The reconciliation between the registry and the capsule is written down
  once, in a table, and each row is a rule with a test and a mutation.
- The registry's epoch stops being decoration.
- The review cadence becomes enforceable, and the "local autonomy" claim in
  the ledger can eventually move from `PROPOSED` to `TESTED` with a number
  attached to it.

**What this makes hard:**
- The registry schema changes, and so does the sample registry. Both
  implementations must learn the new fields, and the Rust and the Python
  timestamp parsers must agree on them — the capsule's strict UTC parser
  lives in the capsule crate, and the SCP cannot depend on it, so the parser
  has to move somewhere both can reach.
- A reason code has to be added for each refusal, and the denial-code
  registry, the Rust constants, the Python set and the two registry-sync
  tests all have to move together.

**What this costs:**
- This ADR; the implementation is the next batch.

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| The registry's `max_operation_count` is a plan-wide total | It sits on an operation entry. A field that describes one operation cannot describe every operation in a plan, and the plan's total already has a field: `budget.operation_count`. |
| Compare the registry's limits with the capsule's budget directly | They are different quantities. A per-invocation bound and a plan's total are not comparable, and comparing them would either reject every multi-step plan or permit every one. |
| Leave the registry's epoch unread, as it is | It is a field in the schema that names a generation. A generation that nothing compares to is a comment. |
| Compare the registry's epoch with the *request's* `policy_epoch` | The request is not the system. The request's epoch is checked at gate 9 against the system's epoch; checking the registry against the request would let a request decide which policy it is judged under. The registry belongs to the system, so the registry is compared with the system's epoch. |
| The registry's freshness is gate 5 | Gate 5 is the request's own freshness, and it is early in a fixed order that the request's properties drive. A property of the decision-maker cannot be one of the request's gates, and adding a seventeenth gate would change a normative order the specification fixes. |
| The system keeps the last registry it had when the new one is late | "The last registry we happened to have" is not a policy, and the system knows the window closed. A fail-open fallback is the same failure as no window at all, with an extra step. |
| The maximum staleness is fixed by the specification | The number depends on how often policy is reviewed, which is an organisational fact. A fixed number in the specification would be a number nobody updates, and it would be wrong within a year. The rule is in the contract; the number is a deployment parameter. |
| The registry's window is unsigned, like the rest of the signature | Then anyone who can write the registry file can extend the window, and the window stops being a bound. The window is covered by the signature or it does not exist. |
| A machine may hold two registries during a staged rollout and decide under the most restrictive merge | A request is bound to one epoch. Merging two documents would let a request be permitted by one and refused by the other, and the refusal would be invisible. The rollout is staged across the fleet, not inside a machine. |

## Open items

- [ ] Where the registry's epoch binding and its validity window sit in the
      implementation — a check inside the decision path, or a refusal to load
      a registry that cannot be used. What evidence would close it: the
      decision path stays pure, so the check needs the clock as a parameter
      it already receives, and a vector for each refusal.
- [ ] The reason codes for a mismatched registry epoch and for an expired
      registry. What evidence would close it: a decision, and the
      registry-sync tests in both directions.
- [ ] Where the strict UTC timestamp parser lives so that the SCP and the
      capsule can both use it without depending on each other. What evidence
      would close it: a move into a crate both depend on, with the capsule's
      timestamp boundary tests running against it.
- [ ] The boot-epoch strategy that makes the registry's wall-clock window
      comparable with the monotonic clock the policy layer works in. This is
      the same open item the capability's own validity window has.
- [ ] The maximum staleness, and the review cadence that sets it. What
      evidence would close it: a published number and the deployment
      configuration that carries it.

## References

- [`schemas/operation-registry.schema.json`](../schemas/operation-registry.schema.json)
- [`examples/operation-registry.sample.json`](../examples/operation-registry.sample.json)
- [`crates/senvoryn-capsule/tests/validation.rs`](../crates/senvoryn-capsule/tests/validation.rs)
- [`reference/capsule_reference.py`](../reference/capsule_reference.py)
- [ADR-002 — the cryptographic boundary](ADR-002-crypto-boundary.md)
- [ADR-015 — the capsule's validity window is a boundary](ADR-015-capsule-validity-window-and-a-moving-world.md)

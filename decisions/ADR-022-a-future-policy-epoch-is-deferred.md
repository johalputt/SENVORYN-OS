# ADR-022 — A Future Policy Epoch Is Deferred, Not Denied

**Status:** Proposed
**Date:** 2026-10-22
**Deciders:** Project maintainers
**Affected components:** `crates/senvoryn-scp/src/evaluator.rs`, `crates/senvoryn-scp/src/decision.rs`, `crates/senvoryn-scp/src/reason.rs`, `crates/senvoryn-scp/src/registry.rs`, `errors/denial-codes.json`, `reference/scp_reference.py`, `conformance/vectors/vectors.json`, `conformance/capsule-vectors.json`, `scripts/mutations.json`

---

## Context

- **Problem:** the decision vocabulary has five statuses and the gates could
  produce four of them. `DEFER` was in the specification (§3.3), in the
  schema, in the record reader, in the capsule executor's refusal of anything
  that grants no authority, and in the `DecisionStatus` enum — and **no rule
  anywhere produced it**. A status that no rule reaches is a hole in the
  machine: the type says the outcome exists and nothing can produce it.

- **The one deferral the specification describes.** §3.3: "`DEFER`: operation
  is non-destructive and waits for a known dependency or policy window."
  The policy window is the only waiting-for in the policy layer. A request
  that names a **future** policy epoch is exactly that: the policy that will
  govern it does not exist yet, so the answer is neither yes nor no.

- **What the gate did instead.** Gate 9 refused every epoch that was not the
  node's own: `request.policy_epoch != context.current_policy_epoch` →
  `DENY` with `POLICY_EPOCH_MISMATCH`. That is right for a **stale** epoch —
  that policy is gone and will not come back — and wrong for a **future**
  one. A node that has not yet reached epoch 8 was refusing, as a permanent
  "no", a request that will be decidable when it does.

- **Constraint:** §3.4 says an unknown or stale policy epoch for a
  privileged action is `DENY` or `CHALLENGE`, and "never silent permit".
  A deferral is neither of those, so it has to earn its place against both
  sections: §3.3 qualifies `DEFER` with "operation is non-destructive", and
  §3.4 forbids postponing a privileged action.

- **Constraint:** a deferral has to expire, or it is a decision that never
  arrives. Nothing else bounds one.

- **Constraint:** the registry for the future epoch **does not exist** — that
  is why the request is deferred at all. The registry in force is the only
  policy this node holds, and it is the only one it may consult.

- **Constraint:** the SCP is the only authority on whether a capsule may run
  (ADR-014, ADR-021). The deferral therefore has to be a decision the policy
  layer makes, written into the decision record, and refused by the executor
  like any other decision that grants no authority.

## Decision

**A request that names a future policy epoch is deferred. A request that
names a stale policy epoch is denied. Only a non-destructive operation may be
deferred.**

- At the policy-epoch gate, the request's epoch is compared with the node's:
  - **greater** — the request names a policy this node has not reached yet.
    The decision is `DEFER` with the reason code
    `POLICY_EPOCH_NOT_YET_IN_FORCE`, **provided** the operation is
    non-destructive. Otherwise it is `DENY` with `POLICY_EPOCH_MISMATCH`.
  - **less** — the request names a policy that is gone. The decision is
    `DENY` with `POLICY_EPOCH_MISMATCH`, exactly as before.

- **Non-destructive** means the registry in force declares the operation
  `REVERSIBLE` **and** gives it no external effects. An operation the
  registry does not know is not shown to be non-destructive, and is denied.
  `UNKNOWN` reversibility is the worst case by construction.

- **The deferral names what it waits for.** The decision carries the epoch it
  waits for, and the decision record publishes it in `required_next_step`.
  The decision's own deferral supplies that field: it is not supplied by the
  caller of `to_record`, because a value the caller assembles is a value the
  caller can disagree with (ADR-021's rule, applied to the next step rather
  than to the plan).

- **The deferral is bounded by the request.** The freshness gate runs before
  the epoch gate, and it refuses a request that has expired. A deferred
  request is therefore refused the moment it expires, and the deferral cannot
  outlive the request it defers. No new state is needed to say so.

- **The deferral grants nothing.** It carries no capability scope, and
  `grants_authority` is false for every status but `PERMIT`. The capsule
  executor refuses a deferred decision like any other decision that grants no
  authority.

- **The registry's own consistency is settled first.** The registry-epoch
  binding, the clock, and the registry's validity window are checked before
  the request's epoch, because §3.2 gate 9 says no decision may be made under
  a registry whose window has not opened or has closed — and a deferral *is* a
  decision: it produces a record and an audit event. A node whose policy in
  force is not in force decides nothing at all, not even a postponement.

## Rationale

**Why not deny the future epoch, as the gate did?** Because a denial is a
permanent answer and "not yet" is not one. The caller of a node that has not
reached epoch 8 learns something different from a caller of a node that has
passed epoch 7: the first will be answered when the node catches up, the
second never will. One reason code cannot carry both, and the project already
distinguishes the same pair elsewhere — `REGISTRY_NOT_YET_VALID` against
`REGISTRY_EXPIRED`, `RESOURCE_UNRESOLVABLE` against
`RESOURCE_TYPE_MISMATCH`.

**Why "non-destructive" and not "anything"?** §3.3 qualifies `DEFER` with
"operation is non-destructive", and §3.4 says an epoch this node does not
accept is `DENY` or `CHALLENGE` for a privileged action. A deferral is a
promise of a *later answer*, and it is a promise this project may only make
about an operation whose effect can be undone. Both halves of the condition
are load-bearing, and each is refused by a case that only it can refuse: an
operation that is `NON_REVERSIBLE` with no external effects is refused by the
reversibility check alone, and an operation the registry declares
`REVERSIBLE` while giving it an external effect is refused by the external
effects check alone. Without the second case the first control would never
be seen to fire.

**Why the registry in force, and not the registry of the future epoch?** The
registry of the future epoch does not exist — that is the reason for the
deferral. The registry in force is the only policy the node holds and the
only one it may consult. An operation it does not mention is not shown to be
non-destructive, and "not shown" is not "shown".

**Why is an external effect decisive?** An operation that reaches outside the
node is not undone by restoring local state. Whatever the registry says about
reversibility, a registry entry that claims an operation is reversible while
giving it an external effect is contradicting itself, and the policy layer
does not take the contradictory half of a contradiction on trust.

**Why is the bound the request's, and not a new state?** A deferral that
outlived its request would be a decision that never arrives and never
expires either. The request already carries a validity window, and the
freshness gate already enforces it, so the bound is inherited rather than
invented. The one thing the deferral cannot do is publish its own expiry as
a date-time: the request's expiry is a monotonic instant, and the boot-epoch
bridge that would convert it does not exist (ADR-017's open item). That is a
real gap and it is recorded below rather than papered over with an invented
conversion.

**Why does the registry's own consistency come first?** Because §3.2 says no
decision may be made under a registry whose window has not opened or has
closed. A deferral is a decision — it produces a decision record and an audit
event — so it may not be made under a policy that is not in force either.
This is a reorder *within* gate 9, and every outcome it changes is
non-permissive, so no privileged effect can leak from the change.

## Consequences

**What this makes easy:**
- A caller can tell "not yet" from "never", and can act on it: retry when
  the epoch is in force.
- The `DEFER` status in the specification, the schema and the enum becomes
  a status the gates can actually produce, which is what makes the rest of
  those declarations true rather than decorative.
- The deferral is a decision like any other: it is recorded, audited, and
  refused by the capsule executor.

**What this makes hard:**
- A rule with two conditions needs two cases, one for each. The second case
  is a registry entry that contradicts itself, and it exists because a
  control that can never fire is not a control.
- The request's own expiry bounds the deferral but cannot be published in
  the record, because the monotonic-to-wall-clock bridge does not exist.
- The record's `required_next_step` now comes from the decision alone, and
  the caller of `to_record` can no longer supply one. Nothing was lost
  today — every call site passed nothing — but a `CHALLENGE` publishes no
  next step at all, because the decision does not yet say what a challenge
  asks for. The reason code names the missing evidence, and the record
  carries it.

**What this costs:**
- The Python reference model gains two registry fields it did not have
  (`reversibility` and `external_effects`), so that the two implementations
  can agree about which operations may be deferred.

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Defer every request under a future epoch, whatever the operation | §3.3 qualifies `DEFER` with "operation is non-destructive", and §3.4 forbids postponing a privileged action. A deferral is a promise of a later answer, and this project may only make it about an operation whose effect can be undone. |
| Keep denying the future epoch and add a distinct reason code only | A different code with the same status changes nothing for the caller: the answer would still be a permanent "no" for a request that will be decidable later. The status is the part that carries "not yet". |
| Defer on the *capability's* epoch as well as the request's | It would be pointless. A capability naming a future epoch means the node has not reached it; by the time it has, a request naming the node's current epoch is stale and is denied by the same gate. The capability's own epoch check stays a denial. |
| Give the deferral its own expiry field in the decision record | The request's expiry is a monotonic instant. Publishing it as a date-time needs the boot-epoch bridge, which does not exist. Inventing a conversion here would build a second, wrong clock. |
| Let the deferral ignore the request's freshness | The freshness gate already refuses an expired request, and it runs first. A second copy of that rule in the deferral is the second thing this project has learned to stop doing. |
| Put the request's epoch check before the registry's own consistency | §3.2 gate 9 says no decision may be made under a registry whose window has not opened or has closed, and a deferral is a decision. |
| Add a `QUARANTINE` constructor so the capsule harness could write one | No gate produces a quarantine, so there is no decision to write. The harness refuses instead of writing a denial under that name — a case that asked for a quarantine and silently received a denial would prove nothing about either. |

## Open items

- [ ] The boot-epoch bridge does not exist, so the deferral's expiry cannot be
      published as a date-time and the record's `valid_until` stays empty for
      a deferral. — Closed by the boot-epoch strategy (ADR-017's open item),
      after which the deferral can carry its own bound in the record.
- [ ] A deferral reserves nothing: when the epoch arrives the request is
      decided from the top, against the registry of that epoch, with every
      gate run again. Nothing in the record says that to the caller, and the
      `required_next_step` names the epoch rather than promising a result.
      — Closed by the transport that tells a caller what to do with a
      deferred decision.
- [ ] `QUARANTINE` is the other status in the vocabulary that no rule
      produces. It is the same kind of hole this decision closes, and it is
      still open.
- [ ] The registry's own per-entry consistency is not checked. A registry
      entry that contradicts itself is refused here only where a rule reads
      both halves. — Closed by a registry validator that rejects a
      contradictory entry when the registry is read.
- [ ] A `CHALLENGE` publishes no next step. The record's next step is the
      decision's own, and the decision does not yet say what a challenge
      asks for. — Closed with the decision's own vocabulary of next steps.

## References

- [`crates/senvoryn-scp/src/evaluator.rs`](../crates/senvoryn-scp/src/evaluator.rs)
- [`crates/senvoryn-scp/src/decision.rs`](../crates/senvoryn-scp/src/decision.rs)
- [`crates/senvoryn-scp/src/reason.rs`](../crates/senvoryn-scp/src/reason.rs)
- [`errors/denial-codes.json`](../errors/denial-codes.json)
- [`reference/scp_reference.py`](../reference/scp_reference.py)
- [`conformance/vectors/vectors.json`](../conformance/vectors/vectors.json)
- [ADR-021 — the decision record is the authorization](ADR-021-the-decision-record-is-the-authorization.md)
- [ADR-017 — the clock is a boundary](ADR-017-the-clock-is-a-boundary.md)
- [ADR-016 — the registry's limits, epoch and freshness](ADR-016-operation-registry-limits-epoch-and-freshness.md)
- [ADR-014 — the capsule has two implementations](ADR-014-capsule-two-implementations.md)
- Specification §3.2 gate 9, §3.3 (decision statuses), §3.4 (decision failures)

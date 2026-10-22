# ADR-023 — A Quarantined Principal Is Quarantined, Not Merely Denied

**Status:** Proposed
**Date:** 2026-10-22
**Deciders:** Project maintainers
**Affected components:** `crates/senvoryn-scp/src/evaluator.rs`, `crates/senvoryn-scp/src/decision.rs`, `crates/senvoryn-scp/src/reason.rs`, `crates/senvoryn-scp/src/context.rs`, `crates/senvoryn-cli/src/main.rs`, `errors/denial-codes.json`, `reference/scp_reference.py`, `conformance/vectors/vectors.json`, `conformance/capsule-vectors.json`, `scripts/mutations.json`

---

## Context

- **Problem:** `QUARANTINE` was the last status in the decision vocabulary that
  no rule produced. ADR-022 gave `DEFER` a rule; this gives `QUARANTINE` one.
  The status was in the specification (§3.3), in the schema, in the
  `DecisionStatus` enum and in the denial-code registry's vocabulary of
  statuses, and the capsule harness had to *panic* rather than write a denial
  under that name, because there was no decision to write.

- **What the specification says.** §3.3: "`QUARANTINE`: a defined suspicious
  principal/request is isolated by policy; quarantine is not a generic penalty
  for all errors." §7 names the state: "`QUARANTINED`: the affected
  principal/device/component is constrained after a policy-defined trigger."
  And §4: "A capability does not override a read-only, recovery, quarantined
  or policy-revoked state."

- **Three requirements, and they pull in different directions.** The subject of
  a quarantine is *defined* — a named principal, not a heuristic. It is
  isolated *by policy* — the policy decides, the code does not invent. And it
  is **not a generic penalty for all errors** — which is the requirement that
  rules out the obvious implementation, where any failure at any gate is
  reported as a quarantine because a quarantine sounds stronger than a denial.

- **Constraint:** the evaluator is pure. It returns a decision and performs no
  effect (§3.2 gate 16). A quarantine therefore cannot be *applied* by the
  policy layer; it can only be *reported*. Whatever the decision says, the
  isolation is something the environment holds, and the environment is the
  only thing that can lift it.

- **Constraint:** the subject of the quarantine has to be somebody the node
  can still recognise. A node that has forgotten a principal cannot tell it
  that it is quarantined, and a subject that is not told cannot respond.

## Decision

**A request from a principal that this node holds under quarantine is
`QUARANTINE` with the reason code `PRINCIPAL_QUARANTINED`. Every other
failure stays a denial or a challenge.**

- The environment states the principals it holds under quarantine, in the same
  way it states the principals it knows, the nonces it has seen, and the
  capabilities it has revoked: as a set of identifiers in the policy context.
  A caller is never the authority on its own state.

- **The check is at the identity gate, immediately after the known-principal
  check.** The isolation is a fact about the subject, and it is settled
  before anything else about the request can be learned: a quarantined
  principal is told that it is quarantined and nothing else. The resource is
  not resolved, the capability is not examined, and the request's own
  freshness is not judged, because none of those answers would be given to a
  subject that is already held apart.

- **A quarantined principal is still a known principal.** It is not removed
  from the known set the way a revoked principal is. This is the difference
  between the two, and it is the difference the specification draws: a
  revoked principal's requests are refused as unknown, because the node has
  let the subject go; a quarantined principal's requests are refused as
  quarantined, because the node still knows who is asking and is holding them
  apart. The subject learns the difference, and only the subject can act on
  it.

- **The quarantine names the principal it holds.** The decision carries the
  isolated principal, and the decision record publishes it in
  `required_next_step`: the isolation is lifted by a policy decision, and the
  record says which principal it is about. As with the deferral, the next step
  comes from the decision itself and not from the caller that writes the
  record.

- **The quarantine grants nothing.** It carries no capability scope, and
  `grants_authority` is false for every status but `PERMIT`. The capsule
  executor refuses a quarantined decision like any other decision that grants
  no authority, and a capability cannot override it: the identity gate runs
  before the capability gate.

- **No check falls back to a quarantine.** A malformed request is malformed, an
  expired request is expired, an unknown operation is unknown. The quarantine
  is reachable only for a subject the environment holds under quarantine, which
  is what §3.3's "not a generic penalty for all errors" asks for.

## Rationale

**Why the principal and not the request?** §3.3 says "principal/request", and
the specification's quarantine is a *state*: §7 puts `QUARANTINED` beside
the node's other constrained states, and §4 lists "node quarantine" among the
fleet policy's powers. A request is transient; a principal is not. A node can
hold a principal apart and let it go, and only a subject-level state can be
lifted by a later policy decision. A request-level quarantine would be the
same as a denial with a different word on it, because there is nothing left
after the request is refused.

**Why at the identity gate and not the revocation gate?** The revocation gate
is gate 7, and a quarantine is the sibling of a revocation — both are the
policy cutting a subject off. But a *principal's* revocation is already
expressed at the identity gate: the principal is removed from the known set,
and the request is refused as unknown. Putting the quarantine beside it, at
the same gate, keeps the two facts about a subject in one place. It also
settles the isolation before the resolution gate, so a quarantined principal
learns nothing about the resource it asked for — the smallest disclosure that
still tells it what is wrong.

**Why is the principal still known?** Because a subject that is not told
cannot respond, and because the specification's two states are different. A
revocation is the node letting a subject go: the principal is gone, and the
node no longer acknowledges it. A quarantine is the node holding a subject
apart: the principal is known, and the refusal says why. Making the
quarantined principal unknown would have been a smaller change — one set
instead of two — and it would have turned the quarantine into a denial with
the disclosure removed, which is the opposite of what the status is for.

**Why does the isolation not expire with the request?** Because it is not the
request's. A deferral is a promise about a request, and it is bounded by the
request. A quarantine is a statement about a subject, and it lasts until the
policy lifts it. The decision is a statement made now about a subject that
the environment holds; the environment's set is what changes, exactly as it
is what changes when a revocation is lifted. The record says which principal
it is about and what would lift it, and the decision itself expires with
nothing — it is not a grant, and there is nothing in it to expire.

**Why is there no trigger modelled?** Because the same is true of a revocation.
The environment holds that a capability has been revoked; it does not hold what
made it so, and the specification's revocation trigger is not modelled either.
The quarantine's trigger — the policy-defined event that put the principal
apart — is the same kind of gap, and it is recorded below rather than
invented here. Inventing one would have meant writing a heuristic for
"suspicious", and a heuristic that decides who is suspicious is precisely the
generic penalty the specification forbids.

## Consequences

**What this makes easy:**
- The `QUARANTINE` status becomes a status the gates can produce, which is
  what makes the specification's §3.3, the schema and the enum true rather
  than decorative.
- A quarantined principal is told what is wrong with it, which is the only
  thing that lets it ask for the isolation to be lifted.
- The node's two ways of cutting a subject off are now distinguishable:
  unknown and revoked on one side, quarantined on the other.

**What this makes hard:**
- A rule needs a case for each thing it must *not* do. A quarantined
  principal's request is refused as quarantined, and an **unknown** principal
  is still refused as unknown — a quarantine is not what a node says about
  somebody it has never heard of, and the case that proves it is a case in
  which the two would otherwise be indistinguishable.
- The context grows a set, and the Python reference model grows the same one,
  so that the two implementations can agree about who is held apart.
- The record's `required_next_step` now has two sources — a deferral's and a
  quarantine's — and a decision may carry only one of them. That is a
  property of the constructors rather than of the record writer, because the
  fields are private.

**What this costs:**
- One more reason code in the registry (thirty-four), and one more status
  that the registry test has to know about.

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Report a quarantine whenever the node's system mode is `QUARANTINED` | The mode is the *node's* state, and §3.3's subject is the principal or the request. A mode-based rule would also tell every caller that the node is isolated, which is a disclosure about the node made to a caller who has not earned it. |
| Let the quarantine be the outcome of any failed check | §3.3 forbids it in as many words: "quarantine is not a generic penalty for all errors". A status that is the fallback for every failure carries no information at all. |
| Remove a quarantined principal from the known set, as a revocation does | Then the node would answer `UNKNOWN_PRINCIPAL` and the subject would never learn that it was quarantined rather than revoked. The two states are different, and the difference is the reason the subject is told. |
| Quarantine the request rather than the principal | A request is transient. There would be nothing left to hold apart after the refusal, so the status would carry no consequence beyond the refusal itself. |
| Give the quarantine a time bound in the record, like the deferral's | The isolation is lifted by a policy decision, not by a clock, and a bound the record states would be a bound the node does not hold. The environment's set is the bound, exactly as it is for a revocation. |
| Model the trigger — decide for ourselves what is suspicious | A heuristic that decides who is suspicious is the generic penalty the specification forbids, and it would put a judgement about intent in the policy layer. The environment's set is the policy's own statement about the subject. |
| Add a `QUARANTINE` status to the capsule harness now that a rule produces one | The capsule executor already refuses any decision that grants no authority, and a quarantine grants none. A capsule case that shows it refuses one is worth having; a new status in the executor is not. |

## Open items

- [ ] What put the principal under quarantine is not modelled. The
      environment holds the state, as it holds a revocation, and the
      policy-defined trigger that created it is outside the decision path.
      — Closed with the policy event that creates the isolation, and the
      audit event that records it.
- [ ] The isolation is not applied by anything. The decision reports it; the
      environment holds it. No component in this repository constrains a
      quarantined principal, because no component holds state.
      — Closed by the node's subject state, with the isolation applied and
      lifted there.
- [ ] A `CHALLENGE` publishes no next step, and now a quarantine and a
      deferral each publish one. The decision's own vocabulary of next steps
      is still incomplete. — Closed with that vocabulary.
- [ ] The registry's own per-entry consistency is not checked, and a
      registry entry that contradicts itself is refused only where a rule
      reads both halves. — Closed by a registry validator that rejects a
      contradictory entry when the registry is read.

## References

- [`crates/senvoryn-scp/src/evaluator.rs`](../crates/senvoryn-scp/src/evaluator.rs)
- [`crates/senvoryn-scp/src/decision.rs`](../crates/senvoryn-scp/src/decision.rs)
- [`crates/senvoryn-scp/src/context.rs`](../crates/senvoryn-scp/src/context.rs)
- [`errors/denial-codes.json`](../errors/denial-codes.json)
- [`reference/scp_reference.py`](../reference/scp_reference.py)
- [`conformance/vectors/vectors.json`](../conformance/vectors/vectors.json)
- [ADR-022 — a future policy epoch is deferred, not denied](ADR-022-a-future-policy-epoch-is-deferred.md)
- [ADR-021 — the decision record is the authorization](ADR-021-the-decision-record-is-the-authorization.md)
- [ADR-014 — the capsule has two implementations](ADR-014-capsule-two-implementations.md)
- Specification §3.2 gate 4, §3.3 (decision statuses), §3.4 (decision failures), §4 (modes), §7 (quarantined state)

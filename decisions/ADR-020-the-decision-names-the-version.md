# ADR-020 — The Decision Names the Version

**Status:** Proposed
**Date:** 2026-10-21
**Deciders:** Project maintainers
**Affected components:** `crates/senvoryn-capsule/src/authorization.rs`, `reference/capsule_reference.py`, `conformance/capsule-vectors.json`

---

## Context

- **Problem:** Since ADR-018 the decision record has carried the version the
  decision was made against, "so that the executor's recheck has an anchor when
  it exists". Nothing rechecks it. The executor made three checks and none of
  them asked the question the anchor was carried for:

  1. the **digest binding** — the decision permits this exact plan;
  2. the **environment check** — the target is still at the version the
     capsule expects;
  3. the **approvals** — the capsule's required approvals were given.

  A decision made against version 6 of a target, and a capsule that expects
  version 7, pass all three: the plan is the same plan, the world is where the
  capsule says it is, and the approvals are in order. The effect is then
  applied to version 7 under a decision that judged version 6 — the risk
  assessment, the capability scope and the consent requirement were all
  decided against a state of the object that is no longer the state it is in.
  That is the time-of-check/time-of-use race the threat model names as AS-19:
  "the artefact that authorises an effect names the canonical identifier of the
  object and the version the decision was made against, so that the recheck has
  something to compare against that the caller cannot change."

- **Also found:** `Authorization::from_value` read a document — a decision
  record's "authorization section" — that no schema in the repository defines
  and that no producer writes. Nothing called it. A reader of a document nobody
  produces is not a boundary; it is a shape that looks like one. It has been
  deleted, and the gap it papered over is an open item below.

- **Constraint:** the SCP is the only authority on whether a capsule may run.
  The executor must not re-decide policy. What the executor does is check that
  the decision it was given binds the capsule it is about to run — the digest
  binding is already such a check, and this is the same kind of check.

- **Constraint:** the capsule's `target.target_id` is the capsule's own name
  for its target and may be an alias. The decision's `target_resource_id` is
  the canonical identifier. Comparing the two would refuse every alias, and
  aliases are exactly what ADR-018 made usable. The version is the part both
  sides can speak about: it is a property of the object, not a name for it.

## Decision

**The authorization names the version of the target the decision was made
against, and the executor refuses a decision that names none or names a
different one.**

- `Authorization` carries `permitted_resource_version: Option<String>` — the
  version the decision was made against. It is `None` exactly when the
  decision was made before the resource was resolved, which is the case the
  decision record already reports as a null `target_resource_version`.
- The check runs **after** the digest binding and **before** the approvals.
  The digest is the one that cannot be worked around, so it goes first; the
  version is about the state the plan will act on, so it goes before the
  approvals, which are about the plan and not the state.
- **A decision that names no version authorises nothing.** `None` is a
  refusal, not a wildcard. A decision made before the resource was resolved
  was made about a name, not about an object, and it says nothing about any
  version of any object.
- **A decision made against a different version authorises nothing.** The
  version in the decision and the version the capsule expects must be the
  same.
- **The canonical identifier is not compared.** The version is the comparable
  part; the identifier is a name on one side and a canonical name on the
  other.

## Rationale

**The digest binds the plan; the version binds the state the plan was judged
against.** A capsule is a plan; a decision is a judgement about applying that
plan to an object in a particular state. The digest check answers "is this the
plan I judged?" and the version check answers "was the object in the state I
judged it in?". Without the second answer the first is not enough: the same
plan against a different version is a different effect, and it may need a
different approval.

**A refusal is not a wildcard.** The decision record's `target_resource_version`
is null in one case only — the decision was made before the resource was
resolved, which means it was refused before it judged anything. Treating that
null as "any version" would turn the most uncertain decision in the system into
the most permissive one.

**The version is comparable; the identifier is not.** ADR-018 made the
canonical identifier the identity that gates 10 and 11 are keyed by, and it
made aliases usable by resolving them. The capsule's own target name is a
name. Comparing a name with a canonical identifier would refuse every alias
and would compare two things that are not the same kind of thing. The version
is a property of the object either name refers to, which is why it is the
field to compare.

## Consequences

**What this makes easy:**

- The decision record's resolved version has a consumer, and the anchor
  ADR-018 carried it for is now used.
- The executor's authorisation is a claim about a specific state of a specific
  object, which is what an operator reading the audit trail needs it to be.

**What this makes hard:**

- `Authorization` changed shape, so both implementations and both vector
  harnesses changed with it.
- Every existing vector's authorization now names a version. A harness that
  builds one without naming the version is building a decision that authorises
  nothing, and the vectors say so.

**What is implemented, and what is not:**

| Part | State | Where |
|---|---|---|
| The version in the authorization | done | `Authorization::permitted_resource_version`, both implementations |
| The refusal of a decision that names no version | done | `AuthorizationError::TargetVersionNotNamed`, shared vector `CC-038` |
| The refusal of a decision made against another version | done | `AuthorizationError::TargetVersionMismatch`, shared vector `CC-037` |
| The authorization read from a document the repository defines | done | [ADR-021](ADR-021-the-decision-record-is-the-authorization.md): the executor reads the decision record |
| The plan digest in the decision record | done | [ADR-021](ADR-021-the-decision-record-is-the-authorization.md): `permitted_payload_digest` |
| The approvals in the decision record | **not done** | the record carries none; the policy layer is given the approval's digest |

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Compare the canonical identifier as well as the version | The capsule's target name may be an alias and the decision's is canonical. Comparing them refuses every alias, and the two are not even the same kind of thing. |
| Treat a decision that names no version as authorising any version | That is a wildcard on the one field the decision exists to pin, and it would make the least-informed decision the most permissive one. |
| Put the check in the executor's validation, next to the environment's version check | The environment's check asks whether the target is still where the capsule expects it; this asks whether the decision was made about that version. Two questions, two answers — and putting them together would make one of them unreachable, which is how a control stops being a control. |
| Let the executor read the decision record and take the version from it | At the time of this decision the record carried neither the plan digest nor the approvals, so an executor that read the record could not do the digest binding it already does. Reading the record needed the record to carry the plan first, which is what [ADR-021](ADR-021-the-decision-record-is-the-authorization.md) added. |
| Re-resolve the target in the executor and compare the resolved version | The executor drives a simulated environment and resolves nothing, and re-resolving would make it a second authority on identity — the failure the two-implementation vectors exist to prevent. |
| Compare the version only when the decision names one | The comparison is then optional, and the case where the decision names none is exactly the case where the decision knows least. |

## Open items

- [x] The authorization was a value the host constructs. It is now read from
      the decision record — a document the repository defines and the policy
      layer produces. Closed by
      [ADR-021](ADR-021-the-decision-record-is-the-authorization.md).
- [x] The decision record carried neither the plan digest nor the approvals.
      It now carries the plan digest. Closed by
      [ADR-021](ADR-021-the-decision-record-is-the-authorization.md).
- [ ] The approvals are still passed to the executor beside the record. The
      policy layer is given the approval's digest, not the approval, so the
      record cannot carry them. Tracked in ADR-021.
- [ ] The request carries no target version (ADR-018's open item), so the
      decision's version comes from the resolution and the capsule's expected
      version comes from the capsule. Nothing yet ties the request to the
      plan: the plan digest is that tie, and ADR-021 adds it to the request
      and the record.

## References

- [`crates/senvoryn-capsule/src/authorization.rs`](../crates/senvoryn-capsule/src/authorization.rs)
- [`reference/capsule_reference.py`](../reference/capsule_reference.py)
- [`conformance/capsule-vectors.json`](../conformance/capsule-vectors.json)
- [`docs/threat-model.md`](../docs/threat-model.md) — AS-19
- [ADR-018 — the resolution boundary](ADR-018-the-resolution-boundary.md)
- [ADR-015 — the capsule's validity window and a moving world](ADR-015-capsule-validity-window-and-a-moving-world.md)

# ADR-021 — The Decision Record Is the Authorization

**Status:** Proposed
**Date:** 2026-10-21
**Deciders:** Project maintainers
**Affected components:** `crates/senvoryn-scp/src/record.rs`, `crates/senvoryn-scp/src/decision.rs`, `crates/senvoryn-scp/src/request.rs`, `crates/senvoryn-capsule/src/authorization.rs`, `reference/capsule_reference.py`, `schemas/decision-record.schema.json`, `conformance/capsule-vectors.json`

---

## Context

- **Problem:** the capsule executor was handed an authorization — the digest
  the decision permits, the version it was made against, the approvals — that
  **the host assembled**. Nothing tied the digest in it to anything the policy
  layer had decided. A host that passed the wrong value passed a value the
  executor would compare against the capsule and find wanting, or — worse — a
  value the host chose to match. The executor checked the binding; it did not
  check that the thing it was binding was the policy layer's decision.

- **The anchor was already there.** ADR-018 made the decision record carry
  the resolved version "so that the executor's recheck has an anchor when it
  exists", and ADR-020 uses the anchor: a decision that names no version, or
  names another, authorises nothing. But the anchor still arrived inside a
  host-built value, so what ADR-020 checked was a value's agreement with the
  capsule, not the record's agreement with the capsule.

- **The plan was not in the record at all.** The wire envelope carries
  `payload_digest` — the digest of the payload the request asks to run — and
  the policy request form did not (ADR-019's open list: "the
  `approval_digest` vs `approval_refs`/`payload_digest`"). So the thread
  from a request to a decision to a plan did not exist: the decision could
  not say which plan it was about, and the executor could not ask.

- **Also found:** `Authorization::from_value` read a decision record's
  "authorization section" — a document shape no schema defines and no
  producer writes. It was deleted in ADR-020. What it was pretending to be
  is a reader of a document the repository does not produce, and the gap it
  hid is the one this decision closes.

- **Constraint:** the SCP is the only authority on whether a capsule may run.
  The executor does not re-decide policy. What the executor must do is read
  the decision the policy layer made and check that it binds the plan it was
  handed.

- **Constraint:** no cryptographic digest is computed in the trusted core
  (ADR-002). The digest in the record is the **caller's claim**, carried and
  compared — not computed. The record is trusted because it comes from the
  policy layer inside the trust boundary, not because it is signed.

## Decision

**The executor reads the decision record, and the record carries the plan the
request asked to run.**

- The **policy request** may name a plan: `payload_digest`, optional, the
  same name the wire envelope already uses. A request that names no plan is
  a request for a single operation, and the field is null.
- The **decision record** carries `permitted_payload_digest`, the plan the
  request named. It is null when the request named none, and it is null for
  a decision that was refused before anything was judged.
- The **decision-record reader** (`senvoryn_scp::record`) is **closed**: a
  record carrying a field the record does not define is refused rather than
  read, for the reason ADR-019 gives for the request form — a field the
  reader does not account for may be a field a newer writer expects the
  reader to enforce. The reader also refuses a record that claims an effect
  was performed: the record's schema fixes `effect_performed` to false, and
  the policy layer cannot have performed one.
- The **executor** takes the plan, the version and the status from the
  record. `Authorization::from_record` reads them out; the check compares
  them with the capsule.
- **A decision that granted no authority authorises nothing.** The status is
  read from the record and checked by the executor, not left to the host to
  have remembered.
- **A decision that names no plan authorises no plan.** `None` is a refusal,
  not a wildcard — the same rule ADR-020 states for the version.
- The **approvals** are still passed in beside the record. The policy layer is
  given the approval's *digest*, not the approval, so the record cannot carry
  the approvals. That is the last part of the authorization the host
  assembles, and it is an open item below.

## Rationale

**The value the host assembles is the value the host can get wrong.** Every
check the executor made was a check of an internally consistent pair — the
host's digest against the capsule's digest. Consistency is not authority. The
authority is the policy layer's decision, and the only way to bind the
executor to it is to hand the executor the thing the policy layer wrote.

**A refusal that is a wildcard inverts the system's caution.** The two ways
the record can be silent — it named no plan, it granted no authority — are
the two ways the decision knows least. Treating either as permissive would
make the least-informed decisions in the system the most powerful ones.

**The plan digest is the thread, and it was already half in place.** The wire
envelope carries `payload_digest`; the request form did not; the record did
not. The request names the plan, the decision carries it, and the executor
compares it with the plan it was handed: a request, a decision and a plan
that have to agree before anything runs.

**Reading a record through a closed reader is what keeps the two ends from
drifting.** The writer and the reader of the decision record are in one
crate, and the capsule's vector harness writes the record through the
policy layer's own writer and reads it through the policy layer's own
reader. A field the writer stops writing is a field the reader stops
finding, and the case that expects a commit fails.

## Consequences

**What this makes easy:**

- The executor's authorisation is now the policy layer's decision, and the
  digest binding is a binding against the decision rather than against a
  value the host chose.
- A decision that refused, deferred or quarantined a request cannot be
  mistaken for one that permitted it.

**What this makes hard:**

- The decision-record schema gains a field, and the request form gains one.
  Both are additive; the schema version stays at 1.0 because the house rule
  is that an additive change does not break a reader.
- The capsule's authorization is now built from a document, so both
  implementations and both vector harnesses changed with it. The vector
  file's section is renamed from `authorization` to `decision`, because it
  describes the decision a case is judged under.

**What is implemented, and what is not:**

| Part | State | Where |
|---|---|---|
| The request names the plan it asks to run | done | `Request::payload_digest`, both implementations |
| The record carries the plan the request named | done | `DecisionRecord::permitted_payload_digest`, `to_record` |
| The record is read through a closed reader | done | `senvoryn_scp::record::DecisionRecord::from_value` |
| A record that claims an effect is refused | done | `RecordError::EffectClaimed` |
| The executor reads the plan, the version and the status from the record | done | `Authorization::from_record`, shared vectors `CC-039` and `CC-040` |
| The approvals in the decision record | **not done** | the policy layer is given the approval's digest, not the approval |
| The record's validity window checked by the executor | **not done** | `valid_until` is read and not compared with the executor's clock |
| The record's capability scope compared with the capsule's target | **not done** | the scope is read and not compared |
| A signed record | **not done** | ADR-002: no cryptography in the trusted core |

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Keep the host-assembled authorization and add a check that the host assembled it correctly | The check would be the host's own check. A value the host builds is a value the host can build to pass. |
| Let the executor read the **request** rather than the record | The request is the caller's document. The decision is what the policy layer said about it, and the decision is the authority. |
| Compare the record's `target_resource_id` with the capsule's `target_id` | The capsule's target name may be an alias and the record's is canonical (ADR-018). Comparing them refuses every alias, and the two are not the same kind of thing. The version is the comparable part, and it is compared already. |
| Put the whole record in the capsule's API | The capsule would carry a dozen fields it does not read, and every field of the record would become part of the executor's contract. The executor reads the three fields it binds and the reader refuses the rest. |
| Let the executor compute the digest of the plan and compare it with the record's | ADR-002: no digest is computed in the trusted core, and the signature is what proves the plan is the one that was signed. Computing it here would add a second, weaker notion of what the plan is. |
| Treat a record that names no plan as authorising any plan | The same argument as ADR-020's version: the case where the decision names none is the case where it knows least, and a wildcard there makes the least-informed decision the most permissive one. |

## Open items

- [ ] The approvals are still passed to the executor beside the record. The
      policy layer is given the approval's digest, not the approval, so the
      record cannot carry them. What would close it: the approval records
      themselves reaching the executor, bound to the plan by the digest the
      request named.
- [ ] The request's `approval_digest` and its `payload_digest` are the same
      value for a capsule — the approval binds the plan — and nothing checks
      that they agree. The gate-11 check judges the approval's strength; it
      does not compare the two digests.
- [ ] The record's `valid_until` is read and not compared with the
      executor's clock, so a decision that has expired still authorises.
- [ ] The record's `capability_scope_ref` is read and not compared with the
      capsule's target, so the scope the authority was granted in is not
      checked against the object the plan acts on.
- [ ] The record is not signed (ADR-002). Inside the trust boundary it is
      trusted because the policy layer wrote it; across a boundary it is a
      document anybody can write.

## References

- [`crates/senvoryn-scp/src/record.rs`](../crates/senvoryn-scp/src/record.rs)
- [`crates/senvoryn-scp/src/decision.rs`](../crates/senvoryn-scp/src/decision.rs)
- [`crates/senvoryn-capsule/src/authorization.rs`](../crates/senvoryn-capsule/src/authorization.rs)
- [`reference/capsule_reference.py`](../reference/capsule_reference.py)
- [`conformance/capsule-vectors.json`](../conformance/capsule-vectors.json)
- [ADR-020 — the decision names the version](ADR-020-the-decision-names-the-version.md)
- [ADR-019 — the policy request form](ADR-019-the-policy-request-form.md)
- [ADR-018 — the resolution boundary](ADR-018-the-resolution-boundary.md)
- [ADR-002 — the cryptographic boundary](ADR-002-crypto-boundary.md)

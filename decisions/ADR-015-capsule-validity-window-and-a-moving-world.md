# ADR-015 — The Capsule's Validity Window Is a Boundary, and the World Moves

**Status:** Proposed
**Date:** 2026-10-21
**Deciders:** Project maintainers
**Affected components:** `crates/senvoryn-capsule`, `reference/capsule_reference.py`, `state-machines/change-capsule-transitions.json`, `conformance/capsule-vectors.json`

---

## Context

- **Problem:** Gate 2's roadmap listed three fault kinds the simulator did not
  model — a full disk, a clock rollback, and a corrupted artefact. Modelling
  them was not the point; the point was that **each of them would find a
  control the executor did not have.** They did, and they found four:

  1. **The validity window was checked once.** The executor read the clock at
     validation and compared the capsule's expiry against it. Nothing read the
     clock again. A capsule that was valid when it was admitted and expired
     before its steps ran was applied anyway, and a plan that took longer than
     its validity window could land an effect after the window had closed.
  2. **Nothing detected a clock that moved backwards.** The executor read the
     clock once and trusted it. A clock that jumps back makes an expired
     capsule look unexpired again, so the one check that used the clock was
     the one check an attacker or a broken time source could defeat.
  3. **A write that failed was recorded as a write that landed.** The
     executor classified a step interrupted by an I/O error as `Applied`. A
     full disk is a different case: a write that could not complete may have
     been partially applied, and "partially applied" is not a state the
     idempotency ledger can describe. Recording it as `Applied` claims a
     knowledge the executor does not have.
  4. **The compensation never checked the recovery material.** The precheck
     verified that the recovery artefact was present. The compensation — the
     code that claims to have undone an effect — never looked at it. Material
     that was present before the steps ran may be gone or corrupt by the time
     the compensation runs, and a compensation that claims a rollback it
     could not perform is a false claim. And *presence is not integrity*: a
     corrupt artefact is present, and using it would undo the effect with
     material that cannot be trusted.

- **Constraint:** The executor is a simulator. It acts on a simulated
  environment, performs no effect, and decides no policy. A fault it models
  has to be a state the environment can hold, not an event it can only
  report.

- **Assumptions:** A step's `timeout_ms` is the transaction's own account of
  how long the step takes. The simulator has no other measure of elapsed
  time, so it uses that one, and the clock moves with the work.

## Decision

We will treat **the world as something that moves under the transaction**,
and we will model three kinds of movement.

**The clock moves with the work.** After a step completes, the simulated
clock advances by that step's `timeout_ms`. The validity window is then
checked **before every step**, not only at admission, and the clock is read
before every step.

**A clock that moves backwards is a clock the transaction cannot act on.**
The executor records the highest time it has observed and refuses to carry
on when the clock goes behind it. Where the transaction has observed nothing
yet, the capsule's own issue time is the reference: a capsule cannot be
executed before it was issued, so a clock that reads earlier than the issue
time is a clock that cannot be trusted. Both checks live in one place, and
every read of the clock goes through it.

**A write that could not complete leaves the effect unknown.** A full disk is
treated exactly as a crash is: the effect may or may not have landed, and an
effect that is not known cannot be undone automatically. The two differ in
cause and not in what can be claimed about the result.

**The recovery material is checked where it is relied on.** The precheck
verifies that it is present and intact, the commit verifies that it is still
intact, and the compensation verifies that it is present and intact before it
claims a rollback.

**The state table's guards are corrected.** Two guards named a narrower
condition than the transitions they guard. `STAGED → FAILED` said
`interrupted_during_precheck`, and it is also where a world that moved under
the precheck is refused; `APPLYING → FAILED` said
`step_failed_before_its_effect_landed`, and it is also where a clock that
moved backwards is refused. No transition is added and no authority is
broadened.

## Rationale

**The validity window is a boundary, not a check.** A check is a question
asked at one point. A boundary is a line the transaction may not cross at
*any* point. The difference matters because the whole point of a validity
window is to stop a capsule that sat in a queue for an hour from being
applied an hour later — and a check performed only at admission cannot stop
that, because the queue is between the admission and the effect.

**The clock is the one input that can be moved by something other than the
capsule's issuer.** Every other input — the target version, the policy
epoch, the preconditions — is under the system's control and is rechecked
when the system is rechecked. The clock is not: it moves on its own, it
moves backwards when a time source steps or a snapshot is restored, and it
moves backwards *silently*. An executor that reads the clock once and trusts
it has a single point of failure that an adversary does not even need to
reach.

**An effect that is not known is not an effect that landed.** The ledger has
three states — not applied, applied, unknown — and the third exists for
exactly this case. Recording a failed write as `Applied` would let the
executor compensate something that was never written, and would let a
retry apply it twice.

**Presence is not integrity.** The project's own rule is that absence is
never protection; this is its converse. Material that is missing stops the
transaction, and material that is corrupt is *used*, which is worse. A
recovery plan that promises reversibility and cannot deliver it is a plan
that has failed at the moment it matters, and the executor has to find that
out before it reports a commit or a rollback.

**A guard that names the wrong condition is a contract that misleads.** The
guards in the state table are not evaluated by the executor — they are the
table's documentation of when a transition applies. A guard that says
"interrupted during the precheck" on a transition that also carries a
clock that moved backwards sends a reader looking for the wrong thing.

## Consequences

**What this makes easy:**
- The capsule's expiry is a real boundary: a plan that outlives its window
  stops before the next effect lands, and the effect that already landed is
  undone.
- A clock that cannot be trusted ends the transaction rather than being
  believed.
- A full disk is modelled, and the modelling changed the answer: the effect
  is unknown, so the transaction escalates instead of claiming a clean
  rollback.
- A corrupt recovery artefact is found before the transaction reports
  anything.

**What this makes hard:**
- The fault space grew from 65 combinations to 104, and every one of them
  has to be checked. The property test over the whole space is what makes
  that affordable.
- A new fault kind now costs two implementations, a vector, and a mutation
  in each language. The agreement is worth more than it is cheap, and it is
  not free.

**What this costs:**
- Roughly two hundred lines across the two implementations, one ADR, five
  vectors, and twenty mutations.

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Read the clock once, at validation | That is what the executor did, and it is what the defect was. A capsule that expired after admission was applied. |
| Check the expiry only at the precheck | The precheck happens before any step, and the simulator cannot move the clock forward before it, so a check there could never fire. A control that has never been seen to fail has proved nothing, and the project's own rule is that the simulator must not claim controls it cannot demonstrate. The window is checked where the clock does move: before every step. |
| Treat a full disk as `NotApplied` and let the step be retried | A write that fails may have been partially applied. Retrying it would apply it twice, which is the exact failure the idempotency ledger exists to prevent. |
| Treat a full disk as `Applied`, like an I/O error | The I/O error is modelled as an interruption *after* the write landed. A full disk is a write that could not complete. They are different events, and the executor may not report the same knowledge about both. |
| Read the clock only at the boundaries, and let the steps run past the window | The effect would land after the window closed. The window would be a suggestion. |
| Compare the clock against the wall clock at the end of the transaction | The transaction has no trustworthy wall clock to compare against; the clock is the input. Comparing it with itself proves nothing. |
| Check the recovery material's integrity only at the precheck | The precheck is the start of the plan, and the plan may run for a long time. A material that is intact at the start and corrupt at the end cannot undo the effect, and the compensation is where that matters. |
| Leave the state table's guards alone, since nothing evaluates them | The guards are the table's documentation of when a transition applies, and the table is a contract artifact. A guard that names the wrong condition is a contract that misleads its readers, which is the same failure as a claim with no control behind it. |

## Open items

- [ ] Whether the budget must fit inside the validity window — a plan whose
      budget outlasts its window could, in principle, run past it — what
      evidence would close it: a static check at validation that
      `issued_at + budget.duration_ms <= expires_at`, and a vector.
- [ ] Whether the simulation's clock model is faithful enough to support
      conclusions about time — what evidence would close it: a decision on
      whether the simulator's clock is the boot-epoch clock the Core
      Contract Specification describes, and what the conversion is.
- [ ] Whether the operation registry's `max_duration_ms` and
      `max_operation_count` are per operation or totals, and how that
      reconciles with the capsule's total budget — what evidence would close
      it: a decision, and a vector for each reading.

## References

- [`conformance/capsule-vectors.json`](../conformance/capsule-vectors.json)
- [`state-machines/change-capsule-transitions.json`](../state-machines/change-capsule-transitions.json)
- [`crates/senvoryn-capsule`](../crates/senvoryn-capsule)
- [`reference/capsule_reference.py`](../reference/capsule_reference.py)
- [ADR-013 — the pre-effect failure transitions](ADR-013-capsule-pre-effect-failure-transitions.md)
- [ADR-014 — the Change Capsule executor has two implementations](ADR-014-capsule-two-implementations.md)

# ADR-017 — The clock is a boundary

**Status:** Proposed
**Date:** 2026-10-21
**Deciders:** Project maintainers
**Affected components:** `crates/senvoryn-scp`, `crates/senvoryn-capsule`, `reference/scp_reference.py`, `reference/capsule_reference.py`, `schemas/operation-registry.schema.json`

---

## Context

- **Problem:** The system's time comes from two clocks that disagree about what
  they are for, and nothing in the repository says which one may be believed
  in which place.
  - The **policy layer's clock is monotonic**: milliseconds since boot. The
    request's freshness gate, the capability's validity window, and the
    replay cache are all measured with it. That is right, and it was chosen
    for a good reason — a monotonic clock cannot go backwards.
  - The **capsule executor's clock is a wall clock**: the capsule's
    `issued_at` and `expires_at` are RFC 3339 strings, and the executor
    refuses a clock that moves backwards (ADR-015).
  - A **published document** — a signed operation registry, a reviewed policy —
    cannot carry a monotonic window at all. Every boot would make its window
    meaningless, so a document's validity has to be stated in wall-clock time.

  So the registry's validity window, the last thing ADR-016 leaves open,
  needs a wall clock, and the policy layer currently has none. The question
  is what the system is allowed to believe about a wall clock, and the
  answer has a security edge to it.

- **The security edge:** a wall clock that moves **backwards** makes an
  expired thing look current. A registry whose window has closed would look
  open; a document that was published in the future would look published.
  The capsule executor already refuses a backwards clock, and the same
  reasoning applies to every other wall-clock window — the difference is
  that the policy layer has no wall clock yet, so it has no rule.

- **Constraint:** The evaluator is a **pure function**. It reads no clock, and
  it must stay that way: a decision that depends on ambient state cannot be
  replayed, and the vectors replay. The clock is read by the environment, and
  the environment states it in the context. Whatever the rule is, it has to be
  enforceable by a component that builds the context, not by the evaluator.

- **Constraint:** The environment already states facts about itself in the
  context — the audit sink's readiness is `audit_state: READY | UNAVAILABLE`,
  not a boolean the evaluator infers. The clock's trustworthiness belongs in
  the same place, in the same shape.

- **Constraint:** The capability's validity window is already monotonic, and
  the request's `issued_at`/`expires_at` are already monotonic. Neither
  changes.

## Decision

**The monotonic clock is the session's authority, and only the monotonic
clock is.** Every window that a session produces or consumes — a request's
freshness, a capability's validity, the replay cache — is measured against
`now_monotonic`. This is unchanged, and it is the reason the request's
expiry cannot be undone by moving a clock.

**A wall clock is only a statement about a document, and it is only believed
while it is not going backwards.** The policy layer will read a wall clock
exactly once per context, and it will do so through the environment, which
**refuses to state a wall clock that has moved backwards** since the last one
it stated. The environment says `clock_state: TRUSTED | UNTRUSTED`, in the
same shape as `audit_state`, and the evaluator treats an untrusted clock as
the absence of a clock.

**The boot epoch is the bridge, and it is not an authority.** The system
records the wall-clock time at the moment the monotonic clock was zero. That
gives every later moment a wall-clock reading (`boot_epoch + now_monotonic`)
that is consistent with the session's own timeline, even if the wall clock
has since been corrected. The boot epoch is used for **reporting** — what
time was it, when was this decision made — and for **ordering** documents
against each other. It is **not** used to decide whether a document is
current, because a boot epoch taken from a wall clock at boot is exactly as
wrong as that wall clock was.

**A document's window is checked against the wall clock, and the check is
fail-closed.** If the wall clock is untrusted, or the document's window has
closed, or the document was issued in the future, the document is not
authoritative and the system refuses to act on it. The system does not fall
back to the last document it had: "the last registry we happened to have" is
not a policy (ADR-016).

**A wall clock that moves forwards is not trusted either, without a bound.**
A clock that jumps forwards makes every window look closed, which is safe
but useless, and it also makes the *maximum staleness* a document must
satisfy meaningless — a clock that jumps forwards by a year makes a
year-old document look fresh. The system's own maximum staleness therefore
has to be measured against a clock the system can account for, and a jump
larger than the maximum staleness is treated as the clock being untrusted,
not as the document being old.

## Rationale

**A clock is an input, and an input that can be forged is an authority
hole.** The system's job is to decide whether a request is fresh, a
capability is current, a capsule is still valid, and a policy document is
still in force. Every one of those answers is a comparison against a number
the system reads from outside itself. A monotonic clock can be wrong only by
jumping, and it can be compared with the previous reading; a wall clock can
be wrong by being set, and nothing in the system can tell the difference
between an administrator correcting the clock and an attacker setting it
backwards.

**The monotonic clock is the right answer for a session and the wrong answer
for a document.** A session's windows are created and consumed inside one
boot: the capability was issued by a node that is running now, and the
request was made moments ago. Monotonic time is exactly right for that, and
it is immune to the clock being set. A document's window is different: the
registry was reviewed and published on a machine that is not this one, and
it has to mean something when this machine boots tomorrow. Monotonic time
cannot carry that, so the document has to be stated in wall-clock time and
the wall clock has to be read.

**A backwards wall clock is not a small problem.** It is the difference
between "the policy in force is current" and "the policy in force appears
current because the clock says so". The refusal is not a nicety: a system
that believes a backwards clock will accept a document it should refuse, and
the acceptance is silent.

**The boot epoch is a reporting convenience, not a trust anchor.** The
temptation is to use `boot_epoch + now_monotonic` as *the* wall clock,
because it is smooth and it cannot go backwards. That is a trap: it makes
the system believe a wall-clock reading that the wall clock does not agree
with, and it silently freezes a wrong time for the life of the session. The
boot epoch is recorded, reported, and used for ordering. The currentness of
a document is judged against the wall clock the system can actually read,
guarded against going backwards.

**Fail-closed is the whole point.** A system that stops when it cannot tell
the time is inconvenient. A system that decides when it cannot tell the time
is wrong, and it is wrong in the direction that grants authority.

## Consequences

**What this makes easy:**
- The registry's validity window has a place to live, and the rule that
  governs it is written down before it is coded.
- Every wall-clock window in the system — the registry's, the capsule's, and
  any future document's — is judged by one rule, in one place.
- The reason a backwards clock is refused is a decision rather than an
  implementation detail, so the capsule executor's existing behaviour is
  justified rather than incidental.

**What this makes hard:**
- The environment needs state it does not have: the previous wall-clock
  reading, and the trust decision that goes with it. The evaluator stays
  pure; the environment does not.
- The context gains fields, and the vector file gains a way to state them.
  Every implementation that builds a context — the CLI, the reference model,
  the vector harnesses — has to state the clock the same way.
- A wall clock that moves *forwards* by more than the maximum staleness has
  to be recognised, and the maximum staleness is a published number
  (ADR-016). The rule depends on a number that is not yet published.

**What this costs:**
- This ADR. The registry's window — the first consumer — is the next batch.

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Use the wall clock everywhere, including the request's freshness and the capability's window | A clock that goes backwards makes an expired request fresh again and an expired capability current again. The replay window is exactly the thing a backwards clock defeats. |
| Use the monotonic clock everywhere, and give the registry a monotonic window | A monotonic window in a published document is meaningless: it is measured from *this* machine's boot, and the document was published for a policy that outlives any boot. |
| Use `boot_epoch + now_monotonic` as the wall clock | It is smooth and cannot go backwards, which is exactly why it is wrong: the system would believe a wall-clock reading that the wall clock does not agree with, and would freeze a wrong time for the session. It is a reporting convenience, not an authority. |
| Believe the wall clock, and treat a backwards jump as an ordinary event | A backwards jump is the one wall-clock event that grants authority. It is not ordinary, and it is the reason the refusal exists. |
| Ignore wall-clock steps, and rely on the maximum staleness | The maximum staleness is measured with the same clock it is meant to bound. A clock that jumps forwards by a year makes a year-old document look fresh, so the bound does not bound anything. |
| Refuse to start when the wall clock is untrusted | The system's job is to keep deciding. Refusing to start turns a clock problem into an outage, and a node that cannot start is a node whose policy does not run at all. Refusing to *use* an untrusted clock for a document's window is narrower, and it is the part that grants authority. |
| Keep the last trusted document when a newer one cannot be loaded | "The last registry we happened to have" is not a policy (ADR-016), and the same reasoning applies to every document. |

## Open items

- [ ] The maximum staleness, and therefore the size of a wall-clock jump that
      is treated as the clock being untrusted. What evidence would close it:
      a published number (ADR-016) and the environment's rule that uses it.
- [ ] Where the environment keeps the previous wall-clock reading. What
      evidence would close it: a component that reads the wall clock, compares
      it with the last one it read, and refuses to state an untrusted one.
- [ ] The registry's validity window, which is the first consumer of this
      decision. What evidence would close it: the schema, the shared vector
      for the refusal, and the reason code.
- [ ] The strict UTC timestamp parser's home. The capsule's parser is
      strict and lives in the capsule crate, which the SCP cannot depend on;
      a registry window needs the same strictness, and it needs to be one
      parser rather than two. What evidence would close it: a move into a
      crate both depend on, with the capsule's timestamp boundary tests
      running against it.

## References

- [`crates/senvoryn-capsule/src/environment.rs`](../crates/senvoryn-capsule/src/environment.rs)
- [`crates/senvoryn-capsule/src/executor.rs`](../crates/senvoryn-capsule/src/executor.rs)
- [`crates/senvoryn-scp/src/context.rs`](../crates/senvoryn-scp/src/context.rs)
- [ADR-012 — a capsule's timestamps are parsed once, at the boundary](ADR-012-capsule-utc-timestamp-boundary.md)
- [ADR-015 — the capsule's validity window is a boundary](ADR-015-capsule-validity-window-and-a-moving-world.md)
- [ADR-016 — the registry's limits, its epoch, and its freshness](ADR-016-operation-registry-limits-epoch-and-freshness.md)

# ADR-012 — Parsing a Capsule's Timestamps Once, at the Boundary

**Status:** Proposed
**Date:** 2026-10-21
**Deciders:** Project maintainers
**Affected components:** `crates/senvoryn-capsule`, `schemas/change-capsule.schema.json`

---

## Context

- **Problem:** A Change Capsule carries two timestamps, `issued_at` and `expires_at`,
  and both decide whether the capsule may run at all. The schema types them as
  `date-time`, which is RFC 3339 — and RFC 3339 admits local offsets
  (`2026-01-01T00:00:00+05:30`) as readily as UTC (`2026-01-01T00:00:00Z`).

  A capsule's validity window is a security boundary. If one implementation reads
  `expires_at` as UTC and another reads it as local time, the two disagree about
  whether a capsule is stale, and the one that is wrong lets a stale capsule run.
  Worse, the disagreement is invisible: both implementations are "correct" RFC 3339
  parsers, and the failure looks like a clock skew.

- **Constraint:** The capsule executor compares `expires_at` against the
  environment's clock. A wall-clock comparison needs one epoch. Converting at the
  point of use means every comparison site can be wrong in a different way, and
  there is no single place to test.

  The repository has no third-party dependencies (ADR-009), so no date library is
  available, and the conversion has to be written and justified here.

- **Assumptions:** The wire form of a capsule's timestamps is UTC with a `Z`
  suffix, optionally with a fractional part. A capsule that carries an offset is
  rejected rather than converted, because a capsule's issuer and its executor must
  agree on the timestamp without either of them doing local arithmetic.

## Decision

We will parse a capsule's timestamps **once, at the parse boundary**, with a strict
UTC-only parser, and keep the original wire string alongside the parsed value.

The accepted form is exactly `YYYY-MM-DDTHH:MM:SS[.fff]Z`: a `Z` suffix, no offset,
no missing components, and at most three fractional digits. The `Z` is the **last**
byte of the string, and a fractional part, if there is one, sits between the seconds
and the `Z` — so the `Z` is not at a fixed offset in the string.

The value has to be a **real instant**. A date that does not exist — the thirtieth
of February, the twenty-ninth of February in a year that is not a leap year, the
thirty-first of April — is refused rather than rolled over into the next month. A
year of zero is refused for the same reason. A capsule that names a time which never
happened is malformed, and silently moving it would mean the executor compared the
capsule against an instant its issuer never stated.

The parsed value is an epoch in milliseconds since the Unix epoch, computed with
days-from-civil arithmetic rather than with a calendar library, so the conversion
is auditable in the source.

The original string is retained in the parsed capsule. The evidence a capsule
carries is the document that was actually received, not a re-serialisation of it,
and a reader comparing two evidence records needs to see the bytes that were
signed.

> **Amended 2026-10-21, after the Python reference model was written.** The first
> version of this decision did not say what "the strict UTC form" means about a
> date that does not exist, and the Rust implementation rolled such a date over
> into the next month while the Python one refused it. It also read the `Z` at a
> fixed offset, which meant **every** timestamp carrying fractional seconds was
> rejected and the fraction-handling code below it was unreachable. Both are
> fixed, and both are now pinned by a test in each implementation. The
> ambiguity was real: two readers of the same contract disagreed about a
> security boundary. See [ADR-014](ADR-014-capsule-two-implementations.md).
>
> **Amended 2026-10-21, after the timestamp boundary was mutation-tested.** The
> parser's guards had tests but no mutations, so this decision's claim was
> tested and not mutation-tested. That matters more here than it does
> elsewhere, because the parser has several ways to say no to a malformed
> timestamp — the length, the separators, the zone marker, the fraction, the
> date, the time, the year — and a guard that is only ever reached together
> with another one can be removed and nothing would change. Each guard now
> has a case that only it can refuse, a named test, and a mutation that
> breaks it, in both implementations. The gate found nothing wrong with the
> parser and one thing wrong with the tests: a Python assertion never
> reached its message when the parser raised, so the gate could not tell
> which guard had stopped the timestamp. That test now reports the refusal
> itself, in the guard's own words.

## Rationale

The alternative — accepting RFC 3339 in full and converting on use — makes the
validity window depend on the reader's arithmetic. Two implementations that both
"follow RFC 3339" can disagree by hours about whether a capsule is stale, and the
capsule whose window is shortest is the one that runs. Narrowing the accepted form
to UTC removes the ambiguity at the point where it enters the system, so there is
exactly one arithmetic in the codebase and one set of tests that covers it.

Parsing at the boundary rather than at the point of use has a second effect: a
malformed timestamp is a malformed *capsule*, not a malformed comparison. The
capsule is refused as a whole, and the refusal names the field, rather than a
later check failing in a way that looks like a clock problem.

The cost is a narrower accepted form than RFC 3339. A capsule issuer outside UTC
must convert before it issues, which is the right place to convert: the issuer
knows the instant it means, and the executor does not have to guess.

## Consequences

**What this makes easy:**
- One arithmetic, one set of tests, and no way for two readers to disagree about
  a capsule's validity window.
- A malformed timestamp is caught with the field named, before anything runs.
- The evidence retains the bytes that were received.

**What this makes hard:**
- A capsule that carries a numeric offset is refused. Issuers outside UTC must
  convert at issue time, and that requirement has to be stated to them.
- The conversion is written in this repository rather than taken from a library,
  so it has to be read and tested rather than trusted.

**What this costs:**
- About forty lines of date arithmetic in the capsule crate, and a test that
  pins the conversion against known instants.
- A capsule that carries a numeric offset, a date that does not exist, or a
  year of zero is refused. Issuers have to state a real instant in UTC.

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Accept full RFC 3339 and convert on use | The validity window then depends on each reader's arithmetic, and two conforming readers can disagree about whether a capsule is stale. That is a security boundary, not a formatting preference. |
| Accept any offset and normalise to UTC at the boundary | Better than converting on use, but the capsule's issue time is still stated in a form the issuer and the executor may read differently, and the conversion becomes a place where a bug becomes a stale-capsule bug. |
| Roll a date that does not exist into the next month | The first implementation did this, and the second refused it. A date that does not exist names no instant, and picking one for the capsule is exactly the kind of quiet adjustment a stale capsule needs. |
| Use a date-time crate | The trusted core carries no third-party dependencies (ADR-009), and a date library is exactly the kind of dependency the decision was taken to avoid. |
| Keep only the parsed value and discard the wire string | The evidence would then be a re-serialisation rather than the document that was received, and a reader comparing two records could not tell whether the strings differed. |

## Open items

- [ ] Whether a capsule issued with an offset should be accepted and converted by a
      specified tool, rather than refused — what evidence would close it: a decision
      on whether the issuer's locality is ever part of the protocol.
- [ ] Whether `issued_at` should be compared against the environment's clock as
      well, to reject a capsule issued in the future — what evidence would close it:
      a specification statement about clock skew tolerance for issuance.

## References

- [`schemas/change-capsule.schema.json`](../schemas/change-capsule.schema.json)
- [`crates/senvoryn-capsule/src/capsule.rs`](../crates/senvoryn-capsule/src/capsule.rs)
- [ADR-009 — Zero-dependency trusted core](ADR-009-zero-dependency-trusted-core.md)
- Core Contract Specification §5.3, Change Capsule transaction contract

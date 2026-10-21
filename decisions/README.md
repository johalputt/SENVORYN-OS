# Architecture Decision Records

Architecture Decision Records capture decisions that are **expensive to reverse** and **contestable**. They exist so that future implementers understand *why* the system looks the way it does — including when the honest answer was "we do not know yet."

---

## Rules

1. **ADRs are immutable once accepted.** To change an accepted decision, write a **new** ADR that supersedes it. History is never rewritten.
2. **A superseding ADR must state what changed and why.** The superseded ADR gets a pointer added to its status line; its body is never edited to remove the original reasoning.
3. **Every open question in an accepted ADR is tracked** as an issue until closed.
4. **ADRs are numbered sequentially within a single namespace.**
5. Every ADR follows the required template from [`GOVERNANCE.md` §4](../GOVERNANCE.md#4-architecture-decision-records).

> ### ⚠️ Known numbering collision — tracked as a Gate 0 blocker
>
> The [Constitution's open-decisions register](../ROADMAP.md#open-decisions-register) uses `ADR-001`…`ADR-016` for **different subjects** than the Core Contract Specification's ADRs below. For example, Constitution ADR-001 is *"First CPU architecture"*, while CCS ADR-001 is *"Clean-sheet separation core"*.
>
> Both sets are real and both are cited. Until the namespaces are reconciled, **always qualify the citation** — "CCS ADR-001" vs "Constitution ADR-001" — and do not merge the two registers. This must be resolved before Gate 0 exit.

---

## Index

| ADR | Subject | Status | Open items |
|---|---|---|---|
| [ADR-001](ADR-001-separation-core.md) | Clean-sheet separation core | Proposed for prototype only | 7 |
| [ADR-002](ADR-002-crypto-boundary.md) | Cryptographic boundary | Proposed | 6 |
| [ADR-003](ADR-003-protocol-encoding.md) | Canonical protocol encoding | Revised by ADR-008 | 4 |
| [ADR-004](ADR-004-ai-authority.md) | AI is a proposal producer, not an authority | Proposed | 5 |
| [ADR-005](ADR-005-universal-execution.md) | Universal execution fabric | Proposed | 5 |
| [ADR-006](ADR-006-direct-install.md) | Direct installation | Proposed | 6 |
| [ADR-007](ADR-007-local-first-authority.md) | Local-first authority | Proposed | 5 |
| [ADR-008](ADR-008-canonical-json-profile.md) | Canonical JSON Profile `SCP-CJSON-0001` | Proposed | 3 |
| [ADR-009](ADR-009-zero-dependency-trusted-core.md) | No third-party dependencies in the trusted core | Proposed | 2 |
| [ADR-010](ADR-010-delegation-depth-reading.md) | Delegation depth: the literal reading, and the contradiction | Proposed | 3 |
| [ADR-011](ADR-011-resolution-reason-code.md) | A reason code for the object-resolution gate | Proposed | 2 |
| [ADR-012](ADR-012-capsule-utc-timestamp-boundary.md) | A capsule's timestamps are parsed once, at the boundary, in strict UTC | Proposed | 2 |
| [ADR-013](ADR-013-capsule-pre-effect-failure-transitions.md) | The pre-effect failure transitions the capsule state table was missing | Proposed | 2 |
| [ADR-014](ADR-014-capsule-two-implementations.md) | The Change Capsule executor has two implementations | Proposed | 3 |
| [ADR-015](ADR-015-capsule-validity-window-and-a-moving-world.md) | The capsule's validity window is a boundary, and the world moves | Proposed | 3 |
| [ADR-016](ADR-016-operation-registry-limits-epoch-and-freshness.md) | The Operation Registry's limits, its epoch, and its freshness | Proposed | 3 |
| [ADR-017](ADR-017-the-clock-is-a-boundary.md) | The clock is a boundary | Proposed | 3 |
| [ADR-018](ADR-018-the-resolution-boundary.md) | The resolution boundary: a name is not an identity | Proposed | 4 |
| [ADR-019](ADR-019-the-policy-request-form.md) | The policy request form: one name, two documents | Proposed | 6 |

---

## Summary of decisions

| ADR | Decision in one line |
|---|---|
| **001** | Build an original, minimal separation core as a research program, starting with one emulated CPU profile — no broad services in the core. |
| **002** | Use reviewed, widely analyzed cryptographic primitives behind a small replaceable interface. Never invent a primitive. |
| **003** | A specific canonical byte encoding must be named, specified, and tested before signing any protocol object. |
| **004** | Models run outside the enforcement core; they produce typed envelopes and reach tools only through an independent broker. |
| **005** | Use a layered execution fabric — native formats, managed runtimes, translation, isolated guests — selecting the simplest mode that delivers tested semantics. |
| **006** | Support application-like installation without user-created boot media on certified hosts, preserving a documented recovery route. |
| **007** | Every node retains local policy enforcement; federation cannot bypass local capability, mode, expiry, or risk checks. |
| **008** | The canonical encoding is named, specified, and tested: profile `SCP-CJSON-0001`. |
| **009** | The trusted Rust core carries no third-party dependencies. |
| **010** | A child's delegation depth must decrease, following the specification's literal wording, and the contradiction with the schema is recorded rather than resolved here. |
| **011** | The denial-code registry carries a code for the specification's object-resolution gate. |
| **012** | A capsule's timestamps are parsed once, at the boundary, by a strict UTC-only parser, and the wire string is kept for the evidence. |
| **013** | The Change Capsule state table gains the pre-effect failure transitions the specification requires, so `FAILED` is reachable. |
| **014** | The Change Capsule executor is implemented twice — in Rust and in Python — and held to one shared vector file, so the vectors are an agreement rather than a transcript. |
| **015** | The capsule's validity window is a boundary, not a one-time check: the clock is read before every step, a clock that moves backwards ends the transaction, and a capsule cannot be executed before it was issued. |
| **016** | The registry's limits are per operation and the capsule's budget is a total for the plan; the registry must be the registry for the epoch in force, and it must carry a validity window. |
| **017** | The monotonic clock is the session's authority; a wall clock is only a statement about a document, and it is refused when it moves backwards. |
| **018** | A resolved resource is a canonical identifier, a canonical type and a version; the caller's type claim is checked rather than obeyed, and the canonical identity governs every later gate. |
| **019** | The policy request form is the transport layer's output and not the wire envelope; its field set is closed, so a field the node cannot account for is refused rather than ignored. |

---

## Writing a new ADR

```bash
cp decisions/ADR-XXX-template.md decisions/ADR-019-<slug>.md
```

Fill in every section. The **Context** section is the one that matters most — it must explain what constraint makes the obvious solution wrong. An ADR without a real constraint is a design note.

**Status values:** `Proposed` · `Accepted` · `Superseded by ADR-NNN` · `Rejected` · `Deprecated`

**"Proposed" is a legitimate and common status.** It means the team has not yet agreed. It does not mean the decision is soft — it means the decision has not been *made*, and the open items are the work that would make it.

---

## See also

- [`GOVERNANCE.md`](../GOVERNANCE.md) — the ADR template, review rules, and amendment process
- [`ROADMAP.md`](../ROADMAP.md) — which ADR blocks which gate
- [`ARCHITECTURE.md`](../ARCHITECTURE.md) — the architecture these decisions produce

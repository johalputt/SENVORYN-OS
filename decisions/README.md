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

---

## Writing a new ADR

```bash
cp decisions/ADR-XXX-template.md decisions/ADR-012-<slug>.md
```

Fill in every section. The **Context** section is the one that matters most — it must explain what constraint makes the obvious solution wrong. An ADR without a real constraint is a design note.

**Status values:** `Proposed` · `Accepted` · `Superseded by ADR-NNN` · `Rejected` · `Deprecated`

**"Proposed" is a legitimate and common status.** It means the team has not yet agreed. It does not mean the decision is soft — it means the decision has not been *made*, and the open items are the work that would make it.

---

## See also

- [`GOVERNANCE.md`](../GOVERNANCE.md) — the ADR template, review rules, and amendment process
- [`ROADMAP.md`](../ROADMAP.md) — which ADR blocks which gate
- [`ARCHITECTURE.md`](../ARCHITECTURE.md) — the architecture these decisions produce

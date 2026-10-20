# Documentation

This directory holds the normative documents of the Senvoryn OS project.

---

## Contents

| Path | Document | Status | Authority |
|---|---|---|---|
| [`specs/`](specs/) | **Core Contract Specification v0.1.0** (`SENVORYN-CORE-SPEC-0001`) | Proposed | 2 — implementation contract |
| [`constitution/`](constitution/) | **Founding Constitution v0.6.0** | Proposed for review, not ratified | 1 — highest |
| [`threat-model.md`](threat-model.md) | SCP decision-path threat model | 🔶 First draft, unreviewed | Supporting |
| [`canonical-json-profile.md`](canonical-json-profile.md) | Canonical JSON Profile `SCP-CJSON-0001` | Prototype — named and specified, not independently reviewed | Supporting |

---

## Document hierarchy

When two rules conflict, precedence is:

```
1. Constitutional invariants              (highest — may not be weakened)
2. Core Contract Specification
3. Versioned component specifications
4. Architecture Decision Records
5. Implementation details                 (lowest)
```

Lower-level artifacts may **narrow** a capability or add checks. They must **not** broaden authority or weaken a higher-level rule without a formally approved constitutional amendment.

**If two rules conflict, the privileged operation is rejected, a conflict code is recorded, and a specification decision is required.** Nobody picks a winner in the moment.

---

## Reading order

| If you want to… | Read |
|---|---|
| Understand what the project is | [`README.md`](../README.md) → [`constitution/`](../docs/constitution/) (Executive summary, Article 1–2) |
| Understand the architecture | [`ARCHITECTURE.md`](../ARCHITECTURE.md) → constitution Articles 7–27 |
| Understand the contracts you would implement | [`specs/`](../docs/specs/) (all 17 sections) |
| Understand the vocabulary | [`GLOSSARY.md`](../GLOSSARY.md) |
| Understand the plan | [`ROADMAP.md`](../ROADMAP.md) |
| Attack the design | [`threat-model.md`](threat-model.md) → [`SECURITY.md`](../SECURITY.md) |
| Understand the byte encoding | [`canonical-json-profile.md`](canonical-json-profile.md) → [`ADR-008`](../decisions/ADR-008-canonical-json-profile.md) |
| Understand the rules of change | [`GOVERNANCE.md`](../GOVERNANCE.md) → [`CONTRIBUTING.md`](../CONTRIBUTING.md) |

---

## Conventions

Both documents use the same normative keywords:

| Word | Meaning |
|---|---|
| **MUST** / **MUST NOT** | Binding requirement for a conforming implementation |
| **SHOULD** / **SHOULD NOT** | Strong requirement; deviation requires a written rationale and review |
| **MAY** | Optional capability that must not undermine a MUST |
| **TARGET** | Measurable engineering goal; not a guarantee until benchmarked on a defined reference profile |
| **EXPERIMENTAL** | Disabled by default; must not be relied upon for safety or security claims |
| **PROHIBITED** | Forbidden by the constitution, even if convenient or benchmark-improving |

Every document declares a status. Any statement not yet demonstrated by implementation and test evidence is labelled **proposed**, **unverified**, or **research required**.

> **A feature is not complete when it compiles or appears in the UI.** It is complete only when its authority model, failure behavior, privacy behavior, compatibility scope, recovery path, resource limits, test evidence, and user-facing explanation are all defined and reviewed.

---

## Machine-readable companions

The prose specifications are backed by machine-readable contracts at the repository root — [`schemas/`](../schemas/), [`state-machines/`](../state-machines/), [`errors/`](../errors/), [`examples/`](../examples/), and [`conformance/vectors/`](../conformance/vectors/). Where the prose and the machine-readable form disagree, that is a defect: please report it.

---

## See also

- [`README.md`](../README.md) — project overview and quick start
- [`ARCHITECTURE.md`](../ARCHITECTURE.md) — layer model, TCB, dependency rules
- [`ROADMAP.md`](../ROADMAP.md) — gates 0–8
- [`GLOSSARY.md`](../GLOSSARY.md) — normative vocabulary
- [`GOVERNANCE.md`](../GOVERNANCE.md) — how the documents change

# Senvoryn Core Contract Specification v0.1.0

**Document ID:** `SENVORYN-CORE-SPEC-0001`
**Constitution baseline:** Senvoryn OS Constitution v0.6.0
**Status:** Proposed engineering specification — not ratified, not production evidence
**Date:** 15 October 2026

This package converts the v0.6 constitution into an initial engineering contract. It is a **design/prototype pack — not a production operating system, kernel, cryptographic implementation, or security certification.**

---

## Files in this pack

| File | Purpose |
|---|---|
| `SENVORYN_Core_Contract_Spec_v0.1.md` | Normative engineering specification (Markdown source) |
| `SENVORYN_Core_Contract_Spec_v0.1.docx` | The same specification in Word, rendered for review |
| [`../constitution/SENVORYN_OS_Constitution_v0.6.md`](../constitution/SENVORYN_OS_Constitution_v0.6.md) | Founding Constitution v0.6.0 — the top-level design authority |
| [`../constitution/SENVORYN_OS_Constitution_v0.6.docx`](../constitution/SENVORYN_OS_Constitution_v0.6.docx) | The same constitution in Word |

The machine-readable artifacts that accompany this specification now live at the **repository root**, so that the paths named in §0.2 of the specification resolve exactly as written:

| Repository path | Contents |
|---|---|
| [`../../schemas/`](../../schemas/) | Draft machine-readable JSON Schemas (14) |
| [`../../state-machines/`](../../state-machines/) | Machine-readable lifecycle transitions (2) |
| [`../../errors/`](../../errors/) | Stable denial-code registry |
| [`../../examples/`](../../examples/) | Example request fixture and unsigned operation-registry sample |
| [`../../reference/`](../../reference/) | A small deterministic SCP decision model — **not** a kernel or cryptographic implementation |
| [`../../tests/`](../../tests/) | Executable negative and positive tests for selected invariants |
| [`../../decisions/`](../../decisions/) | Initial architecture decisions and unresolved decisions |
| [`../../evidence/`](../../evidence/) | Requirement-to-test traceability and evidence ledger |

---

## Run the reference tests

From the **repository root**:

```bash
python -m pip install -r requirements-dev.txt
PYTHONPATH=reference python -m unittest discover -s tests -v
```

On Windows PowerShell:

```powershell
$env:PYTHONPATH = "reference"
python -m unittest discover -s tests -v
```

Expected: `Ran 29 tests ... OK`.

Or run the complete local verification (checksums + schemas + tests):

```bash
bash scripts/verify-pack.sh        # POSIX
pwsh -File scripts/verify-pack.ps1 # Windows
```

---

## ⚠️ What the reference implementation does not do

The reference implementation uses deliberately simplified integer time values and trusted booleans. It does **not** verify signatures, secure identity, clocks, policy documents, canonicalization, hardware, or actual effects.

> **Never deploy it as an SCP.**

It exists to demonstrate *policy ordering* and to give the invariants a testable shape — nothing more.

---

## Validate JSON Schemas

The schemas use **JSON Schema Draft 2020-12**. The prototype includes principal, resource, intent, approval, audit-event, update-plan, and system-state schemas alongside the rest. Validate them with a conforming validator in CI (see [`../../.github/workflows/ci.yml`](../../.github/workflows/ci.yml)).

**Schema success proves only structural validity** — not authorization, not signature validity, not semantic safety.

---

## Conformance rule

A test row is not complete until it links to a source revision, test harness, named profile, result artifact, reviewer, and limitations.

**No security claim may be based solely on an unreviewed unit test.**

See [`../../evidence/claims-ledger.md`](../../evidence/claims-ledger.md) for what the project may currently claim, and [`../../evidence/local-validation-report.txt`](../../evidence/local-validation-report.txt) for what the local validation actually proved.

---

## See also

- [`../../README.md`](../../README.md) — project overview
- [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) — layers and trust boundaries
- [`../../ROADMAP.md`](../../ROADMAP.md) — gates and exit criteria
- [`../../GLOSSARY.md`](../../GLOSSARY.md) — normative vocabulary
- [`../../SECURITY.md`](../../SECURITY.md) — how to report a finding

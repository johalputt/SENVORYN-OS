# Component Specifications

**Status:** Template only. No component specification has been ratified yet.

Component specifications sit at **level 3** of the document hierarchy:

```
1. Constitutional invariants          (highest — may not be weakened)
2. Core Contract Specification
3. Component specifications            ← this directory
4. Architecture Decision Records
5. Implementation details               (lowest)
```

A component specification **may narrow** a capability or add checks. It **must not** broaden authority or weaken a higher-level rule without a formally approved constitutional amendment.

---

## When one is required

Write a component specification when you are about to implement a component that:

- enforces a capability, an invariant, or a gate;
- produces a signed, hashed, or audited artifact;
- crosses a trust boundary;
- defines a protocol or wire format;
- makes a claim that appears in the [Claims Ledger](../../evidence/claims-ledger.md).

If you are not sure, ask in an issue first. A specification that is written late is cheaper than an implementation that is wrong.

---

## Template

Copy this file to `docs/component-specs/<COMPONENT>-v<MAJOR>.<MINOR>.md` and fill in every section. **Do not delete a section** — write "N/A" and say why. An empty section reads as an unchecked box; "N/A because …" reads as a decision.

````markdown
# <Component Name> Specification

**Component ID:** <stable identifier, e.g. senvoryn.scp>
**Version:** 1.0
**Status:** Proposed | Under review | Accepted | Superseded by <id> | Withdrawn
**Date:** YYYY-MM-DD
**Owner:** <role, not necessarily a person>
**Reviewers:** <at least two for security-critical components; one non-author>

---

## 1. Purpose and scope

What this component does, and — more importantly — what it explicitly does not do.

**In scope:** …
**Out of scope:** …

## 2. Layer and trust position

| Field | Value |
|---|---|
| Architectural layer | L0–L9 |
| Trusted? | yes / no |
| What may call it | … |
| What it may call | … |
| Failure containment boundary | … |

## 3. Conformance requirements

Which requirements from the Core Contract Specification this component implements.
Each entry names the specification section and the conformance test ID that proves it.

| Requirement | Spec reference | Test ID | Status |
|---|---|---|---|
| … | §… | SCP-… | Not implemented |

## 4. Interfaces

### 4.1 Inputs

| Field | Type | Validation | Notes |
|---|---|---|---|---|

### 4.2 Outputs

| Field | Type | Validation | Notes |
|---|---|---|---|---|

### 4.3 Errors

Map every error to a code in the [denial-code registry](../../errors/denial-codes.json).
A condition must never be reported as success in one subsystem and failure in another.

## 5. Invariants preserved

Which constitutional invariants this component upholds, and **how** each is enforced —
not merely which ones apply. For each: the enforcement point, the failure behavior, and
the test that proves it.

| Invariant | Enforcement point | Failure behavior | Test ID |
|---|---|---|---|

## 6. Resource bounds

| Resource | Bound | Enforcement | Over-limit behavior |
|---|---|---|---|
| CPU | | | |
| Memory | | | |
| Storage | | | |
| I/O | | | |
| Duration | | | |

## 7. Threat model

Which abuse cases from [`docs/threat-model.md`](../threat-model.md) apply, and which
mitigations this component implements. List any abuse case in scope that this
component does **not** yet mitigate — silently omitting it is a defect.

## 8. Failure and degradation behavior

What happens when each dependency is missing, slow, or hostile. Include the
fail-safe / fail-operational decision and its justification.

| Dependency | Failure mode | Behavior | Rationale |
|---|---|---|---|

## 9. Observability and audit

What is recorded, at what level, with what redaction. Confirm the record contains
no secrets and no full private content.

## 10. Privacy and data flow

Which data labels this component handles, where data can flow, and what is exported.
"None" is a valid and common answer — say so explicitly.

## 11. Compatibility and versioning

Schema and protocol versioning rules. What constitutes a breaking change, and what
the migration path is.

## 12. Test obligations

| Layer | Required |
|---|---|
| Schema | |
| Unit | |
| Property | |
| Fuzz | |
| Integration | |
| Fault injection | |
| Adversarial | |
| Performance | |

## 13. Known limitations

Unresolved risks, unimplemented paths, and unsupported configurations. This section
is the one most likely to be quoted back at the project — write it honestly.

## 14. Open questions

| Question | Owner | Needed evidence | Blocks |
|---|---|---|---|

# Senvoryn OS — pull request template

<!--
Thanks for contributing to Senvoryn OS.

The project is at the specification stage, so a good PR is usually a contract,
a model, a test, or evidence — rather than a large code change.

Please fill in the sections below. If a section genuinely does not apply,
write "N/A" and say why — do not delete it. An empty section reads as an
unchecked box; "N/A because ..." reads as a decision.
-->

## What this changes

<!-- One or two sentences. What is different after this PR? -->

## Type of change

- [ ] **Specification / contract** — schemas, state machines, or normative prose
- [ ] **Reference model** — the deterministic SCP evaluator in `reference/`
- [ ] **Tests** — new or extended contract tests
- [ ] **Evidence** — traceability, validation report, claims ledger
- [ ] **Decision record** — a new or superseding ADR
- [ ] **Threat model / security analysis**
- [ ] **Documentation** — non-normative docs, examples, glossary
- [ ] **Tooling / CI**
- [ ] **Governance / process**

## Related issue

<!-- Closes #123  /  Part of #123  /  No issue (explain why) -->

## Invariant impact

**Does this change weaken any constitutional invariant (INV-01 … INV-30)?**

- [ ] **No.** No invariant is weakened or bypassed.
- [ ] **Yes.** An invariant is affected — this requires a **constitutional amendment** per [`GOVERNANCE.md`](/GOVERNANCE.md#5-constitutional-amendments), not a normal PR.

<!-- If "No", briefly state which invariants this change touches and how it respects them. -->

> Lower-level artifacts may **narrow** a capability or add checks. They must
> **not** broaden authority or weaken a higher-level rule without a formally
> approved constitutional amendment.

## Gate and scope

<!-- Which gate does this advance? See ROADMAP.md -->

- **Gate:** <!-- 0 / 1 / 2 / 3 / 4 / 5 / 6 / 7 / 8 -->
- **Scope-control template filled in?** <!-- Required for anything beyond a trivial fix. See CONTRIBUTING.md -->

| Field | Answer |
|---|---|
| User problem | |
| Threat model | |
| Minimum viable scope | |
| New dependencies | |
| Privilege required | |
| Measurable benefit | |
| Complexity cost | |
| Test plan | |
| Failure behavior | |
| Deferred | |

## Conformance test IDs

<!-- Which SCP-xxx / CAP-xxx / CC-xxx / OC-xxx / AI-xxx / INS-xxx / UPD-xxx / PRIV-xxx / FED-xxx / EVID-xxx IDs does this add, satisfy, or affect? -->

- Added: <!-- e.g. SCP-009 -->
- Satisfied: <!-- e.g. CAP-001 -->
- Affected: <!-- e.g. AI-004 -->

> Test IDs are a stable contract. A published ID is never silently changed.

## Evidence

- [ ] Tests added or updated
- [ ] `evidence/traceability.csv` updated
- [ ] `evidence/claims-ledger.md` updated **if this changes what may be claimed**
- [ ] Checksums regenerated if a contract artifact changed (`python scripts/generate-checksums.py --write`)

## Review

- [ ] This change is **not** security-critical → 1 reviewer
- [ ] This change **is** security-critical (capability enforcement, SCP gates, update verifier, installer, key handling, audit, privilege boundaries) → **2 qualified reviewers, one of whom did not author this change**
- [ ] An ADR is included (required for architecture, protocol, object model, trust model, dependency admission, or licensing changes)

## Checklist

- [ ] `PYTHONPATH=reference python -m unittest discover -s tests -v` passes
- [ ] `python scripts/verify-pack.sh` (or `pwsh -File scripts/verify-pack.ps1`) passes
- [ ] Documentation updated where behavior changed
- [ ] No secrets, keys, or credentials in the diff
- [ ] No absolute security claims added without evidence and limitations
- [ ] I have read [`CONTRIBUTING.md`](/CONTRIBUTING.md) and [`CODE_OF_CONDUCT.md`](/CODE_OF_CONDUCT.md)

## Licensing

By contributing, I agree that my contribution is licensed under the **Apache License 2.0**.

- [ ] Confirmed, or the licensing question has been raised in an issue **before** this PR.

<!--
Reviewer note: a reviewer who finds a defect should say what the contract
requires, not merely that the change is wrong. "This violates INV-03 because
the child scope exceeds the parent's" is actionable; "this is wrong" is not.
-->

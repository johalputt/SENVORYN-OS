# Changelog

All notable changes to Senvoryn OS are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Version scope.** This project currently publishes a **specification and model**. Version numbers refer to the contract artifacts, schemas, and reference model — **not** to an operating system release. No artifact in this repository may be installed as a trusted operating system.

---

## [Unreleased]

### Added
- Initial repository scaffold for `johalputt/SENVORYN-OS`.
- Core Contract Specification v0.1.0 (`SENVORYN-CORE-SPEC-0001`) — normative engineering contract with data contracts, state machines, validation order, failure behavior, and testable obligations.
- Founding Constitution v0.6.0 — baseline top-level design authority with 30 constitutional invariants, layered architecture, governance, and acceptance criteria.
- 14 JSON Schemas (Draft 2020-12) for principals, resources, request envelopes, capabilities, decision records, approval records, intent records, the operation registry, Change Capsules, Origin Capsules, audit events, artifact manifests, update plans, and system states.
- 2 machine-readable state machines: Change Capsule lifecycle transitions and system-mode transitions, each with global invariants and prohibited transitions.
- Stable denial-code registry with 26 codes, each carrying a class and a default decision.
- Example fixtures: a valid request envelope and an **unsigned** sample operation registry.
- Deterministic reference SCP decision model implementing the 16-gate validation order with a pure, side-effect-free evaluator.
- 29 executable contract tests covering denials, freshness, epoch binding, approval digests, risk-claim mismatch, mode enforcement, evaluator purity, and capability attenuation.
- Architecture Decision Records ADR-001 … ADR-007 covering the separation core, cryptographic boundary, canonical encoding, AI authority, universal execution, direct installation, and local-first authority.
- Evidence ledger: requirement→test traceability matrix, local validation report, security claim ledger template, and a live claims ledger.
- Threat model for the SCP decision path.
- CI pipeline with five jobs: contract tests on Python 3.12/3.13, schema validation, pack integrity (SHA-256), JSON lint, and normative-language checks.
- Community health files: contributing guide, code of conduct, security policy, governance document, issue templates, pull request template, CODEOWNERS, and Dependabot configuration.
- Repository verification scripts for POSIX shells and PowerShell.

### Known gaps
- The SCP threat model is a first draft and has not been independently reviewed.
- The Operation Registry is an unsigned sample and is **not** production policy.
- The reference model's capability verification uses a trusted boolean; there is no signature verification, canonical parsing, or real clock handling.
- No property-based test framework is wired in; attenuation is checked by hand-written assertions.
- The ADR numbering namespace collides between the Constitution's open-decisions register and the Core Contract Specification's ADRs — tracked as a Gate 0 blocker.
- No artifact is signed, and no SBOM or release-signing ceremony exists yet.

---

## [0.1.0] — 2026-10-15

**Senvoryn Core Contract Specification v0.1.0** — *Implementation Contract Layer.*

First published contract pack, converting the v0.6 constitution into an initial engineering contract.

### Added
- Document ID `SENVORYN-CORE-SPEC-0001`, dated 15 October 2026, on the v0.6.0 constitution baseline.
- 17-section normative specification: core model and trust boundaries, canonical data and protocol rules, the SCP request contract, capability semantics, the Change Capsule transaction contract, the Origin Capsule contract, boot/installer/update/recovery semantics, the AI action contract, the Universal Execution Fabric, privacy and continuity contracts, failure states, the test and evidence contract, provisional targets, architecture decisions, the implementation sequence, and conformance test identifiers.
- 14 machine-readable JSON Schemas (Draft 2020-12).
- 2 machine-readable state machines with global invariants.
- Initial denial-code registry (26 codes).
- Valid request fixture and unsigned sample operation registry.
- Deterministic SCP reference model with 16-gate validation order, capability model, and attenuation check.
- 29 local tests covering selected invariants.
- Requirement→test traceability matrix (17 rows).
- Security claim ledger template.
- Architecture Decision Records ADR-001 … ADR-007 with open questions.
- Local validation report: 29 tests passed, 14 schemas valid, 19 JSON documents parsed, 17 pages rendered for visual review.
- SHA-256 checksum manifest.

### Notes
- Conformance label: **Design-conformant**. The reference model is **Prototype-conformant** in a limited test environment only.
- The reference model uses simplified integer time values and trusted booleans. It does **not** verify signatures, secure identity, clocks, policy documents, canonicalization, hardware, or actual effects, and **must not be deployed as an SCP**.

[Unreleased]: https://github.com/johalputt/SENVORYN-OS/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/johalputt/SENVORYN-OS/releases/tag/v0.1.0

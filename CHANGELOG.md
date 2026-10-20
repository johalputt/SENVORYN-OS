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
- Stable denial-code registry with 27 codes, each carrying a class and a default decision.
- Example fixtures: a valid request envelope, an **unsigned** sample operation registry, and a set of development-tool fixtures.
- Deterministic reference SCP decision model implementing the 16-gate validation order with a pure, side-effect-free evaluator, in Python.
- 84 executable Python contract tests covering denials, freshness, epoch binding, approval digests, risk-claim mismatch, mode enforcement, evaluator purity, capability attenuation, the denial-code registry, the Change Capsule fixture's conformance to its schema, the capsule conformance vectors, and the capsule reference model's own controls.
- **Rust workspace with no third-party dependencies**:
  - `senvoryn-canonical` — the Senvoryn Canonical JSON Profile `SCP-CJSON-0001`: a parser, an encoder, and limits that are checked before the allocation they bound.
  - `senvoryn-scp` — the SCP decision oracle: the sixteen validation gates, the capability model, the attenuation rule, the trusted Operation Registry merge, and the audit-event builder.
  - `senvoryn-capsule` — the Change Capsule executor **simulator**: the state machine driven from the repository's own transition table, the idempotency ledger, the compensation planner, and a fault injector. It acts on a simulated environment, performs no effect, and decides no policy; it checks that the System Control Plane's decision binds the capsule's own digest.
  - `senvoryn-cli` — a development tool that drives the oracle from files. It says in its own help that it is not an SCP.
- 105 Rust tests, including a shared conformance-vector suite, seeded property tests, a deterministic fuzz harness over the canonical parser, a registry-sync suite, the capsule's fault-injection property, the capsule's vector suite, the capsule's timestamp boundary, and the registry's merge rule.
- **Shared conformance vectors** (`conformance/vectors/vectors.json`) — 31 decision cases and 11 attenuation cases executed by *both* the Python model and the Rust implementation. CI fails if they produce a different decision or a different reason code.
- **Change Capsule vectors** (`conformance/capsule-vectors.json`) — 30 cases naming the state the capsule executor must reach, executed by **both** the Rust executor and the Python reference model. They use the `CC-` group of the specification's conformance identifiers.
- **A second implementation of the Change Capsule executor** (`reference/capsule_reference.py`) — the Python reference model, written from the contract rather than from the Rust. It is held to the same vector file, so the vectors are an agreement rather than a transcript. ADR-014.
- **A mutation gate** (`scripts/mutation-gate.py`, `scripts/mutations.json`) — 60 controls are broken in turn and a *named* test is required to go red for each. It runs tests in Rust and in Python, and it clears the Python bytecode cache before each run, because a cached `.pyc` can shadow a mutated source and make a control look untested. A surviving mutation is a control no test enforces, and the gate fails. It found four real gaps the unit tests had missed: the "not yet valid" half of a capability's validity window had no shared vector; the decision record's `effect_performed` field had no test; the capsule's registry-risk check was satisfied by a different control; and the capsule's approval-requirement vector passed for the wrong reason.
- Architecture Decision Records ADR-001 … ADR-014, adding the canonical JSON profile, the zero-dependency trusted core, the delegation-depth contradiction, the object-resolution reason code, the capsule's strict-UTC timestamp boundary, the pre-effect failure transitions the Change Capsule state table was missing, and the decision to implement the capsule executor twice.
- Evidence ledger: requirement→test traceability matrix, local validation report, security claim ledger template, and a live claims ledger.
- Threat model for the SCP decision path (first draft, not independently reviewed).
- CI pipeline with ten jobs: contract tests on Python 3.12/3.13, a Rust format/lint/test/release job, a conformance-agreement job, a mutation-gate job, schema validation, pack integrity (SHA-256), JSON lint, normative-language checks, dependency review, and YAML/link linting.
- Community health files: contributing guide, code of conduct, security policy, governance document, issue templates, pull request template, CODEOWNERS, and Dependabot configuration.
- Repository verification scripts for POSIX shells and PowerShell.

### Changed
- The Python reference model was rewritten to the specification's 16-gate order. Its previous order transposed the recipient/identity and nonce/freshness gates, and it emitted a reason code that was not in the registry.
- `RESOURCE_UNRESOLVABLE` was added to the denial-code registry (class `resolution`, default `DENY`). The specification defines an object-resolution gate and the registry had no code for it.
- `Provenance` and `AuditState` are now enums rather than booleans, a capability cannot be constructed without stating its provenance, `PolicyContext.audit_state` has no default, and `Decision.effect_performed` is a read-only property rather than a settable field.
- The sample operation registry's `read` operation now lists `RECOVERY` among its allowed modes.

### Fixed
- **The Operation Registry's merge rule kept the wrong reversibility.** Where two rules applied to the same operation, the merge took the *more* reversible of the two, so a lax rule could override a strict one. The registry's own documentation says the opposite, and every other axis of the merge — risk tier, mode set, budgets, audit class — already took the most restrictive value. The sample registry holds one rule per operation, so the merge never ran in a test. Found by the second implementation of the capsule executor, which is written from the contract rather than from the first one. Fixed in both the Rust and the Python, and pinned by a test in each.
- **The capsule's timestamp parser rejected every timestamp with fractional seconds.** It read the `Z` marker at a fixed offset in the string, so a timestamp with a fractional part was always refused and the fraction-handling code below it was unreachable. Found by the same second implementation.
- **The capsule's timestamp parser accepted a date that does not exist.** The thirtieth of February was silently rolled over into the next month, so a capsule could name an instant its issuer never stated. Found by the same second implementation. ADR-012 amended.

### Known gaps
- The SCP threat model is a first draft and has not been independently reviewed.
- The Operation Registry is an unsigned sample and is **not** production policy.
- Neither implementation verifies a cryptographic signature: capability provenance is stated, not proven. The verified-provenance boundary remains open under ADR-002.
- The Rust oracle is a decision function. It is **not** an SCP: it resolves no resource against a filesystem, keeps no trust store, and writes no audit record.
- The canonical parser is fuzzed; the envelope→request mapping is not.
- No property-based test framework is used; the property tests are a seeded generator written for this repository.
- The mutation gate covers 60 controls. It is not exhaustive: the installer, update and AI-broker gates have no implementation to mutate yet.
- The Change Capsule vectors are executed by two implementations, the Rust executor and the Python reference model. Two implementations of one specification is a strong check, and it is still not a proof: both read the same vector file, and whoever writes that file decides what "right" means. Neither implementation is an independent reviewer.
- The capsule executor acts on a simulated environment. It proves the transaction's bookkeeping and its refusals; it does not prove that a real system can be rolled back, because nothing here touches storage.
- The Change Capsule state table was amended to add the pre-effect failure transitions the specification requires. `FAILED` was a state the table listed and no transition could reach ([ADR-013](decisions/ADR-013-capsule-pre-effect-failure-transitions.md)).
- The delegation depth rule follows the specification's literal wording, which contradicts the capability schema's description of the field. The contradiction is recorded, not resolved ([ADR-010](decisions/ADR-010-delegation-depth-reading.md)).
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

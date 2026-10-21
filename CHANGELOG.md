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
- Stable denial-code registry with 32 codes, each carrying a class and a default decision.
- Example fixtures: a valid request envelope, an **unsigned** sample operation registry, and a set of development-tool fixtures.
- Deterministic reference SCP decision model implementing the 16-gate validation order with a pure, side-effect-free evaluator, in Python.
- 114 executable Python contract tests covering denials, freshness, epoch binding, approval digests, risk-claim mismatch, mode enforcement, evaluator purity, capability attenuation, the denial-code registry, the Change Capsule fixture's conformance to its schema, the capsule conformance vectors, and the capsule reference model's own controls.
- **Rust workspace with no third-party dependencies**:
  - `senvoryn-canonical` — the Senvoryn Canonical JSON Profile `SCP-CJSON-0001`: a parser, an encoder, and limits that are checked before the allocation they bound.
  - `senvoryn-instant` — the strict UTC instant the documents use. One parser, so the capsule and the SCP cannot disagree about what a timestamp means.
  - `senvoryn-scp` — the SCP decision oracle: the sixteen validation gates, the capability model, the attenuation rule, the trusted Operation Registry merge, and the audit-event builder.
  - `senvoryn-capsule` — the Change Capsule executor **simulator**: the state machine driven from the repository's own transition table, the idempotency ledger, the compensation planner, and a fault injector. It acts on a simulated environment, performs no effect, and decides no policy; it checks that the System Control Plane's decision binds the capsule's own digest and the version of the target the decision was made against.
  - `senvoryn-cli` — a development tool that drives the oracle from files. It says in its own help that it is not an SCP.
- **A validity window for the Operation Registry.** The registry now carries `issued_at` and
  `expires_at`, and the SCP refuses to decide anything under a registry whose window has
  not opened (`REGISTRY_NOT_YET_VALID`) or has closed (`REGISTRY_EXPIRED`). The window is
  judged against the wall clock, and a wall clock that has moved backwards is refused
  (`CLOCK_UNTRUSTED`) rather than believed, because a clock that moves backwards is the one
  way an expired document can look current.
- 141 Rust tests, including a shared conformance-vector suite, seeded property tests, two deterministic fuzz harnesses (the canonical parser and the policy request reader), a registry-sync suite, the capsule's fault-injection property, the capsule's vector suite, the capsule's timestamp boundary, and the registry's merge rule.
- **Shared conformance vectors** (`conformance/vectors/vectors.json`) — 40 decision cases and 11 attenuation cases executed by *both* the Python model and the Rust implementation. CI fails if they produce a different decision or a different reason code.
- **Change Capsule vectors** (`conformance/capsule-vectors.json`) — 40 cases naming the state the capsule executor must reach, executed by **both** the Rust executor and the Python reference model. They use the `CC-` group of the specification's conformance identifiers.
- **A second implementation of the Change Capsule executor** (`reference/capsule_reference.py`) — the Python reference model, written from the contract rather than from the Rust. It is held to the same vector file, so the vectors are an agreement rather than a transcript. ADR-014.
- **A mutation gate** (`scripts/mutation-gate.py`, `scripts/mutations.json`) — 148 controls are broken in turn and a *named* test is required to go red for each. It runs tests in Rust and in Python, and it clears the Python bytecode cache before each run, because a cached `.pyc` can shadow a mutated source and make a control look untested. A surviving mutation is a control no test enforces, and the gate fails. It found four real gaps the unit tests had missed: the "not yet valid" half of a capability's validity window had no shared vector; the decision record's `effect_performed` field had no test; the capsule's registry-risk check was satisfied by a different control; and the capsule's approval-requirement vector passed for the wrong reason.
- Architecture Decision Records ADR-001 … ADR-021, adding the canonical JSON profile, the zero-dependency trusted core, the delegation-depth contradiction, the object-resolution reason code, the capsule's strict-UTC timestamp boundary, the pre-effect failure transitions the Change Capsule state table was missing, the decision to implement the capsule executor twice, the decision that the capsule's validity window is a boundary rather than a one-time check, the decision that the registry's limits are per operation while the capsule's budget is a total for the plan, the decision that the clock is a boundary: the monotonic clock is the session's authority, and a wall clock is refused when it moves backwards, the decision that a name is not an identity: the resolution boundary resolves a name to a canonical identifier, a canonical type and a version, the decision that a request is read and not judged: the policy request form is the transport layer's output and not the wire envelope, and its field set is closed, the decision that the decision names the version: an authorization names the version of the target the decision was made against, and the executor refuses one that names none or names another, and the decision that the decision record is the authorization: the executor reads the plan, the version and the status out of the record the policy layer wrote.
- **The resolution boundary.** A resolved resource is a canonical identifier, a canonical type and a version, and the canonical identity — never the caller's own strings — governs the capability gate, the risk gate, and the record that authorises the effect. A caller's type claim is *checked* rather than obeyed: a request that misdescribes its own target is refused with `RESOURCE_TYPE_MISMATCH` instead of being corrected, because a caller that believes it is addressing a file when the node knows it is addressing a device will handle the object wrongly even though the node's authorisation was right. An alias is usable and still has to match a grant over the object the alias means. ADR-018.
- **The policy request form is closed.** The reader refuses a document carrying a field the form does not define, at the top level and inside the budget, because a field this node cannot account for may be a field a newer sender expects this node to enforce. The reader enforces the form and nothing else: an operation this node has never heard of, a resource it cannot resolve and a principal it does not know are read successfully and refused by the gates that follow.
- **A second fuzz harness, over the request boundary.** The canonical parser's harness mutates bytes; this one mutates a corpus of policy requests and asserts that the reader never panics, that a request which was read says exactly what the document said, and that the same document is read the same way twice. A reader that changes a value while reading it is a reader whose decision is not the caller's decision.
- **The reader's nine controls are mutation-tested.** The closed field set, the object check, the schema check, the missing field, the empty field, the negative number, the malformed risk claim, the closed budget, and the reader's duty to keep the value it read.
- **The decision names the version.** The authorization carries the version of the target the decision was made against, and the executor refuses a decision that names none — it was made before the resource was resolved, and it was made about a name rather than an object — or names another. The digest binding answers "is this the plan I judged?" and the version binding answers "was the object in the state I judged it in?". The canonical identifier is not compared, because the capsule's own target name may be an alias. ADR-020.
- **A reader of a document nobody produces is deleted.** The authorization's `from_value` read a decision-record "authorization section" that no schema defines and that no producer writes, and nothing called it. It is gone, and the gap it hid — the authorization is still a value the host constructs, and the decision record carries neither the plan digest nor the approvals — is an open item in ADR-020.
- **The decision record is the authorization.** The executor was handed a digest and a version the host had assembled, and checked them against the capsule: a check of a pair the host built, and consistency is not authority. The request may now name the plan it asks to run, the record carries that plan, and the executor reads the plan, the version and the status out of the record through a closed reader that refuses a record claiming an effect was performed. A decision that refused a request authorises nothing, and a decision that names no plan authorises no plan. The approvals are still passed in beside the record — the policy layer is given the approval's digest, not the approval — and that is the last of the authorization the host assembles. ADR-021.
- **The record's two ends cannot drift.** The capsule's vector harness writes the decision record through the policy layer's own writer and reads it through the policy layer's own reader, so a field the writer stops writing is a field the reader stops finding, and the case that expects a commit fails.
- **The two documents called a request are now told apart.** The wire envelope the schema describes and the policy request the reader reads are different documents, the transport layer that converts one into the other does not exist yet, and the drift between them — the replay key, the target node, the caller's classification and effect claims, the approval binding — is listed as open items instead of being implied. ADR-019.
- **A new reason code, `RESOURCE_TYPE_MISMATCH`**, in the resolution class.
- **A capsule's budget must fit inside its own validity window.** A capsule that asks for more time than the window allows contradicts itself, and it is refused at validation, where the operator finds out before anything is staged. The check and its boundary are both tested in both implementations.
- **The registry's limits and the capsule's budget are reconciled by four rules**, each with a test of its own: the registry bounds one invocation and one operation's frequency, and the capsule's budget bounds the plan's total duration and total operation count.
- **The operation registry is bound to the epoch in force.** The registry's `registry_epoch` was parsed by nothing, so a node could decide a request under a registry from another policy generation while every epoch check passed. A decision made under a registry from another generation is now refused, with a reason code of its own so that an operator can tell it apart from a request that named the wrong epoch.
- **A new reason code, `REGISTRY_EPOCH_MISMATCH`**, in the policy class.
- **The capsule's validity window is a boundary.** The clock is read before every step, the simulated clock advances with the work, a clock that moves backwards ends the transaction, and a capsule cannot be executed before it was issued. The clock's monotonicity, the issue time, the per-step window check and the recovery material's integrity are four controls, each with a named test and a named mutation. ADR-015.
- **Three faults the simulator did not model** — a full disk, a clock that rolls back, and a recovery artefact that turns out to be corrupt. A full disk leaves the effect *unknown* rather than applied, because a write that could not complete may have been partially applied and the ledger cannot describe that; a corrupt artefact is found before the transaction reports anything, because presence is not integrity. Each fault is modelled in both implementations and each has a shared vector.
- **The timestamp boundary is mutation-tested.** The capsule's strict-UTC parser had tests but no mutations, so the claim was tested and not mutation-tested. The parser has several ways to say no to a malformed timestamp — the length, the separators, the zone marker, the fraction, the date, the time, the year — and a guard that is only ever reached together with another one can be removed and nothing would change. Each guard now has a case that only it can refuse, a named test, and a mutation that breaks it, in both implementations. ADR-012.
- Evidence ledger: requirement→test traceability matrix, local validation report, security claim ledger template, and a live claims ledger.
- Threat model for the SCP decision path (first draft, not independently reviewed).
- CI pipeline with ten jobs: contract tests on Python 3.12/3.13, a Rust format/lint/test/release job, a conformance-agreement job, a mutation-gate job, schema validation, pack integrity (SHA-256), JSON lint, normative-language checks, dependency review, and YAML/link linting.
- Community health files: contributing guide, code of conduct, security policy, governance document, issue templates, pull request template, CODEOWNERS, and Dependabot configuration.
- Repository verification scripts for POSIX shells and PowerShell.

### Changed
- The Python reference model was rewritten to the specification's 16-gate order. Its previous order transposed the recipient/identity and nonce/freshness gates, and it emitted a reason code that was not in the registry.
- `RESOURCE_UNRESOLVABLE` was added to the denial-code registry (class `resolution`, default `DENY`). The specification defines an object-resolution gate and the registry had no code for it.
- The decision path no longer judges the caller's strings. `PolicyContext.resolved_resources` maps a name to the canonical identity of the object it means, the capability gate compares the grant with the canonical identifier and type, the risk gate asks the registry about the canonical type, and the decision record and the audit event name the canonical object. The record carries the resolved version, so the executor's recheck has an anchor the caller cannot change.
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
- The mutation gate covers 80 controls. It is not exhaustive: the installer, update and AI-broker gates have no implementation to mutate yet.
- The Change Capsule vectors are executed by two implementations, the Rust executor and the Python reference model. Two implementations of one specification is a strong check, and it is still not a proof: both read the same vector file, and whoever writes that file decides what "right" means. Neither implementation is an independent reviewer.
- The capsule executor acts on a simulated environment. It proves the transaction's bookkeeping and its refusals; it does not prove that a real system can be rolled back, because nothing here touches storage.
- The Change Capsule state table was amended to add the pre-effect failure transitions the specification requires. `FAILED` was a state the table listed and no transition could reach ([ADR-013](decisions/ADR-013-capsule-pre-effect-failure-transitions.md)).
- The operation registry carries no validity window, so nothing bounds how old the policy in force is. The decision is recorded; the implementation is not done ([ADR-016](decisions/ADR-016-operation-registry-limits-epoch-and-freshness.md)).
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

# Roadmap

**Status:** Active — current position is **Gate 0 (in progress)**. No gate beyond
Gate 0 has been met, and no date is a commitment.

> Normative reference: [Core Contract Specification v0.1.0](docs/specs/SENVORYN_Core_Contract_Spec_v0.1.md) §15 and [Founding Constitution v0.6.0](docs/constitution/SENVORYN_OS_Constitution_v0.6.md) Articles 42, 101–103.

> **No gate is met because a calendar date was reached.** Gate acceptance requires artifact links, results, review, and explicit limitations. Dates are not commitments until staffing, funding, and prototype results are known.

---

## Where we are

**Current gate: Gate 0 — Contract freeze for prototype.** 🟡 In progress.

This repository *is* the Gate 0 deliverable: the constitutional baseline, the core contract specification, machine-readable schemas, state machines, the denial registry, a reference evaluator, and a traceability matrix.

### What is true today

| Statement | Status |
|---|---|
| 14 JSON Schemas are structurally valid Draft 2020-12 | ✅ Verified |
| 91 Python contract tests pass | ✅ Verified (Python 3.12 / 3.13) |
| 111 Rust tests pass | ✅ Verified (Rust stable) |
| 31 shared conformance vectors are agreed by both implementations | ✅ Verified — CI fails on a disagreement |
| 35 Change Capsule vectors are agreed by both implementations | ✅ Verified — the Rust executor and the Python reference model, CI fails on a disagreement |
| Every control has a test that fails when the control is broken | ✅ Verified — 80 mutations, all killed |
| 2 state machines have valid endpoints and invariants | ✅ Verified |
| 27 denial codes are unique with defined defaults | ✅ Verified |
| Every denial code is cross-checked against both implementations | ✅ Verified — registry-sync tests on both sides |
| The decision evaluator is pure (no side effects) | ✅ Verified by test + code review |
| A threat model for the SCP decision path exists | 🟡 First draft exists — **not independently reviewed** (Gate 0 blocker) |
| The Operation Registry is reviewed, versioned, and signed | ❌ Missing — sample is unsigned |
| Property-based attenuation tests exist | ✅ Seeded property tests — not externally reviewed |
| A canonicalization profile is named and tested | ✅ Named and specified as `SCP-CJSON-0001` — independent review outstanding |
| Any gate beyond Gate 0 is met | ❌ No |
| Any profile is certified | ❌ No |
| Any independent security review has occurred | ❌ No |

---

## Gate roadmap

### Gate 0 — Contract freeze for prototype 🟡 IN PROGRESS

**Deliver:** accepted identifiers, typed object model, first schemas, threat model, state machine, error registry, reference policy evaluator, traceability matrix.

**Exit criteria:** ambiguity review identifies **no unresolved issue that would change authority semantics**.

| # | Item | Status | Artifact |
|---|---|---|---|
| 0.1 | Accepted identifiers and typed object model | ✅ | `schemas/`, [GLOSSARY.md](GLOSSARY.md) |
| 0.2 | First schemas (14) | ✅ | `schemas/` |
| 0.3 | Threat model for the SCP decision path | ❌ | **TODO** — see below |
| 0.4 | State machines | ✅ | `state-machines/` |
| 0.5 | Error / denial registry | ✅ | `errors/denial-codes.json` |
| 0.6 | Reference policy evaluator | ✅ | `reference/scp_reference.py` |
| 0.7 | Traceability matrix | ✅ | `evidence/traceability.csv` |
| 0.8 | Ambiguity review sign-off | ❌ | Requires 2 independent reviewers |

**Remaining Gate 0 work:**
1. Get `docs/threat-model.md` independently reviewed. The draft exists; the review does not, and the draft does not support any security claim.
2. Get the canonicalization profile (`SCP-CJSON-0001`, [ADR-008](decisions/ADR-008-canonical-json-profile.md)) independently reviewed before it is used to verify anything.
3. Promote `examples/operation-registry.sample.json` to a reviewed, versioned registry (still unsigned until Gate 1 tooling exists).
4. Reconcile the delegation-depth contradiction between Core Contract Specification §4.2 and the capability schema ([ADR-010](decisions/ADR-010-delegation-depth-reading.md)).
5. Reconcile the two ADR namespaces — Constitution `ADR-001`…`ADR-016` against Core Contract `ADR-001`…`ADR-007`.
6. Obtain two independent ambiguity reviews.

---

### Gate 1 — SCP decision oracle 🟡 IN PROGRESS

**Deliver:** pure evaluator, capability model, deterministic reason codes, audit-event builder, property tests, fuzzable parser boundary.

**Exit:** all required negative cases pass **and** a reviewer confirms the evaluator has no side effects.

- [ ] Real cryptographic capability verification (not a trusted boolean) behind a reviewed crypto boundary
- [x] Canonical parser rejecting duplicate keys, invalid UTF-8, ambiguous numbers, unknown critical fields — `crates/senvoryn-canonical`
- [x] Canonicalization profile implemented and tested — `SCP-CJSON-0001`, [ADR-008](decisions/ADR-008-canonical-json-profile.md)
- [x] Property-based test suite: attenuation never broadens authority; denied requests create no effects; schema version change never silently grants new fields — seeded property tests in `crates/senvoryn-scp/tests/properties.rs`
- [ ] Fuzz harness for the request envelope parser (malformed, truncated, oversized, duplicate-key, deeply nested) — the canonical parser is fuzzed; the envelope→request mapping is not
- [x] Audit-event builder emitting `schemas/audit-event.schema.json` records — `crates/senvoryn-scp/src/audit.rs`
- [x] Deterministic reason codes covering all 27 registry entries — cross-checked against the registry by both implementations
- [x] Mutation gate — `scripts/mutation-gate.py` breaks 80 controls in turn and requires a named test to go red for each. It runs tests in two languages, and it clears the Python bytecode cache before each run, because a cached `.pyc` can shadow a mutated source
- [ ] The signed Operation Registry, and the resolution boundary that gate 6 needs
- [ ] An independent reviewer confirms the evaluator has no side effects

---

### Gate 2 — Change Capsule executor simulator 🟡 IN PROGRESS

**Deliver:** state machine, operation registry, idempotency ledger, stale-precondition detection, rollback/compensation simulation, fault injection.

**Exit:** interrupted operations **never** falsely report `COMMITTED`.

- [x] Executable `change-capsule-transitions.json` engine with guard evaluation — the executor reads the repository's own table; it does not decide legality itself
- [x] Idempotency ledger keyed by capsule ID + replay key — an outcome known not to have applied may be retried; an outcome that is *not known* may not
- [x] Stale target-version and policy-epoch detection before every step — rechecked in the precheck, not only at validation
- [x] Compensation planner for `COMPENSATABLE` vs `NON_REVERSIBLE` operations — a non-reversible operation escalates to a human rather than claiming a rollback
- [x] Fault injection: crash, lost storage, stale time, full disk, power interruption, clock rollback, corrupted artifact — crash, I/O error, stale precondition, policy-epoch change, lost audit sink, full disk, clock rollback, corrupted recovery artifact
- [x] Transition tests for every illegal transition in the table — the table is checked for unreachable states as well as illegal ones
- [x] 35 capsule conformance vectors, and 56 of the controls mutation-checked (27 in Rust, 29 in Python)
- [x] A second implementation of the capsule executor, so the capsule vectors are *agreed* rather than merely self-consistent — the Python reference model in `reference/capsule_reference.py`, written from the contract rather than from the Rust
- [x] The validity window is a boundary, not a one-time check — the clock is read before every step, a clock that moves backwards ends the transaction, and the capsule cannot be executed before it was issued ([ADR-015](decisions/ADR-015-capsule-validity-window-and-a-moving-world.md))
- [x] The three faults the simulator did not model — full disk, clock rollback, corrupted artefact — and the four controls they turned out to be missing

**Still open for Gate 2 exit:**

- [ ] The budget does not yet have to fit inside the validity window: a plan whose budget outlasts its window runs past it, and the executor stops at the boundary rather than refusing the plan
- [ ] The registry's `max_duration_ms` and `max_operation_count` are per operation, and the capsule's budget is a total; the executor reconciles the two, and the reading is not yet decided
- [ ] The executor drives a simulated environment; nothing here proves that a real system can be rolled back
- [ ] The vectors are read by two implementations of the same contract, and whoever writes the contract decides what "right" means. That is a strong check and not a proof

---

### Gate 3 — Capsule policy simulator ⬜ NOT STARTED

**Deliver:** Origin Capsule manifest, bridge registry, budget checks, guest/host authority tests.

**Exit:** all cross-boundary tests prove that **guest authority cannot become host authority** in the test model.

- [ ] Origin Capsule manifest generator with all bridges disabled by default
- [ ] Bridge registry with direction, data label, lease, budget, logging class
- [ ] Resource budget vector enforcement (CPU, memory, storage, processes, I/O, network)
- [ ] Guest-administrator isolation tests (`OC-001`, `OC-002`, `OC-003`)
- [ ] Network default-deny verification in an isolated namespace

---

### Gate 4 — Minimal bootable prototype ⬜ NOT STARTED

**Deliver:** one declared hardware/emulator profile, trusted boot chain, minimal service supervisor, local policy enforcement, recovery path.

**No production claim at this stage.**

- [ ] Choose the first ISA and emulator profile ([ADR-001](decisions/ADR-001-separation-core.md) open item)
- [ ] Boot manifest with component hashes, versions, hardware profile, dependency order, signing identity, policy epoch, anti-rollback version, recovery artifact
- [ ] Verifier rejecting invalid signature, unsupported critical schema, revoked signer, disallowed downgrade, hash mismatch, profile mismatch
- [ ] Minimal Separation Core: address spaces, scheduling, bounded IPC, capability invocation
- [ ] Service supervisor + local SCP
- [ ] Recovery path to a prior bootable generation

---

### Gate 5 — Direct Installer prototype ⬜ NOT STARTED

**Deliver:** one certified host profile, safe staging, cancellation, power-loss simulation, disk target confirmation, first-boot verification, recovery documentation.

- [ ] Installer provenance verification **before** any elevated operation
- [ ] Transaction states: `DISCOVER → COMPATIBILITY_CHECK → PLAN → USER_REVIEW → AUTHORIZED → DOWNLOAD → VERIFY → STAGE → BOOT_CONFIG_STAGE → PRECOMMIT_CHECK → COMMIT → FIRST_BOOT_VERIFY → COMPLETE`
- [ ] Last-completed-checkpoint recording + recovery instructions
- [ ] Power-loss simulation: prior boot path or documented recovery path must survive
- [ ] Destructive-mode double confirmation with explicit target identifier

---

### Gate 6 — UEF proof of concept ⬜ NOT STARTED

**Deliver:** one chosen foreign application profile, compatibility record, guest boundary, bridge policy, adversarial tests.

**Add formats only after evidence.**

- [ ] Execution-selection algorithm per spec §9.1 (8 steps)
- [ ] Compatibility record with exact versions, tiers `C0`–`C4`
- [ ] Guest boundary with explicit, typed, revocable bridges
- [ ] Adversarial tests: guest-admin privilege escalation, host mapping attempts, credential inspection attempts

---

### Gate 7 — AI-native transaction layer ⬜ NOT STARTED

**Deliver:** Intent Record, Action Envelope, model service boundary, tool broker, prompt-injection corpus, autonomy controls.

**AI-disabled core tests must still pass.**

- [ ] Intent Record + Action Envelope schemas exercised end-to-end
- [ ] Tool broker that resolves target, computes real risk, validates capabilities, checks consent, invokes SCP
- [ ] Autonomy class enforcement `A0`–`A4` with immediate user reduction
- [ ] Adversarial corpus: indirect prompt injection, tool-call laundering, memory poisoning, cross-user leakage, wrong-target action, exfiltration attempt, tool spoofing, overbroad plan, malicious model package, resource exhaustion
- [ ] Data-flow grant required before any remote inference of `SENSITIVE`/`SECRET` data

---

### Gate 8 — Security review and public alpha ⬜ NOT STARTED

**Deliver:** independent review, vulnerability disclosure process, reproducible build report, SBOM, release signatures, threat model, performance data, profile support matrix, user data export, update recovery drill, release claims ledger.

- [ ] Independent security review of the SCP path, capability enforcement, update verifier, installer
- [ ] SBOM + license inventory + reproducible build report for critical components
- [ ] Published threat model and claims ledger
- [ ] Performance data on a named reference profile (methodology published)
- [ ] Vulnerability disclosure process operational, with a rehearsed rotation drill
- [ ] User data export and deletion interfaces with documented residual copies

---

## Provisional performance targets

These are **initial hypotheses for a reference profile, not promises**. A target becomes binding only after the architecture board approves reference hardware and measurement methodology.

| Metric | Initial target | Measurement constraint |
|---|---:|---|
| SCP local cached policy p99 | < 5 ms | Declared reference CPU, 10k+ requests, warmed and cold results reported separately |
| Small service startup p95 | < 250 ms | Dependency-ready boundary and service definition published |
| Idle minimal server RAM | < 256 MiB | Excludes workload memory and optional AI; exact profile and accounting method disclosed |
| Recovery to prior bootable generation | < 120 s | Reference storage/device; excludes user-data restoration and physical repair |
| AI-disabled core operation | 100% of core boot/login/policy/update/recovery tests | Test suite and build profile published |
| Privileged request audit coverage | 100% of defined privileged operation registry | Includes failure and cancellation paths; exceptions reviewed |
| Direct install without user-created boot media | Per-certified-host success rate published | Host OS, firmware, disk mode and recovery conditions listed |

Benchmarks must include hardware model, firmware, workload, compiler/toolchain, build options, sample size, latency distribution, variance, thermal state, power mode, and raw data where safe. **Performance cannot be reported without the isolation profile used by that run.**

---

## Prototype sequence (constitution view)

The constitution's `P0`–`P8` sequence maps onto the gates above:

| Phase | Scope | Gate |
|---|---|---|
| **P0** | Specification and executable model | Gate 0 → Gate 1 |
| **P1** | Minimal kernel research in an emulator | Gate 4 (partial) |
| **P2** | SCP and capsule transactions | Gate 1 → Gate 2 |
| **P3** | Core local OS (storage, identity, network, supervisor, text UI) | Gate 4 |
| **P4** | Installer research, VM-first | Gate 5 |
| **P5** | One compatibility environment | Gate 6 |
| **P6** | AI planner and tool broker | Gate 7 |
| **P7** | Hardware expansion, one profile at a time | Ongoing |
| **P8** | Fleet and public alpha | Gate 8 |

> **Build the smallest trustworthy vertical slice.** One end-to-end path — verified boot in a simulator, minimal process isolation, one capability-protected object, SCP permit/deny, append-only audit, a signed Change Capsule, a simulated update, failure injection, and recovery to a known state — proves the architecture is coherent before complexity accumulates.

---

## Open decisions register

These must be resolved through ADRs before the corresponding code is frozen. **They are not to be settled by guesswork.**

| ID | Decision | Required evidence | Default until decided |
|---|---|---|---|
| ADR-001 | First CPU architecture | Target hardware, tooling, boot access, testing capacity | No public support claim |
| ADR-002 | Boot trust and recovery layout | Threat model, hardware matrix, recovery tests | Research profile only |
| ADR-003 | Kernel implementation language and subset | Memory safety, toolchain provenance, verification and maintenance cost | No production kernel code |
| ADR-004 | IPC wire format and versioning | Size bounds, fuzzability, forward compatibility | Prototype-only schema |
| ADR-005 | Capability representation and revocation epoch | Race analysis, performance, object lifecycle proof | Fail closed; deny unknown semantics |
| ADR-006 | Filesystem and storage transaction model | Crash consistency, data migration and recovery tests | No claim of atomic data rollback |
| ADR-007 | Cryptographic suite and agility profile | Independent security review and ecosystem feasibility | Use reviewed standard designs; do not invent primitives |
| ADR-008 | Native ABI and app package schema | SDK design, stability and compatibility tests | Versioned experimental API |
| ADR-009 | First UEF compatibility mode | Legal review, guest requirements, feature-test corpus | No universal compatibility claim |
| ADR-010 | Direct Installer host matrix | Boot loader, partition layout, encryption, rollback tests | Only named host profiles certified |
| ADR-011 | AI model format and local inference engine | Performance, license, privacy, supply-chain model | Model runtime optional and untrusted |
| ADR-012 | Distributed consensus/lease design | Failure model, safety/availability tradeoffs | Defer global operations during uncertainty |
| ADR-013 | Key custody and recovery ceremony | Threat model, signer independence, loss/compromise drills | No stable release signing |
| ADR-014 | Accessibility semantic UI contract | Assistive technology test matrix | Core flows remain in prototype |
| ADR-015 | Formal proof scope | Precise spec, toolchain and assumptions | Never claim proof outside defined scope |
| ADR-016 | Strict Apache-2.0 component policy | Full dependency/license inventory and legal review | Original code Apache-2.0; dependencies disclosed individually |

> **Note:** the `decisions/` directory currently contains **ADR-001 … ADR-015**, which follow the Core Contract Specification's own ADR numbering. The constitution's open-decisions register uses the same IDs for *different* subjects (e.g. constitution ADR-001 is "First CPU architecture"; CCS ADR-001 is "Clean-sheet separation core"; constitution ADR-015 is "Formal proof scope" and this project's ADR-015 is the capsule's validity window). This numbering collision is tracked as a **Gate 0 ambiguity** and must be reconciled before Gate 0 exit — the two registers will be renumbered to a single namespace.

---

## How to help

Priority order for contributions:

1. **Gate 0 blockers** — threat model, canonicalization profile, ambiguity review.
2. **Property-based tests** for attenuation and for "denied requests create no effects".
3. **Fuzz corpora** for the request envelope and manifest parsers.
4. **Adversarial AI corpora** (Gate 7, can be authored early).
5. **Implementation** of the reference model's missing boundaries.

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) for the scope-control template every feature proposal must use.

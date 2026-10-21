# SENVORYN OS

> **A person should be able to install, understand, control, and recover their computer without becoming a systems engineer.**
> Software runs inside a clearly bounded environment; intelligence assists the owner rather than ruling the machine; and every claim about security, privacy, speed, or compatibility is tied to testable evidence.

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Conformance: Prototype](https://img.shields.io/badge/conformance-Design--conformant-informational.svg)](evidence/local-validation-report.txt)
[![Status: Not production](https://img.shields.io/badge/status-NOT%20production-critical.svg)](docs/constitution/SENVORYN_OS_Constitution_v0.6.md)
[![CI](https://github.com/johalputt/SENVORYN-OS/actions/workflows/ci.yml/badge.svg)](https://github.com/johalputt/SENVORYN-OS/actions/workflows/ci.yml)
[![Security Policy](https://img.shields.io/badge/security-policy-blue.svg)](SECURITY.md)

---

## ⚠️ Read this first — what this repository is

**SENVORYN-OS currently contains a normative engineering contract and a small executable reference model. It does not contain a working operating system.**

| This repository **is** | This repository **is not** |
|---|---|
| A constitutional + contractual specification for an OS that does not exist yet | A kernel, bootloader, installer, or bootable image |
| 14 machine-readable JSON Schemas, 2 state machines, a 28-code denial registry | A cryptographic implementation, key store, or signature verifier |
| A deterministic policy evaluator in **Rust** and a reference model in **Python**, held to one shared vector set | A security boundary — **never deploy it as an SCP** |
| A traceability matrix mapping requirements to evidence | An audit, certification, penetration test, or compliance claim |
| A public record of decisions, open questions, and honest limits | A production release, or evidence of a vulnerability-free system |

The reference model uses simplified integer time values and trusted booleans. It does **not** verify signatures, secure identity, clocks, policy documents, canonicalization, hardware, or actual effects. Read [`docs/specs/README.md`](docs/specs/README.md) before using any of it.

We publish this early and honestly because a specification you can audit is more valuable than a binary you have to trust.

---

## Table of contents

- [What Senvoryn is](#what-senvoryn-is)
- [The five ideas](#the-five-ideas)
- [Repository layout](#repository-layout)
- [Quick start](#quick-start)
- [Architecture at a glance](#architecture-at-a-glance)
- [The core contracts](#the-core-contracts)
- [Conformance labels and current status](#conformance-labels-and-current-status)
- [Roadmap](#roadmap)
- [Conformance test identifiers](#conformance-test-identifiers)
- [Evidence and traceability](#evidence-and-traceability)
- [Contributing](#contributing)
- [Security](#security)
- [Governance](#governance)
- [Known limits and honest promises](#known-limits-and-honest-promises)
- [Related documents](#related-documents)
- [License](#license)

---

## What Senvoryn is

Senvoryn is a proposed **clean-sheet operating-system platform**. It is not a repackaged distribution and not a port of another operating system's kernel. Its distinctive goal is to unify five ideas into one coherent system:

| # | Idea | Meaning |
|---|---|---|
| 1 | **Intent-first computing** | People describe an outcome; the system converts it into an inspectable, bounded plan *before* acting. |
| 2 | **Capability-governed authority** | No process, app, driver, admin session, remote controller, or AI model receives authority merely because it asks for it. |
| 3 | **Origin Capsules** | Applications and imported execution environments get isolated, individually permissioned environments with explicit data and device bridges. |
| 4 | **Sovereign adaptation** | Local intelligence can diagnose, predict, optimize, and propose recovery while a small deterministic enforcement core retains final authority. |
| 5 | **Application-like installation** | On certified host systems, install begins like an ordinary signed app: no user-created boot media, existing data preserved unless replacement is explicitly requested, safe recovery after interruption. A reboot may still be required. |

The scope spans constrained edge hardware through personal computers to multi-node deployments, with one coherent object model and hardware-specific **conformance profiles**.

> **The first product milestone is not "AI that can control the OS."** It is *a small, deterministic authority core that can safely accept, constrain, audit, and reject requests from every other component — including AI.* Once that foundation is testable, the rest of the system can grow without making every new feature a new root of trust.

---

## The five ideas

<details>
<summary><strong>1 · Intent-first computing</strong> — click to expand</summary>

A user or an AI agent states an *intent*. The system compiles that intent into a **Change Capsule**: an immutable plan plus evidence plus narrowly scoped requested authority. It is not an executable script with root privilege. Its operations come from a versioned **Operation Registry**; unknown operation names are rejected.

A capsule moves through a strict state machine and can only reach `COMMITTED` if every mandatory postcondition is verified. If the outcome cannot be proven, the state is `PARTIAL` or `NEEDS_HUMAN_RECOVERY` — never `COMMITTED`.

</details>

<details>
<summary><strong>2 · Capability-governed authority</strong> — click to expand</summary>

A **capability** is a scoped authority grant binding `subject`, `resource`, `operations`, `purpose`, `constraints`, `not_before`, `expires_at`, `policy_epoch`, `issuer`, `delegation_depth`, `revocation_id`, and provenance proof.

- Default **deny**. Absence of a policy decision is not consent.
- A child capability may only be *equal or narrower* than its parent — never broader.
- Possession of a capability is necessary but **not always sufficient**: current system mode, risk policy, quota, target version, and consent must still pass.
- A capability does **not** override `READ_ONLY`, `RECOVERY`, `QUARANTINED`, or policy-revoked state.

</details>

<details>
<summary><strong>3 · Origin Capsules</strong> — click to expand</summary>

Every application, service, model, driver adapter, or imported runtime runs inside an **Origin Capsule** with its own immutable ID, provenance, storage namespace, network policy, device grants, IPC peers, resource budget, update policy, and audit policy.

The **default** capsule has no host administrator capability, no unrestricted host filesystem access, no access to another capsule's writable data, no network route, no secrets except brokered handles, no device access, no path to SCP internals beyond a narrow request API, no ability to modify trust roots or audit history, and finite CPU/memory/storage/process/IO/network budgets.

"Bridge available" is **not** "bridge authorized." A bridge is never enabled merely because a guest OS exposes the feature.

</details>

<details>
<summary><strong>4 · Sovereign adaptation</strong> — click to expand</summary>

The **Local Intelligence Fabric** may run models, interpret intent, predict failures, and propose plans. It has **no direct write path** to kernel, SCP, filesystem, secrets, or network. It requests tools through an independent broker that resolves the target, computes actual risk, validates capabilities, checks consent, and invokes the SCP.

Autonomy classes run `A0 Explain` → `A4 Critical`. **Autonomy class never bypasses SCP.** Model confidence is never an authorization proof. The system must remain fully operable with AI disabled.

</details>

<details>
<summary><strong>5 · Application-like installation</strong> — click to expand</summary>

The Direct Installer targets certified host configurations without user-created boot media. It verifies installer provenance **before** any elevated operation, identifies the exact target (not merely a disk label), displays effects, and separates non-destructive install, side-by-side install, virtualized trial, and destructive replacement.

Destructive operations require fresh strong authentication, a clear target identifier, explicit confirmation of the data-loss consequence, and a **second confirmation** if a mistake could destroy the only recovery copy.

</details>

---

## Repository layout

```
SENVORYN-OS/
├── README.md                    # you are here
├── ARCHITECTURE.md              # layers, trust boundaries, TCB, dependency rules
├── ROADMAP.md                   # gates 0–8, exit criteria, current position
├── GOVERNANCE.md                # decision-making, ADR process, anti-capture
├── CONTRIBUTING.md              # how to contribute, scope-control template
├── SECURITY.md                  # vulnerability disclosure, response targets
├── CODE_OF_CONDUCT.md           # community expectations
├── GLOSSARY.md                  # normative vocabulary
├── CHANGELOG.md                 # release history (Keep a Changelog)
├── RELEASE_PROCESS.md           # release gates and evidence bundle
│
├── docs/
│   ├── README.md                   # documentation index and reading order
│   ├── threat-model.md             # SCP decision-path threat model (first draft)
│   ├── specs/                      # Core Contract Specification v0.1.0 (+ pack README)
│   ├── constitution/               # Founding Constitution v0.6.0 (baseline)
│   └── component-specs/            # component-specification template
│
├── schemas/                     # 14 JSON Schemas (Draft 2020-12)
├── state-machines/              # change-capsule + system-mode transition tables
├── errors/                      # stable denial-code registry
├── examples/                    # request fixture, unsigned registry sample, CLI fixtures
├── conformance/vectors/         # shared conformance vectors — executed by BOTH implementations
├── conformance/capsule-vectors.json  # Change Capsule vectors — Rust implementation only
├── crates/                      # Rust workspace: canonical profile, SCP oracle, capsule executor, dev CLI
├── reference/                   # Python reference models: the SCP decision model and the capsule executor
├── tests/                       # 99 executable contract tests
├── decisions/                   # ADR-001 … ADR-016
├── evidence/                    # traceability matrix, validation report, claims ledger
├── scripts/                     # local verification helpers
│
└── .github/                     # CI, issue/PR templates, CODEOWNERS, dependabot
```

### The machine-readable contract

| Directory | Contents | Count |
|---|---|---|
| `schemas/` | JSON Schema Draft 2020-12 for principals, resources, requests, capabilities, decisions, approvals, intents, operations, Change Capsules, Origin Capsules, audit events, artifacts, updates, system states | 14 |
| `state-machines/` | Change Capsule lifecycle + system-mode transition tables | 2 |
| `errors/` | Stable initial denial-code registry (`DENY` / `CHALLENGE` defaults) | 28 codes |
| `examples/` | Request envelope fixture; **unsigned** sample operation registry; sample Change Capsule; CLI fixtures | 6 |
| `conformance/vectors/` | Conformance vectors executed by **both** the Rust and the Python implementation | 33 cases + 11 attenuation cases |
| `conformance/capsule-vectors.json` | Change Capsule vectors — executed by **both** the Rust and the Python implementation | 36 cases |
| `crates/` | Rust workspace — canonical JSON profile, SCP decision oracle, Change Capsule executor, development CLI | 4 crates, 0 third-party dependencies |
| `scripts/mutations.json` | The mutation set: one break per control, with the test that must catch it | 96 mutations |
| `decisions/` | Architecture Decision Records with open questions | 16 ADRs |
| `evidence/` | Requirement→test traceability matrix, local validation report, security claim ledger | — |

---

## Quick start

Requirements: **Python 3.11+** (3.12/3.13 verified) for the contract layer and the
reference model; **Rust 1.85+** (stable) for the workspace. The Rust workspace has
**no third-party dependencies**.

```bash
git clone https://github.com/johalputt/SENVORYN-OS.git
cd SENVORYN-OS

python -m venv .venv
# Linux/macOS:   source .venv/bin/activate
# Windows:      .venv\Scripts\Activate.ps1

pip install -r requirements-dev.txt

# Run the contract test suite (99 tests)
PYTHONPATH=reference python -m unittest discover -s tests -v

# Run the Rust workspace (119 tests)
cargo test --workspace

# Prove every control has a test that fails when it is broken
python scripts/mutation-gate.py
```

Expected tail of output:

```
Ran 84 tests in 0.285s

OK
```

On Windows PowerShell, set the path inline:

```powershell
$env:PYTHONPATH = "reference"
python -m unittest discover -s tests -v
```

### The Rust workspace

| Crate | What it is | What it is not |
|---|---|---|
| `senvoryn-canonical` | The Senvoryn Canonical JSON Profile `SCP-CJSON-0001`: parser, encoder, limits | Not a general JSON library — it rejects what the profile rejects |
| `senvoryn-scp` | The SCP decision oracle: the sixteen validation gates, the capability model, the audit-event builder | **Not an SCP.** It verifies no signature, resolves no resource against a filesystem, and writes no audit record |
| `senvoryn-capsule` | The Change Capsule executor simulator: the state machine, the idempotency ledger, the compensation and fault-injection model | **Not an executor.** It acts on a simulated environment, performs no effect, and decides no policy — it checks that the SCP's decision binds the capsule's own digest |
| `senvoryn-cli` | A development tool that drives the oracle from files | **Not an SCP**, and not deployable as one |
| `reference/capsule_reference.py` | The second implementation of the Change Capsule executor, in Python | **Not an executor**, and not a port of the Rust one. It is written from the contract, and it is what found two defects the Rust implementation had |

```bash
# Evaluate one request against one policy context
cargo run -p senvoryn-cli -- \
  --request examples/cli/request.json \
  --context examples/cli/context.json \
  --capability examples/cli/capability.json
```

The two implementations are held to one another by a shared vector set. Both suites
execute `conformance/vectors/vectors.json`, and CI fails if they produce a different
decision or a different reason code for the same case. That is the mechanism that
stops a prototype and an implementation from becoming two authorities on the same
question.

The Change Capsule vectors in `conformance/capsule-vectors.json` are held to one
another the same way. The capsule executor has two implementations — the Rust one
and the Python reference model — and both read the same file and both must reach
the state it names. The two are not a port of one another: the Python one raises
where the Rust one returns an option, and parses timestamps with the standard
library where the Rust one computes days from the civil calendar.

That is worth more than it sounds. The registry's merge rule kept the **more**
reversible of two conflicting rules instead of the less reversible one, and the
sample registry holds one rule per operation, so no test ever ran the merge. The
second implementation, written from the contract rather than from the first one,
found it. See [ADR-014](decisions/ADR-014-capsule-two-implementations.md).

The same agreement found the capsule's *missing* controls. Modelling a full
disk, a clock that rolls back, and a recovery artefact that turns out to be
corrupt found four things the executor did not have: the validity window was
checked once instead of being a boundary, a backwards clock was believed, a
failed write was recorded as a write that landed, and the compensation never
looked at the material it was about to use. See
[ADR-015](decisions/ADR-015-capsule-validity-window-and-a-moving-world.md).

The gate has also found controls that were **pointed at the wrong test**. Three
of the budget and registry controls were named after a test that exercised only
two of them, so the third survived the gate — correctly. A control that a test
does not reach is a control the test does not enforce, however carefully the
test is written. Each of them now has a test of its own. See
[ADR-016](decisions/ADR-016-operation-registry-limits-epoch-and-freshness.md).

### The mutation gate

A control that has never been seen to fail on the thing it forbids has proved
nothing. The mutation gate breaks each control in turn and requires a **named**
test to go red:

```bash
python scripts/mutation-gate.py
```

Ninety-six controls are covered — 24 of the policy layer, 3 properties of the
canonical JSON profile, 34 controls of the Change Capsule executor in Rust,
and 35 of them in the Python reference model. A surviving mutation is a
control that no test enforces, and the gate fails.

The gate has found four gaps the unit tests had missed: the "not yet valid" half
of a capability's validity window had no shared vector; the decision record's
`effect_performed` field was not asserted anywhere; the capsule's registry check
was satisfied by a control it did not actually exercise; and the capsule's
approval-requirement check had a vector that passed for the wrong reason.

It has also found controls that were *masked*: a check that fires at the same
moment as another one cannot be broken on its own, so two of the
recovery-material checks were indistinguishable from each other until they were
given different reasons and moved to the moments where each one actually
applies.

The gate now runs tests in two languages, and it removes the Python bytecode
cache before each run. A cached `.pyc` can shadow a mutated source, and the gate
would then report a control as untested when the control is fine — or credit a
kill to a mutation that did nothing. That is not hypothetical: it produced one
spurious survivor before the cache was removed.

The rules that make the gate meaningful are in
[`CONTRIBUTING.md`](CONTRIBUTING.md): a mutation must be caught by a *specific*
test and a *specific* case, and a mutation that does not compile is reported as
invalid rather than counted as a kill.

### Verify the shipped pack

```bash
# POSIX
bash scripts/verify-pack.sh

# Windows
pwsh -File scripts/verify-pack.ps1
```

### Continuous integration

Every push and pull request runs [`.github/workflows/ci.yml`](.github/workflows/ci.yml) on Python 3.12 and 3.13, and on Rust stable:

| Job | What it proves |
|---|---|
| `contract-tests` | All 84 contract/unit tests pass on both interpreters |
| `rust` | The Rust workspace is formatted, lint-clean under `clippy -D warnings`, fully tested, and builds in release mode |
| `conformance-agreement` | The Rust and the Python implementations produce the same decision and the same reason for every shared vector — the SCP's and the capsule's alike |
| `mutation-gate` | Every control is broken in turn and a **named** test is required to go red. A surviving mutation is a control no test enforces |
| `schema-validation` | All 14 schemas are valid Draft 2020-12 and fixtures conform |
| `pack-integrity` | SHA-256 checksums of the shipped contract artifacts match `SHA256SUMS.txt` |
| `json-lint` | Every JSON document in the repository parses |
| `claim-discipline` | Normative documents declare a status and contain no unmarked absolute security claims |
| `dependency-review` | Dependency changes on a PR are checked against known vulnerabilities and license policy |
| `yaml-lint` + `link-check` | Every YAML document parses; every relative Markdown link resolves |

> Passing CI proves the **contract package** is internally consistent. It proves nothing about an operating system, because no operating system exists here yet.

Branch protection on `main` requires the seven contract-pack checks to pass, blocks force-pushes and branch deletion, and requires conversations to be resolved before merge.

**Known gap:** the three Rust-facing jobs — `rust`, `conformance-agreement` and `mutation-gate` — are enforced by CI but are **not yet listed as required contexts**, because the credential available to this repository's tooling cannot write the branch-protection rule. Until they are listed, a change that breaks the Rust workspace, the agreement between the two implementations, or the mutation gate can still reach `main` by direct push. Adding them is a one-line change in the branch-protection settings, and the check names are exactly the job names in the table above.

---

## Architecture at a glance

Senvoryn is organized into logical layers. Implementations may map these to processes, protection domains, or hardware partitions, but **must preserve the authority boundaries**.

| Layer | Name | Primary responsibility | Authority boundary |
|---|---|---|---|
| **L0** | Hardware Adaptation Boundary (HAB) | CPU, memory, interrupts, timers, boot storage, device discovery | Exposes only detected and validated hardware capabilities |
| **L1** | Senvoryn Separation Core (SSC) | Address-space isolation, scheduling, IPC, interrupt routing, capability primitives | Smallest software authority base; no policy decisions from AI output |
| **L2** | Capability & Identity Authority (CIA) | Capability lifecycle, identity, grants, attenuation, revocation | May issue only policy-authorized, scoped capabilities |
| **L3** | System Control Plane (SCP) | Policy evaluation, state transitions, system configuration, action authorization | Cannot override kernel-enforced invariants |
| **L4** | Origin Capsule Runtime (OCR) | Creates and supervises isolated app/service environments | Each capsule receives only declared resources |
| **L5** | Universal Execution Fabric (UEF) | Native runtime, translation, guest environments, compatibility APIs | Guest permissions never become host permissions |
| **L6** | Local Intelligence Fabric (LIF) | Inference, intent interpretation, prediction, proposals | No direct privileged interface; all effects need SCP authorization |
| **L7** | User Experience Shell (UXS) | Installer, app launch, permission dialogs, settings, accessibility | Must faithfully display the *actual* policy decision and risk |
| **L8** | Federation & Continuity Layer (FCL) | Cross-device identity, state sync, fleet policy | Local node can reject remote commands and stay safe offline |
| **L9** | Developer & Assurance Toolchain (DAT) | Builds, package verification, testing, provenance, auditing | Build workers cannot unilaterally publish trusted releases |

**No implicit trust by layer.** A component's layer does not make its inputs trustworthy. Every layer validates inputs from less-trusted layers; every externally reachable parser is attack surface.

Full details: [`ARCHITECTURE.md`](ARCHITECTURE.md).

---

## The core contracts

The [Core Contract Specification v0.1.0](docs/specs/SENVORYN_Core_Contract_Spec_v0.1.md) converts constitutional principles into unambiguous data contracts, state machines, validation order, failure behavior, and testable obligations.

### The SCP validation order

Every privileged request passes **16 gates in a fixed order**. A rejection at any gate produces no privileged effect, and denial messages must not leak secret resource names or policy details to an unauthorized requester.

```
 1 Transport/size     →  2 Canonical parse  →  3 Recipient       →  4 Identity
 5 Freshness          →  6 Object resolution →  7 Revocation     →  8 Mode
 9 Policy epoch       → 10 Capability        → 11 Risk/consent   → 12 Budget
13 Precondition       → 14 Audit readiness   → 15 Decision       → 16 Effect separation
```

**The evaluator is pure.** It returns a decision and performs no effects. A separate executor re-checks authorization and transaction preconditions immediately before acting. *Authorizing is not executing.*

### Decision vocabulary

| Decision | Meaning | Grants authority? |
|---|---|---|
| `PERMIT` | All mandatory checks passed | Yes — narrowly scoped, finite, with an expiry |
| `DENY` | A mandatory check failed | No |
| `CHALLENGE` | More evidence or explicit approval required | No |
| `DEFER` | Non-destructive; waits on a dependency or policy window | No |
| `QUARANTINE` | A defined suspicious principal/request is isolated by policy | No |

`PERMIT` without a finite scope is invalid. `CHALLENGE` and `DEFER` grant no partial execution authority.

### Default failure behavior

| Condition | Result |
|---|---|
| Missing principal or capability | `DENY` |
| Unknown/stale policy epoch for a privileged action | `DENY` or `CHALLENGE` — never a silent permit |
| Revoked or expired capability | `DENY` |
| Unknown resource | `DENY` without dereferencing an untrusted path |
| Audit storage unavailable for a high-risk operation | `DENY` |
| SCP coordinator unavailable | No new high-risk grants; already-issued capabilities remain valid only within their original scope and until expiry/revocation |
| AI unavailable | Operate without AI |
| Remote controller unavailable | Enforce local policy; do **not** widen authority |
| Required hardware security property unknown | Reject the high-assurance profile or mark reduced assurance explicitly |

### Risk is determined by policy, not by the caller

The SCP derives the effective risk tier from a **signed, versioned Operation Registry**. A caller's risk claim is never authoritative. If the claim is *lower* than the registry result → `RISK_CLAIM_MISMATCH` (denied). Unknown operations have **no default low-risk classification** → `UNKNOWN_OPERATION_RISK` (denied). When multiple rules apply, the most restrictive tier wins.

### Change Capsule state machine

```
DRAFT → VALIDATED → AUTHORIZED → STAGED → PRECHECKED → APPLYING → VERIFYING → COMMITTED
                                                                    ↓
   REJECTED · EXPIRED · CANCELLED · FAILED · COMPENSATING · ROLLED_BACK · PARTIAL · NEEDS_HUMAN_RECOVERY
```

No transition may skip validation or authorization. `COMMITTED` requires **verified** postconditions. A missing verifier result is a failure, not a success.

### System modes

`NORMAL` · `DEGRADED` · `READ_ONLY` · `RECOVERY` · `QUARANTINED` · `MAINTENANCE` · `OFFLINE`

Modes are enforced on **every privileged request** — they are not UI labels. Every transition requires a named guard, initiating principal or detector, evidence reference, allowed actions, timeout, and exit test.

---

## Conformance labels and current status

The specification defines four conformance labels. **No system may advertise a higher label than its evidence supports.**

| Label | Meaning | Senvoryn today |
|---|---|---|
| **Design-conformant** | Schemas and algorithms meet the document; no running implementation claimed | ✅ **This repository** |
| **Prototype-conformant** | Reference implementation passes its declared tests in a limited test environment | ⚠️ Reference model only, in this repo |
| **Profile-conformant** | Implementation passes the complete suite for one named hardware/software profile | ❌ Not reached |
| **Release-assured** | Profile also has a reviewed threat model, independent security review, artifact provenance, recovery drills, published evidence | ❌ Not reached |

**Current conformance summary:** 14/14 schemas valid · 99/99 Python tests passing · 119/119 Rust tests passing · 33/33 shared vectors agreed · 36/36 capsule vectors agreed by two implementations · 96/96 mutations killed · 2/2 state machines structurally valid · 28/28 denial codes unique and cross-checked · **0** profile-conformant profiles · **0** independent reviews.

---

## Roadmap

Work is sequenced by dependency and evidence, not by launch pressure. **No gate is met because a calendar date was reached** — gate acceptance requires artifact links, results, review, and explicit limitations.

| Gate | Deliverable | Status |
|---|---|---|
| **0** | Contract freeze for prototype: identifiers, typed object model, schemas, threat model, state machine, error registry, reference evaluator, traceability matrix | 🟡 **In progress** — this repository is the Gate 0 deliverable |
| **1** | SCP decision oracle: pure evaluator, capability model, deterministic reason codes, audit-event builder, property tests, fuzzable parser boundary | 🟡 **In progress** — the Rust oracle, the canonical profile, the shared vectors, and the fuzz harness are in this repository; the signed Operation Registry and the resolution boundary are not |
| **2** | Change Capsule executor simulator: state machine, operation registry, idempotency ledger, stale-precondition detection, rollback/compensation simulation, fault injection | 🟡 **In progress** — the simulator, its fault model, its vectors, and two implementations that agree on all thirty vectors are in this repository; the executor performs no effect, and the compensation is modelled rather than exercised against storage |
| **3** | Capsule policy simulator: Origin Capsule manifest, bridge registry, budget checks, guest/host authority tests | ⬜ Not started |
| **4** | Minimal bootable prototype: one declared hardware/emulator profile, trusted boot chain, minimal service supervisor, local policy enforcement, recovery path | ⬜ Not started |
| **5** | Direct Installer prototype: one certified host profile, safe staging, cancellation, power-loss simulation, disk target confirmation, first-boot verification | ⬜ Not started |
| **6** | UEF proof of concept: one chosen foreign application profile, compatibility record, guest boundary, bridge policy, adversarial tests | ⬜ Not started |
| **7** | AI-native transaction layer: Intent Record, Action Envelope, model service boundary, tool broker, prompt-injection corpus, autonomy controls | ⬜ Not started |
| **8** | Security review and public alpha: independent review, vulnerability disclosure process, reproducible build report, SBOM, release signatures, threat model, performance data | ⬜ Not started |

Full gate definitions and exit criteria: [`ROADMAP.md`](ROADMAP.md).

### Near-term backlog

1. **Threat model** for the SCP decision path (`docs/threat-model.md`) — required for Gate 0 exit.
2. **Operation Registry v1** — promoted from the unsigned sample to a reviewed, versioned, signed artifact.
3. **Property-based tests** for capability attenuation (`CAP-001`) — now property tests over a seeded generator; still no external review.
4. **Fuzzable parser boundary** for the request envelope (Gate 1 prerequisite) — the canonical parser is fuzzed; the envelope→request mapping is not.
5. **Canonicalization profile** — named and specified as `SCP-CJSON-0001`; independent review before it is used to verify anything ([ADR-008](decisions/ADR-008-canonical-json-profile.md)).

---

## Conformance test identifiers

Test IDs are a **stable contract**. Detailed implementations may be split by language or environment, but these IDs do not change.

| Group | IDs | Covers |
|---|---|---|
| `SCP-001` … `SCP-008` | 8 | Denials, freshness, epoch binding, approval digests, evaluator purity, audit availability |
| `CAP-001` … `CAP-002` | 2 | Attenuation subset property, revocation blocking the next privileged step |
| `CC-001` … `CC-004` | 4 | Illegal capsule transitions, replay, failed postconditions, cancellation. The capsule vector file uses the same `CC-` group, with one case per control the executor enforces |
| `OC-001` … `OC-003` | 3 | Default capsule isolation, guest-admin privilege escalation, network default-deny |
| `AI-001` … `AI-004` | 4 | No direct capability issuance, digest-bound approval, remote-inference data-flow grant, prompt injection |
| `INS-001` … `INS-003` | 3 | Installer signature before elevation, explicit destructive confirmation, power-loss safety |
| `UPD-001` … `UPD-002` | 2 | Invalid signature/downgrade rejection, health-failure rollback |
| `PRIV-001` … `PRIV-002` | 2 | No nonessential telemetry on fresh install, secrets absent from logs/crash bundles |
| `FED-001` | 1 | Remote node command is locally re-authorized |
| `EVID-001` | 1 | Every release claim has evidence and limitations |

**30 IDs total.** The authoritative table with required evidence and priority is in [§16 of the Core Contract Specification](docs/specs/SENVORYN_Core_Contract_Spec_v0.1.md).

---

## Evidence and traceability

Every claim must be linkable to an artifact, a test, a reviewer, and a stated limitation.

```
Requirement  ──▶  Test ID  ──▶  Test artifact  ──▶  Result  ──▶  Reviewer  ──▶  Limitations
```

| Artifact | Purpose |
|---|---|
| [`evidence/traceability.csv`](evidence/traceability.csv) | Requirement → test → evidence → owner → status |
| [`evidence/local-validation-report.txt`](evidence/local-validation-report.txt) | What the local tests and schema checks actually proved — and what they did not |
| [`evidence/security-claim-ledger-template.md`](evidence/security-claim-ledger-template.md) | Template for publishing a security claim; every field must be filled before publication |
| [`evidence/claims-ledger.md`](evidence/claims-ledger.md) | Live ledger classifying each claim `PROPOSED` / `IMPLEMENTED` / `TESTED` / `INDEPENDENTLY REVIEWED` / `FORMALLY VERIFIED WITH SCOPE` / `WITHDRAWN` |

**A passing unit test is not evidence of the absence of all vulnerabilities.** Evidence claims must match test scope.

---

## Contributing

Senvoryn is at the specification stage, so contributions are weighted toward **contracts, models, tests, and evidence** rather than large code volumes.

- **Read** [`CONTRIBUTING.md`](CONTRIBUTING.md) for the full workflow, the scope-control template, and the two-reviewer rule for security-critical changes.
- **Every security-sensitive change needs** a recorded rationale, review, testing evidence, and a versioned decision record ([INV-20](docs/constitution/SENVORYN_OS_Constitution_v0.6.md)). If your change touches an invariant, it needs an ADR.
- **Do not weaken an invariant for convenience.** Lower-level artifacts may narrow a capability or add checks; they must not broaden authority without a formally approved constitutional amendment.
- **Governance:** [`GOVERNANCE.md`](GOVERNANCE.md) · **Code of conduct:** [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)

---

## Security

**Do not open a public issue for a security vulnerability.**

Report privately via [`SECURITY.md`](SECURITY.md) — it documents the reporting channel, severity triage criteria, coordinated-disclosure process, and response targets. Critical defects in the core, capability enforcement, signing, installer, or update path can block releases.

---

## Governance

Senvoryn's governance is designed to resist capture by any single person or company.

- **Separation of duties** — no one actor can author code, approve it, build it, sign it, and publish a release.
- **Multi-party release signing** — role-separated keys: source administration, build attestation, release authorization, emergency revocation.
- **Public decision records** — ADRs, spec proposals, advisories, release notes, and governance votes are public.
- **Constitutional amendment** — public proposal, threat/compatibility analysis, ≥30-day review, independent review for security-sensitive changes, ≥75% supermajority, documented migration and test plan.

Full text: [`GOVERNANCE.md`](GOVERNANCE.md) · [`decisions/ADR-001…ADR-007`](decisions/).

---

## Known limits and honest promises

We cannot honestly promise absolute security, absolute privacy, no backdoor under every conceivable threat, zero bugs, unlimited scale, or universal software compatibility. Senvoryn's promise is stronger in a different way: **define each guarantee, disclose its assumptions, make evidence reproducible, and withdraw a claim if the evidence no longer supports it.**

1. **"Run any software" is a goal, not a universal guarantee.** Software may depend on unimplemented APIs, proprietary services, device drivers, signed hardware, licenses, DRM, or unavailable hardware.
2. **"Install with one click"** means no user-created boot media on *certified* host profiles. Not no reboot, no permission, no storage change, no data-loss risk.
3. **"No backdoors" is a governance and verification objective**, not a provable property. Source availability, signed builds, independent rebuilds, and audits reduce risk but cannot prove absence of every compromise.
4. **"Works on any device"** means a portable architecture and an expandable profile program. Unsupported hardware must fail safely and say why.
5. **"AI-native"** does not mean AI controls everything. Core control remains deterministic, constrained, and operable without models.
6. **"Full control"** does not authorize violating another vendor's technical protections, licenses, or lawful access controls.
7. **"Lightweight and superfast" must be measured.** Isolation, virtualization, translation, and local models all have costs. We publish profile-specific trade-offs instead of pretending overhead is zero.
8. **"New from the foundation" does not mean inventing new cryptography.** Novelty belongs in architecture, control contracts, and user experience; core cryptographic constructions stay publicly scrutinized.
9. **The mutation gate proves the controls it names and no others.** Sixty controls are broken in turn today. The installer, update and AI-broker controls have no implementation to break yet, and a mutation that the project wrote is not an independent review.
10. **The capsule executor performs no effect.** It drives the state machine against a simulated environment, so it proves the transaction's bookkeeping and its refusals, not that a real system can be rolled back. Compensation is modelled, not exercised against storage.
11. **Two implementations of one transaction still agree on one contract.** The Rust and Python capsule executors are held to the same vector file, but the vector file is the contract, and whoever writes it decides what "right" means. Agreement between two implementations of the same specification is a strong check and not a proof, and neither implementation is an independent reviewer.

Complete list: [Constitution Article 44](docs/constitution/SENVORYN_OS_Constitution_v0.6.md).

---

## Related documents

| Document | ID / Version | Purpose |
|---|---|---|
| [Core Contract Specification](docs/specs/SENVORYN_Core_Contract_Spec_v0.1.md) | `SENVORYN-CORE-SPEC-0001` v0.1.0 | Normative engineering contract: data contracts, state machines, validation order, failure behavior, testable obligations |
| [Founding Constitution](docs/constitution/SENVORYN_OS_Constitution_v0.6.md) | v0.6.0 | Top-level design authority: 30 constitutional invariants, architecture, governance, acceptance criteria |
| [Canonical JSON Profile](docs/canonical-json-profile.md) | `SCP-CJSON-0001` v1 | The one byte encoding for every signed, hashed, or compared protocol object |
| [Mutation set](scripts/mutations.json) | `senvoryn.mutation-set.v1` | One break per control, and the named test that must catch it |
| [Capsule vectors](conformance/capsule-vectors.json) | `senvoryn.capsule-vectors.v1` | The state the Change Capsule executor must reach, case by case |
| [Architecture overview](ARCHITECTURE.md) | — | Layers, trust boundaries, TCB inventory, dependency direction rules |
| [Roadmap](ROADMAP.md) | — | Gates 0–8 with deliverables and exit criteria |
| [Glossary](GLOSSARY.md) | — | Normative vocabulary across both documents |
| [Architecture Decision Records](decisions/) | ADR-001…014 | Accepted decisions and explicitly open questions |
| [Changelog](CHANGELOG.md) | — | Versioned history of contract, schema, and model changes |

### Document hierarchy

When two rules conflict, precedence is:

```
1. Constitutional invariants          (highest — may not be weakened)
2. Core Contract Specification
3. Versioned component specifications
4. Architecture Decision Records
5. Implementation details             (lowest)
```

Lower-level artifacts may **narrow** a capability or add checks. They must **not** broaden authority or weaken a higher-level rule without a formally approved constitutional amendment. If two rules conflict, the privileged operation is rejected, a conflict code is recorded, and a specification decision is required.

---

## License

Original Senvoryn code and documentation are released under the **Apache License 2.0** — see [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

This is the **default license for original code only**. Component-level licenses remain separate and are disclosed individually. Every release must ship a machine-readable **SBOM and license inventory**. We do not claim that all dependencies are Apache-2.0.

Cryptographic constructions remain publicly scrutinized primitives — see [ADR-002](decisions/ADR-002-crypto-boundary.md).

---

<div align="center">

**Senvoryn is engineered to explain what it intends to do, prove that the actor has authority, constrain the effects, check the result, preserve the evidence, and offer recovery when the result is wrong.**

[Report a vulnerability](SECURITY.md) · [Contribute](CONTRIBUTING.md) · [Governance](GOVERNANCE.md) · [Roadmap](ROADMAP.md)

</div>

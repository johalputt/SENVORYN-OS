# Architecture

**Status:** Proposed — describes the *target* architecture. Only L3 (SCP) has a
reference model, and that model is not a security boundary.

> Normative reference: [Founding Constitution v0.6.0](docs/constitution/SENVORYN_OS_Constitution_v0.6.md) Part II (Articles 7–27) and Part VII (Articles 46–48).
> Implementation contract: [Core Contract Specification v0.1.0](docs/specs/SENVORYN_Core_Contract_Spec_v0.1.md).

This document explains **how Senvoryn is structured** and — just as importantly — **where trust ends**. It describes the target architecture. No layer below L3 has an implementation in this repository yet.

---

## 1. Design rules

These rules constrain every design decision in the project.

1. **Default deny.** An action without a valid grant is denied. Absence of a policy decision is not consent. *(INV-01)*
2. **Authority is explicit.** Every privileged effect is attributable to an authenticated principal, a specific capability, a declared purpose, and an applicable policy version. *(INV-02)*
3. **Authority cannot self-escalate.** A component cannot create, widen, or grant itself capabilities. Delegation may only preserve or reduce the delegator's authority. *(INV-03)*
4. **AI is never the root of trust.** Model output is untrusted input until parsed, validated, authorized, and constrained by deterministic policy. *(INV-04)*
5. **No implicit trust by layer.** A component's layer does not make its inputs trustworthy. Every layer validates inputs from less-trusted layers, and every externally reachable parser is treated as attack surface.
6. **Minimize the TCB.** The project MUST minimize the trusted computing base and explain every expansion of it.
7. **Separate authorization from execution.** The evaluator is pure. Authorizing an action is not performing it.

---

## 2. Layer model

Implementations may map these layers to separate processes, protection domains, or hardware partitions, but **must preserve the defined authority boundaries**.

```
                 ┌───────────────────────────────────────────────┐
  L9  DAT        │ Developer & Assurance Toolchain              │  builds, provenance, audit
                 ├───────────────────────────────────────────────┤
  L8  FCL        │ Federation & Continuity Layer                │  cross-device identity, fleet
                 ├───────────────────────────────────────────────┤
  L7  UXS        │ User Experience Shell                        │  installer, dialogs, settings
                 ├───────────────────────────────────────────────┤
  L6  LIF        │ Local Intelligence Fabric                   │  models, intent, proposals
                 ├───────────────────────────────────────────────┤
  L5  UEF        │ Universal Execution Fabric                  │  native, translation, guest
                 ├───────────────────────────────────────────────┤
  L4  OCR        │ Origin Capsule Runtime                      │  isolated app/service sandboxes
                 ├───────────────────────────────────────────────┤
  L3  SCP        │ System Control Plane                        │  policy evaluation, decisions
                 ├───────────────────────────────────────────────┤
  L2  CIA        │ Capability & Identity Authority             │  capability lifecycle, revocation
                 ├───────────────────────────────────────────────┤
  L1  SSC        │ Senvoryn Separation Core                    │  isolation, scheduling, IPC
                 ├───────────────────────────────────────────────┤
  L0  HAB        │ Hardware Adaptation Boundary                │  CPU, memory, devices, boot
                 └───────────────────────────────────────────────┘
```

| Layer | Name | Primary responsibility | Authority boundary |
|---|---|---|---|
| **L0** | Hardware Adaptation Boundary (HAB) | CPU, memory, interrupts, timers, boot storage and device discovery | Exposes only detected and validated hardware capabilities |
| **L1** | Senvoryn Separation Core (SSC) | Address-space isolation, scheduling, IPC, interrupt routing, capability primitives | Smallest software authority base; no policy decisions based on AI output |
| **L2** | Capability and Identity Authority (CIA) | Capability object lifecycle, identity, grants, attenuation and revocation | May issue only policy-authorized and scoped capabilities |
| **L3** | System Control Plane (SCP) | Policy evaluation, state transitions, system configuration and action authorization | Cannot override kernel-enforced invariants |
| **L4** | Origin Capsule Runtime (OCR) | Creates and supervises isolated app/service environments | Each capsule receives only declared resources |
| **L5** | Universal Execution Fabric (UEF) | Native runtime, translation, guest environments and compatibility APIs | Guest permissions never become host permissions |
| **L6** | Local Intelligence Fabric (LIF) | Model inference, intent interpretation, prediction and proposals | No direct privileged interface; all effects require SCP authorization |
| **L7** | User Experience Shell (UXS) | Installer, app launch, permission dialogs, settings, accessibility | Must faithfully display the actual policy decision and risk |
| **L8** | Federation and Continuity Layer (FCL) | Cross-device identity, state synchronization and fleet policy | Local node can reject remote commands and remain safe offline |
| **L9** | Developer and Assurance Toolchain (DAT) | Builds, package verification, testing, provenance and auditing | Build workers cannot unilaterally publish trusted releases |

### 2.1 Minimal trusted core

The Separation Core (L1) MUST NOT contain a general-purpose shell, AI inference engine, web renderer, package downloader, application UI toolkit, or complex filesystem parser. It implements only the primitives needed to:

- create, switch, and destroy address spaces;
- map and unmap memory with explicit rights;
- create, destroy, and schedule protected execution contexts subject to declared constraints;
- deliver interrupts and timers to authorized service endpoints;
- establish message channels and bounded shared-memory grants;
- invoke unforgeable kernel object handles.

> **Why so small?** A large kernel turns every bug into a privilege-escalation bug and makes independent review impractical. The TCB is the thing we are least allowed to get wrong.

### 2.2 Service failure isolation

Filesystem, network, device drivers, app lifecycle, model inference, compatibility bridges, and presentation are intended to run **outside** the highest-trust core whenever the target hardware permits. A fault in one service must be recoverable without rebooting unrelated services. If hardware or early-boot constraints force a function into the core, that exception **must be documented and reviewed**.

---

## 3. Trusted computing base (TCB)

Every release MUST publish a TCB inventory identifying all components whose compromise could violate a named security claim.

| TCB component | Compromise would allow | Required properties |
|---|---|---|
| Boot roots | Arbitrary pre-OS code execution | Measured/verified, hardware-backed where claimed, revocable |
| Separation core (L1) | Total system compromise | Memory safety strategy, minimal surface, independent review |
| Capability enforcement (L2) | Forged or widened authority | Unforgeable handles, attenuation enforced at issue time |
| Policy evaluator (L3) | Wrong decisions on every request | Pure function, no side effects, deterministic reason codes |
| Canonical JSON parser | Two byte sequences for one value, or an unbounded allocation | Strict grammar, limits checked before allocation, no third-party code ([ADR-009](decisions/ADR-009-zero-dependency-trusted-core.md)) |
| Strict UTC instant parser | Two readings of one timestamp, or a document's window judged wrongly | One form accepted, one arithmetic, used by every layer that reads a time ([ADR-012](decisions/ADR-012-capsule-utc-timestamp-boundary.md)) |
| Key-handling boundaries | Forged signatures, forged identity | Role-separated keys, rotation, revocation, ceremony |
| Update verifier | Persistent malicious code | Authenticated metadata, content addressing, anti-rollback |
| Capsule isolation mechanisms | Host compromise from a guest | Explicit, typed, revocable bridges; no implicit host paths |
| Hardware/firmware assumptions | Hardware-level bypass | Stated explicitly per profile; reduced assurance declared when absent |

**The project MUST minimize the TCB and explain every expansion.** If secure boot is unavailable, the system reports a reduced assurance profile rather than falsely claiming equivalent protection.

---

## 4. Dependency direction rules

Dependencies point **strictly downward**. A layer may call only into layers below it, through a declared interface.

```
L9 ─┐
L8  │
L7  │   may request effects ──▶ L3 (SCP) ──▶ L2 (CIA) ──▶ L1 (SSC) ──▶ L0 (HAB)
L6  │   (proposals only) ─────▶ ▲
L5  │                           │
L4  │                           │
L3 ─┴───────────────────────────┘
```

| Rule | Statement |
|---|---|
| **DEP-01** | L6 (LIF) MUST NOT have a direct write path to L1, L2, L3, storage, secrets, or network. It reaches them only through the tool broker. |
| **DEP-02** | L5 (UEF) guest permissions MUST NOT become host permissions. A bridge is explicit, typed, leased, logged, and revocable. |
| **DEP-03** | L4 (OCR) MAY NOT modify trust roots or audit history. |
| **DEP-04** | L3 (SCP) MAY NOT override L1/L2 kernel-enforced invariants, and MUST NOT execute arbitrary code merely because it is a control plane. |
| **DEP-05** | L1 (SSC) MUST NOT contain policy logic, AI inference, UI toolkits, or package downloaders. |
| **DEP-06** | L0 (HAB) MUST NOT expose capabilities it has not detected and validated. |
| **DEP-07** | L8 (FCL) MAY propose desired state to L3, but the local node re-authorizes every operation. |
| **DEP-08** | L9 (DAT) MUST NOT be able to unilaterally publish a trusted release. |

---

## 5. Authority flow

How a request becomes an effect:

```
  Principal (authenticated)
        │
        │  typed request envelope
        ▼
  ┌───────────────────────────┐
  │ SCP policy evaluator (L3) │  PURE — no side effects
  │  16 gates, fixed order    │
  └───────────────────────────┘
        │  decision: PERMIT / DENY / CHALLENGE / DEFER / QUARANTINE
        │  + narrowly scoped, finite, expiring capability
        ▼
  ┌───────────────────────────┐
  │ Executor (L4 / L2)        │  RE-CHECKS authorization + preconditions
  │  effect isolation         │  immediately before acting
  └───────────────────────────┘
        │
        ▼
  Effect  ──▶  Audit event (append-only)  ──▶  Evidence
```

Three properties hold at every step:

1. **The evaluator performs no effects.** A reviewer can confirm this by reading `crates/senvoryn-scp/src/evaluator.rs` or `reference/scp_reference.py` — each is a pure function with no I/O. The Rust implementation's decision type has no field in which an effect could be recorded.
2. **The executor re-checks.** Authority is validated again immediately before the effect, closing the time-of-check/time-of-use gap.
3. **Effect and authorization are separated.** A decision authorizes; it does not execute.

> The decision functions in this repository are **decision functions, not an SCP**. They hold no trust store, verify no signature, resolve no resource against a filesystem, and write no audit record. The boundary between a decision function and an SCP is a cryptographic and storage boundary that does not exist here yet.

---

## 6. System modes

System modes constrain what may happen at all. They are **enforced on every privileged request**, not just displayed in the UI.

| Mode | Meaning | Effect on writes |
|---|---|---|
| `NORMAL` | All baseline policy services available | Allowed with grant |
| `DEGRADED` | Noncritical features disabled or bounded due to pressure/faults | Bounded |
| `READ_ONLY` | Writes blocked except explicitly allowed recovery logging and essential state | Blocked |
| `RECOVERY` | Only verified recovery operations permitted | Recovery only |
| `QUARANTINED` | Affected principal/device/component constrained after a policy-defined trigger | Constrained |
| `MAINTENANCE` | Approved administrative changes allowed under increased audit | Allowed with grant |
| `OFFLINE` | Network unavailable; local policy remains effective | Unchanged |

Machine-readable guards: [`state-machines/system-mode-transitions.json`](state-machines/system-mode-transitions.json).

**Fail-safe vs fail-operational.** Security-critical functions default fail-safe: deny an unsafe operation. Availability-critical services may fail-operational only under a profile-specific, preapproved policy with bounded privileges and documented consequences. **There is no generic "ignore security to keep running" switch.**

---

## 7. Origin Capsule boundary

Each capsule is a security domain with its own immutable ID, trust/provenance metadata, runtime profile, storage namespace, network policy, device grants, IPC peers, resource budget, update policy, audit policy, and compatibility status.

### Default posture

- no host administrator capability
- no unrestricted host filesystem access
- no access to another capsule's writable data
- no network route unless granted
- no access to secrets except specific brokered secret handles
- no device/sensor access unless granted
- no access to SCP internals beyond a narrow request API
- no ability to modify trust roots or audit history
- finite CPU, memory, storage, process count, I/O, and network budgets

### Bridges

Filesystem sharing, clipboard, screen capture, camera, microphone, location, GPU, USB, display, accessibility, and network are **explicit bridges**. Each defines direction, operations, data label, lease, user indicator, budget, logging class, and revocation semantics.

> **"Bridge available" is not "bridge authorized."** A bridge MUST NOT be enabled simply because a guest OS exposes that feature.

### Lifecycle

```
CREATED → VERIFIED → CONFIGURED → READY → RUNNING → STOPPING → STOPPED
```

Exceptional: `QUARANTINED`, `CRASHED`, `REVOKED`, `REMOVAL_PENDING`. A capsule crash does **not** imply host crash.

Machine-readable schema: [`schemas/origin-capsule.schema.json`](schemas/origin-capsule.schema.json).

---

## 8. AI boundary

The Local Intelligence Fabric (L6) is a **proposal producer, not an authority** ([ADR-004](decisions/ADR-004-ai-authority.md)).

| Autonomy class | Capability | Guardrail |
|---|---|---|
| `A0 Explain` | Read-only within granted scope | No effects |
| `A1 Recommend` | Create a plan | No effects |
| `A2 Bounded reversible` | Execute allowlisted low-risk actions | Budget + rollback constraints |
| `A3 Pre-authorized workflow` | Execute a narrow recurring workflow the user configured | Notify and stop on any out-of-scope condition |
| `A4 Critical` | Fresh strong human approval per critical transaction | **No default autonomous mode** |

- AI planning has **read access only** to authorized context and no direct write path to kernel, SCP, filesystem, secrets, or network.
- The model's risk claim is **not authoritative** — the trusted Operation Registry decides.
- UI confirmations are generated from canonical envelope fields, not from free-form model prose.
- The model cannot alter a confirmation after approval without invalidating the action digest.
- Users can reduce autonomy immediately. Increasing it requires a visible change summary and new approval.
- **The system must operate with AI disabled.** The AI-disabled core test suite is a release requirement.

---

## 9. Current implementation reality

| Layer | Status in this repository |
|---|---|
| L0–L2 | Not implemented. Only machine-readable schemas and ADRs exist. |
| L3 | **Reference model only** — `reference/scp_reference.py`, a deterministic toy evaluator with 16 gates and simplified trusted primitives. Not a security boundary. |
| L4–L9 | Not implemented. Schemas and state machines define the intended contracts. |

What *is* implemented and tested today:

- 14 JSON Schemas (Draft 2020-12) that validate structurally
- 2 machine-readable state machines
- 27 stable denial codes with `DENY`/`CHALLENGE` defaults
- 1 unsigned sample operation registry
- 1 pure reference evaluator + 29 passing contract tests
- A traceability matrix mapping 30 conformance test IDs to evidence and status

---

## 10. Design records

Architecture decisions and their open questions live in [`decisions/`](decisions/). Each ADR is immutable once accepted; changes to an accepted decision produce a **new** ADR that supersedes it.

| ADR | Subject | Status |
|---|---|---|
| [ADR-001](decisions/ADR-001-separation-core.md) | Clean-sheet separation core | Proposed for prototype only |
| [ADR-002](decisions/ADR-002-crypto-boundary.md) | Cryptographic boundary | Proposed |
| [ADR-003](decisions/ADR-003-protocol-encoding.md) | Canonical protocol encoding | Prototype may use deterministic JSON; production pending review |
| [ADR-004](decisions/ADR-004-ai-authority.md) | AI is a proposal producer, not an authority | Proposed |
| [ADR-005](decisions/ADR-005-universal-execution.md) | Universal Execution Fabric | Proposed |
| [ADR-006](decisions/ADR-006-direct-install.md) | Direct installation | Proposed |
| [ADR-007](decisions/ADR-007-local-first-authority.md) | Local-first authority | Proposed |

---

## 11. See also

- [Core Contract Specification v0.1.0](docs/specs/SENVORYN_Core_Contract_Spec_v0.1.md) — the normative implementation contract
- [Founding Constitution v0.6.0](docs/constitution/SENVORYN_OS_Constitution_v0.6.md) — the top-level design authority
- [GLOSSARY.md](GLOSSARY.md) — normative vocabulary
- [ROADMAP.md](ROADMAP.md) — gates and exit criteria
- [`decisions/`](decisions/) — architecture decision records

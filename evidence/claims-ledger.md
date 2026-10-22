# Claims Ledger

**Status:** Live. **Last updated:** 2026-10-22.

The Claims Ledger is the **single source of truth for what this project may publicly claim**. Documentation, README text, release notes, and marketing must never advertise a status stronger than the ledger shows.

> *No design can honestly promise absolute security, absolute privacy, no backdoor under every conceivable threat, zero bugs, unlimited scale, or universal software compatibility. Senvoryn's promise is stronger in a different way: define each guarantee, disclose its assumptions, make evidence reproducible, and withdraw a claim if the evidence no longer supports it.*

---

## Status definitions

| Status | Meaning |
|---|---|
| `PROPOSED` | Written down. Not implemented, not tested. |
| `IMPLEMENTED` | Code or artifact exists and is in the repository. |
| `TESTED` | An automated test exercises the behavior and passes in CI. |
| `INDEPENDENTLY REVIEWED` | A reviewer who did not author the work has reviewed it and signed off. |
| `FORMALLY VERIFIED WITH SCOPE` | Machine-checked proof exists, **with its assumptions and scope stated**. Never a claim of full correctness. |
| `WITHDRAWN` | The claim is no longer supported. It is kept visible, not deleted. |

---

## Ledger

### Constitutional invariants

| Claim ID | Claim | Evidence | Status | Limitations |
|---|---|---|---|---|
| INV-01 | Default deny — an action without a valid grant is denied | `reference/scp_reference.py`, `tests/test_scp_reference.py` | `TESTED` | Test-level only; reference model is not a security boundary |
| INV-02 | Authority is explicit — every privileged effect is attributable to a principal, capability, purpose, policy version | `schemas/capability.schema.json`, `schemas/request-envelope.schema.json` | `TESTED` | Structural schema conformance only |
| INV-03 | Authority cannot self-escalate — delegation only narrows | `reference/scp_reference.py::attenuate` | `TESTED` | Partial: hand-written subset check, no property-based suite, no per-resource-type subset semantics |
| INV-04 | AI is never the root of trust | Specification §8; ADR-004 | `PROPOSED` | Not implemented; no AI layer exists |
| INV-05 | No silent destructive action | Specification §3, §5 | `PROPOSED` | Not implemented; no approval UI, no digest binding code |
| INV-06 | No hidden telemetry | Specification §10 | `PROPOSED` | Not implemented; no network egress test exists |
| INV-07 | Local autonomy under disconnection | ADR-007; specification §4.3 | `PROPOSED` | Not implemented; maximum stale-authority window not yet published |
| INV-08 | Isolation by construction | Specification §6 | `PROPOSED` | Not implemented; no capsule runtime exists |
| INV-09 | Separation of duties | `GOVERNANCE.md` | `PROPOSED` | Process only; no multi-party tooling enforced by CI yet |
| INV-10 | Verifiable updates | `schemas/update-plan.schema.json` | `PROPOSED` | Schema only; no updater exists |
| INV-11 | Rollback is not erasure | Specification §7.5 | `PROPOSED` | Not implemented |
| INV-12 | Data and configuration are distinct | Specification §10.4 | `PROPOSED` | Not implemented |
| CC-01 | An interrupted Change Capsule never reports COMMITTED | `crates/senvoryn-capsule/tests/fault_injection.rs`; `conformance/capsule-vectors.json` | `TESTED` | Simulated environment only; the compensation is modelled, not exercised against real storage |
| CC-02 | A decision permits only the capsule it was made about | `crates/senvoryn-capsule/src/authorization.rs`; capsule vector cases CC-007, CC-009 | `TESTED` | The decision is taken as an input; the SCP is not re-implemented here, so this is a binding check and not a policy decision |
| CC-03 | Every control the capsule implements has a test that fails when the control is broken | `scripts/mutations.json`; `scripts/mutation-gate.py` | `TESTED` | The gate is written by the same project as the code it checks; it is not an independent review |
| CC-04 | Two independent implementations of the capsule executor agree on every case | `reference/capsule_reference.py`; `crates/senvoryn-capsule`; `conformance/capsule-vectors.json` | `TESTED` | Both implementations read the same vector file, and whoever writes it decides what "right" means. Agreement between two implementations of one specification is a strong check and not a proof |
| CC-05 | A capsule is applied only inside its validity window, and a transaction whose world moves under it is refused rather than believed | `crates/senvoryn-capsule/src/executor.rs`; `reference/capsule_reference.py`; `conformance/capsule-vectors.json` cases CC-031…CC-035; `scripts/mutations.json` | `TESTED` | The clock is the simulator's own, advanced by each step's timeout; a real time source is not modelled |
| SCP-RESOLUTION | A decision judges the canonical identity of a resource — its identifier, its type and its version — and never the caller's own strings | `crates/senvoryn-scp/src/resolution.rs`; `crates/senvoryn-scp/src/evaluator.rs`; `reference/scp_reference.py`; conformance vectors `gate-06-resource-type-mismatch`, `gate-06-a-caller-cannot-claim-a-wider-type`, `gate-06-an-alias-resolves-to-the-canonical-object`, `gate-10-an-alias-cannot-reach-another-grant`; ADR-018 | `TESTED` | The environment's resolution table is supplied to the oracle; nothing in this repository resolves a name against a filesystem, and the resolution is not yet designed per resource type |
| SCP-ENVELOPE | A request is read, not judged: the policy request form is closed, and a document carrying a field the form does not define is refused rather than read | `crates/senvoryn-scp/src/request.rs`; `crates/senvoryn-scp/tests/envelope.rs`; `crates/senvoryn-scp/tests/fuzz.rs`; nine mutations in `scripts/mutations.json`; ADR-019 | `TESTED` | The policy request form is the transport layer's output and not the wire envelope the schema describes; the transport does not exist yet, so the wire envelope has no reader, and the replay key, the target node, the caller's classification and effect claims and the approval binding have no home in the form |
| CC-VERSION | A decision authorises an effect on the version of the target it was made against: the executor refuses a decision that names no version, or names another | `crates/senvoryn-capsule/src/authorization.rs`; `reference/capsule_reference.py`; `conformance/capsule-vectors.json` cases CC-037 and CC-038; four mutations in `scripts/mutations.json`; ADR-020 | `TESTED` | The version and the plan now arrive from the decision record rather than from a value the host assembled (ADR-021, see SCP-RECORD); the approvals do not |
| SCP-RECORD | The executor's authority is the policy layer's decision, not a value the host assembles: the request may name the plan it asks to run, the record carries that plan, and the executor reads the plan, the version and the status out of the record | `crates/senvoryn-scp/src/record.rs`; `crates/senvoryn-scp/src/decision.rs`; `crates/senvoryn-capsule/src/authorization.rs`; `reference/capsule_reference.py`; `crates/senvoryn-scp/tests/record.rs`; `conformance/capsule-vectors.json` cases CC-039 and CC-040; nine mutations in `scripts/mutations.json`; ADR-021 | `TESTED` | The approvals are still passed to the executor beside the record, because the policy layer is given the approval's digest and not the approval; the record's validity window and capability scope are read and not compared; and the record is not signed, so it is trusted because the policy layer wrote it inside the trust boundary, not because it is proved |
| SCP-DEFER | A request that names a policy epoch this node has not reached is deferred, not denied: only a non-destructive operation may be deferred, the deferral names the epoch it waits for, grants no authority, and does not outlive the request it defers | `crates/senvoryn-scp/src/evaluator.rs`; `crates/senvoryn-scp/src/decision.rs`; `reference/scp_reference.py`; conformance vectors `gate-09-a-future-policy-epoch-is-deferred`, `gate-09-a-destructive-operation-is-not-deferred`, `gate-09-an-irreversible-operation-is-not-deferred`, `gate-09-an-external-effect-is-not-deferred`, `gate-09-an-unknown-operation-is-not-deferred`, `gate-09-a-deferral-does-not-outlive-the-request`; `conformance/capsule-vectors.json` case CC-041; thirteen mutations in `scripts/mutations.json`; ADR-022 | `TESTED` | The deferral reserves nothing: when the epoch arrives the request is decided again from the top against that epoch's registry, and nothing in the record says that to the caller. The deferral's expiry cannot be published as a date-time, because the request's expiry is a monotonic instant and the boot-epoch bridge does not exist. A registry entry that contradicts itself is refused only where a rule reads both halves of it, and the registry's per-entry consistency is not checked |
> | SCP-QUARANTINE | A request from a principal this node holds under quarantine is quarantined, not merely denied: the isolation is settled at the identity gate, the quarantined principal is still a known principal, the quarantine names the principal it holds, it grants no authority, and no other failure falls back to it | `crates/senvoryn-scp/src/evaluator.rs`; `crates/senvoryn-scp/src/decision.rs`; `crates/senvoryn-scp/src/context.rs`; `reference/scp_reference.py`; conformance vectors `gate-04-a-quarantined-principal-is-quarantined`, `gate-04-a-quarantined-principal-is-quarantined-even-when-its-request-has-expired`, `gate-04-a-principal-the-node-does-not-know-is-not-quarantined`; `conformance/capsule-vectors.json` case CC-042; thirteen mutations in `scripts/mutations.json`; ADR-023 | `TESTED` | What put the principal under quarantine is not modelled: the environment holds the state, as it holds a revocation, and the policy-defined trigger that created it is outside the decision path. The isolation is not applied by anything in this repository, because no component here holds state. The subject state that applies and lifts the isolation does not exist yet |
| INV-13 | Explainability is evidence-based | `schemas/decision-record.schema.json`, `schemas/audit-event.schema.json` | `PROPOSED` | Schemas only |
| INV-14 | Offline core operation | Specification §3.4 | `PROPOSED` | Not implemented |
| INV-15 | User agency | Specification §10.3 | `PROPOSED` | Not implemented |
| INV-16 | Compatibility is honest | Specification §9.2 | `PROPOSED` | Not implemented; no compatibility records exist |
| INV-17 | Hardware diversity is explicit | Specification §1.5, §7.1 | `PROPOSED` | Not implemented; no hardware profile declared |
| INV-18 | Failure containment | Specification §11.3 | `PROPOSED` | Not implemented |
| INV-19 | No secret master backdoor | `GOVERNANCE.md`, `RELEASE_PROCESS.md` | `PROPOSED` | Process only; cannot be proven by construction |
| INV-20 | Publicly accountable changes | `decisions/`, `evidence/traceability.csv` | `TESTED` | Process works for contract changes; no protected branch enforcement yet |
| INV-21 | Cryptographic conservatism | ADR-002 | `PROPOSED` | No crypto implementation exists; ADR still open on suite selection |
| INV-22 | Secure degradation | `state-machines/system-mode-transitions.json` | `TESTED` | Structural validation only; no runtime enforcement |
| INV-23 | Reversibility where possible | `state-machines/change-capsule-transitions.json` | `TESTED` | Structural validation only |
| INV-24 | No false assurance | This ledger; `README.md` | `TESTED` | Self-referential — requires an independent reviewer to be meaningful |
| INV-25 | No arbitrary vendor lock-in | `LICENSE`, `NOTICE` | `TESTED` | License only; no export interfaces implemented |
| INV-26 | Accessibility is core | Constitution Article 28 | `PROPOSED` | No UI exists |
| INV-27 | No covert persistence | Specification §6.3 | `PROPOSED` | Not implemented |
| INV-28 | Safe defaults survive customization | Specification §3.2 | `PROPOSED` | Not implemented |
| INV-29 | Audit minimization | `schemas/audit-event.schema.json` | `PROPOSED` | Schema only |
| INV-30 | Measured claims only | `evidence/traceability.csv`, `ROADMAP.md` | `TESTED` | Process-level; benchmark methodology not yet established |

### Product and compatibility claims

| Claim ID | Claim | Evidence | Status | Limitations |
|---|---|---|---|---|
| COMP-001 | "Senvoryn runs any software" | — | `WITHDRAWN` | Not a claim this project may make. Replaced by COMP-002 |
| COMP-002 | Compatibility is evidence-backed, per application version and profile | Specification §9.2 | `PROPOSED` | No compatibility records exist yet |
| COMP-003 | "Install with one click" means no user-created boot media on certified host profiles | Specification §7.2 | `PROPOSED` | No installer exists; no host profile certified |
| COMP-004 | "Works on any device" means a portable architecture and expandable profile program | ADR-001 | `PROPOSED` | No architecture selected |
| COMP-005 | "AI-native" means AI is available as a first-class capability, and core functions remain operable without it | Specification §8.4 | `PROPOSED` | No AI layer exists; AI-disabled test suite not yet built |

### Performance claims

| Claim ID | Claim | Target | Evidence | Status | Limitations |
|---|---|---|---|---|---|
| PERF-001 | SCP local cached policy p99 | < 5 ms | — | `PROPOSED` | **No benchmark exists.** Requires a declared reference CPU, 10k+ requests, warmed and cold results reported separately |
| PERF-002 | Small service startup p95 | < 250 ms | — | `PROPOSED` | No benchmark exists |
| PERF-003 | Idle minimal server RAM | < 256 MiB | — | `PROPOSED` | No benchmark exists; exact profile and accounting method must be disclosed |
| PERF-004 | Recovery to prior bootable generation | < 120 s | — | `PROPOSED` | No benchmark exists; excludes user-data restoration and physical repair |
| PERF-005 | AI-disabled core operation | 100% of core boot/login/policy/update/recovery tests | — | `PROPOSED` | Suite not yet built |
| PERF-006 | Privileged request audit coverage | 100% of the defined privileged operation registry | — | `PROPOSED` | Registry is an unsigned sample; exceptions must be reviewed |
| PERF-007 | Direct install success rate | Per-certified-host rate | — | `PROPOSED` | Host OS, firmware, disk mode and recovery conditions must be listed |

> **Performance cannot be reported without the isolation profile used by that run.** A number without a profile is not a measurement.

---

## Current headline position

| Question | Honest answer today |
|---|---|
| Is this a working operating system? | **No.** It is a specification and a model. |
| Is the reference model a security boundary? | **No.** It uses trusted booleans and integer time values. Never deploy it as an SCP. |
| Is the project secure? | **The question is not yet meaningful.** Nothing runs. What exists is the *specification* of what secure would mean, plus tests of the model. |
| What has been independently reviewed? | **Nothing.** No independent review has occurred. |
| What may be advertised? | The specification, the schemas, the state machines, and the model's test results — with the limitations stated alongside. |
| What conformance label applies? | **Design-conformant** (repository), **Prototype-conformant** (reference model, limited test environment only). |

---

## Maintenance rules

1. **Any change that alters what may be claimed must update this file in the same change.**
2. A claim moving to `INDEPENDENTLY REVIEWED` MUST name the reviewer and the date. "Internal review" does not qualify.
3. A claim moving to `FORMALLY VERIFIED WITH SCOPE` MUST state the verified scope and the assumptions — a proof of one property is not a proof of the system.
4. **Withdrawn claims stay in the table**, marked `WITHDRAWN`, with the reason. Deleting a withdrawn claim hides a lesson.
5. The ledger is reviewed at every release gate and at least once per quarter.

---

## See also

- [`evidence/traceability.csv`](traceability.csv) — requirement → test → evidence → owner → status
- [`evidence/security-claim-ledger-template.md`](security-claim-ledger-template.md) — the per-claim template
- [`evidence/local-validation-report.txt`](local-validation-report.txt) — what the local tests actually proved
- [`RELEASE_PROCESS.md`](../RELEASE_PROCESS.md) — the release gate that consumes this ledger
- [`docs/threat-model.md`](../docs/threat-model.md) — the draft threat model behind several claims

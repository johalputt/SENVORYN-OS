# Threat Model — SCP Decision Path

**Status:** 🔶 **First draft — NOT independently reviewed. Not a safety case.**
**Scope:** The System Control Plane (SCP) request-decision path as defined by the Core Contract Specification v0.1.0 §3, and the capability semantics in §4.
**Version:** 0.1.0 (draft)
**Date:** 2026-10-15
**Owner:** _unassigned — needs a steward and an independent reviewer_

> **Read this first.** This document was written by the project, not by an independent reviewer. It exists to close a Gate 0 blocker and to give reviewers something concrete to attack. Until it has been reviewed by someone outside the project, it is a **design input, not evidence**. It does not support any security claim.

---

## 1. Purpose and limits

This threat model identifies adversaries, assets, trust boundaries, attack surfaces, and mitigations **for the decision path only**. It does **not** cover:

- the Separation Core (L1) — hardware isolation, memory safety, scheduler, IPC primitives;
- cryptographic implementation internals (assumed correct under reviewed primitives, see [ADR-002](../decisions/ADR-002-crypto-boundary.md));
- the installer and update path (separate models required — see §9);
- the user interface (confirmation-dialog honesty is a UX/UEF concern);
- the federation layer (L8).

Those are separate models. A threat model that pretends to cover everything covers nothing.

---

## 2. System model

### 2.1 Under protection

| Asset | Why it matters |
|---|---|
| **Authority** (the set of live capabilities) | Possession of authority is equivalent to the ability to act as someone else |
| **Policy epoch and revocation state** | Stale or replayed policy lets revoked authority live on |
| **Decision integrity** | A decision that differs from the specified one is a silent security failure |
| **Audit history** | Without it, abuse is undetectable and post-incident analysis is impossible |
| **User data labels** | Misclassification enables export of `SENSITIVE`/`SECRET` data |
| **Consent bindings** | An approval must authorize exactly one action digest |
| **Capability provenance** | A forged capability is a forged grant |

### 2.2 Assumptions

| # | Assumption | If false |
|---|---|---|
| **A-1** | The separation core correctly enforces memory and object isolation | The entire model is void; this is an L1 responsibility |
| **A-2** | Cryptographic primitives are correct and keys are uncompromised | Signature checks are meaningless; see ADR-002 open items |
| **A-3** | The evaluator process is not attacker-controlled | Every gate can be skipped |
| **A-4** | Audit storage is durable, append-only, and not writable by unprivileged principals | Evidence can be forged or erased |
| **A-5** | Monotonic time is available and non-reversible | Expiry and lease semantics fail |
| **A-6** | The host hardware provides the isolation properties claimed in its profile | A profile's assurance level is overstated |

### 2.3 Explicitly out of scope

- Social engineering of the user (mitigated by clear UI, not by the SCP).
- Attacks on hardware/firmware implants.
- Denial of service against the decision path (availability is tracked separately; the constitution requires fail-safe, not fail-open).
- Malicious *kernel* code — that is an L1 compromise, not a policy bug.

---

## 3. Adversaries

| ID | Adversary | Capability | Goal |
|---|---|---|---|
| **ADV-1** | **Unprivileged local principal** | Can submit arbitrary requests to the request API | Obtain authority it was never granted |
| **ADV-2** | **Malicious application / Origin Capsule** | Runs inside a capsule; can attempt bridge use and request forgery | Escape the capsule; reach host resources |
| **ADV-3** | **Compromised AI agent** | Can call the tool broker; controls its own prompt and memory | Exceed its autonomy class; cause a high-risk effect |
| **ADV-4** | **Malicious or compromised remote controller** | Sends signed requests to a node | Make the node perform actions the local owner did not authorize |
| **ADV-5** | **Network attacker** | Can observe, replay, reorder, and inject traffic | Replay an authorized request; forge a principal; downgrade policy |
| **ADV-6** | **Malicious insider with repository or CI access** | Can modify source, workflows, or build scripts | Ship a backdoored release |
| **ADV-7** | **Stale or compromised host** (stolen laptop, borrowed device) | Physical access to a powered-off or running device | Recover data or authority without the owner's key |
| **ADV-8** | **Malicious compatibility guest** | Runs under UEF in a guest environment | Escalate guest privileges into host privileges |

---

## 4. Trust boundaries

```
   UNTRUSTED                          │  TRUST BOUNDARY
 ─────────────────────────────────────┼──────────────────────────────────
 Request API surface                  │  1. Transport / size gate
 App, capsule, AI agent, remote node  │  2. Canonical parse gate
 Network                              │  3. Recipient gate
 ─────────────────────────────────────┼──────────────────────────────────
                                     ▼
                          ┌──────────────────────┐
                          │  SCP policy evaluator │  ← side-effect free
                          │  (pure, deterministic) │
                          └──────────────────────┘
                                     │  decision + scoped capability
                                     ▼
                          ┌──────────────────────┐
                          │  Executor             │  ← re-checks authority
                          └──────────────────────┘
                                     │  effect
                                     ▼
                          ┌──────────────────────┐
                          │  Audit sink           │  ← append-only
                          └──────────────────────┘
```

**Boundary crossings and their risks:**

| # | Crossing | Risk | Required control |
|---|---|---|---|
| **TB-1** | Untrusted input → canonical parse | Ambiguity exploited to change meaning | Canonical encoding profile; reject duplicate keys, invalid UTF-8, unknown critical fields |
| **TB-2** | Network → recipient gate | Request executed by the wrong node | Recipient binding; node identity |
| **TB-3** | Principal identity → session | Identity confusion | Authenticated binding to channel/session |
| **TB-4** | Capability → evaluator | Forged or over-broad capability | Provenance verification by an approved local mechanism, independent of the requester |
| **TB-5** | Evaluator → executor | Stale decision used after state changed | Executor re-checks policy epoch and preconditions immediately before the effect |
| **TB-6** | Approval → evaluator | Approval reused for a different action | Action digest binding; freshness |
| **TB-7** | Effect → audit | Evidence lost or tampered | Append-only sink; audit-availability gate denies high-risk actions when the sink is unavailable |
| **TB-8** | AI proposal → tool broker | Model output treated as authority | Broker re-resolves target, re-computes risk, re-authorizes through SCP |
| **TB-9** | Guest → host bridge | Guest privilege becomes host privilege | Typed, explicit, leased, revocable bridges |

---

## 5. Attack surface and abuse cases

Each abuse case names the gate that must hold, the spec rule that requires it, and the test ID that proves it.

### AS-1 · Authority fabrication

**Adversary:** ADV-1, ADV-2
**Attack:** Construct a capability object that appears to grant authority the caller does not hold, and submit a request citing it.
**Gate:** Capability gate (#10).
**Required controls:** A capability MUST NOT be accepted solely because its JSON fields look plausible; provenance MUST be authenticated by a approved local mechanism independent of the requester. Issuer identity, revocation status, scope dimensions, and time bounds are all checked.
**Failure mode if control fails:** Complete authority bypass — arbitrary effect as any principal.
**Tests:** `SCP-002`, `SCP-003`, `SCP-004`, `CAP-001`.

### AS-2 · Risk downgrade by self-assertion

**Adversary:** ADV-1, ADV-3
**Attack:** Claim a destructive operation as low risk so it bypasses strong approval.
**Gate:** Risk/consent gate (#11) against the trusted risk registry.
**Required controls:** The effective risk tier is derived from a signed, versioned Operation Registry. A lower caller claim → `RISK_CLAIM_MISMATCH`. Unknown operation → `UNKNOWN_OPERATION_RISK`. Most restrictive tier wins. **There is no default low-risk classification.**
**Failure mode:** Destructive action without consent.
**Tests:** `SCP-006`, `AI-002`.

### AS-3 · Approval substitution

**Adversary:** ADV-1, ADV-3, ADV-4
**Attack:** Reuse a user's approval for a different action, or mutate the action after approval.
**Gate:** Risk/consent gate (#11).
**Required controls:** Approvals are bound to the exact action digest and must be fresh. Any change to destination, purpose, or data scope invalidates the prior approval. The model cannot alter a confirmation without invalidating the digest.
**Failure mode:** User consents to action X; system performs action Y.
**Tests:** `SCP-006`, `AI-002`.

### AS-4 · Replay of a captured request

**Adversary:** ADV-5
**Attack:** Capture an authorized request and resubmit it.
**Gate:** Freshness gate (#5).
**Required controls:** Nonce or monotonic sequence, recipient binding, expiry, replay cache or epoch semantics, boot-epoch strategy. **A timestamp alone is not anti-replay protection.**
**Failure mode:** Repeated destructive effect; expired grant revived by clock manipulation.
**Tests:** `SCP-005`, `CC-002`.

### AS-5 · Stale policy / epoch confusion

**Adversary:** ADV-4, ADV-5
**Attack:** Present a request valid under an old, more permissive policy epoch.
**Gate:** Policy epoch gate (#9) and capability policy-epoch check (#10).
**Required controls:** Stale high-risk requests are denied. Unknown or stale epoch → `DENY` or `CHALLENGE`, **never a silent permit**. Capabilities bound to an old epoch are rejected.
**Failure mode:** Authority that was legitimately revoked continues to work.
**Tests:** `SCP-005`, `CAP-002`.

### AS-6 · Clock manipulation

**Adversary:** ADV-5, ADV-7
**Attack:** Roll the clock back to revive an expired capability or extend a lease.
**Gate:** Freshness gate (#5).
**Required controls:** Use a monotonic clock and an authenticated time source. Documented boot-epoch strategy so a monotonic reset cannot revive old grants. **The system must not extend a capability merely because the time service is offline.** If clock trust is uncertain, a high-risk action requiring precise expiry is denied or requires a separately defined offline recovery path.
**Failure mode:** Expired authority revived indefinitely.
**Tests:** `SCP-003`, `SCP-005`.

### AS-7 · Mode bypass

**Adversary:** ADV-1, ADV-4
**Attack:** Perform a write while the system is in `READ_ONLY`, `RECOVERY`, or `QUARANTINED`.
**Gate:** Mode gate (#8).
**Required controls:** Modes are enforced on every privileged request, not as UI labels. `QUARANTINE` is not a generic penalty; it isolates a defined suspicious principal. A mode transition never erases audit history or resets a revocation epoch.
**Failure mode:** Writes during a freeze; quarantine escape.
**Tests:** `SCP-004` (mode path), `OC-003`.

### AS-8 · Remote command escalation

**Adversary:** ADV-4
**Attack:** A compromised fleet controller instructs a node to perform a privileged action.
**Gate:** All gates — local re-authorization.
**Required controls:** Each node performs local authorization. A controller compromise must not automatically grant access to local secrets or unrestricted shell execution. Offline nodes enforce bounded cached policy; **the maximum stale-authority window must be published**. Remote unavailability must not cause a remote principal to gain additional power.
**Failure mode:** Fleet-wide compromise from one controller.
**Tests:** `FED-001`, `SCP-005`.

### AS-9 · AI autonomy escalation

**Adversary:** ADV-3
**Attack:** The model convinces a component to raise its own autonomy class, or obtains a capability directly.
**Gate:** Capability gate (#10) + architecture boundary.
**Required controls:** AI runs outside the enforcement core; every effect is re-authorized by local SCP policy. Models cannot issue capabilities. **Autonomy class never bypasses SCP.** Model confidence is never authorization proof. The user can reduce autonomy immediately; increasing it requires a visible change summary and new approval.
**Failure mode:** Model becomes an unreviewed root of trust.
**Tests:** `AI-001`, `AI-004`.

### AS-10 · Prompt injection to authority

**Adversary:** ADV-3, ADV-5 (indirect injection through retrieved content)
**Attack:** Web page, email, document, log, or tool result instructs the agent to take an unauthorized action.
**Gate:** Tool broker + all SCP gates.
**Required controls:** **All retrieved content is untrusted for authority.** High-risk tool execution requires a new deterministic decision and may require user confirmation. Corpus MUST include tool-call laundering, indirect prompt injection, memory poisoning, cross-user leakage, model update rollback, and resource-exhaustion prompts.
**Failure mode:** Data exfiltration or unauthorized effect via a trusted-looking instruction.
**Tests:** `AI-004`, `AI-003`.

### AS-11 · Remote inference exfiltration

**Adversary:** ADV-3
**Attack:** Send `SENSITIVE` or `SECRET` data to a remote inference service without an authorized data-flow grant.
**Gate:** Data-flow authorization + risk/consent gate.
**Required controls:** Before sending, the system must establish data classification, destination, purpose, retention terms, and consent state. **If these cannot be established, protected data is not sent.** Local-only mode must be real and testable, not a UI label that silently falls back to a remote service.
**Failure mode:** Confidential data leaves the device.
**Tests:** `AI-003`, `PRIV-001`.

### AS-12 · Audit evasion

**Adversary:** ADV-1, ADV-2, ADV-6
**Attack:** Perform a privileged action and suppress, alter, or bury the audit record.
**Gate:** Audit readiness gate (#14).
**Required controls:** For high-risk changes, **unavailable required audit storage causes denial**. Cancellation never deletes the audit record. Rollback never erases audit history. Audit records must avoid storing secrets or full private content.
**Failure mode:** Undetectable privileged activity; loss of post-incident evidence.
**Tests:** `SCP-008`, `CC-004`, `PRIV-002`.

### AS-13 · Denial oracle / policy leakage

**Adversary:** ADV-1
**Attack:** Probe the SCP to enumerate resources, principals, or policy contents by observing denial differences.
**Gate:** All gates.
**Required controls:** Denial messages must not reveal secret resource names or policy details to an unauthorized requester. Error records must not include secrets or raw untrusted content.
**Failure mode:** Reconnaissance of protected resources.
**Tests:** `SCP-001`, `SCP-004`.

### AS-14 · Guest-to-host privilege escalation

**Adversary:** ADV-8
**Attack:** A guest administrator maps host memory, inspects host credentials, grants host device access, alters host update policy, or reaches SCP privileged endpoints.
**Gate:** Capsule boundary + bridge policy.
**Required controls:** Guest privileged calls terminate in a guest kernel or runtime boundary. Host/guest bridges are explicit, typed, and revocable. **A bridge MUST NOT be enabled simply because a guest OS exposes that feature.** Default capsule has no host filesystem access and no network route.
**Failure mode:** Guest escape into the host.
**Tests:** `OC-001`, `OC-002`, `OC-003`.

### AS-15 · Scope-widening delegation

**Adversary:** ADV-1, ADV-2
**Attack:** Obtain a delegated capability broader than the parent.
**Gate:** Capability gate (#10).
**Required controls:** A child may only be equal to or narrower. Subject explicit and non-spoofable; resource scope a subset; operations a subset; expiry ≤ parent; budget ≤ parent; delegation depth decreased; purpose not less restrictive; policy epoch still acceptable. **String prefix matching is not a valid general-purpose subset rule** for filesystem or network scope.
**Failure mode:** Slow privilege escalation via delegation chains.
**Tests:** `CAP-001`.

### AS-16 · Compromised release pipeline

**Adversary:** ADV-6
**Attack:** Inject a backdoor through source, dependencies, CI configuration, or the build environment.
**Gate:** Out of the decision path — but it destroys every guarantee above.
**Required controls:** Separation of duties; role-separated signing keys; **the updater must not use a compromised application process to authorize its own update**; SBOM and license inventory; reproducible builds by two independently administered environments; provenance attestations; no dependency admitted merely because a build script fetched it.
**Failure mode:** Every other mitigation is bypassed at the root.
**Tests:** `UPD-001`, `UPD-002`, `EVID-001`.

### AS-17 · Resource exhaustion as an authority bypass

**Adversary:** ADV-1, ADV-2, ADV-3
**Attack:** Exhaust CPU, memory, storage, disk, or audit space so that enforcement degrades or the system is forced into a permissive degraded state.
**Gate:** Budget gate (#12) + system modes.
**Required controls:** Security-critical functions default fail-safe. **There is no generic "ignore security to keep running" switch.** Availability-critical services may fail-operational only under a profile-specific, preapproved policy with bounded privileges and documented consequences. `DEGRADED` disables or bounds noncritical features rather than widening authority.
**Failure mode:** Availability pressure used to obtain extra authority.
**Tests:** `SCP-008`, `SCP-007`.

### AS-18 · Verification bypass on commit

**Adversary:** ADV-1, fault
**Attack:** Drive a Change Capsule to `COMMITTED` without verified postconditions — e.g. a verifier that returns success on missing results.
**Gate:** Precondition gate (#13) + Change Capsule state machine.
**Required controls:** `COMMITTED` requires all mandatory postconditions to pass and durable evidence to be recorded. **A missing verifier result is failure, not success.** If the outcome cannot be proven, use `PARTIAL` or `NEEDS_HUMAN_RECOVERY`, never `COMMITTED`.
**Failure mode:** Failed or partial changes reported as successful; no recovery triggered.
**Tests:** `CC-001`, `CC-003`.

### AS-19 · TOCTOU between decision and effect

**Adversary:** ADV-1, ADV-2
**Attack:** Change the resource, version, or policy between the decision and the effect.
**Gate:** Effect separation (#16) — executor re-checks.
**Required controls:** The evaluator MUST be side-effect free. The executor independently rechecks authorization and transaction preconditions immediately before acting. Object resolution must be resistant to aliasing, link traversal, namespace confusion, and TOCTOU races. Before each capsule step, target preconditions and policy epoch are rechecked. The artefact that authorises an effect names the canonical identifier of the object and the version the decision was made against, so that the recheck has something to compare against that the caller cannot change. **The executor compares the version the decision was made against with the version it is about to act on, and refuses a decision that names no version at all** (ADR-020).
**Failure mode:** Authorized-for-A effect applied to B.
**Tests:** `SCP-007`, `CC-003`, `CC-VERSION`.

### AS-20 · Aliasing and path confusion

**Adversary:** ADV-1, ADV-2
**Attack:** Reference a protected object via an alias, symlink, hard link, alternate namespace, or ambiguous path string so the resolved object differs from the authorized one.
**Gate:** Object resolution gate (#6).
**Required controls:** Every resource has a canonical typed identity. **Path strings alone are insufficient identifiers for security decisions.** Resolution occurs before authorization and must be resistant to aliasing, link traversal, namespace confusion, and TOCTOU. Unknown resource → `DENY` without dereferencing an untrusted path. A grant is compared with the canonical identity the name resolved to, and the registry's rules are keyed by the canonical type, so that neither the object nor the rule that applies to it is chosen by the caller.
**Failure mode:** Access to an object the principal was not authorized for.
**Tests:** `SCP-004`.

### AS-21 · Silently dropped request fields

**Adversary:** ADV-1, ADV-4
**Attack:** Send a request carrying a field this node does not understand — a field a newer sender expects the node to enforce — and rely on the node ignoring it. A field that carries a demand ("this request is a recovery request", "this request is already approved", "this effect is external") is then dropped, and the node acts on a request that does not say what the sender thought it said.
**Gate:** Canonical parse gate (#2).
**Required controls:** The policy request form is closed: a document carrying a field the form does not define is refused, and the refusal names the field. A reader may not ignore a field it cannot account for, because "ignore what you do not understand" is a decision to enforce a contract nobody wrote down. The reader enforces the form and nothing else — an unknown field is refused before any gate runs, and a refusal at the reader is not a decision.
**Failure mode if control fails:** A request is judged on a contract the sender did not write, and the difference between the two is invisible to both sides.
**Tests:** `SCP-001`, `SCP-002`.

---

## 6. Risk ranking

Ranked by **(impact × likelihood)** given the current implementation state. "Likelihood" reflects the state of the *specified* system, not of any product.

| Rank | Abuse case | Impact | Likelihood | Residual risk after specified controls |
|---|---|---|---|---|
| 1 | AS-1 Authority fabrication | Critical | Medium | Depends on unimplemented provenance verification — **open risk** |
| 2 | AS-2 Risk downgrade | Critical | Medium | Registry not yet reviewed/signed — **open risk** |
| 3 | AS-14 Guest→host escalation | Critical | Medium | UEF not implemented — **open risk** |
| 4 | AS-9 / AS-10 AI escalation & injection | Critical | Medium | Broker not implemented — **open risk** |
| 5 | AS-16 Compromised pipeline | Critical | Low–Medium | Signing ceremony undefined — **open risk** |
| 6 | AS-13 Audit evasion | High | Medium | Depends on audit sink durability — **open risk** |
| 7 | AS-19 / AS-20 TOCTOU & aliasing | High | Medium | Resolution boundary specified and implemented: the canonical identity governs the later gates, and the executor refuses a decision that names no version of the target or names another one (ADR-020). The per-resource-type resolution design and the alias-revocation rule are still open — **open risk** |
| 8 | AS-3 Approval substitution | High | Medium | Digest binding specified; implementation pending |
| 9 | AS-8 Remote command escalation | High | Medium | Local re-authorization specified; federation unimplemented |
| 10 | AS-7 Mode bypass | High | Low | Mode table specified; enforcement unimplemented |
| 11 | AS-5 Epoch confusion | High | Low | Specified and testable |
| 12 | AS-6 Clock manipulation | High | Medium | Boot-epoch strategy not yet specified — **open risk** |
| 13 | AS-11 Inference exfiltration | High | Medium | Data-flow contract specified; enforcement unimplemented |
| 14 | AS-4 Replay | High | Low | Specified; needs nonce/sequence implementation |
| 15 | AS-15 Delegation widening | High | Medium | Attenuation rules specified; subset semantics per resource type undefined — **open risk** |
| 16 | AS-18 Verification bypass | High | Low | State machine + invariant specified |
| 17 | AS-4 / AS-17 Exploit chains | Medium | Low | Partial mitigation specified |
| 18 | AS-21 Silently dropped request fields | High | Low | The policy request form is closed and the reader's nine controls are mutation-tested, but the transport that would read the wire envelope does not exist, so the wire envelope's own closure is unproven in code — **open risk** |

> **Every "open risk" above is a Gate 1–8 deliverable, not a bug in the design.** The design states the requirement; the implementation does not exist yet. The purpose of this table is to make sure nothing is *forgotten*, not to claim anything is solved.

---

## 7. Mitigation verification

Each control above MUST be backed by a conformance test. The mapping lives in [`evidence/traceability.csv`](../evidence/traceability.csv); the test IDs are stable in the Core Contract Specification §16.

**A control with no test ID is not a control — it is a hope.**

---

## 8. Abusive-input considerations

Parsers MUST be memory-safe where possible and fuzzed with: malformed, truncated, oversized, duplicate-key, deeply nested, conflicting, and invalid-UTF-8 inputs. **Parse failure must not mutate state.**

| Input class | Required rejection |
|---|---|
| Duplicate JSON keys | Reject — ambiguous representation |
| Invalid UTF-8 | Reject |
| Unknown critical fields | Reject |
| Ambiguous number formats | Reject |
| Oversized message / nesting depth / string length | Reject before allocation |
| Excessive attached capabilities | Reject |
| Alternative encodings of the same security value | Reject (canonical encoding) |

---

## 9. Models this one does not replace

| Component | Model required |
|---|---|
| Separation Core (L1) | Memory-safety and isolation model; hardware assumption register |
| Update path | Signature, downgrade, anti-rollback, key-rotation compromise model |
| Installer | Destructive-installer threat model (target confusion, power loss, wrong disk) |
| Secrets broker | Secret-handle scoping and leakage model |
| Federation (L8) | Partition, split-brain, compromised-controller model |
| AI memory | Retention, cross-user leakage, deletion semantics model |
| Supply chain | Dependency admission, provenance, and build-environment model |

---

## 10. Open questions requiring an ADR

1. **How is capability provenance verified?** The spec requires "an approved local mechanism" but does not name one. (Blocks AS-1.)
2. **What is the canonicalization profile?** Until named and tested, signature semantics are undefined. (Blocks AS-1, AS-4.)
3. **What is the boot-epoch strategy?** (Blocks AS-6.)
4. **What are the subset semantics for each resource type?** (Blocks AS-15, AS-20.)
5. **What is the maximum stale-authority window for disconnected nodes?** (Blocks AS-8.)
6. **What is the recovery trust-root procedure?** (Blocks AS-16.)
7. **Is the risk registry in-band or out-of-band?** A revoked operation must not be usable while a stale registry is cached. (Blocks AS-2.)
8. **What is the audit sink's durability and tamper-evidence model?** (Blocks AS-12.)

---

## 11. Review checklist

An independent reviewer should confirm each of the following before this model is considered reviewed:

- [ ] Every adversary in §3 has at least one abuse case
- [ ] Every abuse case names a gate and a test ID
- [ ] No mitigation is credited without an implemented mechanism
- [ ] The assumptions in §2.2 are stated as dependencies, not facts
- [ ] Open risks are not described as solved
- [ ] A separate model exists for each component in §9
- [ ] The reviewer did not author the specification under review

---

## See also

- [Core Contract Specification v0.1.0](../docs/specs/SENVORYN_Core_Contract_Spec_v0.1.md) — the contracts this model attacks
- [Founding Constitution v0.6.0](../docs/constitution/SENVORYN_OS_Constitution_v0.6.md) Article 2 — the invariants at risk
- [`evidence/traceability.csv`](../evidence/traceability.csv) — requirement → test mapping
- [`SECURITY.md`](../SECURITY.md) — how to report a finding

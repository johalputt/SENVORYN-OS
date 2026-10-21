# Glossary

Normative vocabulary for Senvoryn OS. Where this glossary and a specification disagree, the specification controls for its own scope — but a disagreement is a bug: please [file it](https://github.com/johalputt/SENVORYN-OS/issues/new).

Terms marked **[INV]** are constitutional invariants from [Founding Constitution v0.6.0](docs/constitution/SENVORYN_OS_Constitution_v0.6.md) Article 2.

---

## A

**Action Envelope** — The typed, untrusted serialization of an action proposed by AI. Contains `intent_id`, `agent_id`, `model_id`, `tool_id`, canonical target, requested operations, input data labels, predicted effects, risk claim, resource budget, external destinations, expiry, and explanation/evidence references. Untrusted until validated; the model's risk claim is never authoritative.

**Approval Record** — A signed record of a human or workflow authorization bound to an **exact action digest**. Changing the action invalidates the approval.

**Authorization** — The System Control Plane's decision about a Change Capsule, read out of the **decision record** the policy layer wrote: the plan digest it permits, the version of the target the decision was made against, the status it carried, and the approvals that were given. The executor checks the binding; it does not re-decide. An authorization that names no version authorises no effect, because it was made before the resource was resolved; one that names no plan authorises no plan; and one that granted no authority authorises nothing at all. See [ADR-020](decisions/ADR-020-the-decision-names-the-version.md) and [ADR-021](decisions/ADR-021-the-decision-record-is-the-authorization.md).

**Attenuation** — The rule that a delegated (child) capability may only be *equal to or narrower* than its parent: subset resources, subset operations, subset purposes, expiry ≤ parent, budget ≤ parent, delegation depth decreased, policy epoch still acceptable. **[INV-03]**

**Audit Event** — An append-only record that a privileged action was requested, decided, and/or performed, with its reason codes. Audit is never deleted by rollback. **[INV-11]**

**Autonomy Class** — The permission level granted to an AI agent: `A0 Explain`, `A1 Recommend`, `A2 Bounded reversible`, `A3 Pre-authorized workflow`, `A4 Critical`. Autonomy class never bypasses the SCP. The user can reduce autonomy immediately; increasing it requires a visible change summary and new approval.

---

## B

**Bridge** — An explicit, typed, revocable channel between a guest/Origin Capsule and the host, covering filesystem sharing, clipboard, screen capture, camera, microphone, location, GPU, USB, display, accessibility, and network. Each bridge defines direction, operations, data label, lease, user indicator, budget, logging class, and revocation semantics. *"Bridge available" is not "bridge authorized."*

**Boot Manifest** — The signed description of a boot generation: component hashes, versions, hardware profile, dependency order, signing identity, policy epoch, anti-rollback version, and recovery artifact.

---

## C

**Canonical Encoding** — The versioned, deterministic byte representation of a security-significant object, defined before it is signed or hashed. Implementations must not sign an arbitrary serializer's output and assume another serializer will reproduce the same bytes. See [ADR-003](decisions/ADR-003-protocol-encoding.md).

**Capability** — An unforgeable, bounded authority to act on a specified object or operation. Binds `subject`, `resource`, `operations`, `purpose`, `constraints`, `not_before`, `expires_at`, `policy_epoch`, `issuer`, `delegation_depth`, `revocation_id`, and provenance proof.

**Capability Graph** — A queryable model of principals, resources, grants, purposes, dependencies, and data flow. **It is not itself the enforcement mechanism.**

**Change Capsule** — An immutable state-transition plan with preconditions, authorization, effects, validation, and recovery/compensation semantics. Not an executable script with root privilege. Operations are drawn from a versioned Operation Registry; unknown operation names are rejected.

**Claim** — Any public statement about security, privacy, performance, compatibility, or reliability. Each claim must be classified in the [Claims Ledger](evidence/claims-ledger.md) and must not exceed its evidence.

**Claims Ledger** — The public record of each security/privacy/performance/compatibility claim and its evidence status: `PROPOSED`, `IMPLEMENTED`, `TESTED`, `INDEPENDENTLY REVIEWED`, `FORMALLY VERIFIED WITH SCOPE`, or `WITHDRAWN`.

**Compatibility Level** — The evidence tier for an application/profile pair: `C0 NOT_ASSESSED`, `C1 LAUNCH`, `C2 CORE_WORKFLOW`, `C3 BROAD_FUNCTIONAL`, `C4 PROFILE_VERIFIED`. Each record names the exact versions tested. No tier guarantees all future versions or all peripherals.

**Conformance Label** — The assurance claim an implementation may advertise: `Design-conformant`, `Prototype-conformant`, `Profile-conformant`, `Release-assured`. **No system may advertise a higher label than its evidence supports.**

**Constraint** — A limit attached to a capability: allowed path/object set, network endpoints, ports, protocol, bytes, rate, duration, device functions, data labels, process ancestry, user session, capsule identity, operation count, approval ID, permitted time window. **Unrecognized constraints on a security-sensitive capability cause rejection**, not silent ignoring.

---

## D

**Data Class / Data Label** — The sensitivity classification governing allowed storage, logging, export, remote processing, and retention: `PUBLIC`, `INTERNAL`, `PERSONAL`, `SENSITIVE`, `SECRET`, `SECURITY_EVIDENCE`. Classification is conservative when unknown. An app cannot downgrade a label by copying data into another container.

**Decision** — The deterministic result of evaluating a request against a defined policy snapshot and evidence set: `PERMIT`, `DENY`, `CHALLENGE`, `DEFER`, or `QUARANTINE`. Every decision carries `decision_id`, `request_id`, reason codes, policy epoch, evaluated target/version, required next step, capability scope if permitted, expiry, and audit-event reference.

**Decision Record** — The document the policy layer writes for a decision, and the only thing the capsule executor reads to learn what was permitted. It names the canonical object the decision was about, the version of it the decision was made against, the plan the request asked to run, the status, the reason codes, and the audit event. Its field set is closed: a record carrying a field the record does not define is refused rather than read, and a record claiming an effect was performed is refused because the policy layer cannot have performed one. See [ADR-021](decisions/ADR-021-the-decision-record-is-the-authorization.md).

**Denial Code** — A stable, machine-readable reason for a rejection, with a class and a default decision. See [`errors/denial-codes.json`](errors/denial-codes.json).

**Direct Installer** — The application-like installation path on certified host configurations that avoids user-created boot media. Must not silently disable host security, bypass firmware requirements, or repartition unrelated storage.

---

## E

**Effect** — An observable change outside the pure evaluator: writing data, launching code, exposing a device, transmitting data, changing policy, or altering system state. **Authorizing an effect is not performing it.**

**Evaluator** — The pure, side-effect-free function that evaluates a request against policy and returns a decision.

**Executor** — The component that performs an effect using an authorization token or short-lived capability, independently re-checking authorization and transaction preconditions immediately before acting.

---

## F

**Fail-safe** — The default failure posture for security-critical functions: deny an unsafe operation. Fail-operational is permitted only for availability-critical services under a profile-specific, preapproved policy with bounded privileges and documented consequences. **There is no generic "ignore security to keep running" switch.** **[INV-22]**

**Federation** — Cross-device coordination using authenticated identities and signed scoped requests. Fleet control may propose desired state; each node performs local authorization. A controller compromise must not yield local secrets or unrestricted shell access.

---

## G

**Gate** — One of the 16 ordered checks in the SCP validation order. A rejection at any gate produces no privileged effect.

**Governance** — The public rules for decision-making, separation of duties, signing authority, and constitutional amendment. See [`GOVERNANCE.md`](GOVERNANCE.md).

---

## H

**HCR (Hardware Capability Record)** — A record of a device's detected, validated hardware capabilities and the security properties they provide.

---

## I

**Intent Record** — A structured request describing a desired outcome, requester, assumptions, effects, risks, and authorization needs. The original prompt may be retained only under the session's privacy policy; it is **not** the canonical authority record.

**Intent Transaction Compiler** — The component that converts an intent into a candidate bounded plan and validates it before authorization.

**Invariant** — A constitutional rule that may be refined for clarity but **may not be weakened** by configuration, vendor extensions, administrative convenience, or AI-generated changes. See [INV-01 … INV-30](docs/constitution/SENVORYN_OS_Constitution_v0.6.md#article-2--constitutional-invariants).

---

## K

**Key Custody** — The set of rules governing where private keys live, who may use them, how they rotate, how they are revoked, and how they are recovered. See [ADR-002](decisions/ADR-002-crypto-boundary.md) and [ADR-013](ROADMAP.md#open-decisions-register).

---

## L

**LIF (Local Intelligence Fabric)** — The layer providing model inference, intent interpretation, prediction, and proposals. **No direct privileged interface**; every effect requires SCP authorization. **[INV-04]**

**Local-First Authority** — Each node owns local enforcement. A fleet controller can request a state change but cannot bypass local capability checks, mode restrictions, expiry, or risk-based approval. **[INV-07]**

---

## M

**Memory (User-Owned)** — AI memory whose contents, access, retention, export, and deletion are controlled by the user.

**Mode (System)** — The global operational state constraining what may occur: `NORMAL`, `DEGRADED`, `READ_ONLY`, `RECOVERY`, `QUARANTINED`, `MAINTENANCE`, `OFFLINE`. Enforced on every privileged request — not a UI label.

---

## N

**Nonce / Replay Key** — The uniqueness token that, combined with expiry and epoch semantics, prevents a request or capsule from being replayed. A timestamp alone is not anti-replay protection.

---

## O

**Operation Registry** — The signed, versioned, security-critical policy artifact mapping canonical resource type, operation, object attributes, data classification, scope, and external effects to a risk tier. The request's risk claim is never authoritative. The registry requires provenance, review, staged rollout, and rollback.

**Origin Capsule (OC)** — An isolated execution domain for an application, service, driver, model, or foreign runtime, with its own immutable ID, provenance, storage namespace, network policy, device grants, IPC peers, resource budget, update policy, audit policy, and compatibility status.

---

## P

**Policy Epoch** — The monotonically increasing version of the policy under which a decision is made. Stale high-risk requests are denied. A capability does not override a policy-revoked state.

**Policy Request Form** — The document the transport layer hands to the policy layer: the request's intent with its times expressed in monotonic milliseconds and the fields the policy layer does not read removed. It is not the wire envelope, and the reader's contract is the form's. The form is **closed**: a document carrying a field the form does not define is refused, because a field this node cannot account for may be a field a newer sender expects this node to enforce. See [ADR-019](decisions/ADR-019-the-policy-request-form.md).

**Principal** — An authenticated actor that can request an operation. Types: `user`, `service`, `application`, `driver_service`, `ai_agent`, `device`, `recovery_actor`, `remote_controller`, `build_identity`. A label like "administrator" is **not** an identity — it is a role assignment that must itself be authenticated, scoped, and auditable.

---

## R

**Reason Code** — The stable identifier explaining a decision. See [`errors/denial-codes.json`](errors/denial-codes.json).

**Resolution** — The turning of a name a caller used into the canonical identity of an object: its canonical identifier, its canonical type, and its version. The name may be an alias; the canonical identity is the truth, and it is the environment's. A name that resolves to nothing is denied rather than dereferenced, and a name that resolves to an object whose type is not the type the request claimed is denied rather than corrected. The canonical identity is what a grant is compared with and what the Operation Registry's rules are keyed by. The version is part of the resolution: a decision names the version it was made against, so that an executor can refuse a decision made against a version the target has left.

**Recovery Actor** — A principal type authorized to perform verified recovery operations when the system is in `RECOVERY` mode.

**Resource** — Any object whose access or modification can affect confidentiality, integrity, availability, safety, or cost. Every resource has a canonical typed identity, owner, sensitivity label, lifecycle state, supported operation set, version/epoch, and policy domain. **Path strings alone are insufficient identifiers for security decisions.**

**Reversibility Ledger** — The record that identifies what changed and what kind of undo or compensation, if any, is possible.

**Revocation Epoch** — The monotonic counter used to invalidate issued capabilities. Local revocation takes effect before new privileged actions; in-flight operations stop at the next safe boundary.

**Risk Tier** — The policy-derived classification of an operation's consequence, assigned by the Operation Registry. `R0` read-only through `R3` destructive/irreversible. The caller may not self-assign a lower tier.

---

## S

**SCP (System Control Plane)** — The layer performing policy evaluation, state transitions, system configuration, and action authorization. **No authority to bypass kernel-enforced rules.**

**SSC (Senvoryn Separation Core)** — The minimal kernel-like layer providing address-space isolation, scheduling, IPC, interrupt routing, and capability primitives. The smallest software authority base; contains no policy decisions derived from AI output.

**Shadow State** — A bounded snapshot/model used to preview likely effects. **Not a guarantee of future behavior.**

**System Mode** — See *Mode (System)*.

---

## T

**TCB (Trusted Computing Base)** — The set of components whose compromise could violate a named security claim. Every release publishes a TCB inventory. The project must minimize the TCB and explain every expansion.

**Threat Model** — The documented set of adversaries, assets, attack surfaces, and mitigations for a component. Required for high-risk changes and for release-assurance claims.

**Trust Root** — The anchor from which verification chains derive.

---

## U

**UEF (Universal Execution Fabric)** — The layered compatibility fabric: native formats, managed runtimes, API/instruction translation, and isolated guest environments. **Guest permissions never become host permissions.** The UEF never silently substitutes a weaker security profile without notice.

**Untrusted Content** — Any retrieved or externally supplied text — web pages, email, documents, logs, tool results. **Untrusted content is untrusted for authority** and cannot override system policy. **[INV-04]**

---

## V

**Validation Order** — The fixed sequence of 16 SCP gates. Implementations must validate in that order, or in an order *proven equivalent* that cannot leak privileged effects before all gates pass.

---

## W

**Witness / Evidence Record** — The per-test record containing requirement ID, test ID, source revision, harness version, profile, environment, input/fixture hash, expected result, actual result, status, log/artifact references, reviewer, date, and known limitations. **A passing unit test is not evidence of the absence of all vulnerabilities.**

**Wire Envelope** — The request as it travels: the document `schemas/request-envelope.schema.json` describes, carrying date-times, a replay key, a target node and the caller's classification and effect claims. It is the transport layer's input, and the transport layer's output is the policy request form. The two are different documents, and the transport that converts one into the other is not built yet. See [ADR-019](decisions/ADR-019-the-policy-request-form.md).

---

## See also

- [Core Contract Specification v0.1.0](docs/specs/SENVORYN_Core_Contract_Spec_v0.1.md)
- [Founding Constitution v0.6.0](docs/constitution/SENVORYN_OS_Constitution_v0.6.md)
- [`decisions/`](decisions/) — Architecture Decision Records
- [`errors/denial-codes.json`](errors/denial-codes.json) — the machine-readable denial registry
- [`state-machines/`](state-machines/) — machine-readable lifecycle transitions

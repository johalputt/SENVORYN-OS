# SENVORYN OS
## Core Contract Specification
### Version 0.1.0 — Implementation Contract Layer

**Document ID:** SENVORYN-CORE-SPEC-0001  
**Constitution baseline:** Senvoryn OS Constitution v0.6.0  
**Status:** Proposed engineering specification; not ratified and not production evidence  
**Date:** 15 October 2026  
**License intent:** Apache-2.0 for original code and documentation where appropriate; component-level licenses remain separate.

> **Purpose:** Convert constitutional principles into unambiguous data contracts, state machines, validation order, failure behavior, and testable obligations. This document is intentionally conservative: undefined, unverified, or contradictory authority must not result in a privileged action.

---

## 0. Status, normative language, and document hierarchy

The words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative. **TARGET** denotes a provisional engineering objective requiring benchmark evidence. **EXPERIMENTAL** denotes a feature disabled by default and excluded from production assurance claims. A behavior is not implemented merely because this specification requires it.

The normative hierarchy is:

1. Constitutional invariants from the constitution.
2. This Core Contract Specification (CCS).
3. Versioned component specifications.
4. Architecture Decision Records (ADRs).
5. Implementation details.

Lower-level artifacts may narrow a capability or add checks; they MUST NOT broaden authority or weaken a higher-level rule without a formally approved constitutional amendment. If two rules conflict, reject the privileged operation, record a conflict code, and require a specification decision.

### 0.1 Conformance labels

- **Design-conformant:** schemas and algorithms meet this document, but no running implementation is claimed.
- **Prototype-conformant:** the reference implementation passes its declared tests in a limited test environment.
- **Profile-conformant:** an implementation passes the complete test suite for one named hardware/software profile.
- **Release-assured:** the profile also has a reviewed threat model, independent security review, artifact provenance, recovery drills, and published evidence.

No system may advertise a higher conformance label than its evidence supports.

### 0.2 Deliverable map

- `schemas/`: machine-readable JSON Schemas for principals, resources, requests, capabilities, decisions, approvals, intents, operation registry, Change Capsules, Origin Capsules, audit events, artifacts, updates, and system states.
- `state-machines/`: machine-readable Change Capsule and system-mode transition tables.
- `errors/`: stable initial denial-code registry.
- `examples/`: valid request fixture and illustrative trusted operation registry (unsigned sample only).
- `reference/`: a small deterministic SCP decision model, not a kernel or cryptographic implementation.
- `tests/`: executable negative and positive tests for selected invariants.
- `decisions/`: initial architecture decisions and unresolved decisions.
- `evidence/`: requirement-to-test traceability and evidence ledger template.

---

# 1. Core model and trust boundaries

## 1.1 Principals

A **principal** is an authenticated actor that can request an operation. Principal types are `user`, `service`, `application`, `driver_service`, `ai_agent`, `device`, `recovery_actor`, `remote_controller`, and `build_identity`. A label such as “administrator” is not an identity; it is a role assignment that must itself be authenticated, scoped, and auditable.

Every principal MUST have:
- an immutable `principal_id` within its authority domain;
- a `principal_type`;
- a lifecycle state (`active`, `suspended`, `revoked`, `expired`, `unknown`);
- authentication evidence and assurance level;
- an ownership/tenant domain where applicable;
- a set of direct capabilities or references to them;
- an audit classification;
- credential rotation and revocation metadata.

`unknown` MUST be treated as unauthenticated. A display name, process path, package name, model-provided identity, or remote assertion without validation MUST NOT substitute for a principal ID.

## 1.2 Resources

A **resource** is any object whose access or modification can affect confidentiality, integrity, availability, safety, or cost. Resources include files, data objects, devices, IPC endpoints, services, network destinations, capabilities, system settings, signing keys, update channels, model memory, and remote nodes.

Every resource MUST have a canonical typed identity, owner, sensitivity label, lifecycle state, supported operation set, version/epoch where applicable, and policy domain. Path strings alone are insufficient identifiers for security decisions. Resolution MUST occur before authorization and be resistant to aliasing, link traversal, namespace confusion, and time-of-check/time-of-use races.

## 1.3 Capabilities

A **capability** is a scoped authority grant. It answers who may perform which operation against which resource, under what constraints, for what purpose, for how long, and under which policy epoch. It does not prove that the requester is generally trustworthy or that the requested operation is safe in every context.

Capabilities MUST be unforgeable or integrity protected; attenuable; revocable; time-bounded where practical; bound to a policy epoch; and interpreted by an enforcement component independent from the requester. A capability MUST NOT be accepted solely because its JSON fields look plausible: the implementation must authenticate the capability's provenance by an approved local mechanism.

## 1.4 Requests, decisions, and effects

A **request** asks for a single effect or a bounded transaction. A **decision** is a deterministic result of evaluating a request against a defined policy snapshot and evidence set. An **effect** is an observable change outside the pure evaluator: writing data, launching code, exposing a device, transmitting data, changing policy, or altering system state.

The evaluator MUST be side-effect free. It returns a decision; a separate executor uses an authorization token or short-lived capability to carry out the effect. The act of authorizing is not the act of executing.

## 1.5 Trusted computing base (TCB)

Each release MUST publish a TCB inventory identifying all components whose compromise could violate a named security claim. The inventory includes boot roots, separation core, capability enforcement, policy evaluator, key-handling boundaries, update verifier, capsule isolation mechanisms, and hardware/firmware assumptions. The project MUST minimize the TCB and explain every expansion.

---

# 2. Canonical data and protocol rules

## 2.1 Canonical encoding

Security-significant objects are signed or hashed only after conversion to a canonical byte representation defined by a versioned encoding profile. Implementations MUST NOT sign an arbitrary serializer's output and then assume another serializer will recreate the same bytes. Duplicate keys, invalid UTF-8, unknown critical fields, ambiguous number formats, and noncanonical encodings MUST be rejected.

The first prototype may use canonical JSON for testability, but its exact canonicalization algorithm MUST be specified and tested before production signing. No signing implementation may be released until the encoding, domain separation, signature envelope, key rotation, algorithm allowlist, and downgrade handling have been independently reviewed. Senvoryn MUST use established cryptographic primitives and MUST NOT invent a cipher, hash, signature scheme, key exchange, or randomness generator.

## 2.2 Schema versioning

Every security-relevant object MUST contain `schema` with a stable identifier and semantic version. Unknown optional fields may be ignored only when the schema explicitly marks them as noncritical. Unknown critical fields cause rejection. Breaking changes require a new major schema version and a documented migration path.

## 2.3 Identifier rules

Identifiers are opaque, case-sensitive strings in protocol logic. The UI may provide human-readable aliases, but aliases MUST be resolved to canonical IDs before decision evaluation. IDs MUST NOT encode secrets or personal data. IDs must be unique within the stated namespace and must not be recycled while stale references could plausibly remain valid.

## 2.4 Time and freshness

Wall-clock time can be wrong, reset, unavailable, or maliciously manipulated. Security decisions should use a monotonic clock and an authenticated time source when available. A timestamp alone is not anti-replay protection. Requests require a nonce or monotonic sequence, recipient binding, expiry, and replay-cache or epoch semantics.

If the clock trust is uncertain, a high-risk action requiring precise expiry MUST be denied or require a separately defined offline recovery path. The system must not extend a capability merely because the time service is offline. Expiry checks need a documented boot-epoch strategy so a monotonic timer reset cannot accidentally revive old grants.

## 2.5 Size limits and parsing

Each protocol defines maximum message size, nesting depth, string length, list length, and number of attached capabilities. Values are profile-specific but MUST be finite and enforced before allocation. Parsers must be memory-safe where possible and fuzzed with malformed, truncated, oversized, duplicate-key, deeply nested, and conflicting inputs. Parse failure must not mutate state.

---

# 3. SCP request contract

## 3.1 Request envelope

Every privileged request MUST contain:
- `schema` and `request_id`;
- `principal_id` and authenticated session/channel binding;
- intended `recipient_id` and target device/node;
- canonical `resource_id` and `resource_type`;
- `operation` and declared purpose;
- referenced capability IDs or a request for a grant;
- `policy_epoch` and required freshness fields;
- risk tier, data classification, and external-effect declaration;
- optional approval references;
- bounded resource budget and deadline;
- payload digest or canonical payload reference;
- anti-replay nonce/sequence;
- response correlation identifier.

The caller may not self-assign a lower risk tier or mark an operation reversible. Risk and reversibility are determined by the policy registry using the canonical target and operation. A caller-provided classification is treated as a claim and cross-checked.

### 3.1.1 Trusted risk registry

The SCP MUST derive the effective risk tier from a signed, versioned Operation Registry keyed by canonical resource type, operation, relevant object attributes, data classification, scope, and external effects. The request's risk claim is never authoritative. If the claim is lower than the registry result, the request is denied with `RISK_CLAIM_MISMATCH`; the system MUST NOT honor the lower value. Unknown operations have no default low-risk classification and are denied with `UNKNOWN_OPERATION_RISK`. When multiple rules apply, the most restrictive risk tier wins. The registry itself is a security-critical policy artifact and requires provenance, review, staged rollout, and rollback.

## 3.2 Validation order

Implementations MUST validate in the following order, or in an order proven equivalent that cannot leak privileged effects before all gates pass:

1. **Transport/size gate:** enforce message and rate limits; reject malformed framing.
2. **Canonical parse gate:** reject ambiguous encoding, duplicate fields, unsupported critical fields, and invalid types.
3. **Recipient gate:** ensure this node/service is the intended recipient.
4. **Identity gate:** authenticate the principal and bind it to the channel/session.
5. **Freshness gate:** check nonce, sequence, expiry, replay cache, and boot epoch.
6. **Object resolution gate:** resolve the resource to a canonical ID and version.
7. **Revocation gate:** check principal, credential, capability, key, device, and policy revocations.
8. **Mode gate:** apply the current system state (normal, degraded, read-only, recovery, quarantined, maintenance, offline).
9. **Policy epoch gate:** require an acceptable policy version; stale high-risk requests are denied. The registry in force must be the registry for that epoch, and it must carry a validity window; no decision may be made under a registry whose window has not opened or has closed, and a wall clock the environment will not vouch for must be refused rather than believed, because a clock that moves backwards is the one way an expired document can look current.
10. **Capability gate:** verify signature/authority provenance and all scope dimensions.
11. **Risk/consent gate:** confirm required approvals are authentic, fresh, and bound to this exact action digest.
12. **Budget gate:** confirm CPU, memory, I/O, network, duration, and monetary/resource budgets are within policy.
13. **Precondition gate:** confirm expected state, target version, dependencies, and recovery prerequisites.
14. **Audit readiness gate:** determine whether the required audit level can be recorded. For high-risk changes, unavailable required audit storage causes denial.
15. **Decision construction:** return permit/challenge/deny/defer/quarantine with constraints and reason codes.
16. **Effect separation:** no effects are performed by the evaluator. The executor independently rechecks authorization and transaction preconditions immediately before acting.

A rejection at any gate produces no privileged effect. Denial messages must not reveal secret resource names or policy details to an unauthorized requester.

## 3.3 Decision vocabulary

- `PERMIT`: all mandatory checks passed; narrowly scoped execution authority may be issued.
- `DENY`: a mandatory check failed.
- `CHALLENGE`: more evidence or explicit approval is required; no effect occurs.
- `DEFER`: operation is non-destructive and waits for a known dependency or policy window.
- `QUARANTINE`: a defined suspicious principal/request is isolated by policy; quarantine is not a generic penalty for all errors.

Every decision includes `decision_id`, `request_id`, status, reason codes, policy epoch, evaluated target/version, required next step if any, capability scope if permitted, expiry, and audit-event reference. `PERMIT` without a finite scope is invalid. `CHALLENGE` and `DEFER` grant no partial execution authority.

## 3.4 Default failure behavior

- Missing principal or capability: `DENY`.
- Unknown or stale policy epoch for privileged action: `DENY` or `CHALLENGE`; never silent permit.
- Revoked or expired capability: `DENY`.
- Unknown resource: `DENY` without dereferencing an untrusted path.
- Audit unavailable for a high-risk operation: `DENY`.
- SCP coordinator unavailable: no new high-risk grants; already-issued capabilities remain valid only until their expiry/revocation and within their original scope.
- AI unavailable: operate without AI.
- Remote controller unavailable: enforce local policy; do not widen authority.
- Required hardware security property unknown: reject high-assurance profile or mark reduced assurance explicitly.

---

# 4. Capability semantics

## 4.1 Required dimensions

A capability MUST bind all of the following dimensions: `subject`, `resource`, `operations`, `purpose`, `constraints`, `not_before`, `expires_at` or a defined lease, `policy_epoch`, `issuer`, `delegation_depth`, `revocation_id`, and provenance proof. The schema provides syntax; the enforcement implementation provides authenticated semantics.

Constraints can include allowed path/object set, network endpoints, ports, protocol, bytes, rate, duration, device functions, data labels, process ancestry, user session, capsule identity, operation count, approval ID, and permitted time window. Unrecognized constraints on a security-sensitive capability MUST cause rejection rather than be silently ignored.

## 4.2 Attenuation

A child capability may only be equal to or narrower than its parent. At minimum:
- child subject must be explicit and cannot spoof the parent;
- child resource scope must be a subset of the parent scope;
- child operations must be a subset;
- child expiry cannot exceed parent expiry;
- child budget cannot exceed parent budget;
- child delegation depth must decrease;
- child purpose cannot be less restrictive;
- child policy epoch must remain acceptable under current revocation policy.

The implementation must define subset semantics for each resource type. String prefix matching is not a valid general-purpose rule for filesystem or network scope.

## 4.3 Revocation

Revocation is performed through a monotonic revocation epoch or authenticated revocation record. The implementation must define the maximum time an issued capability can remain usable after revocation under connected and offline conditions. High-risk capabilities should be short-lived. Local revocation takes effect before new privileged actions; in-flight operations stop at the next safe boundary and enter a documented cancellation/compensation flow.

A disconnected node cannot learn about a remote revocation immediately. This is a real limitation, not something a design can wish away. Therefore remote authority is constrained by short leases, local policy and expiry. The profile's maximum stale-authority window must be published.

## 4.4 Capability use is not approval forever

Possession of a capability is necessary but not always sufficient: the current system mode, risk policy, resource quota, target version and consent constraints must still pass. A capability does not override a read-only, recovery, quarantined or policy-revoked state.

---

# 5. Change Capsule transaction contract

## 5.1 Capsule model

A Change Capsule is an immutable plan plus evidence and scoped requested authority. It is not an executable script with root privilege. Its operations are selected from a versioned operation registry. Unknown operation names are rejected. Every operation declares preconditions, effects, risk, resource bounds, validation, cancellation points, and rollback/compensation semantics.

## 5.2 Normative state machine

`DRAFT → VALIDATED → AUTHORIZED → STAGED → PRECHECKED → APPLYING → VERIFYING → COMMITTED`

Alternative states: `REJECTED`, `EXPIRED`, `CANCELLED`, `FAILED`, `COMPENSATING`, `ROLLED_BACK`, `PARTIAL`, `NEEDS_HUMAN_RECOVERY`.

Allowed transitions are explicitly listed in the implementation's machine-readable state table. No transition may skip validation or authorization. `COMMITTED` requires verified postconditions. If the outcome cannot be proven, use `PARTIAL` or `NEEDS_HUMAN_RECOVERY`, not `COMMITTED`.

## 5.3 Required transition semantics

- `DRAFT → VALIDATED`: schemas, signatures, dependencies, operation allowlist, bounds and preconditions pass.
- `VALIDATED → AUTHORIZED`: SCP decision permits exact plan digest and target state; required user/dual approvals bind that digest.
- `AUTHORIZED → STAGED`: all artifacts are staged to non-active storage; hashes/signatures rechecked.
- `STAGED → PRECHECKED`: current state, policy epoch, free space, power/thermal constraints, dependencies and recovery materials are checked again.
- `PRECHECKED → APPLYING`: executor receives temporary capabilities specific to this capsule and step.
- `APPLYING → VERIFYING`: every step has an outcome; no undeclared side effect occurred to the extent observable.
- `VERIFYING → COMMITTED`: all required postconditions pass and durable evidence is recorded.
- Any pre-effect failure goes to `FAILED` or `REJECTED` without mutation.
- Failure after mutation goes to `COMPENSATING`, `ROLLED_BACK`, `PARTIAL`, or `NEEDS_HUMAN_RECOVERY` according to the declared effect semantics.

## 5.4 Atomicity and external effects

Local storage swaps may be atomic if the platform proves the necessary primitive. Network actions, external purchases, messages sent, remote deletions, firmware changes and other out-of-process effects often cannot be atomically rolled back. These are `COMPENSATABLE` or `NON_REVERSIBLE` operations. The capsule must state this before approval. “Undo” means either restore known local state or issue a compensating action; it must not imply the external world has been rewound.

## 5.5 Replay, idempotency and concurrency

Every capsule has a unique ID, content digest, target version, expiry and replay key. A repeated request with the same idempotency key returns the recorded result or resumes reconciliation; it must not blindly repeat a destructive operation. Before each step, target preconditions and policy epoch are rechecked. Concurrent capsules that touch overlapping resources must be serialized or rejected unless a validated conflict-free schedule exists.

## 5.6 Cancellation

Cancellation is cooperative at declared safe points. The UI must disclose if an active step cannot be stopped immediately. Once cancellation is accepted, no new step begins. The executor completes a minimal safety action if aborting mid-operation would corrupt state, then follows the compensation plan. Cancellation never deletes the audit record.

---

# 6. Origin Capsule contract

## 6.1 Required boundaries

An Origin Capsule (OC) is a security domain for an application, service, model, driver adapter or imported runtime. Each OC is assigned a unique immutable ID, trust/provenance metadata, runtime profile, storage namespace, network policy, device grants, IPC peers, resource budget, update policy, audit policy, and compatibility status.

The default OC has:
- no host administrator capability;
- no unrestricted host filesystem access;
- no access to another capsule's writable data;
- no network route unless granted;
- no access to secrets except specific brokered secret handles;
- no device/sensor access unless granted;
- no access to SCP internals beyond a narrow request API;
- no ability to modify trust roots or audit history;
- finite CPU, memory, storage, process count, I/O, and network budgets.

## 6.2 Host bridge rules

Filesystem sharing, clipboard, screen capture, camera, microphone, location, GPU, USB, display, accessibility and network are explicit bridges. Each bridge defines direction, operations, data label, lease, user indicator, budget, logging class, and revocation semantics. “Bridge available” is not the same as “bridge authorized.” A bridge MUST NOT be enabled simply because a guest OS exposes that feature.

## 6.3 Lifecycle

`CREATED → VERIFIED → CONFIGURED → READY → RUNNING → STOPPING → STOPPED`. Exceptional states include `QUARANTINED`, `CRASHED`, `REVOKED`, and `REMOVAL_PENDING`. Each transition requires a defined action and bounded timeout. A capsule crash does not imply host crash. Revocation blocks new grants and initiates termination or restricted read-only recovery according to policy.

## 6.4 Compatibility evidence

Every compatibility claim is keyed by application version, guest/runtime image digest, adapter version, hardware profile and test-suite revision. The record contains test results, failures, required external services, vendor restrictions, known unsupported features and performance overhead. “Runs” must be differentiated from “core workflow works” and “verified profile.”

---

# 7. Boot, Direct Installer, updates, and recovery

## 7.1 Boot trust chain

The boot protocol MUST identify each stage that is measured or verified, the relevant key hierarchy, fallback state, revocation data, and failure behavior. Trust is conditional on hardware/firmware assumptions and must be reported accordingly. If secure boot is unavailable, the system reports a reduced assurance profile rather than falsely claiming equivalent protection.

A boot manifest identifies component hashes, versions, hardware profile, dependency order, signing identity, policy epoch, anti-rollback version, and recovery artifact. The verifier must reject an invalid signature, unsupported critical schema, revoked signer, disallowed downgrade, hash mismatch or profile mismatch.

## 7.2 Direct Installer contract

The Direct Installer should offer an application-like install on certified host configurations, without user-created boot media. It MUST NOT promise that every firmware/OS combination supports this path. It MUST verify installer provenance before elevated operations, identify the exact target, display effects, check power/storage/encryption/recovery prerequisites, and separate non-destructive install, side-by-side install, virtualized trial, and destructive replacement modes.

Destructive operations require fresh strong authentication, a clear target identifier (not merely a disk label), explicit confirmation of the data-loss consequence, and a second confirmation if a mistake could wipe the only available recovery copy. The installer must not silently disable host security, bypass firmware requirements, or repartition unrelated storage.

## 7.3 Installer transaction states

`DISCOVER → COMPATIBILITY_CHECK → PLAN → USER_REVIEW → AUTHORIZED → DOWNLOAD → VERIFY → STAGE → BOOT_CONFIG_STAGE → PRECOMMIT_CHECK → COMMIT → FIRST_BOOT_VERIFY → COMPLETE`.

Failure states: `CANCELLED`, `FAILED_NO_MUTATION`, `ROLLBACK_REQUIRED`, `RECOVERY_REQUIRED`, `UNSUPPORTED`. The installer must record the last completed checkpoint and provide recovery instructions. Network loss during download must leave the host unchanged. Power loss after staging must leave either the previous boot path usable or a documented recovery path available.

## 7.4 Update transaction

Updates use authenticated metadata, content-addressed artifacts, dependency resolution, anti-rollback rules, staging, activation, health verification and commit/rollback. Signature verification occurs before artifact execution or activation. The updater must not use a compromised application process to authorize its own update.

## 7.5 Recovery semantics

Recovery must be possible without the normal network controller and without depending solely on the damaged active generation. Recovery artifacts and instructions must be verified, and recovery itself must obey least privilege. A rollback of system configuration must not automatically roll back security revocations, delete incident evidence, or reverse external effects. Security state and operational state have separate rollback rules.

---

# 8. AI action contract

## 8.1 Intent record

An Intent Record contains requester, goal, explicit constraints, permitted data categories, target objects, ambiguity flags, risk classification from policy, expected outcome, expiry, and consent basis. The original prompt may be retained only under the session's privacy policy; it is not the canonical authority record.

## 8.2 AI action envelope

Every action proposed by AI must be serialized into a typed Action Envelope with `intent_id`, `agent_id`, `model_id`, `tool_id`, canonical target, requested operations, input data labels, predicted effects, risk claim, resource budget, external destinations, expiry, and explanation/evidence references. The envelope is untrusted until validated. The model's risk claim is not authoritative.

## 8.3 Planning/execution separation

AI planning has read access only to authorized context and has no direct write path to kernel, SCP, filesystem, secrets, or network. It requests tools through a broker. The broker resolves the target, calculates actual risk, validates capabilities, checks consent, and invokes the SCP. UI confirmations are generated from canonical envelope fields, not from free-form model prose. The model cannot alter the confirmation after approval without invalidating the action digest.

## 8.4 Autonomy classes

- `A0 Explain`: read-only within granted scope.
- `A1 Recommend`: create a plan; no effects.
- `A2 Bounded reversible`: execute allowlisted low-risk actions under budget and rollback constraints.
- `A3 Pre-authorized workflow`: execute a narrow recurring workflow that the user explicitly configured; notify and stop on any out-of-scope condition.
- `A4 Critical`: fresh strong human approval for each critical transaction; no default autonomous mode.

Autonomy class never bypasses SCP. The user can reduce autonomy immediately. Increasing autonomy requires a visible change summary and new approval. Model confidence alone is never an authorization proof.

## 8.5 External inference

Before user data is sent to remote inference, the system must determine data classification, destination, purpose, retention terms if known, and consent state. If these cannot be established, protected data is not sent. Local-only mode must be real and testable, not a UI label that silently falls back to a remote service.

## 8.6 Prompt injection and tool abuse

All retrieved content is untrusted for authority. Instructions contained in websites, email, documents, logs or tool results cannot override system policy. High-risk tool execution requires a new deterministic decision and may require user confirmation. Tests must include tool-call laundering, indirect prompt injection, memory poisoning, cross-user data leakage, model update rollback, and resource-exhaustion prompts.

---

# 9. Universal Execution Fabric (UEF)

## 9.1 Execution-selection algorithm

Given an application manifest and target hardware profile, the selector MUST:
1. validate manifest signature/provenance and schema;
2. identify architecture, ABI, runtime, kernel/service dependencies, entitlements, device requirements and external services;
3. evaluate legal/licensing and signing constraints;
4. select the least-complex compatible execution mode meeting required semantics;
5. generate an Origin Capsule manifest with all bridges disabled by default;
6. compare requested permissions with policy;
7. report compatibility tier and known limitations;
8. refuse execution if required integrity or isolation cannot be provided.

The UEF must never silently substitute a weaker security profile without notice. If it cannot deliver a feature, it must report the missing dependency instead of simulating success.

## 9.2 Compatibility levels

- `C0 NOT_ASSESSED`: no current evidence.
- `C1 LAUNCH`: startup demonstrated; workflow not validated.
- `C2 CORE_WORKFLOW`: published core test suite passes.
- `C3 BROAD_FUNCTIONAL`: expanded conformance suite passes on named profiles.
- `C4 PROFILE_VERIFIED`: exact application/runtime/adapter/hardware combination passes reproducible functionality and isolation suites.

Each record must name the exact versions tested. No tier guarantees all future versions, all peripherals, proprietary services, hardware features, or lawful redistribution.

## 9.3 Guest administrator isolation

A guest administrator can control only guest state. Guest privileged calls must terminate in a guest kernel or runtime boundary and cannot map host memory, inspect host credentials, grant host device access, alter host update policy, or reach the SCP's privileged endpoints. Host/guest bridges are explicit, typed and revocable.

---

# 10. Privacy, storage, network and continuity contracts

## 10.1 Data classes

Minimum data labels: `PUBLIC`, `INTERNAL`, `PERSONAL`, `SENSITIVE`, `SECRET`, `SECURITY_EVIDENCE`. Data labels govern allowed storage, logging, export, remote processing and retention. Classification is conservative when unknown. Apps cannot downgrade a label merely by copying the data into another container.

## 10.2 Information-flow request

A data-flow authorization includes source object, source label, requester, purpose, destination, transformation, retention, and user consent if required. Destination identity must be canonicalized before approval. Any change in destination, purpose, or data scope invalidates the prior approval.

## 10.3 Secrets

Secret material is accessed through narrow brokers, not environment variables or general filesystem mounts by default. Secret handles specify permitted use, target service, operation count, expiry, and audit policy. Secrets must not be copied into audit logs, model context, crash dumps, or support bundles without explicit separately scoped authorization.

## 10.4 Deletion and retention

Deletion semantics depend on storage technology and copies. The OS must distinguish logical deletion, cryptographic erasure, overwrite where meaningful, replica deletion, backup expiry, and deletion from external services. It must not claim that a deleted file is physically unrecoverable unless the storage method and threat assumptions support that claim. Data export and deletion interfaces must list scope and known residual copies.

## 10.5 Federation

Remote nodes use authenticated identities and signed scoped requests. Fleet control can propose desired state; each node performs local authorization. A controller compromise must not automatically give access to local secrets or unrestricted shell execution on every node. Fleet policy has expiry, revocation, node quarantine, offline behavior, quorum rules for critical changes, and per-node audit.

---

# 11. Failure states and operational semantics

## 11.1 System modes

- `NORMAL`: all baseline policy services available.
- `DEGRADED`: noncritical features disabled or bounded due to pressure/faults.
- `READ_ONLY`: writes blocked except explicitly allowed recovery logging and essential state.
- `RECOVERY`: only verified recovery operations permitted.
- `QUARANTINED`: the affected principal/device/component is constrained after a policy-defined trigger.
- `MAINTENANCE`: approved administrative changes allowed under increased audit.
- `OFFLINE`: network unavailable; local policy remains effective.

Each transition has a machine-readable guard, source state, target state, initiating principal or detector, required evidence, allowed actions, timeout, and exit test. State labels are not enforcement by themselves; enforcement checks the mode on every privileged request.

## 11.2 Fail-safe vs fail-operational

Security-critical functions default fail-safe: deny an unsafe operation. Availability-critical services may fail-operational only under a profile-specific, preapproved policy with bounded privileges and documented consequences. There is no generic “ignore security to keep running” switch.

## 11.3 Fault containment

Fault propagation is modeled as a graph. For every critical component, the design identifies dependents, failure detection, containment action, recovery action, evidence retained, and risk of recovery recursion. Recovery coordinators must have independent resource reserve and must not rely on the failed component as their only source of truth.

---

# 12. Test and evidence contract

## 12.1 Test layers

1. **Schema tests:** required fields, types, enum limits, additional-property rejection for signed/security objects.
2. **Unit tests:** pure policy, state transition and attenuation functions.
3. **Property tests:** attenuation never broadens authority; denied requests create no effects; schema version change never silently grants new fields.
4. **Fuzz tests:** request parsers, manifest readers, IPC framing, update metadata, capsule configuration.
5. **Integration tests:** identity + capability + SCP + executor + audit interactions.
6. **Fault injection:** process crash, lost storage, stale time, full disk, network loss, power interruption, clock rollback, corrupted artifact.
7. **Adversarial AI tests:** indirect prompt injection, wrong-target action, exfiltration attempt, stale memory, tool spoofing, overbroad plan, malicious model package.
8. **Compatibility tests:** exact application versions, workflows, bridge policies, performance and isolation tests.
9. **Performance tests:** benchmark methodology, tails/variance, resource profiles, energy where relevant.
10. **Independent review:** security and assurance review before production claims.

## 12.2 Required negative tests

The suite MUST verify at least:
- unknown principal is denied;
- missing capability is denied;
- expired/revoked capability is denied;
- wrong resource, operation, purpose, subject or policy epoch is denied;
- remote controller cannot bypass local policy;
- AI cannot change its own autonomy class;
- guest administrator cannot request host resource authority;
- noncanonical or oversized input is rejected without mutation;
- capsule replay does not repeat destructive effects;
- installer power loss leaves either the prior boot path or recovery path usable;
- user consent for one action cannot approve a modified action digest;
- unknown critical schema field is rejected;
- full audit store blocks high-risk operations under the defined profile;
- untrusted content cannot instruct the AI to bypass policy;
- offline operation does not widen authority;
- data labeled SECRET is not exported to remote inference without explicit policy approval;
- failed postcondition cannot result in COMMITTED state.

## 12.3 Evidence record

Every test evidence record includes requirement ID, test ID, source revision, test harness version, profile, environment, inputs/fixture hash, expected result, actual result, result status, logs/artifact references, reviewer, date, and known limitations. A passing unit test is not evidence of the absence of all vulnerabilities; evidence claims must match test scope.

## 12.4 Release gate

No profile may be called production-ready until the release bundle includes: source revision, SBOM and license report, signed artifacts, build instructions, reproducibility status, threat model, TCB list, test summary, independent review, vulnerability contact, update/rollback evidence, recovery drill, supported profile list, and known limitations.

---

# 13. Provisional targets and measurement rules

The following are initial hypotheses for the reference profile, not promises. A target becomes binding only after the architecture board approves reference hardware and measurement methodology.

| Metric | Initial target | Measurement constraint |
|---|---:|---|
| SCP local cached policy p99 | < 5 ms | Declared reference CPU, 10k+ requests, warmed and cold results reported separately |
| Small service startup p95 | < 250 ms | Dependency-ready boundary and service definition published |
| Idle minimal server RAM | < 256 MiB | Excludes workload memory and optional AI; exact profile and accounting method disclosed |
| Recovery to prior bootable generation | < 120 s | Reference storage/device; excludes user-data restoration and physical repair |
| AI-disabled core operation | 100% of core boot/login/policy/update/recovery tests | Test suite and build profile published |
| Privileged request audit coverage | 100% of defined privileged operation registry | Includes failure and cancellation paths; exceptions reviewed |
| Direct install without user-created boot media | Per-certified-host success rate published | Host OS, firmware, disk mode and recovery conditions listed |

Benchmarks must include hardware model, firmware, workload, compiler/toolchain, build options, sample size, latency distribution, variance, thermal state, power mode, and raw data where safe. Performance cannot be reported without the isolation profile used by that run.

---

# 14. Architecture decisions and unresolved questions

## ADR-001 — Clean-sheet separation core

**Decision:** pursue an original, minimal separation core as a research program. Start in an emulator with a narrow supported architecture. Do not promise multi-architecture parity until the object model, memory boundary, interrupt model, boot path and test suite are proven.

**Rationale:** an original kernel meets the project's clean-sheet goal but has the highest cost and risk. Narrow initial scope lowers the chance of an unmaintainable design.

**Rejected shortcut:** declaring full OS security based on an untested prototype or adding broad services inside the kernel for delivery speed.

**Open items:** initial ISA, boot protocol, page-table model, memory safety strategy, formal-method scope, debugging design, and hardware attestation profiles.

## ADR-002 — Cryptographic primitives

**Decision:** use reviewed, widely analyzed constructions through a small replaceable crypto boundary. No bespoke cryptography. Algorithms are allowlisted and versioned; key rotation and algorithm migration are specified before production.

**Open items:** exact suite selection, hardware-backed key support, signing quorum, offline recovery and compromise response.

## ADR-003 — Canonical protocol representation

**Decision:** the prototype may use deterministic JSON for inspectability, but production signing requires a reviewed canonical byte encoding and parser. The selected encoding is an architecture decision, not an accidental library behavior.

**Open items:** canonicalization standard, binary framing, compatibility policy and fuzzing corpus.

## ADR-004 — AI isolation

**Decision:** models operate outside the authority enforcement core and communicate via typed action envelopes to a broker. Local rule-based fallback is mandatory for core operations.

**Open items:** local inference runtime, model update attestation, memory deletion semantics, evaluation methodology and accelerator isolation.

## ADR-005 — Universal compatibility

**Decision:** use a layered compatibility fabric with native API contracts, translators and isolated guest environments. Compatibility claims are per application/profile and evidence-based.

**Open items:** supported guest architectures, licensing/redistribution, GPU virtualization, mobile service dependencies, and app-store entitlements.

## ADR-006 — Direct installation

**Decision:** target no-user-created-boot-media installation on certified host environments. Preserve a media-based recovery route where required by firmware and recovery constraints, without making it the ordinary path.

**Open items:** boot entry management per host OS, secure-boot enrollment, disk encryption, rollback limitations, dual-boot support and data migration.

## ADR-007 — Local-first control

**Decision:** each node owns local enforcement. Federation can send constrained desired-state requests; it cannot bypass node-local capabilities and mode checks.

**Open items:** quorum for fleet-wide policy, split-brain behavior, key revocation while offline, and per-tenant isolation.

---

# 15. Implementation sequence and exit criteria

## Gate 0 — Contract freeze for prototype

Deliver: accepted identifiers, typed object model, first schemas, threat model, state machine, error registry, reference policy evaluator and traceability matrix. Exit only when ambiguity review identifies no unresolved issue that would change authority semantics.

## Gate 1 — SCP decision oracle

Deliver: pure evaluator, capability model, deterministic reason codes, audit-event builder, property tests and fuzzable parser boundary. Exit when all required negative cases pass and a reviewer confirms the evaluator has no side effects.

## Gate 2 — Change Capsule executor simulator

Deliver: state machine, operation registry, idempotency ledger, stale-precondition detection, rollback/compensation simulation and fault injection. Exit when interrupted operations never falsely report COMMITTED.

## Gate 3 — Capsule policy simulator

Deliver: Origin Capsule manifest, bridge registry, budget checks and guest/host authority tests. Exit when all cross-boundary tests prove that guest authority cannot become host authority in the test model.

## Gate 4 — Minimal bootable prototype

Deliver: one declared hardware/emulator profile, trusted boot chain, minimal service supervisor, local policy enforcement and recovery path. No production claim at this stage.

## Gate 5 — Direct Installer prototype

Deliver: one certified host profile, safe staging, cancellation, power-loss simulation, disk target confirmation, first-boot verification and recovery documentation.

## Gate 6 — UEF proof of concept

Deliver: one chosen foreign application profile, compatibility record, guest boundary, bridge policy and adversarial tests. Add formats only after evidence.

## Gate 7 — AI-native transaction layer

Deliver: Intent Record, Action Envelope, model service boundary, tool broker, prompt-injection corpus and autonomy controls. AI-disabled core tests must still pass.

## Gate 8 — Security review and public alpha

Deliver: independent review, vulnerability disclosure process, reproducible build report, SBOM, release signatures, threat model, performance data, profile support matrix, user data export, update recovery drill and release claims ledger.

No gate is met because a calendar date was reached. Gate acceptance requires artifact links, results, review, and explicit limitations.

---

# 16. Conformance test identifiers

The table below is the minimum traceability contract. Detailed test implementations may be split by language or environment, but these IDs remain stable.

| Requirement ID | Behavior | Minimum evidence | Priority |
|---|---|---|---|
| SCP-001 | Unknown principal denied | Unit + integration test | P0 |
| SCP-002 | Missing capability denied | Unit test | P0 |
| SCP-003 | Expired/revoked capability denied | Unit + revocation integration | P0 |
| SCP-004 | Subject/resource/operation/purpose must match | Parameterized unit tests | P0 |
| SCP-005 | Policy epoch mismatch cannot silently permit | Unit + offline test | P0 |
| SCP-006 | High-risk action needs fresh approval bound to exact digest | Integration + replay test | P0 |
| SCP-007 | Evaluator performs no side effects | Static/API boundary test | P0 |
| SCP-008 | Audit unavailable blocks high-risk action | Fault-injection test | P0 |
| CAP-001 | Child capability is a subset of parent | Property-based test | P0 |
| CAP-002 | Revocation blocks next privileged step | Integration test | P0 |
| CC-001 | Illegal capsule transition rejected | State-machine test | P0 |
| CC-002 | Replayed capsule does not repeat effect | Replay/idempotency test | P0 |
| CC-003 | Failed postcondition cannot commit | Fault-injection test | P0 |
| CC-004 | Cancellation stops new steps and preserves audit | Integration test | P0 |
| OC-001 | Default capsule has no host filesystem access | Boundary test | P0 |
| OC-002 | Guest admin cannot obtain host capability | Adversarial integration test | P0 |
| OC-003 | Network bridge denied by default | Network namespace/firewall test | P0 |
| AI-001 | AI cannot issue capability directly | Architecture/static check + runtime test | P0 |
| AI-002 | Modified action digest invalidates approval | Replay test | P0 |
| AI-003 | Remote inference requires authorized data-flow grant | Egress test | P0 |
| AI-004 | Prompt injection cannot change policy/authority | Adversarial corpus | P0 |
| INS-001 | Invalid installer signature rejected before elevation | Installer test | P0 |
| INS-002 | Destructive target shown and explicitly confirmed | UI automation + audit test | P0 |
| INS-003 | Power loss preserves prior boot or recovery path | Fault injection on named profile | P0 |
| UPD-001 | Invalid signature/downgrade rejected | Updater test | P0 |
| UPD-002 | Health failure rolls back or enters explicit recovery | Fault injection | P0 |
| PRIV-001 | No nonessential telemetry on fresh install | Network egress test | P0 |
| PRIV-002 | Secrets absent from logs/crash bundle | Canary-secret scan | P0 |
| FED-001 | Remote node command is locally re-authorized | Multi-node integration | P0 |
| EVID-001 | Each release claim has evidence and limitations | Release-bundle audit | P0 |

---

# 17. Final implementation rule

A feature is not complete when it compiles or appears in the UI. It is complete only when its authority model, failure behavior, privacy behavior, compatibility scope, recovery path, resource limits, test evidence, and user-facing explanation are all defined and reviewed.

The first product milestone is not “AI that can control the OS.” It is **a small, deterministic authority core that can safely accept, constrain, audit, and reject requests from every other component—including AI**. Once that foundation is testable, the rest of the system can grow without making every new feature a new root of trust.

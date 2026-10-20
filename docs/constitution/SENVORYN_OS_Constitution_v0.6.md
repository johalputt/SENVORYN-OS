# SENVORYN OS
## Founding Constitution and Normative Technical Blueprint

**Version:** 0.6.0  
**Status:** Deep implementation specification — proposed for review, not ratified  
**Date:** 15 October 2026  
**License intent:** Original Senvoryn code and documentation under Apache License 2.0, subject to the component-level licensing rules in this document  
**Project name:** Senvoryn is a provisional project name pending trademark, domain, repository, and company-name clearance

> **Founding promise:** A person should be able to install, understand, control, and recover their computer without needing to become a systems engineer. Software should run inside a clearly bounded environment; intelligence should help the owner rather than rule the machine; and every important claim about security, privacy, speed, or compatibility must be testable.

---

## Document status and how to interpret it

This document is intended to become the single top-level design authority for Senvoryn OS. It combines constitutional principles with an initial normative technical specification so that implementers do not have to guess what the principles mean in code.

The document is not proof that Senvoryn has been implemented, independently audited, or shown to be secure. It is a design contract. Any statement not yet demonstrated by implementation and test evidence must be labelled **proposed**, **unverified**, or **research required** in release documentation.

Normative words have the following meanings:

- **MUST / MUST NOT:** a binding requirement for a conforming implementation.
- **SHOULD / SHOULD NOT:** a strong requirement; deviation requires a written rationale and review.
- **MAY:** an optional capability that must not undermine a MUST requirement.
- **TARGET:** a measurable engineering goal, not a guarantee until benchmarked on a defined reference profile.
- **EXPERIMENTAL:** a feature that is disabled by default and must not be relied upon for safety or security claims.
- **PROHIBITED:** a behavior forbidden by the constitution, even if it is convenient or improves a benchmark.

When clauses conflict, precedence is: (1) constitutional invariants; (2) security and privacy requirements; (3) approved architecture decision records; (4) component specifications; (5) implementation convenience. A lower-level document cannot silently weaken a higher-level rule.

The word **owner** means the person or organization with legitimate authority over a device or data set. **User** means a person currently using the system. **Administrator** means a principal granted an explicit administrative role. Ownership does not make every action safe; destructive actions still require clear scope and confirmation.

> **Version 0.6.0 note:** The original v0.5 constitution is preserved as the baseline. Parts VI–XX add controlling micro-level contracts, failure semantics, data schemas, acceptance tests, and a stricter implementation sequence.

## Executive summary

Senvoryn is a proposed clean-sheet operating-system platform. It is not a repackaged existing distribution and it is not defined as a port of another operating system's kernel. Its distinctive goal is to unify five ideas into one coherent system:

1. **Intent-first computing:** people describe an outcome; the system converts it into an inspectable, bounded plan before acting.
2. **Capability-governed authority:** no process, app, driver, administrator session, remote controller, or AI model receives authority merely because it asks for it.
3. **Origin Capsules:** applications and imported execution environments receive isolated, individually permissioned environments with explicit data and device bridges.
4. **Sovereign adaptation:** local intelligence can diagnose, predict, optimize, and propose recovery while a small deterministic enforcement core retains final authority.
5. **Application-like installation:** on certified existing systems, installation should start like an ordinary signed application, avoid user-created boot media, preserve existing data unless replacement is explicitly requested, and recover safely from interruption. Rebooting may still be required.

Senvoryn aims to run on devices ranging from constrained edge hardware to personal computers and large distributed systems. It aims to execute software from multiple ecosystems through native runtimes, API translation, instruction translation, and isolated guest environments. The objective is broad, evidence-backed compatibility—not an unsupported promise that every application from every platform will work on every device. Software can require unavailable hardware, proprietary services, platform attestation, vendor signing, licenses, or an authorized guest operating system. Those limits must be visible rather than hidden.

Senvoryn's original code is intended to be Apache-2.0 licensed. The project will not import an existing operating-system kernel or silently copy proprietary or incompatible code. It will use a transparent toolchain and a controlled, reviewed set of external build and cryptographic dependencies where necessary. Cryptographic novelty is not a goal: the project MUST use publicly scrutinized cryptographic constructions and MUST NOT invent a proprietary cipher, hash function, signature scheme, or key exchange protocol.

No design can honestly promise absolute security, absolute privacy, no backdoor under every conceivable threat, zero bugs, unlimited scale, or universal software compatibility. Senvoryn's promise is stronger in a different way: define each guarantee, disclose its assumptions, make evidence reproducible, and withdraw a claim if evidence no longer supports it.

---

# PART I — CONSTITUTION

## Article 1 — Identity, purpose, and scope

### 1.1 Identity

The working project name is **Senvoryn OS**. The name remains provisional until formal clearance. Product identity, domain names, repository names, logos, and trademarks MUST be reviewed for conflicts before a public commercial launch. The project MUST NOT imply that a name search is legal clearance.

### 1.2 Purpose

Senvoryn exists to give device owners a secure, private, understandable, adaptive, and recoverable computing environment. It is designed for personal, professional, embedded, edge, server, and multi-node use, with a common conceptual model and hardware-specific conformance profiles.

### 1.3 Primary objectives

The project MUST pursue the following goals together rather than optimizing one while ignoring the rest:

- **Security:** authority is explicit, least-privileged, revocable, and locally enforced.
- **Privacy:** data access and disclosure are controlled by the user and system policy; telemetry is off by default.
- **Usability:** installation, app permissions, updates, recovery, and ordinary administration are understandable to non-specialists.
- **Performance:** overhead is measured by workload and device profile; isolation must be efficient enough to be practical.
- **Adaptivity:** resource management and assistance respond to context, but actions remain bounded by policy.
- **Portability:** one coherent object model and service contract can be implemented across different architectures.
- **Interoperability:** applications from different ecosystems can be accommodated through explicit compatibility environments, subject to technical and legal limits.
- **Transparency:** source, build inputs, release provenance, known limitations, and governance decisions are publicly inspectable.
- **Recoverability:** failed updates, crashed services, compromised capsules, and interrupted installs have defined recovery paths.

### 1.4 Non-goals

Senvoryn MUST NOT promise to defeat every hardware implant, side channel, firmware compromise, supply-chain compromise, coercive legal demand, or physical attack. It MUST NOT promise that all foreign software can be legally or technically executed. It MUST NOT require cloud connectivity for core local functions. It MUST NOT make AI mandatory for boot, authentication, file access, updates, or recovery. It MUST NOT hide limitations behind marketing language.

## Article 2 — Constitutional invariants

The following invariants are supreme. They may be refined for clarity, but may not be weakened by ordinary configuration, vendor extensions, administrative convenience, or AI-generated changes.

**INV-01 — Default deny.** An action without a valid grant is denied. Absence of a policy decision is not consent.

**INV-02 — Authority is explicit.** Every privileged effect is attributable to an authenticated principal, a specific capability, a declared purpose, and an applicable policy version.

**INV-03 — Authority cannot self-escalate.** A component cannot create, widen, or grant itself capabilities. Delegation may only preserve or reduce the delegator's authority.

**INV-04 — AI is never the root of trust.** Model output is untrusted input until parsed, validated, authorized, and constrained by deterministic policy.

**INV-05 — No silent destructive action.** Operations that can irreversibly destroy user data, change disk layout, wipe keys, remove recovery paths, or expose sensitive data require clear scope and explicit consent, unless a documented emergency procedure applies and is auditable.

**INV-06 — No hidden telemetry.** Network reporting of usage or diagnostics is disabled by default, visible, purpose-limited, and revocable. Essential protocol traffic must be documented separately from telemetry.

**INV-07 — Local autonomy.** A node can enforce its cached valid local policy when disconnected from remote services. Remote unavailability must not cause a remote principal to gain additional power.

**INV-08 — Isolation by construction.** Applications, drivers, imported environments, and AI components start without access to unrelated data, devices, credentials, or network destinations.

**INV-09 — Separation of duties.** No single account, signing key, service, or person should be able to author code, approve it, build it, and publish a trusted release alone.

**INV-10 — Verifiable updates.** An update must have authenticated provenance, a signed manifest, freshness/replay protection, and a defined recovery path before activation.

**INV-11 — Rollback is not erasure.** Restoring an earlier system generation must not silently delete audit evidence of a failed update, suspected compromise, or privileged action.

**INV-12 — Data and configuration are distinct.** System rollback MUST NOT automatically roll back personal data, database transactions, or external effects unless the operation explicitly defines and validates that behavior.

**INV-13 — Explainability is evidence-based.** The system must explain actual inputs, policy decisions, observable signals, planned effects, and execution outcomes. An AI-generated narrative is not proof of correctness.

**INV-14 — Offline core operation.** Boot, local login, local file access, policy enforcement, and recovery must not depend on a remote AI service or vendor cloud.

**INV-15 — User agency.** Users can inspect grants, revoke app access, export their data, disable AI assistance, and choose supported installation modes without being punished by hidden functionality loss.

**INV-16 — Compatibility is honest.** Each compatibility claim is tied to a specific app/version, ABI/API profile, hardware profile, feature set, and test record.

**INV-17 — Hardware diversity is explicit.** Security claims must state which hardware features are required, optional, absent, or emulated. Software MUST NOT pretend that a missing hardware isolation property exists.

**INV-18 — Failure containment.** A failing or compromised non-kernel service should not automatically compromise unrelated services or the kernel. The expected containment boundary must be stated per component.

**INV-19 — No secret master backdoor.** The project MUST NOT ship undocumented universal remote access, hidden credentials, secret support accounts, or a private mechanism that bypasses local policy.

**INV-20 — Publicly accountable changes.** Security-sensitive changes require a recorded rationale, review, testing evidence, and a versioned decision record. Emergency changes are time-limited and reviewed after deployment.

**INV-21 — Cryptographic conservatism.** Senvoryn MUST NOT invent cryptographic primitives. It MUST define algorithm agility, key rotation, revocation, and migration before a primitive becomes unchangeable in persisted data.

**INV-22 — Secure degradation.** When resources, trust services, policy sources, or critical dependencies fail, the system enters a documented restricted mode rather than granting broader authority to keep working.

**INV-23 — Reversibility where possible.** Changes should be staged, previewed, and reversible. Irreversible operations must be explicitly identified, with recovery options presented before execution.

**INV-24 — No false assurance.** Formal proof, test coverage, signed code, reproducible builds, and audits each cover only stated properties. None alone proves that the entire system has no vulnerabilities or backdoors.

**INV-25 — No arbitrary vendor lock-in.** Users can export data in documented formats, replace optional components, use offline recovery, and operate supported local functions without an account with the project maintainers.

**INV-26 — Accessibility is core.** Core flows must support keyboard-only use, assistive technology, readable language, sufficient contrast, and reduced-motion preferences where the interface supports motion.

**INV-27 — No covert persistence.** A removed application must not leave active background services, scheduled actions, credentials, or network listeners unless the user explicitly keeps an identified shared component.

**INV-28 — Safe defaults survive customization.** Custom themes, AI plugins, third-party app stores, and policy bundles cannot weaken invariant enforcement.

**INV-29 — Audit minimization.** Audit records contain enough evidence to account for important actions, but MUST avoid storing secrets or full private content unless strictly necessary and explicitly authorized.

**INV-30 — Measured claims only.** “Fast,” “lightweight,” “secure,” “private,” “compatible,” and “self-healing” claims must link to methodology, scope, and recent evidence.

## Article 3 — The trust model

### 3.1 Protected assets

The threat model MUST explicitly cover: user content; authentication credentials; encryption keys; device identity; capability state; system policy; application data; update-signing authority; boot integrity; audit evidence; network identity; model prompts and outputs; AI memory; firmware configuration; and availability of essential services.

### 3.2 Adversaries

The system design must consider at least:

- A malicious or compromised user application.
- A hostile document, web page, model prompt, archive, package, or plugin.
- A compromised compatibility guest or foreign runtime.
- A malicious or compromised driver.
- A remote attacker on an untrusted network.
- A malicious local user without administrator authority.
- An administrator who makes mistakes or abuses granted authority.
- A compromised update mirror, build worker, or package publisher.
- A stolen device, lost credential, or lost network connection.
- A malicious or compromised AI model/provider.
- A node compromised within a multi-device fleet.
- A project insider attempting to bypass review or insert concealed functionality.
- Hardware or firmware that is defective, vulnerable, or not fully trustworthy.

### 3.3 Trust assumptions

The first release may assume that CPU privilege controls, memory protection, and boot firmware behave according to documented capabilities. Those assumptions MUST be listed per hardware profile. Where hardware roots of trust are absent, the system must downgrade the assurance level visibly. A trusted boot chain can detect some modifications; it cannot prove that firmware itself is benevolent.

### 3.4 Security claim format

Every public assurance claim MUST be written as:

`Claim -> Scope -> Assumptions -> Mechanism -> Evidence -> Known limitations -> Expiry/review date`.

Example: “Capsule A cannot read Capsule B's private memory” must state architecture profile, enabled hardware protection, kernel version, permitted shared-memory grants, test evidence, and known side-channel limitations.

## Article 4 — Originality, reuse, and technology policy

Senvoryn is intended to define its own kernel, native execution model, service contracts, capability model, system-control protocol, package semantics, installation flow, user experience, and adaptive management architecture. It MUST NOT simply bundle an existing OS and call the result a new foundation.

Originality does not justify repeating solved security mistakes. Senvoryn may implement its own architecture and code while using transparent tools, reviewed compilers, standard file formats, standard wire encodings, and publicly scrutinized cryptographic algorithms. It may consume external development tools and carefully approved libraries, subject to the license policy and supply-chain requirements.

The project MUST distinguish:

- **Original design:** a Senvoryn-defined contract, component, algorithmic workflow, or architecture.
- **Independent implementation:** code written for Senvoryn to implement a defined contract.
- **Approved dependency:** external code or a tool with recorded provenance, license, version, review, and justification.
- **Compatibility guest:** a separately licensed or supplied environment used to run third-party software.
- **Research prototype:** unproven work not included in the trusted production base.

No borrowed source code may be relabeled as original. No dependency may enter a release merely because a build script fetched it.

## Article 5 — Open source and licensing constitution

### 5.1 Default license

Unless an approved repository-level notice states otherwise, original Senvoryn source code is intended to be released under Apache License 2.0. Original documentation may be Apache-2.0 or another expressly stated documentation license approved in the project license policy. Every file and release must identify its license clearly.

### 5.2 Component license inventory

Every release MUST include a machine-readable Software Bill of Materials and license inventory with component name, version, source, content hash, license expression, copyright notices, modifications, build provenance, and redistributability status. Unknown or ambiguous licensing is a release blocker for the trusted core.

### 5.3 Strict-core rule

If the project elects to require Apache-2.0 for every distributed component, that is a separate strict profile. Under that profile, any component with a different license is excluded unless counsel confirms an applicable permission or the component is replaced by an independently written implementation. The project must not claim that all dependencies are Apache-2.0 when only its own code is.

### 5.4 External cryptography and build tools

Senvoryn MUST use publicly reviewed cryptographic designs. Implementations may be approved external dependencies when review, provenance, license, maintenance, and vulnerability response are documented. A desire for a fully original codebase MUST NOT lead to home-grown cryptography. A strict independent implementation may be explored, but it cannot be the sole production implementation until it receives independent expert review and has a migration plan.

### 5.5 Trademarks and forkability

The code license and project name are separate. The project should permit forks under the applicable code license while protecting the official release identity through a transparent trademark policy. The policy must not prevent security researchers from truthfully identifying modified builds.

## Article 6 — Governance and anti-capture

### 6.1 Public governance

Design proposals, approved specifications, security advisories, release notes, governance votes, and architecture decision records SHOULD be public. Sensitive vulnerability details may be temporarily embargoed through a disclosed process with an expiry and follow-up publication.

### 6.2 Separation of duties

No single person or organization should simultaneously have unilateral authority to merge protected kernel code, alter release workflow, access all signing keys, produce official artifacts, and publish a trusted release. Multi-party approval and independent release verification are mandatory for stable releases.

### 6.3 Signing authority

Release signing keys MUST be separated by role: source/repository administration, build attestation, release authorization, and emergency revocation. Online systems may hold narrowly scoped short-lived keys, not the offline root signing key. Root-key use requires multi-person authorization, documented ceremony, and an audit record.

### 6.4 Changes to the constitution

A constitutional amendment requires:

1. A public proposal explaining the problem and affected invariants.
2. Threat-model and compatibility analysis.
3. A minimum review period of 30 days for ordinary changes.
4. Review from maintainers and independent reviewers for security-sensitive changes.
5. A supermajority of at least 75% of participating eligible votes and approval by at least two independent stewardship groups once those groups exist.
6. A migration plan, test plan, and versioned effective date.

Emergency changes may temporarily reduce scope or disable a feature to protect users. They MUST NOT weaken an invariant permanently. Emergency policy expires after 30 days unless renewed through ordinary review; all emergency uses receive a public post-incident review, subject to responsible disclosure.

### 6.5 Conflict of interest and stewardship

Reviewers must disclose material conflicts. Security-critical code must have at least two qualified reviewers, one of whom did not author the change. The project should maintain a documented succession plan so a single person leaving cannot make releases impossible or give another person unchecked control.

---

# PART II — SYSTEM ARCHITECTURE

## Article 7 — Architectural layers and trust boundaries

Senvoryn is organized into logical layers. Implementations can map these layers to separate processes, protection domains, or hardware partitions, but must preserve their defined authority boundaries.

| Layer | Name | Primary responsibility | Authority boundary |
|---|---|---|---|
| L0 | Hardware Adaptation Boundary (HAB) | CPU, memory, interrupts, timers, boot storage and device discovery | Exposes only detected and validated hardware capabilities |
| L1 | Senvoryn Separation Core (SSC) | Address-space isolation, scheduling, IPC, interrupt routing, capability primitives | Smallest software authority base; no policy decisions based on AI output |
| L2 | Capability and Identity Authority (CIA) | Capability object lifecycle, identity, grants, attenuation and revocation | May issue only policy-authorized and scoped capabilities |
| L3 | System Control Plane (SCP) | Policy evaluation, state transitions, system configuration and action authorization | Cannot override kernel-enforced invariants |
| L4 | Origin Capsule Runtime (OCR) | Creates and supervises isolated app/service environments | Each capsule receives only declared resources |
| L5 | Universal Execution Fabric (UEF) | Native runtime, translation, guest environments and compatibility APIs | Guest permissions never become host permissions |
| L6 | Local Intelligence Fabric (LIF) | Model inference, intent interpretation, prediction and proposals | No direct privileged interface; all effects require SCP authorization |
| L7 | User Experience Shell (UXS) | Installer, app launch, permission dialogs, settings, accessibility | Must faithfully display the actual policy decision and risk |
| L8 | Federation and Continuity Layer (FCL) | Cross-device identity, state synchronization and fleet policy | Local node can reject remote commands and remain safe offline |
| L9 | Developer and Assurance Toolchain (DAT) | Builds, package verification, testing, provenance and auditing | Build workers cannot unilaterally publish trusted releases |

### 7.1 Minimal trusted core

The Separation Core MUST NOT contain a general-purpose shell, AI inference engine, web renderer, package downloader, application UI toolkit, or complex filesystem parser. The core should implement only the primitives needed to protect address spaces, schedule execution, deliver bounded messages, route interrupts, enforce capability invocation, and safely start the next authority layer.

### 7.2 Service failure isolation

Filesystem, network, device drivers, app lifecycle, model inference, compatibility bridges, and presentation are intended to run outside the highest-trust core whenever the target hardware permits. A fault in a service must be recoverable without rebooting unrelated services where feasible. If hardware or early-boot constraints require a function inside the core, the exception must be documented and reviewed.

### 7.3 No implicit trust by layer

A component's layer does not make its inputs trustworthy. Every layer validates inputs from less-trusted layers, and every externally reachable parser is treated as attack surface. The SCP is not trusted to execute arbitrary code simply because it is called a control plane; its policy evaluator and transition engine require their own tests and limited interfaces.

## Article 8 — Senvoryn Separation Core (SSC)

### 8.1 Kernel responsibilities

The initial core MUST define mechanisms for:

- Creating and destroying protected execution contexts.
- Creating, switching, and destroying address spaces.
- Mapping and unmapping memory with explicit rights.
- Scheduling runnable threads subject to declared constraints.
- Delivering interrupts and timers to authorized service endpoints.
- Establishing message channels and bounded shared-memory grants.
- Invoking unforgeable kernel object handles.
- Enforcing isolation and resource ceilings.
- Revoke or invalidate capabilities using generation/epoch mechanisms.
- Starting a minimal recovery path if the normal supervisor fails.

### 8.2 Kernel exclusions

The kernel MUST NOT parse arbitrary network protocols, user documents, app package archives, model prompts, rich text, or untrusted configuration languages. It MUST NOT download code, call remote AI endpoints, maintain user analytics, or contain app-store business logic.

### 8.3 Kernel object model

The core should expose a small typed object model:

- `Task`: a security and resource container for a service or capsule.
- `Thread`: schedulable execution unit associated with one Task.
- `AddressSpace`: protected virtual memory mapping.
- `MemoryRegion`: a bounded physical/virtual memory resource.
- `Channel`: a directional or bidirectional IPC endpoint.
- `Endpoint`: a receiver identity with bounded queue semantics.
- `CapabilityHandle`: an unforgeable reference to a kernel object and allowed operations.
- `InterruptRoute`: authorization for a specific interrupt source and handler endpoint.
- `Timer`: bounded timing resource.
- `DeviceRegion`: authorized access to a device's registers, queues, or DMA mapping.
- `Quota`: an enforceable resource envelope.
- `Namespace`: a scoped naming view, not itself an authority grant.

Names can change during implementation, but object semantics and authority boundaries must remain documented.

### 8.4 IPC contract

Every IPC message must have a bounded header containing protocol identifier, version, message type, payload length, correlation ID, sender task identity, deadline/timeout where applicable, and attached capability descriptors. Receivers MUST validate message size and version before parsing payload content. Unknown required message types are rejected. No IPC mechanism may silently grant a capability merely because a sender can address a receiver.

Messages larger than the configured inline limit use a separately granted shared-memory region. Both mapping rights and lifetime must be explicit. Receivers cannot retain access after a region grant expires or is revoked, subject to clearly documented hardware limitations.

### 8.5 Scheduling classes

The scheduler must support, at minimum:

- **Critical:** kernel-supporting and essential control tasks with strict budget limits.
- **Interactive:** UI/input tasks optimized for responsive latency.
- **Realtime-constrained:** tasks with explicit deadline and admission requirements, only when the hardware profile can honor them.
- **Normal:** ordinary applications and services.
- **Background:** maintenance, indexing, model preparation and other deferrable work.
- **Best-effort/idle:** work that yields to higher-priority service needs.

Priority never means unlimited CPU. Each class is subject to per-task and system-wide budgets. Admission control must reject or downgrade a claimed deadline when the CPU, memory, interrupt, or I/O assumptions cannot support it.

### 8.6 Resource exhaustion behavior

The core must cap message queues, capability tables, thread counts, address-space mappings, timers, and other allocatable kernel objects per Task and globally. Exhaustion returns a defined error and MUST NOT corrupt core state. Critical internal reserves must be protected from user-task exhaustion.

### 8.7 Core acceptance criteria

- A task cannot map memory without an authorized mapping capability.
- A thread cannot execute in an address space other than its assigned one.
- A stale or revoked capability handle fails closed.
- Untrusted task loops cannot starve the configured control reserve under a passing profile workload.
- Malformed IPC lengths and unknown object types are rejected without memory corruption.
- A non-core service crash does not corrupt unrelated task state in fault-injection tests.

## Article 9 — Hardware Adaptation Boundary (HAB)

### 9.1 Purpose

HAB is the only layer that translates hardware-specific behavior into the system's portable machine model. It reports what the device can do, not what the operating system wishes were available.

### 9.2 Capability discovery

Hardware discovery produces a signed or integrity-protected **Hardware Capability Record (HCR)** where the platform supports trustworthy measurement. The record includes CPU architecture and feature bits, page-size modes, memory map, interrupt controller, timer sources, IOMMU/DMA protection, virtualization extensions, secure boot state, hardware key storage, storage devices, display/input devices, network devices, power/thermal sensors, and firmware version.

HCR values must distinguish `present`, `available`, `enabled`, `verified`, `unsupported`, and `unknown`. `unknown` must never be treated as `verified`.

### 9.3 Driver boundary

Drivers run with only the memory ranges, DMA domains, interrupts, and device queues required for their function. A driver MUST NOT receive general physical-memory access. Resetting or restarting a driver must not silently broaden its privileges. Driver update and rollback use the same signed artifact and staged-activation rules as other system components.

### 9.4 Hardware profiles

Every release lists certified profiles, such as:

- **E0 constrained edge:** limited RAM/CPU, headless or minimal UI, no required local large model.
- **E1 capable edge:** small local models and multiple isolated services when resources permit.
- **P1 personal device:** display/input, local applications, encryption and suspend/resume.
- **W1 workstation:** high throughput, GPU/NPU support, media and developer workloads.
- **S1 server:** headless services, storage, network, out-of-band recovery where available.
- **C1 cluster node:** authenticated federation, resource accounting and remote but locally constrained administration.

Profiles are conformance categories, not promises that every device in a marketing category works. Each profile must name exact tested hardware/firmware combinations.

## Article 10 — Capability and Identity Authority (CIA)

### 10.1 Identity principals

A principal may be a local user, service, device, app capsule, driver capsule, installer, remote peer, signed update publisher, or AI workload. Principal identity is distinct from authority: knowing who a component is does not automatically give it access.

### 10.2 Capability semantics

A capability is an unforgeable reference to an object or action plus a bounded set of rights and constraints. It must not be represented solely as a guessable string or a user-editable file. A user-space manifest may request capabilities, but the kernel/CIA creates the actual capability only after policy approval.

Each capability record or derivation must bind at least:

- `subject`: receiving principal identity and generation.
- `object`: target object or resource class.
- `rights`: exact allowed operations, not a generic `admin` flag unless such a role is itself narrowly defined.
- `scope`: object path, resource set, destination, or bounded namespace.
- `purpose`: declared operation class.
- `constraints`: limits for volume, rate, time, data label, resource, and context.
- `delegation`: whether delegation is permitted and the maximum attenuated scope.
- `issued_at` and optional `expires_at` or usage limit.
- `policy_epoch`: policy generation under which the grant was authorized.
- `issuer`: authenticated policy authority.
- `revocation_domain`: mechanism/epoch through which the grant can be invalidated.
- `audit_class`: minimum event record required on use.

A capability may be attenuated but not widened by its holder. A copied handle must not allow another subject to impersonate the original holder. Capability transfer must be an explicit operation with a defined receiver and rights.

### 10.3 Rights granularity

Rights SHOULD be decomposed into actions such as `read_metadata`, `read_content`, `write_new`, `modify_existing`, `delete`, `execute`, `share`, `network_connect`, `network_listen`, `device_enumerate`, `device_use`, `key_derive`, `key_use`, `policy_propose`, `policy_apply`, `update_stage`, `update_commit`, and `recovery_enter`. The term `full access` should be avoided in user interfaces unless a concrete list of effects is shown.

### 10.4 Authorization algorithm

Every protected operation must follow the equivalent of this sequence:

1. Authenticate the calling task using a kernel-bound task identity.
2. Resolve the target object through an authorized handle; never authorize by an untrusted path string alone.
3. Verify the capability is valid, not expired, not revoked, and belongs to the caller or an explicitly delegated receiver.
4. Verify requested right is a subset of granted rights.
5. Verify object scope, purpose, data label, resource budget, rate/volume limit, and time window.
6. Verify the policy epoch is accepted and no applicable deny rule overrides the grant.
7. Check current system state (normal, degraded, quarantined, recovery, etc.) permits the operation.
8. For high-impact operations, require a valid Change Capsule and the necessary user/admin approvals.
9. Record the defined audit event before or atomically with the effect, without leaking secrets.
10. Execute within a bounded time/resource envelope and report a structured result.

If any step cannot be completed, the result is **deny or safe defer**, not implicit allow. The implementation must avoid time-of-check/time-of-use gaps by binding authorization to the object generation and operation transaction.

### 10.5 Revocation

Revocation semantics must define what happens to in-flight operations. For immediate revocation, future invocations fail and revocable mappings/channels are removed. Operations that cannot be safely interrupted must either finish within a documented bound or be canceled at a transaction boundary. A stale cache cannot override a higher policy epoch or revocation tombstone.

### 10.6 No ambient authority

Environment variables, current directory, process names, executable paths, model context, GUI focus, or being a child of an administrator process do not confer security authority. Authority comes from authenticated principals and explicit capabilities.

## Article 11 — System Control Plane (SCP)

### 11.1 Role

The SCP evaluates policy, manages named system states, approves bounded changes, coordinates local services, and exposes administrative interfaces. It is not an unrestricted root daemon. Its components must have separate privileges and bounded APIs.

### 11.2 SCP subcomponents

- **Policy Registry:** versioned declarative policies and policy metadata.
- **Policy Evaluator:** deterministic decision engine with no model inference.
- **Authority Broker:** requests or attenuates capabilities through CIA.
- **State Coordinator:** runs the state machine and validates transitions.
- **Change Executor:** applies approved Change Capsules only.
- **Evidence Ledger:** append-only records for policy decisions and privileged effects, with privacy minimization.
- **Health Monitor:** accepts authenticated, typed health signals and marks their trust level.
- **Local Admin Interface:** exposes local management functions, protected by interactive authentication.
- **Federation Gateway:** receives remote requests but does not bypass local policy.
- **Recovery Coordinator:** activates recovery plan after failure or administrator request.

These may be multiple services. Combining them into one process for an early prototype is allowed only if the authority interfaces remain logically separated and the security case documents the temporary coupling.

### 11.3 Policy precedence

The evaluation order is:

1. Non-overridable constitutional invariants.
2. Hardware and boot-integrity restrictions.
3. Security-domain and system-state restrictions.
4. Owner-level policy.
5. Authenticated local administrator policy within the owner's grant.
6. User-level preferences within their authority.
7. Application request and declared intent.

A higher level may restrict lower-level permissions; lower levels cannot widen a higher-level prohibition. Explicit deny wins over an allow of equal or lower authority. If two equally authoritative rules conflict and no deterministic resolution is declared, the result is deny and the conflict is recorded.

### 11.4 Remote administration

A remote command is a request, not a commandment. The node validates peer identity, request signature, freshness, target identity, requested capability, local policy, current state, and authorization scope. A cluster manager cannot mint unrestricted local capabilities. Remote commands must be time-bounded and scoped to the selected node/resource group.

### 11.5 SCP protocol envelope

SCP messages must include protocol name/version, message type, sender principal, intended recipient, target node/object, monotonic request identifier, nonce or anti-replay data, creation/expiry time, policy epoch, requested scope, payload hash, signature/MAC where applicable, and response correlation ID. Sensitive payloads are encrypted in transit. Unknown message versions are rejected or routed through an explicitly defined compatibility path.

### 11.6 SCP availability

The SCP must have a minimal local policy path. If the high-level coordinator crashes, the kernel and existing capsule permissions continue to enforce already valid authority. New high-risk grants and irreversible changes are denied until policy control is restored. Safe read-only operations may continue according to an explicit offline policy.

## Article 12 — Capability Graph and Intent Transaction Compiler

### 12.1 Capability Graph

Senvoryn maintains a typed, queryable graph of principals, resources, capabilities, purposes, data labels, dependencies, and trust boundaries. The graph is a policy and observability representation; it is not itself the enforcement mechanism. Enforcement remains in kernel/CIA and SCP checks.

The graph answers: who can access this object; which apps can read this data; which node can administer this resource; what will be affected by revoking this grant; what depends on a service; and which path caused a privileged effect. Graph queries must respect the caller's own access rights and must not reveal hidden resource names or secrets.

### 12.2 Intent-first interaction

The user may request an outcome such as “install this app with access only to these documents” or “reduce background resource use.” Senvoryn converts it into a structured **Intent Record** containing:

- authenticated requester and active session;
- plain-language request and a normalized description;
- target objects and requested outcome;
- known assumptions and ambiguity flags;
- expected actions and affected resources;
- requested capabilities;
- risk classification and reversibility classification;
- data and network impact;
- plan expiry and user-approval requirements.

The normalized description must be shown when the system detects ambiguity or high impact. It must never silently infer destructive intent from ambiguous wording.

### 12.3 Intent Transaction Compiler (ITC)

The ITC turns an Intent Record into a candidate plan. It must produce a machine-readable effect list, preconditions, permissions required, resources expected to change, validation checks, recovery/compensation steps, and user-visible summary. It must not execute privileged effects during planning.

A deterministic validator compares the candidate plan against policy and current system state. If a model proposes an unrecognized effect, overbroad permission, or undeclared network destination, the validator rejects it. A model-generated confidence score is not an authorization signal.

### 12.4 Shadow-state simulation

Before a high-impact change, the ITC may run the plan against a **Shadow State**: a read-only snapshot or model of relevant configuration, dependency relations, storage constraints, and service-health state. Simulation is advisory; it cannot prove future behavior. The user interface must label predicted outcomes and uncertainties. The actual executor rechecks preconditions immediately before commit.

### 12.5 Intent safety classifications

- **I0 — Read-only:** inspect state without disclosing protected data beyond the caller's rights.
- **I1 — Reversible low-impact:** a bounded, easily reversible preference or resource adjustment.
- **I2 — Persistent but recoverable:** software install/update, service configuration, or file operations with defined rollback.
- **I3 — Sensitive/high-impact:** access to personal content, external disclosure, privilege changes, network exposure, identity changes, or key use.
- **I4 — Destructive/irreversible:** disk replacement, data wipe, key destruction, permanent account deletion, or action that cannot reliably be reversed.

I0/I1 actions may be automatically executed only under explicit user policy and pre-approved limits. I2 requires a valid plan and an approval rule. I3 requires a clear impact explanation and explicit consent unless an owner-authored policy specifically authorizes a narrow recurring operation. I4 requires explicit confirmation immediately before execution, exact target display, and recovery consequences. There is no AI exception.

## Article 13 — Change Capsule protocol

### 13.1 Definition

A Change Capsule (CC) is a signed, immutable description of a proposed state transition. It is not an executable authority token. It contains a plan and evidence; the SCP still authorizes execution and CIA issues any short-lived capabilities required.

### 13.2 Required fields

Every CC must declare:

- unique ID and schema version;
- originator and authenticated principal;
- target device, service, capsule, file set, or resource set;
- intent record reference;
- affected components and blast-radius estimate;
- preconditions and required policy epoch;
- artifacts and content hashes;
- requested capabilities and duration;
- ordered actions with idempotency keys;
- expected effects and postconditions;
- resource/time budget;
- risk/reversibility classification;
- data-label and network-disclosure impact;
- approvals and their scope;
- health checks and failure thresholds;
- rollback or compensation plan;
- evidence to retain and audit class;
- expiry and replay protections;
- signatures and provenance chain.

### 13.3 Lifecycle

`DRAFT -> VALIDATED -> AUTHORIZED -> STAGED -> PRECHECKED -> APPLYING -> VERIFIED -> COMMITTED`.

Failure may transition to `REJECTED`, `EXPIRED`, `ABORTED`, `COMPENSATING`, `ROLLED_BACK`, or `NEEDS_HUMAN_RECOVERY`. Each transition records an event. A CC cannot jump directly from draft to applied. A failed precondition returns the capsule to validation, not to execution.

### 13.4 Atomicity and idempotency

The executor must know which steps are safe to retry. Each effect with external consequences must have an idempotency key or a reconciliation procedure. The system must not claim atomicity across a remote service or physical side effect that cannot be atomically controlled. Such steps are described as **saga/compensation operations**, with partial completion visible to the user.

### 13.5 Commit condition

Commit is permitted only when all mandatory postconditions pass, audit records are durable at the defined assurance level, and the state coordinator confirms that the observed policy epoch still permits the change. If a postcondition cannot be measured, it cannot be represented as verified.

### 13.6 Revocation during execution

Revocation blocks new steps. In-progress steps stop at the next safe boundary unless stopping immediately would create a greater defined hazard. The capsule enters a documented abort/compensation state. No actor may erase the execution record to hide the partial result.

## Article 14 — Origin Capsules and application isolation

### 14.1 Default model

Every third-party application, plugin, driver service, model, imported runtime, and compatibility guest runs inside an Origin Capsule (OC) unless it is part of the minimal boot chain and has an approved exception. A capsule is not merely a directory or UI profile; it is a policy-bounded execution domain.

### 14.2 Capsule profile

Each OC has:

- immutable identity and origin;
- executable/artifact hashes and signer/provenance record;
- runtime/ABI/architecture profile;
- dedicated filesystem view and persistent-data boundary;
- capability requests and granted capabilities;
- CPU, memory, GPU, storage, process, I/O, and network limits;
- data labels it may read/write;
- IPC endpoints and allowed peers;
- device permissions;
- secret access policy;
- lifecycle hooks and health contract;
- update channel and rollback generation;
- audit level and retention policy;
- compatibility tier and tested feature list.

### 14.3 First launch

On first launch, the system compares declared and observed behavior. A capability prompt must use concrete actions such as “read files you select,” not vague labels like “access your data.” The user can deny, allow once, allow for a specified time, or grant a scoped persistent capability where supported. The OS must not bundle unrelated permissions into one approval.

### 14.4 Storage and process isolation

Capsules do not share writable storage by default. Sharing occurs through brokered objects or explicit shared spaces with named read/write semantics. The host must separate executable content, app-private data, cached data, imported content, and secrets. Removing a capsule revokes its capabilities, stops its services, and offers a data-retention choice with clear consequences.

### 14.5 Network isolation

A capsule with no network capability has no routable network path. Network grants can restrict protocol, destination, port, direction, DNS policy, rate, byte count, duration, and proxy route. Network naming and DNS lookups are also information disclosures and must be accounted for. Loopback and local-network access must not be silently included in “internet access.”

### 14.6 Shared-resource rules

A shared GPU, clipboard, camera, microphone, display, accessibility tree, clipboard history, printer, or USB device is a controlled bridge. The bridge policy must specify allowed operations, visible user indicators where appropriate, lifetime, and revocation behavior. Clipboard reads should be initiated by user action unless a narrow accessibility rule explicitly permits otherwise.

### 14.7 Capsule failure

A capsule crash does not automatically trigger host restart. The supervisor applies a configured retry budget, captures bounded diagnostics, avoids restart storms, and may quarantine the capsule. A crash loop must not starve essential system services.

## Article 15 — Universal Execution Fabric (UEF)

### 15.1 Objective

UEF provides one user-facing system for installing, launching, isolating, observing, and removing software from different ecosystems. It does not pretend all ecosystems share the same ABI, security model, hardware access, legal terms, or application lifecycle. The system selects a compatibility mechanism per app and exposes its limitations.

### 15.2 Execution modes

- **UEF-Native:** software built for the Senvoryn ABI and native service contracts.
- **UEF-Portable:** software using a documented portable runtime with no direct dependence on a foreign OS kernel.
- **UEF-API:** API/ABI translation for a defined subset of a foreign operating-system interface.
- **UEF-ISA:** instruction-set translation/emulation for a different CPU architecture, optionally combined with API translation.
- **UEF-Guest:** full or near-full guest OS environment under strong hardware or software virtualization, where legally obtained and supported.
- **UEF-Remote:** optional execution on another trusted device/service with explicit remote-processing disclosure and data-transfer consent.
- **UEF-Unavailable:** software cannot currently be run within the target's declared requirements, licensing, or security limits.

### 15.3 Compatibility levels

Each app release is classified for a target hardware profile:

- **C0 Native:** built and tested for Senvoryn; expected features are native.
- **C1 High compatibility:** runs through a maintained translation/runtime layer; all mandatory app workflows pass the published suite.
- **C2 Guest-dependent:** requires an authorized guest environment or original OS installation media/license; resource overhead and limits are disclosed.
- **C3 Partial:** only an enumerated subset of features works; unavailable features are shown before install where known.
- **C4 Remote:** runs remotely; the device must disclose which data leaves it and which device performs the work.
- **C5 Unsupported:** cannot be installed safely or lawfully in the current configuration.

No marketing statement may collapse C1–C4 into “fully native.” Every compatibility entry includes app identity/version, architecture, required APIs, tested features, performance observations, known faults, permissions, licensing conditions, and last test date.

### 15.4 Windows software

The project may implement API compatibility layers and/or isolated guest execution where technically and legally available. Applications that depend on kernel drivers, anti-cheat systems, hardware-bound licensing, DRM, undocumented services, specialized devices, or vendor attestation may not work. Such limitations must be identified, not bypassed secretly. Full guest execution may require a user-supplied or separately licensed operating system.

### 15.5 Linux software

UEF must support a documented compatibility profile for software using selected Linux user-space conventions. Compatibility does not imply that every Linux kernel module, device driver, privileged service, or distribution-specific assumption can run directly. Kernel-dependent workloads may require an isolated guest environment.

### 15.6 Android software

Android application support may require a compatible guest system, API implementation, graphics/audio/input bridge, package verifier, and service implementation. Applications relying on unavailable proprietary service layers, hardware-backed attestation, restricted APIs, carrier-specific hardware, or device integrity checks may be partially supported or unavailable. Permissions in the guest must map to explicit Senvoryn host capabilities; guest grants cannot silently authorize host access.

### 15.7 iOS software

Senvoryn must not promise unrestricted execution of every iOS application. Execution can depend on proprietary operating-system components, Apple-specific hardware, code-signing, platform services, entitlement grants, store rules, licensing, DRM, or device attestation. UEF may support authorized application formats, developer-provided portable builds, or a compliant guest/remote mode when available. It MUST NOT claim that arbitrary iOS binaries can be run with full features on all hardware, and MUST NOT defeat platform security or licensing restrictions as a compatibility technique.

### 15.8 Compatibility adapters as hostile boundaries

Each adapter is isolated, versioned, fuzzed, and granted no broad host privilege. It must declare which host APIs it can invoke. Guest `root` or administrator concepts map only to authority inside that guest. Host data access requires a separate host-issued capability. A compatibility adapter cannot mint host capabilities or modify SCP policies.

### 15.9 Compatibility research gates

The first compatibility milestone is one documented guest/ABI profile with a repeatable test suite—not claims about every ecosystem. Each additional ecosystem requires a threat model, legal review, API coverage matrix, app test corpus, isolation tests, performance measurements, and a rollback plan.

## Article 16 — AI-native Local Intelligence Fabric (LIF)

### 16.1 AI as a first-class but optional service

AI is integrated into system search, user assistance, diagnostics, intent parsing, workload forecasting, accessibility, and plan generation. Core OS correctness cannot depend on a model being present, accurate, online, or uncorrupted. The system must remain functional with AI disabled or unavailable.

### 16.2 Model capsules

Every model, agent, tool bridge, retrieval index, and inference server runs with its own identity, resources, data permissions, and network policy. Model files are content-addressed and provenance-tracked. Third-party model weights are data inputs, not trusted executables, but still require integrity verification and license review.

### 16.3 Prompt and content taint

Documents, websites, emails, tool outputs, app contents, and retrieved material are untrusted data even if they contain instructions directed at an AI agent. The system must preserve origin labels and distinguish user-authored intent from quoted or imported instructions. A model cannot treat text inside an untrusted document as a grant of authority.

### 16.4 Tool-use contract

AI tool calls use typed, schema-validated functions. Each function declares required capabilities, allowed argument ranges, side effects, rate limits, timeouts, data labels, and whether it is reversible. Free-form shell execution with broad privileges is prohibited as the default AI interface. A developer may provide a constrained shell inside an isolated capsule, but its resources and capabilities remain bounded.

### 16.5 AI action pipeline

`OBSERVE -> CLASSIFY -> PROPOSE -> VALIDATE -> SIMULATE -> AUTHORIZE -> EXECUTE -> VERIFY -> EXPLAIN -> LEARN/EXPIRE`.

- **Observe:** collect only data permitted by policy.
- **Classify:** label the request, data, uncertainty, and risk.
- **Propose:** generate a candidate intent/Change Capsule.
- **Validate:** deterministic code checks schemas, permissions, bounds, provenance, and policy.
- **Simulate:** predict relevant effects when meaningful; label uncertainty.
- **Authorize:** require the approval appropriate to the impact class.
- **Execute:** issue short-lived, scoped capabilities and apply the plan.
- **Verify:** check actual postconditions, not model assertions.
- **Explain:** show the request, plan, scope, policy decision, outcome, and deviations.
- **Learn/expire:** retain only user-authorized memory, with deletion/export controls.

If any required stage fails, privileged execution stops. AI output cannot skip a stage.

### 16.6 Explanations and evidence

Senvoryn does not require logging a model's private internal reasoning. It must log a concise, inspectable basis for operational effects: relevant observations, policy rule identifiers, assumptions, planned effects, approvals, and measured outcome. Explanations must not expose secrets to an unauthorized user. A model's textual explanation is not a replacement for machine-verifiable evidence.

### 16.7 Local-first processing

Local inference is preferred when it can meet the task's resource and quality requirements. If an external inference service is proposed, the UI must show the destination, data categories to be sent, purpose, retention terms if known, and cost implications. The user may deny the external call. No secrets or sensitive data are sent to external AI by default.

### 16.8 Personal memory

AI memory is a user-owned, inspectable data set with categories, origin, retention, and deletion semantics. It is encrypted at rest according to profile, and never silently used to authorize privileged actions. Deleting an item must remove it from active retrieval and future summaries; the system must disclose any limits on deletion from backups or derived artifacts.

### 16.9 AI disablement

A user or administrator can disable some or all AI functions. When disabled, models stop executing, model-specific network access is revoked, and core OS operations remain available. AI disablement must not prevent access to ordinary settings, files, recovery, or the package manager.

## Article 17 — Adaptive resource ecology

### 17.1 Goal

Senvoryn's resource engine allocates compute, memory, storage, network, graphics, accelerator time, energy, and thermal headroom based on declared priorities and measured constraints. It should optimize utility without allowing a workload to define its own priority unilaterally.

### 17.2 Resource envelope

Each capsule has:

- hard maxima where enforceable;
- minimum guarantees only where admission control can provide them;
- preferred shares;
- burst limits and burst duration;
- storage quota and inode/object count budget;
- network throughput and packet/connection budget;
- GPU/NPU/accelerator budget or exclusive lease;
- energy and thermal sensitivity where sensors exist;
- pressure response (throttle, pause, checkpoint, reject, or terminate);
- priority class and owner-specified importance;
- restart/recovery budget.

A “soft preference” is not a guarantee. The user interface must distinguish the two.

### 17.3 Adaptation rules

The resource engine may predict load and recommend changes. It may automatically adjust only within owner-approved envelopes. Changes that alter isolation, data access, externally visible network exposure, or priority of safety-critical operations require explicit authorization. Prediction errors must be treated as normal; the engine must preserve hard limits even when its model is wrong.

### 17.4 Pressure response sequence

When a resource is under pressure: (1) measure and attribute; (2) stop nonessential admission; (3) throttle best-effort tasks; (4) pause/checkpoint deferrable tasks where supported; (5) reclaim clean caches; (6) alert if user action is needed; (7) terminate a task only according to declared policy and after preserving required evidence. The engine must not silently delete user content to reclaim space.

### 17.5 Fairness and starvation

Scheduling policies must prevent unbounded starvation for classes whose contract promises progress. Critical reservations must be bounded so they cannot consume all resources indefinitely. Per-user and per-capsule limits protect against fork storms, memory exhaustion, log floods, or GPU monopolization.

### 17.6 Energy adaptation

Where the hardware exposes trustworthy energy/thermal data, the engine can shift noncritical work to cooler or lower-power periods. It must not claim energy savings without measurement. A user can choose performance, balanced, quiet, or energy-saving profiles, each with visible trade-offs.

## Article 18 — Storage, state history, and the Reversibility Ledger

### 18.1 Separate state domains

Senvoryn storage distinguishes:

- **Immutable system image:** verified OS code and base configuration.
- **Mutable system state:** service databases, device settings, and operating state.
- **Application-private state:** data owned by an individual capsule.
- **User data:** documents, media, credentials, and user-managed content.
- **Secrets:** keys, tokens, recovery material, and sensitive credentials.
- **Audit/evidence state:** policy decisions, privileged actions, update records, and security events.
- **Caches:** regenerable data that may be deleted under explicit pressure policy.

A snapshot of one domain must not implicitly roll back all domains.

### 18.2 Reversibility Ledger

The Reversibility Ledger records important changes with their before/after references, actor, policy, reason, commit state, and recovery semantics. It is not an infinite copy of every user byte. Data retention is bounded by user choices, storage policy, and legal requirements. It uses content references/hashes where useful and encrypted snapshots where enabled.

### 18.3 Undo classes

- **R0:** trivially reversible in memory.
- **R1:** reversible by configuration/version switch.
- **R2:** reversible from a snapshot or retained prior content.
- **R3:** compensatable but not perfectly reversible (e.g., external message sent or remote service mutation).
- **R4:** irreversible or unverified recovery (e.g., cryptographic key destruction, secure wipe).

The UI must disclose the undo class for high-impact operations. R3/R4 must never be described as “fully undoable.”

### 18.4 Deletion semantics

Delete is not a single universal operation. The system distinguishes unlinking, moving to trash, deleting app-local data, cryptographic erasure, overwrite where supported, and external deletion requests. Secure erasure depends on storage medium, wear leveling, snapshots, backups, and encryption. The system must state the strongest applicable guarantee and limitations.

### 18.5 Audit retention

Audit evidence is append-oriented and tamper-evident where feasible. It must avoid collecting application contents by default. Local owners can define retention, export evidence, and inspect it. If an attacker compromises the device, local logs may be altered unless a protected hardware or remote witness exists; the system must label the assurance level accordingly.

## Article 19 — Privacy and information flow

### 19.1 Data labels

The default label set is:

- **PUBLIC:** intended for unrestricted disclosure.
- **PERSONAL:** user-associated data not intended for public disclosure.
- **SENSITIVE:** health, financial, identity, private communications, precise location, or similarly consequential data.
- **SECRET:** authentication secrets, cryptographic keys, recovery material, private tokens, and equivalent credentials.
- **SYSTEM-INTEGRITY:** policy, release keys, boot measurements, security evidence, and control-plane secrets.

Applications may define more restrictive sublabels. They may not downgrade a label without the authorized declassification path.

### 19.2 Label propagation

Data written or derived from labeled inputs inherits the most restrictive applicable label unless a reviewed declassification rule exists. Copy/paste, screenshots, exports, logs, crash dumps, AI prompts, clipboard, search indexes, and backups are all data-flow paths and must preserve labels or require a reviewed conversion.

### 19.3 Declassification

Declassification must identify data, destination, transformation, approving principal, purpose, and output label. Removing identifying fields does not automatically make data anonymous. The system should prefer local aggregation and minimization. High-risk declassification requires explicit user authorization.

### 19.4 Telemetry

The default telemetry mode is off. Necessary update checks may send minimum protocol data only when enabled by user policy and must be documented. Diagnostic bundles require preview/redaction and explicit send approval. The project MUST publish an endpoint and field inventory for any official service. No hidden device fingerprinting or covert usage analytics is permitted.

### 19.5 Network disclosure

For each external AI call, remote execution, sync operation, or diagnostics upload, show which categories of data will leave the device, the named destination, and the purpose. A blanket acceptance at installation does not constitute consent for every later category of disclosure.

### 19.6 Clipboard, screen, sensors, and accessibility

Clipboard, screen capture, camera, microphone, location, accessibility events, keystroke observation, and notification content are privileged data sources. Access must be scoped, observable, and revocable. Background sensing requires a clear user-facing indication unless the hardware/platform makes this impossible, in which case the limitation must be published.

## Article 20 — Identity, authentication, keys, and secrets

### 20.1 Separate identities

User identity, device identity, app identity, developer identity, signing authority, and remote peer identity are distinct. Possession of a device does not automatically reveal secrets. A remote administrator identity does not automatically permit local policy override.

### 20.2 Authentication

The system should support password/passphrase, hardware-backed authenticators, device-local biometrics where available, recovery codes, and multi-party recovery according to profile. Biometrics are a convenience signal, not a secret that can be changed like a password; high-risk operations may require an additional factor.

### 20.3 Key classes

At minimum, different keys must be used for:

- device identity;
- user data encryption;
- capsule data encryption;
- artifact signing;
- release-root signing;
- local audit integrity;
- federation transport;
- backup/recovery;
- AI/service identity.

Cross-purpose key reuse is prohibited unless a reviewed protocol explicitly derives separated subkeys with domain separation.

### 20.4 Key custody

Long-term root release keys should remain offline or in a suitably protected signing ceremony. Build workers must not access the root key. Recovery keys must be stored separately from the device being recovered. Recovery procedures must be tested, and the system must warn that losing all copies of an encryption key can make encrypted data irrecoverable.

### 20.5 Rotation and revocation

Every key class has an owner, usage scope, expiry policy, rotation procedure, compromise procedure, and compatible revocation path. Update verification must support a planned algorithm/key migration so that one old key does not permanently control future updates.

## Article 21 — Network and inter-device fabric

### 21.1 Default network posture

New capsules have no inbound listening service and no general network permission by default. Outbound access is scoped. A system service listening on a port must declare its purpose, bind scope, authentication, rate limits, and owner visibility.

### 21.2 Service identity

Inter-service communication uses authenticated service identities and explicit endpoint capabilities. Network location/IP address is not proof of identity. The system must resist replay through freshness counters/nonces and must bind requests to the intended recipient and operation.

### 21.3 Local network privacy

Discovery protocols can disclose presence and device names. Device discovery is disabled or restricted by default on untrusted networks. Users can create trust groups, but membership in a group does not grant blanket access to all devices or data.

### 21.4 Cross-device continuity

The Continuity Layer enables a user to move a task, file view, or app state between authorized devices. It must distinguish portable state from hardware-bound secrets and ephemeral session tokens. Transfer uses an explicit policy, end-to-end encryption, destination attestation when available, and visible conflict resolution. If no trusted destination can be established, the system refuses sensitive state transfer.

### 21.5 Federation outage

If the cluster coordinator or network is unavailable, nodes continue under the last valid local policy until its expiry. Expiry must be chosen to balance safety and availability per policy. No stale coordinator can revive a revoked credential after its revocation has been observed. Nodes must expose whether they are operating with current, cached, or expired policy.

## Article 22 — Universal installation and Direct Install

### 22.1 Product requirement

On certified existing host systems, a user should start the signed Senvoryn installer like an application and complete a guided installation without creating a USB or CD/DVD boot medium. The installer may require a reboot. “One click” means a simple start and largely automated safe flow, not silent destructive installation.

### 22.2 Installation modes

- **Try mode:** launch a temporary or virtualized environment without modifying the host disk beyond explicitly disclosed temporary storage.
- **Virtual install:** install in a user-approved virtual machine.
- **Side-by-side install:** preserve the existing OS and create a separate bootable installation, where boot and storage layout permit.
- **Replacement install:** replace the selected OS only after explicit, immediate, target-specific confirmation and a tested recovery path.
- **Managed/factory mode:** optional vendor or fleet deployment with declared policy and owner authority.

### 22.3 Host bootstrapper

The bootstrapper must run without requiring the user to create boot media. It verifies its own signature and downloads only signed manifests/artifacts. It detects host OS and version, architecture, firmware mode, disk layout, free space, encryption state, boot settings, battery/power state, network state, and required hardware features. Unknown state forces a safe pause.

### 22.4 Installation plan

Before any persistent change, the installer displays target disk/partition, estimated space, existing systems affected, encryption consequences, boot changes, data-preservation plan, network transfer, required reboot, and recovery method. “Recommended” cannot conceal a destructive option. Replacement mode requires the user to type or explicitly confirm the target identity and the fact that data may be destroyed.

### 22.5 Transaction states

`DISCOVER -> VERIFY_BOOTSTRAPPER -> FETCH/MOUNT_ARTIFACTS -> VERIFY_MANIFEST -> COMPATIBILITY_CHECK -> PLAN -> USER_APPROVAL -> BACKUP/RECOVERY_CHECK -> STAGE -> VERIFY_STAGED -> REGISTER_BOOT_PATH -> REBOOT_IF_REQUIRED -> FIRST_BOOT_VERIFY -> COMMIT`.

Failure states include `ABORTED`, `RETRYABLE`, `RESTORE_HOST_BOOT`, and `MANUAL_RECOVERY_REQUIRED`. Each state has idempotent retry behavior and a defined maximum retry count. No state is allowed to mark installation complete before first-boot verification passes.

### 22.6 Recovery requirements

The installer must maintain an internal recovery plan independent of the newly installed root filesystem where hardware allows. It must test whether the previous boot path can be restored, store recovery instructions locally, verify sufficient free space for staging and recovery, and protect user data. It must not claim rollback if the platform offers no tested route to undo the changes.

### 22.7 Host-specific limits

A running OS cannot always safely replace its own in-use system partition without rebooting or using a separate environment. A blank device with no existing OS and no firmware-supported network/factory bootstrap cannot universally install from an ordinary app because no app is available to run. On such devices, factory provisioning, network boot, built-in recovery, or removable media may be required. Senvoryn must state these cases clearly rather than promise impossible installation behavior.

### 22.8 Install acceptance tests

The installer MUST be fault-injected at every transition: power loss, network loss, corrupt image, signature failure, low disk space, invalid target, cancellation, failed boot registration, failed first boot, and inability to restore. It must prove data preservation for modes that promise preservation and must never continue through an unverified artifact.

## Article 23 — Updates, package system, and supply chain

### 23.1 Immutable artifact model

Released system components are identified by immutable content hashes and signed metadata. Mutable configuration is separated from code. Package resolution uses explicit versions and dependency hashes so the same declared input set does not silently change because a remote repository changed.

### 23.2 Package manifest

Every package manifest declares component identity, version, target profile, architecture/ABI, source provenance, source hash, build recipe hash, license, SBOM reference, permissions requested, services/ports installed, data migration steps, resource estimates, update channel, health checks, rollback class, and signing authority.

### 23.3 Dependency rules

Dependency graphs must be acyclic where the package type requires it; cycles must be rejected or handled by an explicit bootstrap mechanism. An undeclared runtime download is prohibited for trusted system packages. Vendored code must be listed. Floating dependency ranges are allowed only in development and must be resolved to immutable versions in official builds.

### 23.4 Update transaction

An update has `DISCOVERED`, `VERIFIED`, `STAGED`, `PRECHECKED`, `ACTIVATING`, `HEALTH_CHECK`, `COMMITTED`, `ROLLED_BACK`, or `BLOCKED` states. A signature failure, provenance mismatch, revoked signer, forbidden license, failed migration precondition, or unmet hardware profile blocks activation.

### 23.5 Anti-downgrade

The system must detect replay of an older vulnerable release. Rollback to a prior release is allowed only if policy explicitly permits it and the downgrade security impact is presented. Emergency security revocation must survive a normal system rollback. Data migrations that are not backward-compatible must be staged with an explicit backup or conversion plan.

### 23.6 Reproducible and independently verifiable builds

Critical components should be reproducibly built by independent parties. For every official release, the project publishes source revision, toolchain versions, build recipe, dependency lock data, SBOM, attestations, hashes, signature chain, test results, and a list of components whose builds are not yet reproducible. A reproducibility mismatch triggers investigation; it must not be waved away without a public finding.

## Article 24 — Distributed scale and Sovereign Federation

### 24.1 Scale model

Senvoryn uses a common node contract across single-device and multi-node profiles. The local kernel and local SCP enforce boundaries. Fleet services distribute intent, policy, and workload requests, but each node validates and applies them locally.

### 24.2 Fleet authority

Fleet-wide actions use scoped group membership and per-node grants. “Admin of cluster” is not an unbounded permission. Policies may define distinct authorities for inventory, software deployment, data access, power management, network changes, and recovery. High-risk fleet actions require staged rollout and blast-radius limits.

### 24.3 Rollout rings

Updates use progressive rings: canary, limited cohort, broad cohort, and complete deployment. Each ring has health gates and automatic stop conditions. A failure in a canary prevents automatic expansion. Fleet rollback preserves local evidence and does not roll back unrelated application data blindly.

### 24.4 Partition behavior

During network partitions, the node continues permitted local workloads under cached policy and queues safe sync operations. Operations requiring global uniqueness or cross-node atomicity must either use a defined consensus/lease protocol or fail/defer. The system must never fake global consensus when partitions prevent it.

### 24.5 No global hidden administrator

Federation has no universal hidden credential. Emergency fleet recovery requires declared identity, scoped authorization, multi-party approval where configured, and a visible event on each reachable node. Unreachable nodes cannot be assumed to have applied a command.

## Article 25 — Universal Device and Driver Contracts

### 25.1 Hardware independence

Portable services use stable Senvoryn device contracts rather than directly accessing vendor-specific registers. The HAB maps native drivers to standard classes for storage, network, graphics, input, audio, sensors, accelerators, power, clocks, and cryptographic hardware.

### 25.2 Device lease

A device lease contains device identity, operations allowed, memory/DMA boundaries, interrupt assignment, expiry, user visibility requirement, reset policy, and recovery owner. Leases are revoked when the associated task is stopped or quarantined, except for documented device-reset sequences.

### 25.3 Driver certification

Driver certification includes fuzzing of device-provided inputs, malformed queue handling, DMA boundary tests, crash/restart, timeout behavior, power-state changes, firmware mismatch behavior, and proof that access is restricted to the intended device. Devices that cannot provide required isolation are assigned a lower assurance tier and may be unavailable for high-security use.

### 25.4 Architecture profiles

The first architecture profile must be chosen through an explicit Architecture Decision Record (ADR) based on toolchain maturity, device access, emulator availability, target hardware, contributor capability, and proof/testing cost. Additional CPU architectures require conformance tests and a verified boot path. “Portable by design” is not the same as “already runs everywhere.”

## Article 26 — User experience and universal operations

### 26.1 One control vocabulary

The UI should consistently distinguish **request**, **permission**, **policy**, **plan**, **execution**, **verification**, **recovery**, and **evidence**. These are different states and must not be hidden behind a single spinner or green check mark.

### 26.2 Permission language

Prompts must say what an app will access, why the permission is requested, what specific data or device is in scope, whether data leaves the device, how long permission lasts, and how to revoke it. Avoid ambiguous “Allow all” controls. The system should offer least-privilege alternatives.

### 26.3 Human factors

Critical actions use confirmation patterns resistant to misclicks, deceptive UI, and prompt fatigue. A confirmation shows the target and consequence. Repeated low-risk operations may be covered by a user-authored rule, but the rule is inspectable, scoped, and revocable.

### 26.4 Accessibility

All essential settings, recovery, installer, app launch, permission review, and privacy controls must be navigable without a mouse. Screen-reader labels and focus order are specified. Motion is optional. A plain-language mode may simplify presentation but must not omit critical consequences.

### 26.5 Automation control center

Every automation or AI agent has a visible card showing owner, purpose, allowed tools/capabilities, schedule/event trigger, resource budget, data access, network destinations, last run, last effect, next run, failure history, and disable/revoke action. No hidden background agent is allowed in the official distribution.

## Article 27 — Developer platform and application contract

### 27.1 Native application manifest

A native application manifest must declare name, identity, version, ABI, architectures, entrypoint, dependency set, requested capabilities, data labels, resource envelope, network policy, device access, service endpoints, lifecycle hooks, update policy, license, and provenance.

### 27.2 App lifecycle

`AVAILABLE -> VERIFIED -> INSTALLED -> PERMISSIONED -> READY -> RUNNING -> SUSPENDED -> STOPPED -> REMOVED`.

Not every app uses every state. Transitions must be explicit, and service apps must declare whether restart is safe. Removal revokes capabilities first, stops execution, then handles user data according to the user's retention choice.

### 27.3 APIs and ABI stability

Senvoryn defines a versioned native ABI, IPC contracts, application manifest schema, capsule lifecycle API, UI semantics API, and package metadata format. Breaking changes require a compatibility plan and documented deprecation period. The kernel ABI should remain as small and stable as possible; complex features belong in versioned user-space service contracts.

### 27.4 App distribution

No single app store is mandatory. Users may install from the official repository, an organizational repository, or a third-party source. Third-party source trust is separate from application permissions. The OS must show signer identity, source, license, provenance status, and requested capabilities before first launch.

## Article 28 — Accessibility, language, and low-friction operation

Senvoryn's ease-of-use goal includes first boot, account setup, app installation, software compatibility selection, permissions, updates, backup, recovery, and removal. A novice must not need to understand the kernel, mount tables, firmware variables, or package dependency internals to perform ordinary safe tasks.

The UI may automate complexity only after it has established a safe plan. Advanced controls remain available to experts. A novice mode must not silently remove user choice or hide the effects of high-impact actions.

The system should support offline documentation, localization, font fallback, input methods, screen readers, high contrast, magnification, keyboard navigation, and recovery instructions accessible without network access.

---

# PART III — SAFETY, ASSURANCE, AND OPERATIONS

## Article 29 — System states and recovery

### 29.1 Required states

- **NORMAL:** validated required components and policies are healthy.
- **DEGRADED:** some functions are unavailable; essential functions continue within constraints.
- **READ_ONLY:** state-changing operations are disabled except explicitly approved recovery work.
- **QUARANTINED:** a capsule, service, device, node, credential, or artifact is isolated after a defined trigger.
- **RECOVERY:** the system runs a minimal trusted recovery path.
- **MAINTENANCE:** authorized service/update operations run under reduced user workload.
- **SAFE_BOOT:** minimal drivers/services are used to diagnose startup failures.
- **UNTRUSTED_BOOT:** integrity verification is incomplete or failed; sensitive capabilities are restricted.

### 29.2 Transition rules

Each state transition requires a trigger, authority, preconditions, actions, timeout, exit criteria, and audit event. No AI model can directly switch system state. An AI may submit a typed anomaly report; a deterministic transition policy decides whether to change state.

### 29.3 Quarantine

Quarantine blocks new capabilities, restricts network paths, limits resource usage, preserves a bounded evidence snapshot, and records the trigger. It must not destroy user data by default. Releasing quarantine requires a named policy condition or authenticated authority; the system must show why quarantine was entered and what evidence remains unresolved.

### 29.4 Recovery independence

The recovery path must not depend solely on the currently running root filesystem, package manager, model, or network. Recovery artifacts are signed and verified. Where hardware prevents separate recovery storage, the profile must publish that limitation and define an alternative.

### 29.5 Recovery acceptance

Release candidates must survive fault injection at boot, update staging, policy activation, driver startup, storage recovery, capsule creation, and AI service failure. Recovery claims must specify whether they cover software fault, filesystem corruption, disk failure, key loss, firmware compromise, or physical damage.

## Article 30 — Audit, evidence, and operational observability

### 30.1 Events

Security-relevant events include capability grant/denial/revocation, policy changes, installer steps, update verification, privileged API use, device lease changes, capsule lifecycle transitions, external data transfer, key rotation, recovery transitions, AI-triggered proposals, human approvals, and failed integrity checks.

### 30.2 Event envelope

An event records timestamp/clock quality, node identity, actor principal, action type, target identity, policy epoch, capability/reference, result, reason code, Change Capsule/Intent ID where applicable, evidence hash, and sensitivity label. Do not include secrets or full private payloads by default.

### 30.3 Tamper evidence

The ledger should chain event hashes or use another reviewed tamper-evident scheme. If the device lacks a protected clock or hardware-backed storage, the log must state that a privileged attacker could alter or truncate local evidence. Optional external witnesses must be opt-in and privacy-aware.

### 30.4 User access

Users can view an understandable history of sensitive access and changes. Administrators can export detailed audit records subject to data access controls. An app cannot erase its own denial events or records of actions it requested.

## Article 31 — Formal specification and verification

### 31.1 Specification before implementation

The core object model, capability transitions, IPC schemas, policy merge rules, Change Capsule lifecycle, installer state machine, and update state machine must have precise definitions before their stable APIs are frozen.

### 31.2 Verification ladder

Each critical component must use an appropriate combination of:

1. contract examples and state-machine tests;
2. unit and integration tests;
3. property-based testing;
4. malformed-input fuzzing;
5. concurrency and race tests;
6. model checking for small but critical state spaces;
7. static analysis and memory-safety analysis;
8. formal proof where feasible and justified;
9. independent review and adversarial testing;
10. hardware-profile testing and failure injection.

Formal proof applies only to the model, code, compiler, assumptions, and properties covered by the proof. The release must publish the proof scope and exclusions.

### 31.3 Security invariants as executable tests

Every constitutional invariant with observable behavior must map to at least one automated test or a written reason why it cannot be tested automatically. A test must define setup, adversarial action, expected outcome, telemetry/evidence, failure severity, and owning component.

### 31.4 Reproducibility

Critical releases should be built by at least two independently administered build environments from pinned inputs. Differences must be explained before the affected artifact receives the highest trust tier. Reproducibility improves provenance but does not prove source code is benign.

## Article 32 — Incident response and vulnerability handling

### 32.1 Reporting

The project must maintain a private reporting path for vulnerabilities and a public policy describing response timelines, researcher safety, coordinated disclosure, credit, and the transition from embargo to public advisory.

### 32.2 Severity and response

Security issues are triaged by impact, exploitability, privilege boundary, data sensitivity, affected profiles, and availability. Critical defects in the core, capability enforcement, signing, installer, or update path can block releases. A fixed calendar alone does not determine severity.

### 32.3 Incident containment

A response plan covers key revocation, update channel suspension, affected-version inventory, quarantining optional services, safe downgrade/rollback, user notification, forensic evidence preservation, and recovery-key risk. If a release-signing key is suspected compromised, the project must have a recovery trust-root procedure prepared before the event.

### 32.4 No blame concealment

The project must publish a post-incident report describing affected versions, impact, root causes to the extent known, mitigations, test additions, remaining uncertainty, and next review date. Sensitive exploit details may be delayed when necessary for remediation, with a documented reason and review date.

## Article 33 — Safety cases and claims ledger

Every stable feature that can affect security, privacy, data integrity, compatibility, or installation must have a safety case containing:

- top-level claim;
- decomposed subclaims;
- assumptions and dependencies;
- implementation references;
- evidence and test results;
- independent reviewers;
- known counterexamples/limitations;
- residual risk owner;
- next review date.

Senvoryn maintains a public **Claims Ledger** classifying each claim as `PROPOSED`, `IMPLEMENTED`, `TESTED`, `INDEPENDENTLY REVIEWED`, `FORMALLY VERIFIED WITH SCOPE`, or `WITHDRAWN`. A feature's UI and marketing must not advertise a stronger status than the ledger shows.

## Article 34 — Non-functional goals and performance methodology

### 34.1 Measurement rules

Performance targets are provisional until measured on named reference hardware. Every benchmark records device model, firmware, build identifier, workload, repetitions, warm/cold cache state, thermal state, power profile, and result distribution (at least median and p95/p99 where meaningful). Published comparisons must use the same workload and conditions.

### 34.2 Proposed target categories

The first architecture review must set final targets, but the project should begin with measurable goals for:

- boot-to-interactive and headless service readiness;
- native capsule launch latency;
- capability-check latency;
- IPC latency/throughput;
- memory used by idle base system per hardware profile;
- CPU/energy overhead of enforcement;
- guest startup and memory overhead;
- installer time and network usage;
- update staging and recovery time;
- scheduler responsiveness under resource exhaustion;
- local AI token latency and energy when a model is used;
- privacy/security control overhead.

### 34.3 Initial benchmark posture

Do not set arbitrary universal microsecond or memory promises before the kernel prototype exists. Establish a reference implementation, publish raw benchmark harnesses, and set per-profile targets after early measurement. Once targets are ratified, regressions beyond the agreed threshold block stable release or require a public waiver.

## Article 35 — Interoperability and compatibility governance

A compatibility claim belongs to a tuple: `(application identity, application version, execution mode, guest/runtime version, device profile, feature suite, permission profile, test date)`. “Supports Windows apps” or “supports Android apps” is too broad to be a meaningful release claim by itself.

The compatibility lab maintains a curated suite of open test applications, API conformance tests, file-format tests, input/output tests, GPU/media tests, network tests, accessibility tests, license checks, and failure-isolation tests. Proprietary app results are published only where legally permitted and must not expose user data or secret test credentials.

The OS must allow an app to fail compatibility validation without treating the app as malicious. Compatibility failure and security violation are distinct conditions and need distinct messages.

## Article 36 — Migration, backup, and user data sovereignty

### 36.1 Import

Migration tools must read source data with least privilege, show the categories and destinations of imported content, preserve source files unless deletion is explicitly requested, and produce a validation report. Imported executables are untrusted and must not automatically run.

### 36.2 Export

Personal data must be exportable in documented, reasonably accessible formats. The export tool should separate user content, app-specific opaque data, settings, credentials, AI memory, and system policy. Secrets must be excluded unless explicitly selected and re-encrypted for the destination.

### 36.3 Backup

Backups include integrity checks, encryption, retention controls, and restore drills. A backup status is not considered healthy until a sample restore or a verifiable integrity test has succeeded. Backups cannot rely solely on the same credentials and device that they are intended to recover.

### 36.4 Data migrations

Each app/system data migration declares version preconditions, backup requirement, expected writes, rollback feasibility, duration, and failure behavior. Destructive migrations are staged or copy-on-write when feasible. Where rollback is impossible, the user must be told before commit.

## Article 37 — Research frontier and explicitly experimental capabilities

The following ideas are candidates for differentiating Senvoryn, but must remain experimental until evidence supports them:

- **Intent-to-Effect Compiler:** compile a human goal into a validated set of machine effects and explicit capability requests.
- **Capability Graph Explorer:** show who or what can affect a resource and the exact permission path.
- **Shadow-State Rehearsal:** simulate a proposed change against a bounded snapshot before execution.
- **Proof-Carrying Effects:** each privileged state change carries a bundle of authorization, provenance, precondition, postcondition, and recovery evidence.
- **Reversibility Ledger:** classify actions by how they can be undone, compensated, or not undone.
- **Private Compute Twin:** a user-controlled local model of system dependencies, preferences, and resource patterns, not a copy of all user data and not an authority source.
- **Resource Ecology Scheduler:** coordinate energy, thermal load, latency, and workload importance under hard quotas.
- **Sovereign Continuity:** transfer a task or session among enrolled devices without a mandatory central cloud, while keeping hardware-bound secrets local.
- **Self-describing compatibility capsules:** apps publish machine-readable API needs, features and limits; the system measures and reports actual coverage.
- **Policy-aware adaptive UI:** simplify workflows and predict needs without hiding material consequences or changing permissions implicitly.

No experimental feature may weaken INV-01 through INV-30. Novelty is not sufficient evidence of usefulness or safety.

## Article 38 — Delivery, release levels, and lifecycle

### 38.1 Release channels

- **Research:** incomplete, may break, no stable security promise.
- **Developer Preview:** selected hardware and features, explicit gaps.
- **Beta:** feature testing, compatibility matrix, known issue register, limited assurance.
- **Stable:** required conformance suites pass; critical release pipeline and recovery procedures are audited to the declared scope.
- **Long-Term Support:** narrow supported profile, documented patch window, compatible migration policy, and maintained recovery artifacts.

### 38.2 Release gate

A stable release MUST NOT ship unless: the SBOM and license inventory are complete; critical build provenance is published; no unresolved release-blocking security issue remains; capability tests pass; installation/recovery tests pass for the certified matrix; the Claims Ledger is current; known limitations are public; and rollback/revocation procedures are rehearsed.

### 38.3 End of support

End-of-support dates must be published. The system warns before a profile becomes unsupported and explains the security consequences. It should not silently continue to label an unsupported version as secure.

---

# PART IV — NORMATIVE INTERFACES AND DATA CONTRACTS

## Specification A — Error model

All system APIs return structured results. An error contains a stable code, component, operation ID, retryability, safe user message key, diagnostic reference, and optional remediation. Errors must not include secrets or raw untrusted content. At minimum, define distinct codes for:

- `DENIED_NO_CAPABILITY`
- `DENIED_POLICY`
- `DENIED_STATE`
- `DENIED_EXPIRED_CAPABILITY`
- `DENIED_REVOKED_CAPABILITY`
- `DENIED_DATA_LABEL`
- `DENIED_UNTRUSTED_PROVENANCE`
- `INVALID_ARGUMENT`
- `INVALID_SCHEMA_VERSION`
- `RESOURCE_LIMIT`
- `RESOURCE_TEMPORARILY_UNAVAILABLE`
- `PRECONDITION_FAILED`
- `POSTCONDITION_FAILED`
- `SIGNATURE_INVALID`
- `SIGNER_REVOKED`
- `REPLAY_DETECTED`
- `COMPATIBILITY_UNAVAILABLE`
- `RECOVERY_REQUIRED`
- `PARTIAL_EXTERNAL_EFFECT`
- `USER_APPROVAL_REQUIRED`
- `EXPIRED_PLAN`
- `INTERNAL_FAULT`

Client applications may localize messages but must preserve machine codes. The same condition should not be reported as success in one subsystem and failure in another.

## Specification B — Origin Capsule manifest example

The following is illustrative schema content, not a finalized parser grammar. Field names can change during RFC review, but all required semantics must remain.

```yaml
schema: senvoryn.origin-capsule/v1
identity:
  id: org.example.editor
  version: 1.4.2
  publisher: example-publisher-key-id
  artifact_digest: sha256:<digest>
execution:
  mode: UEF-Native
  abi: senvoryn-abi/v1
  architecture: [x86_64]
  entrypoint: app/main
resources:
  cpu_class: normal
  memory_max_bytes: 536870912
  storage_max_bytes: 2147483648
  process_count_max: 64
  network_bytes_per_minute: 0
permissions_requested:
  files:
    - object: user-selected:documents
      rights: [read, create_new]
      lifetime: session
  devices: []
  network:
    mode: none
  secrets: []
data:
  reads: [PERSONAL]
  writes: [PERSONAL]
  declassification: none
lifecycle:
  restart_policy: bounded
  max_restarts_per_hour: 3
  removal_policy: ask_user
provenance:
  source_revision: <immutable-reference>
  build_recipe_digest: sha256:<digest>
  sbom_digest: sha256:<digest>
  license_expression: Apache-2.0
assurance:
  compatibility_level: C0
  test_suite: <test-suite-id>
  last_verified: <date>
```

The OS must reject inconsistent declarations, unsupported modes, unknown required fields, illegal permission combinations, and any attempt to imply a host permission through guest metadata.

## Specification C — Change Capsule example

```yaml
schema: senvoryn.change-capsule/v1
id: cc-<unique-id>
origin:
  principal: local-user:<id>
  intent_id: intent-<id>
target:
  node: local-node
  objects: [service:search-index]
impact:
  class: I2
  reversibility: R1
  blast_radius: [service:search-index]
preconditions:
  policy_epoch: 42
  require_free_space_bytes: 4294967296
  require_backup_reference: backup:<id>
requested_capabilities:
  - object: service:search-index
    rights: [stop, update, start, inspect_health]
    expires_after_seconds: 900
steps:
  - id: stage
    action: install_verified_artifact
    idempotency_key: <key>
  - id: switch
    action: activate_generation
    depends_on: [stage]
  - id: verify
    action: run_health_suite
    depends_on: [switch]
postconditions:
  - service_health: healthy
  - audit_record_durable: true
recovery:
  plan: restore_previous_generation
  max_attempts: 1
  preserve_failure_evidence: true
approval:
  policy_required: true
  human_confirmation: not_required_if_preapproved_narrow_rule
provenance:
  artifact_digest: sha256:<digest>
  signer: release-key-id
  signature: <signature>
expiry: <timestamp>
```

The executor must treat the manifest as data. It validates schema and authority; it never evaluates arbitrary code embedded in descriptive fields.

## Specification D — SCP policy decision record

Every policy evaluation yields a record with:

- request ID and caller identity;
- requested operation, object, purpose and scope;
- capability handles presented (not secret token contents);
- applicable policy IDs and policy epoch;
- data labels and system state;
- decision: allow, deny, defer, or require approval;
- reason codes;
- constraints and expiry of any grant;
- related Intent/Change Capsule ID;
- audit sensitivity class;
- evaluator version and integrity/provenance reference.

The record must explain which deterministic rules decided the result. It must not depend on an AI-generated justification.

## Specification E — State transition contract

For every state transition, define:

`Transition = (source_state, event_type, authenticated_source, preconditions, guard_policy, effects, timeout, target_state, evidence, compensation, retry_rule)`.

A transition must be idempotent or must declare how duplicate events are detected. Illegal transitions return a structured error and leave the prior state unchanged. Timeouts lead to an explicit fallback state, not indefinite limbo. A source that is not authorized to emit a trusted health event cannot directly cause a privileged transition.

## Specification F — Device Capability Record (HCR)

The HCR must include schema version, device identity/assurance level, hardware inventory, architecture features, memory layout, interrupt topology, DMA isolation availability, virtualization capabilities, secure boot state, cryptographic hardware state, boot measurement references when available, storage/encryption features, thermal/power sensors, driver versions, unknown fields, and timestamp/clock quality. Sensitive identifiers are minimized and are never transmitted without a policy-approved purpose.

## Specification G — AI Proposal record

An AI proposal must include task ID, requesting principal, model identity/version, input data labels, source references, candidate effects, requested tool/capability set, uncertainty flags, assumptions, expected outcome, failure modes, simulation results if any, and expiry. It must be signed or integrity-bound to the AI capsule's identity where feasible. The proposal itself never authorizes effects.

## Specification H — Package and release manifest

A release manifest must include release ID, target profiles, source revision, build recipe, immutable artifact digests, package graph, SBOM, license inventory, test reports, reproducibility reports, signer identities, freshness metadata, minimum accepted policy/boot versions, known issues, rollback class, key revocation information, and signature chain. A device rejects a manifest missing mandatory fields for its trust tier.

## Specification I — Compatibility record

A compatibility record binds app/version, source and signer, execution mode, runtime/guest versions, hardware profile, required licenses, feature checklist, security boundary, permission defaults, test corpus digest, results, performance measurements, known unsupported features, date, reviewer, and expiration date. Records expire when the app, runtime, driver, guest, or OS changes in a relevant way.

---

# PART V — CONFORMANCE AND ACCEPTANCE

## Article 39 — Conformance classes

Conformance is graduated, not binary.

- **S0 Architecture Prototype:** boot or run in a simulator; APIs are unstable; no production security claim.
- **S1 Isolated Core:** core object model, task isolation, IPC validation, capability denial, basic resource limits and fault containment pass automated tests.
- **S2 Secure Local System:** installer, signing, encrypted data options, update staging, rollback, audit, privacy defaults and local SCP are implemented and reviewed.
- **S3 AI-Native System:** Intent Transaction Compiler, AI capsule isolation, permission-aware tool bridge, shadow-state rehearsal, user memory controls and AI disablement pass adversarial tests.
- **S4 Compatibility Platform:** UEF supports named execution modes with an audited compatibility matrix and guest-isolation evidence.
- **S5 Federated Platform:** multi-node identity, policy distribution, local enforcement, staged fleet rollout, partition behavior, and node quarantine pass conformance tests.
- **S6 Independently Assessed Stable Profile:** critical code and release procedures have been independently assessed to a published scope; the claims ledger matches evidence.

No release may claim a class until all mandatory tests for that class pass on the declared reference profiles.

## Article 40 — Acceptance-test matrix

The following is a minimum suite. Every test must be automated where feasible and retained with a versioned report.

| ID | Requirement | Test action | Passing result | Severity if failed |
|---|---|---|---|---|
| SEC-001 | Default deny | Invoke a protected operation without capability | Operation denied; structured event recorded | Release blocker |
| SEC-002 | No self-escalation | Capsule attempts to create a broader capability | Denied; existing authority unchanged | Release blocker |
| SEC-003 | Attenuation | Delegate a narrower capability, then attempt original broader right | Receiver can only exercise attenuated rights | Release blocker |
| SEC-004 | Revocation | Revoke a capability during repeated invocations | New invocations fail within documented revocation bound | Release blocker |
| SEC-005 | Cross-capsule isolation | Capsule A attempts to read/write Capsule B memory and private files | Denied except explicitly granted shared objects | Release blocker |
| SEC-006 | Network default off | Launch capsule with no network grant | No outbound/inbound route or DNS disclosure | Release blocker |
| SEC-007 | Guest privilege containment | Guest administrator attempts host operation | Host operation denied unless separately authorized | Release blocker |
| SCP-001 | Deterministic policy | Replay same request/policy/state inputs | Same decision and reason codes | Release blocker |
| SCP-002 | Conflicting policy | Create equal-priority allow/deny conflict | Deny plus conflict record | Release blocker |
| SCP-003 | Offline autonomy | Disconnect fleet service | Local policy enforcement continues; unsafe grants stop | Release blocker |
| CC-001 | Invalid signature | Apply tampered Change Capsule | Rejected before effect | Release blocker |
| CC-002 | Stale precondition | Change policy epoch after planning | Capsule blocked and revalidated | Release blocker |
| CC-003 | Partial failure | Fail a step after one prior step committed | Explicit compensation/partial state; no false atomicity claim | Release blocker |
| CC-004 | Replay | Re-submit consumed capsule ID | Duplicate rejected or idempotently handled | Release blocker |
| AI-001 | Prompt injection | Malicious document asks model to exfiltrate files | Model text cannot grant permission; tool call denied | Release blocker |
| AI-002 | Disable AI | Disable all model services | Boot, files, settings, updates, recovery remain usable | Release blocker |
| AI-003 | External disclosure | Ask model to send sensitive data to remote inference | User consent and policy required; no send by default | Release blocker |
| AI-004 | Unsupported tool call | Model emits unknown tool/schema | Rejected without execution | Release blocker |
| PRIV-001 | Telemetry off | Fresh install, default settings, observe traffic | No optional telemetry egress | Release blocker |
| PRIV-002 | Label propagation | Send sensitive content to a lower-label log/export | Blocked or requires authorized declassification | Release blocker |
| PRIV-003 | Memory deletion | Delete AI memory item and query retrieval | Item is absent from active memory; backup limits disclosed | High |
| SUP-001 | SBOM complete | Scan official artifact graph | All shipped components have identity/version/license status | Release blocker |
| SUP-002 | Dependency tampering | Alter dependency without manifest update | Hash/signature verification fails | Release blocker |
| SUP-003 | Rebuild mismatch | Independently rebuild critical component | Mismatch is recorded and release trust tier downgraded pending explanation | High |
| INS-001 | Power failure | Cut power at each installer state | System resumes safely, restores host path, or enters documented recovery | Release blocker |
| INS-002 | Wrong target | Select a different disk than intended | Plan names target; destructive action requires fresh explicit confirmation | Release blocker |
| INS-003 | Corrupt image | Tamper with OS image | Rejected before activation | Release blocker |
| INS-004 | No media | Install on certified host without user-created USB/CD | Installer completes without user-created boot media | Release blocker for certified host |
| INS-005 | Preservation | Run side-by-side install with test data | Source data remains unchanged and integrity-checked | Release blocker for side-by-side mode |
| UPD-001 | Anti-downgrade | Replay revoked vulnerable release | Blocked unless narrowly authorized emergency exception | Release blocker |
| UPD-002 | Rollback evidence | Fail update then restore prior generation | Failure evidence remains and old system boots | Release blocker |
| RES-001 | CPU exhaustion | Run unbounded task loop | Other tasks/control plane meet profile latency bounds | High |
| RES-002 | Memory exhaustion | Consume task memory quota | Allocation denied or task contained; kernel state remains intact | Release blocker |
| RES-003 | Restart storm | Crash a supervised service repeatedly | Retry budget prevents unbounded restart and resource consumption | High |
| AUD-001 | Audit minimization | Inspect events for secret contents | No secret or full private payload stored without explicit policy | Release blocker |
| AUD-002 | Tamper evidence | Modify/truncate local audit stream | Tamper detected where profile claims tamper evidence; limits reported | High |
| FED-001 | Fleet blast radius | Deploy failing update to canary | Rollout stops before next ring | Release blocker for S5 |
| FED-002 | Node compromise | Revoke node identity | Peer rejects new grants/requests from revoked node after receiving revocation | Release blocker for S5 |
| COMP-001 | Compatibility honesty | Publish app compatibility result | Record specifies version, profile, features, runtime and date | Release blocker for UEF stable claims |
| COMP-002 | License constraint | Try to run app requiring unavailable license/guest | Install flow discloses blocker; does not silently bypass restrictions | High |
| UX-001 | Permission clarity | New app asks for file/network/device access | Prompt names action, scope, purpose, lifetime and revocation path | High |
| UX-002 | Accessibility | Complete install and recovery with keyboard/screen reader | Critical flow remains operable and understandable | High |

## Article 41 — Provisional measurable targets

The project must not pretend these values are already achieved. They are starting targets for the benchmark council to ratify after a runnable prototype exists.

| Metric | Initial target method | Gate |
|---|---|---|
| Capability authorization latency | Measure p50/p95/p99 for hot and cold object paths on each reference profile | Publish before S2; regressions over 15% require review |
| IPC overhead | Publish latency/throughput by message size and protection boundary | Baseline in S1; optimize only without weakening isolation |
| Idle memory | Measure kernel plus essential services, excluding app data and page cache, per profile | Per-profile budget must be ratified before stable release |
| Installer safety | Fault-inject every state transition and repeat on each certified host | 100% of required recovery cases pass before stable installer |
| Critical SBOM coverage | All shipped executables and linked runtime components identified | 100% required for stable release |
| Default telemetry | Fresh-install network capture with all optional diagnostics off | Zero optional telemetry requests |
| Capability tests | Property/fuzz tests over grant/attenuate/revoke sequences | No unauthorized successful operation in published corpus |
| AI authority | Adversarial prompt/tool suite | Zero direct privilege escalation in test suite; residual risk documented |
| Rollback | Inject update failure, power loss and failed health checks | All supported scenarios reach known-good or explicit recovery state |
| Native application launch | Measure cold/warm p50/p95 by profile | Set target after prototype measurement; no universal claim |
| Compatibility coverage | Feature-based suites per app/guest/profile | Publish mandatory feature pass percentage and known failures |
| Recovery documentation | Offline test on fresh device | Required steps available without network or currently running root |

A numerical target is valid only when the workload, hardware and measurement procedure are published. “100% secure” or “zero vulnerabilities forever” is never a valid metric.

## Article 42 — Roadmap and exit criteria

Dates are not commitments until staffing, funding, and prototype results are known. Work is sequenced by dependency and evidence rather than by marketing launch pressure.

### Phase A — Specification freeze (Gate A)

Deliver: architecture decision records; threat model; kernel object definitions; capability semantics; IPC contract; SCP policy semantics; capsule schema; installer states; license policy; benchmark harness; test harness. Exit only when two reviewers can independently interpret core contracts without contradictory behavior.

### Phase B — Simulator and model (Gate B)

Deliver: executable state machines for capabilities, SCP, Change Capsules, installer, updates, and resource budgets. Run property-based tests and adversarial event sequences before writing large amounts of kernel code. Exit when invariants hold in the model for declared assumptions and counterexamples are tracked.

### Phase C — Minimal native system (Gate C)

Deliver: boot to a minimal shell on one selected architecture; create two isolated tasks; IPC; capability grant/revoke; driver/service boundary; basic storage; signed boot artifact; offline operation. Exit when S1 conformance tests pass.

### Phase D — Trusted local system (Gate D)

Deliver: installer prototype, verified package format, update staging, rollback, audit ledger, local SCP, encrypted user storage, safe recovery, privacy defaults. Exit when S2 tests pass on the certified reference hardware and an external reviewer assesses the threat model.

### Phase E — AI-native control (Gate E)

Deliver: local AI capsule, typed tools, Intent Transaction Compiler, shadow-state rehearsal, user memory controls, policy-visible explanations, AI-off mode. Exit when all AI authority tests pass and high-impact actions remain deterministic and approval-bound.

### Phase F — UEF compatibility (Gate F)

Start with one deliberately selected execution mode, one architecture, and a legally redistributable test suite. Add compatibility modes individually. Exit per mode only when its feature matrix, guest isolation, license review, and performance evidence are published.

### Phase G — Multi-device and federation (Gate G)

Deliver: node identity, local policy authority, staged rollout rings, local autonomy under partition, capability-scoped remote management, continuity protocol. Exit when adversarial partition and compromised-node tests pass.

### Phase H — Stable profiles (Gate H)

Deliver: independent security review, public claims ledger, key-rotation rehearsal, incident response drills, certified installer matrix, reproducible builds for critical components, documentation and support policy. Exit only when all stable release gates are met.

## Article 43 — Open decisions register

The following decisions must be resolved through Architecture Decision Records before the corresponding code is frozen. They are not to be settled by guesswork.

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

## Article 44 — Known hard limits and honest promises

Senvoryn's ambition is expansive, but engineering statements must remain accurate:

1. **“Run any software” is a goal, not a universal guarantee.** Software may depend on unimplemented APIs, proprietary services, device-specific drivers, signed hardware, licenses, DRM, or unavailable hardware.
2. **“Install with one click” means no user-created boot media on certified existing host profiles.** It does not mean no reboot, no permission, no storage change, no data-loss risk, or installation onto every blank device.
3. **“No backdoors” is a governance and verification objective.** Source availability, signed builds, independent rebuilds, and audits reduce risk but cannot prove absence of every hardware, compiler, or design compromise.
4. **“Works on any device” means a portable architecture and expandable profile program.** Unsupported hardware must fail safely and say why.
5. **“AI-native” does not mean AI controls everything.** Core control must remain deterministic, constrained and operable without models.
6. **“Full control” does not authorize violating another vendor's technical protections, licenses, or lawful access controls.** Compatibility must use supported, authorized, or independently implemented interfaces and disclose where a feature cannot be provided.
7. **“Lightweight and superfast” must be measured.** Strong isolation, virtualization, translation and local models have costs. The project must offer profile-specific trade-offs rather than pretending every feature has zero overhead.
8. **“New from the foundation” does not mean inventing new cryptography.** Novelty belongs in architecture, control contracts and user experience, while core cryptographic constructions remain publicly scrutinized.

## Article 45 — Final constitutional commitment

Senvoryn is to be engineered as a system that explains what it intends to do, proves that the actor has authority, constrains the effects, checks the result, preserves evidence, and offers recovery when the result is wrong.

No feature is above its invariants. No AI is above the SCP. No remote controller is above local enforcement. No compatibility claim is above its test record. No release is above its provenance. No person or organization is above the public change process.

The definition of success is not that Senvoryn never fails. The definition is that failures are bounded where possible, visible where they matter, recoverable where recovery is technically possible, and honestly documented where they are not.

---

# APPENDIX 1 — REQUIRED ARCHITECTURE DECISION RECORD TEMPLATE

Each ADR contains:

1. Identifier, title, owner, status, date, and reviewers.
2. Problem statement and non-goals.
3. Constraints and affected constitutional invariants.
4. Candidate designs and trade-offs.
5. Threat-model impact and trusted-computing-base impact.
6. Performance and memory estimates with method.
7. Portability and hardware-profile impact.
8. Compatibility and migration impact.
9. Licensing, provenance, and maintenance impact.
10. Decision and rationale.
11. Test plan, acceptance criteria, and observable evidence.
12. Rollback/exit strategy.
13. Known risks, unresolved questions, and review date.

No security-critical architecture decision may remain only in chat, issue comments, or an individual's memory.

# APPENDIX 2 — REQUIRED COMPONENT SPECIFICATION TEMPLATE

Every component specification contains:

- Purpose and explicit non-goals.
- Owner and supported profiles.
- Threat model and trust assumptions.
- Inputs, outputs, schemas, limits and error codes.
- Identity and capability requirements.
- State machine and concurrency model.
- Persistence, secret handling, and data labels.
- Resource budget and overload behavior.
- Timeout, retry, idempotency, cancellation and recovery rules.
- Observability and audit events.
- Versioning, compatibility, and migration rules.
- Test cases, fuzz targets, fault-injection scenarios, and evidence requirements.
- Security/privacy review and residual risks.

If a component has no defined failure behavior, it is not ready for implementation in the trusted system.

# APPENDIX 3 — GLOSSARY

- **AI-native:** AI is available as a first-class system capability, while essential OS functions remain operable without it.
- **Capability:** An unforgeable, bounded authority to act on a specified object or operation.
- **Capability Graph:** A queryable model of principals, resources, grants, purposes, dependencies, and data flow; it is not itself the enforcement mechanism.
- **Change Capsule:** An immutable state-transition plan with preconditions, authorization, effects, validation, and recovery/compensation semantics.
- **Claims Ledger:** Public record of each security/privacy/performance/compatibility claim and its evidence status.
- **HCR:** Hardware Capability Record.
- **Intent Record:** A structured request describing desired outcome, requester, assumptions, effects, risks, and authorization needs.
- **Intent Transaction Compiler:** Component that converts intent into a candidate bounded plan and validates it before authorization.
- **LIF:** Local Intelligence Fabric.
- **Origin Capsule:** Isolated execution domain for an application, service, driver, model, or foreign runtime.
- **Reversibility Ledger:** Record that identifies what changed and what kind of undo/compensation, if any, is possible.
- **SCP:** System Control Plane; policy and state coordination with no authority to bypass kernel-enforced rules.
- **SSC:** Senvoryn Separation Core.
- **Shadow State:** A bounded snapshot/model used to preview likely effects; not a guarantee of future behavior.
- **UEF:** Universal Execution Fabric.
- **User-owned memory:** AI memory whose contents, access, retention, export and deletion are controlled by the user.

# APPENDIX 4 — MINIMUM RELEASE EVIDENCE BUNDLE

Every official release must publish or privately provide to the responsible independent assessor, as appropriate:

1. Source tag and immutable source revision.
2. Signed release manifest and complete artifact hashes.
3. Build recipe, toolchain pins, dependency locks, SBOM and license inventory.
4. Build provenance and attestations.
5. Reproducibility reports and explained mismatches.
6. Results of unit, integration, property, fuzz, race, resource-exhaustion and fault-injection tests.
7. Capability and SCP invariant test results.
8. Installer and recovery test results for the certified host matrix.
9. Compatibility matrix with feature-level outcomes and limitations.
10. Performance/energy measurements by reference profile.
11. Known issues, threat assumptions, residual risks and advisories.
12. Signing-key status and revocation instructions.
13. Claims Ledger status and reviewer sign-offs.
14. Upgrade/downgrade and user-data migration notes.
15. A supported recovery path and offline documentation.

# APPENDIX 5 — MINIMUM FIRST PROTOTYPE SCOPE

The first working prototype should be deliberately small. It must demonstrate the core idea, not every future feature.

**Required prototype:** one architecture in a simulator or development board; a boot path; two isolated tasks; typed IPC; kernel-bound capabilities; a deterministic SCP policy evaluator; a simple Origin Capsule manifest; a signed artifact verifier; a Change Capsule state machine; a local event ledger; a minimal shell; an offline mode; and automated tests showing that denied actions stay denied.

**Out of scope for the first prototype:** universal foreign-app compatibility, large local AI models, production fleet management, universal hardware support, full graphical desktop, and claims of formal verification. These can follow only after the core trust boundaries are demonstrated.

**Prototype success criterion:** an independent reviewer can inspect a reproducible build, run the test suite, attempt known privilege-escalation paths, interrupt the change transaction, and observe that the system denies unauthorized actions, reports the reason, preserves evidence, and reaches a defined recovery state.

---

**End of Senvoryn OS Founding Constitution and Normative Technical Blueprint v0.5.0**


# PART VI — DEEP IMPLEMENTATION SPECIFICATION v0.6.0

**Controlling status:** This part expands v0.5.0 into implementation-level contracts. Where it clarifies a prior clause, this part controls for v0.6.0. If a contradiction remains, implementers MUST stop and file an Architecture Decision Record (ADR); they MUST NOT choose silently.

## Reading this expansion

This is a deep design specification, not a claim that the kernel or services already exist. Each requirement is assigned an identifier. Implementation is not complete until the requirement is linked to code, tests, review evidence, and a supported-profile declaration. Terms such as “secure,” “private,” “fast,” “universal,” and “self-healing” are not acceptance criteria by themselves.

## Normative requirement record

Every requirement record MUST contain: (1) ID; (2) rationale; (3) owner component; (4) preconditions; (5) required behavior; (6) failure behavior; (7) security/privacy implications; (8) tests; (9) evidence; (10) status. Status values are `PROPOSED`, `SPECIFIED`, `IMPLEMENTED`, `TESTED`, `REVIEWED`, `SUPPORTED`, or `DEPRECATED`. Status may advance only with evidence.

---

# PART VII — SYSTEM BOUNDARIES AND DEPENDENCY RULES

## Article 46 — The trusted computing base (TCB) must be explicit

### 46.1 TCB inventory

The project MUST publish a per-profile TCB inventory. At minimum it must identify: boot verification logic; initial kernel and memory manager; capability enforcement; interrupt and DMA isolation; key-handling code; SCP policy evaluator; update verifier; recovery selector; and any firmware or hardware root on which the profile relies. The inventory MUST name components that are outside the TCB but can influence it, such as build infrastructure, signing operators, package metadata, model services, drivers, and update mirrors.

### 46.2 TCB minimization

A component enters the TCB only through an ADR that explains why it cannot run outside it, the privilege it needs, the alternatives considered, and the tests that constrain it. A component MUST NOT be promoted into the TCB merely because it is difficult to isolate. If a feature cannot be isolated, the limitation must be stated in the profile's security label.

### 46.3 No circular trust

A component cannot attest to its own correctness using evidence it alone controls. The SCP must not rely solely on a statement from the process it is authorizing. A release signer must not be the only build verifier. A recovery image must not be trusted solely because the running OS says it is valid.

## Article 47 — Component dependency direction

The dependency graph MUST be acyclic across architectural layers. Higher-level services may call lower-level contracts; lower-level enforcement components MUST NOT import policy decisions, AI libraries, user-interface code, network fleet logic, or application compatibility policy.

Required conceptual dependency direction:

1. Hardware and boot contract.
2. Minimal kernel and isolation primitives.
3. Capability runtime and object model.
4. Local SCP enforcement and event ledger.
5. Core service supervisors, storage, network and identity.
6. Package, update, installer and recovery services.
7. User experience, compatibility fabric, and developer interfaces.
8. AI planner, adaptive optimizer, optional fleet coordination.

A higher layer may propose policy to a lower layer only through a versioned schema. The lower layer must independently validate it.

## Article 48 — Failure containment graph

Every service specification MUST declare which other services it can affect if it crashes, hangs, is compromised, or exhausts resources. The default containment expectation is that one application failure affects only that application's capsule. A failure of a non-core service MUST NOT prevent booting into recovery or using local administrative controls. A fleet coordinator failure MUST NOT remove local node policy. An AI failure MUST NOT disable core operation.

---

# PART VIII — CANONICAL OBJECT AND PROTOCOL CONTRACTS

## Article 49 — Canonical encoding and schema evolution

All privileged control-plane messages MUST use a versioned, unambiguous canonical representation. The implementation must reject duplicate fields, ambiguous numeric encodings, invalid Unicode, unsupported critical extensions, excessive nesting, oversized arrays, and unknown fields marked mandatory. Parsers MUST enforce maximum byte size, nesting depth, string length, list length, and processing time before allocating unbounded resources.

Schemas MUST distinguish absent, null, empty, default, and explicitly denied values. Defaults affecting authority, data export, network access, persistence, or destruction MUST be explicit rather than inferred. A schema change that alters authorization semantics requires a major schema version and migration tests.

## Article 50 — Request envelope

Every SCP request MUST contain the following logical fields, even if a compact wire format is used:

- `schema_version`: supported request schema.
- `request_id`: unpredictable or globally unique request identifier.
- `principal_id`: authenticated caller.
- `process_instance_id`: the concrete process or workload instance.
- `operation`: enumerated operation, not an arbitrary shell command.
- `resource_id`: canonical target object.
- `arguments_digest`: digest of normalized arguments; secret arguments are separately protected.
- `intent_id`: user or policy intent that authorizes the work, if applicable.
- `capability_refs`: capabilities presented by the caller.
- `risk_class`: computed by enforcement; caller-provided risk is advisory only.
- `deadline`: bounded operation deadline using monotonic time where available.
- `idempotency_key`: required for operations that may be retried.
- `origin`: UI, service, remote request, compatibility guest, or AI agent.
- `evidence_refs`: signatures, attestation, approvals or other evidence.

The SCP MUST recompute security-relevant classifications; it MUST NOT trust the caller's declared risk class or display text.

## Article 51 — Decision record

Each decision MUST return a structured result with: decision enum; stable reason code; policy version; capability issued or denial; expiry; constraints; required user action; audit event ID; and retry guidance. The explanation MUST be understandable without exposing policy secrets or another user's data. A denial must not be converted into a permit by retrying with a different spelling, path alias, or untrusted metadata.

## Article 52 — Time and expiry

Authorization expiry MUST use a monotonic time source where possible. Wall-clock time is useful for human records but can be wrong, reset, or manipulated. If trusted time is unavailable, the system must apply a profile-specific conservative policy. Long-lived capabilities SHOULD be avoided; persistent grants must be represented as policy objects that can be inspected, scoped, and revoked, not as immortal bearer tokens.

## Article 53 — Idempotency and replay defense

Every mutating operation MUST state whether it is idempotent. If it is not naturally idempotent, the coordinator must persist an idempotency record before committing the effect, or provide an equivalent exactly-once safety mechanism within the declared failure model. Network retries must not repeat disk destruction, duplicate purchases, duplicate account changes, or repeated external submissions. Where exactly-once semantics cannot be guaranteed, the interface must say so and require reconciliation.

---

# PART IX — CAPABILITY SYSTEM: MICRO-LEVEL RULES

## Article 54 — Capability dimensions

A capability is the intersection of all of the following dimensions; omission does not mean unrestricted access:

1. **Subject:** exact principal or principal class.
2. **Object:** canonical resource or bounded resource set.
3. **Verb:** exact operation set.
4. **Scope:** object subrange, path namespace, record set, device interface, or API method.
5. **Purpose:** declared reason or intent class.
6. **Context:** device posture, system mode, network state, or user session.
7. **Time:** issue time, expiry, and renewal rule.
8. **Budget:** byte, call, CPU, memory, rate, or transaction ceiling.
9. **Delegation:** whether and how the authority can be attenuated and delegated.
10. **Data class:** sensitivity levels the operation may read or write.
11. **Destination:** permitted network endpoint or recipient where relevant.
12. **Audit:** required event granularity and retention class.

The enforcement system MUST calculate the effective capability as the intersection of the grant, parent grants, policy constraints, resource policy, system mode, and current user consent. It MUST NOT union independent grants into a broader capability unless a documented policy explicitly defines the combination.

## Article 55 — Capability attenuation

A child capability MUST be no more powerful than its parent. Attenuation must be monotonic across all dimensions: fewer verbs, narrower object scope, shorter duration, lower budgets, fewer destinations, equal-or-higher audit requirements, and no increase in data sensitivity. The test suite MUST include property-based tests proving attenuation does not widen authority.

## Article 56 — Revocation semantics

Revocation must define its effect on active operations. For immediate revocation, the broker must stop new operations and cancel active operations at a safe boundary. For operations that cannot be interrupted safely, the system must define a bounded completion window and expose the limitation. Revocation must invalidate delegated descendants or advance a revocation epoch that descendants must check. Cache invalidation is security-critical and must be tested under concurrency.

## Article 57 — Object lifetime and stale references

A capability referencing a deleted, replaced, or re-created object MUST NOT silently authorize the new object merely because it reuses a path or name. Object identities must include a generation or equivalent version discriminator. Time-of-check/time-of-use races must be prevented by resolving and binding the target object before authorization is committed, or by equivalent atomic semantics.

## Article 58 — Ambient authority prohibition tests

Every official SDK and service API MUST be tested for ambient authority. Tests must attempt to access ungranted files, discover other applications, enumerate devices, connect to arbitrary endpoints, read clipboard or sensors, spawn child processes, and access secrets. The expected result is denial plus a stable audit reason, without leaking protected existence information unless policy permits it.

---

# PART X — SCP EXECUTION, TRANSACTIONS, AND MODES

## Article 59 — SCP state machine

The SCP has its own state independent of the overall OS mode: `BOOTSTRAP`, `READY`, `DEGRADED`, `RECOVERY_ONLY`, `QUARANTINED`, and `SHUTDOWN`. Each transition must have signed or locally protected causes, allowed operations, and recovery criteria. A request to leave `QUARANTINED` must be evaluated by a component that is not solely controlled by the quarantined principal.

### 59.1 BOOTSTRAP

Only verification, minimal storage initialization, policy loading, key validation, and recovery selection are allowed. User workloads do not start before the minimum policy is active.

### 59.2 READY

Normal decisions are permitted under current policy. The SCP still has no unrestricted filesystem or network access beyond its declared capabilities.

### 59.3 DEGRADED

Nonessential AI planning, background indexing, noncritical updates, and expensive diagnostics may be suspended. The SCP must preserve authentication, policy enforcement, audit continuity, local control, and recovery.

### 59.4 RECOVERY_ONLY

Only signed recovery actions and read-only inspection are allowed unless a reviewed repair procedure authorizes a specific write. Application workloads are not launched.

### 59.5 QUARANTINED

A principal, service, device, or policy domain is restricted. Quarantine scope must be as narrow as evidence permits; it must not disable unrelated emergency access.

## Article 60 — Transaction boundaries

A state-changing transaction MUST declare its atomic boundary. A transaction may include multiple resources only if the coordinator can detect partial completion and has a documented compensation plan. “Rollback succeeded” may be reported only after postconditions prove that the rollback restored the declared state.

## Article 61 — Preflight contract

Before execution, preflight MUST verify: target identity; caller authority; policy version; required dependencies; storage capacity; resource budget; network destinations; user approvals; system mode; recovery availability; schema compatibility; and conflict detection. Preflight results expire when any relevant state changes. The executor must revalidate critical conditions immediately before the irreversible step.

## Article 62 — Cancellation contract

Every long-running operation must declare whether cancellation is: immediate, cooperative, boundary-safe, or unsupported. Unsupported cancellation is permitted only for a narrowly justified operation and must be disclosed before approval. A cancellation request must prevent new side effects and report completed, undone, compensated, and uncertain effects separately.

## Article 63 — State transition table requirement

Every stateful subsystem must publish a table with: current state, event, guard, side effects, next state, timeout, audit event, recovery path, and forbidden transitions. If a transition is not in the table, it is invalid.

---

# PART XI — BOOT, INSTALLER, AND DEVICE OWNERSHIP

## Article 64 — Boot trust chain

The boot chain must verify the next stage before transferring control, where platform support exists. Each stage must bind the verified artifact identity, version, profile, and rollback policy. Verification failure must stop normal boot or enter a clearly labeled recovery path; it must not silently continue with an unverified executable. Devices without hardware-backed verification must declare a lower assurance profile.

The system must distinguish **integrity** (the bytes match a signed or measured artifact), **authenticity** (a trusted identity signed it), **freshness** (it is not an unauthorized rollback), and **safety** (the artifact has been reviewed and tested). These are different properties.

## Article 65 — Direct installation modes

The installer must expose at least these modes where supported:

- **Try mode:** run a non-destructive trial environment, preferably without modifying the existing system.
- **Virtual mode:** install into a virtual machine or managed virtual disk.
- **Side-by-side mode:** allocate a new target region and preserve the existing OS, with explicit boot and recovery implications.
- **Replace mode:** replace the selected system only after explicit, informed confirmation.
- **Managed deployment mode:** apply a signed administrator-approved image to managed devices.

The UI must show the exact disk identity, size, partition map, encryption state, and expected data impact. “Continue” must never conceal a destructive operation.

## Article 66 — Host integration and privilege elevation

The installer must use the host's documented privilege-elevation mechanism. It must not install a persistent privileged service merely to simplify later updates unless the user is told, the service is minimal, its interface is audited, and it can be removed. The elevated component must validate every request independently of the UI. The UI process is untrusted for authorization because it can be compromised or spoofed.

## Article 67 — Power-loss safety

Installation and update operations must be tested with power loss at every persistent-write boundary. The design should use staged writes, durable journals, atomic switching where supported, and integrity checks. On restart, the installer must classify the state as: old system intact; new system staged; new system verified; commit complete; or recovery required. Ambiguous state must not trigger destructive cleanup.

## Article 68 — Device ownership and enrollment

Enrollment into a fleet requires proof of device possession and an owner-approved enrollment action. Factory reset must revoke local credentials and remove user data according to a documented sanitization profile. Remote wipe is a requested operation, not a hidden capability; it requires prior enrollment, strong authorization, and clear limits where the device is offline.

---

# PART XII — SOFTWARE PORTABILITY AND UEF INTERFACES

## Article 69 — Execution selection algorithm

The UEF selects a runtime using this order, unless a security or compatibility policy says otherwise:

1. native Senvoryn package with a verified manifest;
2. source build using a supported, reproducible toolchain;
3. compatible managed runtime;
4. user-space API/ABI translation inside an Origin Capsule;
5. full guest environment with a separately licensed and authorized OS image;
6. remote execution on a user-approved device or service, clearly labeled as remote;
7. refusal with an explanation and exportable diagnostic bundle.

The selector must not choose a less-isolated mode merely because it is faster. A downgrade in isolation requires explicit policy authorization and a visible warning.

## Article 70 — Guest boundary and bridge policy

A guest is treated as hostile even if it boots successfully and its internal administrator is authenticated. Host bridges are individually permissioned: shared folder, clipboard, drag-and-drop, USB passthrough, camera, microphone, GPU, display capture, smart card, location, network, and identity. Each bridge defines directionality, data types, rate limits, logging, revocation, and whether the bridge can be disabled without terminating the guest.

## Article 71 — Compatibility test record

Each compatibility result must include application identifier, publisher, version, package digest, source provenance, runtime/guest version, hardware profile, permissions, test suite version, passed and failed cases, performance data, known limitations, licensing notes, and date of test. The result expires or becomes stale when a material component changes.

## Article 72 — Vendor service and platform entitlement limits

The OS cannot create vendor credentials, bypass legal entitlement, reproduce unavailable server-side services, or manufacture hardware capabilities. When an application requires a vendor service or platform-specific entitlement, the UI must say so. Any interoperability layer must be reviewed for lawful operation and must not misrepresent itself as an official platform implementation.

---

# PART XIII — PRIVACY DATA-FLOW CONTRACTS

## Article 73 — Data classification

Minimum classifications are:

- **D0 Public:** intended for public disclosure.
- **D1 Operational:** non-personal operational data with limited sensitivity.
- **D2 Personal:** user documents, preferences, identifiers, and routine personal data.
- **D3 Sensitive:** credentials, financial/health/legal material, private communications, precise location, and high-risk identifiers.
- **D4 Secret:** cryptographic private keys, recovery secrets, authentication factors, and material whose disclosure enables system compromise.

Classification may be raised by the user or policy. A component must not lower classification simply to make export easier. Unknown data defaults to the more protective classification until reviewed.

## Article 74 — Data-flow authorization

Any process that moves data between classification domains or across the device boundary must declare source, destination, purpose, transformation, data categories, recipient identity, retention, and user authorization. AI summarization does not automatically declassify its source. A generated summary may still reveal sensitive information and inherits the source's classification unless a reviewed declassification policy applies.

## Article 75 — External AI and remote inference

Before personal data is sent to a remote model, the system must disclose the recipient, data categories, purpose, retention/processing terms where known, and whether the request may be used for training. The user must be able to choose local-only behavior or cancel. If the provider's data handling is unknown, the OS must label it unknown and not represent it as private. Secret data is blocked from external inference by default.

## Article 76 — Deletion and retention

Deletion semantics must distinguish logical deletion, key destruction, local overwrite where meaningful, backup expiry, synced-copy deletion requests, and deletion from external services. Solid-state storage, wear levelling, snapshots, and remote copies mean that overwrite is not a universal proof of erasure. The UI must state what was deleted and what cannot be verified. Retention policies must be purpose-limited and visible.

---

# PART XIV — AI SAFETY, MEMORY, AND ADAPTIVE CONTROL

## Article 77 — AI action envelope

Every tool call or system action proposed by AI must be represented in a canonical envelope containing: model identity; input data references; user intent; action name; exact target; parameters; expected effects; risk tier; required capabilities; resource budget; expiry; reversibility classification; validation steps; and cancellation behavior. Natural-language prose is not an authorization envelope.

## Article 78 — Separation of planning and execution

The AI planner generates plans; a deterministic planner validator checks schema and policy; the SCP authorizes; the executor performs; a verifier checks results. These roles must be separated so that a single model response cannot plan, approve, execute, and certify its own action. Where the implementation combines components for resource reasons, the logical boundaries and independent checks must remain enforceable and testable.

## Article 79 — Learning and adaptation guardrails

Online learning must not silently change policy, capability semantics, security classification, update trust, or user consent. A learned model or policy is treated as a new artifact with version, evaluation evidence, rollback path, and approval requirements. Personalization data remains user-controlled. Fleet learning must not upload raw personal data by default. Aggregation or privacy-preserving learning is not automatically safe and must be threat-modeled for leakage and poisoning.

## Article 80 — AI unavailability and adversarial behavior

The OS must remain operable when the model times out, produces malformed output, hallucinates a resource, requests a forbidden tool, loops, exceeds its budget, or returns contradictory instructions. Each failure maps to a defined status and safe fallback. The system must never treat model confidence as proof of authorization or truth.

## Article 81 — User mental model

The UI must distinguish “the AI suggested,” “the system policy allowed,” “the user approved,” “the operation executed,” and “the result was verified.” These are five separate states. A task must not be shown as completed merely because an action was requested or a command returned success.

---

# PART XV — RESOURCE ECOLOGY AND SCHEDULING

## Article 82 — Resource budget vector

Each workload receives a budget vector containing, as applicable: CPU time, memory, resident set, storage quota, I/O rate, network bytes, file descriptors/handles, child-process count, GPU time/memory, energy, and wall-clock duration. A workload cannot trade an unlimited amount of one resource for another. Budgets must be enforced by the lowest practical layer and accounted against the workload's stable identity.

## Article 83 — Admission control

Before launching a workload, the resource manager checks current availability, reservations, pressure forecasts, priority, and minimum guarantees. A forecast is advisory; hard limits are enforced regardless of forecast accuracy. Under pressure, the system applies a documented priority order and graceful degradation. The SCP, audit, authentication, and recovery functions receive protected minimum resources.

## Article 84 — Fairness and starvation

Scheduling policy must define fairness for interactive, batch, background, real-time, and safety-critical workloads. Priority cannot be changed by a workload itself. Long-running low-priority work must either make bounded progress or be explicitly classified as best-effort. A high-priority workload cannot obtain unlimited resources without a reviewed system policy.

## Article 85 — Energy and thermal adaptation

On devices exposing reliable thermal and power information, the OS may adapt frequency, scheduling, display, radios, and background work. It must prioritize hardware safety and user-declared constraints. Energy estimates must be labeled as estimates unless directly measured. AI must not override thermal protection or device safety limits.

---

# PART XVI — TESTING AND PROOF OBLIGATIONS

## Article 86 — Test pyramid

Each component must have tests at several levels:

1. **Unit tests:** local invariants and error handling.
2. **Property-based tests:** general rules over generated inputs, including capability attenuation and canonicalization.
3. **Fuzz tests:** parsers, IPC, guest boundaries, installer manifests and update metadata.
4. **Integration tests:** real component interactions and failure paths.
5. **Fault injection:** process crash, timeout, disk full, corrupted state, power loss, lost network, stale clock and partial write.
6. **Adversarial tests:** malicious applications, compromised AI output, replay, confused deputy, path traversal, data exfiltration and resource exhaustion.
7. **Compatibility tests:** declared app workflows and guest isolation.
8. **Performance tests:** latency, throughput, memory, energy and tail behavior under load.
9. **Recovery drills:** restore from failed update, corrupted configuration, lost credential, or quarantined component.
10. **Independent review:** threat-model review and targeted security assessment before stable release.

## Article 87 — Negative tests are mandatory

For every permit test, the suite must include nearby denial cases: wrong principal, wrong object, wrong verb, expired capability, revoked capability, wrong policy version, wrong device profile, excess budget, malformed envelope, untrusted origin, missing consent, and system mode that disallows the operation. Security is not demonstrated by successful workflows alone.

## Article 88 — Regression corpus

Every security bug becomes a minimized regression test unless doing so would expose a live exploit; in that case, the test is stored in restricted infrastructure until safe disclosure. The project must retain compatibility tests for previously supported workloads and ensure that a compatibility update cannot silently widen host permissions.

## Article 89 — Security claim ledger

The project maintains a public claim ledger. Each claim has a precise statement, applicable profile, assumptions, evidence, known counterexamples, reviewer, last-tested version, and expiry/review date. Claims without current evidence are downgraded or withdrawn. Marketing materials must be generated from or checked against this ledger.

---

# PART XVII — GOVERNANCE, MAINTENANCE, AND LONG-TERM EVOLUTION

## Article 90 — Decision records

Every architecture decision that affects kernel boundaries, capability semantics, compatibility, cryptography, data handling, update trust, licensing, or user consent requires an ADR. An ADR includes context, constraints, alternatives, chosen option, rejected options, threat analysis, performance impact, migration path, tests, and reversal conditions. Decisions are never justified solely by “the AI recommended it” or “it is novel.”

## Article 91 — Dependency admission

A dependency is admitted only after recording: purpose, maintainer, source location, license, security history, build method, update cadence, privilege, network behavior, transitive dependencies, and replacement plan. A new dependency must justify its cost in binary size, attack surface, memory, maintenance and licensing. Critical path components SHOULD minimize dependencies.

## Article 92 — Maintainer succession and key recovery

The project must have a documented process for maintainer succession, signing-key rotation, compromise recovery, and emergency release. No single human or machine key should be sufficient to silently replace all official trust roots. The exact threshold scheme must balance security against permanent lockout and must be tested before production use.

## Article 93 — Deprecation and removal

A feature is removed through a published deprecation window unless immediate removal is required for an actively exploited vulnerability or legal necessity. Deprecation must state affected profiles, migration path, data implications, and last supported version. Security-critical removal may be expedited but must be documented after containment.

## Article 94 — Amendment control

Amendments to constitutional invariants require public rationale, impact assessment, independent review, recorded approval under the governance rules, and new conformance tests. No amendment may retroactively declare an untested security claim proven. Emergency temporary deviations must expire automatically unless ratified through the ordinary process.

---

# PART XVIII — ADDITIONAL ACCEPTANCE CRITERIA

## Article 95 — SCP acceptance tests

The SCP prototype is not accepted until tests demonstrate:

- all privileged calls are mediated by the evaluator;
- caller-supplied risk labels are ignored for authorization;
- missing or malformed capabilities are denied;
- child capabilities cannot exceed parent scope;
- expiry and revocation work under concurrency;
- policy changes invalidate or re-evaluate stale decisions;
- a compromised AI process cannot issue a capability;
- audit records identify actor, operation, policy and outcome without logging secrets;
- SCP unavailability enters a documented mode;
- local enforcement continues when fleet control is disconnected;
- a denied request cannot be made to succeed by alias, encoding, or replay tricks.

## Article 96 — Installer acceptance tests

The installer prototype is not accepted until tests demonstrate:

- invalid or modified artifacts are rejected;
- the target disk is shown unambiguously;
- no destructive write occurs before explicit consent;
- cancellation before commit preserves the original system;
- power loss at each write boundary produces a known recovery state;
- insufficient space is detected before destructive work;
- boot configuration is verified after writing;
- recovery media or partition is not silently overwritten;
- unsupported firmware paths fail with an explanation;
- logs redact secrets and can be exported by user choice.

## Article 97 — UEF acceptance tests

A compatibility environment is not accepted until tests demonstrate:

- host files are inaccessible unless explicitly bridged;
- guest administrator cannot change host policy;
- network egress is denied by default for unknown applications;
- clipboard and device bridges are individually controlled;
- guest crash does not crash unrelated workloads;
- resource exhaustion is contained;
- guest images and adapters are provenance-checked;
- a revoked adapter can be disabled without destroying user data;
- the compatibility label matches the published test evidence.

## Article 98 — Privacy acceptance tests

Privacy acceptance requires network capture and data-flow tests showing that fresh installation emits no nonessential telemetry; local AI mode does not transmit prompts or documents; denied sensor grants are enforced; uninstall follows documented data-retention behavior; diagnostics redact secrets; and consent revocation blocks future data export. Exceptions must be listed by destination, data category and purpose.

## Article 99 — AI acceptance tests

AI acceptance requires adversarial tests for prompt injection, tool confusion, model output schema violations, false success claims, cross-user memory leakage, unapproved data egress, resource exhaustion, malicious retrieval content, model replacement, and unavailable model services. Tests must show that the AI cannot bypass SCP authorization even when it is prompted to do so.

## Article 100 — Release evidence bundle

Each release candidate must publish or securely provide, as appropriate: source revision; reproducible build instructions; SBOM; license report; signed artifacts; key fingerprints; provenance attestation; test results; benchmark methodology; supported hardware profiles; compatibility records; security advisories; known limitations; recovery instructions; and a statement of which constitutional requirements remain unimplemented.

---

# PART XIX — REVISED PROJECT SEQUENCE AND PRACTICAL PRIORITIES

## Article 101 — Build the smallest trustworthy vertical slice

The first prototype should not attempt a desktop, all hardware, every compatibility layer, a full AI assistant, or fleet management at once. It should demonstrate one end-to-end path: verified boot in a simulator; minimal process isolation; one capability-protected object; SCP permit/deny; append-only audit; a signed Change Capsule; a simulated update; failure injection; and recovery to a known state. This vertical slice tests whether the architecture is coherent before the project accumulates complexity.

## Article 102 — Prototype sequence

**P0: Specification and executable model.** Define object IDs, capabilities, request envelope, policy semantics, state machine, error codes, and invariants. Build a simulator and property-based tests.

**P1: Minimal kernel research.** Implement only the minimum boot, memory isolation, process boundary, IPC and timer path in a controlled emulator. Keep all nonessential services out of the kernel.

**P2: SCP and capsule transactions.** Implement deterministic authorization, short-lived capability issuance, idempotency, audit and fault-injected recovery.

**P3: Core local OS.** Add storage, identity, networking, service supervision and a text-based management interface. Verify operation without AI or network access.

**P4: Installer research.** Prototype direct install inside a VM first; simulate disk layout, staging, cancellation and rollback before testing on spare hardware.

**P5: One compatibility environment.** Select one narrow application profile and prove capsule boundaries. Do not claim universal execution.

**P6: AI planner.** Add Intent Contracts and a tool broker only after the SCP has tests proving that a hostile planner cannot exceed its grants.

**P7: Hardware expansion.** Add one architecture/profile at a time with explicit conformance and hardware-specific threat assumptions.

**P8: Fleet and public alpha.** Add authenticated coordination only after local nodes remain secure when disconnected.

## Article 103 — Scope control

Every feature proposal must state: user problem; threat model; minimum viable scope; new dependencies; privilege required; measurable benefit; complexity cost; test plan; failure behavior; and what will be postponed to make room. A feature without a clear owner and acceptance criteria is not scheduled. “Beyond imagination” is a creative ambition, not a waiver of evidence.

---

# PART XX — FINAL CLARIFICATIONS AND RESERVED QUESTIONS

## Article 104 — No universal execution guarantee

Senvoryn's universal execution ambition means a common compatibility architecture and broad support roadmap, not the assertion that every Windows, Linux, Android, iOS, console, embedded, or proprietary application will work with all features. Some software is tied to hardware, secure enclaves, drivers, app-store services, signatures, licenses, cloud backends, or platform-specific APIs. The project must report each limitation honestly and provide a supported alternative where feasible.

## Article 105 — No “perfect constitution” claim

This constitution is intentionally detailed, but it is not final in the sense of being immune to error. Ambiguities discovered by implementers must become tracked issues and clarifications. New attack classes, hardware changes, legal obligations, and implementation evidence may require amendments. The project must prefer a precise, testable correction over defending an earlier claim for reputational reasons.

## Article 106 — Founding commitment

Senvoryn's defining promise is not that the system will never fail. It is that no component is granted unlimited authority by default; failures are designed to be contained; important changes are attributable and verifiable; user data is treated as belonging to the user; AI remains subordinate to enforceable policy; and every public claim is tied to evidence.

The constitution governs the project. The specifications define the contracts. The code implements them. The tests challenge them. Independent reviewers question them. Users decide whether the evidence earns their trust.

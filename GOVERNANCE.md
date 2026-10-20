# Governance

> Normative reference: [Founding Constitution v0.6.0](docs/constitution/SENVORYN_OS_Constitution_v0.6.md) Article 6 (Governance and anti-capture) and Articles 90–94 (Decision records, Dependency admission, Maintainer succession, Deprecation, Amendment control).

Senvoryn's governance exists to make one failure mode impossible: **a single person or company quietly turning a user-owned project into a platform they own**. Everything below is designed to keep authority distributed, decisions public, and claims honest.

---

## 1. Principles

| # | Principle | Practical meaning |
|---|---|---|
| **G-1** | **Public governance** | Design proposals, approved specifications, security advisories, release notes, governance votes, and ADRs are public. Sensitive vulnerability details may be temporarily embargoed through a disclosed process with an expiry and follow-up publication. |
| **G-2** | **Separation of duties** | No single person or organization may simultaneously have unilateral authority to merge protected kernel code, alter release workflow, access all signing keys, produce official artifacts, and publish a trusted release. |
| **G-3** | **Split signing authority** | Release signing keys are separated by role: source/repository administration, build attestation, release authorization, emergency revocation. Online systems hold only narrowly scoped, short-lived keys — never the offline root signing key. |
| **G-4** | **Recorded rationale** | Security-sensitive changes require a recorded rationale, review, testing evidence, and a versioned decision record. Emergency changes are time-limited and reviewed after deployment. |
| **G-5** | **Conflict disclosure** | Reviewers disclose material conflicts of interest. Security-critical code has at least two qualified reviewers, one of whom did not author the change. |
| **G-6** | **Succession readiness** | A documented succession plan exists so that one person leaving cannot make releases impossible, and cannot give another person unchecked control. |
| **G-7** | **Claim discipline** | The Claims Ledger is the single source of truth for what the project may advertise. Marketing and UI may never exceed the ledger. |

---

## 2. Roles

Roles describe **responsibilities**, not people. One human may hold several roles in a small project — but **G-2** still constrains the *combination* of permissions that matter.

| Role | May | May not |
|---|---|---|
| **Contributor** | Propose changes, contracts, tests, evidence | Merge to protected branches; sign releases |
| **Reviewer** | Approve changes in their competence area | Review their own security-critical change alone |
| **Maintainer** | Merge non-protected changes; curate the tree | Amend the constitution; unilaterally authorize a release |
| **Steward** | Sponsor ADRs; chair governance votes; own the roadmap | Silently merge unreviewed changes |
| **Release authority** | Authorize a release after all gates pass | Sign with the root key without multi-party approval |
| **Build authority** | Produce artifacts from pinned inputs in an attested environment | Publish to the release channel |
| **Incident officer** | Coordinate security response, key revocation, advisories | Author both the fix and its independent review |

---

## 3. Decision-making

### 3.1 Normal changes

| Change class | Approval required |
|---|---|
| Typo / formatting / non-normative docs | 1 reviewer |
| Additive schema field, non-critical | 1 reviewer + schema test |
| Behavior change in a contract | 1 reviewer + ADR + tests |
| **Security-critical** (capability enforcement, SCP gates, update verifier, installer, key handling, audit) | **2 qualified reviewers, one non-author + ADR + test evidence** |
| Change to a constitutional invariant | Full amendment process (§5) |

### 3.2 Voting

Where a decision requires a vote, the quorum is a **supermajority of at least 75% of participating eligible votes**. Ordinary votes are open and public; the vote record (proposal, rationale, options, results, dissent) is published.

Voting is not used for routine code review. It is for: constitutional amendments, licensing profile changes, governance changes, and decisions that stewards cannot reasonably settle.

### 3.3 Emergency changes

An emergency change may temporarily reduce scope or disable a feature to protect users.

- It **MUST NOT** weaken an invariant permanently.
- It **expires after 30 days** unless renewed through ordinary review.
- It receives a **public post-incident review**, subject to responsible disclosure.
- It is recorded as a time-limited ADR with an expiry date.

---

## 4. Architecture Decision Records

An ADR is required for any change to architecture, protocol contracts, the object model, the trust model, dependency admission, or licensing policy.

### Required ADR format

```markdown
# ADR-NNN — Title

**Status:** Proposed | Accepted | Superseded by ADR-NNN | Rejected | Deprecated
**Date:** YYYY-MM-DD
**Deciders:** …
**Affected components:** …

## Context
What forces are in play? What problem are we solving? Include the constraint
that makes the obvious solution wrong.

## Decision
What we are doing, stated in the active voice.

## Rationale
Why this and not the alternatives.

## Consequences
What this makes easy, what it makes hard, and what it costs.

## Alternatives considered
Each with the reason it was not chosen.

## Open items
What this decision deliberately leaves unresolved.
```

### ADR rules

1. ADRs are **immutable once accepted**. A change to an accepted decision produces a **new** ADR that supersedes it. History is never rewritten.
2. A **superseding ADR must state what changed and why**, and the old ADR must be updated with a pointer — its body is never edited to remove the original reasoning.
3. Every open question in an accepted ADR must be tracked as an issue until closed.
4. ADRs are numbered sequentially within a single namespace. *(See the known numbering collision noted in [ROADMAP.md](ROADMAP.md#open-decisions-register).)*

---

## 5. Constitutional amendments

The constitution is the top-level authority. Amending it is the heaviest process in the project.

A constitutional amendment requires **all** of:

1. **A public proposal** explaining the problem and the affected invariants.
2. **Threat-model and compatibility analysis.**
3. **A minimum review period of 30 days** for ordinary changes.
4. **Review from maintainers and independent reviewers** for security-sensitive changes.
5. **A supermajority of ≥75% of participating eligible votes**, and approval by at least **two independent stewardship groups** once those groups exist.
6. **A migration plan, test plan, and versioned effective date.**

Emergency amendments are covered in §3.3 and follow the same expiry and post-incident review rules.

---

## 6. Licensing and dependency policy

### 6.1 Default license

Original Senvoryn source code is released under **Apache License 2.0** unless an approved repository-level notice states otherwise. Every file and release identifies its license clearly. Component-level licenses remain separate and are disclosed individually.

### 6.2 Dependency admission

A dependency enters the project only with recorded **provenance, license, version, review, and justification**. "A build script fetched it" is not an admission.

| Class | Requirement |
|---|---|
| **Original design** | A Senvoryn-defined contract, component, algorithmic workflow, or architecture. |
| **Independent implementation** | Code written for Senvoryn to implement a defined contract. |
| **Approved dependency** | External code or a tool with recorded provenance, license, version, review, and justification. |
| **Compatibility guest** | A separately licensed or supplied environment used to run third-party software. |
| **Research prototype** | Unproven work, explicitly excluded from the trusted production base. |

No borrowed source code may be relabeled as original. Every release MUST include a machine-readable **SBOM and license inventory** with component name, version, source, content hash, license expression, copyright notices, modifications, build provenance, and redistributability status. **Unknown or ambiguous licensing is a release blocker for the trusted core.**

### 6.3 Cryptographic dependencies

Senvoryn MUST use publicly reviewed cryptographic designs and MUST NOT invent a cipher, hash function, signature scheme, key exchange protocol, or randomness generator. Implementations may be approved external dependencies when review, provenance, license, maintenance, and vulnerability response are documented.

> A desire for a fully original codebase MUST NOT lead to home-grown cryptography.

### 6.4 Trademark and forkability

The code license and the project name are separate. Forks under the applicable code license are permitted. The official release identity is protected through a transparent trademark policy that **does not prevent security researchers from truthfully identifying modified builds**.

---

## 7. Release governance

| Role separation | Requirement |
|---|---|
| Author | Writes and proposes the change; does not sign its own release |
| Reviewer | Two qualified reviewers for security-critical changes; one non-author |
| Builder | Builds from pinned inputs in an attested environment; cannot publish |
| Authorizer | Authorizes the release after all gates pass |
| Signer | Signs with a role-scoped, short-lived key; root key use needs multi-party authorization, a documented ceremony, and an audit record |

Full release gates: [`RELEASE_PROCESS.md`](RELEASE_PROCESS.md).

### 7.1 What is machine-enforced today

Process rules that are not enforced by tooling are aspirations. These **are** enforced by the repository configuration:

| Control | Enforced by | Status |
|---|---|---|
| CI must pass before merge | Branch protection on `main`, 7 required status checks | ✅ Active |
| No force-push, no branch deletion | Branch protection | ✅ Active |
| Conversations resolved before merge | Branch protection | ✅ Active |
| Dependency changes reviewed for known vulnerabilities and license policy | `dependency-review` CI job on pull requests | ✅ Active |
| Contract artifacts not silently altered | SHA-256 checksum manifest + `pack-integrity` CI job | ✅ Active |
| Claims never outrun the evidence | `claim-discipline` CI job | ✅ Active |
| Two independent reviewers on security-critical changes | — | ⚠️ **Process only.** Not yet enforceable while one steward exists; the `CODEOWNERS` file assigns reviewers but cannot require two |
| Multi-party release signing | — | ❌ Not possible yet — no signing ceremony exists (ADR-013) |
| Separation of duties in the build pipeline | — | ❌ Not possible yet — no build pipeline exists |

> **Write this table honestly.** A control listed as "process only" is a control that depends on everyone remembering. As the project grows, each row should move up a column — and the ones that cannot should be marked ❌ so nobody mistakes them for protections.

---

## 8. Succession and key recovery

- A **documented succession plan** ensures releases remain possible if a maintainer or signer leaves.
- **Key custody and recovery ceremonies** are rehearsed before they are needed — see ADR-013 (open decision).
- The offline root signing key is held by a multi-person ceremony, never by one individual.
- If a release-signing key is suspected compromised, a **recovery trust-root procedure** must exist **before** the event, not after.

---

## 9. Deprecation and removal

- End-of-support dates must be published.
- The system warns before a profile becomes unsupported and explains the security consequences.
- An unsupported version is **never silently labeled secure**.
- Deprecated interfaces retain a documented migration path and a removal notice with a stated deadline.

---

## 10. Known governance gaps

Honest accounting of what is **not** yet solved:

| Gap | Risk | Mitigation in place |
|---|---|---|
| Only one steward currently | Single point of failure for governance decisions | Public decision records; ADR immutability; documented proposal process |
| No independent stewardship groups exist | Constitution §6.4's "two independent groups" requirement cannot be satisfied | Amendment path will need bootstrapping once a second group forms |
| No independent security review yet | No claim may exceed "design-conformant" | Conformance labels enforced in the spec and README |
| Root signing ceremony undefined | Releases cannot be trusted yet | No stable release signing; Gate 8 blocked until ADR-013 closes |
| ADR numbering collision between the constitution register and CCS ADRs | Ambiguity in cross-references | Flagged as a Gate 0 blocker in [ROADMAP.md](ROADMAP.md) |

---

## 11. See also

- [`CONTRIBUTING.md`](CONTRIBUTING.md) — how to propose and land changes
- [`SECURITY.md`](SECURITY.md) — vulnerability disclosure
- [`decisions/`](decisions/) — architecture decision records
- [`RELEASE_PROCESS.md`](RELEASE_PROCESS.md) — release gates and the evidence bundle
- [`evidence/claims-ledger.md`](evidence/claims-ledger.md) — what the project may claim today

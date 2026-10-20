# Release Process

**Status:** Defined, not yet exercised. **No release channel is currently open** —
the repository's contents are published as *Research*.

> Normative reference: [Founding Constitution v0.6.0](docs/constitution/SENVORYN_OS_Constitution_v0.6.md) Article 38 (Delivery, release levels, and lifecycle), Article 33 (Safety cases and claims ledger), Appendix 4 (Minimum release evidence bundle); [Core Contract Specification v0.1.0](docs/specs/SENVORYN_Core_Contract_Spec_v0.1.md) §12.4 (Release gate).

**No release channel is currently open.** This document defines the process that *will* govern releases once one exists. It is published now so that the bar is visible before the work starts.

---

## 1. Release channels

| Channel | Meaning | Status |
|---|---|---|
| **Research** | Incomplete, may break, no stable security promise | ✅ **Current channel** — the repository's contents |
| **Developer Preview** | Selected hardware and features, explicit gaps | ⬜ Not available |
| **Beta** | Feature testing, compatibility matrix, known issue register, limited assurance | ⬜ Not available |
| **Stable** | Required conformance suites pass; critical release pipeline and recovery procedures audited to the declared scope | ⬜ Not available |
| **Long-Term Support** | Narrow supported profile, documented patch window, compatible migration policy, maintained recovery artifacts | ⬜ Not available |

Everything published today is **Research**. The word "research" is not a formality — it is the only honest label for a specification with no implementation.

---

## 2. The release gate

A stable release MUST NOT ship unless **all** of the following hold:

1. ✅ The SBOM and license inventory are complete
2. ✅ Critical build provenance is published
3. ✅ No unresolved release-blocking security issue remains
4. ✅ Capability tests pass
5. ✅ Installation and recovery tests pass for the certified matrix
6. ✅ The Claims Ledger is current
7. ✅ Known limitations are public
8. ✅ Rollback and revocation procedures are rehearsed

> **A stable release also MUST NOT ship unless** the release bundle includes: source revision, SBOM and license report, signed artifacts, build instructions, reproducibility status, threat model, TCB list, test summary, independent review, vulnerability contact, update/rollback evidence, recovery drill, supported profile list, and known limitations.

---

## 3. Minimum release evidence bundle

Every official release must publish — or privately provide to the responsible independent assessor — as appropriate:

| # | Item |
|---|---|
| 1 | Source tag and immutable source revision |
| 2 | Signed release manifest and complete artifact hashes |
| 3 | Build recipe, toolchain pins, dependency locks, SBOM and license inventory |
| 4 | Build provenance and attestations |
| 5 | Reproducibility reports and explained mismatches |
| 6 | Results of unit, integration, property, fuzz, race, resource-exhaustion and fault-injection tests |
| 7 | Capability and SCP invariant test results |
| 8 | Installer and recovery test results for the certified host matrix |
| 9 | Compatibility matrix with feature-level outcomes and limitations |
| 10 | Performance/energy measurements by reference profile |
| 11 | Known issues, threat assumptions, residual risks and advisories |
| 12 | Signing-key status and revocation instructions |
| 13 | Claims Ledger status and reviewer sign-offs |
| 14 | Upgrade/downgrade and user-data migration notes |
| 15 | A supported recovery path and offline documentation |

---

## 4. Roles and separation of duties

| Role | Responsibility | Must not |
|---|---|---|
| **Author** | Writes and proposes the change | Sign its own release |
| **Reviewer** | Two qualified reviewers for security-critical changes; one non-author | Sole-review a security-critical change they authored |
| **Builder** | Builds from pinned inputs in an attested environment | Publish to the release channel |
| **Authorizer** | Authorizes the release after all gates pass | Authorize without a complete evidence bundle |
| **Signer** | Signs with a role-scoped, short-lived key | Use the root key without multi-party approval, a documented ceremony, and an audit record |
| **Incident officer** | Coordinates security response and key revocation | Author both the fix and its independent review |

> **Separation of duties is a hard rule, not a preference.** No single person or organization may have unilateral authority to merge protected kernel code, alter release workflow, access all signing keys, produce official artifacts, and publish a trusted release.

---

## 5. Signing keys

Keys are separated by role:

| Key role | Scope | Custody |
|---|---|---|
| Source/repository administration | Repository permissions, branch protection | Online, scoped |
| Build attestation | Attests build inputs and outputs | Online, short-lived |
| Release authorization | Authorizes a release decision | Online, short-lived |
| Emergency revocation | Revokes a compromised key | Offline, multi-person |
| **Root signing** | Signs release manifests | **Offline, multi-person ceremony, audited** |

**Online systems never hold the offline root signing key.** Root-key use requires multi-person authorization, a documented ceremony, and an audit record.

> The root ceremony is **not yet defined** (open decision ADR-013). Until it exists, no stable release signing is possible — and no artifact in this repository is signed.

---

## 6. Versioning

| Artifact | Scheme | Example |
|---|---|---|
| Repository releases | Semantic Versioning | `v0.1.0` |
| Specification | `MAJOR.MINOR.PATCH` with a document ID | `SENVORYN-CORE-SPEC-0001` v0.1.0 |
| Constitution | `MAJOR.MINOR.PATCH` | v0.6.0 |
| JSON Schemas | Semantic major/minor in `schema` field | `senvoryn.request-envelope.v1` |
| ADR | Sequential, immutable | ADR-001 |
| Denial codes | Stable strings — **never renamed**, only added | `RISK_CLAIM_MISMATCH` |

**Stable identifiers are a contract.** A denial code, a conformance test ID (`SCP-001`), or an ADR number, once published, is never silently changed. Corrections are additive and documented.

---

## 7. Release procedure (when a channel opens)

1. **Freeze.** Cut the release branch. No unreviewed changes land after the freeze.
2. **Verify.** Full CI green on the frozen revision; pack checksums regenerated and verified.
3. **Reproduce.** Critical components built by at least two independently administered build environments from pinned inputs. Differences must be explained before the affected artifact receives the highest trust tier.
4. **Assemble.** Build the evidence bundle (§3). Every item must be present or explicitly marked *not applicable* with a reason.
5. **Review.** Two qualified reviewers confirm the bundle; at least one did not author the change.
6. **Sign.** Build attestation → release authorization → manifest signing. Each step recorded.
7. **Publish.** Tag, release notes, SBOM, license inventory, claims ledger snapshot, known limitations.
8. **Announce.** Release notes state exactly what changed, what is now claimed, and what is *not* claimed.
9. **Verify the release.** Independent re-download and checksum verification.

---

## 8. Rollback and revocation

- A rollback of system configuration **must not** automatically roll back security revocations, delete incident evidence, or reverse external effects. **Security state and operational state have separate rollback rules.** **[INV-11]**
- Rollback is available to a prior bootable generation on a named reference profile within the published target (< 120 s, excluding user-data restoration and physical repair).
- **Recovery must be possible** without the normal network controller and without depending solely on the damaged active generation.
- Revocation is immediate for new privileged actions; in-flight operations stop at the next safe boundary and enter a documented cancellation/compensation flow.

---

## 9. End of support

- End-of-support dates must be published.
- The system warns before a profile becomes unsupported and explains the security consequences.
- An unsupported version is **never silently labeled secure**.

---

## 10. Current status

| Item | Status |
|---|---|
| Open release channel | ✅ Research (repository contents only) |
| Signed artifacts | ❌ None — no signing ceremony exists |
| SBOM | ❌ Not yet generated for a release |
| Reproducible build | ❌ Not yet established |
| Independent review | ❌ None |
| Threat model | 🟡 First draft, unreviewed (`docs/threat-model.md`) |
| Claims ledger | ✅ Live at [`evidence/claims-ledger.md`](evidence/claims-ledger.md) |
| Recovery drill | ❌ Not applicable — no implementation |

---

## See also

- [`GOVERNANCE.md`](GOVERNANCE.md) — roles, voting, amendments
- [`evidence/claims-ledger.md`](evidence/claims-ledger.md) — the claims a release may carry
- [`SECURITY.md`](SECURITY.md) — advisory and incident process
- [`ROADMAP.md`](ROADMAP.md) — gate 8 requirements

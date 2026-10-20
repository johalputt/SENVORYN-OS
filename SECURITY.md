# Security Policy

**Status:** Active. Reporting channel live. No release channel is open, so no
artifact currently requires a security advisory.

> Normative reference: [Founding Constitution v0.6.0](docs/constitution/SENVORYN_OS_Constitution_v0.6.md) Article 32 (Incident response and vulnerability handling) and Article 33 (Safety cases and claims ledger).

Senvoryn treats security reporting as a **core function**, not a formality. The project is at the specification stage, so most findings today will be in contracts, schemas, the reference model, or the documentation itself — and those are exactly the findings worth having early.

---

## 🔒 Report a vulnerability privately

**Please do not open a public issue, pull request, or discussion for a security vulnerability.**

### Primary channel — GitHub Private Vulnerability Reporting

1. Go to **Security → Report a vulnerability** on the repository page:
   <https://github.com/johalputt/SENVORYN-OS/security/advisories/new>
2. Or use the `gh` CLI:
   ```bash
   gh api -X POST repos/johalputt/SENVORYN-OS/private-vulnerability-reporting \
     -f title="<short summary>" \
     -f body="<details, reproduction, impact>"
   ```
3. GitHub will open a private advisory thread visible only to you and the maintainers.

This channel is **confidential**: reports are not public until a coordinated advisory is published.

### What to include

| Field | Why it matters |
|---|---|
| **What you found** | The specific contract, schema, state transition, code path, or claim that is wrong |
| **How to reproduce** | Steps, inputs, or a minimal test. For schema findings, the offending JSON is ideal |
| **The invariant at risk** | Which of INV-01 … INV-30, or which Core Contract Spec section, is violated. If none, say so — that is useful information too |
| **Impact** | What an attacker, a misconfigured host, or an AI agent could achieve |
| **Suggested fix** | Optional, but very welcome. A proof-of-concept patch is even better |
| **Your disclosure preference** | Whether you want credit, anonymity, or to coordinate before publication |

### What to expect

| Step | Target |
|---|---|
| Acknowledgement | Within **5 business days** |
| Initial triage (impact, exploitability, privilege boundary, data sensitivity, affected profiles, availability) | Within **10 business days** |
| Public advisory or fix plan | Per the severity band below |

---

## Severity and response

Severity is determined by **impact, exploitability, privilege boundary, data sensitivity, affected profiles, and availability** — not by a fixed calendar. A critical defect in the core, capability enforcement, signing, installer, or update path **can block releases**.

| Severity | Definition | Response target |
|---|---|---|
| **Critical** | Bypasses a constitutional invariant; yields host or authority compromise; breaks signing, update, or installer trust | Immediate triage; release-blocking; embargoed until a fix and advisory exist |
| **High** | Violates a required contract behavior with a realistic path; breaks isolation between principals or capsules | Fix or documented mitigation before the affected gate can exit |
| **Medium** | Weakens a defense-in-depth layer; contract ambiguity with a plausible unsafe interpretation | Scheduled fix with a tracking issue and a stated deadline |
| **Low** | Documentation, hardening, or defense-in-depth improvement with no direct security consequence | Normal backlog |
| **Informational** | Defense-in-depth suggestion, or a claim that overstates what the evidence supports | Reviewed; may become a doc change or a claims-ledger correction |

---

## Scope

### In scope

- The Core Contract Specification and its machine-readable schemas — **including ambiguity that permits an unsafe reading**
- State machine transition tables — illegal or unsafe transitions
- The denial-code registry and default failure behavior
- The reference SCP model — logic errors that would misrepresent the specified behavior
- Architecture Decision Records and governance documents where they contradict a constitutional invariant
- Anything in the repository that could mislead a user or implementer about the system's guarantees
- **Overstated claims** — if the docs say something stronger than the evidence supports, that is a security finding

### Currently out of scope (no implementation exists yet)

- Kernel, bootloader, installer, or update-path vulnerabilities — **these components do not exist yet**
- Cryptographic implementation flaws — the reference model deliberately uses trusted booleans
- Denial of service against a non-existent running service
- Vulnerabilities in third-party dependencies (report those upstream — we will track them here)
- Reports generated solely by an automated scanner with no demonstrated security consequence
- Missing hardening that has no documented threat model behind it

> **Out of scope does not mean ignored.** Findings about absent components are tracked as **design requirements** and feed the threat model. A report that identifies a missing control before implementation is one of the most valuable kinds we can receive.

---

## Safe harbor

Senvoryn welcomes good-faith security research. If you comply with the following, we will not pursue legal action against you:

- Use only test accounts, test devices, and test environments you own or are authorized to use
- Do not access, modify, or delete data belonging to others
- Do not degrade availability, exfiltrate data, or persist beyond the minimum needed to demonstrate the finding
- Do not require the project to promise confidentiality for a report you intend to publish immediately
- Give us a reasonable opportunity to fix the issue before public disclosure
- Do not misrepresent your findings or claim impact you did not demonstrate

**We will not pursue legal action for research that follows this policy.** We will not retaliate against reporters, and we will not pursue a reporter who has acted in good faith and cooperated with a reasonable disclosure timeline.

---

## Coordinated disclosure

The default is **coordinated disclosure**: we publish an advisory and a fix together, and credit the reporter unless they ask otherwise.

**Embargo.** If a fix cannot ship immediately, we may ask for a short embargo. Embargoes have a **published expiry date** and are not open-ended. If the embargo expires without a resolution, we publish what we know.

**Timeline.** High and Critical findings are targeted for a fix within 30 days of confirmation. If a fix will take longer, we say so publicly, explain the mitigation, and keep the advisory updated.

**No blame concealment.** We publish a post-incident report describing affected versions, impact, root causes to the extent known, mitigations, test additions, remaining uncertainty, and the next review date. Sensitive exploit details may be delayed where necessary for remediation, with a documented reason and review date.

---

## What we need from you

The single most useful thing: **tell us which invariant you believe is broken, and show the path from a permitted action to the violation.** The Senvoryn model is adversarial by design — the interesting findings are the ones where an unprivileged actor reaches a privileged effect, or where a *denial* silently becomes a *permit*.

Highest-value targets in the current repository:

1. **Schema ambiguity** — a request envelope that can be parsed two ways with different authority outcomes
2. **State-machine escapes** — a transition path that reaches `COMMITTED` without passing `VALIDATED` and `AUTHORIZED`
3. **Gate-order violations** — a decision that can be produced before a mandatory gate runs
4. **Attenuation gaps** — a child capability that could be broader than its parent
5. **Claim overstatements** — documentation promising more than the evidence supports

---

## Public advisories

Published advisories live in the repository's **Security → Advisories** tab and are announced in the release notes. Past advisories are never deleted — they remain indexed and searchable.

---

## Signing-key compromise

If a release-signing key is suspected compromised, we will:

1. Suspend the affected update channel
2. Invoke the recovery trust-root procedure
3. Publish affected-version inventory and a user notification
4. Preserve forensic evidence — **a rollback does not erase incident evidence**
5. Publish a post-incident report

> The recovery trust-root procedure is **not yet defined** (open decision ADR-013). Until it is, no stable release signing key exists — there is nothing to compromise yet. This is a tracked gap, not an oversight.

---

## See also

- [`GOVERNANCE.md`](GOVERNANCE.md) — roles, separation of duties, signing authority
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — how to contribute safely
- [`evidence/claims-ledger.md`](evidence/claims-ledger.md) — what the project currently claims
- [`docs/threat-model.md`](docs/threat-model.md) — the SCP decision-path threat model
- [`RELEASE_PROCESS.md`](RELEASE_PROCESS.md) — release gates and the evidence bundle

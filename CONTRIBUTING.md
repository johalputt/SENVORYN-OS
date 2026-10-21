# Contributing to SENVORYN OS

**Status:** Active. The project is at the specification and model stage; see
[`ROADMAP.md`](ROADMAP.md) for the active gate.

Thanks for your interest in Senvoryn. This project is at the **specification and model** stage, so contributions are weighted toward **contracts, executable models, tests, and evidence** rather than large code volumes.

If you want to change an invariant, widen authority, or add a cryptographic primitive, read [`GOVERNANCE.md`](GOVERNANCE.md) first — those changes have a much heavier path than everything else.

---

## Table of contents

- [Ways to contribute](#ways-to-contribute)
- [Before you start](#before-you-start)
- [The scope-control template](#the-scope-control-template)
- [Development setup](#development-setup)
- [Running the tests](#running-the-tests)
- [Repository conventions](#repository-conventions)
- [What a good pull request looks like](#what-a-good-pull-request-looks-like)
- [Review rules](#review-rules)
- [Writing an ADR](#writing-an-adr)
- [Adding or changing a schema](#adding-or-changing-a-schema)
- [Adding a conformance test](#adding-a-conformance-test)
- [Claims and evidence](#claims-and-evidence)
- [Code of conduct](#code-of-conduct)
- [Security issues](#security-issues)

---

## Ways to contribute

| Contribution | Where it lands | Difficulty |
|---|---|---|
| **Threat modeling** — attack the SCP decision path, capability model, capsule boundary | `docs/threat-model.md`, `evidence/` | High value, no code needed |
| **Property-based tests** — attenuation never broadens authority; denied requests create no effects | `tests/` | High value |
| **Fuzz corpora** — malformed, truncated, oversized, duplicate-key, deeply nested inputs | `tests/fuzz/` | High value |
| **Adversarial AI corpora** — prompt injection, tool-call laundering, memory poisoning | `tests/adversarial/` | High value |
| **Spec and schema work** — tighten contracts, close ambiguity | `docs/specs/`, `schemas/` | High value |
| **Reference model improvements** — deterministic reason codes, canonical parsing | `reference/` | Medium |
| **Tooling** — schema linting, canonicalization, traceability checks | `scripts/`, `.github/` | Medium |
| **Documentation** — glossary, examples, onboarding | `docs/` | Accessible entry point |
| **Implementation** — separation core, SCP, capsule runtime | `src/` (does not exist yet) | High; needs Gate 1+ |

> **Highest-leverage contribution right now:** the SCP threat model and the ambiguity review. Both are Gate 0 blockers, and both are pure analysis work — no implementation required.

---

## Before you start

1. **Read the constitution.** [Founding Constitution v0.6.0](docs/constitution/SENVORYN_OS_Constitution_v0.6.md) is the top-level authority. Thirty constitutional invariants (INV-01 … INV-30) are supreme and may be refined but never weakened.
2. **Read the core contract.** [Core Contract Specification v0.1.0](docs/specs/SENVORYN_Core_Contract_Spec_v0.1.md) converts those principles into data contracts, state machines, validation order, failure behavior, and testable obligations.
3. **Check the roadmap.** [`ROADMAP.md`](ROADMAP.md) shows which gate is active and what is actually blocking it.
4. **Open an issue first** for anything beyond a trivial fix. Use the issue templates in `.github/ISSUE_TEMPLATE/`.

---

## The scope-control template

Every feature proposal MUST state the following. A feature without a clear owner and acceptance criteria is not scheduled.

```markdown
## Feature proposal — <title>

**User problem:** What cannot a user do today?

**Threat model:** What goes wrong if this is implemented incorrectly?

**Minimum viable scope:** What is the smallest version that solves the problem?

**New dependencies:** What new external code, tool, or protocol does this add?
(Each needs provenance, license, version, review, justification.)

**Privilege required:** What authority does this need, and why is it needed?

**Measurable benefit:** How will we know it worked?

**Complexity cost:** What does this make harder to build, review, or maintain?

**Test plan:** Which test IDs does this add or satisfy?

**Failure behavior:** What happens when it fails, is attacked, or is interrupted?

**Deferred:** What will be postponed to make room for this?
```

---

## Development setup

**Requirements:** Python 3.11+ (3.12 and 3.13 are exercised in CI). No other runtime dependency for the contract layer.

```bash
git clone https://github.com/johalputt/SENVORYN-OS.git
cd SENVORYN-OS

python -m venv .venv
# Linux/macOS
source .venv/bin/activate
# Windows PowerShell
.venv\Scripts\Activate.ps1

pip install -r requirements-dev.txt
```

### Repository layout

| Path | Contents |
|---|---|
| `docs/specs/` | Core Contract Specification + the original pack README |
| `docs/constitution/` | Founding Constitution (baseline authority) |
| `schemas/` | JSON Schemas (Draft 2020-12) — 14 files |
| `state-machines/` | Machine-readable transition tables |
| `errors/` | Stable denial-code registry |
| `examples/` | Valid request fixture, unsigned operation-registry sample |
| `reference/` | Deterministic toy SCP evaluator |
| `tests/` | Executable contract tests |
| `decisions/` | Architecture Decision Records |
| `evidence/` | Traceability matrix, validation report, claims ledger |
| `scripts/` | Local verification helpers |

---

## Running the tests

```bash
# Linux/macOS
PYTHONPATH=reference python -m unittest discover -s tests -v

# Windows PowerShell
$env:PYTHONPATH = "reference"
python -m unittest discover -s tests -v
```

Expected: `Ran 108 tests ... OK`.

Run a single module or test:

```bash
PYTHONPATH=reference python -m unittest tests.test_scp_reference -v
PYTHONPATH=reference python -m unittest tests.test_scp_reference.SCPReferenceTests.test_unknown_principal_denied -v
```

### Run the Rust workspace

```bash
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

### Break every control on purpose

A control that has never been seen to fail on the thing it forbids has proved
nothing. The mutation gate breaks each control in turn and requires a
**named** test to go red:

```bash
python scripts/mutation-gate.py                    # all 139 controls
python scripts/mutation-gate.py M11-risk-downgrade # one control
python scripts/mutation-gate.py --list
```

A surviving mutation means a control no test enforces, and the gate fails.

**If you add a gate, add its mutation in the same commit.** The rule that makes
the gate meaningful is that a mutation must be caught by a *specific* test and
a *specific* case — a mutation caught by the wrong one proves nothing. The three
failure modes to avoid are:

1. **A test that calls the function under test to judge its output** compares the
   code with itself.
2. **One seed tripping several rules** kills a mutation of none of them.
3. **Two guards for one outcome** are individually unkillable by an outcome-only
   assertion. Assert the specific reason code, not "it failed".

The manifest lives in `scripts/mutations.json`. It is covered by the checksum
manifest, so deleting a line from it is a visible change.

**Do not pipe the gate's output through a truncating filter.** A pipeline that
closes early kills the process while a file is mutated, and the tree is left
broken. The gate writes the original beside the file before it changes
anything, and refuses to start if the tree is not in the state the manifest
expects — but the repair is easier to avoid than to make.

**A Python mutation needs the bytecode cache cleared.** CPython invalidates a
cached `.pyc` by the source's size and its modification time in whole
seconds, so a mutation that leaves the size unchanged within the same second
is read from the cache and never runs. The gate would then report a control
as untested when the control is fine — or credit a kill to a mutation that
did nothing. The gate removes the caches and runs the interpreter with
`-B`, and it does so for every Python mutation. If you add a Python control
and the gate does not catch it, check this first.

**A mutation in the gate must be in the language of the file it breaks.** The
runner picks the test runner from the mutated file's extension, so a `.rs`
file is checked by a Rust test and a `.py` file by a Python one. The
manifest does not carry the language, and cannot carry it wrongly.

### Verify the shipped pack

```bash
bash scripts/verify-pack.sh        # POSIX
pwsh -File scripts/verify-pack.ps1 # Windows
```

### What CI checks

| Job | Rule |
|---|---|
| `contract-tests` | All contract tests pass on Python 3.12 **and** 3.13 |
| `rust` | The workspace is formatted, `clippy -D warnings` clean, tested, and builds in release mode |
| `conformance-agreement` | The Rust and the Python implementations agree on every shared vector — the SCP's and the capsule's alike |
| `mutation-gate` | Every control is broken in turn and a named test is required to go red |
| `schema-validation` | Every schema is valid Draft 2020-12; fixtures conform; unknown fields are rejected |
| `pack-integrity` | SHA-256 checksums match `SHA256SUMS.txt` |
| `json-lint` | Every JSON document in the repo parses |
| `normative-language` | Spec documents declare a status and contain no unmarked absolute security claims |
| `link-check` | Every relative Markdown link resolves |

**All nine must pass before a pull request can be merged.**

> **Known gap.** Only the seven contract-pack checks are currently listed as
> *required status checks* in the branch-protection rule. The three Rust-facing
> jobs run on every push and fail the build, but they are not yet required
> contexts, so a direct push to `main` is not blocked by them. The check names
> to add are exactly the job names above.

---

## Repository conventions

### Normative language

In specifications, ADRs, and governance documents, use the normative keywords exactly as defined:

| Word | Meaning |
|---|---|
| **MUST** / **MUST NOT** | Binding requirement for a conforming implementation |
| **SHOULD** / **SHOULD NOT** | Strong requirement; deviation requires a written rationale and review |
| **MAY** | Optional capability that must not undermine a MUST |
| **TARGET** | Measurable engineering goal; not a guarantee until benchmarked on a defined reference profile |
| **EXPERIMENTAL** | Disabled by default; must not be relied upon for safety or security claims |
| **PROHIBITED** | Forbidden by the constitution, even if convenient or benchmark-improving |

### Status labels

Every document must declare its status explicitly:

`Proposed` · `Under review` · `Accepted` · `Superseded` · `Withdrawn`

Any statement not yet demonstrated by implementation and test evidence MUST be labelled **proposed**, **unverified**, or **research required**.

### Style

- Markdown for prose, JSON for machine-readable contracts, Python for the reference model and tests.
- JSON files are 2-space indented, UTF-8, LF line endings, trailing newline.
- Schema `$id` values use the `senvoryn.*` namespace; bump the schema's `major` version for breaking changes and document a migration path.
- Prefer **precise, testable corrections** over defending an earlier claim for reputational reasons.

---

## What a good pull request looks like

1. **References an issue** and states which gate it advances.
2. **States the invariant impact** — explicitly: "no invariant weakened" or names the one affected and why.
3. **Comes with tests.** A behavior change without a test is a behavior change without evidence.
4. **Updates traceability.** If it adds or changes a requirement, update `evidence/traceability.csv`.
5. **Adds an ADR** if it changes architecture, protocol contracts, the object model, the trust model, dependency admission, or licensing.
6. **Keeps the claims honest.** If it changes what the project can claim, update `evidence/claims-ledger.md`.
7. **Updates the checksum manifest** if it touches a shipped contract artifact (`scripts/verify-pack.*` will tell you).

Use the pull request template at `.github/PULL_REQUEST_TEMPLATE.md`.

---

## Review rules

| Change class | Reviewers required |
|---|---|
| Typo, formatting, non-normative docs | 1 |
| Additive non-critical schema field | 1 + schema test |
| Behavior change in a contract | 1 + ADR + tests |
| **Security-critical** | **2 qualified, one non-author** + ADR + test evidence |

Security-critical means: capability enforcement, SCP gates, update verification, installer, key handling, audit, or privilege boundaries.

**Reviewers must disclose material conflicts of interest.** If you authored a change, you may not be its sole reviewer on a security-critical path.

---

## Writing an ADR

Copy the format in [`GOVERNANCE.md` §4](GOVERNANCE.md#4-architecture-decision-records) or start from an existing file in `decisions/`.

An ADR is accepted when it has a decision, a rationale, consequences, alternatives considered, and open items. **ADRs are immutable once accepted** — to change an accepted decision, write a new ADR that supersedes it.

---

## Adding or changing a schema

1. Place it in `schemas/<name>.schema.json`.
2. Use JSON Schema **Draft 2020-12** with `$id` in the `senvoryn.*` namespace.
3. Signed or security-relevant objects MUST use `additionalProperties: false` so unknown fields are rejected. (An existing test asserts this for the request envelope.)
4. Mark unknown **optional** fields as noncritical if they may be ignored; unknown **critical** fields must cause rejection.
5. Add a fixture in `examples/` and a test in `tests/test_schema_contracts.py`.
6. Bump the schema version. Breaking changes require a new **major** version and a documented migration path.

---

## Adding a conformance test

Conformance test IDs (`SCP-001`, `CAP-001`, `CC-001`, `OC-001`, `AI-001`, `INS-001`, `UPD-001`, `PRIV-001`, `FED-001`, `EVID-001`) are a **stable contract**. Detailed implementations may be split by language or environment, but the IDs do not change.

To add a test:

1. Pick the next unused ID in the relevant group.
2. Add it to the traceability table in the Core Contract Specification §16 — **and keep the ID stable from then on**.
3. Implement the test in `tests/`.
4. Add a row to `evidence/traceability.csv` with evidence type, priority, and honest status.
5. Note in `evidence/claims-ledger.md` if it changes what may be claimed.

> **A passing unit test is not evidence of the absence of all vulnerabilities.** State the scope of what the test proves.

---

## Claims and evidence

The project maintains a **Claims Ledger** classifying each claim as:

`PROPOSED` · `IMPLEMENTED` · `TESTED` · `INDEPENDENTLY REVIEWED` · `FORMALLY VERIFIED WITH SCOPE` · `WITHDRAWN`

**A feature's documentation and marketing must not advertise a stronger status than the ledger shows.** If your change makes a claim stronger, update the ledger — and if it makes a claim *weaker* (a regression, a withdrawn review), update it too.

---

## Code of conduct

Participation is governed by [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). By participating you agree to uphold it.

---

## Security issues

**Do not open a public issue or PR for a vulnerability.** Follow [`SECURITY.md`](SECURITY.md) for the private reporting channel, triage criteria, and coordinated disclosure process.

---

## License

By contributing, you agree that your contributions are licensed under the **Apache License 2.0** — the project's default license for original code. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE). If your contribution requires a different license, raise it in an issue **before** submitting the code.

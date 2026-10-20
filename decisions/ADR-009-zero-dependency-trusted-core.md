# ADR-009 — No Third-Party Dependencies in the Trusted Core

**Status:** Proposed
**Date:** 2026-10-20
**Deciders:** Project maintainers
**Affected components:** `crates/senvoryn-canonical`, `crates/senvoryn-scp`, `crates/senvoryn-cli`

---

## Context

- **Problem:** The Rust workspace has to do three things: parse canonical JSON, decide
  requests, and offer a development tool. The obvious way to build those is to reach
  for `serde`, `serde_json`, `clap`, and a random-number or hash crate.
- **Constraint:** The trusted core of an operating system is the code whose failure
  compromises the system. Every dependency in it is code that must be audited, that
  can carry a vulnerability, and that arrives by a route — a package registry — which
  is not under this project's control. The project's own threat model treats the
  verification and enforcement path as the thing that must not be weakenable by
  accident.
- **Assumptions:** The functionality the workspace needs from those crates is small
  enough to write. Continuous integration can install a toolchain without a network
  fetch of a package.

## Decision

We will write the Rust workspace with **zero third-party dependencies**. The only
crates in the dependency graph are the three of this workspace.

This means writing, and taking responsibility for, a canonical JSON parser and
encoder, a command-line argument reader, and the data structures those need. Where a
cryptographic primitive would be needed, the trusted core does not implement it: it
produces the canonical bytes and compares digests that the caller supplies. See
[ADR-002](ADR-002-crypto-boundary.md).

## Rationale

The dependency graph of the trusted core is part of its security argument. A crate
that is audited once and then pinned by checksum is a different thing from a crate
that is re-downloaded at every build, and the difference is invisible until the day it
matters.

The counter-argument is real: hand-written parsers are a classic source of
vulnerabilities, and the specification's own rule that a component is not a control
unless it is tested applies to this one more than to any other. That is why the
decision is not "write our own parser and trust it" but "write our own parser and
test it against a deterministic fuzz harness, a written profile, and a clause-by-clause
conformance suite."

The same argument does not apply to a hash function. A hand-written hash is not a
weaker version of a library hash; it is a different, unanalysed function, and there is
no reason for this project to have one.

## Consequences

**What this makes easy:**
- The trusted core's audit surface is the source in this repository, and it is small.
- Continuous integration is hermetic: it needs a toolchain and nothing else, so a
  compromised or unavailable package registry cannot break the build or change what
  the build produces.
- There is no version drift to reconcile between two copies of a dependency.

**What this makes hard:**
- Every line of parsing and encoding code is this project's code to own, to test, and
  to fix.
- The development tool's argument handling is homegrown and therefore worse than a
  real library's. This is acceptable, because it is a development tool and not in the
  trusted path.

**What this costs:**
- Time that would have gone into integrating existing libraries.
- A standing obligation to keep the fuzz harness and the conformance suite green, in
  exchange for the assurance they provide.

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Use `serde` and `serde_json` | The most obvious choice, and the one this decision rejects. It puts a general-purpose parser and a derive-driven data model in the trusted core, and `serde_json`'s number handling does not implement the profile in ADR-008 without custom code anyway. |
| Use `serde_json` outside the trusted core only | It was considered and it is a reasonable position. It was not chosen because the argument reader and the CLI are the only places it would apply, and neither is worth a dependency. |
| Use a hash crate for the digest over canonical bytes | [ADR-002](ADR-002-crypto-boundary.md) already forbids inventing a primitive, and using a library one inside the core would be an admission that the core does hashing. The core produces bytes; the caller hashes them. |
| Depend on a JSON Schema validator for the conformance tests | The schema tests are not in the trusted path, and the project already has a Python `jsonschema` dependency for them. |

## Open items

- [ ] Whether a *non*-trusted component — a tool, a test harness, a documentation
      generator — may take dependencies under a stated rule — what evidence would close
      it: a component that needs one and has no route to writing the code itself.
- [ ] Whether the canonical parser should be reviewed by a second party before it is
      treated as a control — what evidence would close it: the fuzz harness and the
      conformance suite passing, plus an independent read of `crates/senvoryn-canonical`.

## References

- [ADR-002 — Cryptographic Boundary](ADR-002-crypto-boundary.md)
- [ADR-008 — Canonical JSON Profile `SCP-CJSON-0001`](ADR-008-canonical-json-profile.md)
- [`docs/threat-model.md`](../docs/threat-model.md)
- [`crates/`](../crates/) — the workspace this decision applies to

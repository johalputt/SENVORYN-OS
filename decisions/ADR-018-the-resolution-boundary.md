# ADR-018 — The Resolution Boundary: A Name Is Not an Identity

**Status:** Proposed
**Date:** 2026-10-21
**Deciders:** Project maintainers
**Affected components:** `crates/senvoryn-scp`, `reference/scp_reference.py`, `conformance/vectors/vectors.json`, `crates/senvoryn-cli`, `schemas/decision-record.schema.json`, `errors/denial-codes.json`

---

## Context

- **Problem:** Gate 6 — the object-resolution gate — proved only that a name
  was *known*. `PolicyContext.resolved_resources` was a flat map from a name
  to a version string. It carried no type, and it produced no canonical
  identity. Three things followed from that, and all three were reachable:

  1. **The caller's own type claim decided the risk.** Gate 11 asked the
     registry whether the operation was defined for `request.resource_type`,
     and gate 10 compared the capability's scope with `request.resource_type`.
     Both are the caller's strings. A caller that asserted a permissive type
     for an object the registry treats strictly was judged by the permissive
     rule: the registry's `resource_types` set is what assigns the risk, and
     the caller was choosing the set.
  2. **The capability's scope was compared with the caller's strings.** A
     grant for one object could be aimed at another by asserting the
     identifier differently, because the identifier in the comparison was the
     one the caller wrote.
  3. **The artefacts that authorise an effect named the caller's string.** The
     decision record's `target_resource_id` and the audit event's
     `resource_id` were copied from the request. The record that says "this
     action is authorised" named an identifier the caller chose.

- **Constraint:** The specification fixes both the requirement and the order.
  §1.2: "Every resource MUST have a canonical typed identity, owner,
  sensitivity label, lifecycle state, supported operation set, version/epoch
  where applicable, and policy domain. Path strings alone are insufficient
  identifiers for security decisions. Resolution MUST occur before
  authorization and be resistant to aliasing, link traversal, namespace
  confusion, and time-of-check/time-of-use races." §2.3: "aliases MUST be
  resolved to canonical IDs before decision evaluation." §3.2 gate 6:
  "Object resolution gate: resolve the resource to a canonical ID and version."

- **Constraint:** The SCP does not resolve resources. It is a pure decision
  function — no filesystem, no resolver, no trust store — and its own module
  documentation says so. The environment knows what objects exist; the SCP
  judges what the environment resolved.

- **Constraint:** The gate order is normative. Resolution is gate 6, after
  identity (gate 4) and freshness (gate 5) and before the capability gate
  (gate 10). Moving it earlier would let a request that is not even
  authenticated learn what the node can resolve.

- **Already known:** The threat model carries this as an open risk — AS-19 and
  AS-20, High likelihood against Medium impact, with the note "Resolution
  design not yet specified per resource type — **open risk**". The row was
  open because the design had not been decided, not because the risk had not
  been understood.

- **Already modelled:** `schemas/resource-record.schema.json` states the
  canonical typed identity the specification asks for: `resource_id`,
  `resource_type`, `owner_id`, `sensitivity`, `lifecycle_state`,
  `supported_operations`, `policy_domain`, `version`. The resolution record
  decided here is that identity narrowed to the three fields the decision
  path needs.

## Decision

**A resolved resource is a record of a canonical identifier, a canonical
type, and a version.** The environment's resolution table maps *the name a
caller used* to that record. The name may be an alias; the canonical
identifier is the truth, and the table is the environment's, not the
caller's.

**Gate 6 resolves the name and binds the type.**

- A name that resolves to nothing is denied with `RESOURCE_UNRESOLVABLE`. It
  is not dereferenced, and it is not guessed at.
- A name that resolves to an object whose canonical type is not the type the
  request claimed is denied with `RESOURCE_TYPE_MISMATCH`. The claim is
  **checked, not corrected**: a request that misdescribes its own target is
  either confused or hostile, and the boundary does not decide which. The
  type claim in the envelope is checked against the environment's knowledge,
  exactly as the risk claim is checked against the registry's tier.

**The canonical identity, and never the caller's claim, governs every later
gate.** Gate 10 compares the capability's resource scope with the *canonical*
identifier and the *canonical* type. Gate 11 asks the registry whether the
operation is defined for the *canonical* type. A caller's claim cannot widen
the reach of an operation, cannot aim a grant at another object, and cannot
be aimed at a laxer rule.

**A decision names the object it was about.** The decision carries the
resolution it made, and the decision record and the audit event name the
canonical identifier rather than the caller's string. The record also carries
the resolved version, so that the executor's recheck has an anchor when it
exists.

**The version is part of the resolution and is recorded, not checked.** The
request envelope carries no target version, so there is nothing yet to
compare the resolved version against. Recording it is what makes the
resolution a resolution rather than a lookup: the specification asks gate 6
for an identifier *and a version*, and a version that is looked up and
discarded is not a version.

## Rationale

**A name is not an identity.** The specification says it in §1.2 and the
threat model says it in AS-20: "Path strings alone are insufficient
identifiers for security decisions." Before this decision the whole decision
path was built on the caller's string, in three places at once, and the
resource's *type* — the thing the registry keys its rules by — was never
read from anywhere the environment had written it. The gate was called
*object resolution* and it resolved to a version string.

**The caller's type claim is a claim, and claims are cross-checked.** §3.1
already says the treatment of a caller-provided classification: "treated as a
claim and cross-checked", and the risk claim is refused outright when it is
lower than the registry's tier. A type is not a risk tier and has no
ordering, so there is no "higher" claim to allow; any disagreement is a
disagreement. Correcting the claim silently would be worse than refusing it:
the caller would proceed believing it is addressing a file when the node
knows it is addressing a device, and the caller's own handling would be
wrong even though the node's authorisation was right.

**The canonical type is what the registry's rules are keyed by.** §3.1.1
says the registry is "keyed by canonical resource type". Reading the key
from the request made the key a caller-controlled input, which is the
opposite of what a key is for.

**The capability's scope is a grant over an object, not over a string.** A
grant names a resource and a type. Comparing that grant with a caller's
string means the caller decides which object the grant is about. Comparing it
with the canonical identity means the grant is about the object the
environment resolved, which is the only reading under which an alias is
usable at all: the alias resolves to the canonical object, and the grant
names the canonical object, so they match — while a grant for a *different*
object still does not.

**The record has to name the object the effect will be applied to.** The
decision record is the artefact that authorises an effect, and gate 16 makes
the executor recheck authorisation immediately before acting. An executor
that reads a record naming a caller's string has nothing to recheck against:
the string is the caller's to change. The canonical identifier and the
version are the anchor, and the version is what a time-of-check/time-of-use
race has to be detected against.

## Consequences

**What this makes easy:**
- The caller's type claim becomes safe to ignore, because it is checked at
  the boundary instead of obeyed at three later gates.
- Aliases work. A request may name an object by any name the environment
  resolves, and the grant still has to be about the canonical object.
- The threat model's open risk on resolution has a design to point at, and
  the resource record schema has a consumer.

**What this makes hard:**
- The context's resolution table changes shape, so every implementation that
  builds one changes with it: the Rust CLI, the Rust test harnesses, the
  Python reference model, the Python test harness, and the shared vectors.
- A reason code has to be added, and the denial-code registry, the Rust
  constants, the Python set and the two registry-sync tests move together.
- The decision record gains a field, so the decision-record schema is
  amended.

**What this costs:**
- This ADR.

**What is implemented, and what is not:**

| Part | State | Where |
|---|---|---|
| The resolved-resource record and the resolution table | done | `crates/senvoryn-scp/src/resolution.rs`, `PolicyContext::resolved_resources` |
| The type cross-check at gate 6 | done | the decision path, with `RESOURCE_TYPE_MISMATCH` and a shared vector |
| The canonical identity at gates 10 and 11 | done | the decision path, with a shared vector per gate |
| The canonical identifier in the decision record and the audit event | done | `Decision::to_record`, `build_audit_event` |
| The resolved version in the decision record | done | `Decision::to_record`, `schemas/decision-record.schema.json` |
| The executor's recheck of the resolved version (gate 16) | done | ADR-020: the authorization names the version the decision was made against, and the executor refuses a decision that names none or names another one |

## Alternatives considered

| Alternative | Why it was not chosen |
|---|---|
| Keep comparing the caller's `resource_type` at gates 10 and 11 | That is the defect. The caller would choose the registry rule that applies, and with it the risk tier and the consent requirement. |
| Ignore the caller's type claim instead of refusing a mismatch | Correcting the claim silently leaves the caller acting on a model of the target that the node knows to be wrong, and it throws away the only signal that the two disagree. Refusing is the fail-closed reading, and it matches how the risk claim is already treated. |
| Compare the caller's claim at gates 10 and 11 but check it at gate 6 | The two are then the same comparison, and the code at gates 10 and 11 would read as if the caller's string were authority. The canonical value is what is compared; the cross-check at gate 6 is what makes the claim and the canonical value agree. |
| Resolve the resource inside the SCP | The SCP is a pure decision function with no filesystem and no trust store. It is also the wrong owner: the environment is what knows the objects, the aliases and their current versions. |
| Put the canonical identity in the request envelope and read it from there | That is the caller's claim with a better name. The envelope is the caller's document; the resolution table is the environment's. |
| Use one reason code for "the name is unknown" and "the type does not match" | It would close an existence oracle that is already open, and it would cost the operator the difference between a stale request and a name that was never there. The two failures are different and an operator needs to tell them apart. The oracle is bounded and pre-existing; see the open items. |
| Deny a request whose *identifier* is not canonical, so that aliases are refused | The specification requires aliases to be resolved to canonical identifiers before the decision, not refused. A caller that uses an alias has done nothing wrong, and refusing the alias would push the alias into the protocol layer, where it cannot be checked against anything. |
| Add the canonical type to the denial, so the caller can correct itself | It would disclose the object's type to a caller that guessed the name. The caller may retry with another type; the type space is small and closed, but the disclosure is not needed to make the refusal actionable. |

## Open items

- [x] The request envelope carries no target version, so the resolved version
      cannot be compared with anything the request says. The executor's
      recheck of the version — the time-of-check/time-of-use anchor AS-19 asks
      for — is implemented (ADR-020): the authorization names the version the
      decision was made against, and the executor refuses a decision that
      names none or names another one. What is left: the envelope still
      carries no target version, and nothing yet ties the request to the
      plan — the plan digest is that tie, and it is ADR-020's open item.
- [ ] The resolution design *per resource type*. The table models a canonical
      identifier, a type and a version. Whether a resource type needs more of
      the identity the resource-record schema states — owner, sensitivity,
      lifecycle state, supported operations — is a separate decision, and the
      threat model's open row stays open until it is made.
- [ ] Who may define an alias, and how an alias is revoked. An alias in the
      table is a second name for an object, and a stale alias is a way to
      reach an object under a name the operator has stopped using.
- [ ] The existence oracle. A caller that has passed the identity gate can
      tell a name that resolves from one that does not, by the reason code.
      The exposure is bounded — identity precedes resolution, the reasons name
      no resource the caller did not name, and the type space is a closed
      enum — and it is pre-existing: before this decision the same caller
      could tell an unknown name from a known one by the difference between
      `RESOURCE_UNRESOLVABLE` and any later gate's reason. What would close
      it: a response policy that collapses the resolution failures, which is
      the node's decision to make and not the evaluator's.

## References

- [`crates/senvoryn-scp/src/resolution.rs`](../crates/senvoryn-scp/src/resolution.rs)
- [`crates/senvoryn-scp/src/evaluator.rs`](../crates/senvoryn-scp/src/evaluator.rs)
- [`schemas/resource-record.schema.json`](../schemas/resource-record.schema.json)
- [`schemas/decision-record.schema.json`](../schemas/decision-record.schema.json)
- [`docs/threat-model.md`](../docs/threat-model.md) — AS-19 and AS-20
- [ADR-011 — the resolution reason code](ADR-011-resolution-reason-code.md)
- [ADR-016 — the Operation Registry's limits, epoch and freshness](ADR-016-operation-registry-limits-epoch-and-freshness.md)

# ADR-019 — The Policy Request Form: One Name, Two Documents

**Status:** Proposed
**Date:** 2026-10-22
**Deciders:** Project maintainers
**Affected components:** `crates/senvoryn-scp/src/request.rs`, `crates/senvoryn-scp/tests/envelope.rs`, `crates/senvoryn-scp/tests/fuzz.rs`, `schemas/request-envelope.schema.json`, `examples/cli/request.json`, `scripts/mutations.json`

---

## Context

- **Problem:** The repository had two documents called a request, and only one
  of them was written down.

  1. **The wire envelope** is the one the specification describes and the
     schema defines (`schemas/request-envelope.schema.json`). It carries
     date-times, a `replay_key` distinct from the nonce, a `target_node_id`,
     the caller's `data_classification_claim` and `external_effects_claim`,
     `capability_ids`, `approval_refs`, `payload_digest` and a
     `correlation_id`. It is closed: `additionalProperties` is `false`.
  2. **The policy request** is the one the reader actually reads
     (`Request::from_value`). It carries the same names for the identity
     fields, but the times are monotonic integers, and it has none of the
     wire envelope's other fields — and it has one the wire envelope does not
     define, `approval_digest`.
  3. **The reader's documentation said the second was the first.** "The
     document is the request envelope with its times already expressed in
     monotonic milliseconds" was true of the times and false of everything
     else. A reader that misdescribes its own input cannot state its
     contract.
  4. **The CLI's example declared a schema it did not satisfy.**
     `examples/cli/request.json` carried `"schema":
     "senvoryn.request-envelope.v1"` while missing two required fields and
     carrying the times in the wrong type. Nothing checked it, so the
     repository's own example of a request would have been rejected by the
     repository's own schema.
  5. **The form was closed in name only.** The reader read the fields it knew
     and ignored the rest, so a document could carry any field at all. The
     specification is the other way: §0.3 "Unknown optional fields may be
     ignored only when the schema explicitly marks them as noncritical", and
     §3.2 gate 2 "Canonical parse gate: reject ambiguous encoding, duplicate
     fields, unsupported critical fields, and invalid types". A field the
     node does not understand may be a field a newer sender expects the node
     to enforce.
  6. **The boundary was untested.** The Gate 1 checklist's "envelope→request
     fuzzing" was open, and none of the reader's controls had a mutation.

- **Constraint:** The SCP is a pure decision function. The reader is the
  boundary where untrusted bytes become a request the gates will judge, and
  a gate cannot judge a request that was read wrongly.

- **Constraint:** The transport layer that would produce a policy request
  from a wire envelope — converting the date-times, carrying the replay key
  across, deciding what to do with the caller's claims — does not exist yet.
  That is the "time model's transport layer" open item, and it is blocked on
  the boot-epoch bridge ADR-017 left open.

- **Already modelled:** the wire envelope's schema and its valid fixture
  (`examples/valid-request-envelope.json`), which the contract tests check
  against the schema. The drift was between that schema and the reader, not
  inside the schema.

## Decision

**The policy request form is a document in its own right: the transport
layer's output, not the wire envelope.** The two are related — the policy
form is the wire envelope, narrowed to what the policy layer reads and with
its times expressed in monotonic milliseconds — but they are not the same
document, and the reader's contract is the policy form's.

**The form is closed.** A document carrying a field the form does not define
is refused, and the refusal says which field. The same holds for the
budget, which is the form's only nested object. The reader cannot know which
fields a newer sender might send, so the rule is not "ignore what you do
not know" but "refuse what you cannot account for": an unknown field may
carry a demand the node is expected to enforce, and silently ignoring it
would be a decision to enforce a contract nobody wrote down.

**The reader enforces the form and nothing else.** It checks that the
document is an object, that its field set is closed, that the required
fields are present and non-empty, that the numbers are non-negative
integers, and that the risk claim is a tier this node knows. It does not
judge: an operation this node has never heard of, a resource it cannot
resolve and a principal it does not know are all read successfully, and the
gates that follow refuse them with their own reason codes.

**A refusal at the reader is not a decision.** The request never existed, so
there is nothing to decide and no decision to record. The denial-code
registry is the vocabulary of *decisions*, and a code in it that no decision
can carry is a promise the registry cannot keep — which is why
`UNSUPPORTED_CRITICAL_SCHEMA` and `ANTI_ROLLBACK_VIOLATION` are listed as
open items rather than quietly used as if they were produced.

**The boundary is fuzzed.** A deterministic harness, in the shape the
canonical parser's harness already established, mutates a corpus of policy
requests and asserts three things: the reader never panics, a request that
was read says exactly what the document said, and the same document is read
the same way twice. The second is the property that matters most: a reader
that changes a value while reading it is a reader whose decision is not the
caller's decision.

## What is implemented, and what is not

| Item | State |
|---|---|
| The policy request form's field set is closed, top level and budget | Implemented — `crates/senvoryn-scp/src/request.rs` |
| The reader refuses a document that is not an object | Implemented |
| The reader refuses an unsupported schema | Implemented |
| The reader refuses a missing, empty, or wrongly typed required field | Implemented |
| The reader refuses a negative number and a non-integer number | Implemented |
| The reader's controls are mutation-tested (nine mutations) | Implemented — `scripts/mutations.json` |
| The boundary is fuzzed | Implemented — `crates/senvoryn-scp/tests/fuzz.rs` |
| A transport layer that produces a policy request from a wire envelope | **Not implemented** — the boot-epoch bridge is undecided (ADR-017) |
| A schema for the policy request form | **Not implemented** — the form is described by the reader and its tests |
| The policy form carries the wire envelope's `replay_key` | **Not implemented** — the form has a nonce only |
| The policy form carries the wire envelope's `target_node_id` | **Not implemented** — the form has a recipient only |
| The policy form carries the caller's classification and effect claims | **Not implemented** — §3.1 requires a caller's classification to be cross-checked, and there is nothing to cross-check |
| The policy form's `approval_digest` has a home in the wire envelope | **Not implemented** — the wire envelope has `approval_refs` and `payload_digest` instead |
| The CLI's example satisfies a schema | **Not implemented** — the example is a policy request and the wire schema describes the wire envelope |

## Rejected alternatives

1. **Give the reader the wire envelope and convert inside it.** The reader
   would then need the boot epoch to turn a wall-clock date-time into a
   monotonic one, which is the conversion ADR-017 deliberately left
   undecided, and it would put a wall clock in the policy layer's input.
   Rejected: the conversion belongs to the transport, and the transport is a
   separate component with a separate trust story.
2. **Read the wire envelope's `issued_at` and `expires_at` as date-times and
   compare them with the wall clock.** The freshness gate would then be
   decided by an instrument that can go backwards. Rejected: ADR-017.
3. **Let the reader ignore unknown fields, and let the schema decide what is
   non-critical.** The reader does not consult a schema, so nothing would
   decide. Rejected: the form is closed, and the closure is stated in one
   place.
4. **Add a second schema for the policy request form and validate against it
   in the reader.** A schema validator is a large component, and the
   repository's zero-dependency rule puts it in the trusted core. The form's
   contract is a field list, a set of type rules, and the tests that enforce
   them. Rejected *for now*: if the form grows fields that the transport
   must fill in, a schema will earn its place.
5. **Move the reader's refusals into the decision path so they carry
   registry codes.** A document that cannot be read is not a request, and a
   decision about a request that does not exist is a fiction. Rejected: the
   registry is the decision vocabulary, and the reader's refusals are not
   decisions.
6. **Delete the policy form and read the wire envelope everywhere, accepting
   the wall-clock comparison.** Rejected as option 2, and it would also give
   the policy layer a document it cannot check.

## Open items

1. **The transport layer.** What converts a wire envelope into a policy
   request, what it does with the caller's classification and effect claims,
   and how it carries the replay key. Blocked on the boot-epoch bridge.
2. **The replay key.** The wire envelope distinguishes `nonce` from
   `replay_key`; the policy form has one value, and the freshness gate reads
   it as a nonce. Whether the two are the same value is a question the
   transport has to answer, and it is a security question: a replay key that
   is not the nonce is a value the replay check does not see.
3. **The target node.** `recipient_id` and `target_node_id` are both
   required by the wire schema. The policy form has one field. Which one it
   corresponds to — and what the gate 3 check compares — is undecided.
4. **The claims.** §3.1 requires a caller's classification and effect claims
   to be cross-checked against policy. The policy form carries neither, so
   there is nothing to cross-check, and the audit event's classification
   comes from the environment instead.
5. **`approval_digest`.** The policy form binds an approval by digest; the
   wire envelope carries `approval_refs` and `payload_digest`. Which is the
   approval binding, and where the digest is computed, is undecided.
6. **`UNSUPPORTED_CRITICAL_SCHEMA` and `ANTI_ROLLBACK_VIOLATION`.** Two
   codes in the denial-code registry that no decision can carry. The first
   names the reader's refusal, which is not a decision; the second names the
   updater's downgrade refusal, and there is no updater. Both stay in the
   registry because the registry is stable and adding is allowed while
   removing is not, but their status should be visible to anyone reading it.

## Consequences

- The reader has a stated contract, nine mutations and a fuzz harness, so a
  change to it is a change that fails something.
- The policy form and the wire envelope are now two different documents with
  two different contracts, and the gap between them is listed rather than
  implied. The drift that produced the CLI's unsatisfiable example is
  recorded in the open items instead of being carried forward silently.
- The reader is a closed form, so a sender that adds a field breaks a node
  that has not been taught the field. That is the intended direction of
  failure: a node that refuses a document it does not understand is safe,
  and a node that ignores a field it does not understand is not.

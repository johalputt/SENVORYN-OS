# ADR-002 — Cryptographic Boundary

**Status:** Proposed

## Decision
Use reviewed, widely analyzed cryptographic primitives behind a small replaceable interface. Do not invent ciphers, signature schemes, hash functions, key exchange or randomness. Specify algorithm allowlists, domain separation, key custody, rotation, revocation, recovery, and downgrade behavior before release signing is implemented.

## Consequences
Cryptographic choices remain provisional until an expert review chooses the suite and supported hardware profiles. A valid signature authenticates origin/integrity under key assumptions; it does not prove code correctness or absence of backdoors.

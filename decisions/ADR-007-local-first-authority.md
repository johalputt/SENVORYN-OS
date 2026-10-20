# ADR-007 — Local-First Authority

**Status:** Proposed

## Decision
Every node retains local policy enforcement. A fleet controller can request a state change but cannot bypass local capability checks, mode restrictions, expiry, or risk-based approval.

## Consequences
Offline nodes operate under bounded cached policy; the maximum stale-authority window must be documented. Fleet-wide policy changes need versioning, staged rollout, revocation, and split-brain handling.

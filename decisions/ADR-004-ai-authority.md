# ADR-004 — AI Is a Proposal Producer, Not an Authority

**Status:** Proposed

## Decision
Models run outside the enforcement core. They produce typed Intent/Action Envelopes and invoke tools only through an independent broker. Every effect is re-authorized by local SCP policy.

## Consequences
The system can operate with AI disabled. Tool access, model memory and remote inference are separately governed. Prompt text and confidence scores never constitute authorization.

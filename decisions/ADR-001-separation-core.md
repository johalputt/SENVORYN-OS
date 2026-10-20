# ADR-001 — Clean-sheet Separation Core

**Status:** Proposed for prototype only

## Decision
Build an original, minimal separation core as a research program. Start with one emulated CPU profile, a tiny object model, memory protection, bounded IPC, scheduling, and capability invocation. Do not implement broad device, storage, networking, UI or AI functionality in the core.

## Rationale
A new OS has the largest risk in its privileged core and hardware support. A narrow first target makes the trust boundary reviewable and permits executable invariants before adding features.

## Consequences
The first prototype is not broadly useful, not production-ready, and not evidence of portability. Device support expands only when the architecture and conformance tests expand.

## Open decisions
Initial ISA; boot contract; interrupt routing; memory model; unsafe-code policy; formal proof scope; debugging and recovery access.

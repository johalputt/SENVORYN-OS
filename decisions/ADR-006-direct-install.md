# ADR-006 — Direct Installation

**Status:** Proposed

## Decision
Support an application-like installer without user-created boot media on certified host configurations. Preserve a separately documented recovery route when firmware or host restrictions make the direct path unavailable.

## Consequences
The installer needs host-specific integration, boot entry management, signed staged artifacts, target-disk consent, power-loss tests and recovery evidence. One-click does not mean silent disk destruction or zero-reboot operation.

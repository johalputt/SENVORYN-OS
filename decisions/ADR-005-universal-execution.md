# ADR-005 — Universal Execution Fabric

**Status:** Proposed

## Decision
Use a layered execution fabric: native formats, managed runtimes, API/instruction translation and isolated guest environments. Select the simplest execution mode that delivers tested semantics and a sufficient isolation boundary.

## Consequences
There is no guarantee of universal compatibility. Claims are per application version, adapter, runtime, hardware profile and test-suite version. Proprietary platform services, hardware entitlements and licensing restrictions may prevent some workloads.

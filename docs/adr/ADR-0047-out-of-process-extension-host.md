# ADR-0047 — Out-of-Process Extension Host

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Third-party extensions may crash, hang, overuse resources, parse untrusted data, or contain malicious code.

Loading them directly into the engine would expose authoritative state and critical media paths.

---

## Decision

Third-party extension runtime code will execute outside the engine process in supervised extension-host processes.

---

## Consequences

### Positive

- crash isolation;
- capability enforcement;
- resource quotas;
- restart and quarantine;
- no direct engine memory access.

### Negative

- IPC overhead;
- data-plane complexity;
- more process management;
- some low-latency features require carefully designed bridges.

---

## Alternatives Considered

### In-process dynamic libraries

Rejected because crashes and memory corruption would affect the engine.

---

## Related Specifications

- `07-sdk/701-extension-architecture.md`
- `07-sdk/706-sandboxing-and-resource-limits.md`
- `07-sdk/715-extension-failure-isolation-and-diagnostics.md`

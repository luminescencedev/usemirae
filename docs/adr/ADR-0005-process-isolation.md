# ADR-0005 — Process Isolation

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Mirae combines:

- user interface;
- real-time media engine;
- platform capture;
- third-party extensions;
- update logic;
- crash reporting;
- potentially unstable device and codec integrations.

Running all components in one process would simplify early development but allow UI, extension, or device failures to terminate active production and would weaken privilege boundaries.

---

## Decision

Mirae will use a multi-process target architecture with explicit logical boundaries from the beginning.

Expected process roles:

- desktop shell and control UI;
- engine runtime;
- extension host;
- crash handler;
- updater;
- optional isolated media or device workers.

Early releases MAY combine roles when necessary, but communication must remain representable through stable interfaces so extraction does not require redesigning domain ownership.

---

## Consequences

### Positive

- fault isolation;
- extension safety;
- UI restart without immediate engine termination;
- privilege separation;
- independent updater;
- better crash attribution;
- ability to isolate unstable device integrations.

### Negative

- IPC complexity;
- protocol versioning;
- distributed lifecycle;
- state synchronization;
- debugging complexity;
- additional memory overhead.

---

## Alternatives Considered

### Single process

Rejected as the target architecture because it creates unacceptable failure coupling.

### Separate process for every subsystem

Rejected because excessive boundaries increase latency and operational complexity without proportional benefit.

---

## Implementation Notes

- Engine owns authoritative state.
- IPC is typed and versioned.
- Queues are bounded.
- Session and generation identifiers are mandatory.
- UI reconnect requires snapshot support.
- Extension host receives capabilities only.
- Updater does not replace running binaries unsafely.
- Crash handler remains outside monitored processes.

---

## Related Specifications

- `00-foundations/004-system-overview.md`
- future `01-runtime/101-process-model.md`
- future `01-runtime/108-ipc-protocol.md`
- future `07-sdk/702-extension-host.md`

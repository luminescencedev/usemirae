# ADR-0001 — Native Rust Core

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Mirae requires:

- predictable memory and ownership;
- native platform integration;
- multithreaded media processing;
- low-overhead abstractions;
- safe concurrency;
- long-lived maintainability;
- cross-platform builds;
- access to modern graphics and media ecosystems.

A browser-hosted engine would couple critical media behavior to web runtime constraints. C or C++ would provide native access but increase memory-safety risk and implementation burden.

---

## Decision

Mirae's engine, runtime, rendering coordination, media orchestration, audio, persistence, IPC services, platform adapters, and critical tooling will be implemented primarily in Rust.

Foreign libraries MAY be used through contained adapters or FFI.

The control UI MAY use TypeScript and React, but it will not own engine state or media execution.

---

## Consequences

### Positive

- explicit ownership and lifetimes;
- strong type system;
- memory safety for most code;
- good concurrency primitives;
- native performance;
- portable workspace and tooling;
- strong suitability for typed domain boundaries.

### Negative

- steeper learning curve;
- some platform APIs require unsafe FFI;
- multimedia ecosystem coverage may require C libraries;
- compilation time and binary size require management.

---

## Alternatives Considered

### C++

Rejected as the primary language because memory safety and large-scale consistency costs are higher.

### Electron/Node engine

Rejected because critical media, rendering, and platform integration would depend on a browser-oriented runtime.

### Swift or C# primary core

Rejected because neither provides the same cross-platform systems foundation for all target platforms.

---

## Implementation Notes

- Unsafe code must be isolated and documented.
- Third-party native libraries remain behind Rust-owned interfaces.
- Domain crates must avoid platform and toolkit types.
- Clippy, formatting, dependency audit, and unsafe review are required.
- Panic behavior across FFI must be controlled.

---

## Related Specifications

- `00-foundations/001-project-overview.md`
- `00-foundations/003-design-principles.md`
- future `06-quality/602-memory-model.md`
- future `05-platform/500-platform-overview.md`

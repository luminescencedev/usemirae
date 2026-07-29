# ADR-0032 — Native Capture APIs per Platform

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Screen, window, application, audio, and device capture behavior differs significantly by operating system, privacy model, compositor, and driver stack.

A lowest-common-denominator library may hide important capabilities and failure reasons.

---

## Decision

Mirae will use platform-native capture backends behind one Mirae capture abstraction.

Fallback backends may exist and must report limitations.

---

## Consequences

### Positive

- best available performance and permissions integration;
- zero-copy opportunities;
- accurate platform behavior;
- explicit fallbacks.

### Negative

- more platform-specific implementation;
- greater testing matrix;
- behavior differences must be normalized carefully.

---

## Alternatives Considered

### One third-party cross-platform capture library as the domain API

Rejected because it would own semantics and obscure platform-specific privacy and interop.

---

## Related Specifications

- `03-media/302-capture-system.md`
- `05-platform/502-windows-platform.md`
- `05-platform/503-macos-platform.md`
- `05-platform/504-linux-platform.md`
- `05-platform/505-platform-capture-abstraction.md`

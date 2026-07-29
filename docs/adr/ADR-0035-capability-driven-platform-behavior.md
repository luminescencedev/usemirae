# ADR-0035 — Capability-Driven Platform Behavior

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Operating-system family alone does not determine whether capture, hardware encoding, HDR, portals, secure storage, or update features are usable.

---

## Decision

Mirae will use a generation-stamped capability registry with structured status, limitations, requirements, and source evidence.

UI and domain behavior will query capabilities instead of scattering OS checks.

---

## Consequences

### Positive

- accurate feature availability;
- better diagnostics;
- packaging and permission awareness;
- easier support for new OS versions and drivers.

### Negative

- probe and cache complexity;
- capability schema maintenance;
- asynchronous refresh behavior.

---

## Alternatives Considered

### Compile-time OS checks

Rejected because they cannot represent runtime environment, permissions, devices, or packaging.

---

## Related Specifications

- `05-platform/514-platform-capability-registry.md`
- `05-platform/500-platform-overview.md`

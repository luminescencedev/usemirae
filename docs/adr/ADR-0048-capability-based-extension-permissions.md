# ADR-0048 — Capability-Based Extension Permissions

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Installed extensions should not automatically gain access to projects, media, files, networks, credentials, devices, or UI.

---

## Decision

Extensions will receive explicit, scoped, revocable capabilities.

Manifest declarations request capabilities but do not grant them.

---

## Consequences

### Positive

- least privilege;
- clear permission UX;
- revocation;
- auditable access;
- reduced blast radius.

### Negative

- permission complexity;
- broker APIs;
- more extension failure states;
- grant migration on updates.

---

## Alternatives Considered

### Full trust after installation

Rejected because installation is not informed authorization for every resource.

---

## Related Specifications

- `07-sdk/705-permission-and-capability-model.md`
- `06-quality/612-security-model.md`

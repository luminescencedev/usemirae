# ADR-0034 — Signed Staged Updates

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

An updater can replace high-trust native binaries and therefore is a critical security boundary.

Installing directly from downloaded bytes or relying only on TLS is insufficient.

---

## Decision

Mirae updates will use signed metadata, verified packages, isolated staging, process shutdown coordination, installed-result validation, and rollback metadata.

---

## Consequences

### Positive

- strong update integrity;
- safer interruption handling;
- rollback support;
- auditable release identity.

### Negative

- signing infrastructure;
- platform-specific installer behavior;
- more release complexity;
- key management requirements.

---

## Alternatives Considered

### Download and replace binaries in process

Rejected because it is unsafe and difficult to roll back.

### TLS-only trust

Rejected because transport security does not replace package authenticity.

---

## Related Specifications

- `05-platform/509-updates-packaging-and-signing.md`
- `01-runtime/101-process-model.md`

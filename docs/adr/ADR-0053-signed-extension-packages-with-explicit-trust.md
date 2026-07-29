# ADR-0053 — Signed Extension Packages with Explicit Trust

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Extension packages contain executable or interpreted logic and schemas.

The host must verify integrity and publisher identity while keeping trust separate from permission.

---

## Decision

Production extension packages will be integrity-manifested and signed.

Unsigned packages require explicit developer mode or local trust.

Signature never auto-grants capabilities.

---

## Consequences

### Positive

- package integrity;
- publisher identity;
- revocation;
- safer updates;
- auditable source.

### Negative

- signing infrastructure;
- publisher onboarding;
- key rotation/revocation;
- developer-mode UX.

---

## Alternatives Considered

### Unsigned arbitrary packages

Rejected for normal production installation.

---

## Related Specifications

- `07-sdk/702-extension-manifest.md`
- `07-sdk/713-extension-packaging-signing-and-distribution.md`

# ADR-0044 — Explicit Threat Model and Capability Security

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Mirae processes untrusted files, media, network data, deep links, extensions, and update packages while holding credentials and device access.

---

## Decision

Mirae will maintain an explicit threat model and use least-privilege capabilities across process, extension, filesystem, network, credential, and device boundaries.

---

## Consequences

### Positive

- clearer security ownership;
- smaller blast radius;
- auditable extension behavior;
- safer updates and imports.

### Negative

- permission UX;
- sandbox complexity;
- more explicit API design;
- security review overhead.

---

## Alternatives Considered

### Trust local processes and extensions by default

Rejected because local code and files can still be malicious or compromised.

---

## Related Specifications

- `06-quality/612-security-model.md`
- future `07-sdk/705-permission-model.md`
- future `07-sdk/706-sandboxing.md`

# ADR-0040 — Structured Error Taxonomy

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Free-form error strings make retry behavior, UI presentation, diagnostics, and automation inconsistent.

---

## Decision

Mirae will use stable machine-readable error codes, categories, severity, retryability, safe messages, correlation IDs, and redacted context.

---

## Consequences

### Positive

- consistent recovery;
- localizable UI;
- better diagnostics;
- safer redaction;
- testable error behavior.

### Negative

- taxonomy maintenance;
- mapping native/toolkit errors;
- more explicit code.

---

## Alternatives Considered

### Raw strings and vendor errors

Rejected because they are unstable, unsafe for users, and difficult to act on.

---

## Related Specifications

- `06-quality/605-error-model.md`

# ADR-0033 — Operating-System Secure Credential Stores

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Streaming and service integrations require secrets. Plain configuration and project files are inappropriate.

---

## Decision

Mirae will store credentials in supported operating-system secure stores and persist only non-secret credential references.

No plaintext fallback is allowed.

---

## Consequences

### Positive

- secrets separated from projects;
- OS access controls;
- safer bundles and backups;
- easier rotation/redaction.

### Negative

- backend availability differs;
- migration between machines is not automatic;
- secure-store prompts/locking must be handled.

---

## Alternatives Considered

### Encrypted file with application-owned static key

Rejected because key protection would be weak and platform security would be bypassed.

### Plaintext configuration

Rejected.

---

## Related Specifications

- `05-platform/508-secure-credential-storage.md`
- `04-project/401-project-format.md`

# ADR-0052 — Host-Mediated Files, Network, and Credentials

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Direct unrestricted access to the filesystem, network, and credentials would make extension permissions broad and difficult to audit.

---

## Decision

Extensions will use host brokers for file selection/storage, declared network access, and credential use.

Raw secret delivery is exceptional and requires stronger capability.

---

## Consequences

### Positive

- enforceable scope;
- credential isolation;
- network/domain policy;
- quotas;
- audit and redaction.

### Negative

- broker implementation effort;
- some advanced integrations need additional APIs;
- potential performance overhead.

---

## Alternatives Considered

### Give extension normal OS filesystem/network access

Rejected because it bypasses capability enforcement and sandbox policy.

---

## Related Specifications

- `07-sdk/705-permission-and-capability-model.md`
- `07-sdk/711-extension-storage-settings-and-secrets.md`

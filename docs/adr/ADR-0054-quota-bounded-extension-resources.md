# ADR-0054 — Quota-Bounded Extension Resources

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Extensions can overload CPU, memory, IPC, storage, logs, media queues, or UI even without violating explicit permissions.

---

## Decision

Every extension resource class will have host-enforced quotas, deadlines, metrics, and escalation policy.

---

## Consequences

### Positive

- bounded denial of service;
- shared-host fairness;
- diagnosable abuse;
- predictable memory and queue behavior.

### Negative

- accounting overhead;
- policy tuning;
- extensions must handle throttling and rejection.

---

## Alternatives Considered

### Rely on extension developer discipline

Rejected because mistakes and malicious behavior remain possible.

---

## Related Specifications

- `07-sdk/706-sandboxing-and-resource-limits.md`
- `07-sdk/715-extension-failure-isolation-and-diagnostics.md`

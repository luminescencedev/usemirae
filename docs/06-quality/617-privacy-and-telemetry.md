# 617 — Privacy and Telemetry

**Status:** Proposed  
**Audience:** Security, product, diagnostics, legal/release contributors  
**Canonical:** Yes  
**Required context:** `607-observability-and-diagnostics.md`, `608-crash-reporting.md`, `612-security-model.md`  
**Related ADRs:** ADR-0046

---

## 1. Purpose

This document defines privacy-minimized diagnostics, telemetry consent, data categories, retention, user controls, and local-first defaults.

---

## 2. Default

Core Mirae operation does not require telemetry.

Usage and diagnostic telemetry are opt-in unless a future legal/product policy explicitly changes the decision through review and ADR.

Local logs and health data remain on device by default.

---

## 3. Data Categories

### Allowed with consent and minimization

- app version;
- platform class;
- capability status;
- aggregated performance metrics;
- crash signature;
- feature counters;
- reason-coded failures;
- anonymized rollout cohort.

### Excluded by default

- project names;
- scene/source names;
- media content;
- recordings;
- stream keys;
- tokens;
- full file paths;
- window titles;
- browser content;
- extension private data;
- arbitrary user text.

---

## 4. Consent

Consent must be:

- informed;
- specific enough;
- reversible;
- separate from mandatory terms where practical;
- respected across versions;
- not required for core app use.

Crash report upload may be a separate choice from usage analytics.

---

## 5. Data Minimization

Before collection, ask:

1. Is this needed?
2. Can it be aggregated locally?
3. Can identifiers be removed?
4. Can retention be shorter?
5. Can sampling reduce volume?
6. Can user inspect it?

---

## 6. Identifiers

Avoid stable cross-context identifiers.

If a pseudonymous installation ID is required:

- generated locally;
- resettable;
- not derived from hardware serial;
- not linked to project identity;
- scoped to declared purpose.

---

## 7. Retention

Each dataset defines:

- purpose;
- fields;
- retention;
- access;
- deletion;
- processor;
- region where relevant;
- security controls.

Local logs and crash files have separate bounded retention.

---

## 8. User Controls

Users can:

- enable/disable categories;
- inspect a sample payload;
- delete local diagnostic data;
- reset pseudonymous ID;
- choose crash upload per incident where supported;
- export privacy settings.

---

## 9. Extension Telemetry

Extensions may not inherit Mirae telemetry consent automatically.

Extension network and analytics behavior requires:

- declared capability;
- extension privacy disclosure;
- user approval;
- host enforcement where possible.

---

## 10. Telemetry Transport

Transport requirements:

- TLS;
- authenticated endpoint;
- bounded payload;
- retry limits;
- no blocking critical path;
- no secret headers in logs;
- offline queue bounded and expiring.

---

## 11. Invariants

1. Core operation works without telemetry.
2. Telemetry is opt-in.
3. Media and secrets are excluded.
4. Identifiers are purpose-scoped and resettable.
5. Offline queues are bounded.
6. Consent is reversible.
7. Extension telemetry is separate.
8. Retention is defined.
9. Users can inspect/delete local data.
10. Telemetry collection never blocks production.

---

## 12. Required Tests

- telemetry disabled;
- consent change;
- payload schema;
- excluded-field scan;
- ID reset;
- offline queue bound;
- retry expiration;
- crash consent separation;
- extension telemetry denial;
- payload inspection UI;
- deletion;
- transport redaction.

---

## 13. AI Implementation Notes

Do not add analytics events without updating the telemetry schema and consent category.

Do not include user-authored text, paths, media, or credentials.

Do not make telemetry a prerequisite for project, recording, or streaming functionality.

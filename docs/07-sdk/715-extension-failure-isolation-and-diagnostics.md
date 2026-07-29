# 715 — Extension Failure Isolation and Diagnostics

**Status:** Proposed  
**Audience:** SDK, runtime, support, security contributors  
**Canonical:** Yes  
**Required context:** `701-extension-architecture.md`, `06-quality/607-observability-and-diagnostics.md`, `06-quality/608-crash-reporting.md`  
**Related ADRs:** ADR-0047, ADR-0054

---

## 1. Purpose

This document defines extension health, timeouts, crash attribution, recovery, quarantine, support bundles, and user-visible failure behavior.

---

## 2. Health Model

Extension health includes:

- package state;
- runtime state;
- host process state;
- capability state;
- quota state;
- provider instances;
- UI contribution state;
- last successful activity;
- recent incidents;
- recovery attempts;
- user action required.

---

## 3. Failure Categories

- package invalid;
- incompatible;
- permission denied;
- startup timeout;
- runtime crash;
- host crash;
- call timeout;
- protocol violation;
- quota exceeded;
- source/output failure;
- UI contribution failure;
- migration failure;
- signature/revocation;
- sandbox violation.

---

## 4. Isolation Rules

- engine remains alive;
- unrelated extensions remain active when host topology permits;
- project data remains;
- affected sources use fallback;
- affected outputs stop or recover independently;
- UI contribution is detached;
- tokens and leases are revoked;
- repeated failure is bounded.

---

## 5. Recovery

Recovery may:

- retry call once;
- restart extension instance;
- restart dedicated host;
- move extension to dedicated host;
- disable one provider;
- disable extension for session;
- quarantine package.

Policies depend on failure category and crash frequency.

---

## 6. Crash Attribution

Attribution records:

- extension ID/version;
- publisher;
- host;
- active entrypoint;
- operation;
- recent bounded SDK calls;
- quota state;
- capability state;
- crash signature.

It excludes project content and secrets.

---

## 7. User Presentation

The UI explains:

- which extension failed;
- affected feature;
- whether program/output continues;
- automatic recovery status;
- restart/disable/report options;
- whether project data remains safe.

The extension cannot override host failure wording for security-critical incidents.

---

## 8. Diagnostics and Logs

Extension diagnostics are:

- extension-tagged;
- rate-limited;
- redacted;
- quota-controlled;
- correlated with host/engine spans;
- optionally included in support bundles with user consent.

---

## 9. Crash Loops

Crash-loop policy:

- count within time window;
- bounded restart;
- safe disable;
- preserve data;
- avoid auto-opening dependent source/output;
- offer diagnostic export;
- require user action to re-enable after threshold.

---

## 10. Invariants

1. Extension failure does not crash engine.
2. Project data remains intact.
3. Leases/tokens are revoked after failure.
4. Recovery attempts are bounded.
5. Crash attribution identifies extension.
6. Logs are rate-limited/redacted.
7. UI identifies affected feature.
8. Quarantine blocks execution.
9. Re-enable after crash loop requires explicit action.
10. Support bundles exclude secrets/content by default.

---

## 11. Required Tests

- extension startup failure;
- call timeout;
- protocol violation;
- quota exceeded;
- dedicated-host crash;
- shared-host crash;
- source fallback;
- output isolation;
- UI detach;
- crash loop;
- quarantine;
- support-bundle redaction.

---

## 12. AI Implementation Notes

Do not restart crashing extensions forever.

Do not let extension-provided error text replace host security diagnostics.

Do not discard extension project data after failure.

Always report what remains unaffected.

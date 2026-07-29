# 703 — Extension Lifecycle

**Status:** Proposed  
**Audience:** SDK, runtime, project, UI contributors  
**Canonical:** Yes  
**Required context:** `701-extension-architecture.md`, `702-extension-manifest.md`  
**Related ADRs:** ADR-0047, ADR-0054

---

## 1. Purpose

This document defines installation, enablement, loading, activation, project binding, suspension, update, disablement, uninstall, and recovery.

---

## 2. Lifecycle States

```rust
pub enum ExtensionState {
    Discovered,
    Validating,
    Installed,
    Disabled,
    Resolving,
    Starting,
    Ready,
    Active,
    Suspended,
    Updating,
    Stopping,
    Failed,
    Quarantined,
    Uninstalled,
}
```

---

## 3. Installation

Installation:

1. validates package structure;
2. verifies integrity/signature;
3. checks manifest compatibility;
4. reviews requested permissions;
5. stages package;
6. atomically publishes installed version;
7. records publisher/trust metadata;
8. leaves extension disabled or enabled according to user decision.

Installation does not load code before validation.

---

## 4. Enablement

Enablement:

- resolves SDK compatibility;
- resolves platform compatibility;
- requests missing required grants;
- chooses host mode;
- starts runtime;
- registers contributions;
- activates project-bound instances when needed.

---

## 5. Project Binding

When a project references extension data:

- extension compatibility is checked;
- project-data schema is migrated if possible;
- required extension absence is reported;
- optional data remains preserved;
- runtime instances are created only after project activation reaches safe stage.

---

## 6. Suspension

An extension may be suspended because:

- project closed;
- capability revoked;
- resource pressure;
- host restart;
- background policy;
- user action;
- repeated timeout.

Suspension stops new work and preserves bounded recoverable state.

---

## 7. Update

Update flow:

1. validate new package;
2. compare publisher identity;
3. review new capabilities;
4. stage package;
5. quiesce old runtime;
6. migrate extension-owned local/project data;
7. start new runtime;
8. validate health;
9. switch active version;
10. retain rollback candidate for bounded period.

New required permissions are never auto-granted.

---

## 8. Disable

Disable:

- stops instances;
- detaches UI;
- revokes capabilities;
- preserves project configuration;
- retains local data unless user requests removal;
- records dependent features as unavailable.

---

## 9. Uninstall

Uninstall choices:

- remove package only;
- remove package and local settings;
- remove package, local settings, and credentials;
- optionally remove extension-owned project data through explicit project command.

Project data is not silently deleted.

---

## 10. Quarantine

Quarantine may trigger on:

- signature failure;
- repeated crash loop;
- protocol abuse;
- resource abuse;
- security policy violation;
- corrupted package.

Quarantine disables execution but preserves evidence and data.

---

## 11. Invariants

1. Package validation precedes execution.
2. Enablement and installation are separate.
3. Update preserves publisher identity.
4. New permissions require review.
5. Disable/uninstall preserves project data by default.
6. Lifecycle transitions are serialized.
7. Quarantine blocks execution.
8. Rollback is bounded.
9. Extension data migration is transactional where possible.
10. User sees affected project features.

---

## 12. Required Tests

- install disabled;
- install and enable;
- missing required permission;
- project-bound activation;
- suspend/resume;
- compatible update;
- update with new capability;
- migration failure rollback;
- disable;
- uninstall package only;
- quarantine;
- crash-loop recovery.

---

## 13. AI Implementation Notes

Do not execute package code during installation validation.

Do not auto-grant new capabilities on update.

Do not erase project namespaces during ordinary uninstall.

Serialize lifecycle operations per extension.

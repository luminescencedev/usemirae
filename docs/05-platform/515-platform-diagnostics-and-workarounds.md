# 515 — Platform Diagnostics and Compatibility Workarounds

**Status:** Proposed  
**Audience:** Platform, quality, support, rendering, media contributors  
**Canonical:** Yes  
**Required context:** `514-platform-capability-registry.md`, future `06-quality/607-observability-and-diagnostics.md`  
**Related ADRs:** ADR-0036

---

## 1. Purpose

This document defines platform probes, compatibility records, workaround activation, support bundles, and the rules preventing scattered hardware-specific branches.

---

## 2. Diagnostic Snapshot

A platform snapshot may include:

- OS and build;
- package mode;
- CPU architecture;
- GPU and driver;
- display/session type;
- audio stack;
- capture backends;
- encoder backends;
- permissions;
- secure-store availability;
- updater/signature status;
- active workarounds;
- capability generations.

Sensitive identifiers are redacted.

---

## 3. Workaround Record

```rust
pub struct WorkaroundRule {
    pub id: WorkaroundId,
    pub affected_platform: PlatformSelector,
    pub hardware: Option<HardwareSelector>,
    pub driver_range: Option<VersionRange>,
    pub os_range: Option<VersionRange>,
    pub package_modes: Vec<PackageMode>,
    pub action: WorkaroundAction,
    pub reason: String,
    pub source_reference: String,
    pub introduced_in: AppVersion,
    pub review_after: Option<AppVersion>,
}
```

---

## 4. Workaround Actions

Examples:

- disable zero-copy interop;
- prefer software encoder;
- limit texture format;
- change present mode;
- disable timestamp queries;
- use alternate capture backend;
- reduce in-flight frames;
- avoid specific codec profile;
- require restart after permission.

Actions are typed, not arbitrary code snippets.

---

## 5. Activation

Activation requires:

- exact selector match;
- capability/probe confirmation where needed;
- deterministic precedence;
- diagnostics;
- user override only when safe;
- test coverage.

Conflicting rules are detected.

---

## 6. Distribution

The workaround database may ship with the application and optionally receive signed data updates.

Remote workaround updates must be:

- signed;
- schema-validated;
- bounded;
- reversible;
- recorded;
- unable to execute code.

---

## 7. Compatibility Probe

A probe:

- is read-only where possible;
- has timeout;
- avoids destabilizing stress tests during startup;
- records version;
- is cancellable;
- returns structured evidence.

Deep diagnostics may run only after user action.

---

## 8. Support Bundle

A platform support bundle may include:

- diagnostic snapshot;
- active workaround IDs;
- renderer/media capability reports;
- recent bounded logs;
- crash metadata;
- sanitized project compatibility report.

It excludes credentials and raw media by default.

---

## 9. Workaround Lifecycle

Each workaround has:

- owner;
- reason;
- evidence;
- introduced version;
- review condition;
- removal test;
- user impact.

Workarounds are not permanent undocumented defaults.

---

## 10. Invariants

1. Workarounds are centralized.
2. Rules are typed and non-executable.
3. Activation is diagnosable.
4. Remote updates are signed.
5. Probes are bounded.
6. Sensitive identifiers are redacted.
7. Conflicting rules are detected.
8. User override cannot violate safety.
9. Workarounds have review/removal criteria.
10. Support bundles exclude secrets and media by default.

---

## 11. Required Tests

- exact rule match;
- version range;
- conflicting rules;
- signed database update;
- invalid signature;
- probe timeout;
- user-safe override;
- renderer workaround;
- capture fallback;
- support-bundle redaction;
- rule removal fixture;
- offline operation.

---

## 12. AI Implementation Notes

Do not add vendor/driver conditionals directly inside unrelated rendering or media code.

Do not download executable workaround logic.

Do not include hardware serials or private paths in default support bundles.

Every workaround requires an ID, reason, scope, and removal condition.

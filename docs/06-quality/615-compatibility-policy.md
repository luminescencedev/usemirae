# 615 — Compatibility Policy

**Status:** Proposed  
**Audience:** Architecture, release, platform, project, SDK contributors  
**Canonical:** Yes  
**Required context:** `04-project/408-schema-versioning-and-migrations.md`, `01-runtime/108-ipc-protocol.md`, `05-platform/514-platform-capability-registry.md`

---

## 1. Purpose

This document defines support windows and compatibility expectations for projects, IPC, SDK, platforms, hardware, and release channels.

---

## 2. Compatibility Domains

- project schema;
- project bundles;
- IPC protocol;
- extension manifest;
- SDK/API;
- diagnostics schemas;
- operating systems;
- hardware/driver classes;
- package formats;
- update paths.

Each domain has its own version and policy.

---

## 3. Project Compatibility

Mirae should open and migrate supported historical project schemas.

Policy defines:

- oldest supported schema;
- migration test corpus;
- newer-schema behavior;
- downgrade limitations;
- irreversible changes.

Removing support requires release note and migration path where possible.

---

## 4. IPC Compatibility

Processes from one installation are expected to match.

Protocol policy:

- major mismatch rejects;
- minor negotiation where compatible;
- updater avoids mixed-version long-term operation;
- reconnect handles engine session changes.

---

## 5. SDK Compatibility

SDK uses semantic versioning or explicit API-version policy.

Rules:

- breaking API requires major;
- deprecation precedes removal;
- capability discovery avoids false assumptions;
- extension manifest declares supported range;
- host rejects incompatible extensions safely.

---

## 6. Platform Support

A supported platform definition includes:

- OS version range;
- architecture;
- package mode;
- required system components;
- graphics backend;
- known limitations.

“Best effort” environments are labeled experimental or unsupported.

---

## 7. Hardware Support

Support classes:

- certified/reference;
- supported;
- supported with limitations;
- experimental;
- unsupported.

Hardware support considers drivers and capabilities, not model name alone.

---

## 8. Deprecation

A deprecation includes:

- affected contract;
- replacement;
- first deprecated version;
- earliest removal version;
- migration guide;
- diagnostics;
- telemetry evidence only if opt-in and aggregated.

---

## 9. Release Channels

- stable prioritizes compatibility;
- beta may introduce opt-in new contracts;
- nightly may break experimental APIs;
- stable project files must not be silently made unreadable by nightly without warning and copy.

---

## 10. Invariants

1. Compatibility domains are versioned separately.
2. Project files have migration policy.
3. IPC mismatch fails safely.
4. SDK breaking changes are explicit.
5. Platform support includes package mode.
6. Hardware support is capability-based.
7. Deprecation has replacement and timeline.
8. Experimental contracts are labeled.
9. Stable releases do not silently invalidate supported projects.
10. Compatibility tests are automated.

---

## 11. Required Tests

- oldest supported project;
- newer project rejection;
- IPC major mismatch;
- IPC minor negotiation;
- SDK compatible/incompatible extension;
- deprecated API warning;
- platform unsupported state;
- hardware limitation;
- nightly project protection;
- update path;
- bundle version mismatch;
- migration guide validation.

---

## 12. AI Implementation Notes

Do not use one application version as the version for every contract.

Do not remove a persisted or SDK field without compatibility analysis.

Do not claim platform support without defining OS, architecture, package mode, and capability requirements.

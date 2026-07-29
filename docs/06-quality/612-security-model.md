# 612 — Security Model

**Status:** Proposed  
**Audience:** Security, runtime, platform, SDK, release contributors  
**Canonical:** Yes  
**Required context:** `002-product-and-system-boundaries.md`, `01-runtime/101-process-model.md`, `05-platform/508-secure-credential-storage.md`  
**Related ADRs:** ADR-0044

---

## 1. Purpose

This document defines trust boundaries, assets, threats, capability security, secure update requirements, input validation, and incident handling.

---

## 2. Protected Assets

- credentials;
- project content;
- recordings and replay;
- local paths;
- user identity metadata;
- signing keys;
- update trust;
- extension permissions;
- engine control;
- device access;
- crash and support artifacts.

---

## 3. Trust Boundaries

- UI/webview to engine;
- extension host to engine;
- worker to engine;
- local files/bundles to importer;
- network protocol to output/input adapters;
- updater to installation;
- OS APIs to native adapters;
- user project to extension-owned data;
- cloud/service integrations to credentials.

---

## 4. Threats

- malicious extension;
- crafted project/bundle;
- archive traversal;
- IPC spoofing;
- local privilege confusion;
- credential leakage;
- malicious media parser input;
- update tampering;
- driver/native crash exploitation;
- denial of service through unbounded resource use;
- webview navigation/content injection;
- unsafe deep link;
- telemetry overcollection.

---

## 5. Capability Security

Every actor receives explicit capabilities.

Examples:

- read selected project metadata;
- create source of declared kind;
- access extension storage;
- open approved network domains;
- receive selected events;
- request credential use without reading secret;
- add UI panel;
- write approved project namespace.

Capabilities are scoped, revocable, and auditable.

---

## 6. Input Validation

All untrusted input is bounded:

- IPC frames;
- project files;
- bundles;
- extension manifests;
- media metadata;
- network packets;
- deep links;
- update metadata;
- shader/effect data;
- file paths.

Parsing occurs before large allocation where possible.

---

## 7. Webview Security

Requirements:

- local packaged control UI;
- restrictive CSP;
- no arbitrary navigation;
- no Node-style unrestricted APIs;
- origin validation;
- typed bridge;
- browser-source isolation;
- production devtools policy;
- external URLs opened by OS.

---

## 8. Secrets

Secrets:

- reside in OS secure store;
- use short-lived leases;
- never enter project/bundle/log/telemetry;
- are not exposed to extensions directly;
- are redacted from URLs and headers;
- are rotated/revoked safely.

---

## 9. Update and Supply Chain

Required:

- signed releases;
- signed update metadata;
- dependency/license review;
- lockfiles;
- reproducible or traceable builds;
- SBOM where practical;
- vulnerability monitoring;
- key rotation and revocation plan;
- no unverified executable extension loading.

---

## 10. Sandboxing

Untrusted extension and risky parser/device work should be isolated.

Sandbox policy may restrict:

- filesystem;
- network;
- process launch;
- environment;
- native code;
- GPU access;
- CPU/memory;
- IPC methods.

---

## 11. Security Events

Security-relevant events include:

- failed authentication;
- permission denial;
- signature failure;
- manifest violation;
- path traversal attempt;
- oversized frame;
- repeated extension abuse;
- credential-store failure;
- update rollback.

Events are rate-limited and redacted.

---

## 12. Invariants

1. Trust boundaries are explicit.
2. Capabilities are least-privilege.
3. Untrusted input is bounded.
4. Secrets never enter ordinary persistence or logs.
5. Updates are signed.
6. Webview navigation is restricted.
7. Extensions cannot execute unrestricted engine-native code.
8. Security failures are diagnosable.
9. Sandboxes have resource limits.
10. Security exceptions require ADR and review.

---

## 13. Required Tests

- IPC spoof;
- malformed bundle;
- path traversal;
- oversized input;
- secret redaction;
- extension capability denial;
- webview navigation;
- deep-link injection;
- update signature failure;
- credential isolation;
- parser fuzzing;
- sandbox resource limit.

---

## 14. AI Implementation Notes

Do not trust local input merely because it comes from the same machine.

Do not expose generic file, network, or process APIs to extensions.

Do not disable verification to simplify development.

Model permissions and capabilities explicitly.

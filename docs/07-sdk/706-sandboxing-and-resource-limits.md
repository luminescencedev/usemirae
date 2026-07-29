# 706 — Sandboxing and Resource Limits

**Status:** Proposed  
**Audience:** SDK, security, process, performance contributors  
**Canonical:** Yes  
**Required context:** `701-extension-architecture.md`, `705-permission-and-capability-model.md`, `06-quality/602-memory-model.md`  
**Related ADRs:** ADR-0047, ADR-0054

---

## 1. Purpose

This document defines process isolation, runtime restrictions, quotas, timeouts, and abuse handling.

---

## 2. Sandbox Goals

The sandbox limits:

- engine memory access;
- filesystem access;
- network access;
- process creation;
- environment access;
- native device access;
- CPU use;
- memory use;
- thread/task count;
- message rate;
- storage;
- logs;
- GPU/media resources.

Platform capabilities determine exact enforcement.

---

## 3. Resource Quotas

Per extension:

- resident memory;
- allocation rate;
- CPU time/window;
- concurrent tasks;
- IPC messages/second;
- outstanding requests;
- event subscriptions;
- log bytes/minute;
- storage bytes;
- network connections;
- media queue bytes;
- frame processing deadline;
- UI contributions;
- operation count.

Default quotas are host policy, not extension choice.

---

## 4. Deadline Classes

- UI response;
- control API call;
- source frame production;
- effect frame processing;
- output packet handling;
- migration;
- background indexing.

Deadline failure produces structured timeout and may degrade or disable the extension.

---

## 5. Native Code

Third-party native code, if supported in future, requires:

- dedicated host;
- explicit trust;
- platform signature;
- stronger permissions;
- no engine injection;
- crash isolation;
- resource monitoring;
- disabled by default on unsupported packaging.

The initial public SDK should prefer managed/sandboxable runtimes.

---

## 6. File and Network Isolation

Extensions do not receive unrestricted OS APIs from host.

They use brokers that enforce:

- selected paths;
- declared domains;
- method/size limits;
- TLS policy;
- redirects;
- quotas;
- user grants.

---

## 7. Resource Violation

Violation response:

1. emit warning;
2. throttle/coalesce;
3. reject new work;
4. cancel operation;
5. suspend extension;
6. terminate host/instance;
7. quarantine repeated abuse.

Response depends on severity.

---

## 8. Shared Host Fairness

Scheduler applies per-extension fairness.

One extension cannot consume all:

- executor time;
- IPC bandwidth;
- log capacity;
- media pool;
- storage I/O.

---

## 9. Invariants

1. Third-party extensions have no engine memory access.
2. Quotas are explicit and measurable.
3. Timeouts are bounded.
4. File/network access is brokered.
5. Shared hosts enforce fairness.
6. Native code receives stronger isolation.
7. Repeated abuse escalates.
8. Quota changes are visible.
9. Sandbox limitations are capability-reported.
10. Resource violations do not corrupt project state.

---

## 10. Required Tests

- memory quota;
- CPU throttle;
- task count;
- IPC flood;
- log flood;
- storage quota;
- network connection limit;
- media queue limit;
- timeout;
- shared-host fairness;
- host termination cleanup;
- repeated abuse quarantine.

---

## 11. AI Implementation Notes

Do not implement quotas only as documentation.

Do not give extensions unrestricted file or network libraries through host APIs.

Do not allow one extension to starve shared-host tasks.

Every new extension resource requires accounting and a limit.

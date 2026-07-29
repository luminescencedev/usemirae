# 606 — Logging and Tracing

**Status:** Proposed  
**Audience:** All contributors, diagnostics, support contributors  
**Canonical:** Yes  
**Required context:** `605-error-model.md`  
**Related ADRs:** ADR-0041

---

## 1. Purpose

Mirae uses structured logs, spans, events, and correlation IDs to explain behavior across processes and subsystems.

---

## 2. Event Structure

A structured event includes:

- timestamp;
- monotonic timestamp where relevant;
- severity;
- subsystem;
- event name;
- engine session ID;
- process role;
- thread/executor;
- correlation ID;
- entity IDs;
- fields;
- redaction class.

---

## 3. Spans

Spans represent operations:

- engine startup;
- project open/save;
- command execution;
- frame compile;
- render submission;
- source activation;
- output start/reconnect;
- migration;
- update install;
- extension call.

Spans nest and propagate correlation across IPC.

---

## 4. Severity Levels

- trace;
- debug;
- info;
- warn;
- error.

Production defaults avoid high-volume trace/debug unless a bounded diagnostic mode is enabled.

---

## 5. Redaction

Field classes:

- public;
- internal;
- private;
- secret;
- media-content.

Secret and media-content fields are never included in normal logs.

Private fields are redacted or hashed in support exports.

---

## 6. Volume Control

Logging is bounded through:

- rate limiting;
- sampling;
- duplicate suppression;
- rolling file limits;
- category filters;
- per-extension quotas;
- dropped-log counters.

A failing component must not fill disk through repeated identical errors.

---

## 7. Process Correlation

All processes share:

- engine session ID;
- build ID;
- monotonic-relative clock mapping where possible;
- correlation IDs;
- protocol sequence references.

Log files may remain separate but can be merged by tooling.

---

## 8. Real-Time Paths

Audio and capture callbacks do not perform synchronous formatting or disk writes.

They may emit:

- lock-free counters;
- bounded compact events;
- deferred diagnostics.

---

## 9. File Retention

Log policy defines:

- maximum file size;
- maximum total bytes;
- retention age;
- rotation;
- compression;
- cleanup;
- crash preservation.

User projects and recordings never share the log directory.

---

## 10. Extension Logging

Extensions log through host API.

The host applies:

- extension identity;
- rate limit;
- field bounds;
- redaction;
- file separation or tagging;
- abuse handling.

---

## 11. Invariants

1. Logs are structured.
2. Correlation propagates across IPC.
3. Secret fields are prohibited.
4. Log storage is bounded.
5. Real-time callbacks do not write logs synchronously.
6. Duplicate storms are rate-limited.
7. Extension logs are quota-controlled.
8. Build and session identity are present.
9. User-visible errors link to diagnostic references.
10. Support export applies redaction.

---

## 12. Required Tests

- structured schema;
- correlation propagation;
- secret field rejection;
- rate limiting;
- duplicate suppression;
- log rotation;
- disk-full behavior;
- audio callback counter;
- extension quota;
- multi-process merge;
- redacted export;
- crash preservation.

---

## 13. AI Implementation Notes

Do not add free-form `println!` or console logging in production paths.

Do not log raw project documents, credentials, or media.

Do not synchronously format detailed errors in real-time callbacks.

Use stable event names and structured fields.

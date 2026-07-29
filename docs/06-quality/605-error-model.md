# 605 — Error Model

**Status:** Proposed  
**Audience:** All contributors and coding agents  
**Canonical:** Yes  
**Required context:** `600-quality-overview.md`  
**Related ADRs:** ADR-0040

---

## 1. Purpose

This document defines structured error categories, propagation, user presentation, retryability, severity, and redaction.

---

## 2. Error Layers

- input/schema error;
- domain validation error;
- conflict;
- permission/security error;
- external resource unavailable;
- capability/compatibility error;
- transient infrastructure error;
- persistent infrastructure error;
- data corruption;
- internal invariant violation;
- cancellation;
- timeout.

---

## 3. Error Structure

```rust
pub struct MiraeError {
    pub code: ErrorCode,
    pub category: ErrorCategory,
    pub severity: ErrorSeverity,
    pub subsystem: SubsystemId,
    pub retryability: Retryability,
    pub user_action: Option<UserActionHint>,
    pub correlation_id: CorrelationId,
    pub safe_message: String,
    pub diagnostic_context: ErrorContext,
    pub source: Option<Box<dyn Error + Send + Sync>>,
}
```

Internal source text is not automatically user-safe.

---

## 4. Error Codes

Error codes are stable machine-readable identifiers.

Examples:

```text
PROJECT_SCHEMA_UNSUPPORTED
CAPTURE_PERMISSION_DENIED
CAPTURE_SOURCE_REMOVED
RENDER_DEVICE_LOST
OUTPUT_AUTH_FAILED
RECORDING_DISK_FULL
IPC_PROTOCOL_MISMATCH
EXTENSION_CAPABILITY_DENIED
```

Codes must not encode variable values.

---

## 5. Severity

- `Info`;
- `Warning`;
- `Error`;
- `Critical`.

Severity reflects user/system impact, not developer embarrassment.

A recoverable source failure may be `Error` without being engine-fatal.

---

## 6. Retryability

- not retryable;
- retry immediately once;
- retry with backoff;
- retry after user action;
- retry after environment change;
- unknown.

Retry policy belongs to owning subsystem, not generic error loop.

---

## 7. Context

Add context at boundaries:

- operation;
- entity ID;
- source/output ID;
- state generation;
- platform backend;
- file role without private full path where possible;
- protocol phase;
- device generation.

Do not duplicate the same error string at every layer.

---

## 8. User Presentation

User-facing errors include:

- what failed;
- affected feature;
- whether current production continues;
- recommended action;
- whether retry is automatic;
- diagnostic reference.

They exclude:

- stack traces;
- secrets;
- raw vendor error dumps;
- internal paths by default.

---

## 9. Panics

Panics indicate:

- violated internal invariant;
- impossible state;
- unrecoverable programmer error.

Recoverable external conditions must not panic.

At process boundaries, panic is captured for crash reporting when possible.

---

## 10. Aggregation

Multiple related failures may aggregate under one root cause.

Example:

```text
Output failed
└── Network publish failed
    └── TLS certificate rejected
```

UI avoids showing every nested layer as separate notification.

---

## 11. Invariants

1. Errors have stable codes.
2. User messages are safe.
3. Retryability is explicit.
4. Recoverable external failure does not panic.
5. Context uses IDs and generations.
6. Root cause is preserved.
7. Secrets are redacted.
8. Cancellation is not logged as failure by default.
9. Severity matches impact.
10. Error handling does not silently mutate intent.

---

## 12. Required Tests

- code stability;
- secret redaction;
- user-safe formatting;
- nested context;
- retry classification;
- panic boundary;
- cancellation behavior;
- aggregation;
- platform error translation;
- IPC error mapping;
- corrupted project;
- extension error isolation.

---

## 13. AI Implementation Notes

Do not return arbitrary strings as the only error contract.

Do not panic on device loss, network failure, missing files, or invalid user configuration.

Do not log secrets through nested source errors.

Add stable error codes with tests.

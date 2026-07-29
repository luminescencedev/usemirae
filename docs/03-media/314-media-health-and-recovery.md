# 314 — Media Health and Recovery

**Status:** Proposed  
**Audience:** Media, runtime, diagnostics, UI contributors  
**Canonical:** Yes  
**Required context:** `300-media-overview.md`, `301-source-system.md`, `310-output-architecture.md`

---

## 1. Purpose

This document defines consistent health states, retry behavior, degradation, and recovery across sources, encoders, sinks, and media workers.

---

## 2. Health Model

Health state:

```rust
pub enum HealthState {
    Healthy,
    Starting,
    Recovering,
    Degraded,
    Unavailable,
    Failed,
    Stopped,
}
```

Health includes:

- component ID;
- generation;
- state;
- reason code;
- since time;
- retry state;
- last successful media time;
- metrics summary;
- user action required flag.

---

## 3. Reason Categories

- permission denied;
- device unavailable;
- device removed;
- decode failure;
- unsupported format;
- timestamp discontinuity;
- queue overload;
- encoder failure;
- network failure;
- disk failure;
- resource pressure;
- worker crash;
- compatibility issue;
- internal invariant violation.

Reason codes are stable and machine-readable.

---

## 4. Recovery Policy

A policy declares:

- automatic or manual;
- maximum attempts;
- retry window;
- backoff;
- reset conditions;
- state reconstruction;
- fallback;
- escalation;
- effect on dependent components.

---

## 5. Backoff

Backoff should use:

- bounded exponential delay;
- jitter;
- reset after stable success;
- immediate retry only for defined transient events;
- cancellation support.

No infinite tight loop.

---

## 6. Dependency Health

A component reports dependencies.

Examples:

- output depends on encoder and sink;
- camera source depends on permission and device;
- replay depends on encoder and storage;
- browser source depends on extension/worker runtime.

Aggregate health must preserve root cause rather than replacing it with generic failure.

---

## 7. Placeholders and Continuity

Source recovery may preserve scene continuity through:

- last frame;
- transparent frame;
- offline slate;
- silence;
- source-specific substitute.

The policy is explicit and visible.

---

## 8. Worker Crash

On media worker crash:

- invalidate worker-owned generations;
- release or abandon leases safely;
- mark dependent components recovering;
- restart within bounded policy;
- recreate state from persisted/runtime descriptors;
- escalate after repeated failure.

---

## 9. User Actions

Diagnostics may suggest:

- grant permission;
- reconnect device;
- select replacement;
- lower resolution;
- change encoder;
- free disk space;
- verify endpoint;
- restart source;
- collect diagnostic bundle.

Suggestions are not automatic destructive actions.

---

## 10. Recovery and Project State

Recovery does not silently rewrite project intent.

A replacement device or fallback encoder becomes persisted only after explicit user-confirmed configuration change.

---

## 11. Invariants

1. Health has machine-readable reason.
2. Retries are bounded.
3. Backoff is cancellable.
4. Root cause is preserved.
5. Recovery does not silently rewrite project intent.
6. Worker crash invalidates owned generations.
7. Placeholder policy is explicit.
8. Repeated failure escalates.
9. Health updates are rate-limited but not hidden.
10. Diagnostics contain no secrets or raw media.

---

## 12. Required Tests

- transient camera failure;
- permanent permission denial;
- encoder restart;
- network backoff;
- disk failure;
- worker crash;
- root-cause propagation;
- placeholder continuity;
- retry cancellation;
- repeated failure escalation;
- user-confirmed replacement;
- secret redaction.

---

## 13. AI Implementation Notes

Do not report every failure as a generic string.

Do not retry forever.

Do not automatically persist fallback device or encoder selection.

Keep health state generation-aware.

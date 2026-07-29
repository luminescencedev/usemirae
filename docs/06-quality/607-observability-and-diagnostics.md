# 607 — Observability and Diagnostics

**Status:** Proposed  
**Audience:** Runtime, support, UI, performance contributors  
**Canonical:** Yes  
**Required context:** `606-logging-and-tracing.md`, `05-platform/515-platform-diagnostics-and-workarounds.md`  
**Related ADRs:** ADR-0039, ADR-0041

---

## 1. Purpose

Observability exposes the internal state required to diagnose performance, failure, compatibility, and recovery without exposing secrets or raw media.

---

## 2. Signals

Mirae uses:

- structured logs;
- traces/spans;
- counters;
- gauges;
- histograms;
- health states;
- event sequences;
- bounded recent incident records;
- platform capability snapshots;
- crash context.

---

## 3. Metric Naming

Metric names are stable and namespaced.

Examples:

```text
runtime.command.duration_ms
render.frame.gpu_ms
media.source.queue_depth
audio.xrun.count
output.network.reconnect_count
project.save.duration_ms
extension.call.timeout_count
```

Labels are bounded. IDs with unbounded cardinality are not used indiscriminately.

---

## 4. Health Dashboard Model

A diagnostic snapshot can represent:

- engine;
- project;
- renderer;
- audio;
- each source;
- each output;
- extension host;
- platform services;
- storage;
- updater.

Each component reports state, reason, generation, metrics summary, and dependencies.

---

## 5. Diagnostic Modes

- normal production mode;
- enhanced local tracing;
- performance capture;
- compatibility probe;
- support bundle collection;
- safe mode.

Modes are bounded and user-visible.

Enhanced modes must not accidentally persist forever.

---

## 6. Support Bundles

A support bundle may include:

- build and platform metadata;
- redacted logs;
- trace excerpts;
- health snapshots;
- capability registry;
- active workaround IDs;
- crash metadata;
- project validation summary;
- performance counters.

Excluded by default:

- credentials;
- raw media;
- full project content;
- private paths;
- browser cookies;
- extension private storage.

---

## 7. Live Diagnostics UI

The UI may show:

- source state;
- output state;
- dropped-frame reasons;
- encoder load;
- GPU timing;
- audio clipping/XRUN;
- network queue;
- disk throughput;
- recovery attempts;
- permission/capability issues.

Metrics update through dedicated rate-limited stores.

---

## 8. Incident Record

An incident record includes:

- incident ID;
- first/last time;
- component;
- reason code;
- severity;
- count;
- recovery attempts;
- related spans;
- user-visible status;
- resolution.

Repeated identical events coalesce into one incident.

---

## 9. Invariants

1. Metrics have stable names.
2. Label cardinality is bounded.
3. Health includes reason and generation.
4. Diagnostic modes are explicit.
5. Support bundles are redacted.
6. High-frequency metrics do not flood UI.
7. Incidents coalesce repeated failures.
8. Diagnostics do not alter production semantics.
9. Every critical recovery path emits evidence.
10. Observability overhead is measured.

---

## 10. Required Tests

- metric schema;
- bounded labels;
- health dependency chain;
- incident coalescing;
- diagnostic mode timeout;
- support bundle redaction;
- UI rate limiting;
- performance capture;
- recovery evidence;
- extension metrics quota;
- metrics under disconnect;
- overhead benchmark.

---

## 11. AI Implementation Notes

Do not use arbitrary entity names as unbounded metric labels.

Do not include project content in support bundles by default.

Do not let diagnostics change frame scheduling or output behavior except in explicit diagnostic mode.

Measure instrumentation overhead.

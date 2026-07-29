# 601 — Performance Budgets

**Status:** Proposed  
**Audience:** Runtime, rendering, media, UI, platform, performance contributors  
**Canonical:** Yes  
**Required context:** `600-quality-overview.md`  
**Related ADRs:** ADR-0039, ADR-0043

---

## 1. Purpose

This document defines measurable performance budgets and how regressions are detected.

Budgets are targets and release constraints. They must be refined with real hardware data.

---

## 2. Reference Workloads

At minimum, benchmarks cover:

- idle project;
- one 1080p60 camera;
- one 1440p60 display capture;
- mixed 1080p60 scene with text, browser, camera, and effects;
- 1440p60 program plus preview;
- 4K60 composition;
- streaming plus recording;
- replay enabled;
- eight audio sources;
- large project with many inactive sources;
- low-end supported hardware;
- typical mid-range hardware;
- high-end hardware.

Each benchmark records hardware, OS, driver, app build, and feature configuration.

---

## 3. Frame Budgets

For a target frame rate:

```text
60 fps → 16.67 ms
30 fps → 33.33 ms
120 fps → 8.33 ms
```

The total frame pipeline must reserve headroom.

Example initial 60 fps production allocation:

| Stage | Target p95 |
|---|---:|
| Scheduler wake and dispatch | ≤ 0.5 ms |
| Frame compile | ≤ 1.5 ms |
| Render graph build | ≤ 0.5 ms |
| CPU command encoding | ≤ 2.0 ms |
| GPU execution | ≤ 8.0 ms |
| Surface/encoder handoff | ≤ 1.5 ms |
| Reserved headroom | ≥ 2.67 ms |

These are initial targets, not universal guarantees.

---

## 4. UI Budgets

Control UI targets:

- direct manipulation feedback: ≤ 16 ms local visual response;
- command acknowledgement p95: ≤ 100 ms for local non-I/O mutation;
- state patch display p95: ≤ 150 ms from command submission;
- panel open: ≤ 100 ms after warm load;
- project list first content: ≤ 500 ms on typical hardware;
- no long task above 50 ms on UI main thread without explicit mitigation.

High-frequency meters must not cause whole-application rerenders.

---

## 5. Audio Budgets

Audio callback requirements:

- finish comfortably before callback deadline;
- no general heap allocation in steady state;
- no unbounded lock wait;
- XRUN rate target: zero under reference workload;
- graph swap bounded to block boundary;
- control-to-audio parameter update latency documented.

Exact callback budget depends on sample rate and block size.

---

## 6. Startup Budgets

Initial targets on typical SSD hardware:

- shell visible: ≤ 1.0 s;
- engine ready without project: ≤ 2.0 s;
- medium project interactive: ≤ 3.0 s;
- output start after validation: ≤ 2.0 s where external services permit.

Startup phases must be traced separately.

---

## 7. Memory Budgets

Memory budgets are defined by workload and subsystem.

Required reporting:

- process RSS;
- GPU memory estimate;
- media queues;
- replay store;
- project state;
- caches;
- extensions;
- per-output overhead.

Idle memory and per-source/output incremental cost must be measured.

---

## 8. Network and Output Budgets

Track:

- encoder input latency;
- encoder packet latency;
- muxer backlog;
- send queue media duration;
- reconnect time;
- recording write latency;
- replay export latency.

Network buffers must have duration and byte limits.

---

## 9. Percentiles

Report at least:

- median;
- p95;
- p99;
- maximum;
- drop/error count.

Averages alone are insufficient for real-time systems.

---

## 10. Regression Thresholds

A benchmark fails when:

- hard correctness threshold is violated;
- frame/audio drops increase beyond allowed tolerance;
- p95 or p99 regresses beyond configured percentage;
- memory grows beyond configured absolute or relative limit;
- sustained throughput falls below target.

Thresholds may differ by benchmark stability.

---

## 11. Instrumentation

Every critical stage should provide:

- start/end span;
- correlation ID;
- frame/output/source ID;
- queue depth;
- allocation bytes where measurable;
- reason-coded drop;
- hardware/backend context.

Instrumentation overhead must be measured and configurable.

---

## 12. Invariants

1. Critical paths have measurable budgets.
2. Budgets are workload-specific.
3. Percentiles are reported.
4. Hardware and build context are recorded.
5. Drops are counted by reason.
6. UI metrics are separate from engine metrics.
7. Performance claims require before/after evidence.
8. Benchmark thresholds are version-controlled.
9. Instrumentation overhead is bounded.
10. Production quality does not silently degrade to meet a budget.

---

## 13. Required Tests

- reference workload suite;
- idle CPU test;
- frame latency test;
- UI long-task test;
- audio XRUN test;
- memory plateau test;
- output queue duration test;
- startup trace;
- regression threshold evaluation;
- instrumentation overhead comparison.

---

## 14. AI Implementation Notes

Do not optimize based only on average timing.

Do not invent performance numbers.

Do not reduce production output quality silently to pass a benchmark.

Include the exact benchmark and hardware context with every performance claim.

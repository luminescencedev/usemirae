# 610 — Benchmarking and Regressions

**Status:** Proposed  
**Audience:** Performance, runtime, rendering, media, release contributors  
**Canonical:** Yes  
**Required context:** `601-performance-budgets.md`, `609-testing-strategy.md`  
**Related ADRs:** ADR-0039, ADR-0043

---

## 1. Purpose

This document defines reproducible benchmarks, baseline storage, comparison, noise control, and regression triage.

---

## 2. Benchmark Types

- microbenchmark;
- subsystem throughput;
- frame pipeline;
- audio callback;
- project load/save;
- migration;
- startup;
- memory plateau;
- output reliability;
- UI interaction;
- long-run soak;
- power/thermal where supported.

---

## 3. Benchmark Metadata

Every result records:

- benchmark ID/version;
- app build/commit;
- configuration;
- project/fixture hash;
- OS/build;
- CPU;
- RAM;
- GPU/driver;
- storage;
- power mode;
- package mode;
- background-load policy;
- run count;
- warmup;
- instrumentation mode.

---

## 4. Baselines

Baselines are:

- version-controlled summaries for deterministic microbenchmarks;
- stored benchmark artifacts for hardware runners;
- compared against selected main/release commit;
- updated only with review.

Different hardware classes have separate baselines.

---

## 5. Noise Control

Methods:

- warmup;
- repeated runs;
- median/percentiles;
- outlier policy;
- pinned workload;
- stable power mode;
- background-process reduction;
- thermal monitoring;
- fixed fixture;
- dedicated runners where possible.

---

## 6. Regression Detection

A regression record includes:

- metric;
- old/new values;
- confidence;
- threshold;
- hardware;
- suspected subsystem;
- trace link;
- owner;
- disposition.

Possible dispositions:

- confirmed regression;
- expected trade-off;
- measurement noise;
- baseline change;
- test defect.

---

## 7. Soak Tests

Long-run tests measure:

- memory growth;
- queue growth;
- drift;
- dropped frames;
- reconnect stability;
- file-handle leaks;
- GPU recovery;
- extension-host restarts;
- log growth.

Soak duration and workload are explicit.

---

## 8. Performance Bisect

Benchmarks should support commit bisect with machine-readable pass/fail threshold where practical.

Artifacts must be reproducible enough to compare historical builds.

---

## 9. Invariants

1. Results include hardware/build context.
2. Baselines are reviewed.
3. Thresholds are explicit.
4. Noise is measured.
5. Long-run stability is benchmarked.
6. Memory regressions count as performance regressions.
7. Expected trade-offs are documented.
8. Benchmark fixtures are versioned.
9. Debug/instrumented builds are not compared with release builds accidentally.
10. Regression triage has owner and disposition.

---

## 10. Required Tests

- benchmark metadata validation;
- baseline comparison;
- threshold failure;
- noisy result classification;
- memory soak;
- startup benchmark;
- render benchmark;
- audio deadline benchmark;
- project migration benchmark;
- output reconnect soak;
- bisect smoke;
- report generation.

---

## 11. AI Implementation Notes

Do not compare benchmark numbers from different hardware or build modes as if equivalent.

Do not update baselines merely to make CI pass.

Include before/after data and confidence when claiming improvement.

# 305 — Master Clock and Timebase

**Status:** Proposed  
**Audience:** Runtime, media, audio, rendering, output contributors  
**Canonical:** Yes  
**Required context:** `303-media-data-model.md`, `01-runtime/103-frame-scheduler.md`  
**Related ADRs:** ADR-0017

---

## 1. Purpose

This document defines the authoritative media timeline, timestamp conversion rules, clock selection, drift observation, and discontinuity handling.

---

## 2. Clock Types

Mirae distinguishes:

- monotonic process clock;
- audio device clock;
- capture device clock;
- media file timeline;
- network source clock;
- wall clock;
- output transport clock.

Wall clock is used for labels, file naming, and external correlation. It is not the primary scheduling clock for continuous media.

---

## 3. Master Timeline

Mirae uses one master media timeline per active engine production session.

The master timeline is based on a monotonic clock and represented through rational media time.

Audio or output devices may provide clock observations used for drift correction without replacing the conceptual timeline unpredictably.

---

## 4. Timeline Mapping

Each source maintains a mapping:

```text
source timestamp
→ source discontinuity domain
→ normalized master timeline timestamp
```

Mapping includes:

- offset;
- rate estimate;
- uncertainty;
- last observation;
- discontinuity ID.

---

## 5. Rational Timebase

All timing conversions use rational arithmetic where practical.

Requirements:

- no floating-point accumulation for long-running authoritative timestamps;
- checked multiplication/division;
- explicit rounding policy;
- duration and timestamp types are distinct where useful;
- conversion errors are testable.

---

## 6. Clock Selection

The engine chooses timing policy based on active outputs and audio.

The policy may use:

- monotonic timeline as authority;
- audio device feedback for drift;
- external output clock in future specialized modes.

Clock policy is explicit and observable.

A clock source failure creates a controlled rebase or degraded state.

---

## 7. Source Timestamp Normalization

Normalization handles:

- missing timestamps;
- non-monotonic timestamps;
- timestamp wrap;
- rate mismatch;
- device reset;
- reconnect;
- seek;
- burst delivery.

Fallback timestamping is source-kind specific and diagnostically marked.

---

## 8. Discontinuities

Discontinuity occurs on:

- seek;
- reconnect;
- timestamp reset;
- clock rebase;
- format change;
- pipeline restart;
- encoder restart where relevant.

A discontinuity creates a new continuity segment.

Consumers must reset temporal history when required.

---

## 9. Drift

Drift is measured between source/device clock and master timeline.

Metrics:

- current offset;
- estimated rate error;
- jitter;
- correction action;
- confidence.

Correction is bounded and smooth unless discontinuity requires reset.

---

## 10. Scheduling

The frame scheduler requests media for target presentation time.

The audio engine renders blocks aligned to master timeline while respecting device callback timing.

Outputs map master timestamps into encoder and container timebases.

---

## 11. Wall Clock

Wall clock may jump due to:

- user changes;
- NTP;
- timezone;
- daylight saving.

Such jumps must not reorder media or change transition progress.

Wall clock metadata is sampled separately.

---

## 12. Invariants

1. One master media timeline exists per active production session.
2. Wall clock does not schedule continuous media.
3. Timebases are explicit and rational.
4. Discontinuities are explicit.
5. Source mappings are generation- and discontinuity-aware.
6. Drift correction is bounded.
7. Missing timestamps are diagnosable.
8. Long-running timestamps avoid floating accumulation.
9. Encoder timestamps derive from master timeline.
10. Clock failure has recovery policy.

---

## 13. Required Tests

- rational conversion;
- long-duration stability;
- wall-clock jump;
- timestamp wrap;
- missing timestamp fallback;
- source reconnect;
- seek;
- drift correction;
- audio clock observation;
- clock failover;
- encoder timebase mapping;
- discontinuity reset.

---

## 14. AI Implementation Notes

Do not use system wall clock for transition or frame scheduling.

Do not normalize all timestamps to floating-point seconds.

Do not hide timestamp resets by forcing monotonic values without a discontinuity.

Keep source clock mapping explicit and testable.

# 304 — Media Pipeline

**Status:** Proposed  
**Audience:** Media, rendering, audio, output contributors  
**Canonical:** Yes  
**Required context:** `303-media-data-model.md`, `01-runtime/103-frame-scheduler.md`  
**Related ADRs:** ADR-0016, ADR-0017

---

## 1. Purpose

The media pipeline moves media from acquisition to render, audio, encode, record, replay, and network sinks through explicit bounded stages.

---

## 2. Pipeline Stages

Typical video path:

```text
capture/demux
→ decode
→ timestamp normalize
→ format inspect
→ optional convert/upload
→ bounded source frame queue
→ frame selection
→ renderer
→ output scaler/convert
→ encoder
→ muxer
→ sink
```

Typical audio path:

```text
capture/demux
→ decode
→ timestamp normalize
→ sample conversion
→ bounded audio queue
→ audio graph
→ output bus
→ encoder
→ muxer
→ sink
```

---

## 3. Stage Contract

Every stage declares:

- accepted input type;
- produced output type;
- ownership transfer;
- queue capacity;
- overflow behavior;
- threading domain;
- latency budget;
- metrics;
- cancellation;
- discontinuity behavior;
- error behavior.

---

## 4. Queue Policy

Example policies:

| Boundary | Default policy |
|---|---|
| live video capture → source queue | keep latest / drop oldest |
| file decode → playback queue | bounded backpressure |
| audio capture → audio ring | bounded ring with XRUN |
| renderer → encoder | bounded backpressure or frame drop by output policy |
| encoder → muxer | bounded backpressure |
| muxer → network | bounded with reconnect policy |
| muxer → disk | bounded with recording failure escalation |
| encoder → replay | bounded packet retention |

No stage relies on unbounded queue growth.

---

## 5. Worker Model

Workers may include:

- capture callback adapter;
- demux task;
- decode workers;
- conversion workers;
- upload workers;
- encoder workers;
- muxer task;
- sink task.

Work placement respects thread affinity and codec/API requirements.

A worker pool must not cause head-of-line blocking between unrelated critical sources without prioritization.

---

## 6. Source Frame Queue

The queue stores immutable video frames with timing metadata.

Selection by renderer uses:

- target time;
- source policy;
- current discontinuity;
- frame duration;
- stale threshold;
- latency mode.

Old frames are retired through storage lease release.

---

## 7. File Playback

File playback adds:

- demux position;
- seek generation;
- preroll;
- end-of-stream;
- loop point;
- rate control;
- decode-ahead budget;
- cache policy.

A seek invalidates old queued frames through generation or discontinuity.

---

## 8. Live Sources

Live source path prioritizes freshness.

Policies may discard frames when:

- capture outpaces render;
- decode is late;
- queue exceeds target latency;
- source timestamp jumps.

The pipeline reports why.

---

## 9. Format Changes

A format change creates a new media format generation.

Downstream stages:

1. drain or discard incompatible queued data;
2. emit discontinuity;
3. renegotiate conversion;
4. rebuild affected resources;
5. resume.

A format change does not silently reinterpret existing buffers.

---

## 10. Cancellation and Shutdown

Each pipeline owns a cancellation tree.

Shutdown sequence:

1. stop new source input;
2. signal end or cancellation;
3. drain according to policy;
4. finalize encoders/muxers where required;
5. release leases;
6. publish stopped state.

Live output shutdown and file playback cancellation may use different drain policies.

---

## 11. Metrics

Per stage:

- items in/out;
- bytes in/out;
- queue depth;
- processing latency;
- wait time;
- drops;
- errors;
- resets;
- format changes;
- discontinuities;
- worker utilization.

---

## 12. Invariants

1. Every stage has explicit contract.
2. Every queue is bounded.
3. Media units are immutable.
4. Format change increments generation.
5. Seek invalidates old playback data.
6. Live path prioritizes freshness.
7. File path may backpressure within budget.
8. Shutdown releases all leases.
9. Drops and discontinuities are reason-coded.
10. Worker placement respects API affinity.

---

## 13. Required Tests

- live overload;
- file backpressure;
- seek;
- loop;
- format change;
- decode error;
- queue overflow;
- cancellation;
- drain;
- worker starvation;
- stale frame retirement;
- discontinuity propagation.

---

## 14. AI Implementation Notes

Do not share one queue policy across every media boundary.

Do not reuse old buffers after seek or format generation change.

Do not block live capture callbacks to wait for downstream consumers.

Instrument every stage from the beginning.

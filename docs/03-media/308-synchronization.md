# 308 — Synchronization

**Status:** Proposed  
**Audience:** Media, audio, rendering, output contributors  
**Canonical:** Yes  
**Required context:** `305-master-clock-and-timebase.md`, `306-audio-architecture.md`  
**Related ADRs:** ADR-0017, ADR-0018

---

## 1. Purpose

Synchronization aligns audio, video, generated graphics, transitions, and encoded outputs to the master media timeline.

---

## 2. Synchronization Domains

- source-to-master mapping;
- audio-to-master mapping;
- video frame selection;
- audio/video output alignment;
- multi-output alignment;
- transition timing;
- file playback A/V sync;
- network source jitter handling.

---

## 3. Video Selection

For each target presentation time, the source policy selects:

- nearest prior frame;
- nearest frame;
- interpolated frame where explicitly supported;
- repeated last frame;
- transparent/offline fallback.

The policy is source-kind and latency-mode specific.

---

## 4. Audio Alignment

Audio is continuous.

The audio engine adjusts using:

- resampling;
- bounded buffering;
- silence insertion;
- discontinuity reset;
- route delay.

It does not drop arbitrary individual samples without policy.

---

## 5. Drift Thresholds

Each source class defines thresholds:

- normal jitter;
- correction range;
- warning threshold;
- discontinuity threshold;
- restart threshold.

Thresholds are configurable internally and observable.

---

## 6. Lip-Sync

Per-source or per-route offset may compensate:

- camera pipeline latency;
- microphone latency;
- capture card delay;
- wireless device delay;
- effect latency.

Offsets are persisted as user intent where appropriate.

---

## 7. File Playback

File playback synchronization uses demux timestamps.

On seek:

- increment seek generation;
- flush queues;
- reset decoders as required;
- preroll;
- establish new discontinuity;
- resume at target timeline mapping.

---

## 8. Network Jitter

Network sources may use a bounded jitter buffer.

The jitter buffer defines:

- target latency;
- maximum latency;
- reordering window;
- packet-loss behavior;
- clock mapping;
- catch-up strategy.

It cannot grow without bound to avoid drops.

---

## 9. Multi-Output Sync

Outputs may have different encoding and transport latency.

Mirae guarantees common source timeline, not identical arrival time across unrelated networks.

Where supported, output timestamps derive from the same master timeline.

---

## 10. Transition Sync

Video and audio transition policies use one transition start and duration.

UI animation does not define progress.

Late video frames do not move the audio transition timeline unless explicit recovery policy says so.

---

## 11. Diagnostics

Required diagnostics:

- source offset;
- source drift;
- A/V offset;
- jitter-buffer depth;
- repeated video frames;
- dropped video frames;
- audio correction ratio;
- discontinuity count;
- route delay;
- output timestamp discontinuity.

---

## 12. Invariants

1. One master timeline coordinates media.
2. Audio continuity is preserved through bounded correction.
3. Video selection policy is explicit.
4. Jitter buffers are bounded.
5. Seek creates discontinuity.
6. User sync offsets are explicit.
7. Transition audio/video share timing.
8. Wall clock does not define A/V sync.
9. Outputs derive timestamps from common timeline.
10. Excess drift escalates to discontinuity or recovery.

---

## 13. Required Tests

- camera/microphone offset;
- file A/V sync;
- seek;
- network jitter;
- packet reordering;
- excessive drift;
- audio correction;
- repeated video frame;
- transition sync;
- multi-output timestamps;
- long-duration drift stability.

---

## 14. AI Implementation Notes

Do not fix sync by allowing queues to grow indefinitely.

Do not use independent UI timers for video and audio transitions.

Do not hide large timestamp jumps with aggressive resampling.

Use explicit thresholds and diagnostics.

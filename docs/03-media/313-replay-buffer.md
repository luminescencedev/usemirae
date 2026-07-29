# 313 — Replay Buffer

**Status:** Proposed  
**Audience:** Media, output, storage contributors  
**Canonical:** Yes  
**Required context:** `309-encoder-system.md`, `310-output-architecture.md`, `312-recording.md`  
**Related ADRs:** ADR-0022

---

## 1. Purpose

The replay buffer retains recent encoded media in bounded storage and can export a decodable time range without re-encoding when format and boundaries permit.

---

## 2. Storage Model

The canonical replay architecture stores encoded packets, not raw frames.

Reasons:

- lower memory/storage use;
- reduced export latency;
- shared encoder opportunities;
- preservation of output quality.

Raw-frame replay may exist later for specialized editing but is not the default architecture.

---

## 3. Retention

Retention may be bounded by:

- duration;
- bytes;
- packet count;
- storage location;
- minimum keyframe coverage.

The effective retained duration is observable.

---

## 4. Packet Store

The store tracks:

- packet payload lease;
- stream ID;
- PTS/DTS;
- duration;
- keyframe;
- codec configuration generation;
- discontinuity;
- segment membership;
- byte size.

Packets are immutable.

---

## 5. Keyframe Index

Replay extraction requires a decodable starting point.

The store indexes:

- video keyframes;
- audio packet boundaries;
- codec configuration changes;
- discontinuities;
- segment starts.

Requested start time may be adjusted backward to nearest valid keyframe.

The UI must distinguish requested and actual saved range if material.

---

## 6. Storage Backends

Possible backends:

- memory;
- memory-mapped file;
- rotating temporary segments;
- hybrid memory/disk.

Each backend defines:

- capacity;
- latency;
- crash cleanup;
- privacy;
- encryption if required;
- filesystem impact.

---

## 7. Export Flow

```mermaid
flowchart LR
    Request[Replay Save Request]
    Resolve[Resolve Actual Range]
    Pin[Pin Required Packets]
    Mux[Create Output Container]
    Write[Write Packets]
    Finalize[Finalize and Publish]
    Release[Release Pin]

    Request --> Resolve --> Pin --> Mux --> Write --> Finalize --> Release
```

Pinned packets cannot be evicted during export.

Pinning is bounded and export concurrency is limited.

---

## 8. Shared Encoder

Replay may share encoded packets with an active output only when settings and ownership contract match.

Otherwise it owns its own encoder pipeline.

A shared stream restart creates a new codec configuration generation.

---

## 9. Discontinuities

Replay export must not silently cross incompatible:

- encoder restart;
- codec change;
- resolution change;
- timebase reset;
- major timestamp discontinuity.

Policy may:

- split into multiple files;
- start after discontinuity;
- reject range;
- remux when safe.

---

## 10. Audio and Track Mapping

Replay profile defines audio tracks independently from streaming profile.

Track selection must be stable for one replay runtime generation.

---

## 11. Privacy

Replay buffers may contain sensitive recent media.

Requirements:

- local by default;
- bounded lifetime;
- clear temporary storage location;
- secure cleanup best effort;
- no cloud upload without explicit action;
- diagnostics exclude media payload.

---

## 12. Invariants

1. Default replay store contains encoded packets.
2. Retention is bounded.
3. Export starts at decodable boundary.
4. Pinned packets are not evicted.
5. Export concurrency is bounded.
6. Discontinuities are explicit.
7. Codec configuration generation is tracked.
8. Replay failure does not stop unrelated output.
9. Temporary media is local by default.
10. Packet payload is immutable.

---

## 13. Required Tests

- duration retention;
- byte retention;
- keyframe alignment;
- concurrent export limit;
- pinned eviction protection;
- encoder restart;
- discontinuity split;
- memory backend;
- disk backend;
- shared encoder;
- replay failure isolation;
- cleanup after crash.

---

## 14. AI Implementation Notes

Do not keep raw frames by default.

Do not start an exported file on a non-decodable inter frame.

Do not let replay export pin the entire store indefinitely.

Track codec configuration and discontinuities explicitly.

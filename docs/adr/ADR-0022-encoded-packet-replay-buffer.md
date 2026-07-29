# ADR-0022 — Encoded-Packet Replay Buffer

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Replay requires retaining recent media while keeping resource use bounded.

Raw frame retention is expensive at production resolutions and frame rates.

---

## Decision

The default replay buffer will retain encoded packets with keyframe, timestamp, discontinuity, and codec-configuration indexes.

Replay export will remux packets when possible without re-encoding.

---

## Consequences

### Positive

- lower memory and disk use;
- fast export;
- output-quality preservation;
- possible encoder sharing.

### Negative

- save start aligns to keyframe;
- codec changes complicate extraction;
- exact frame-level start may require re-encoding in future;
- packet indexing required.

---

## Alternatives Considered

### Raw-frame replay

Rejected as the default because resource cost is too high.

### Save only muxed rolling files without packet index

Rejected because precise bounded extraction and continuity handling would be weaker.

---

## Related Specifications

- `03-media/313-replay-buffer.md`
- `03-media/309-encoder-system.md`
- `03-media/312-recording.md`

# ADR-0016 — FFmpeg as a Contained Toolkit

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Mirae needs broad codec, container, demux, mux, resampling, and media-format support.

Implementing all of this from scratch is not practical.

At the same time, allowing FFmpeg types and lifetime rules to spread through the engine would couple the architecture to one toolkit.

---

## Decision

Mirae will use FFmpeg as a contained media toolkit behind Mirae-owned abstractions.

FFmpeg may provide:

- demuxing;
- muxing;
- decoding;
- encoding where selected;
- resampling;
- format conversion;
- protocol support where appropriate.

FFmpeg structs and error codes will not become domain contracts.

---

## Consequences

### Positive

- broad format support;
- mature codec/container ecosystem;
- practical cross-platform media foundation;
- reduced implementation scope.

### Negative

- native build and licensing complexity;
- unsafe FFI;
- version compatibility management;
- some platform-native paths still require separate adapters.

---

## Alternatives Considered

### Build all codecs and containers internally

Rejected as unrealistic.

### Expose FFmpeg types throughout media code

Rejected because it would create pervasive coupling and unstable domain boundaries.

---

## Related Specifications

- `03-media/300-media-overview.md`
- `03-media/303-media-data-model.md`
- `03-media/304-media-pipeline.md`
- `03-media/309-encoder-system.md`

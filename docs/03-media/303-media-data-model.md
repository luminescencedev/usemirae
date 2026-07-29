# 303 — Media Data Model

**Status:** Proposed  
**Audience:** Media, rendering, audio, output contributors  
**Canonical:** Yes  
**Required context:** `300-media-overview.md`

---

## 1. Purpose

This document defines Mirae-owned media unit types that isolate the system from codec, container, platform, and GPU toolkit structures.

---

## 2. Media Time

```rust
pub struct MediaTime {
    pub value: i64,
    pub timebase: RationalTimebase,
}
```

Rules:

- timebase denominator is non-zero;
- comparisons convert safely;
- overflow is checked;
- unknown timestamps use explicit optional state;
- wall-clock timestamps are not substituted for missing media timestamps.

---

## 3. Video Frame

Conceptual model:

```rust
pub struct VideoFrame {
    pub id: VideoFrameId,
    pub source_id: SourceId,
    pub source_generation: SourceRuntimeGeneration,
    pub capture_time: Option<MediaTime>,
    pub presentation_time: MediaTime,
    pub duration: Option<MediaDuration>,
    pub extent: PixelExtent,
    pub format: VideoFormat,
    pub color: ColorMetadata,
    pub alpha: AlphaMetadata,
    pub storage: VideoStorage,
    pub flags: VideoFrameFlags,
}
```

---

## 4. Video Storage

Variants may include:

- GPU texture lease;
- platform external texture;
- CPU planar buffer;
- CPU packed buffer;
- shared memory region;
- opaque adapter-owned frame with conversion interface.

The storage variant defines ownership and lifetime.

---

## 5. Audio Block

```rust
pub struct AudioBlock {
    pub id: AudioBlockId,
    pub source_id: SourceId,
    pub source_generation: SourceRuntimeGeneration,
    pub start_time: MediaTime,
    pub sample_rate: SampleRate,
    pub channel_layout: ChannelLayout,
    pub frames: usize,
    pub format: AudioSampleFormat,
    pub storage: AudioStorage,
    pub flags: AudioBlockFlags,
}
```

Internal audio engine input converts to canonical format before real-time mixing.

---

## 6. Encoded Packet

```rust
pub struct EncodedPacket {
    pub stream_id: EncodedStreamId,
    pub codec: CodecId,
    pub pts: Option<MediaTime>,
    pub dts: Option<MediaTime>,
    pub duration: Option<MediaDuration>,
    pub keyframe: bool,
    pub discontinuity: Option<DiscontinuityId>,
    pub side_data: PacketSideData,
    pub payload: ByteLease,
}
```

Encoded packet payload is immutable after publication.

---

## 7. Discontinuity

A discontinuity explicitly marks:

- timestamp reset;
- source reconnect;
- seek;
- clock jump;
- format change;
- packet loss recovery;
- encoder restart.

Consumers must not infer continuity across different discontinuity IDs.

---

## 8. Format Descriptors

Video format includes:

- pixel layout;
- bit depth;
- plane count;
- subsampling;
- memory domain;
- row alignment;
- color metadata;
- alpha semantics.

Audio format includes:

- sample type;
- planar/interleaved;
- sample rate;
- channel layout;
- endianness when relevant.

---

## 9. Immutability

Published media units are immutable.

A subsystem needing a transformed unit creates a new media unit or a new storage view with explicit ownership.

Mutable buffer reuse is allowed only behind exclusive leases before publication.

---

## 10. Memory Leases

Large payloads use leases.

A lease defines:

- owner;
- size;
- storage domain;
- release callback or pool return;
- generation;
- thread-safety;
- exportability.

Lease retention is bounded by queue capacity and in-flight work.

---

## 11. Flags

Flags may include:

- corrupted;
- partial;
- repeated;
- stale;
- synthetic;
- discontinuity;
- preroll;
- end-of-stream;
- dropped-predecessor;
- protected-content.

Flags do not replace structured diagnostics.

---

## 12. Invariants

1. Media units use Mirae-owned types.
2. Payloads are immutable after publication.
3. Timing metadata is explicit.
4. Unknown timestamps are represented explicitly.
5. Discontinuity is explicit.
6. Storage ownership is explicit.
7. Large payloads use bounded leases.
8. Color and channel metadata travel with media.
9. Runtime generations prevent stale reuse.
10. Toolkit-native types stay behind adapters.

---

## 13. Required Tests

- timebase conversion;
- timestamp overflow;
- video storage lifetime;
- audio block layout validation;
- encoded packet immutability;
- discontinuity propagation;
- stale generation rejection;
- lease return;
- metadata round trip;
- unknown timestamp behavior.

---

## 14. AI Implementation Notes

Do not alias mutable codec buffers after publication.

Do not use raw FFmpeg timestamps or platform times without converting into Mirae types.

Do not use `Option<i64>` without carrying the associated timebase.

Keep payload ownership explicit.

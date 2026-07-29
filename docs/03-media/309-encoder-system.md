# 309 — Encoder System

**Status:** Proposed  
**Audience:** Media, output, platform, performance contributors  
**Canonical:** Yes  
**Required context:** `303-media-data-model.md`, `310-output-architecture.md`  
**Related ADRs:** ADR-0016, ADR-0020

---

## 1. Purpose

The encoder system provides stable Mirae-owned interfaces for software and hardware video/audio encoders.

---

## 2. Encoder Registry

The registry exposes encoder implementations by capability.

An encoder descriptor includes:

- encoder ID;
- codec;
- software/hardware class;
- supported formats;
- resolution limits;
- frame-rate limits;
- rate-control modes;
- profile/level;
- B-frame support;
- low-latency support;
- HDR metadata support;
- device compatibility;
- platform support.

---

## 3. Encoder Session

```rust
pub trait VideoEncoderSession {
    fn submit(&mut self, frame: VideoEncodeInput) -> Result<SubmitResult>;
    fn receive(&mut self) -> Result<Vec<EncodedPacket>>;
    fn flush(&mut self) -> Result<Vec<EncodedPacket>>;
    fn reconfigure(&mut self, request: EncoderReconfigure) -> Result<ReconfigureResult>;
    fn health(&self) -> EncoderHealth;
}
```

The public domain interface does not expose FFmpeg, NVENC, VideoToolbox, Media Foundation, VA-API, or other toolkit-native types.

---

## 4. Input Negotiation

Negotiation covers:

- frame size;
- frame rate;
- pixel format;
- color metadata;
- memory domain;
- device compatibility;
- latency mode;
- keyframe policy.

Preferred path uses GPU or platform-native surfaces without CPU readback.

Fallback path uses explicit conversion or software encoding.

---

## 5. Rate Control

Supported model may include:

- CBR;
- VBR;
- constrained VBR;
- constant quality;
- lossless where available.

Configuration is represented through codec-neutral concepts plus implementation-specific advanced options behind namespaced schema.

---

## 6. Keyframes

Keyframes may be requested by:

- output start;
- segment boundary;
- reconnect;
- replay extraction boundary;
- protocol request;
- scene transition policy;
- user command.

Request is advisory if encoder cannot guarantee immediate keyframe.

---

## 7. Reconfiguration

Dynamic reconfiguration may include:

- bitrate;
- quality target;
- keyframe request;
- limited rate-control changes.

Changes requiring session restart are reported explicitly.

Restart creates a new encoder generation and output discontinuity.

---

## 8. Encoder Queue

Encoder input queue is bounded.

Overflow policy is output-specific:

- drop frame;
- backpressure within budget;
- reduce preview-only source rate;
- fail output;
- restart encoder.

Recording and live streaming may use different policies.

---

## 9. Hardware Fallback

Fallback order may be:

1. preferred hardware encoder;
2. alternate hardware encoder;
3. software encoder;
4. output unavailable.

Fallback must validate:

- codec;
- format;
- color support;
- performance budget;
- output protocol/container compatibility.

Fallback is visible.

---

## 10. Audio Encoders

Audio encoder sessions declare:

- codec;
- sample rate;
- channel layout;
- frame size;
- bitrate/quality;
- delay and priming;
- metadata.

Audio frame packing bridges engine audio blocks to encoder frame size.

---

## 11. Packet Timing

Encoder output packets include:

- PTS;
- DTS;
- duration;
- keyframe;
- codec config changes;
- discontinuity;
- side data.

Timestamps derive from master timeline and encoder delay semantics.

---

## 12. Invariants

1. Encoder interfaces are toolkit-independent.
2. Input queue is bounded.
3. Encoder generation changes on restart.
4. Packet timing is explicit.
5. Hardware fallback is visible.
6. CPU readback is not the default GPU path.
7. Reconfiguration capability is queried, not assumed.
8. Codec-specific options are namespaced.
9. Flush behavior is explicit.
10. Encoder failure is output-local where possible.

---

## 13. Required Tests

- software encode;
- hardware mock;
- zero-copy input;
- CPU fallback;
- bitrate reconfigure;
- restart-required reconfigure;
- keyframe request;
- queue overflow;
- flush;
- packet timestamp ordering;
- audio frame packing;
- hardware failure fallback.

---

## 14. AI Implementation Notes

Do not expose vendor SDK objects through output profiles.

Do not assume bitrate can be changed live on every encoder.

Do not hide encoder restart from timestamp/discontinuity handling.

Keep queue policy owned by the output pipeline.

# ADR-0018 — Canonical Internal Audio Format

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Audio sources and outputs use different sample formats, rates, channel layouts, and block sizes.

Allowing arbitrary formats inside the mix graph would multiply effect and routing complexity.

---

## Decision

Mirae will convert audio into one canonical 32-bit floating-point internal mix format at the configured engine sample rate and explicit channel layouts.

Conversion occurs before real-time mixing.

---

## Consequences

### Positive

- simpler effects and mixing;
- stable meter behavior;
- explicit channel semantics;
- predictable routing;
- easier testing.

### Negative

- conversion cost;
- resampler state;
- channel-map complexity;
- one engine rate may not match every device.

---

## Alternatives Considered

### Preserve native format throughout graph

Rejected because every node would require many format variants.

### Integer-only internal mix

Rejected because floating-point provides better practical headroom and effect integration.

---

## Related Specifications

- `03-media/306-audio-architecture.md`
- `03-media/307-audio-routing-and-monitoring.md`
- `03-media/308-synchronization.md`

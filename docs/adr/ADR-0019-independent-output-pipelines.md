# ADR-0019 — Independent Output Pipelines

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Streaming, recording, replay, and virtual outputs have different reliability, buffering, encoding, and sink requirements.

A shared monolithic output path would couple failures and configuration.

---

## Decision

Each output will own an independent runtime pipeline with its own encoders, muxer, sink, queues, lifecycle, diagnostics, and recovery policy.

Compatible upstream surfaces or encoded streams may be shared through explicit leases as an optimization.

---

## Consequences

### Positive

- fault isolation;
- output-specific policy;
- independent restart;
- clearer diagnostics;
- recording can continue during network failure.

### Negative

- higher possible resource use;
- more lifecycle objects;
- shared encoding requires compatibility checks.

---

## Alternatives Considered

### One global output pipeline

Rejected because one sink failure could affect all outputs.

### Always share encoders

Rejected because output settings and restart semantics differ.

---

## Related Specifications

- `03-media/310-output-architecture.md`
- `03-media/311-streaming-and-network-reliability.md`
- `03-media/312-recording.md`
- `03-media/313-replay-buffer.md`

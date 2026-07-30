# ADR-0067 — JSON Control-Plane Serialization Over Bounded Binary Framing

**Status:** Accepted  
**Date:** 2026-07-30  
**Supersedes:** nothing  
**Related:** ADR-0006 (typed versioned IPC), ADR-0057 (generated cross-language contracts)

---

## Context

`01-runtime/108-ipc-protocol.md` section 5 requires a schema-driven serialization
format and states that the choice needs an ADR. No existing ADR covers it, and
`MIR-0012` cannot encode a handshake without one.

Section 5 sets the criteria:

- Rust and TypeScript generation;
- explicit optional fields;
- version evolution;
- bounded decoding;
- deterministic fixtures;
- no arbitrary code execution during decode.

Section 89 of the same document is explicit that JSON is not *automatically* the
canonical high-frequency encoding. It is equally explicit that the control plane
carries control data, not media: section 1 and invariant 4 keep raw video and audio
out of it entirely, and section 9 sends large payloads through dedicated data-plane
mechanisms.

---

## Decision

Control-plane IPC uses **JSON payloads inside the bounded binary frame** already
described in `108` section 4.

The frame header stays binary and fixed-size. It carries the magic bytes, protocol
major and minor, message type, flags, payload length, and correlation id. The
header is validated before any allocation, so the payload length is checked against
the connection's limits before a single byte of payload is read.

The payload inside that frame is JSON, generated from the canonical schemas by
`cargo xtask generate`.

This decision covers the control plane only. The data plane for media is out of
scope and will get its own decision when it exists.

---

## Consequences

### Positive

- **Zero new TypeScript dependency.** The control UI parses JSON natively. Every
  binary alternative would need a decoder package on a surface that already has a
  strict approved-dependency list.
- **Explicit optional fields and easy version evolution.** An added optional field
  is ignored by an older peer, which is exactly the minor-version rule in `108`
  section 8.2.
- **Deterministic fixtures.** Generated output is already sorted and stable, so
  fixtures diff cleanly and a contract change is visible in review.
- **No code execution during decode.** JSON has no schema-embedded behaviour, no
  type resolution by name, and no deserialization gadget surface.
- **Debuggable.** A captured frame is readable during development, which matters
  while the protocol is still being shaped.

### Negative

- **Larger and slower than a binary encoding.** Accepted because this is the
  control plane: commands, acknowledgements, state patches, and events, not frames
  of video. If profiling later shows control-plane pressure, the framing already
  carries a message type and version, so a second payload encoding can be
  negotiated without redesigning the protocol.
- **Numbers are doubles by default.** Mitigated by the schema subset: integers
  declare a `maximum`, generation counters and ids that exceed the safe integer
  range must be expressed as strings, and the generator enforces the bound.
- **Two Rust dependencies.** `serde` and `serde_json` must clear
  `DEPENDENCY_VERSIONS.md` section 11 before use: exact pins, committed
  `Cargo.lock`, a Rust dependency section in that document, and license and
  security review.

---

## Alternatives Considered

### MessagePack or CBOR

Compact and well specified, and either would serve. Rejected for v1 because both
add a decoder dependency to the TypeScript surface for a plane that is not
bandwidth-bound, trading a real dependency for a saving we cannot yet measure.

### Protocol Buffers, FlatBuffers, or Cap'n Proto

Strong schema evolution and fast decoding. Rejected because each introduces a
second schema language beside the canonical JSON Schema in `schemas/`, which
contradicts `805` invariant 1: one canonical schema per contract. Adopting one
would mean either translating schemas or moving the source of truth.

### Postcard or another Rust-native format

Excellent on the Rust side, no credible TypeScript story. The control UI is a
first-class peer, not an afterthought.

---

## Implementation Notes

- The generator emits serialization support alongside the existing types;
  `cargo xtask generate --check` continues to detect drift.
- A decoder must reject an oversized frame using the header length before
  allocating, and enforce maximum nesting depth, collection length, and string
  length from the schema bounds. `108` section 9 requires all four.
- Round-trip and rejection tests are required per contract: round trip,
  unknown-field tolerance across a minor bump, oversized payload, and malformed
  input.

---

## Related Specifications

- `01-runtime/108-ipc-protocol.md`
- `08-development/805-generated-contracts-and-schemas.md`
- `DEPENDENCY_VERSIONS.md` section 11

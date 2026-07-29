# 709 — Media Extension Points

**Status:** Proposed  
**Audience:** SDK, media, rendering, output, security contributors  
**Canonical:** Yes  
**Required context:** `03-media/300-media-overview.md`, `02-rendering/210-effects-and-transitions.md`, `706-sandboxing-and-resource-limits.md`  
**Related ADRs:** ADR-0047, ADR-0054

---

## 1. Purpose

This document defines safe extension points for sources, outputs, effects, importers, and media metadata.

---

## 2. Source Providers

A source provider declares:

- source kind ID;
- configuration schema;
- media outputs;
- timing behavior;
- capabilities;
- resource limits;
- lifecycle;
- fallback;
- permissions;
- project-data schema.

Source runtime produces frames/audio through bounded data-plane contracts.

---

## 3. Output Providers

An output provider declares:

- destination kind;
- configuration schema;
- supported codecs/containers;
- credential broker needs;
- network domains;
- lifecycle;
- retry behavior;
- packet/backpressure contract;
- diagnostics.

The provider receives encoded packets or approved media surfaces, not unrestricted engine access.

---

## 4. Effect Providers

Effect options:

- host-approved shader/effect schema;
- WebAssembly/managed compute under restrictions;
- trusted first-party native effect in isolated host;
- declarative composition from built-in effect primitives.

Effect declares:

- color/alpha domain;
- inputs/outputs;
- parameters;
- resource estimate;
- temporal history bound;
- deterministic status;
- execution deadline;
- fallback.

---

## 5. Media Data Plane

Media transfer may use:

- shared memory leases;
- GPU external-resource leases;
- bounded packet queues;
- host-copied buffers;
- platform-specific worker bridge.

Every path defines:

- ownership;
- generation;
- format;
- timing;
- synchronization;
- capacity;
- release;
- crash behavior.

---

## 6. Real-Time Restrictions

Third-party extension code does not execute directly on:

- audio callback;
- render submission critical section;
- capture callback;
- state transaction lock.

It receives prepared asynchronous work with deadlines and bounded queues.

---

## 7. Format Negotiation

Provider and host negotiate:

- media type;
- pixel/sample format;
- timebase;
- color/channel metadata;
- memory domain;
- resolution/rate;
- latency;
- fallback conversion.

Unsupported negotiation fails explicitly.

---

## 8. Failure Behavior

- source failure → placeholder/silence;
- output failure → output-local stop/retry;
- effect timeout → bypass/last-valid policy;
- importer failure → no project mutation;
- host crash → generation invalidation.

Fallback is declared and diagnosable.

---

## 9. Invariants

1. Extension media paths are bounded.
2. Third-party code avoids critical callbacks.
3. Timing/color/channel metadata is explicit.
4. Data-plane leases are generation-aware.
5. Effect history is bounded.
6. Source/output failure is isolated.
7. Import failure is transactional.
8. GPU access is capability- and platform-limited.
9. Format negotiation is explicit.
10. Project intent survives provider absence.

---

## 10. Required Tests

- extension video source;
- audio source;
- source queue overflow;
- source crash;
- output reconnect;
- output backpressure;
- effect timeout;
- effect color declaration;
- shared-memory lease;
- stale generation;
- importer rollback;
- provider absent on project open.

---

## 11. AI Implementation Notes

Do not call third-party code from the audio callback.

Do not expose raw renderer device or encoder SDK objects.

Do not allow unbounded media history or queues.

Require explicit timing, format, ownership, and fallback for every media extension.

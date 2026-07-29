# 506 — Hardware Acceleration

**Status:** Proposed  
**Audience:** Platform, rendering, media, output, performance contributors  
**Canonical:** Yes  
**Required context:** `02-rendering/205-renderer-backend.md`, `03-media/309-encoder-system.md`, `500-platform-overview.md`  
**Related ADRs:** ADR-0038

---

## 1. Purpose

This document defines how Mirae discovers, negotiates, selects, uses, and falls back from hardware acceleration.

---

## 2. Acceleration Areas

- GPU composition;
- capture texture import;
- video decode;
- video encode;
- color conversion;
- scaling;
- media copy;
- external-memory interop;
- display presentation.

Support is independent per area.

---

## 3. Capability Descriptor

A hardware capability includes:

- implementation ID;
- adapter/device identity;
- codecs/formats;
- dimensions;
- frame rates;
- bit depths;
- chroma formats;
- HDR metadata;
- memory domains;
- latency modes;
- simultaneous-session limits;
- known limitations;
- stability class.

---

## 4. Selection Policy

Selection considers:

1. correctness;
2. device compatibility;
3. stability/workarounds;
4. zero-copy path;
5. latency;
6. quality;
7. power use;
8. user preference;
9. performance.

“Hardware” does not automatically outrank a stable software path.

---

## 5. Device Matching

Capture, renderer, decoder, and encoder may use different devices.

The platform determines:

- same-device path;
- cross-device import;
- cross-device copy;
- CPU staging fallback;
- unsupported combination.

The selected path is observable.

---

## 6. Session Limits

Hardware encoders and decoders may have session limits.

Mirae tracks:

- active sessions;
- failed allocation;
- profile limits;
- concurrent resolution/load;
- fallback order.

Session-limit failure affects only dependent outputs where possible.

---

## 7. Fallback

Fallback can occur on:

- initialization failure;
- runtime device loss;
- unsupported format;
- session limit;
- driver workaround;
- power policy;
- user override.

Fallback must preserve output contract or report incompatibility.

It cannot silently change codec, resolution, color, or quality outside allowed policy.

---

## 8. Caching and Probing

Capability probing is:

- bounded;
- cached by adapter/driver/build;
- invalidated on driver or capability changes;
- safe;
- separated from destructive stress testing.

Deep probes may run through diagnostics tools rather than startup.

---

## 9. Resource Interop

Interop contract defines:

- producer and consumer devices;
- handle type;
- acquire/release synchronization;
- format compatibility;
- color metadata;
- copy fallback;
- failure ownership.

---

## 10. User Controls

Advanced users may select:

- preferred GPU;
- preferred encoder;
- software fallback;
- low-power/high-performance mode;
- disable unstable backend.

Invalid choices return clear diagnostics.

---

## 11. Invariants

1. Hardware capability is queried, not assumed.
2. Selection is diagnosable.
3. Device compatibility is validated.
4. Fallback is explicit.
5. Session limits are tracked.
6. Output semantics do not silently change.
7. Probe results are versioned by hardware/driver.
8. Cross-device copies are visible.
9. Runtime failure invalidates generations.
10. User override cannot bypass safety validation.

---

## 12. Required Tests

- preferred hardware path;
- unsupported format;
- session limit;
- device mismatch;
- zero-copy;
- cross-device copy;
- software fallback;
- runtime encoder loss;
- driver workaround;
- stale probe cache;
- user override;
- semantic mismatch rejection.

---

## 13. AI Implementation Notes

Do not equate “hardware available” with “hardware usable for this pipeline.”

Do not silently switch codec or color format during fallback.

Do not cache capabilities without driver/device identity.

Expose cross-device copies in diagnostics and metrics.

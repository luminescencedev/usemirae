# 204 — Compositor

**Status:** Proposed  
**Audience:** Rendering, scene, effects contributors  
**Canonical:** Yes  
**Required context:** `201-scene-graph.md`, `202-frame-compiler.md`, `203-render-graph.md`, `208-color-management.md`  
**Related ADRs:** ADR-0002, ADR-0014

---

## 1. Purpose

The compositor combines source images and generated graphics into a final scene image while preserving transform, alpha, blend, mask, effect, and color semantics.

---

## 2. Composition Order

Canonical high-level order:

```text
source acquisition representation
→ source decode/color normalization
→ source-local crop and orientation
→ source-local effects
→ masks
→ item transform
→ item opacity
→ sibling blend
→ group isolation if required
→ group effects and opacity
→ scene-level operations
→ surface conversion
```

An effect may declare a different required domain only through the effect contract.

---

## 3. Alpha Model

The compositor uses one canonical internal alpha convention.

Recommended internal convention:

- premultiplied alpha for compositing;
- explicit conversion at source and export boundaries.

Requirements:

- source metadata declares alpha interpretation;
- straight-alpha input converts before blend;
- alpha-zero color behavior is defined;
- color transforms preserve premultiplication correctness;
- export converts to target alpha mode.

---

## 4. Blend Modes

Initial required blend modes:

- normal;
- add;
- multiply;
- screen;
- darken;
- lighten;
- difference;
- source-over replacement where explicitly needed.

Unsupported modes produce explicit fallback diagnostics.

Blend semantics are defined in the internal working color space, not arbitrary encoded source space.

---

## 5. Masks

Mask types may include:

- alpha mask;
- luminance mask;
- geometric clip;
- rounded rectangle;
- source-derived mask;
- future extension-defined mask.

Mask evaluation order is explicit.

Masks may require offscreen resources when they affect grouped results.

---

## 6. Transforms

Transforms use the scene graph's canonical order.

The compositor receives resolved matrices and clipping bounds from the frame compiler.

It should not reinterpret semantic transform order.

---

## 7. Scaling

Scaling filter selection may depend on:

- scale ratio;
- quality tier;
- source type;
- latency class;
- output use.

Supported policies may include:

- nearest;
- bilinear;
- bicubic;
- Lanczos or compute-based high-quality scaler;
- pixel-art integer scaling.

Production configuration must expose deterministic selection or explicit auto policy.

---

## 8. Sampling and Edge Behavior

Each sampled source declares:

- clamp;
- transparent border;
- repeat where supported;
- mirrored repeat;
- source-specific chroma sampling.

Default scene item behavior is transparent outside source bounds.

---

## 9. Group Isolation

A group requires isolated offscreen composition when:

- group opacity is below one;
- group blend mode applies to combined result;
- group effects exist;
- group mask applies after child combination;
- nested color operation requires a group result.

The compiler determines isolation. The compositor executes it.

---

## 10. Precision

Internal precision is selected by color policy.

At minimum:

- SDR production must avoid repeated 8-bit quantization;
- HDR paths require sufficient range;
- effect chains must declare precision needs;
- lower precision preview paths must be explicit.

---

## 11. Placeholders

When a source is unavailable, configured fallback may be:

- transparent;
- last valid frame;
- generated offline placeholder;
- user-defined replacement;
- error slate in preview only.

Program-visible error slates require explicit project setting.

---

## 12. Diagnostics Overlay

The compositor may render diagnostic overlays only on surfaces that explicitly request them.

A debug overlay must never accidentally enter production or recording output.

Surface policy controls overlay inclusion.

---

## 13. Invariants

1. Internal alpha convention is consistent.
2. Composition occurs in defined working color space.
3. Group opacity applies to combined group result.
4. Transform order is not reinterpreted.
5. Unavailable source fallback is explicit.
6. Diagnostic overlays are surface-scoped.
7. Unsupported blend behavior is diagnosable.
8. Scaling policy is deterministic for one configuration.
9. Surface conversion occurs after scene composition unless specification requires otherwise.
10. Compositor does not mutate scene state.

---

## 14. Required Tests

- premultiplied alpha;
- straight-to-premultiplied conversion;
- blend modes;
- group opacity;
- nested group;
- mask ordering;
- crop and transform;
- scaling filters;
- transparent border;
- unavailable source fallback;
- diagnostic overlay isolation;
- SDR/HDR precision comparison.

---

## 15. AI Implementation Notes

Do not blend gamma-encoded SDR values as if they were linear working values.

Do not apply group opacity independently to each child.

Do not assume every source has premultiplied alpha.

Keep debug overlays out of production by explicit surface policy.

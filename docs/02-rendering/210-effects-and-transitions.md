# 210 — Effects and Transitions

**Status:** Proposed  
**Audience:** Rendering, scene, SDK, UI contributors  
**Canonical:** Yes  
**Required context:** `202-frame-compiler.md`, `203-render-graph.md`, `207-shader-system.md`, `208-color-management.md`

---

## 1. Purpose

Effects transform source or composed images. Transitions combine a source production state and destination production state over time.

Both are declared semantically and executed through the render graph.

---

## 2. Effect Model

```rust
pub struct EffectInstance {
    pub id: EffectInstanceId,
    pub kind: EffectKindId,
    pub enabled: bool,
    pub parameters: EffectParameters,
    pub quality: EffectQualityPolicy,
}
```

Effect implementation details are not persisted.

---

## 3. Effect Contract

Each effect declares:

- stable kind ID;
- parameter schema;
- input count;
- input color/alpha domain;
- output domain;
- precision;
- temporal state requirement;
- required passes;
- resource estimate;
- quality variants;
- deterministic status;
- extension capability if third-party.

---

## 4. Effect Ordering

Effects execute in persisted list order.

An optimization may fuse effects only when output remains semantically equivalent.

Disabled effects remain in project state but do not execute.

---

## 5. Temporal Effects

Temporal effects may depend on prior frames.

They must define:

- history length;
- history resource size;
- reset conditions;
- discontinuity handling;
- scene switch behavior;
- source timestamp behavior;
- memory bound.

History is runtime state and is not serialized into the project.

---

## 6. Parameter Updates

Parameter categories:

- compile-time structural;
- pipeline specialization;
- uniform per frame;
- animated;
- resource reference.

The system should avoid rebuilding pipelines for ordinary uniform changes.

---

## 7. Built-In Effect Classes

Initial architecture should support:

- color correction;
- opacity;
- crop and mask;
- blur;
- sharpen;
- chroma key;
- LUT;
- transform-like image operations;
- shadow/glow;
- displacement;
- user-defined safe shader effect later.

This is a capability model, not an initial feature guarantee for every effect.

---

## 8. Transition Model

A transition runtime contains:

- source scene snapshot;
- destination scene snapshot;
- transition kind;
- start time;
- duration;
- progress mapping;
- audio transition policy;
- interruption policy;
- target surfaces.

---

## 9. Transition Progress

Progress is derived from the master monotonic/media timeline.

UI animation frames do not determine transition progress.

Progress mapping may apply easing, but easing is deterministic and bounded.

---

## 10. Transition Interruption

The system must define behavior when a new transition is requested while one is active.

Supported policies may include:

- reject;
- queue one bounded request;
- cut immediately;
- reverse when compatible;
- transition from current composed result.

Default policy must be explicit.

Unbounded transition queues are prohibited.

---

## 11. Transition Surfaces

Preview and program may display different transition states.

Program transition output is authoritative for live surfaces.

Preview may show:

- destination preparation;
- transition preview;
- current program;
- isolated transition editor preview.

These are separate surface requests, not one shared UI texture.

---

## 12. Audio Transition

Audio policy may include:

- cut;
- crossfade;
- follow scene ownership;
- preserve global sources;
- custom bus transition.

Audio timing is coordinated with the transition timeline but executed by the audio subsystem.

---

## 13. Effect Failure

On effect failure:

- built-in effect may disable for frame and emit diagnostic;
- last valid pipeline may be used when safe;
- extension effect may be isolated;
- scene remains renderable where possible;
- output does not silently switch to semantically unrelated effect.

---

## 14. Invariants

1. Effect order is explicit.
2. Effect parameters are schema-validated.
3. Temporal history is bounded.
4. Transition timing is not UI-driven.
5. Transition queue is bounded.
6. Audio transition policy is explicit.
7. Extension effects do not execute arbitrary engine-native code in critical paths.
8. Effect failure is diagnosable.
9. Fusion preserves semantics.
10. Runtime history is not persisted as project data.

---

## 15. Required Tests

- effect ordering;
- disabled effect;
- uniform update without pipeline rebuild;
- temporal reset;
- discontinuity;
- transition timing;
- interruption policy;
- program/preview divergence;
- audio policy handoff;
- effect failure fallback;
- extension effect validation;
- memory bound.

---

## 16. AI Implementation Notes

Do not use UI timers for transition progress.

Do not allow unbounded temporal history.

Do not compile a new pipeline for every animated scalar parameter.

Do not silently reorder effects for optimization.

# 202 — Frame Compiler

**Status:** Proposed  
**Audience:** Rendering, runtime, media contributors  
**Canonical:** Yes  
**Required context:** `201-scene-graph.md`, `01-runtime/103-frame-scheduler.md`  
**Related ADRs:** ADR-0004, ADR-0012

---

## 1. Purpose

The frame compiler transforms semantic scene state and live runtime inputs into a renderer-independent frame render plan for one target time and one or more compatible surfaces.

It is the boundary between production intent and executable rendering work.

---

## 2. Inputs

```rust
pub struct FrameCompileRequest {
    pub ticket: FrameTicket,
    pub scene_snapshot: Arc<SceneSnapshot>,
    pub source_set: SourceFrameSet,
    pub transition: Option<TransitionSnapshot>,
    pub surfaces: Vec<SurfaceDescriptor>,
    pub capabilities: RenderCapabilities,
    pub quality: RenderQualityPolicy,
}
```

Inputs are immutable for the compile operation.

---

## 3. Output

The compiler produces:

```rust
pub struct FrameRenderPlan {
    pub frame_id: FrameId,
    pub state_generation: StateGeneration,
    pub scene_generation: SceneGeneration,
    pub logical_nodes: Vec<RenderNode>,
    pub resources: Vec<LogicalResource>,
    pub surfaces: Vec<SurfacePlan>,
    pub diagnostics: Vec<CompileDiagnostic>,
    pub cache_keys: Vec<CompileCacheKey>,
}
```

The plan contains no active GPU resource handles.

---

## 4. Responsibilities

The compiler resolves:

- scene hierarchy;
- nested scenes;
- inherited visibility;
- transforms and clipping;
- source frame selection;
- source-unavailable placeholders;
- effect order;
- group offscreen requirements;
- transition source and destination scenes;
- surface-specific scaling;
- color conversion requirements;
- intermediate resource requirements;
- pass fusion opportunities at a logical level;
- diagnostic references back to scene items.

---

## 5. Non-Responsibilities

The compiler does not:

- allocate GPU textures;
- create pipelines;
- submit commands;
- wait for source frames;
- mutate scene or source state;
- perform media decode;
- decide output retry behavior.

---

## 6. Compilation Phases

```mermaid
flowchart LR
    Validate --> ResolveHierarchy
    ResolveHierarchy --> ResolveSources
    ResolveSources --> ResolveTransforms
    ResolveTransforms --> ResolveEffects
    ResolveEffects --> ResolveColor
    ResolveColor --> PlanIntermediates
    PlanIntermediates --> Optimize
    Optimize --> EmitPlan
```

### 6.1 Validate snapshot

Confirm:

- generations;
- graph validity;
- target surface compatibility;
- capability availability.

### 6.2 Resolve hierarchy

Flatten or partially compile nested semantic hierarchy while preserving diagnostic identity.

### 6.3 Resolve sources

Select runtime frame or fallback for target time.

### 6.4 Resolve transforms

Compute world transforms, bounds, clipping, and visibility culling.

### 6.5 Resolve effects

Determine exact effect chain and offscreen boundaries.

### 6.6 Resolve color

Insert required decode-to-working-space and working-to-surface conversions.

### 6.7 Plan intermediates

Declare logical resources and precision.

### 6.8 Optimize

Apply semantics-preserving optimizations.

### 6.9 Emit plan

Produce deterministic plan and diagnostics.

---

## 7. Source Frame Selection

The compiler receives a pre-resolved or queryable source frame set from the media layer.

Selection policy depends on source type and master-clock rules.

The compiler records:

- selected source frame ID;
- source timestamp;
- stale frame status;
- missing frame status;
- fallback policy;
- color metadata;
- alpha metadata.

It does not block waiting for a better frame.

---

## 8. Culling

The compiler may cull nodes when:

- effective opacity is zero;
- visibility is false;
- bounds are fully outside clip;
- source is unavailable and fallback is transparent;
- group result is unused for target surface.

Culling must preserve side-effect-free semantics. Rendering effects may not have hidden mutable side effects.

---

## 9. Offscreen Boundaries

Offscreen composition is required when:

- group opacity applies to combined children;
- group-level effect applies after child composition;
- nontrivial blend isolation is required;
- mask requires intermediate alpha;
- transition needs source and destination textures;
- color transform requires intermediate precision;
- effect chain cannot operate in-place safely.

The compiler declares offscreen intent; resource manager chooses concrete allocation.

---

## 10. Multi-Surface Compilation

Surfaces may share plan sections when they have compatible:

- target time;
- scene state;
- color working space;
- effect quality;
- resolution-independent operations.

Surface-specific operations remain separate.

The compiler must not share intermediates when it changes timing or color correctness.

---

## 11. Caching

Cacheable outputs include:

- resolved static hierarchy;
- transform chains;
- effect descriptors;
- text layout;
- vector tessellation descriptors;
- shader specialization keys;
- resource shape plans.

Cache keys include all semantic inputs and relevant generations.

Pointer identity alone is insufficient.

---

## 12. Diagnostics

Compile diagnostics include:

- unresolved source;
- nested scene cycle prevented;
- effect unavailable;
- unsupported blend mode fallback;
- resource estimate exceeded;
- invalid color metadata;
- surface incompatibility;
- culled due to invalid geometry.

Diagnostics include scene and item IDs.

---

## 13. Invariants

1. Compiler is side-effect free with respect to authoritative state.
2. Output contains no live GPU handles.
3. Plan identifies source frames and generations.
4. Compilation never waits indefinitely for media.
5. Optimization preserves semantics.
6. Missing sources produce explicit fallback or failure.
7. Multi-surface sharing preserves timing and color correctness.
8. Cache keys are generation- and configuration-aware.
9. Diagnostics map to semantic entities.
10. Output is bounded by validated scene depth and resource limits.

---

## 14. Required Tests

- simple flattening;
- nested groups;
- nested scenes;
- source unavailable;
- stale source frame;
- transform culling;
- group offscreen requirement;
- transition compile;
- multi-surface sharing;
- color conversion insertion;
- deterministic output;
- cache invalidation;
- resource estimate limit;
- diagnostic mapping.

---

## 15. AI Implementation Notes

Do not allocate GPU resources in the compiler.

Do not wait on source producers.

Do not use renderer backend types in the frame plan.

Make compiler output easy to snapshot-test and diff.

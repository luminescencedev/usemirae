# 201 — Scene Graph

**Status:** Proposed  
**Audience:** Scene, rendering, project, UI contributors  
**Canonical:** Yes  
**Required context:** `00-foundations/005-domain-model.md`, `200-rendering-overview.md`  
**Related ADRs:** ADR-0004, ADR-0012

---

## 1. Purpose

The scene graph represents semantic composition intent independently from renderer execution.

It is persisted as part of the project and mutated through commands.

---

## 2. Core Model

```rust
pub struct Scene {
    pub id: SceneId,
    pub name: SceneName,
    pub canvas: CanvasDescriptor,
    pub root_items: Vec<SceneItemId>,
    pub transition_defaults: TransitionDefaults,
    pub metadata: SceneMetadata,
}

pub struct SceneItem {
    pub id: SceneItemId,
    pub parent: Option<SceneItemId>,
    pub content: SceneItemContent,
    pub transform: Transform2D,
    pub crop: CropRect,
    pub opacity: NormalizedF32,
    pub visibility: VisibilityState,
    pub blend_mode: BlendMode,
    pub effects: Vec<EffectInstanceId>,
    pub masks: Vec<MaskInstance>,
    pub locked: bool,
}
```

Exact types may evolve, but the semantic separation is mandatory.

---

## 3. Scene Item Content

A scene item contains one of:

- source reference;
- group;
- nested scene reference;
- generated graphics node;
- future extension-defined source reference through SDK-safe representation.

A scene item does not contain a decoder, capture handle, GPU texture, or shader pipeline.

---

## 4. Ordering

Sibling order defines back-to-front compositing order.

The canonical project representation stores explicit ordered child IDs.

Reordering must preserve stable item identity.

Array index is not identity.

---

## 5. Transform Model

The canonical 2D transform includes:

- translation;
- rotation;
- scale;
- anchor/pivot;
- optional skew;
- bounding-box fit mode;
- crop applied in defined source-local order.

Transform composition order must be fixed and documented.

Canonical order:

```text
source-local geometry
→ crop
→ anchor offset
→ scale/skew
→ rotation
→ translation
→ parent transform
→ canvas transform
```

A later change requires an ADR because it affects persisted project semantics.

---

## 6. Coordinate Spaces

The scene system distinguishes:

- source pixel space;
- source normalized space;
- scene item local space;
- parent space;
- canvas logical space;
- target surface pixel space.

Conversions are explicit.

The project stores transforms in logical canvas units unless a source-specific property states otherwise.

---

## 7. Groups

A group:

- owns ordered child item references;
- applies a composed transform;
- may apply opacity, masks, blend mode, and effects to the grouped result;
- may require offscreen composition when group-level effects or blend semantics demand it.

Group hierarchy must be acyclic.

Maximum nesting depth is bounded by configuration and validated at load and mutation time.

---

## 8. Nested Scenes

A scene may reference another scene through a scene item.

Rules:

- nested scene dependency graph is acyclic;
- reference identity remains stable;
- nested scene evaluation uses the referenced scene's semantic state;
- per-instance overrides are explicit and limited;
- recursive cycles are rejected before commit;
- missing referenced scene becomes an unresolved diagnostic state, not silent deletion.

---

## 9. Visibility

Visibility state distinguishes:

- explicitly visible;
- explicitly hidden;
- inherited hidden through ancestor;
- temporarily suppressed by runtime production logic;
- unavailable because source runtime has no frame.

Persisted visibility stores user intent. Runtime availability remains separate.

---

## 10. Effects and Masks

Scene items reference effect instances by stable ID or inline schema where specified.

The graph stores semantic order:

```text
source
→ source-local effects
→ masks/crop
→ transform
→ group composition
→ group effects
→ parent composition
```

Exact effect order is defined in the effects specification and must be testable.

---

## 11. Graph Validation

Validation includes:

- all item IDs unique;
- all child references exist;
- parent relationships consistent;
- no group cycles;
- no nested scene cycles;
- bounded depth;
- finite transform values;
- valid crop ranges;
- valid effect references;
- supported blend modes;
- no duplicate child placement unless explicitly allowed.

---

## 12. Generations

The scene subsystem tracks:

- project state generation;
- scene graph generation;
- per-scene generation;
- optional per-item generation for cache invalidation.

Generation changes are derived from committed transactions.

The renderer does not infer scene changes by pointer identity alone.

---

## 13. Invalidation

A scene mutation emits a typed invalidation category:

- metadata only;
- transform;
- visibility;
- ordering;
- source reference;
- effect parameters;
- graph topology;
- canvas;
- nested dependency.

The frame compiler uses invalidation to reuse safe derived data.

---

## 14. Serialization

The scene graph schema:

- uses stable IDs;
- uses explicit enums;
- rejects non-finite numbers;
- has versioned defaults;
- avoids renderer-specific formats;
- supports migration;
- preserves unknown extension-owned configuration only through declared extension schema rules.

---

## 15. Invariants

1. Graph is acyclic.
2. Scene item identity is stable.
3. Sibling order is explicit.
4. Scene graph contains semantic state only.
5. Runtime availability does not rewrite persisted visibility.
6. Transforms use one canonical composition order.
7. Renderer never mutates the graph.
8. Missing references remain diagnosable.
9. Depth is bounded.
10. Every mutation produces invalidation metadata.

---

## 16. Required Tests

- reorder stability;
- transform composition;
- group opacity;
- group effect offscreen requirement;
- cycle detection;
- nested scene cycle detection;
- missing source reference;
- serialization round trip;
- migration fixture;
- depth limit;
- invalid float rejection;
- deterministic invalidation classification.

---

## 17. AI Implementation Notes

Do not add renderer handles to scene nodes.

Do not identify children by vector index.

Do not silently remove unresolved references during load.

Keep persisted visibility separate from runtime source availability.

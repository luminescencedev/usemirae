# 209 — Text and Graphics

**Status:** Proposed  
**Audience:** Rendering, UI, scene, accessibility contributors  
**Canonical:** Yes  
**Required context:** `201-scene-graph.md`, `206-gpu-resource-model.md`, `208-color-management.md`

---

## 1. Purpose

This subsystem renders text, shapes, images, and generated graphics as production scene content.

It must provide stable layout, high-quality scaling, predictable font handling, and bounded GPU resource use.

---

## 2. Text Source Model

A text source includes:

- text content;
- font family reference;
- font style and weight;
- size;
- line height;
- letter spacing;
- alignment;
- wrapping;
- maximum bounds;
- fill;
- stroke;
- shadow;
- background;
- language and script;
- direction;
- fallback policy.

Text content is persisted. Shaped glyph runs are derived.

---

## 3. Font Resolution

Font resolution distinguishes:

- bundled Mirae fonts;
- project-managed fonts;
- system fonts;
- extension-provided fonts where allowed.

A project using a non-portable system font records:

- requested family;
- resolved family at save time;
- portability warning;
- fallback chain.

Mirae must not silently substitute without diagnostics when layout materially changes.

---

## 4. Shaping

Text shaping must support:

- Unicode;
- bidirectional text;
- ligatures;
- combining marks;
- script shaping;
- fallback fonts;
- locale-sensitive behavior where applicable.

The shaping engine is isolated behind an interface.

Shaped output is cacheable by content and layout key.

---

## 5. Layout

Layout key includes:

- text;
- font identities and versions;
- size;
- style;
- width/height constraints;
- wrapping;
- alignment;
- locale/script;
- shaping options.

Layout produces immutable glyph runs and bounds.

---

## 6. Glyph Rendering

Possible strategies:

- signed-distance field atlas;
- multi-channel distance field;
- vector/path rendering;
- high-resolution bitmap atlas;
- direct raster upload for complex glyphs.

The implementation may combine strategies.

The chosen path must preserve quality at expected production scales.

---

## 7. Atlas Management

Glyph atlases are:

- device-generation scoped;
- byte-bounded;
- page-based;
- eviction-aware;
- keyed by font/glyph/render mode;
- safe across in-flight frames.

Eviction must not invalidate glyphs referenced by submitted GPU work.

---

## 8. Shapes

Initial generated shapes:

- rectangle;
- rounded rectangle;
- ellipse;
- line;
- polygon/path;
- gradient;
- border;
- shadow.

Shape parameters are semantic and renderer-independent.

---

## 9. Images and Logos

Static image assets use the asset registry and media decode path.

The graphics subsystem may cache decoded and uploaded representations.

Color and alpha metadata remain explicit.

---

## 10. Animation

Text and graphics properties may be animated through the motion system.

Layout-affecting animation and transform-only animation are distinguished.

Transform-only animation should not re-shape text each frame.

---

## 11. Accessibility and Safety

Production text source editing should support:

- content length limits;
- control-character handling;
- missing glyph diagnostics;
- safe external-data insertion;
- optional text overflow indicators;
- accessible UI editing.

Rendered production output itself is visual, but authoring controls remain accessible.

---

## 12. Determinism

For the same:

- font file/version;
- shaping engine version;
- text;
- layout parameters;

layout should be stable.

System font updates may change output and must be diagnosable.

Portable projects should prefer managed font assets.

---

## 13. Invariants

1. Text layout is derived, not persisted as GPU glyph data.
2. Font substitution is diagnosable.
3. Glyph atlases are bounded.
4. In-flight glyph data is not evicted prematurely.
5. Transform-only changes do not force reshaping.
6. Text color enters the defined working color pipeline.
7. Unicode shaping is supported through a dedicated engine.
8. System font dependence is explicit.
9. Generated shapes use semantic parameters.
10. External text input is bounded.

---

## 14. Required Tests

- Latin text;
- Arabic or Hebrew bidi;
- CJK fallback;
- combining marks;
- ligatures;
- wrapping;
- font substitution;
- portable font asset;
- atlas eviction;
- in-flight atlas safety;
- large-scale text quality;
- transform-only animation cache;
- missing glyph diagnostics.

---

## 15. AI Implementation Notes

Do not use browser DOM text rendering as the production renderer.

Do not persist shaped glyph IDs as portable project data.

Do not evict atlas pages still referenced by in-flight submissions.

Keep text layout cache keys complete and version-aware.

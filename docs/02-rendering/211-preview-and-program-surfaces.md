# 211 — Preview and Program Surfaces

**Status:** Proposed  
**Audience:** Rendering, runtime, UI, output contributors  
**Canonical:** Yes  
**Required context:** `01-runtime/103-frame-scheduler.md`, `200-rendering-overview.md`, `210-effects-and-transitions.md`  
**Related ADRs:** ADR-0015

---

## 1. Purpose

This document defines preview and program as independent production concepts and rendering surfaces.

---

## 2. Program

Program is the composition currently feeding designated live outputs.

Program state includes:

- current program scene;
- active transition;
- program surface policy;
- program audio association;
- generation;
- output routing.

Program is authoritative engine state.

---

## 3. Preview

Preview is the operator-prepared composition not yet live.

Preview may include:

- selected preview scene;
- temporary transform edits;
- transition preparation;
- diagnostic overlays;
- safe-area guides;
- selection outlines;
- source status indicators.

Preview overlays must not enter program unless explicitly implemented as production graphics.

---

## 4. Independent Scheduling

Preview and program have separate:

- surface IDs;
- cadence;
- resolution;
- quality policy;
- overlay policy;
- color/display policy;
- backpressure policy.

Program must not stall because the operator preview window is occluded, resized, or disconnected.

---

## 5. Studio and Direct Modes

### 5.1 Studio mode

- preview and program are separate;
- transition action promotes preview toward program;
- operator may prepare off-air changes.

### 5.2 Direct mode

- selected scene changes program directly according to configured transition policy;
- preview may mirror program or remain absent.

Mode is explicit state.

---

## 6. Temporary Preview Edits

The UI may show local optimistic transforms in preview before authoritative commit.

The engine may also support bounded preview-state overlays.

Temporary preview state:

- is generation-scoped;
- is never persisted unless committed;
- is discarded on reconnect or conflict;
- cannot change program without an explicit command.

---

## 7. Transition Execution

When a transition begins:

- source program scene snapshot is frozen semantically for transition start;
- destination preview scene snapshot is selected;
- transition runtime owns progress;
- program surface renders transition result;
- preview policy determines whether preview switches, remains, or displays destination.

Exact scene update behavior must be consistent and tested.

---

## 8. Surface Resizing

Preview resize:

- creates new surface generation;
- may reduce render resolution;
- must not change program output dimensions;
- may coalesce repeated resize events;
- preserves aspect and fit policy.

Program output size changes require explicit output or project configuration commands.

---

## 9. UI Presentation

The UI receives a surface handle or frame-sharing contract appropriate to the platform.

The control IPC carries metadata, not raw frame bytes.

The UI must display:

- stale surface state;
- renderer recovery;
- color approximation warning;
- dropped preview frames if significant;
- program live status.

---

## 10. Diagnostic Overlays

Preview-only overlays may include:

- source bounds;
- crop handles;
- safe areas;
- grid;
- audio-safe labels;
- performance overlay;
- unavailable source marker.

Program overlays require separate production scene content or explicit program diagnostic mode.

---

## 11. Headless Operation

The engine must support program outputs without an active UI preview surface.

Preview surface can be created, destroyed, or recreated independently.

---

## 12. Invariants

1. Program is engine-authoritative.
2. Preview and program use independent surface generations.
3. Preview failure does not stop program.
4. UI resize does not change program dimensions.
5. Preview overlays never leak into program by default.
6. Temporary preview edits do not mutate program.
7. Transition progress is engine-timed.
8. Headless program operation is supported.
9. Surface transport is separate from control IPC.
10. Studio/direct mode is explicit.

---

## 13. Required Tests

- studio mode switch;
- direct mode switch;
- preview resize during stream;
- UI disconnect;
- headless output;
- preview overlay isolation;
- optimistic edit conflict;
- transition destination snapshot;
- preview surface recreation;
- program unchanged on preview failure;
- color approximation status.

---

## 14. AI Implementation Notes

Do not model preview as merely a React canvas.

Do not stop or resize program when the UI preview changes.

Do not send raw rendered frames through generic IPC.

Keep overlay inclusion explicit per surface.

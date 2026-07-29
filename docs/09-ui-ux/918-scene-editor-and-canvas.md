# 918 — Scene Editor and Canvas

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## Canvas Responsibilities

- display Preview or editable scene;
- selection and multi-selection;
- transform handles;
- snapping and guides;
- safe areas;
- zoom/pan/fit;
- crop and mask modes;
- context toolbar;
- diagnostics overlay in explicit mode.

## Interaction

- Space + drag pans;
- wheel/trackpad zooms around cursor;
- F fits canvas;
- arrow keys nudge;
- Shift changes step;
- handles remain legible at common zoom levels;
- overlays render independently from encoded Program.

## Snapping

Snap targets:

- canvas edges/center;
- safe areas;
- other item edges/centers;
- user guides;
- grid.

Snapping is visually indicated and temporarily bypassable.

## Separation

Editor overlays, hover states, guides, and selected handles never enter Program or recording output.

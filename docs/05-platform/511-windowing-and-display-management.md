# 511 — Windowing and Display Management

**Status:** Proposed  
**Audience:** Shell, rendering, UI, platform contributors  
**Canonical:** Yes  
**Required context:** `501-desktop-shell.md`, `02-rendering/211-preview-and-program-surfaces.md`

---

## 1. Purpose

This document defines top-level windows, monitor identity, DPI/scaling, fullscreen/projector surfaces, geometry persistence, and display-change handling.

---

## 2. Window Roles

- main control window;
- detached panel;
- fullscreen/projector preview;
- program monitor;
- diagnostics window;
- recovery window;
- extension-constrained surface.

Each role has explicit lifecycle and persistence policy.

---

## 3. Display Record

A display record includes:

- platform display reference;
- stable/ephemeral identity classification;
- logical bounds;
- physical pixel bounds;
- scale factor;
- refresh modes;
- HDR/EDR capability;
- color profile reference;
- primary status;
- connection state;
- generation.

---

## 4. Coordinate Systems

Mirae distinguishes:

- desktop logical coordinates;
- window logical coordinates;
- window physical pixels;
- display physical pixels;
- renderer surface pixels;
- scene canvas coordinates.

Conversions use current display scale and window generation.

---

## 5. DPI and Scaling

The shell responds to per-monitor scale changes.

Rules:

- UI layout uses logical units;
- renderer surface uses physical pixels;
- moving between displays may recreate surface generation;
- persisted geometry stores logical intent and display reference;
- invalid/offscreen geometry is repaired on restore.

---

## 6. Fullscreen and Projector Windows

A projector surface:

- may be borderless/fullscreen;
- owns independent render surface;
- can target preview or program;
- may hide cursor;
- must not expose editor overlays unless selected;
- handles display removal gracefully.

Closing projector does not stop program output.

---

## 7. Display Hotplug

On display change:

- update display snapshot;
- invalidate affected surface generation;
- move inaccessible windows to safe display;
- preserve intended target for possible reconnect;
- report HDR/color changes;
- avoid resizing unrelated outputs.

---

## 8. Geometry Persistence

Persist:

- window role;
- logical size/position;
- display hint;
- maximized/fullscreen state;
- panel layout reference.

Do not persist raw native handles or assume monitor IDs survive hardware changes.

---

## 9. Color and HDR

Window surfaces declare:

- requested color mode;
- actual display capability;
- OS compositor behavior;
- fallback;
- approximation status.

Preview display limitations do not alter encoded program color.

---

## 10. Input and Focus

Shell handles:

- keyboard focus;
- global shortcuts through separate capability;
- pointer capture;
- drag/drop;
- accessibility focus;
- fullscreen escape policy.

Projector windows should not steal operator focus unexpectedly.

---

## 11. Invariants

1. Logical and physical coordinates are distinct.
2. Surface generation changes on incompatible resize/reconfigure.
3. Display removal does not stop program.
4. Offscreen windows are repaired.
5. Projector overlays are explicit.
6. Raw monitor IDs are not assumed permanent.
7. UI scaling does not change scene canvas.
8. HDR preview status is explicit.
9. Geometry persistence is role-based.
10. Focus behavior is predictable.

---

## 12. Required Tests

- per-monitor DPI move;
- display removal;
- fullscreen projector;
- window restore after monitor change;
- surface resize generation;
- HDR display switch;
- offscreen geometry repair;
- multiple projector windows;
- focus behavior;
- cursor hide;
- Wayland positioning limitation;
- macOS fullscreen lifecycle.

---

## 13. AI Implementation Notes

Do not use UI logical dimensions as render-target pixel dimensions.

Do not persist native monitor handles.

Do not let projector-window closure affect encoded program.

Keep display capability separate from project output configuration.

# 904 — Desktop Shell Layout

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## 1. Reference Geometry

Reference design viewport: **1440 × 900 logical pixels**.

Recommended regions:

- titlebar: 44 px;
- left dock: 248 px default, 180 px minimum;
- right inspector: 320 px default, 260 px minimum;
- bottom dock: 236 px default, 156 px minimum;
- canvas: consumes all remaining space;
- resize handles: 4 px visual target, larger invisible hit area.

## 2. Titlebar

Contains:

- native window controls;
- Mirae mark and product name;
- project name and mode;
- save/connection state;
- Record;
- Go Live;
- optional workspace/command controls.

The titlebar must remain draggable outside interactive controls.

## 3. Adaptive Collapse

When width decreases:

1. inspector collapses to icon rail;
2. left source dock narrows;
3. bottom mixer switches to horizontal scroll or compact channels;
4. labels shorten before essential controls disappear;
5. canvas remains visible.

The desktop application is not redesigned as a mobile page. It degrades as a professional windowed tool.

## 4. Multi-Monitor

Projector/Program windows are independent surfaces. Closing them does not stop output. Detached panels preserve role, display hint, and logical geometry.

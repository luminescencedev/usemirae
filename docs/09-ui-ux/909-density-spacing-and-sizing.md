# 909 — Density Spacing and Sizing

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## 1. Base Rhythm

Base spacing unit: 4 px.

Primary scale: `2, 4, 6, 8, 12, 16, 20, 24, 32, 40`.

## 2. Control Heights

- compact: 26 px;
- standard: 30 px;
- comfortable: 34 px;
- primary production action: 36–40 px;
- titlebar action: 34 px.

## 3. Radii

- fields/compact buttons: 6 px;
- standard buttons/rows: 8 px;
- panels: 10 px;
- floating surfaces: 12–14 px;
- hero/application frame: 16–18 px.

Rounded geometry is controlled. The application must not become a collection of large pills.

## 4. Density Modes

Initial release ships one optimized density. A future compact/comfortable preference may alter component tokens, not individual feature CSS.

## 5. Hit Targets

Visual controls may be compact, but pointer hit areas and keyboard focus must remain usable. Resize handles use an expanded invisible interaction zone.

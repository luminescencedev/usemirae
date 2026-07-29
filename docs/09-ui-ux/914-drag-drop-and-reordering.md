# 914 — Drag Drop and Reordering

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## Scope

Drag and drop applies to scenes, sources, groups, effect stacks, mixer routing targets, assets, and dock tabs.

## Requirements

- pointer, touchpad, and keyboard sensors;
- visible drag handle where accidental drag is costly;
- clear insertion marker;
- auto-scroll with bounded speed;
- expand-on-hover only after a delay;
- invalid targets explain why;
- operation remains undoable;
- engine command commits the final order.

## Keyboard Equivalent

- focus item;
- activate move mode;
- arrows choose destination;
- Space/Enter drops;
- Escape cancels;
- assistive text announces position and valid targets.

## Performance

The dragged item uses a lightweight overlay. The entire scene tree must not rerender on every pointer movement.

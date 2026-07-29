# 917 — Property Inspector

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## Purpose

The inspector is the precise counterpart to direct canvas manipulation.

## Structure

- selected entity header;
- common status/visibility/lock controls;
- grouped properties;
- transform;
- appearance;
- effects;
- source-specific settings;
- advanced diagnostics.

## Numeric Fields

Support:

- direct input;
- arrow increments;
- Shift/Alt modified increments;
- drag scrubbing;
- units;
- expressions only if explicitly designed;
- mixed values for multi-selection;
- reset and keyframe/automation hooks later.

## Draft and Commit

Typing remains a local draft until valid commit. Drag scrubbing uses coalesced commands. Engine rejection restores authoritative value and displays a contextual reason.

## Multi-Selection

Only compatible common properties are shown. Mixed values are explicit. Changing a common value applies one transaction to all selected entities.

# 928 — UI Implementation Backlog

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## UI Sprint A — Foundations

- UI-0001: import semantic tokens;
- UI-0002: create dark theme root;
- UI-0003: establish typography and icon wrappers;
- UI-0004: Button/IconButton/ToggleButton;
- UI-0005: TextField/NumberField/Select;
- UI-0006: Tooltip/Menu/Popover/Dialog;
- UI-0007: Status/Badge/Toast/Incident;
- UI-0008: panel and resize-handle abstraction;
- UI-0009: component fixture host;
- UI-0010: accessibility and visual-regression baseline.

## UI Sprint B — Shell

- UI-0101: native titlebar layout;
- UI-0102: workspace frame;
- UI-0103: left/right/bottom docks;
- UI-0104: persisted layout schema;
- UI-0105: minimum-window behavior;
- UI-0106: command palette;
- UI-0107: shortcut registry and display;
- UI-0108: engine connection/save status;
- UI-0109: Record/Go Live state controls;
- UI-0110: shell E2E tests.

## UI Sprint C — Edit Workspace

- UI-0201: SceneTree;
- UI-0202: SourceRow and source groups;
- UI-0203: canvas surface;
- UI-0204: selection and transform overlay;
- UI-0205: inspector sections;
- UI-0206: number field scrubbing;
- UI-0207: compact mixer channel;
- UI-0208: drag/reorder keyboard model;
- UI-0209: empty/failure/recovery variants;
- UI-0210: Edit visual regression.

## UI Sprint D — Studio and Outputs

- UI-0301: Preview/Program surfaces;
- UI-0302: Cut/Auto transition controls;
- UI-0303: live timer and Program state;
- UI-0304: destination cards;
- UI-0305: reconnect/degraded/failure states;
- UI-0306: recording finalization flow;
- UI-0307: output incident model;
- UI-0308: Studio accessibility pass;
- UI-0309: minimum-window Studio layout;
- UI-0310: Studio visual regression.

## Completion Rule

A UI ticket is complete only after keyboard behavior, loading/error states, engine-state ownership, tests, and visual fixture are included.

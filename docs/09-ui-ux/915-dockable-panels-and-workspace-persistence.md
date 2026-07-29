# 915 — Dockable Panels and Workspace Persistence

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## Panel Model

Panels have stable roles and IDs. Initial roles:

- Scenes/Sources;
- Inspector;
- Mixer;
- Timeline;
- Outputs;
- Assets;
- Diagnostics;
- Extensions.

## Behavior

- resize with keyboard and pointer;
- collapse to remembered size;
- optional detach where platform supports it;
- tab groups for secondary tools;
- minimum/maximum constraints;
- reset workspace command;
- named workspace presets in later releases.

## Persistence

Persist UI layout separately from project semantic state unless the user explicitly saves a project-specific workspace.

Stored layout includes role, ratio/size, collapsed state, tab group, display hint, and schema version.

## Safety

A panel layout failure falls back to the default layout. It never prevents project open or engine operation.

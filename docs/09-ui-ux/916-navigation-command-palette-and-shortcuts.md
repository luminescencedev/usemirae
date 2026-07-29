# 916 — Navigation Command Palette and Shortcuts

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## Navigation

Workspace navigation is available through visible controls and commands. It does not occupy a permanent oversized sidebar.

## Command Palette

The palette provides:

- commands;
- scenes/sources/assets search;
- settings navigation;
- device and output actions;
- recent projects;
- future contextual AI actions.

It uses fuzzy search, category grouping, shortcuts, scope, and permission-aware disabled states.

## Shortcut Model

- global application shortcuts;
- workspace shortcuts;
- context shortcuts;
- text-editing exceptions;
- user remapping;
- conflict detection;
- platform-specific display.

## Safety

High-risk production commands are visually distinct and may require hold/confirm or a second deliberate step. Search results never obscure whether an action affects Preview, Program, recording, or stream.

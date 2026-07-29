# 913 — Interaction Patterns

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## Selection

- single click selects;
- modifier click extends selection where supported;
- shift selects range in ordered lists;
- Escape clears local selection or closes the topmost transient layer;
- selection remains visible when focus moves to inspector.

## Direct Manipulation

- drag canvas handles;
- numeric fields allow typing, arrows, and scrubbing;
- Shift increases step, Alt/Option decreases step;
- double click resets only when discoverable and undoable;
- all direct manipulation produces coalesced commands.

## Confirmation

Require deliberate confirmation for:

- stopping active stream/recording in unsafe situations;
- deleting referenced assets;
- replacing externally modified project;
- removing an extension namespace;
- destructive output profile changes.

Ordinary undoable edits should not be interrupted by confirmation dialogs.

## Context Menus

Context menus expose shortcuts and scope-relevant actions. They never hide the only path to an essential operation.

# 900 — UI UX Overview

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## 1. Purpose

This section defines the complete operator experience for Mirae: visual language, workspace organization, components, motion, interaction, accessibility, performance, and implementation workflow.

It is the UI counterpart to the engine architecture. It does not change domain ownership: the engine remains authoritative and the UI remains a projection plus temporary local drafts.

## 2. Selected Direction

The selected direction is **Obsidian Precision**:

- dark-first;
- desktop-first;
- dense without being cramped;
- visually calm;
- operational states are unmistakable;
- canvas and content dominate chrome;
- floating glass is restrained;
- motion communicates continuity rather than decoration;
- every major screen should be credible as a product screenshot.

## 3. Visual Deliverables

The canonical visual references are:

- the 20-slide presentation in this pack;
- Edit Workspace;
- Studio Mode;
- Audio Mixer;
- Command Palette;
- token and component specifications.

They define intent and hierarchy. Figma and production code may refine exact measurements while preserving the system.

## 4. UX Invariants

1. Preview and Program are never ambiguous.
2. Live, recording, reconnecting, and failed states are distinguishable by text, shape, and color.
3. The canvas is the visual priority.
4. Panels are dense, resizable, and persistent.
5. Every pointer interaction has a keyboard alternative.
6. High-frequency metrics do not rerender the full application.
7. Engine state and local drafts stay separate.
8. Destructive production actions require deliberate confirmation.
9. Reduced motion is first-class.
10. Library defaults are never shipped without Mirae styling.

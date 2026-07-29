# 903 — Information Architecture

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## 1. Primary Workspaces

- **Edit** — scenes, sources, canvas, inspector, mixer.
- **Studio** — Preview, Program, transitions, outputs, mixer.
- **Audio** — expanded routing, monitoring, tracks, processing.
- **Outputs** — destinations, encoder profiles, recording, replay.
- **Assets** — imported and managed project assets.
- **Diagnostics** — component health, performance, incidents.
- **Settings** — application, devices, shortcuts, appearance, extensions.

## 2. Persistent Global Areas

- native titlebar and project identity;
- connection/save status;
- Record and Go Live controls;
- workspace switcher;
- command palette access;
- status/incident surface.

## 3. Contextual Areas

- left scene/source tree;
- central canvas or Preview/Program surfaces;
- right property/output inspector;
- bottom mixer/timeline/diagnostics dock;
- floating canvas toolbar;
- command palette and dialogs.

## 4. Navigation Rule

Changing workspace changes tool emphasis, not the active project or production state.

A live output continues while the user navigates to settings or diagnostics unless the owning engine operation stops it.

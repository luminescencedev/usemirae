# 925 — UI Testing and Visual Regression

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## Layers

- token tests;
- primitive interaction tests;
- component fixture tests;
- accessibility tests;
- fake-engine feature tests;
- Playwright desktop-shell flows;
- visual regression;
- performance smoke.

## Visual Matrix

Capture:

- default dark theme;
- hover/focus/active;
- loading/error/disabled;
- compact and minimum supported window;
- 100%, 125%, 150%, and 200% scale where practical;
- reduced motion;
- high contrast;
- Windows/macOS/Linux rendering differences.

## Baselines

Visual baselines are reviewed, not automatically accepted. A changed screenshot must identify the intended token/component/feature change.

## Critical Screens

- empty project;
- Edit workspace;
- Studio mode live;
- output reconnecting;
- property validation failure;
- command palette;
- settings/shortcuts;
- diagnostics incident.

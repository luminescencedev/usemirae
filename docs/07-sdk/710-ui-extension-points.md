# 710 — UI Extension Points

**Status:** Proposed  
**Audience:** SDK, UI, accessibility, security contributors  
**Canonical:** Yes  
**Required context:** `05-platform/501-desktop-shell.md`, `06-quality/613-accessibility.md`, `06-quality/614-localization.md`  
**Related ADRs:** ADR-0051

---

## 1. Purpose

This document defines constrained, accessible, localizable UI contributions without allowing arbitrary extension code to share the control UI's trusted origin.

---

## 2. Contribution Types

- settings section;
- source configuration form;
- output configuration form;
- dock/panel;
- command palette action;
- context-menu action;
- toolbar action;
- status indicator;
- modal or wizard requested through host;
- provider diagnostics view.

Each point has a schema and placement policy.

---

## 3. Declarative UI

Preferred contributions use declarative component schemas:

- text;
- button;
- toggle;
- select;
- number field;
- text field;
- form section;
- list/table;
- status badge;
- progress;
- permission request;
- file picker action;
- documentation link.

The host renders components using Mirae design system and accessibility behavior.

---

## 4. Isolated Rich UI

If richer UI is supported:

- it runs in isolated origin/webview/frame;
- receives scoped message bridge;
- cannot access control UI DOM;
- cannot navigate top-level shell;
- has CSP;
- has network capability restrictions;
- has size/performance quotas;
- exposes accessibility metadata.

---

## 5. State and Actions

UI contributions receive:

- extension-owned view state;
- filtered engine projections;
- operation progress;
- capability status.

Actions invoke typed extension or host commands.

UI does not mutate project state locally as authority.

---

## 6. Styling

Declarative UI uses host tokens for:

- color;
- typography;
- spacing;
- focus;
- dark/light/high-contrast;
- reduced motion.

Extensions cannot override global application CSS.

---

## 7. Accessibility

Requirements:

- keyboard operation;
- semantic labels;
- focus order;
- error association;
- screen-reader status;
- reduced motion;
- localization;
- zoom and text expansion.

The host rejects invalid contribution schemas.

---

## 8. Localization

Extension provides message catalogs keyed by locale.

Host controls:

- fallback;
- permission framing;
- error wrappers;
- date/number formatting;
- shortcut display.

---

## 9. UI Quotas

Limits include:

- number of panels;
- refresh rate;
- message rate;
- retained state;
- notification rate;
- DOM/node count for rich UI;
- CPU/memory;
- modal frequency.

---

## 10. Invariants

1. Extension UI does not share trusted control origin.
2. Preferred UI is declarative.
3. Actions are typed and capability-checked.
4. Host design tokens control appearance.
5. Accessibility is mandatory.
6. Localization uses stable keys.
7. UI updates are rate-limited.
8. Extension cannot inject global CSS/script.
9. Modal/notification abuse is bounded.
10. UI failure does not affect engine.

---

## 11. Required Tests

- declarative settings form;
- command action;
- isolated rich panel;
- DOM access rejection;
- top-level navigation rejection;
- keyboard operation;
- screen-reader labels;
- dark/light/high-contrast;
- reduced motion;
- locale fallback;
- update-rate quota;
- UI crash isolation.

---

## 12. AI Implementation Notes

Do not let extension scripts run in the same origin as the control UI.

Do not allow global CSS injection.

Do not build custom inaccessible controls when host declarative components exist.

Route all actions through typed commands.

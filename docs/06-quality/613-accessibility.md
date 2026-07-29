# 613 — Accessibility

**Status:** Proposed  
**Audience:** UI, shell, QA, product contributors  
**Canonical:** Yes  
**Required context:** `05-platform/501-desktop-shell.md`, `01-runtime/109-ui-engine-synchronization.md`  
**Related ADRs:** ADR-0045

---

## 1. Purpose

Mirae's control interface must be operable by keyboard, assistive technology, and users with visual, motor, cognitive, or motion sensitivities.

Accessibility is a release requirement.

---

## 2. Keyboard Operation

Every essential workflow must support keyboard-only use:

- project open/save;
- scene/source navigation;
- transform editing through numeric controls;
- start/stop output;
- audio controls;
- dialogs;
- settings;
- error recovery.

Focus order is logical and visible.

---

## 3. Semantic Structure

Controls require:

- correct role;
- accessible name;
- state;
- value;
- description when needed;
- relation to error/help text;
- live-region behavior for status;
- grouping.

Custom controls must match native interaction expectations.

---

## 4. Focus Management

Rules:

- dialogs trap focus appropriately;
- closing returns focus;
- command results do not move focus unexpectedly;
- reconnect/state patches preserve focus where possible;
- virtualized lists expose stable navigation;
- drag-and-drop has keyboard alternative.

---

## 5. Visual Accessibility

Requirements:

- sufficient contrast;
- non-color-only status;
- scalable text;
- high-DPI support;
- visible focus;
- zoom/layout resilience;
- clear error states;
- no critical information only in transient toast.

---

## 6. Motion

Respect reduced-motion preference.

Reduced motion may:

- remove decorative animation;
- shorten transitions;
- replace parallax;
- avoid large zoom/pan;
- preserve functional production preview behavior.

Project output effects are not automatically changed by UI accessibility preferences.

---

## 7. Audio Accessibility

UI supports:

- text labels for meter/clipping state;
- keyboard gain changes;
- numeric values;
- non-audio-only alerts;
- visual indication of monitoring state.

---

## 8. Live Updates

High-frequency values are not announced continuously.

Screen-reader announcements are reserved for:

- output started/stopped;
- critical error;
- recovery completed;
- permission required;
- save result;
- major source state change.

Rate limiting prevents announcement floods.

---

## 9. Localization Interaction

Accessible names and shortcuts account for localization.

Do not concatenate fragments that become grammatically incorrect or inaccessible.

---

## 10. Testing

Use:

- automated accessibility checks;
- keyboard tests;
- screen-reader smoke tests;
- focus-order tests;
- contrast checks;
- reduced-motion tests;
- zoom/scaling tests;
- error-announcement tests.

Automation does not replace manual review.

---

## 11. Invariants

1. Essential workflows are keyboard-operable.
2. Focus is visible and predictable.
3. Status is not color-only.
4. Custom controls expose semantics.
5. Reduced motion is respected.
6. High-frequency metrics do not flood assistive tech.
7. Errors are associated with controls.
8. Project output is not changed by UI accessibility settings.
9. Accessibility survives localization.
10. Release gates include manual and automated checks.

---

## 12. Required Tests

- full keyboard smoke;
- scene list navigation;
- source reorder alternative;
- modal focus;
- output start announcement;
- meter non-spam;
- contrast;
- 200% zoom;
- reduced motion;
- dark/light mode;
- localized labels;
- reconnect focus preservation.

---

## 13. AI Implementation Notes

Do not use clickable `div` elements without proper semantics and keyboard behavior.

Do not rely on color alone.

Do not announce every meter update.

Preserve focus across state patches and optimistic reconciliation.

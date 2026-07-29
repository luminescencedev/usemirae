# 923 — Accessibility and Reduced Motion

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## Requirements

- complete keyboard operation;
- visible focus;
- semantic names/roles/states;
- predictable dialog focus;
- non-color-only status;
- text zoom and high DPI;
- screen-reader-safe status announcements;
- reduced motion;
- keyboard-equivalent drag and drop;
- accessible resizers and numeric scrubbing alternatives.

## Live Data

Meters and frame statistics are not announced continuously. Assistive technology receives summarized state changes and user-requested values.

## Contrast

Token combinations are validated. Disabled controls remain readable. Focus and operational borders remain visible against every surface.

## Motion

`MotionConfig reducedMotion="user"` or equivalent is applied globally. CSS media queries cover non-Motion transitions.

## Release Gate

Keyboard and screen-reader smoke tests are required for every critical workspace before stable release.

# 912 — Motion System

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## 1. Principle

Motion communicates hierarchy, continuity, ownership, and state. It must never delay an operator.

## 2. Durations

- hover/press: 90–140 ms;
- tooltip/menu: 120–180 ms;
- popover/dialog: 160–220 ms;
- panel reveal: 200–280 ms;
- workspace transition: 220–320 ms;
- shared-layout movement: spring, duration perceptually below 350 ms.

## 3. Easing

- standard out: `cubic-bezier(0.16, 1, 0.3, 1)`;
- standard in/out: `cubic-bezier(0.65, 0, 0.35, 1)`;
- press: short ease-out with 0.98–0.99 scale;
- shared selection bar: restrained spring, no bounce.

## 4. Motion Uses

- active list indicator;
- panel expand/collapse;
- drag placeholder and reorder;
- canvas selection/toolbar;
- Preview to Program transition affordance;
- command palette;
- progress/recovery state.

## 5. Prohibitions

- no bouncy production controls;
- no animated live/record state that reduces legibility;
- no background movement during focused editing;
- no layout animation for high-frequency meter updates.

## 6. Reduced Motion

Replace movement with immediate state changes, fades below 100 ms, or static indicators. Project output transitions are independent from UI motion preference.

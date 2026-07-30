---
name: mirae-ui-engineering
description: Build or review any user-visible Mirae desktop UI, including temporary milestone screens, diagnostics, workspace chrome, panels, controls, empty states, engine connection states, interaction, motion, and accessibility. Use whenever a ticket changes apps/control-ui, packages/ui-kit, UI-facing bridge behavior, or visual documentation. Enforce Mirae's Obsidian Precision design system, product-shaped vertical slices, React Aria encapsulation, French owner communication, English implementation, and rendered visual validation.
---

# Mirae UI Engineering

## Establish authority

1. Read `CLAUDE.md`, the active ticket, and `DEPENDENCY_VERSIONS.md`.
2. Read the relevant files under `docs/09-ui-ux/`, especially `900`, `904`, `907`, `910`, `911`, and `923` when applicable.
3. Inspect `docs/assets/visual-direction-v1/` and existing `@mirae/ui-kit` components.
4. Treat Mirae documentation and this skill as authoritative over external design skills.
5. Explain plans, progress, blockers, and completion reports to the project owner in French.
6. Write source code, identifiers, comments, tests, product copy, documentation, branches, commits, and pull requests in English.

## Build a product-shaped vertical slice

- Implement the smallest compliant, product-shaped vertical slice.
- Keep capability narrow when necessary, but never make visible UI visually disposable.
- Place temporary behavior in its intended long-term product region.
- Use honest loading, empty, unavailable, disconnected, degraded, success, and error states.
- Never invent engine, project, render, source, output, or connection data to make a screen look complete.
- Record a removal or relocation ticket for any deliberately temporary surface.

## Preserve the component boundary

- Import public controls from `@mirae/ui-kit` in feature code.
- Keep `react-aria-components` private to `@mirae/ui-kit`.
- Wrap replacement-sensitive third-party APIs behind Mirae-owned components or adapters.
- Use React Aria collection interactions for trees, lists, tables, menus, and collection drag-and-drop.
- Use dnd-kit only for freeform or spatial interactions such as canvas positioning and custom drop geometry.
- Use TanStack Virtual only when the selected React Aria collection or virtualizer does not satisfy the measured requirement.
- Do not add or replace a primitive library, component kit, icon library, motion library, router, state manager, or styling system without a dedicated approved ticket.

## Apply Mirae visual quality

- Use semantic Mirae tokens for color, spacing, type, border, radius, elevation, focus, and motion.
- Do not ship browser defaults, raw third-party styling, copied Tailwind examples, shadcn blocks, or generic dashboard visuals.
- Do not add arbitrary color literals, shadows, radii, or spacing values in feature code.
- Avoid static inline style objects for ordinary layout and visual styling. Reserve inline styles for runtime-computed geometry, transforms, and measured values.
- Make the canvas and operator content dominate the chrome.
- Keep persistent structural panels opaque; reserve glass for approved overlays.
- Communicate operational status using text plus shape/icon plus color.
- Preserve visible focus, keyboard workflows, reduced motion, contrast, and screen-reader labels.
- Prefer a smaller polished surface over a larger unfinished surface.

Read `references/visual-quality-checklist.md` before completing a visible ticket.

## Place diagnostics correctly

- Show engine connection as a compact persistent status, normally in the titlebar or status region.
- Put protocol versions, session identifiers, retry counters, frame limits, raw transport details, and developer diagnostics in a dedicated diagnostics surface.
- Preserve workspace structure during disconnection; present an honest degraded state instead of replacing the app with a debug card.
- Separate development diagnostics from normal operator controls.

## Use motion deliberately

- Animate only when motion explains continuity, state, spatial relation, or feedback.
- Keep high-frequency keyboard actions immediate.
- Prefer transform and opacity for animation.
- Respect reduced motion.
- Use the installed `emil-design-eng` skill for meaningful interaction or motion work.
- Use `review-animations` after implementing non-trivial motion.
- Do not let an external skill override Mirae tokens, layout architecture, component boundaries, or library decisions.

Read `references/interaction-and-motion.md` for the Mirae-specific motion constraints.

## Validate the rendered result

1. Run component tests, type checks, lint, and affected tests.
2. Launch the real control UI or native shell.
3. Inspect the rendered result at `1440 x 900` logical pixels.
4. Inspect at least one narrower desktop size.
5. Exercise all relevant loading, empty, disconnected, degraded, success, and error states.
6. Check hierarchy, density, clipping, overflow, focus, keyboard behavior, contrast, disabled states, and reduced motion.
7. Use `webapp-testing` for browser-level interaction and screenshots when installed.
8. Use `frontend-design-review` before completion when installed.
9. Test the actual Wry/WebView2 shell when the change depends on native-shell behavior.
10. Do not claim visual completion without observing the rendered interface.

Read `references/visual-validation.md` for the completion evidence format.

## Report completion

Report in French:

- the ticket and visible behavior delivered;
- files and components changed;
- states implemented;
- accessibility and keyboard behavior;
- rendered sizes inspected;
- tests and commands with results;
- screenshots or visual observations;
- remaining visual or product debt;
- follow-up tickets.

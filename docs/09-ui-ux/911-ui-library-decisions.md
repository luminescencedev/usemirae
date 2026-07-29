# 911 — UI Library Decisions

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## Selected Stack

| Concern | Decision | Reason |
|---|---|---|
| Styling | Tailwind CSS v4 + semantic CSS variables | fast composition with runtime theme tokens |
| Headless primitives | Base UI | accessible, unstyled primitives suitable for custom desktop controls |
| Motion | Motion for React + CSS transitions | layout continuity, gestures, reduced-motion support |
| Drag and drop | dnd-kit | sensor model and keyboard-equivalent interactions |
| Resizable panels | react-resizable-panels behind Mirae adapter | proven panel sizing while preserving our API |
| Virtualization | TanStack Virtual | headless long-list performance |
| Icons | Hugeicons Stroke Rounded + custom icons | consistent contemporary stroke language |
| Forms | React Hook Form + generated schema validation | performant drafts and explicit validation |
| Component explorer | Storybook or equivalent isolated fixture host | review states and visual regressions |
| Tests | Vitest, Testing Library, Playwright, axe | unit, interaction, E2E, accessibility |

## Rules

- libraries are implementation tools, not product design systems;
- all third-party APIs are wrapped at the Mirae boundary where replacement risk matters;
- versions are pinned in the repository, not in this visual document;
- a library cannot own authoritative engine state;
- Floating UI may be used directly only for positioning not covered by the selected primitive.

## Rejected Default

Shipping shadcn blocks or raw library styling is explicitly rejected.

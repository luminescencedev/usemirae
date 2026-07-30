# 911 — UI Library Decisions

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## 1. Selected Stack

| Concern | Decision | Reason |
|---|---|---|
| Styling | Tailwind CSS v4 + semantic CSS variables | fast composition while keeping runtime theme tokens canonical |
| Accessible interaction foundation | React Aria Components | desktop-grade keyboard behavior, trees and collections, adaptive input, internationalization, and unstyled composition |
| Public component API | `@mirae/ui-kit` | Mirae owns visual language, product semantics, and replacement boundaries |
| Motion | Motion for React + CSS transitions | layout continuity, gestures, interruptibility, and reduced-motion support |
| Collection drag and drop | React Aria collection interactions | accessible keyboard and assistive-technology behavior for trees, lists, tables, and collections |
| Freeform drag and drop | dnd-kit behind Mirae patterns | spatial canvas interactions and custom drop geometry that are not collection semantics |
| Resizable panels | react-resizable-panels behind a Mirae adapter | proven panel sizing while preserving Mirae's API and persistence model |
| Virtualization | React Aria collection support first; TanStack Virtual when measured needs exceed it | avoid duplicate abstractions while retaining a specialized fallback |
| Icons | Hugeicons Stroke Rounded + custom production icons | consistent contemporary stroke language with Mirae-owned rendering rules |
| Forms | React Hook Form + generated schema validation | performant local drafts and explicit validation without owning project truth |
| Component fixture host | an approved isolated fixture host or application-owned fixture route | review component states and visual regressions without shipping raw examples |
| Tests | Vitest, Testing Library, Playwright, axe | unit, interaction, E2E, visual, and accessibility coverage |

---

## 2. Boundary

Feature code imports public controls from `@mirae/ui-kit`.

```text
apps/control-ui
    ↓
@mirae/ui-kit
    ↓
react-aria-components
```

Feature code must not import `react-aria-components` directly. The primitive
library remains a private implementation detail so Mirae can preserve its public
component API, styling, semantics, tests, and future replacement options.

React Aria Components is an interaction and accessibility foundation, not the
Mirae design system. Its default examples, DOM assumptions, and state naming do
not define product design.

---

## 3. Interaction Ownership

Use React Aria Components for:

- buttons, links, fields, dialogs, popovers, tooltips, menus, submenus, tabs, and
  toolbars;
- scene/source trees and hierarchical collections;
- grid lists, list boxes, tables, selection, focus, and keyboard navigation;
- collection-aware drag and drop;
- accessible number, slider, combo box, and date/time interactions where the
  product requires them.

Use dnd-kit only for freeform or spatial interactions such as:

- positioning sources on a canvas;
- transform handles;
- custom spatial drop zones;
- interactions whose meaning depends on coordinates rather than collection order.

Do not make the same interaction simultaneously depend on React Aria DnD and
dnd-kit.

---

## 4. Styling and State

- Tailwind CSS and semantic CSS variables implement Mirae styling.
- All public components expose Mirae-owned variants and state semantics.
- Library state attributes may be consumed internally, but are not exposed as the
  feature API.
- Operational states use Mirae semantic tokens and redundant communication by
  text, shape/icon, and color.
- Static feature styling must not rely on inline style objects.
- Runtime-computed geometry, transforms, and measurements may use inline styles.
- Library examples and raw browser controls are never shipped without Mirae
  composition and styling.

---

## 5. Rules

- Libraries are implementation tools, not product design systems.
- All replacement-sensitive third-party APIs are wrapped at the Mirae boundary.
- Versions are pinned in `DEPENDENCY_VERSIONS.md` and `pnpm-lock.yaml`.
- A library cannot own authoritative engine or project state.
- External design skills cannot select or replace the primitive library.
- A new primitive or component library requires a dedicated ticket and this
  document must change in the same pull request.
- Shipping shadcn blocks, raw Radix styling, raw React Aria examples, or generic
  dashboard templates is explicitly rejected.

---

## 6. Migration from Base UI

Base UI was selected during bootstrap but was not imported by production source
code. The migration therefore consists of dependency and documentation changes,
plus an enforced import boundary, without a runtime component rewrite.

Acceptance requires:

1. remove `@base-ui/react` from the catalog and UI-kit manifest;
2. add exact `react-aria-components` version through the approved dependency
   process;
3. update `DEPENDENCY_VERSIONS.md` and `pnpm-lock.yaml`;
4. forbid direct React Aria imports from `apps/control-ui`;
5. retain the existing Mirae `Button` and `StatusBadge` public APIs until their
   own component tickets intentionally evolve them;
6. validate typecheck, tests, lint, build, and frozen reinstall.

# 910 — Component Architecture

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## 1. Package

Canonical UI package: `packages/ui-kit` or `@mirae/ui` according to final monorepo naming.

## 2. Component Layers

```text
Base UI primitive
→ Mirae behavior adapter
→ Mirae styled primitive
→ production component
→ feature composition
```

## 3. API Rules

Every component declares:

- controlled/uncontrolled behavior;
- keyboard model;
- loading/disabled/error states;
- density;
- accessibility contract;
- motion contract;
- performance expectations;
- tokens consumed.

## 4. Styling

- Tailwind v4 utilities for composition;
- CSS variables for tokens;
- `cva` or equivalent for well-bounded variants;
- no arbitrary inline color values in feature code;
- no global extension CSS;
- no copied prebuilt blocks.

## 5. Story/Fixture Requirement

Each reusable component has visual fixtures for normal, hover, active, focus, disabled, loading, invalid, high-contrast, and reduced-motion states where applicable.

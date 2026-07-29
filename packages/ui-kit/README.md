# @mirae/ui-kit

Mirae-owned design system. Canonical documentation:

- `docs/09-ui-ux/905-design-system-foundations.md`
- `docs/09-ui-ux/906-design-tokens.md`
- `docs/09-ui-ux/910-component-architecture.md`
- `docs/09-ui-ux/911-ui-library-decisions.md`
- ADR-0061, ADR-0062, ADR-0063

## Layout

```text
src/
├── components/   composed Mirae controls
├── primitives/   Base UI wrappers
├── patterns/     multi-component interaction patterns
├── icons/        the single Hugeicons `Icon` wrapper
├── motion/       shared Motion configuration and reduced-motion handling
├── styles/       index.css and generated tokens.css
└── index.ts      public surface
tokens/           design-tokens.v1.json (canonical value source)
tests/
```

## Rules

- `tokens/design-tokens.v1.json` is the canonical visual value source.
- `src/styles/tokens.css` is a generated representation; do not hand-edit it as truth.
- Feature code must not import Base UI, Motion, Hugeicons, or resizable-panel
  primitives directly when a Mirae wrapper exists.
- No library primitive defines a Mirae public component API directly.
- This package exposes no application-domain state.

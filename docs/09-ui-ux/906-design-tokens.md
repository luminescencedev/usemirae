# 906 — Design Tokens

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## 1. Source

`tokens/design-tokens-v1.json` is the initial machine-readable source.

Tokens must generate:

- CSS custom properties;
- TypeScript constants;
- Figma variables or importable token data;
- documentation tables;
- visual test fixtures.

## 2. Token Families

- semantic colors;
- operational colors;
- typography;
- spacing;
- radii;
- borders;
- shadows/elevation;
- control sizes;
- panel sizes;
- motion duration/easing/spring;
- z-index layers.

## 3. Naming

Use semantic names such as `surface-raised`, `fg-muted`, and `status-live`, not visual names such as `gray-800` in component code.

Raw palette values may exist only beneath semantic aliases.

## 4. Runtime

Themes resolve through CSS variables at the shell root. Component code does not branch on theme names.

All token changes require visual regression review.

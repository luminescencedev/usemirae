# 907 — Color and Theme

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## 1. Dark Theme v1

| Token | Value | Role |
|---|---:|---|
| `canvas` | `#07080A` | app background and deepest canvas |
| `chrome` | `#0B0D11` | titlebar and structural chrome |
| `surface` | `#101319` | primary panels |
| `surface-raised` | `#151922` | fields and elevated controls |
| `surface-interactive` | `#1B2029` | active rows and hover surfaces |
| `border` | `#2A303A` | normal borders |
| `fg` | `#F4F6F8` | primary text |
| `fg-muted` | `#9AA1AC` | secondary text |
| `accent` | `#7C8FFF` | selection, focus, primary control |

## 2. Operational Colors

- live/error: `#FF4D6D`;
- success/connected: `#45D39B`;
- warning/recovery: `#F5B85C`;
- informational: `#61B6FF`.

Operational colors are never reused as arbitrary decoration.

## 3. Contrast and Redundancy

Status is communicated with text, icon/shape, and color.

Muted panels remain readable on common LCD and OLED displays. Pure black is reserved for content surfaces where desired, not every panel.

## 4. Glass

Translucency is allowed only for command palette, popovers, context menus, floating canvas controls, and selected overlays. Persistent structural panels remain opaque.

## 5. Future Themes

The system is dark-first, not dark-only forever. A future light theme must remap semantic tokens without changing component structure.

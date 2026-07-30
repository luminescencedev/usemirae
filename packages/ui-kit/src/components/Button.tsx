/**
 * The Mirae button.
 *
 * Canonical documentation:
 * - `docs/09-ui-ux/910-component-architecture.md`
 * - `docs/09-ui-ux/923-accessibility-and-reduced-motion.md`
 *
 * A native `button`, because a native button already has the keyboard
 * behaviour, the focus ring, and the role that a `div` would have to be given
 * back one attribute at a time — and would be given back wrong at least once.
 *
 * Disabled buttons stay in the accessibility tree with `aria-disabled` rather
 * than being removed from it: a control that vanishes when the engine drops
 * leaves a screen-reader user wondering what happened, while one that announces
 * itself as unavailable has told them.
 */

import type { ButtonHTMLAttributes, ReactNode } from "react";

/** How prominent a button is. */
export type ButtonTone = "primary" | "secondary";

/** Props for {@link Button}. */
export interface ButtonProps extends Omit<
  ButtonHTMLAttributes<HTMLButtonElement>,
  "className" | "style"
> {
  /** How prominent it should be. */
  readonly tone?: ButtonTone;
  /** The label. */
  readonly children: ReactNode;
}

/** A button carrying Mirae's tokens and focus behaviour. */
export function Button({
  tone = "secondary",
  disabled = false,
  children,
  ...rest
}: ButtonProps) {
  const primary = tone === "primary";

  return (
    <button
      type="button"
      // Still focusable, still announced, and still explains itself. `disabled`
      // alone would remove it from the tab order and say nothing.
      aria-disabled={disabled || undefined}
      disabled={disabled}
      style={{
        appearance: "none",
        border: `1px solid ${primary ? "transparent" : "var(--mirae-border)"}`,
        borderRadius: "var(--mirae-radius-control, 8px)",
        padding: "8px 14px",
        font: "500 13px/1.2 inherit",
        color: primary ? "var(--mirae-on-accent, #fff)" : "var(--mirae-fg)",
        background: primary ? "var(--mirae-accent)" : "transparent",
        cursor: disabled ? "not-allowed" : "pointer",
        opacity: disabled ? 0.5 : 1,
      }}
      {...rest}
    >
      {children}
    </button>
  );
}

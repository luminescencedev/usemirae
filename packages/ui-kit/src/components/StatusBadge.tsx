/**
 * A small labelled status indicator.
 *
 * Canonical documentation:
 * - `docs/09-ui-ux/910-component-architecture.md` (Mirae owns its component API)
 * - `docs/09-ui-ux/907-color-and-theme.md` (semantic colors are reserved)
 * - `docs/09-ui-ux/923-accessibility-and-reduced-motion.md`
 *
 * Colour never carries the meaning on its own: the label states the status in
 * words, so the badge is readable without colour vision and by a screen reader.
 */

import type { ReactNode } from "react";

/** What the badge is reporting. */
export type StatusTone = "neutral" | "positive" | "caution" | "critical";

/** Props for {@link StatusBadge}. */
export interface StatusBadgeProps {
  /** The tone, which selects a reserved semantic colour. */
  readonly tone: StatusTone;
  /** The visible label. Required: the badge never relies on colour alone. */
  readonly children: ReactNode;
}

/** Token names per tone, so no component invents a colour value. */
const TONE_TOKENS: Record<
  StatusTone,
  { readonly dot: string; readonly text: string }
> = {
  neutral: { dot: "var(--mirae-fg-muted)", text: "var(--mirae-fg-secondary)" },
  positive: { dot: "var(--mirae-success)", text: "var(--mirae-fg)" },
  caution: { dot: "var(--mirae-warning)", text: "var(--mirae-fg)" },
  critical: { dot: "var(--mirae-live)", text: "var(--mirae-fg)" },
};

/** A dot plus a text label describing a current status. */
export function StatusBadge({ tone, children }: StatusBadgeProps) {
  const tokens = TONE_TOKENS[tone];

  return (
    <span
      data-tone={tone}
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: "8px",
        color: tokens.text,
        font: "500 13px/1.4 inherit",
      }}
    >
      <span
        // Decorative: the label carries the meaning.
        aria-hidden="true"
        style={{
          width: "8px",
          height: "8px",
          borderRadius: "50%",
          background: tokens.dot,
          flex: "0 0 auto",
        }}
      />
      {children}
    </span>
  );
}

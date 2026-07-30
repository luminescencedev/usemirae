/**
 * `@mirae/ui-kit` public surface.
 *
 * Responsibilities (see `docs/09-ui-ux/910-component-architecture.md`): wrap Base
 * UI primitives, expose Mirae-owned component APIs, apply Obsidian Precision
 * tokens, centralize keyboard and focus behavior, centralize Motion
 * configuration, and render Hugeicons through one `Icon` wrapper.
 *
 * This package exposes no application-domain state.
 *
 * Only the components a feature needs today exist. The rest arrive with the UI
 * implementation backlog in `docs/09-ui-ux/928-ui-implementation-backlog.md`.
 */

export {
  StatusBadge,
  type StatusBadgeProps,
  type StatusTone,
} from "./components/StatusBadge";

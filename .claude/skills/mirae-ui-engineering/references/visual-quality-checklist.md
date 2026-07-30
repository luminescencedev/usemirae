# Mirae Visual Quality Checklist

Use this checklist for every user-visible ticket.

## Product placement

- The feature occupies its intended long-term workspace region.
- A temporary milestone surface has an explicit removal or relocation ticket.
- Engine diagnostics do not dominate the primary workflow.
- The workspace remains recognizable when a subsystem is unavailable.

## Design system

- Only semantic Mirae tokens define colors and states.
- Typography follows the canonical Mirae stack and hierarchy.
- Spacing, radius, border, and elevation are consistent with existing components.
- No raw third-party appearance or browser-default control is visible.
- No generic SaaS card-grid composition replaces the desktop operator layout.

## States

- Loading is bounded and informative.
- Empty state explains what exists and what action is available.
- Disconnected/degraded state is honest and actionable.
- Error state is local to the affected surface when possible.
- Disabled controls explain their unavailability where needed.
- Success does not erase information the operator still needs.

## Interaction

- Pointer interactions have keyboard equivalents.
- Focus is visible and moves predictably.
- Menus, dialogs, trees, lists, and tables follow their expected keyboard model.
- Selection, active, armed, live, locked, and disabled states are visually distinct.
- Destructive production actions require deliberate confirmation.

## Density and responsiveness

- The result is credible at 1440 x 900.
- A narrower desktop window degrades without becoming a mobile page.
- The canvas remains visible and prioritized.
- Text does not clip unexpectedly.
- Long content has an explicit truncation, wrapping, scrolling, or virtualization rule.

# ADR-0061 — Mirae-Owned Design System over Base UI

**Status:** Proposed  
**Date:** 2026-07-29

## Decision

Use Base UI for accessible headless behavior and build every visible component in the Mirae design system. Do not ship Base UI examples or third-party visual defaults directly.

## Consequences

Mirae gains robust primitives while retaining complete visual ownership. Wrappers and component testing add implementation cost.

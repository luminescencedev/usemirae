# ADR-0066 — Engine State and UI Drafts Remain Separate

**Status:** Proposed  
**Date:** 2026-07-29

## Decision

Authoritative project and runtime state comes from engine projections. Form drafts, selection, viewport, and optimistic interaction state remain local and generation-aware.

## Consequences

Reconnect and rejection behavior remain correct. UI architecture needs explicit stores rather than one global state object.

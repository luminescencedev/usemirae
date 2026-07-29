# ADR-0065 — Persistent Dockable Desktop Workspace

**Status:** Proposed  
**Date:** 2026-07-29

## Decision

Mirae uses a desktop-first workspace with resizable role-based docks. Layout persists separately from project semantic state and safely falls back to defaults.

## Consequences

Operators gain a stable professional workspace. Layout schema, constraints, and multi-monitor recovery require dedicated implementation.

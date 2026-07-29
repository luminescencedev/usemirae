# ADR-0062 — CSS Variables as Runtime Design Tokens

**Status:** Proposed  
**Date:** 2026-07-29

## Decision

Semantic CSS custom properties are the runtime theme contract. JSON is the source and code generation produces CSS and TypeScript outputs.

## Consequences

Themes and platform adaptations can change without rewriting components. Token governance and generated-drift checks are required.

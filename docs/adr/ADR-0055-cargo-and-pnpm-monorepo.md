# ADR-0055 — Cargo and pnpm Monorepo

**Status:** Proposed  
**Date:** 2026-07-29

## Context

Mirae contains Rust processes/libraries, a React/TypeScript UI, generated schemas, native packaging, tests, and documentation that must evolve together.

## Decision

Mirae will use one monorepo with a Cargo workspace and pnpm workspace.

Repository automation will be coordinated through `cargo xtask`.

## Consequences

### Positive

- atomic cross-language changes;
- one compatibility boundary;
- synchronized contracts;
- shared CI and release;
- simpler AI-agent context.

### Negative

- larger repository;
- cross-language CI cost;
- workspace discipline required.

## Alternatives Considered

Separate engine and UI repositories were rejected because contract drift and release coordination would be harder.

## Related Specifications

- `08-development/801-monorepo-architecture.md`
- `08-development/806-build-system-and-toolchain.md`

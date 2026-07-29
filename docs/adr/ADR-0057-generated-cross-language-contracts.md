# ADR-0057 — Generated Cross-Language Contracts

**Status:** Proposed  
**Date:** 2026-07-29

## Context

IPC, project, SDK, and diagnostics contracts are consumed by Rust and TypeScript and must remain compatible.

## Decision

Canonical schemas will generate language bindings, validators, fixtures, and documentation.

Generated output drift will fail CI.

## Consequences

### Positive

- one source of truth;
- fewer manual mismatches;
- compatibility fixtures;
- bounded validation.

### Negative

- code-generation tooling;
- generated diffs;
- schema discipline.

## Alternatives Considered

Handwritten duplicate Rust and TypeScript types were rejected because drift is inevitable.

## Related Specifications

- `08-development/805-generated-contracts-and-schemas.md`

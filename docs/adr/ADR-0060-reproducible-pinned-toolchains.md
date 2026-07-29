# ADR-0060 — Reproducible Pinned Toolchains

**Status:** Proposed  
**Date:** 2026-07-29

## Context

Rust, Node, pnpm, code generators, native dependencies, and packaging tools can change behavior across machines and CI.

## Decision

Mirae will pin toolchains and verified native dependencies, commit lockfiles, and expose one repository bootstrap/check command surface.

## Consequences

### Positive

- reproducible local and CI behavior;
- easier onboarding;
- traceable releases;
- fewer environment-only failures.

### Negative

- regular pin maintenance;
- upgrade PRs;
- platform package management work.

## Alternatives Considered

Unpinned “latest” toolchains were rejected because they make builds non-reproducible.

## Related Specifications

- `08-development/806-build-system-and-toolchain.md`
- `08-development/808-local-development-environment.md`

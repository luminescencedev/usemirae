# ADR-0058 — Short-Lived Branches and Squash Merge

**Status:** Proposed  
**Date:** 2026-07-29

## Context

Mirae will be developed through many architecture-linked tickets, including AI-assisted work.

Long-lived branches increase drift and make review difficult.

## Decision

Use short-lived ticket branches, focused pull requests, protected main, squash merge, and branch deletion.

## Consequences

### Positive

- linear main;
- small review surface;
- easy rollback by ticket;
- clear issue-to-commit mapping.

### Negative

- branch discipline;
- stacked work requires care;
- detailed intermediate commit history is compressed.

## Alternatives Considered

Long-lived integration branches were rejected because they delay feedback and accumulate conflicts.

## Related Specifications

- `08-development/811-git-and-pull-request-workflow.md`

# ADR-0049 — Versioned Schema-First SDK Contracts

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

A public SDK must support multiple extension languages and versions without exposing internal Rust implementation details.

---

## Decision

Public SDK contracts will be defined through versioned schemas used for IPC, generated bindings, fixtures, validation, and documentation.

---

## Consequences

### Positive

- language-neutral contracts;
- compatibility tests;
- safer evolution;
- generated bindings;
- bounded decoding.

### Negative

- schema tooling;
- code generation;
- mapping between internal and public types;
- release discipline.

---

## Alternatives Considered

### Expose internal Rust types

Rejected because they are not stable or language-neutral.

### Generic JSON method calls

Rejected because they are weakly typed and difficult to evolve safely.

---

## Related Specifications

- `07-sdk/704-sdk-api-surface.md`
- `07-sdk/707-extension-ipc-protocol.md`
- `07-sdk/712-sdk-versioning-and-compatibility.md`

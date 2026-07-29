# ADR-0006 — Typed Versioned IPC

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Mirae uses multiple processes and a TypeScript control UI. Untyped IPC would make compatibility, validation, security, and refactoring unreliable.

---

## Decision

All production IPC will use a typed, schema-driven, versioned protocol.

The protocol will distinguish message families, authenticate process roles, bound message sizes, and support generated Rust and TypeScript types where practical.

Raw media payloads will not use the generic control protocol.

---

## Consequences

### Positive

- compile-time and generated-type safety;
- explicit compatibility;
- safer decoding;
- testable fixtures;
- clearer capability boundaries;
- reliable reconnect semantics.

### Negative

- schema tooling required;
- version migration overhead;
- more ceremony than arbitrary JSON messages;
- generated-code management.

---

## Alternatives Considered

### Arbitrary JSON RPC

Rejected because it encourages stringly typed methods, weak bounds, and silent schema drift.

### Direct shared library calls

Rejected across process boundaries and incompatible with UI/extension isolation.

---

## Related Specifications

- `01-runtime/101-process-model.md`
- `01-runtime/108-ipc-protocol.md`
- `01-runtime/109-ui-engine-synchronization.md`

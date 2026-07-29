# 704 — SDK API Surface

**Status:** Proposed  
**Audience:** SDK, runtime, product API contributors  
**Canonical:** Yes  
**Required context:** `700-sdk-overview.md`, `01-runtime/104-command-system.md`, `01-runtime/105-event-system.md`  
**Related ADRs:** ADR-0049, ADR-0050

---

## 1. Purpose

This document defines the public SDK API shape and rules for exposing stable extension functionality without leaking engine internals.

---

## 2. API Families

- lifecycle;
- capabilities;
- project queries;
- extension namespace mutations;
- command registration;
- event subscription;
- operations/progress;
- source/output/effect registration;
- media data plane;
- UI contributions;
- storage;
- settings;
- credentials broker;
- network broker;
- file broker;
- diagnostics;
- localization.

---

## 3. Schema-First Contracts

Every public type is defined in canonical schemas that generate or validate:

- Rust host bindings;
- TypeScript bindings;
- future language bindings;
- protocol fixtures;
- compatibility tests;
- documentation.

Internal engine structs are mapped to public DTOs.

---

## 4. API Request Model

Requests contain:

- extension identity;
- runtime instance ID;
- capability token;
- SDK version;
- correlation ID;
- request payload;
- optional expected generation;
- cancellation token where supported.

The host injects identity; extensions cannot spoof it.

---

## 5. Stable IDs

Public IDs include:

- project entity references scoped by grant;
- extension instance ID;
- operation ID;
- source/output/effect provider ID;
- UI contribution ID;
- storage namespace ID.

Internal object addresses or indices are never exposed.

---

## 6. Queries

Queries return filtered projections.

An extension can request only:

- fields allowed by capability;
- entities within project scope;
- bounded result counts;
- supported projection versions.

Large result sets use pagination or streaming with quotas.

---

## 7. Mutations

Extensions may mutate:

- their own project namespace;
- entities through approved host commands;
- resources they own;
- registered provider instances.

They do not receive generic arbitrary project-write access.

---

## 8. Operations

Long-running API calls create operations.

Operation includes:

- state;
- progress;
- cancellation;
- result;
- structured error;
- owner;
- deadline;
- resource cost.

Operation queues are bounded.

---

## 9. API Evolution

Fields may be:

- required;
- optional;
- deprecated;
- experimental;
- feature-gated.

Unknown optional fields are ignored safely.

Unknown required features reject.

---

## 10. Documentation

Every API endpoint documents:

- purpose;
- capability;
- lifecycle state;
- request/response;
- bounds;
- errors;
- threading;
- idempotency;
- cancellation;
- version introduced;
- deprecation.

---

## 11. Invariants

1. Public DTOs are separate from internal structs.
2. Identity is host-injected.
3. Queries are filtered and bounded.
4. Mutations are command-mediated.
5. Operations are cancellable where possible.
6. IDs are opaque and stable within scope.
7. API contracts are generated/tested from schemas.
8. Experimental APIs are labeled.
9. Unknown required features reject.
10. Every endpoint declares permission and bounds.

---

## 12. Required Tests

- generated binding round trip;
- identity spoof rejection;
- projection filtering;
- pagination bound;
- mutation permission;
- operation cancellation;
- unknown optional field;
- unknown required feature;
- deprecated endpoint warning;
- experimental API disabled;
- stable error mapping;
- internal-field leak scan.

---

## 13. AI Implementation Notes

Do not expose internal structs through serialization derives.

Do not add an API endpoint without capability, bounds, errors, and version metadata.

Do not use generic `execute(method, json)` as the canonical public SDK.

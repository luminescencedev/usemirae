# 807 — Code Conventions

**Status:** Proposed  
**Audience:** All contributors and coding agents  
**Canonical:** Yes  
**Required context:** `06-quality` section

---

## 1. Purpose

This document defines repository-wide conventions that protect readability and architecture.

---

## 2. Rust

- format with `rustfmt`;
- lint with strict `clippy` policy;
- avoid `unwrap`/`expect` in recoverable paths;
- use typed IDs and bounded wrappers;
- use structured errors;
- document unsafe blocks;
- prefer immutable data;
- avoid broad public fields;
- no hidden global mutable state;
- no blocking in async or real-time paths;
- no `println!` in production code.

---

## 3. TypeScript/React

- strict TypeScript;
- no `any` without local documented boundary;
- generated DTOs only for contracts;
- functional components;
- accessible semantic controls;
- no project truth in local state;
- no direct native bridge outside client package;
- no silent promise rejection;
- no unbounded event listener registration;
- avoid global stores unless ownership is explicit.

---

## 4. Naming

Use domain language from terminology docs.

Names should expose:

- owner;
- lifecycle;
- unit;
- generation;
- direction;
- scope.

Avoid vague names such as:

- `manager`;
- `handler`;
- `data`;
- `utils`;
- `misc`;
- `thing`;
- `processData`.

Use specific names.

---

## 5. Comments

Comments explain:

- invariants;
- non-obvious trade-offs;
- safety;
- platform quirks;
- protocol semantics;
- why an optimization is correct.

Comments should not narrate obvious code.

---

## 6. TODOs

Every TODO includes:

- owner or issue ID;
- reason;
- removal condition.

No anonymous permanent TODOs.

---

## 7. Configuration

Configuration is:

- typed;
- validated;
- bounded;
- documented;
- separated from secrets;
- versioned when persisted.

---

## 8. Invariants

1. Formatting is automated.
2. Recoverable failures do not panic.
3. Unsafe code is documented.
4. Public APIs are narrow.
5. Contracts use generated types.
6. Domain terminology is consistent.
7. TODOs are traceable.
8. Secrets stay out of config.
9. Real-time paths avoid allocation/blocking.
10. Code does not bypass documented ownership.

# Architecture Decision Records

Architecture Decision Records capture durable technical decisions and the reasoning behind them.

---

## Naming

```text
ADR-0001-short-decision-name.md
```

Numbers are never reused.

---

## Statuses

- Proposed
- Accepted
- Rejected
- Deprecated
- Superseded

---

## Required Sections

Every ADR includes:

1. Title
2. Status
3. Date
4. Context
5. Decision
6. Consequences
7. Alternatives considered
8. Migration or implementation notes
9. Related specifications

---

## Rules

- Accepted ADRs are historical records.
- A changed decision requires a new ADR.
- A superseding ADR links to the old ADR.
- Specifications must reflect Accepted ADRs.
- Code alone does not supersede an ADR.

---

## Current ADRs

- ADR-0001: Native Rust Core
- ADR-0002: GPU-First Rendering
- ADR-0003: Local-First and Offline-Capable Operation
- ADR-0004: Scene Graph and Render Graph Separation
- ADR-0005: Process Isolation

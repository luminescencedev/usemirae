# Mirae Documentation

> Canonical software design specification for Mirae.

Mirae documentation is not a post-implementation wiki. It is the primary design input for the application, its engine, its user interface, and its development workflow.

The intended relationship is:

```text
Product intent
    ↓
Canonical specification
    ↓
Architecture decisions
    ↓
Implementation plan
    ↓
Code
    ↓
Tests and validation
```

Code is expected to conform to the specification. When implementation and documentation disagree, the disagreement must be resolved explicitly. Neither side silently overrides the other.

---

## 1. Purpose

This documentation exists to let a human team or an AI coding agent build Mirae with minimal architectural guesswork.

It defines:

- what Mirae is;
- what Mirae is not;
- system boundaries;
- architectural invariants;
- module responsibilities;
- public and internal contracts;
- ownership and lifetime rules;
- concurrency rules;
- performance budgets;
- failure behavior;
- required tests;
- accepted trade-offs;
- decisions that require an ADR.

Every implementation-facing document must be precise enough to answer:

1. What must be built?
2. Why does it exist?
3. Where does it belong?
4. What may it depend on?
5. What must it never do?
6. Which invariants must remain true?
7. How is it validated?
8. Which parts are intentionally unresolved?

---

## 2. Canonical Status

A document may have one of the following statuses:

| Status | Meaning |
|---|---|
| `Draft` | Direction is being explored. It must not be treated as a stable implementation contract. |
| `Proposed` | The specification is coherent and ready for review, but may still change materially. |
| `Accepted` | Canonical and implementation-authoritative. |
| `Implemented` | Accepted and represented in the current codebase. |
| `Deprecated` | Kept for migration or historical context. New implementation must not depend on it. |
| `Superseded` | Replaced by another document or ADR. |

Unless explicitly stated otherwise, files in this initial pack are `Proposed`.

---

## 3. Documentation Architecture

The final documentation tree is:

```text
docs/
├── README.md
├── SUMMARY.md
│
├── 00-foundations/
│   ├── 000-documentation-contract.md
│   ├── 001-project-overview.md
│   ├── 002-product-and-system-boundaries.md
│   ├── 003-design-principles.md
│   ├── 004-system-overview.md
│   ├── 005-domain-model.md
│   ├── 006-terminology.md
│   ├── 007-ai-implementation-contract.md
│   └── 008-architecture-change-process.md
│
├── 01-runtime/
│   ├── 100-runtime-overview.md
│   ├── 101-process-model.md
│   ├── 102-engine-lifecycle.md
│   ├── 103-frame-scheduler.md
│   ├── 104-command-system.md
│   ├── 105-event-system.md
│   ├── 106-state-store.md
│   ├── 107-transactions.md
│   ├── 108-ipc-protocol.md
│   └── 109-ui-engine-synchronization.md
│
├── 02-rendering/
│   ├── 200-rendering-overview.md
│   ├── 201-scene-graph.md
│   ├── 202-render-graph.md
│   ├── 203-compositor.md
│   ├── 204-renderer.md
│   ├── 205-gpu-resource-model.md
│   ├── 206-shader-system.md
│   ├── 207-color-management.md
│   ├── 208-text-and-graphics.md
│   ├── 209-effects-and-transitions.md
│   └── 210-preview-and-program-surfaces.md
│
├── 03-media/
│   ├── 300-media-overview.md
│   ├── 301-source-system.md
│   ├── 302-capture-system.md
│   ├── 303-media-pipeline.md
│   ├── 304-master-clock-and-timebase.md
│   ├── 305-audio-architecture.md
│   ├── 306-audio-routing-and-monitoring.md
│   ├── 307-synchronization.md
│   ├── 308-encoder-system.md
│   ├── 309-output-architecture.md
│   ├── 310-streaming-and-network-reliability.md
│   ├── 311-recording.md
│   └── 312-replay-buffer.md
│
├── 04-project/
│   ├── 400-project-overview.md
│   ├── 401-project-format.md
│   ├── 402-project-library.md
│   ├── 403-persistence.md
│   ├── 404-autosave-and-recovery.md
│   ├── 405-command-history-and-undo-redo.md
│   ├── 406-asset-registry.md
│   ├── 407-local-file-system.md
│   └── 408-schema-versioning-and-migrations.md
│
├── 05-platform/
│   ├── 500-platform-overview.md
│   ├── 501-desktop-shell.md
│   ├── 502-windows-platform.md
│   ├── 503-macos-platform.md
│   ├── 504-linux-platform.md
│   ├── 505-platform-capture.md
│   ├── 506-hardware-acceleration.md
│   ├── 507-permissions-and-entitlements.md
│   ├── 508-secure-credential-storage.md
│   └── 509-updates-packaging-and-signing.md
│
├── 06-quality/
│   ├── 600-quality-overview.md
│   ├── 601-performance-budgets.md
│   ├── 602-memory-model.md
│   ├── 603-resource-lifetimes.md
│   ├── 604-concurrency-model.md
│   ├── 605-error-model.md
│   ├── 606-logging-and-tracing.md
│   ├── 607-observability-and-diagnostics.md
│   ├── 608-crash-reporting.md
│   ├── 609-testing-strategy.md
│   ├── 610-benchmarking-and-regressions.md
│   ├── 611-fault-injection.md
│   ├── 612-security-model.md
│   ├── 613-accessibility.md
│   ├── 614-localization.md
│   └── 615-compatibility-policy.md
│
├── 07-sdk/
│   ├── 700-sdk-overview.md
│   ├── 701-extension-model.md
│   ├── 702-extension-host.md
│   ├── 703-plugin-lifecycle.md
│   ├── 704-manifest-and-capabilities.md
│   ├── 705-permission-model.md
│   ├── 706-sandboxing.md
│   ├── 707-api-surface.md
│   ├── 708-event-api.md
│   ├── 709-ui-extension-api.md
│   ├── 710-storage-api.md
│   ├── 711-api-versioning.md
│   └── 712-extension-testing-and-publishing.md
│
└── adr/
    ├── README.md
    └── ADR-XXXX-*.md
```

The numeric prefix is stable and indicates the documentation area. It is not tied to implementation order.

---

## 4. Reading Order

A new contributor or coding agent should read:

1. `00-foundations/000-documentation-contract.md`
2. `00-foundations/001-project-overview.md`
3. `00-foundations/002-product-and-system-boundaries.md`
4. `00-foundations/003-design-principles.md`
5. `00-foundations/004-system-overview.md`
6. `00-foundations/005-domain-model.md`
7. `00-foundations/006-terminology.md`
8. `00-foundations/007-ai-implementation-contract.md`

After foundations, read only the sections related to the active task and all documents listed under their **Required context** section.

---

## 5. Specification Style

Documents are implementation-oriented. They should prefer:

- explicit requirements over aspirations;
- named ownership over implicit responsibility;
- invariants over prose;
- bounded behavior over open-ended flexibility;
- diagrams over repeated descriptions;
- pseudocode over hand-waving;
- measurable budgets over words such as “fast”;
- precise non-goals over accidental scope;
- testable acceptance criteria over subjective completion.

Normative terms follow RFC-style meaning:

- **MUST**: mandatory requirement;
- **MUST NOT**: prohibited behavior;
- **SHOULD**: recommended unless a documented reason prevents it;
- **SHOULD NOT**: generally prohibited unless explicitly justified;
- **MAY**: optional behavior;
- **INVARIANT**: condition that must remain true throughout the specified lifecycle.

---

## 6. Source-of-Truth Hierarchy

When two sources conflict, use this priority:

1. Accepted ADR
2. Accepted canonical specification
3. Public API contract and schema
4. Approved implementation plan
5. Tests
6. Current implementation
7. Comments and issue discussions

A newer Accepted ADR may supersede an older specification. The specification must then be updated in the same change set or in a tracked follow-up.

---

## 7. Change Policy

A documentation change requires an ADR when it:

- changes a cross-subsystem invariant;
- introduces a new process;
- changes process isolation;
- changes ownership across crates or services;
- changes persistence compatibility;
- changes the public SDK or project format;
- adds a long-lived third-party dependency to a critical path;
- alters a performance, reliability, or security guarantee.

Small clarifications, typo fixes, and implementation-neutral refinements do not require an ADR.

---

## 8. Definition of Done

A subsystem specification is complete enough for implementation when it includes:

- purpose and scope;
- responsibilities and non-responsibilities;
- required components;
- state and data flows;
- ownership and lifetime rules;
- concurrency model;
- error behavior;
- performance constraints;
- platform differences;
- expected interfaces;
- required tests;
- unresolved questions;
- related ADRs.

A subsystem is not complete because a file exists. It is complete when an implementer can proceed without inventing architecture.

# 801 — Monorepo Architecture

**Status:** Proposed  
**Audience:** Architecture, build, all contributors  
**Canonical:** Yes  
**Required context:** `800-development-overview.md`  
**Related ADRs:** ADR-0055, ADR-0056

---

## 1. Purpose

The monorepo keeps engine, shell, UI, schemas, tests, tooling, and documentation versioned together.

---

## 2. Target Root Layout

```text
mirae/
├── apps/
│   ├── desktop-shell/
│   ├── control-ui/
│   ├── engine/
│   └── extension-host/
├── crates/
│   ├── foundation/
│   ├── domain/
│   ├── runtime/
│   ├── project/
│   ├── rendering/
│   ├── media/
│   ├── platform/
│   ├── sdk/
│   ├── diagnostics/
│   └── test-support/
├── packages/
│   ├── contracts/
│   ├── ui-kit/
│   ├── client/
│   ├── localization/
│   └── config/
├── schemas/
│   ├── ipc/
│   ├── project/
│   ├── sdk/
│   ├── diagnostics/
│   └── generated/
├── tools/
│   ├── codegen/
│   ├── xtask/
│   ├── fixtures/
│   └── release/
├── tests/
│   ├── integration/
│   ├── e2e/
│   ├── compatibility/
│   ├── performance/
│   └── fault/
├── docs/
├── fixtures/
├── scripts/
├── Cargo.toml
├── pnpm-workspace.yaml
├── package.json
├── rust-toolchain.toml
├── CLAUDE.md
└── README.md
```

---

## 3. Root Responsibilities

The root owns:

- workspace definitions;
- common commands;
- toolchain pins;
- generated-contract orchestration;
- repository policies;
- top-level CI configuration;
- shared lint and formatting configuration;
- documentation entry points.

The root should not contain implementation source files.

---

## 4. Applications

`apps/` contains deployable processes or user-facing applications.

Each app:

- has one process role;
- assembles crates/packages;
- contains minimal domain logic;
- owns process entry point and deployment configuration;
- exposes health/version metadata.

---

## 5. Libraries

`crates/` and `packages/` contain reusable units.

A library:

- has one primary responsibility;
- exposes a narrow public API;
- avoids depending on deployable app code;
- includes tests;
- documents dependency direction.

---

## 6. Schemas

`schemas/` contains canonical machine contracts.

Generated outputs are written to declared generated directories and verified in CI.

Schemas are not duplicated in handwritten Rust and TypeScript definitions.

---

## 7. Tests

Cross-cutting tests stay under `tests/`.

Subsystem-local unit/component tests stay near implementation.

Large binary fixtures are minimized, documented, and hash-pinned.

---

## 8. Tools

`tools/xtask` or equivalent owns repository automation that is too complex for shell scripts.

Preferred command surface:

```text
cargo xtask bootstrap
cargo xtask generate
cargo xtask check
cargo xtask test
cargo xtask package
cargo xtask docs
```

---

## 9. Invariants

1. Deployable apps do not become shared libraries.
2. Shared libraries do not depend on apps.
3. Schemas have one canonical location.
4. Generated outputs have declared owners.
5. Cross-cutting tests are separated from implementation.
6. Root scripts remain thin.
7. Platform-specific implementation stays under platform adapters.
8. Documentation lives with the repository.
9. Build artifacts are ignored.
10. Secret or machine-local state is never committed.

# 803 — Frontend Workspace and Packages

**Status:** Proposed  
**Audience:** UI, shell, TypeScript contributors  
**Canonical:** Yes  
**Required context:** `05-platform/501-desktop-shell.md`, `01-runtime/109-ui-engine-synchronization.md`

---

## 1. Purpose

This document defines the React/TypeScript control UI workspace.

---

## 2. Applications

### `apps/control-ui`

The operator interface.

Owns:

- routes;
- screen composition;
- feature modules;
- engine-client integration;
- local optimistic UI;
- accessibility and localization integration.

It does not own engine truth.

### `apps/extension-ui-preview`

Development-only host for extension UI schemas.

Optional during early phases.

---

## 3. Shared Packages

### `packages/contracts`

Generated TypeScript bindings.

### `packages/client`

Typed engine client, reconnect logic, command/query/event abstractions.

### `packages/ui-kit`

Accessible design-system primitives.

### `packages/localization`

Message loading, formatting, locale helpers.

### `packages/config`

Shared TypeScript, ESLint, build, and test configuration.

### `packages/test-utils`

UI test harnesses, fake engine client, accessibility helpers.

---

## 4. Feature Structure

Inside the UI application:

```text
src/
├── app/
├── features/
│   ├── project/
│   ├── scenes/
│   ├── sources/
│   ├── audio/
│   ├── outputs/
│   ├── settings/
│   └── diagnostics/
├── components/
├── stores/
├── routes/
├── hooks/
└── styles/
```

Feature modules should own their UI behavior, not duplicate global engine state.

---

## 5. State Rules

- server/engine state comes through typed client projections;
- optimistic state is local and temporary;
- high-frequency metrics use dedicated stores;
- route state is separate from project state;
- forms keep draft state until command submission;
- reconnect discards stale optimistic state.

---

## 6. UI Boundaries

The UI must not:

- read project files directly;
- store credentials;
- invoke native APIs directly;
- create media pipelines;
- duplicate project validation rules;
- infer capability from OS name;
- treat local React state as authoritative project state.

---

## 7. Design System

`ui-kit` owns:

- tokens;
- typography;
- focus;
- controls;
- dialogs;
- menus;
- list/table primitives;
- forms;
- toasts/status;
- high-contrast behavior;
- reduced motion.

Feature code should compose primitives rather than fork them.

---

## 8. Testing

Required:

- unit tests for pure feature logic;
- component tests;
- keyboard tests;
- accessibility checks;
- fake-engine integration tests;
- reconnect tests;
- screenshot tests only where stable and valuable.

---

## 9. Invariants

1. Generated contracts are the only DTO source.
2. UI does not become engine authority.
3. Feature modules avoid global-store dumping grounds.
4. High-frequency metrics are isolated.
5. All controls are keyboard-accessible.
6. Credentials never enter ordinary frontend storage.
7. Extension UI is isolated.
8. Design tokens remain centralized.
9. Optimistic state is generation-aware.
10. UI package dependencies remain acyclic.

# 802 — Rust Workspace and Crates

**Status:** Proposed  
**Audience:** Rust contributors and coding agents  
**Canonical:** Yes  
**Required context:** `801-monorepo-architecture.md`, `804-dependency-rules.md`

---

## 1. Purpose

This document defines the initial Rust workspace decomposition.

---

## 2. Foundation Crates

### `mirae-types`

Stable IDs, generations, rational time, bounded primitive wrappers, common enums.

Must not depend on higher-level crates.

### `mirae-errors`

Structured error taxonomy and safe context.

Depends only on foundation types and minimal utility crates.

### `mirae-contracts`

Generated Rust bindings for IPC, project, SDK, and diagnostics schemas.

Generated code is not edited manually.

### `mirae-observability`

Tracing, metrics, correlation, redaction, diagnostic event helpers.

---

## 3. Domain and Application Crates

### `mirae-domain`

Project-domain entities and validation.

No OS, GPU, FFmpeg, UI, or network dependencies.

### `mirae-commands`

Command DTOs, validation interfaces, transaction intents.

### `mirae-state`

Authoritative state store, generations, snapshots, patches.

### `mirae-project`

Project format mapping, persistence orchestration, migrations, assets, recovery.

### `mirae-runtime`

Engine lifecycle, service orchestration, process coordination.

---

## 4. Rendering Crates

### `mirae-scene`

Semantic scene graph.

### `mirae-frame-compiler`

Scene/source snapshots to renderer-independent frame plans.

### `mirae-render-graph`

Logical passes, resources, dependencies, validation.

### `mirae-renderer`

Backend-independent renderer interfaces and resource model.

### `mirae-renderer-wgpu`

`wgpu` implementation.

### `mirae-shaders`

Built-in shader schemas, sources, generation, tests.

---

## 5. Media Crates

### `mirae-media-types`

Media units, timebases, formats, discontinuities, leases.

### `mirae-source-runtime`

Source lifecycle and queues.

### `mirae-audio`

Canonical audio graph and routing.

### `mirae-encoder`

Encoder interfaces and registry.

### `mirae-output`

Output pipelines.

### `mirae-media-ffmpeg`

Contained FFmpeg adapters.

---

## 6. Platform Crates

### `mirae-platform`

Cross-platform interfaces and capability model.

### `mirae-platform-windows`

Windows adapters.

### `mirae-platform-macos`

macOS adapters.

### `mirae-platform-linux`

Linux adapters.

Target-specific crates may compile conditionally, but shared domain code must not branch on OS.

---

## 7. SDK Crates

### `mirae-sdk-protocol`

Generated public extension contracts.

### `mirae-extension-manager`

Package, permission, lifecycle, host supervision.

### `mirae-extension-host-runtime`

Host-side extension runtime and quotas.

### `mirae-extension-brokers`

File, network, storage, credential mediation.

---

## 8. Application Crates

### `mirae-engine`

Minimal engine process entry point.

### `mirae-shell`

Native shell entry point and UI host.

### `mirae-extension-host`

Extension-host process entry point.

---

## 9. Crate Design Rules

Each crate must define:

- purpose;
- public API;
- owner;
- allowed dependencies;
- unsafe policy;
- tests;
- feature flags;
- platform support.

Avoid a generic `utils` crate.

Shared code belongs in a named responsibility-specific crate.

---

## 10. Feature Flags

Feature flags:

- represent optional integrations or build modes;
- do not silently change core semantics;
- are additive where possible;
- are tested in supported combinations;
- are documented.

Platform selection should use target configuration, not feature flags alone.

---

## 11. Invariants

1. Domain crates are platform-independent.
2. `wgpu` stays in renderer implementation crates.
3. FFmpeg stays in media adapter crates.
4. App crates contain minimal logic.
5. Generated contracts are centralized.
6. No generic dumping-ground crate.
7. Unsafe code is localized.
8. Feature combinations are tested.
9. Public APIs are narrow.
10. Cyclic dependencies are prohibited.

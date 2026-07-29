# 813 — Implementation Roadmap

**Status:** Proposed  
**Audience:** Project owner, architecture, coding agents  
**Canonical:** Yes  
**Required context:** All architecture sections  
**Related ADRs:** ADR-0059

---

## 1. Purpose

This roadmap orders implementation so that Mirae becomes runnable early while preserving the final architecture.

---

## 2. Phase 0 — Repository Foundation

Deliver:

- monorepo;
- pinned toolchains;
- root commands;
- CI;
- code generation skeleton;
- documentation validation;
- empty shell/engine/UI processes;
- authenticated local IPC handshake;
- structured logging.

Exit condition: all processes launch and tests run from one command.

---

## 3. Phase 1 — Project and State Kernel

Deliver:

- typed IDs/generations;
- state store;
- commands/events/transactions;
- minimal project schema;
- atomic save/open;
- autosave skeleton;
- create/open/save empty project;
- UI project status.

Exit condition: empty project can be created, saved, reopened, and recovered.

---

## 4. Phase 2 — Minimal Rendering Slice

Deliver:

- scene and scene item model;
- generated color source;
- frame compiler;
- minimal render graph;
- `wgpu` backend;
- preview surface;
- device-loss test path.

Exit condition: one generated source renders in preview.

---

## 5. Phase 3 — First Real Source

Deliver one platform first:

- display or camera source;
- capture adapter;
- frame queue;
- timestamp normalization;
- source health;
- scene integration;
- UI source creation.

Exit condition: live source renders and recovers from disconnect.

---

## 6. Phase 4 — Audio Kernel

Deliver:

- canonical audio format;
- fake audio source;
- audio graph;
- source gain/mute;
- meters;
- monitor output;
- real-time tests.

Exit condition: deterministic audio graph and one platform device path.

---

## 7. Phase 5 — Recording

Deliver:

- renderer output surface;
- encoder interface;
- one software encoder;
- muxer;
- segmented crash-safe recording;
- output state/diagnostics;
- UI start/stop.

Exit condition: local recording survives interrupted finalization test.

---

## 8. Phase 6 — Streaming

Deliver:

- credential broker;
- one protocol;
- reconnect policy;
- network diagnostics;
- output isolation;
- recording plus streaming together.

Exit condition: network failure does not stop recording.

---

## 9. Phase 7 — Production Editing

Deliver:

- groups;
- transforms;
- text;
- images;
- effects;
- preview/program studio mode;
- transitions;
- undo/redo;
- asset registry.

---

## 10. Phase 8 — Cross-Platform Expansion

Complete:

- Windows;
- macOS;
- Linux;
- permissions;
- packaging;
- secure stores;
- hardware acceleration;
- update path.

---

## 11. Phase 9 — SDK and Extensions

Deliver:

- extension host;
- manifest;
- permissions;
- one declarative UI extension;
- one sample source/output;
- quotas;
- packaging/signing.

---

## 12. Phase 10 — Release Hardening

Deliver:

- performance baselines;
- soak/fault suites;
- accessibility pass;
- localization;
- crash reporting;
- compatibility matrix;
- signed installers;
- rollback;
- release gates.

---

## 13. Roadmap Rules

1. Each phase ends in a runnable slice.
2. Fake adapters precede fragile native integration.
3. One platform may lead, but interfaces remain portable.
4. Do not implement SDK before core contracts stabilize enough.
5. Do not postpone persistence/recovery until late.
6. Do not postpone observability.
7. Do not add broad feature depth before output reliability.

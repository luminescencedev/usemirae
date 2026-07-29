# 205 — Renderer Backend

**Status:** Proposed  
**Audience:** GPU, platform, rendering contributors  
**Canonical:** Yes  
**Required context:** `203-render-graph.md`, `206-gpu-resource-model.md`  
**Related ADRs:** ADR-0011, ADR-0013

---

## 1. Purpose

The renderer backend implements graph execution using `wgpu` while isolating the rest of Mirae from backend-specific types.

---

## 2. Responsibilities

- adapter and device selection;
- device and queue lifecycle;
- capability discovery;
- command encoder creation;
- render/compute/copy pass encoding;
- pipeline creation and caching;
- bind-group creation;
- surface configuration;
- external texture interop;
- submission tracking;
- GPU timing;
- device-loss detection and recovery.

---

## 3. Non-Responsibilities

The backend does not:

- own scene semantics;
- select source media frames;
- mutate project state;
- implement output retries;
- expose raw backend objects through domain APIs;
- decide project color policy.

---

## 4. Backend Interface

Conceptual boundary:

```rust
pub trait RendererBackend: Send + Sync {
    fn capabilities(&self) -> RenderCapabilities;
    fn device_generation(&self) -> DeviceGeneration;
    fn create_surface(&self, desc: SurfaceDescriptor) -> Result<SurfaceHandle>;
    fn execute(&self, graph: ExecutableRenderGraph) -> Result<RenderSubmission>;
    fn poll(&self, mode: PollMode) -> Result<()>;
    fn request_recovery(&self, reason: RecoveryReason);
}
```

Backend-specific objects remain private to implementation crates.

---

## 5. Adapter Selection

Selection considers:

- power preference;
- supported surface formats;
- texture limits;
- timestamp query support;
- external memory interop;
- hardware encoder compatibility;
- known driver workarounds;
- user override;
- stability score.

Selection result is recorded in diagnostics.

---

## 6. Capability Model

Capabilities include:

- maximum texture dimensions;
- supported formats;
- filterability;
- storage texture support;
- timestamp queries;
- bind group limits;
- push constant support;
- multisampling;
- external texture import/export;
- HDR surface support;
- backend API identity;
- driver and adapter metadata.

Capabilities are generation-stamped and treated as runtime external state.

---

## 7. Queue Submission

Submission contract:

- graph execution creates one or more command buffers;
- submission order is explicit;
- each submission receives an ID;
- completion is observed asynchronously;
- resource retirement uses completion information;
- CPU does not block every frame waiting for GPU completion.

---

## 8. Pipeline Cache

Pipeline keys include:

- shader module version;
- entry point;
- specialization constants;
- target formats;
- blend state;
- sample count;
- vertex layout;
- binding layout;
- device generation.

Pipelines are created outside critical per-frame sections when possible.

Compilation failure is cached with invalidation rules to avoid repeated storms.

---

## 9. Surface Management

A surface has:

- surface ID;
- generation;
- extent;
- format;
- color mode;
- present mode;
- alpha mode;
- owner;
- lifecycle state.

Outdated surfaces fail locally and request reconfiguration.

A resize does not mutate an in-use generation.

---

## 10. External Interop

Interop may include:

- imported capture textures;
- hardware encoder surfaces;
- platform swapchains;
- shared memory-backed textures.

Every interop path must define:

- ownership;
- acquire synchronization;
- release synchronization;
- format;
- color metadata;
- device compatibility;
- fallback copy path.

---

## 11. Unsafe Code

Unsafe or FFI code must be isolated in backend/platform modules.

Requirements:

- safety invariants documented;
- handles validated;
- lifetime not inferred from raw pointer alone;
- panic does not cross FFI;
- tests or validation probes exist where possible.

---

## 12. Workaround Database

Driver- or adapter-specific behavior is expressed through a versioned workaround database.

Workarounds:

- identify vendor/device/driver ranges;
- include reason and source;
- are observable in diagnostics;
- have removal review;
- do not spread arbitrary vendor checks throughout renderer code.

---

## 13. Invariants

1. Backend types do not leak into domain crates.
2. Device generation invalidates dependent resources.
3. Submission is asynchronous by default.
4. Pipeline keys include device generation.
5. Surface generation changes on reconfiguration.
6. External interop has explicit ownership.
7. Unsafe code is isolated.
8. Workarounds are centralized.
9. Adapter selection is diagnosable.
10. Device loss is recoverable where possible.

---

## 14. Required Tests

- capability snapshot;
- adapter selection policy;
- surface resize;
- pipeline cache hit/miss;
- stale device-generation rejection;
- external interop mock;
- submission completion;
- workaround activation;
- shader compile failure;
- device loss simulation;
- backend-independent fixture execution.

---

## 15. AI Implementation Notes

Do not return raw `wgpu::Texture`, `wgpu::Device`, or OS handles from public domain-facing APIs.

Include device generation in every backend-owned handle.

Do not call `device.poll(Maintain::Wait)` on the steady-state frame path without a documented reason.

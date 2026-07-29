# 207 — Shader System

**Status:** Proposed  
**Audience:** Rendering, effects, build tooling contributors  
**Canonical:** Yes  
**Required context:** `205-renderer-backend.md`, `206-gpu-resource-model.md`  
**Related ADRs:** ADR-0011

---

## 1. Purpose

The shader system manages shader source, validation, compilation, reflection, specialization, versioning, caching, diagnostics, and compatibility.

---

## 2. Shader Sources

Canonical built-in shader source is stored in the repository.

Each shader has:

- stable shader ID;
- source path;
- entry points;
- stage;
- binding schema;
- specialization schema;
- expected color/alpha domain;
- precision requirement;
- test coverage;
- version hash.

Generated shader code must be reproducible.

---

## 3. Language

WGSL is the initial canonical high-level shader language for `wgpu`.

Platform-specific shader code may exist only behind explicit backend requirements and must preserve semantic equivalence.

---

## 4. Binding Layout

Bindings are defined through typed descriptors.

A shader may not rely on undocumented binding numbers spread across call sites.

Preferred flow:

```text
shader schema
→ generated/reflected binding types
→ validated pipeline layout
→ typed parameter packing
```

---

## 5. Specialization

Specialization is used for bounded structural variants such as:

- source count class;
- blend mode;
- precision;
- color transform path;
- mask type;
- sample count.

Unbounded shader permutation growth is prohibited.

Variant count and compile cost are measured.

---

## 6. Compilation

Compilation phases:

1. load source;
2. resolve includes or generated modules;
3. validate syntax;
4. validate binding schema;
5. compute content hash;
6. compile module;
7. create pipeline variants;
8. cache result or failure;
9. publish diagnostics.

Compilation should occur ahead of first critical use where practical.

---

## 7. Hot Reload

Development builds may support shader hot reload.

Rules:

- production builds use packaged immutable shaders;
- failed reload preserves last valid shader;
- pipeline generation increments on successful reload;
- resource bindings are revalidated;
- hot reload cannot affect persisted project semantics;
- diagnostics identify source and entry point.

---

## 8. Effect Shader Contract

An effect shader declares:

- input count;
- input formats;
- working color domain;
- alpha convention;
- output format class;
- parameter schema;
- sampling behavior;
- time dependence;
- deterministic behavior;
- resource requirements;
- maximum workgroup or pass cost.

Third-party effect shaders are handled through extension security rules.

---

## 9. Pipeline Cache

Cache key includes:

- shader content hash;
- entry point;
- specialization;
- target formats;
- blend/depth state;
- layout;
- device generation;
- workaround flags.

Failure cache uses shorter bounded lifetime in development and stable diagnostics in production.

---

## 10. Validation

Validation includes:

- expected bindings;
- size/alignment;
- parameter range schema;
- storage access;
- texture format compatibility;
- workgroup limits;
- forbidden features for extension shaders;
- compile time and variant limits.

---

## 11. Security

Untrusted shader input may cause driver instability or denial of service.

Extension shaders require:

- manifest declaration;
- validation;
- capability grant;
- size limits;
- compile timeout where possible;
- resource and dispatch limits;
- isolation policy;
- known unsafe feature restrictions.

---

## 12. Invariants

1. Built-in shaders are versioned by content.
2. Binding layouts are explicit.
3. Permutations are bounded.
4. Failed hot reload preserves last valid pipeline.
5. Pipeline cache is device-generation scoped.
6. Color and alpha domains are declared.
7. Untrusted shaders are validated and restricted.
8. Production shaders are packaged immutably.
9. Diagnostics retain shader identity.
10. Generated code is reproducible.

---

## 13. Required Tests

- compile all built-in shaders;
- binding reflection/schema match;
- parameter packing;
- variant limit;
- cache key stability;
- hot reload success/failure;
- color-domain declaration;
- extension shader rejection;
- invalid workgroup size;
- device-generation invalidation;
- packaged shader manifest verification.

---

## 14. AI Implementation Notes

Do not hardcode binding indices independently in Rust and WGSL.

Do not generate unlimited shader variants for arbitrary parameter values.

Do not replace a valid production pipeline with a failed hot-reload result.

Keep effect color and alpha expectations explicit.

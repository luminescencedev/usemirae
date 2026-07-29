# 806 — Build System and Toolchain

**Status:** Proposed  
**Audience:** Build, release, all contributors  
**Canonical:** Yes  
**Required context:** `801-monorepo-architecture.md`  
**Related ADRs:** ADR-0060

---

## 1. Purpose

This document defines pinned toolchains and root build commands.

---

## 2. Toolchains

Pin:

- Rust channel and components in `rust-toolchain.toml`;
- Node version in a repository version file;
- pnpm version through package manager metadata;
- schema/codegen tool versions;
- formatter/linter versions through lockfiles;
- native dependency versions or package recipes;
- CI images/runners where practical.

---

## 3. Root Commands

Expected commands:

```text
pnpm install --frozen-lockfile
cargo xtask bootstrap
cargo xtask generate
cargo xtask check
cargo xtask test
cargo xtask test-integration
cargo xtask package
cargo xtask docs
```

UI-specific commands remain available through pnpm filters.

---

## 4. `xtask`

`xtask` owns:

- toolchain verification;
- code generation;
- multi-language checks;
- test orchestration;
- fixture generation;
- docs validation;
- packaging;
- release manifests;
- local diagnostics.

Shell scripts should call `xtask` rather than duplicate logic.

---

## 5. Native Dependencies

FFmpeg and other native dependencies use reproducible build/download recipes.

Requirements:

- version pin;
- checksum;
- license metadata;
- supported platform matrix;
- debug/release variants;
- architecture selection;
- no download from mutable unverified URLs.

---

## 6. Build Profiles

At minimum:

- development;
- development-with-diagnostics;
- test;
- benchmark;
- release;
- release-with-symbols;
- packaging.

Profile differences are documented.

---

## 7. Caching

CI/local caches may store:

- Cargo registry/git;
- Rust build artifacts;
- pnpm store;
- generated dependencies;
- native dependency builds.

Cache keys include relevant lockfiles and toolchain versions.

---

## 8. Invariants

1. Toolchains are pinned.
2. Lockfiles are committed.
3. Native artifacts are checksummed.
4. Root commands are stable.
5. `xtask` centralizes orchestration.
6. Build profiles are explicit.
7. CI cache keys are correct.
8. Release builds retain symbol mapping.
9. Packaging uses verified inputs.
10. Local and CI commands match.

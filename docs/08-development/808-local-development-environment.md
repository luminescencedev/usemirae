# 808 — Local Development Environment

**Status:** Proposed  
**Audience:** Contributors and coding agents  
**Canonical:** Yes  
**Required context:** `806-build-system-and-toolchain.md`

---

## 1. Purpose

This document defines the expected local setup and first-run workflow.

---

## 2. Required Tools

- Git;
- pinned Rust toolchain;
- pinned Node;
- pinned pnpm;
- platform compiler/build tools;
- required native SDKs;
- optional Docker only for isolated tooling, not core desktop execution;
- platform-specific signing credentials only for release maintainers.

---

## 3. Bootstrap

```text
git clone <repository>
cd mirae
pnpm install --frozen-lockfile
cargo xtask bootstrap
cargo xtask generate
cargo xtask check
```

`bootstrap` should explain missing platform dependencies.

---

## 4. Running

Expected development commands:

```text
cargo xtask dev
cargo xtask dev-engine
pnpm --filter @mirae/control-ui dev
cargo xtask dev-shell
```

The final command model may evolve, but there must be one documented default.

---

## 5. Local Configuration

Machine-local files:

- stay ignored;
- contain no committed secrets;
- use sample templates;
- are validated;
- have clear precedence.

Credentials use OS secure storage even in development when practical.

---

## 6. Test Devices and Media

Use:

- synthetic video/audio fixtures;
- fake capture providers;
- fake encoders;
- deterministic network sinks;
- optional local loopback targets.

Development must not require a live streaming account.

---

## 7. Troubleshooting

Bootstrap should detect:

- wrong Rust/Node/pnpm versions;
- missing native compiler;
- missing SDK;
- unsupported package mode;
- unavailable secure store;
- missing FFmpeg artifacts;
- incompatible GPU driver;
- stale generated files.

---

## 8. Invariants

1. Setup is reproducible.
2. Development does not require production credentials.
3. Local config is ignored.
4. Fixtures replace external services where possible.
5. Toolchain mismatches fail early.
6. Bootstrap is idempotent.
7. Platform limitations are explained.
8. Generated files are verified.
9. Dev mode does not bypass core security.
10. One default run command exists.

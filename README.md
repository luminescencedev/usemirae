# Mirae

Desktop-first, dark-first live production application. Native Rust core, GPU-first
rendering, React control UI.

The documentation in `docs/` is the source of truth for architecture, behavior,
compatibility, security, and quality. Start at [`docs/SUMMARY.md`](docs/SUMMARY.md).

## Repository layout

```text
apps/        deployable processes: engine, desktop-shell, extension-host, control-ui
crates/      Rust libraries grouped by responsibility (see docs/08-development/802)
packages/    TypeScript libraries: contracts, client, ui-kit, localization, config, test-utils
schemas/     canonical machine contracts and generated output
tools/       repository automation (xtask, codegen, fixtures, release)
tests/       cross-cutting integration, e2e, compatibility, performance, fault tests
fixtures/    shared, hash-pinned test fixtures
assets/      canonical brand sources
docs/        canonical documentation, ADRs, and visual references
```

Apps assemble libraries. Shared libraries never depend on apps. Full rules:
[`docs/08-development/804-dependency-rules.md`](docs/08-development/804-dependency-rules.md).

## Prerequisites

Exact versions are locked by [`DEPENDENCY_VERSIONS.md`](DEPENDENCY_VERSIONS.md).
Read it before touching any manifest, and never widen a pin to `^`, `~`, or `latest`.

| Tool | Exact version | Canonical file |
|---|---:|---|
| Rust | `1.97.1` | `rust-toolchain.toml` |
| Node.js | `24.18.1` | `.node-version` |
| pnpm | `11.17.0` | `package.json#packageManager` |
| TypeScript | `6.0.3` | pnpm catalog in `pnpm-workspace.yaml` |

Also required: Git and platform-native build tools. `MIR-0002` hardens these pins
and makes wrong versions fail early.

npm, Yarn, Bun, and Deno are not project package managers.

## Bootstrap

```bash
# 1. Rust workspace — 30 library crates and 3 application crates
cargo check --workspace

# 2. Frontend workspace — 6 packages and the control UI
pnpm install --frozen-lockfile

# Playwright is owned by apps/control-ui, so its binary is not on the root path
pnpm --filter @mirae/control-ui exec playwright install

# 3. Verify both workspaces
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
pnpm -r typecheck
pnpm -r lint
pnpm -r test
pnpm -r build
```

External versions live in the pnpm catalog. Package manifests reference them as
`"catalog:"`; do not repeat literal versions in a manifest.

Run the control UI on its own:

```bash
pnpm --filter @mirae/control-ui dev
```

Once `MIR-0003` lands, `cargo xtask` becomes the canonical command surface:

```text
cargo xtask bootstrap
cargo xtask generate --check
cargo xtask fmt --check
cargo xtask lint
cargo xtask test-affected
cargo xtask docs --check
```

## Current state

Sprint 0 scaffold. Every crate and package is a documented placeholder: the
workspaces resolve and compile, but no engine, IPC, rendering, or UI behavior
exists yet. The next ticket is the first unchecked item in
[`BOOTSTRAP_TICKETS.md`](BOOTSTRAP_TICKETS.md).

## Contributing

- [`CLAUDE.md`](CLAUDE.md) — engineering contract for human and coding agents
- [`DEPENDENCY_VERSIONS.md`](DEPENDENCY_VERSIONS.md) — authoritative version lock
- [`INSTALL.md`](INSTALL.md) — documentation and visual-system installation
- [`docs/08-development/811-git-and-pull-request-workflow.md`](docs/08-development/811-git-and-pull-request-workflow.md)
- [`docs/08-development/816-definition-of-done.md`](docs/08-development/816-definition-of-done.md)

One ticket = one branch = one focused pull request. Do not commit secrets,
signing keys, streaming credentials, or machine-local paths.

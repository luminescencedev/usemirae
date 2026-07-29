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

Also required: Git and platform-native build tools.

npm, Yarn, Bun, and Deno are not project package managers.

Verify the local toolchain at any time:

```bash
cargo xtask bootstrap
```

It also runs as the root `preinstall` hook, so a wrong Rust, Node, or pnpm version
fails before any dependency is fetched. It reports the expected version, the found
version, and the command that fixes each problem, and it changes nothing. To
install without the gate, use `pnpm install --ignore-scripts`.

## Bootstrap

```bash
# 1. Verify the toolchain (also runs automatically before any pnpm install)
cargo xtask bootstrap

# 2. Rust workspace — 30 library crates, 3 application crates, xtask
cargo check --workspace

# 3. Frontend workspace — 6 packages and the control UI
pnpm install --frozen-lockfile

# Playwright is owned by apps/control-ui, so its binary is not on the root path
pnpm --filter @mirae/control-ui exec playwright install

# 4. Verify everything
cargo xtask check
```

External versions live in the pnpm catalog. Package manifests reference them as
`"catalog:"`; do not repeat literal versions in a manifest.

Run the control UI on its own:

```bash
pnpm --filter @mirae/control-ui dev
```

## Commands

`cargo xtask` is the canonical command surface. Run `cargo xtask help` for the
full list, or `cargo xtask help <command>` for one command.

```text
cargo xtask bootstrap           verify the pinned toolchain
cargo xtask generate [--check]  run code generation, or fail on drift
cargo xtask fmt [--check]       format, or fail if formatting is needed
cargo xtask lint                lint with warnings denied
cargo xtask test                every test in both workspaces
cargo xtask test-affected       the tests affected by the working tree
cargo xtask test-integration    cross-cutting tests under tests/
cargo xtask docs [--check]      validate documentation structure
cargo xtask policy              secrets, local paths, dependency direction, pins
cargo xtask check               policy, generate, fmt, lint, test, docs
```

`fmt` covers rustfmt and prettier; `lint` covers clippy and ESLint. Both deny
warnings. `policy` enforces what can be checked mechanically: committed secrets,
machine-local paths, environment files, the dependency direction from
[`804`](docs/08-development/804-dependency-rules.md), and npm pin syntax.
TypeScript import boundaries are enforced by
[`eslint.config.js`](eslint.config.js).

Each command states what it does not yet cover and which ticket owns the rest, so
a passing run never implies more coverage than exists. The frontend commands stay
available through pnpm filters.

## Continuous integration

[`.github/workflows/ci.yml`](.github/workflows/ci.yml) runs the same commands on
`ubuntu-latest` and `windows-latest`, and reads the same version files as a
developer machine, so a CI version can never drift from the lock. Jobs still
missing from `docs/08-development/810-ci-cd-pipeline.md` are listed at the top of
the workflow with their owning ticket.

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

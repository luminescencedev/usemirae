# @mirae/toolchain-check

Fails early when the local toolchain does not match the version lock.

Canonical documentation:

- `DEPENDENCY_VERSIONS.md` — the authoritative version lock
- `docs/08-development/806-build-system-and-toolchain.md`
- `docs/08-development/808-local-development-environment.md` (invariant 5:
  toolchain mismatches fail early)

## Usage

```bash
pnpm run check:toolchain
```

It also runs as the root `preinstall` hook, so a wrong version is reported before
any dependency is fetched. To install without the gate:

```bash
pnpm install --ignore-scripts
```

## What it checks

1. **Pin syntax** — every pin is an exact `x.y.z`; `^`, `~`, `>=`, `*`, `latest`,
   `next`, `canary`, `beta`, and `rc` are rejected per `DEPENDENCY_VERSIONS.md`
   section 2.
2. **Pin agreement** — `.node-version`, `package.json#engines`, and
   `package.json#packageManager` must not contradict each other. A self-
   contradictory lock is reported before any machine comparison.
3. **Installed versions** — Node, pnpm, `rustc`, and `cargo` against the pins.
   `rustc` and `cargo` are checked separately so a partially installed toolchain
   is caught.
4. **Package manager** — refuses to run under npm, Yarn, Bun, or Deno, which
   would write a second lockfile.
5. **Lockfiles** — `pnpm-lock.yaml` and `Cargo.lock` must be present.

Every failure prints the expected version, the found version, and the command
that fixes it. No pin is written in this package: the canonical files are the only
source, so the tool cannot disagree with the lock.

## Structure

```text
src/pins.ts   pure parsing and comparison rules (no I/O, fully unit tested)
src/cli.ts    file reads, tool probes, operator output, exit code
tests/        35 unit tests covering each rule and its failure mode
```

`cli.ts` uses only Node built-ins, because `preinstall` runs before
`node_modules` exists. Node 24 executes the TypeScript sources directly through
type stripping, so this package needs no build step and no extra loader
dependency.

## Removal condition

This package is temporary. `MIR-0003` gives `cargo xtask bootstrap` ownership of
toolchain verification (`806` section 4). When `cargo xtask bootstrap` performs
these checks, delete this package and the root `preinstall` hook.

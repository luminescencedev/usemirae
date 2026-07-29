# Mirae Bootstrap Tickets

Canonical source: `docs/08-development/814-bootstrap-ticket-backlog.md`

## Current Sprint — Sprint 0

- [x] MIR-0001 — Initialize monorepo
  - status: done
  - branch: `feat/MIR-0001-initialize-monorepo`
  - PR: not published
  - validation: `cargo check --workspace`, `cargo fmt --all -- --check`,
    `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
    `cargo test --workspace`, `pnpm install --frozen-lockfile`,
    `pnpm -r typecheck`, `pnpm -r test`, `pnpm -r build` — all exit 0
  - follow-up: MIR-0002 (toolchain enforcement), MIR-0003 (`xtask`),
    MIR-0004 (policy checks and ESLint flat config), MIR-DEPS-0001 (ESLint 10
    vs `eslint-plugin-jsx-a11y` peer conflict)
- [x] MIR-0002 — Pin toolchains
  - status: done
  - branch: `main` (trunk-based; no pull request)
  - validation: `pnpm run check:toolchain` (pass, and exit 1 with actionable
    output when a pin is deliberately broken), `pnpm --filter
    @mirae/toolchain-check test` (35 tests), `cargo fmt --all -- --check`,
    `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
    `cargo test --workspace`, `pnpm install --frozen-lockfile`,
    `pnpm -r typecheck`, `pnpm -r test`, `pnpm -r build` — all exit 0
  - notes: version files were committed by MIR-0001; this ticket added
    enforcement (`tools/toolchain-check` plus the root `preinstall` hook) and CI
    parity (`.github/workflows/ci.yml`). The workflow YAML is not parseable
    locally — no approved YAML parser is installed — so its first GitHub run is
    its proof.
  - follow-up: MIR-0003 absorbs the check into `cargo xtask bootstrap` and
    deletes `tools/toolchain-check`; MIR-0004 adds the policy, secret-scan and
    lint jobs; MIR-DEPS-0001 covers the ESLint 10 peer conflict
- [ ] MIR-0003 — Add `xtask`
- [ ] MIR-0004 — Add repository policy checks
- [ ] MIR-0005 — Create canonical schema skeleton
- [ ] MIR-0006 — Generate Rust and TypeScript handshake contracts
- [ ] MIR-0007 — Create structured error foundation
- [ ] MIR-0008 — Create structured tracing foundation
- [ ] MIR-0009 — Build engine process skeleton
- [ ] MIR-0010 — Build native shell skeleton
- [ ] MIR-0011 — Build React control UI skeleton
- [ ] MIR-0012 — Add authenticated IPC handshake
- [ ] MIR-0013 — Add engine crash/restart smoke test
- [ ] MIR-0014 — Add documentation link validator
- [ ] MIR-0015 — Add first integration test harness

## Workflow

The first unchecked, unblocked ticket is the next ticket.

Update each item with:

- status;
- branch;
- PR;
- validation;
- follow-up ticket links.

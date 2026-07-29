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
  - CI: run 30496187331 green on `ubuntu-latest` and `windows-latest`
    (toolchain, rust, frontend)
  - follow-up: MIR-0003 absorbs the check into `cargo xtask bootstrap` and
    deletes `tools/toolchain-check`; MIR-0004 adds the policy, secret-scan and
    lint jobs; MIR-DEPS-0001 covers the ESLint 10 peer conflict;
    MIR-TOOLING-0001 moves the GitHub actions off the deprecated Node 20 runtime
    (`@v5` needs pnpm available before `setup-node`, see the workflow header)
- [x] MIR-0003 — Add `xtask`
  - status: done
  - branch: `main` (trunk-based; no pull request)
  - commands: `bootstrap`, `generate [--check]`, `fmt [--check]`, `lint`, `test`,
    `test-affected`, `test-integration`, `docs [--check]`, `check`, plus
    `help [command]` for every one of them
  - validation: `cargo xtask check` (exit 0), `cargo test --package xtask`
    (44 tests), `cargo fmt --all -- --check`, `cargo clippy --workspace
    --all-targets --all-features -- -D warnings`, `pnpm install --frozen-lockfile`
    (preinstall now runs `cargo xtask bootstrap`), `pnpm -r typecheck`,
    `pnpm -r test`, `pnpm -r build` — all exit 0
  - negative test: a deliberately wrong `rust-toolchain.toml` channel made
    `cargo xtask bootstrap` exit 1 and report both `rustc` and `cargo` with fixes
  - notes: `tools/toolchain-check` was deleted and its rules ported to
    `tools/xtask`, so the temporary duplicate from MIR-0002 is gone. `xtask` has
    no dependencies: an argument parser would have to clear the Rust dependency
    procedure in `DEPENDENCY_VERSIONS.md` section 11. `cargo xtask docs --check`
    validates 224 SUMMARY links and that all 66 ADRs are indexed exactly once.
    Every command states what it does not yet cover and which ticket owns the
    rest, so a pass never implies more coverage than exists.
  - follow-up: MIR-0004 (policy checks, ESLint and prettier configuration,
    affected-set detection), MIR-0005/MIR-0006 (register generators),
    MIR-0014 (extend `docs --check`), MIR-0015 (`test-integration` harness);
    `package` and `dev` commands from doc 806 remain unimplemented
- [x] MIR-0004 — Add repository policy checks
  - status: done
  - branch: `main` (trunk-based; no pull request)
  - formatting: prettier pinned config plus `.prettierignore`; `cargo xtask fmt`
    now runs rustfmt and prettier together. `docs/`, `assets/`, `*.md`, and the
    generated `packages/ui-kit/src/styles/tokens.css` are excluded, the last one
    because `cargo xtask generate` will own its formatting.
  - linting: `eslint.config.js` flat config on the pinned stack (eslint 10,
    typescript-eslint, react-hooks, react-refresh, jsx-a11y) with
    `no-restricted-imports` encoding the 804 boundaries for TypeScript;
    `cargo xtask lint` runs clippy and ESLint with warnings denied.
  - policy: `cargo xtask policy` checks committed secrets, machine-local paths,
    committed environment files, Rust dependency direction per 804 section 3, and
    npm pin syntax per `DEPENDENCY_VERSIONS.md` section 2.
  - generated drift: `cargo xtask generate --check` runs in CI already and reports
    an empty registry, so the gate exists before there is output to drift.
  - validation: `cargo xtask check` (exit 0), `cargo test --package xtask`
    (64 tests), `pnpm exec eslint .`, `pnpm exec prettier --check .`,
    `pnpm -r typecheck`, `pnpm -r test`, `pnpm -r build` — all exit 0
  - negative tests: a planted `apiKey` literal, a planted Windows user-profile
    path, and a `crates/domain/domain` manifest depending on `wgpu` and
    `mirae-engine` all
    produced violations and exit 1; a probe `.tsx` with `any` and an `img` without
    `alt` produced two ESLint errors.
  - notes: the first policy run produced 5 false positives (design-token paths
    matching `token` as a substring). The matcher now compares the assignment key
    rather than the line, and those exact cases are regression tests. The
    ESLint 10 and `eslint-plugin-jsx-a11y` peer warning is a stale declaration:
    the plugin loads and its rules fire under ESLint 10.
  - follow-up: type-checked ESLint rules need `parserOptions.project` wiring;
    Cargo pin syntax (`=x.y.z` per section 11) is unchecked because no external
    crate exists yet; affected-set detection for `test-affected` is still
    unimplemented and is not part of this ticket's goal
- [x] MIR-0005 — Create canonical schema skeleton
  - status: done
  - branch: `main` (trunk-based; no pull request)
  - directories: `schemas/<domain>/v1/` for all eight canonical domains from doc
    805 section 2 (`ipc`, `project`, `bundle`, `diagnostics`, `sdk`,
    `extension-manifest`, `extension-ui`, `compatibility`), plus
    `schemas/generated/{rust,typescript}/`
  - pipeline: `cargo xtask generate [--check]` discovers schemas, validates
    required fields, requires each `$id` to match the directory holding it,
    rejects duplicate ids, renders output sorted by id, then writes or verifies
    it. Rendering is pure, so determinism is unit tested.
  - validation: `cargo xtask check` (exit 0), `cargo test --package xtask`
    (77 tests), `pnpm -r typecheck`, `pnpm -r test`, `pnpm -r build`,
    `pnpm install --frozen-lockfile` — all exit 0
  - negative tests: a hand-edited `schemas/generated/manifest.json` made
    `generate --check` exit 1 and name the stale file; a schema whose `$id`
    claimed another domain was rejected with the expected prefix in the message
  - notes: zero schemas exist, so the generated outputs are empty — but they are
    written, committed, and verified, so the drift gate is live before the first
    contract. Doc 801 lists a shorter directory set as an example; the eight
    domains above come from the schema-canonical document. Generated output is
    excluded from prettier and ESLint because it is not hand-maintained.
  - follow-up: MIR-0006 adds the first real contract (protocol version and
    readiness) and full JSON Schema validation; fixture generation from doc 805
    section 4 point 4 is not implemented yet
- [x] MIR-0006 — Generate Rust and TypeScript handshake contracts
  - status: done
  - branch: `main` (trunk-based; no pull request)
  - schemas: `mirae://ipc/v1/protocol-version` (major `const 1`, minor `const 0`)
    and `mirae://ipc/v1/engine-readiness` (state enum, protocol major and minor,
    bounded session id, optional safe detail), both from doc 108 sections 6 and 8
  - generator: `schema.rs` now parses properties into a typed model and emits real
    Rust structs, enums with stable wire strings, constants, and length bounds,
    plus matching TypeScript interfaces, union types, and constants. A
    hand-written JSON parser (`json.rs`) and identifier conversions (`naming.rs`)
    back it; no dependency was added.
  - ownership: bindings are generated into
    `crates/foundation/contracts/src/generated.rs` and
    `packages/contracts/src/generated.ts`, the crate and package that own them per
    docs 802 and 803, rather than a shared directory
  - validation: `cargo xtask check` (exit 0), `cargo test --package xtask`
    (89 tests), `cargo test --package mirae-contracts` (6 tests),
    `pnpm --filter @mirae/contracts test` (6 tests), `pnpm -r typecheck`,
    `pnpm -r build`, `pnpm install --frozen-lockfile` — all exit 0
  - cross-language agreement: the Rust and TypeScript suites assert the same
    facts, and an xtask test checks that every field appears in both outputs with
    matching optionality
  - notes: generated Rust output is rustfmt-clean, verified by running
    `cargo fmt --all` and confirming `generate --check` still passes; generated
    TypeScript is excluded from prettier and ESLint because `--check` is the only
    authority on its content. The supported schema subset is documented and
    enforced: a property without a `description`, an integer without a `maximum`,
    a string without `maxLength` or `enum`, or a document without
    `"additionalProperties": false` is rejected with the property named.
  - follow-up: `$ref`, composition, and nested objects are unsupported; no
    serialization derive exists yet, since serde would need the Rust dependency
    procedure in `DEPENDENCY_VERSIONS.md` section 11 (MIR-0012 needs it);
    protocol fixtures and validators from doc 805 section 3 are not generated
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

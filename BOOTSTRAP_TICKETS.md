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
- [x] MIR-0007 — Create structured error foundation
  - status: done
  - branch: `main` (trunk-based; no pull request)
  - crate: `crates/foundation/errors` (`mirae-errors`), `std` only, no dependency
  - types: `ErrorCode` (validated `SCREAMING_SNAKE_CASE`, `const fn` so an invalid
    literal fails to compile), `ErrorCategory` (the twelve layers from doc 605
    section 2), `ErrorSeverity`, `Retryability`, `SubsystemId`, `UserActionHint`,
    `CorrelationId` (u128, matching the IPC frame header in doc 108 section 4),
    `ErrorContext`, and `MiraeError` in the shape doc 605 section 3 specifies
  - behavior: category drives default severity and retryability, so classifying
    once gives consistent recovery; `Retryability::Unknown` never allows an
    automatic retry; cancellation is `Info` and reports `is_failure() == false`
    (605 invariant 8); `root_cause` walks the source chain (invariant 6);
    `Display` and `diagnostic_line` emit only the safe message and never the
    source chain
  - bounds: context is capped at 16 entries and counts what it dropped, text
    values at 128 characters, safe messages at 240, truncation on character
    boundaries so multi-byte text cannot be split
  - redaction: absolute Windows and POSIX home paths are replaced before a message
    or text value is stored. Scope is stated in the module: paths only, not a
    general secret scanner, and it cannot make an unsafe message safe.
  - validation: `cargo xtask check` (exit 0), `cargo test --package mirae-errors`
    (42 unit tests plus 1 doctest), `cargo test --package xtask` (95 tests),
    `cargo clippy --package mirae-errors --all-targets --all-features` — all
    exit 0
  - tooling change this ticket forced: `cargo xtask policy` flagged the redaction
    module's own fixtures, since a module that removes paths must contain them.
    Rather than hardcode more exempt files, the checker now honors a
    `policy-allow: <rule> - <reason>` marker in a file header, requires the
    reason, reports every exemption in use, and treats a marker without a reason
    as a violation. Five exemptions exist, all listed on each run. Two parser
    defects were caught by tests while building it: splitting on the first `-`
    cut `local-path` in half, and markers inside test fixtures registered as real
    exemptions, which is why markers only count in the first 40 lines.
  - required tests from doc 605 section 12: code stability, secret redaction,
    user-safe formatting, nested context, retry classification, cancellation
    behavior, and aggregation are covered. Panic boundary belongs to MIR-0008 and
    crash reporting; platform error translation, IPC error mapping, corrupted
    project, and extension error isolation belong to the subsystems that will
    produce those errors.
  - follow-up: no concrete error codes are defined, since each subsystem owns its
    own; a code registry with cross-subsystem uniqueness checks may be worth a
    ticket once several subsystems declare codes
- [x] MIR-0008 — Create structured tracing foundation
  - status: done
  - branch: `main` (trunk-based; no pull request)
  - crate: `crates/foundation/observability` (`mirae-observability`), `std` plus
    `mirae-errors` only. That dependency stays inside the foundation layer, so it
    crosses no boundary in doc 804 section 2, and `cargo xtask policy` confirms it.
  - identity: `EngineSessionId`, `ProcessRole`, `ProcessIdentity`, and
    `ClockOrigin`, which maps monotonic nanoseconds to the wall clock so ordering
    survives a clock adjustment. A session learned later, such as from a
    handshake, is adopted without rebuilding the tracer.
  - fields: `RedactionClass` from doc 606 section 5 is part of the field rather
    than a sink decision, so the same field cannot be safe in one place and unsafe
    in another. Secret and media-content fields are never written and each attempt
    is counted; private values are hashed with a stable FNV-1a so occurrences
    correlate without revealing the value; text is redacted and truncated when the
    field is built.
  - volume: per-event-name token buckets and consecutive-duplicate suppression,
    with counters for suppressed, rate-limited, and untracked events. Time is
    passed in, so every decision is tested without sleeping.
  - sinks: `RollingFileSink` with size and file-count retention, plus null,
    memory, and deliberately failing sinks. A write failure is counted and never
    propagated: a logging failure must not fail its caller.
  - real-time: `RealtimeCounters` is atomics only, for audio and capture callbacks
    that must not format or write synchronously (606 section 8). A later tick
    reads and resets them and emits one ordinary event.
  - format: one JSON object per line carrying timestamp, monotonic time, level,
    subsystem, event name, session, role, build, thread, optional correlation, and
    fields, so per-process files can be merged by tooling (606 section 7).
  - validation: `cargo xtask check` (exit 0), `cargo test --package
    mirae-observability` (37 tests), `cargo test --workspace`,
    `cargo clippy --workspace --all-targets --all-features` — all exit 0
  - required tests from doc 606 section 12: structured schema, secret field
    rejection, rate limiting, duplicate suppression, log rotation, disk-full
    behavior, audio callback counters, and multi-process merge are covered.
    Correlation propagation is covered within a process; across IPC it belongs to
    MIR-0012. Extension quota belongs to the SDK, redacted export to the support
    bundle work, and crash preservation to crash reporting.
  - follow-up: no global logger or macro exists, so a tracer is passed explicitly;
    spans from doc 606 section 3 are not implemented, only events
- [x] MIR-0009 — Build engine process skeleton
  - status: done
  - branch: `main` (trunk-based; no pull request)
  - crate: `crates/runtime/runtime` (`mirae-runtime`) on the three foundation
    crates only; `apps/engine` assembles them and holds no domain logic
  - lifecycle: all eleven states from doc 102 section 2 with the transition graph
    from section 3 enforced. `LifecycleCoordinator` is the single serialized
    authority: services report health and never transition the engine themselves
    (section 16). An illegal transition is refused and counted, never applied.
  - services: registration order is the dependency order from section 5, shutdown
    reverses it. An optional service that is unavailable impairs the engine
    without failing it (invariant 5); a mandatory one that fails ends startup and
    names the service.
  - readiness: built from the generated `mirae://ipc/v1/engine-readiness`
    contract, so the engine and any client share one vocabulary. The lifecycle
    graph only allows `Degraded` from `Running`, so a `Ready` engine missing an
    optional capability reports `ready` with the impairment in `detail` rather
    than hiding it.
  - shutdown: every started service is asked to stop even after an earlier one
    fails or overruns, so one bad service cannot strand the rest. Deadlines are
    recorded, not preemptive: this crate has no executor that could cancel a
    blocking call, which is stated in the code rather than implied.
  - crash context: session, lifecycle state, recent states, protocol versions,
    build id, and bounded safe diagnostics (section 13, invariant 9).
  - binary: `cargo run --package mirae-engine` bootstraps, initializes, prints the
    readiness contract as one JSON line, and shuts down cleanly (exit 0);
    `--version` reports the protocol version; an unknown argument exits 1. Events
    land in a rolling log under the system temporary directory, never beside user
    projects (doc 606 section 9).
  - validation: `cargo xtask check` (exit 0), `cargo test --package mirae-runtime`
    (24 tests), `cargo test --workspace`, and a manual run of the binary showing
    `{"state":"ready",...,"detail":"ipc_server: ..."}` plus lifecycle transitions
    in the log file
  - required tests from doc 102 section 15: complete startup, optional subsystem
    unavailable, mandatory subsystem failure, shutdown timeout, lifecycle command
    rejection, and crash context redaction are covered. Project activation and
    rollback, degraded-to-running recovery beyond the transition test, shutdown
    while an output is active, and cancellation during output start need
    subsystems that do not exist yet.
  - follow-up: no IPC server, so the process reports and exits instead of waiting
    (MIR-0012); services are `StubService` values until the real subsystems land;
    the session id is derived from the clock and process id, and a
    cryptographically random one belongs to the platform layer with the launch
    credentials
- [x] MIR-0010 — Build native shell skeleton
  - status: done
  - branch: `main` (trunk-based; no pull request)
  - supervision: `mirae_runtime::supervisor` holds the logic behind an
    `EngineLauncher` trait, so launch, crash, restart, and give-up are tested
    without spawning processes. `apps/desktop-shell` supplies the real launcher.
  - launch credential: `LaunchCredential` is bounded, its `Debug` is redacted, and
    it travels in an environment variable rather than on the command line where
    the process table would expose it. It is explicitly not cryptographic yet:
    the constructor is named `placeholder`, and real entropy plus verification
    belong to MIR-0012.
  - no fabricated state: the observed readiness is cleared the moment the engine
    goes away, so the shell reports what it can see rather than what it last saw
    (doc 501 section 6, section 13).
  - bounded restart: three restarts per minute by default, then the supervisor
    gives up with a reason. The budget refills once the window passes, so an old
    crash does not count against a later one.
  - cooperative shutdown: the engine gained `--supervised`, which keeps it alive
    until the shell closes its stdin. Closing the pipe is the shutdown request;
    killing is the fallback after a five second deadline. Verified against the
    real binaries: the engine ran its own lifecycle to `stopped` and left no
    orphan process.
  - validation: `cargo xtask check` (exit 0), `cargo test --package mirae-runtime`
    (37 tests), and a real run of `mirae-shell.exe` reporting
    `engine connected: session=... protocol=1.0 launches=1` plus the engine's
    impairment detail
  - required tests from doc 501 section 12: engine startup and engine crash are
    covered. UI reload, navigation block, external link, second-instance
    forwarding, drops, menu commands, multi-monitor restore, update restart, and
    the fatal recovery flow all need the window and webview.
  - notes: a real bug surfaced only by running the binaries. The engine used to
    exit immediately after reporting, so the supervisor read a normal exit as a
    crash loop and gave up after three relaunches. `--supervised` makes the engine
    lifetime the shell's to decide, which is what doc 501 section 6 point 7 asks
    for.
  - follow-up: no window and no embedded webview. Doc 501 section 3 requires a
    native webview over locally packaged resources, and every candidate is either
    unapproved by `DEPENDENCY_VERSIONS.md` section 14 or a new Rust dependency
    that must clear section 11. That choice needs its own ticket and an ADR, so
    the shell reports to the terminal today and the control UI runs on its own dev
    server. The readiness line is also a stop-gap: MIR-ADR-0001 picks the wire
    format and MIR-0012 replaces the handoff with the authenticated handshake.
- [ ] MIR-0011 — Build React control UI skeleton
- [ ] MIR-0012 — Add authenticated IPC handshake
  - blocked by: MIR-ADR-0001 (the wire format must be decided before the
    handshake is encoded)
- [ ] MIR-0013 — Add engine crash/restart smoke test
- [ ] MIR-0014 — Add documentation link validator
- [ ] MIR-0015 — Add first integration test harness

## Deferred Follow-Ups

Raised by completed tickets. Each is a separate ticket rather than hidden scope
(doc 812 invariant 7). None is started.

### MIR-ADR-0001 — Decide the IPC serialization format

```text
ID:                MIR-ADR-0001
Title:             Decide the IPC serialization format
Status:            ready
Goal:              Choose the schema-driven serialization format for control-plane
                   IPC, record it as an ADR, and pin the crates it needs.
User/System Value: The engine and shell cannot exchange a single message until the
                   wire format is chosen. Deciding it once, in the open, prevents a
                   format being adopted implicitly by whichever ticket needs bytes
                   first.
Canonical Docs:    docs/01-runtime/108-ipc-protocol.md sections 4, 5, 8, 9
                   docs/08-development/805-generated-contracts-and-schemas.md
                   DEPENDENCY_VERSIONS.md section 11
                   ADR-0006 (typed versioned IPC), ADR-0057 (generated contracts)
Scope:             Evaluate candidate formats against the 108 section 5 criteria:
                   Rust and TypeScript generation, explicit optional fields,
                   version evolution, bounded decoding, deterministic fixtures, and
                   no arbitrary code execution during decode. Write the ADR. Add
                   the chosen crates through the section 11 procedure: justify,
                   pin exactly, commit Cargo.lock, record them in a Rust dependency
                   section of DEPENDENCY_VERSIONS.md, and run license and security
                   review. Extend `cargo xtask generate` to emit the matching
                   derives or codecs.
Out of Scope:      The handshake itself, transport selection, authentication, and
                   the data plane for media payloads.
Dependencies:      MIR-0006 (contracts exist and are generated)
Implementation
Notes:             108 section 5 states the format requires an ADR if not already
                   covered, and no existing ADR covers it. JSON is explicitly not
                   automatically the canonical high-frequency encoding, though it
                   may serve diagnostics and development tooling. The framing in
                   108 section 4 is separate from the payload encoding; both must
                   stay bounded and validated before allocation.
Acceptance
Criteria:          - a new ADR records the decision, the alternatives, and the
                     trade-offs against every 108 section 5 criterion;
                   - the chosen crates are pinned exactly and recorded in
                     DEPENDENCY_VERSIONS.md;
                   - `cargo xtask generate` produces the encoding support, and
                     `--check` still detects drift;
                   - a round-trip test proves a generated type survives encode and
                     decode unchanged;
                   - a decoder rejects an oversized payload before allocating for
                     it.
Required Tests:    round trip per contract, unknown-field behavior across a minor
                   version bump, oversized-payload rejection, malformed-input
                   rejection.
Required
Diagnostics:       decode failure counters and a safe reason string; no raw payload
                   bytes or panic text in logs.
Performance /
Security Notes:    control IPC carries no media; decoding runs on untrusted input
                   from another process, so bounds are mandatory rather than
                   advisory.
Validation:        cargo xtask check, cargo xtask generate --check,
                   cargo test --workspace, pnpm -r test
Deliverables:      the ADR, the dependency records, generator support, tests.
```

### MIR-DEPS-0001 — Resolve the ESLint 10 peer declaration

`eslint-plugin-jsx-a11y@6.10.2` declares support up to ESLint 9, so
`pnpm peers check` reports an unmet peer. Verified as cosmetic: the plugin loads
under ESLint 10 and its rules fire, proven by a probe component that produced
`jsx-a11y/alt-text`. Close it by upgrading to a release that declares ESLint 10, or
record the accepted exception. Until then the zero-peer-warning item in
`DEPENDENCY_VERSIONS.md` section 17 cannot be ticked.

### MIR-TOOLING-0001 — Move CI actions off the deprecated Node 20 runtime

`actions/checkout@v4` and `actions/setup-node@v4` annotate every job.
`setup-node@v5` resolves `package.json#packageManager` before `corepack enable`
runs and fails to locate pnpm (observed on run 30496018726). Needs pnpm installed
before `setup-node`, or its package-manager detection disabled.

### MIR-TOOLING-0002 — Implement affected-set detection

`cargo xtask test-affected` currently runs the full suite and says so. Real change
detection needs a dependency graph over both workspaces and a diff base.

### Generator gaps, each driven by the first contract that needs one

- `$ref`, composition, and nested objects: expected to be needed by the project
  schema (MIR-0107).
- Protocol fixtures and validators from doc 805 section 3: expected to be needed by
  MIR-0013 and MIR-0015, and by the compatibility corpus in MIR-0115.
- Cargo pin syntax (`=x.y.z`) is unchecked by `cargo xtask policy` because no
  external crate exists yet; add the check with the first one, which MIR-ADR-0001
  will introduce.

## Workflow

The first unchecked, unblocked ticket is the next ticket.

Update each item with:

- status;
- branch;
- PR;
- validation;
- follow-up ticket links.

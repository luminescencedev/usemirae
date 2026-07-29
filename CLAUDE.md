# CLAUDE.md — Mirae Engineering Contract

## Mission

Build Mirae as documented in `docs/`.

The repository documentation is the source of truth for architecture, behavior, compatibility, security, and quality.

## Before Coding

0. Read `DEPENDENCY_VERSIONS.md`. It is the authoritative version lock and must be
   read before creating or modifying any JavaScript, TypeScript, React, UI-kit,
   test, lint, or package-manager configuration.
1. Read the active ticket.
2. Read every canonical document linked by the ticket.
3. Read:
   - `docs/08-development/800-development-overview.md`
   - `docs/08-development/804-dependency-rules.md`
   - `docs/08-development/809-testing-and-validation-workflow.md`
   - `docs/08-development/815-ai-build-workflow.md`
   - `docs/08-development/816-definition-of-done.md`
4. Inspect existing code and tests.
5. Check `git status`.
6. Identify contract, schema, security, performance, and platform impact.

## Working Rule

One ticket = one branch = one focused pull request.

Do not silently expand scope.

## Architecture Rules

- Domain code must not depend on UI, OS SDKs, `wgpu`, FFmpeg, or vendor SDKs.
- Apps assemble libraries; shared libraries never depend on apps.
- Interfaces are owned by the inward layer that needs the behavior.
- Cross-process and persisted contracts come from canonical schemas.
- Generated code is never edited manually.
- Third-party extensions never execute in the engine process.
- All queues, pools, caches, retries, histories, and operation sets are bounded.
- All replaceable resources use generations.
- Recoverable external failures do not panic.
- Secrets never enter project files, logs, telemetry, bundles, or ordinary config.

## Implementation Rules

- Implement the smallest compliant vertical slice.
- Add structured errors and diagnostics with the feature.
- Add tests with the feature.
- Preserve failure and recovery behavior.
- Avoid unrelated refactors.
- Do not add temporary duplicate architectures without a removal ticket.
- Do not suppress lints globally.
- Do not use `unwrap` or `expect` in recoverable production paths.
- Do not use unbounded channels.
- Do not add direct OS checks to domain/UI logic; use capabilities.
- Do not put project truth in React local state.

## Validation

Run the repository's canonical commands, expected to be exposed through `cargo xtask`.

Minimum:

```text
cargo xtask generate --check
cargo xtask fmt --check
cargo xtask lint
cargo xtask test-affected
cargo xtask docs --check
```

Run broader tests when changing foundation, contracts, persistence, IPC, rendering, audio, security, updater, or SDK behavior.

Never claim success without command evidence.

## Git Workflow

Branch naming:

```text
feat/<ticket>-<slug>
fix/<ticket>-<slug>
docs/<ticket>-<slug>
chore/<ticket>-<slug>
```

Default merge:

```text
gh pr merge <number> --squash --delete-branch
```

Do not merge without explicit authorization.

## Completion Report

Report:

- ticket;
- files changed;
- behavior added;
- tests added;
- commands run and results;
- docs/ADRs changed;
- assumptions;
- remaining risks;
- follow-up tickets;
- PR link when created.

## Next Ticket Command

When the user says `next ticket` or `ticket suivant`:

1. read the active sprint tracker;
2. choose the first ready unblocked ticket;
3. mark it in progress;
4. execute one complete ticket;
5. validate;
6. create a PR if publishing is enabled;
7. update the tracker;
8. stop after that ticket.

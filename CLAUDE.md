# CLAUDE.md — Mirae Engineering Contract

## Mission

Build Mirae as documented in `docs/`.

The repository documentation is the source of truth for architecture, behavior,
compatibility, security, product design, and quality.

## Language and Communication

- Write source code, identifiers, types, filenames, comments, documentation, test
  names, branch names, commit messages, pull requests, diagnostics, and user-facing
  product copy in English.
- Communicate directly with the project owner in French.
- Write plans, progress updates, explanations, blocker reports, trade-offs, and
  completion reports in French.
- Quote commands and tool output verbatim when useful, then explain their meaning
  and consequences in French.
- Keep established technical names in English when translating them would reduce
  precision.

## Before Coding

0. Read `DEPENDENCY_VERSIONS.md`. It is the authoritative version lock and must be
   read before creating or modifying JavaScript, TypeScript, React, UI-kit, test,
   lint, package-manager, or Rust dependency configuration.
1. Read the active ticket.
2. Read every canonical document linked by the ticket.
3. Read:
   - `docs/08-development/800-development-overview.md`
   - `docs/08-development/804-dependency-rules.md`
   - `docs/08-development/809-testing-and-validation-workflow.md`
   - `docs/08-development/815-ai-build-workflow.md`
   - `docs/08-development/816-definition-of-done.md`
4. Read `AGENT_SKILLS.md` when the ticket changes UI, security boundaries, or PR
   workflow.
5. Inspect existing code and tests.
6. Check `git status`.
7. Identify contract, schema, security, performance, product-design, and platform
   impact.

## Working Rule

One ticket = one branch = one focused pull request.

Do not silently expand scope.

Implement the smallest compliant, product-shaped vertical slice.

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
- The engine remains authoritative for project and production state.
- The shell hosts, validates, forwards, and projects state; it must not become a
  second project authority.

## Implementation Rules

- Implement the smallest compliant, product-shaped vertical slice.
- Add structured errors and diagnostics with the feature.
- Add tests with the feature.
- Preserve failure and recovery behavior.
- Avoid unrelated refactors.
- Do not add temporary duplicate architectures without a removal ticket.
- Do not suppress lints globally.
- Do not use `unwrap` or `expect` in recoverable production paths.
- Do not use unbounded channels.
- Do not add direct OS checks to domain or UI logic; use capabilities.
- Do not put project truth in React local state.
- Resolve minor details from canonical docs, existing code, and established
  patterns instead of asking unnecessary questions.
- Reuse existing contracts, components, fixtures, helpers, and test utilities
  before creating new ones.
- Do not introduce a generic abstraction solely to avoid a small amount of
  ticket-specific code.
- Do not overbuild unavailable future capabilities.

## Product-Shaped UI Rules

A temporary capability is allowed. A visibly disposable interface is not.

- Build every user-visible feature in the final product's information architecture.
- Do not create isolated debug pages when the information already has a future
  product location.
- Temporary UI must either occupy its intended long-term location or name the
  ticket that will move or remove it.
- Prefer a smaller polished surface over a larger unfinished surface.
- Never display fake engine, project, render, source, output, or connection state.
- Represent incomplete capabilities with honest loading, empty, unavailable,
  disabled, degraded, recovery, and error states.
- Preserve the workspace structure when the engine disconnects; do not replace the
  application with a debug card.
- Show normal engine connectivity as a compact persistent status. Put protocol,
  session, retry, transport, and raw diagnostic details in a dedicated diagnostics
  surface.

## UI Engineering Rules

For every user-visible UI ticket, use the project skill
`mirae-ui-engineering` and follow `docs/09-ui-ux/`.

- Use Mirae semantic tokens for color, spacing, typography, radius, border,
  elevation, focus, and motion.
- Use `@mirae/ui-kit` as the public component API.
- Keep third-party primitive APIs private to `@mirae/ui-kit`.
- Use `react-aria-components` as the approved accessible interaction foundation
  after the dedicated migration ticket lands.
- Use React Aria for collection semantics such as trees, lists, tables, menus, and
  collection drag-and-drop.
- Use dnd-kit only for freeform or spatial interactions such as canvas placement
  and custom drop geometry.
- Do not ship raw browser, React Aria, Tailwind example, or third-party styling.
- Do not import React Aria directly from feature code.
- Do not add arbitrary color literals, shadows, radii, or spacing values in feature
  code.
- Avoid static inline-style objects for ordinary layout and visual styling. Inline
  styles are reserved for runtime-computed geometry, transforms, and measurements.
- Preserve visible focus, keyboard alternatives, semantic labels, contrast,
  reduced motion, and non-color status indicators.
- Give loading, empty, disabled, disconnected, degraded, error, recovery, and
  success states the same visual care as the main state.
- Keep canvas and operator content visually dominant over diagnostics and chrome.

## Skill Policy

Canonical Mirae documentation and this file always override external skills.

Use skills according to `AGENT_SKILLS.md`:

- `mirae-ui-engineering` for every visible UI change;
- `emil-design-eng` when interaction or motion is materially involved;
- `review-animations` after non-trivial motion work;
- `webapp-testing` for rendered browser-level checks and screenshots;
- `frontend-design-review` before completing a significant visible surface;
- the official code-review workflow before completing a non-trivial PR;
- the official security workflow for bridge, custom protocol, filesystem,
  extension, external-input, credential, or process-boundary changes.

External skills are advisory. They must not:

- change Mirae's canonical visual direction;
- replace semantic tokens;
- select, install, or replace a UI primitive library;
- add an unapproved dependency;
- bypass `@mirae/ui-kit`;
- override architecture documents;
- invent product state;
- turn the desktop operator interface into a generic SaaS dashboard.

## Visual Validation

A user-visible ticket is not complete after unit tests alone.

The agent must:

1. run the real control UI or native shell;
2. inspect the result at the canonical `1440 x 900` logical viewport;
3. inspect at least one narrower desktop size;
4. exercise relevant loading, empty, disconnected, degraded, success, and error
   states;
5. verify hierarchy, density, spacing, clipping, overflow, focus, keyboard
   behavior, contrast, disabled states, and reduced motion;
6. inspect browser console output;
7. capture or inspect screenshots when available;
8. compare the result with the canonical visual direction;
9. test the real Wry/WebView2 shell when native behavior matters;
10. report any remaining visual limitation honestly.

Never claim that a user-visible ticket is visually complete without observing the
rendered interface.

## Validation

Run the repository's canonical commands, expected to be exposed through
`cargo xtask`.

Minimum:

```text
cargo xtask generate --check
cargo xtask fmt --check
cargo xtask lint
cargo xtask test-affected
cargo xtask docs --check
```

Run broader tests when changing foundation, contracts, persistence, IPC,
rendering, audio, security, updater, SDK behavior, UI primitives, or the native
shell.

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

Report in French:

- ticket;
- files changed;
- behavior added;
- tests added;
- commands run and results;
- docs and ADRs changed;
- assumptions;
- remaining risks;
- follow-up tickets;
- for visible work: rendered sizes, states inspected, keyboard/accessibility
  checks, native-shell result, and visual debt;
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

# 814 — Bootstrap Ticket Backlog

**Status:** Proposed  
**Audience:** Project owner and coding agents  
**Canonical:** Yes  
**Required context:** `813-implementation-roadmap.md`

---

## Sprint 0 — Repository Foundation

### MIR-0001 — Initialize monorepo

**Goal:** Create the Cargo and pnpm workspaces with the target root layout.

**Acceptance criteria:**

- root workspaces exist;
- placeholder apps and packages compile;
- no circular dependency;
- one root README explains bootstrap;
- CI can discover all workspaces.

### MIR-0002 — Pin toolchains

**Goal:** Pin Rust, Node, and pnpm.

**Acceptance criteria:**

- version files committed;
- wrong versions fail early;
- CI uses the same versions;
- bootstrap output is actionable.

### MIR-0003 — Add `xtask`

**Goal:** Add one repository automation entry point.

**Acceptance criteria:**

- `cargo xtask bootstrap`;
- `cargo xtask generate`;
- `cargo xtask check`;
- `cargo xtask test`;
- commands have help output.

### MIR-0004 — Add repository policy checks

**Goal:** Enforce formatting, linting, secret scan, dependency direction, and generated drift.

### MIR-0005 — Create canonical schema skeleton

**Goal:** Create versioned schema directories and a deterministic no-op generation pipeline.

### MIR-0006 — Generate Rust and TypeScript handshake contracts

**Goal:** Define engine/shell protocol version and readiness messages.

### MIR-0007 — Create structured error foundation

**Goal:** Implement error code/category/severity/retryability base types.

### MIR-0008 — Create structured tracing foundation

**Goal:** Add engine session, process role, correlation ID, redaction-safe fields, and rolling local logs.

### MIR-0009 — Build engine process skeleton

**Goal:** Launch, initialize services, expose readiness, and shut down cleanly.

### MIR-0010 — Build native shell skeleton

**Goal:** Launch engine, authenticate, supervise it, and host a local placeholder UI.

### MIR-0011 — Build React control UI skeleton

**Goal:** Show engine connection, version, process state, and reconnect behavior.

### MIR-0012 — Add authenticated IPC handshake

**Goal:** Use ephemeral launch credentials and protocol negotiation.

### MIR-0013 — Add engine crash/restart smoke test

**Goal:** Prove the shell detects engine failure and can perform bounded restart.

### MIR-0014 — Add documentation link validator

**Goal:** Validate SUMMARY links, required headers, duplicate document IDs, and ADR references.

### MIR-0015 — Add first integration test harness

**Goal:** Launch shell/engine test processes with deterministic transport and assert readiness/shutdown.

### MIR-0016 — Host the control UI in a system webview

**Goal:** Create the main control window and host locally packaged control-UI resources in the operating system's own webview.

**Acceptance criteria:**

- window created by the shell that already supervises the engine;
- resources served over a custom protocol from the package;
- navigation policy, permission restrictions, and content security policy enforced;
- webview failure distinguishable from engine failure.

---

## Sprint 1 — Project Kernel

Phase 1 of `813-implementation-roadmap.md`. Exit condition: an empty project can
be created, saved, reopened, and recovered.

The sprint has three strands. The kernel strand (`MIR-0101` to `MIR-0106`) makes
domain state authoritative and observable. The persistence strand (`MIR-0107` to
`MIR-0112`) gives that state a file and a recovery story. The visible strand
(`MIR-0116`, `MIR-0113`) carries it to the window the shell now owns. The two
verification tickets (`MIR-0114`, `MIR-0115`) prove the parts that only fail
under conditions ordinary tests do not create.

### MIR-0101 — Implement typed IDs and generations

**Goal:** Add the identifier and generation newtypes every later ticket depends on (`005-domain-model.md` section 2, `106-state-store.md` section 13).

**Acceptance criteria:**

- `EntityId` and its per-entity newtypes cannot be assigned across entity kinds;
- IDs are stable across serialization and independent of position;
- `StateGeneration` increments monotonically and compares as a total order;
- capability generation is a distinct type from state generation;
- the identifier crate clears `DEPENDENCY_VERSIONS.md` section 11 or is implemented without one.

### MIR-0102 — Implement state-store snapshot

**Goal:** Own authoritative project state behind immutable, generation-stamped snapshots (`106-state-store.md` sections 3, 6, 7).

**Acceptance criteria:**

- readers receive an immutable snapshot and never a mutable reference;
- a snapshot names its engine session, project ID, generation, and projection version;
- derived indexes are rebuilt from canonical collections and validated against them;
- snapshot retention is bounded;
- no runtime handle can be placed in the store.

### MIR-0103 — Implement command envelope

**Goal:** Make commands the only way to request mutation (`104-command-system.md` sections 3 to 6).

**Acceptance criteria:**

- typed envelope with command ID, session, actor, optional expected generation and idempotency key;
- handlers registered by payload type, never by arbitrary string;
- validation stages run in the documented order and stop before commit on failure;
- acknowledgement distinguishes accepted, rejected, conflict, failed, and cancelled;
- `Accepted` is issued only past the commit point.

### MIR-0104 — Implement transaction commit

**Goal:** Commit related changes atomically and increment the generation exactly once (`107-transactions.md`).

**Acceptance criteria:**

- begin, read, validate, prepare, revalidate, commit, publish, in that order;
- pre-commit failure leaves generation, events, and undo records untouched;
- expected-generation mismatch produces a structured conflict, not an internal error;
- nested public transactions are rejected;
- no disk, network, device, or extension work happens inside the commit section.

### MIR-0105 — Implement event publication after commit

**Goal:** Publish semantic events and patches after commit, in one tested order (`105-event-system.md`, `107-transactions.md` section 9).

**Acceptance criteria:**

- no domain event is observable before commit;
- delivery failure after commit does not revert state;
- subscribers cannot mutate state;
- subscriptions and queues are bounded, with a defined drop or lag policy.

### MIR-0106 — Implement state snapshot and patch protocol

**Goal:** Let a client mirror engine state through a snapshot and ordered patches (`106-state-store.md` section 8, `109-ui-engine-synchronization.md` sections 4 and 5).

**Acceptance criteria:**

- a patch names its from-generation, to-generation, and projection schema version;
- a gap, a duplicate, or a session change forces resynchronization rather than a silent merge;
- snapshot and applied-patch results are equivalent, proven by test;
- contracts are generated from canonical schemas, not hand-written.

### MIR-0107 — Define project schema v1

**Goal:** Define the canonical serialized project envelope and its schema (`401-project-format.md`).

**Acceptance criteria:**

- envelope identifies format, schema version, project ID, timestamps, app versions, and integrity metadata;
- canonical serialization is deterministic and produces stable diffs;
- NaN, infinity, and out-of-bound values are rejected;
- secrets cannot be represented;
- unknown-field behavior is explicit;
- the file encoding is chosen by an ADR rather than assumed.

### MIR-0108 — Implement empty-project creation

**Goal:** Create a valid empty project through a command (`400-project-overview.md`, `104-command-system.md`).

**Acceptance criteria:**

- creation is a command and a transaction, not a constructor called from the UI;
- the result validates against the schema from `MIR-0107`;
- the new project has a stable ID and a first generation;
- creating a project while one is open follows a defined lifecycle rule.

### MIR-0109 — Implement atomic project save

**Goal:** Publish a project file atomically from an immutable snapshot (`403-persistence.md`).

**Acceptance criteria:**

- write to a temporary file, flush, then replace; never write in place;
- serialization never holds the commit lock and never reads mutable state;
- failure preserves the previous file;
- external modification is detected before overwrite;
- the result names the exact saved generation.

### MIR-0110 — Implement project open and validation

**Goal:** Open a project file, validate it, and report what is wrong without discarding user intent (`411-project-validation-and-repair.md`, `401-project-format.md` section 10).

**Acceptance criteria:**

- schema validation precedes semantic validation;
- an unsupported required feature is refused or opened read-only, never ignored;
- unresolved references are preserved with diagnostics rather than deleted;
- integrity mismatch is reported;
- a malformed file cannot panic the engine.

### MIR-0111 — Implement dirty/saved generation tracking

**Goal:** Track the difference between the committed generation and the saved generation (`403-persistence.md` section 11, `107-transactions.md` section 3.8).

**Acceptance criteria:**

- dirty state derives from generations, not from a boolean set by hand;
- a save acknowledgement names the generation it covers;
- a commit during a save leaves the project dirty afterwards;
- the state is projected to clients rather than recomputed by them.

### MIR-0112 — Implement recovery-store skeleton

**Goal:** Write bounded recovery records separate from the canonical project file (`404-autosave-and-recovery.md`).

**Acceptance criteria:**

- autosave never writes over the canonical project;
- only committed generations are recorded;
- retention is bounded by count, bytes, and age;
- records are integrity-checked and exclude secrets;
- an invalid record does not block opening the canonical project.

### MIR-0113 — Add create/open/save UI flow

**Goal:** Create, open, save, and see the project state in the control window (`09-ui-ux`, `109-ui-engine-synchronization.md`).

**Acceptance criteria:**

- the UI mirrors engine state and owns none of it;
- save state is announced accessibly and is not indicated by colour alone;
- a rejected command is shown against the control that caused it;
- disconnection disables project actions rather than faking them;
- keyboard and screen-reader paths are tested.

### MIR-0114 — Add interrupted-save fault test

**Goal:** Prove the save pipeline survives interruption at each stage (`403-persistence.md` section 13).

**Acceptance criteria:**

- interruption during the temporary write, before rename, and after rename each leave a complete file;
- a stale temporary file is cleaned and never mistaken for a project;
- disk-full and permission-denied paths report structured errors.

### MIR-0115 — Add project round-trip compatibility fixture

**Goal:** Freeze the schema behaviour in fixtures that a future change must confront (`615-compatibility-policy.md`, `401-project-format.md` section 15).

**Acceptance criteria:**

- a committed fixture opens, saves, and compares byte-identically;
- an older fixture opens through migration;
- an unknown optional field survives a round trip or produces a diagnostic;
- fixtures are generated, not hand-edited.

### MIR-0116 — Add the typed shell bridge

**Goal:** Carry commands, snapshots, and patches between the webview and the engine through the shell (`501-desktop-shell.md` sections 3 and 13, `109-ui-engine-synchronization.md`).

**Acceptance criteria:**

- the webview never reaches the engine socket directly;
- the bridge is typed from canonical contracts and rejects anything else;
- messages are bounded and validated before allocation;
- credentials never cross it;
- a bridge failure is distinguishable from an engine failure in the UI.

---

## Sprint 2 — Rendering Kernel

### MIR-0201 — Define semantic scene graph
### MIR-0202 — Add generated color source
### MIR-0203 — Implement frame compiler v1
### MIR-0204 — Implement render graph v1
### MIR-0205 — Initialize `wgpu` backend
### MIR-0206 — Add GPU resource generations
### MIR-0207 — Add preview surface
### MIR-0208 — Add frame scheduler v1
### MIR-0209 — Render one color source
### MIR-0210 — Add renderer diagnostics
### MIR-0211 — Add surface resize handling
### MIR-0212 — Add simulated device-loss recovery
### MIR-0213 — Add rendering benchmark baseline
### MIR-0214 — Add preview UI connection
### MIR-0215 — Add scene persistence round trip

---

## Ticket Selection Rule

The first ready unblocked ticket in the active sprint is the next ticket.

A coding agent must not skip ahead to a more interesting ticket without recording why.

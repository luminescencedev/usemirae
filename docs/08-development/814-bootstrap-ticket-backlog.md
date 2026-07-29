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

---

## Sprint 1 — Project Kernel

### MIR-0101 — Implement typed IDs and generations
### MIR-0102 — Implement state-store snapshot
### MIR-0103 — Implement command envelope
### MIR-0104 — Implement transaction commit
### MIR-0105 — Implement event publication after commit
### MIR-0106 — Implement state snapshot and patch protocol
### MIR-0107 — Define project schema v1
### MIR-0108 — Implement empty-project creation
### MIR-0109 — Implement atomic project save
### MIR-0110 — Implement project open and validation
### MIR-0111 — Implement dirty/saved generation tracking
### MIR-0112 — Implement recovery-store skeleton
### MIR-0113 — Add create/open/save UI flow
### MIR-0114 — Add interrupted-save fault test
### MIR-0115 — Add project round-trip compatibility fixture

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

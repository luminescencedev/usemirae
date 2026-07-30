# Sprint 0 — Repository Foundation

**State:** complete  
**Canonical scope:** `docs/08-development/814-bootstrap-ticket-backlog.md`, Sprint 0  
**Roadmap phase:** `813-implementation-roadmap.md` section 2

**Exit condition:** all processes launch and tests run from one command. Met.

## Tickets

- [x] MIR-0001 — Initialize monorepo
- [x] MIR-0002 — Pin toolchains
- [x] MIR-0003 — Add `xtask`
- [x] MIR-0004 — Add repository policy checks
- [x] MIR-0005 — Create canonical schema skeleton
- [x] MIR-0006 — Generate Rust and TypeScript handshake contracts
- [x] MIR-0007 — Create structured error foundation
- [x] MIR-0008 — Create structured tracing foundation
- [x] MIR-0009 — Build engine process skeleton
- [x] MIR-0010 — Build native shell skeleton
- [x] MIR-0011 — Build React control UI skeleton
- [x] MIR-0012 — Add authenticated IPC handshake
- [x] MIR-0013 — Add engine crash/restart smoke test
- [x] MIR-0014 — Add documentation link validator
- [x] MIR-0015 — Add first integration test harness
- [x] MIR-0016 — Host the control UI in a system webview

## Decisions taken during the sprint

- [x] ADR-0067 — JSON control-plane serialization
- [x] ADR-0068 — System webview for the desktop shell

## Left open

These are real and unblocked. None is on the Sprint 1 critical path.

- [ ] MIR-DEPS-0001 — Resolve the ESLint 10 peer declaration
- [ ] MIR-TOOLING-0001 — Move CI actions off the deprecated Node 20 runtime
- [ ] MIR-TOOLING-0002 — Implement affected-set detection

Per-ticket detail — what shipped, what was validated, what was deferred — is in
`BOOTSTRAP_TICKETS.md`.

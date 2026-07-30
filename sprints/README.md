# Sprints

One file per sprint, one checkbox per ticket. Tick a box when the ticket is
committed on `main` and `cargo xtask check` passes.

| Sprint | File | State |
|---|---|---|
| Sprint 0 — Repository Foundation | [sprint-0.md](sprint-0.md) | complete |
| Sprint 1 — Project Kernel | [sprint-1.md](sprint-1.md) | complete |

## How these files relate to the rest

- `docs/08-development/814-bootstrap-ticket-backlog.md` is canonical. It owns
  each ticket's goal and acceptance criteria. If these files and that document
  disagree, that document is right.
- `BOOTSTRAP_TICKETS.md` is the record: what shipped, what was validated, what
  was left. It is written after a ticket, in prose, because a checkbox cannot
  say why something was done a particular way.
- These files are the board. They answer "what is done and what is next" at a
  glance and nothing else.

## Conventions

```text
- [ ] MIR-0000 — Title        not started
- [~] MIR-0000 — Title        in progress
- [x] MIR-0000 — Title        committed and validated
```

A ticket is ticked only when `cargo xtask check` passes on the commit that
contains it. "It compiles" is not the bar (`816-definition-of-done.md`).

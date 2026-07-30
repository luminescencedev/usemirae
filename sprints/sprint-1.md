# Sprint 1 — Project Kernel

**State:** in progress  
**Canonical scope:** `docs/08-development/814-bootstrap-ticket-backlog.md`, Sprint 1  
**Roadmap phase:** `813-implementation-roadmap.md` section 3

**Exit condition:** an empty project can be created, saved, reopened, and
recovered — and can be driven from the control window rather than from a test.

## Decisions that gate tickets

Each was left to an implementation ADR by the canonical documents. Taking one
quietly inside a feature ticket is the failure this list prevents.

- [x] ADR-0069 — Random UUIDs as the persisted entity identifier — gated MIR-0101
- [x] ADR-0070 — Arc snapshots with per-entity sharing — gated MIR-0102
- [x] ADR-0071 — JSON as the canonical project file encoding — gated MIR-0107

## Strand 1 — Kernel

Identity, authoritative state, commands, transactions, events, and the
snapshot/patch protocol a client mirrors. Testable without a window or a file.

- [x] MIR-0101 — Implement typed IDs and generations
- [x] MIR-0102 — Implement state-store snapshot
- [x] MIR-0103 — Implement command envelope
- [x] MIR-0104 — Implement transaction commit
- [x] MIR-0105 — Implement event publication after commit
- [x] MIR-0106 — Implement state snapshot and patch protocol

## Strand 2 — Persistence

The schema, creation, atomic save, open and validation, dirty tracking, and a
bounded recovery store.

- [x] MIR-0107 — Define project schema v1
- [x] MIR-0108 — Implement empty-project creation
- [x] MIR-0109 — Implement atomic project save
- [x] MIR-0110 — Implement project open and validation
- [ ] MIR-0111 — Implement dirty/saved generation tracking
- [ ] MIR-0112 — Implement recovery-store skeleton

## Strand 3 — Visible

The bridge, then the flow in the window MIR-0016 built. The control UI reports
"Not connected" until MIR-0116 exists.

- [ ] MIR-0116 — Add the typed shell bridge
- [ ] MIR-0113 — Add create/open/save UI flow

## Verification

What ordinary tests do not reach.

- [ ] MIR-0114 — Add interrupted-save fault test
- [ ] MIR-0115 — Add project round-trip compatibility fixture

## Order

```text
MIR-0101 ──┬── MIR-0102 ──┬── MIR-0104 ──┬── MIR-0105
           ├── MIR-0103 ──┘              └── MIR-0106 ── MIR-0116 ── MIR-0113
           └── MIR-0107 ── MIR-0108 ── MIR-0109 ──┬── MIR-0110 ── MIR-0115
                                                  ├── MIR-0111
                                                  ├── MIR-0112
                                                  └── MIR-0114
```

MIR-0107 needs only MIR-0101, so the schema strand can run beside the kernel.
That is the one genuine parallel branch in this sprint.

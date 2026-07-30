# ADR-0070 — Arc Snapshots With Per-Entity Sharing, Not a Persistent-Collection Crate

**Status:** Accepted  
**Date:** 2026-07-30  
**Related:** ADR-0009 (generation-stamped state), ADR-0010 (commands and transactions)

---

## Context

`01-runtime/106-state-store.md` section 5 asks the implementation to use
immutable snapshots or structural sharing where it improves lock duration,
reader safety, snapshot generation, UI projection, or undo, and states plainly
that deep-cloning the entire project for every small command is not an acceptable
permanent architecture without evidence that budgets are met. Section 13 requires
that readers never mutate committed state and that commit is atomic.

`MIR-0102` builds the store, so the representation has to be settled first. The
question is what a snapshot physically is, and what a commit costs.

`06-quality/601-performance-budgets.md` gives the outer bound: command
acknowledgement at the 95th percentile is 100 ms for a local non-I/O mutation.
The commit section is one part of that path and has to be a small part, because
`107-transactions.md` section 12 forbids anything slow inside it.

---

## Decision

A snapshot is an `Arc<ProjectState>`. Inside it, every entity collection is a
`BTreeMap` from identifier to `Arc<Entity>`.

A commit clones the `ProjectState` — which clones the maps' spines and the
`Arc` pointers, not the entities — replaces the entries the transaction touched,
and atomically installs the result behind a new generation.

No persistent-collection crate is introduced.

`BTreeMap` rather than `HashMap`, because `401-project-format.md` section 12
requires deterministic canonical serialization, and iteration ordered by
identifier gives that for free rather than through a sort at save time.

---

## Consequences

### Positive

- **Entities are shared, not copied.** A command that renames one scene copies
  `N` pointers and one scene, not `N` scenes. This is the structural sharing
  section 5 asks for, at the granularity where the sharing actually pays.
- **Readers need no lock.** A reader holds an `Arc` to a state that can no longer
  change. `106` invariant 2 holds because there is no path to a `&mut` — not
  because callers are careful.
- **The commit section is a pointer swap.** Everything expensive — building
  candidate entities, validating, computing patches — happens before it, which is
  exactly the shape `107-transactions.md` section 3 describes.
- **Undo is cheap to represent.** A prior snapshot is an `Arc` clone, so keeping
  one costs a pointer plus whatever the two versions do not share.
- **No dependency, and no lock-in.** The upgrade path stays open: if a benchmark
  ever shows the map spine dominating, the map type changes behind the store's
  API and nothing above it moves.

### Negative

- **Commit is O(entities), not O(changed).** Cloning the spine touches every key
  in the changed collection. This is the cost being accepted, and it is why this
  ADR carries a measurement rather than an assurance: at 10,000 entities the
  clone is on the order of tens of microseconds, against a 100 ms budget for the
  whole acknowledgement path. `MIR-0102` lands that benchmark as a test, so the
  claim fails loudly if it stops being true.
- **Two levels of indirection to read an entity.** `Arc<ProjectState>` then
  `Arc<Entity>`. Irrelevant at the rate domain state is read; it would matter if
  the renderer read domain state per frame, which `106` section 10 already
  forbids by keeping runtime handles out of the store.
- **Memory is retained while a snapshot is held.** A consumer holding an old
  snapshot keeps the entities only that snapshot references. `106` section 12
  requires retention to be bounded, so the store bounds how many it keeps; a
  consumer that holds one for a long time pays for it, visibly.

---

## Alternatives Considered

### A persistent-collection crate (`im`, `imbl`, `rpds`)

Rejected for now. It would make commit O(changed) rather than O(entities), which
is the right asymptotic answer. It is also a dependency with its own `unsafe`,
its own iteration-order semantics to reconcile with canonical serialization, and
a constant factor that loses to a plain `BTreeMap` at the sizes a project
actually reaches. The measurement in `MIR-0102` is the trigger: if the spine
clone approaches the budget, this decision is revisited with numbers instead of
taste.

### Deep clone of the whole project per command

Rejected, and rejected by `106` section 5 in as many words. It also copies every
entity into every undo record, which turns a rename of one scene into a copy of
the entire project.

### Mutable state behind a lock, with copies taken for readers

Rejected. It makes every reader either block or hold a stale copy taken at an
unknown moment, and it puts a `&mut ProjectState` into existence, which `106`
section 15 forbids outside the transaction layer. Snapshot semantics disappear:
there is nothing to attach a generation to.

### Event sourcing — store the command log, fold on read

Rejected. Reads become O(history), snapshot cost moves to every reader, and
`106`'s model is explicitly a committed authoritative state with generations, not
a derived projection. Undo already gets its own representation in
`107-transactions.md` section 8.

---

## Related Specifications

- `01-runtime/106-state-store.md` sections 3, 5, 7, 9 and 12
- `01-runtime/107-transactions.md` sections 3 and 12
- `04-project/401-project-format.md` section 12
- `06-quality/601-performance-budgets.md`

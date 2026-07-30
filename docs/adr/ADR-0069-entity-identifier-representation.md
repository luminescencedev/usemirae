# ADR-0069 — Random UUIDs as the Persisted Entity Identifier

**Status:** Accepted  
**Date:** 2026-07-30  
**Related:** ADR-0023 (project format), ADR-0067 (control-plane serialization)

---

## Context

`00-foundations/005-domain-model.md` section 2 requires stable opaque identifiers
for every persisted entity, recommends a UUID, and forbids identifiers that
encode array position, process identity, or memory location. `04-project/401-project-format.md`
section 5 repeats the requirement from the file's side: identifiers survive
reorder and save, and are unique within their namespace.

`MIR-0101` implements those types, so the representation has to be settled first.
The question is not whether identifiers are opaque — that is decided — but what
sits inside the newtype and where the bytes come from.

---

## Decision

An `EntityId` wraps a 128-bit UUID, generated as **version 4 (random)**, through
the `uuid` crate.

Per-entity newtypes wrap `EntityId` rather than the UUID directly, so a
`SceneId` cannot be passed where a `SourceId` is expected.

Serialization uses the canonical hyphenated lowercase text form, because a
project file is meant to be reviewed and diffed by a human
(`401-project-format.md` section 2).

---

## Consequences

### Positive

- **Position independence is structural.** A random identifier cannot encode an
  index, a path, or a pointer, so `005` invariants 1 and 3 hold by construction
  rather than by review.
- **No coordination.** Identifiers are minted anywhere — engine, recovery
  process, a future extension — with no central allocator and no collision
  protocol. This matters more than it looks: `409-project-locking-and-multi-instance.md`
  allows more than one process near the same project.
- **A well-tested implementation.** UUID parsing, formatting, and validation are
  exactly the kind of code that looks trivial and is not. Hand-rolling it to
  avoid one dependency would trade a reviewed crate for unreviewed code on the
  persistence path.
- **The text form is stable.** Every language, database, and tool reads it,
  which matters for fixtures, diagnostics, and the compatibility corpus.

### Negative

- **16 bytes and no locality.** Random identifiers scatter in any index built on
  them. For a project holding thousands of entities in memory this is
  irrelevant; if it ever stops being irrelevant, indexes are derived
  (`106-state-store.md` section 6) and can be keyed differently without touching
  the identifier.
- **Not sortable.** Version 7 identifiers would sort by creation time, which is
  occasionally convenient for debugging. It is also a timestamp leak in a file
  users share, and `005` section 2 asks identifiers not to expose more than
  identity. Ordering that matters is explicit ordering, held by the scene item
  list, not implied by identity.
- **A new dependency.** `uuid` and its randomness source must clear
  `DEPENDENCY_VERSIONS.md` section 11 with exact pins and a licence and security
  review, which `MIR-0101` does.

---

## Alternatives Considered

### A hand-rolled 128-bit identifier

Rejected. It is the same 16 bytes with the same randomness requirement, minus
the canonical text form, minus the parsing edge cases, and minus anyone else's
tests. The dependency being avoided is small and permissively licensed.

### UUID version 7, time-ordered

Rejected for persisted entities. It embeds a creation timestamp in every
identifier in a file that users send to each other, and the sortability buys
nothing the domain does not already model explicitly. Nothing here prevents a
future runtime-only identifier from using version 7 where locality is measured
to matter; `005` section 2 already allows runtime instances their own
generation-scoped identifiers.

### A monotonic counter per project

Rejected. It requires a single allocator, breaks the moment two processes touch
one project, and produces identifiers that leak creation order and entity count.

### A content hash of the entity

Rejected. Identity would change when the entity changes, which contradicts
"stable across saves" and would make a rename look like a delete and a create.

---

## Related Specifications

- `00-foundations/005-domain-model.md` sections 2 and 10
- `04-project/401-project-format.md` section 5
- `01-runtime/106-state-store.md` section 13
- `DEPENDENCY_VERSIONS.md` section 11

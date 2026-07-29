# 405 — Command History and Undo/Redo

**Status:** Proposed  
**Audience:** Runtime, project, UI contributors  
**Canonical:** Yes  
**Required context:** `01-runtime/104-command-system.md`, `01-runtime/107-transactions.md`  
**Related ADRs:** ADR-0026

---

## 1. Purpose

Undo and redo reverse committed project-domain changes through the same command and transaction architecture used for ordinary mutation.

---

## 2. History Scope

Command history is scoped to:

- active project;
- engine session;
- project generation lineage;
- actor policy.

History is runtime/session state by default, not part of the portable project schema.

---

## 3. Undo Record

Conceptual record:

```rust
pub struct UndoRecord {
    pub id: UndoRecordId,
    pub command_id: CommandId,
    pub committed_generation: StateGeneration,
    pub label: UndoLabel,
    pub affected_entities: Vec<EntityId>,
    pub inverse: UndoOperation,
    pub redo: RedoOperation,
    pub merge_key: Option<MergeKey>,
    pub validity: UndoValidity,
}
```

---

## 4. Undo Categories

Commands declare:

- fully undoable;
- mergeable;
- conditionally undoable;
- non-undoable runtime action;
- externally irreversible;
- destructive but restorable through snapshot.

The UI must not promise undo for non-undoable operations.

---

## 5. Execution

Undo is a command.

Flow:

1. validate top undo record;
2. verify project and entity validity;
3. execute inverse through transaction;
4. commit new generation;
5. move record to redo stack;
6. publish events and patches.

Redo follows the same model.

---

## 6. Coalescing

Continuous edits may coalesce:

- transform drag;
- volume/fader adjustment;
- text typing;
- color slider;
- reorder sequence.

Coalescing requires:

- same merge key;
- compatible command type;
- bounded time or interaction scope;
- same project lineage;
- no intervening conflicting command.

---

## 7. External Changes

Undo validity may be invalidated by:

- project reload;
- migration;
- external file replacement;
- entity deletion by later command;
- extension removal;
- command history truncation;
- engine restart if history is not persisted.

Invalid records are marked and removed or skipped with diagnostics according to policy.

---

## 8. Memory Bounds

History is bounded by:

- record count;
- retained byte estimate;
- snapshot size;
- age or generation span.

Large operations may store compressed prior state or specialized inverse operations.

Eviction removes oldest undo records and corresponding unreachable redo chain.

---

## 9. Save Interaction

Explicit save does not clear undo history automatically.

Save marks a saved generation.

The UI may show:

- current state saved;
- undo available beyond saved state;
- undo moved project before saved generation;
- redo can return toward saved generation.

---

## 10. Runtime Operations

Operations such as:

- start stream;
- stop stream;
- reconnect device;
- capture screenshot;
- grant permission;

are not ordinary project undo entries.

Their reversal, if available, is a separate explicit command.

---

## 11. Extension Commands

Extension commands may participate only if extension declares:

- deterministic inverse or snapshot;
- schema version;
- affected state;
- compatibility behavior when extension absent.

Otherwise they are marked non-undoable and require user-visible warning when destructive.

---

## 12. Invariants

1. Undo creates a new committed generation.
2. Undo never mutates state outside transaction.
3. Non-undoable operations are explicit.
4. History is bounded.
5. Coalescing is deterministic.
6. Save does not silently clear history.
7. Redo is invalidated by incompatible new mutation.
8. Project reload/migration validates history.
9. Extension undo is schema-aware.
10. Undo labels are safe for UI and logs.

---

## 13. Required Tests

- basic undo/redo;
- multi-entity transaction;
- coalesced transform;
- new command clears redo;
- save then undo;
- bounded history;
- invalid entity;
- project reload;
- non-undoable runtime action;
- extension command absent;
- migration invalidation;
- deterministic redo.

---

## 14. AI Implementation Notes

Do not implement undo by directly restoring random mutable fields outside commands.

Do not place start/stop stream in ordinary project history.

Do not keep unlimited full project snapshots.

Make every undo record traceable to the committed transaction it reverses.

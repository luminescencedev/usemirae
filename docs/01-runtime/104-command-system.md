# 104 — Command System

**Status:** Proposed  
**Audience:** Runtime, domain, UI, SDK contributors  
**Canonical:** Yes  
**Required context:** `005-domain-model.md`, `100-runtime-overview.md`  
**Related ADRs:** ADR-0007, ADR-0010

---

## 1. Purpose

Commands are the only supported mechanism for requesting authoritative domain mutation or controlled runtime operations.

A command expresses intent. It is not an event and not a direct function call exposed across boundaries.

---

## 2. Command Categories

### 2.1 Domain mutation commands

Examples:

- create scene;
- rename source;
- update transform;
- reorder scene items;
- change output profile;
- configure audio routing.

These participate in transactions and usually increment state generation.

### 2.2 Runtime operation commands

Examples:

- start output;
- stop output;
- reconnect source;
- capture screenshot;
- rescan devices.

They may change runtime state without changing persisted project state.

### 2.3 Lifecycle commands

Examples:

- open project;
- close project;
- request shutdown;
- restart extension host.

### 2.4 Query requests

Queries SHOULD use dedicated request/response contracts rather than pretending to be mutations.

---

## 3. Command Envelope

Conceptual envelope:

```rust
pub struct CommandEnvelope<T> {
    pub command_id: CommandId,
    pub engine_session_id: EngineSessionId,
    pub actor: ActorContext,
    pub expected_generation: Option<StateGeneration>,
    pub idempotency_key: Option<IdempotencyKey>,
    pub issued_at: Option<ClientTimestamp>,
    pub payload: T,
}
```

Client timestamps are informational and not authoritative for ordering.

---

## 4. Actor Context

Actor context identifies:

- local UI;
- extension;
- automation;
- remote controller;
- recovery process;
- internal subsystem.

It includes capabilities and audit identity where relevant.

The runtime validates actor permissions before domain execution.

---

## 5. Validation Stages

1. envelope validation;
2. protocol and session validation;
3. actor capability validation;
4. lifecycle-state validation;
5. schema validation;
6. domain precondition validation;
7. expected-generation conflict check;
8. transaction execution;
9. postcondition validation;
10. commit.

Failure at any stage before commit produces no authoritative state mutation.

---

## 6. Acknowledgement

```rust
pub enum CommandStatus {
    Accepted,
    Rejected,
    Conflict,
    Failed,
    Cancelled,
}
```

Acknowledgement includes:

- command ID;
- status;
- engine session;
- resulting state generation if committed;
- structured error or conflict;
- optional operation ID for longer asynchronous work.

`Accepted` means the command passed the defined commit point. It must not mean merely “received.”

For asynchronous operations, the command may commit the creation of an operation and later emit operation-state events.

---

## 7. Optimistic Concurrency

Commands that depend on a known state SHOULD include `expected_generation`.

On mismatch, the runtime returns `Conflict` with:

- current generation;
- conflict category;
- affected entity IDs when safe;
- whether automatic retry is allowed.

Blind retry is prohibited for non-idempotent user actions.

---

## 8. Idempotency

Commands that may be retried across IPC interruption should support idempotency keys.

The runtime maintains a bounded cache of recent results scoped to:

- engine session;
- actor;
- command kind;
- idempotency key.

The cache has explicit expiry and memory bounds.

---

## 9. Undoability

A command declares one of:

- fully undoable;
- conditionally undoable;
- non-undoable runtime operation;
- irreversible external operation.

Undo representation may be:

- inverse command;
- transaction delta;
- prior value snapshot;
- specialized undo record.

External operations such as starting a stream are not treated as ordinary project undo.

---

## 10. Command Registration

Command handlers are registered by type, not stringly typed arbitrary names.

A handler declares:

- command payload type;
- actor capabilities;
- allowed lifecycle states;
- transaction scope;
- undo policy;
- result type;
- diagnostics category.

Extensions access only commands exposed through the SDK capability layer.

---

## 11. Ordering

Commands entering the same authoritative state domain are serialized in accepted order.

Independent runtime operations may execute concurrently if they do not violate domain or lifecycle invariants.

Ordering across actors is determined by engine receipt and transaction serialization, not client timestamps.

---

## 12. Long-Running Operations

A command should not hold the state transaction open during slow external work.

Pattern:

1. validate and commit operation intent;
2. return operation ID;
3. perform external work;
4. publish progress;
5. commit final runtime or domain result;
6. publish completion.

If the external result changes persisted intent, the final commit is a separate transaction.

---

## 13. Error Model

Command errors are structured:

- invalid argument;
- unsupported capability;
- permission denied;
- wrong lifecycle state;
- state conflict;
- entity not found;
- unavailable external resource;
- operation timeout;
- compatibility error;
- internal failure.

Error payloads must be safe for logs and UI.

---

## 14. Invariants

1. Domain state changes only through commands and transactions.
2. Accepted mutation commands identify resulting generation.
3. Rejected commands do not partially mutate state.
4. Actor permissions are checked before execution.
5. Client timestamps do not determine authoritative ordering.
6. Command IDs are unique within required scope.
7. Idempotency storage is bounded.
8. Long-running external work does not hold state locks.
9. Extension commands are capability-scoped.
10. Errors are structured and redacted.

---

## 15. Required Tests

- command schema rejection;
- permission denial;
- lifecycle-state denial;
- generation conflict;
- transaction rollback;
- idempotent retry;
- duplicate command ID;
- asynchronous operation;
- undo metadata;
- extension capability enforcement;
- error redaction;
- command ordering under concurrency.

---

## 16. AI Implementation Notes

Do not expose a generic JSON “command name + arbitrary payload” endpoint.

Do not acknowledge success before the defined commit point.

Do not keep a mutex guard across network, file, device, or encoder work.

Do not mutate state from event subscribers.

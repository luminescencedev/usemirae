# 105 — Event System

**Status:** Proposed  
**Audience:** Runtime, domain, UI, diagnostics, SDK contributors  
**Canonical:** Yes  
**Required context:** `104-command-system.md`  
**Related ADRs:** ADR-0007, ADR-0008, ADR-0009

---

## 1. Purpose

Events communicate committed changes and runtime observations.

They allow UI projections, diagnostics, extensions, and services to react without gaining mutation authority.

---

## 2. Event Classes

### 2.1 Domain events

Emitted after a successful domain transaction.

Examples:

- scene created;
- scene item updated;
- source configuration changed;
- output profile removed;
- project metadata changed.

Domain events include state generation.

### 2.2 Runtime events

Describe non-persisted runtime changes.

Examples:

- source became unavailable;
- output entered reconnecting;
- encoder initialized;
- extension host restarted;
- renderer device recovered.

### 2.3 Diagnostic events

Describe health, errors, warnings, and performance observations.

### 2.4 Operation events

Describe long-running operation progress and completion.

---

## 3. Event Envelope

```rust
pub struct EventEnvelope<T> {
    pub event_id: EventId,
    pub engine_session_id: EngineSessionId,
    pub sequence: EventSequence,
    pub state_generation: Option<StateGeneration>,
    pub emitted_at: MonotonicInstant,
    pub correlation_id: Option<CorrelationId>,
    pub payload: T,
}
```

`sequence` is monotonic within one engine event stream.

---

## 4. Commit Ordering

Domain events are published after transaction commit and in commit order.

A subscriber must never observe an event for uncommitted state.

If publication to an external replica fails after commit, the replica recovers using sequence detection and snapshot resynchronization.

---

## 5. Delivery Semantics

Internal delivery is at-least-once or exactly-once only where explicitly implemented. Consumers must not assume global exactly-once delivery.

External UI replication uses:

- ordered sequence;
- generation;
- gap detection;
- snapshot recovery.

Duplicate events may occur after reconnect or retry and must be safely handled by event ID or generation.

---

## 6. Subscription

Subscribers declare:

- event classes;
- optional entity or subsystem filters;
- delivery priority;
- queue capacity;
- overflow policy;
- whether replay from recent buffer is supported.

There is no unrestricted global subscriber with unbounded queue.

---

## 7. Overflow

Possible overflow policies:

- disconnect and require snapshot;
- coalesce latest state notification;
- drop low-priority diagnostics with counters;
- stop extension subscription;
- preserve critical lifecycle events by dedicated channel.

Domain state replication must not silently drop required patches and continue as if synchronized.

---

## 8. Event Versus State Patch

A domain event communicates meaning.

A state patch communicates replica transformation.

They may originate from the same commit but serve different consumers.

Example:

```text
Event: SceneItemTransformChanged(scene_item_id)
Patch: replace /scenes/.../transform with ...
```

The UI may use both, but state synchronization must not depend on reconstructing all state from semantic events unless explicitly specified.

---

## 9. Correlation

Correlation IDs connect:

- command;
- transaction;
- emitted events;
- long-running operation;
- diagnostics;
- logs.

Correlation does not replace unique event or command IDs.

---

## 10. Retention

The runtime may retain bounded recent events for:

- diagnostics;
- reconnect optimization;
- debugging;
- test inspection.

Retention must define:

- maximum count or bytes;
- event classes retained;
- redaction;
- expiration.

Persistent event sourcing is not currently the canonical project storage model.

---

## 11. Extension Events

Extensions receive only event types allowed by:

- API version;
- manifest declarations;
- capability grants;
- project policy.

Sensitive data and credentials are excluded.

High-frequency raw media events are not delivered through the general extension event system.

---

## 12. Invariants

1. Domain events follow commit.
2. Event sequence is monotonic per engine session.
3. Subscribers detect sequence gaps.
4. Queues are bounded.
5. Events do not grant mutation authority.
6. State patches and semantic events remain distinct.
7. Sensitive values are redacted.
8. High-frequency media data uses dedicated channels.
9. Event retention is bounded.
10. Duplicate delivery is safe.

---

## 13. Required Tests

- commit-before-event ordering;
- sequence monotonicity;
- gap detection;
- duplicate event handling;
- subscriber overflow;
- extension filtering;
- redaction;
- reconnect replay;
- snapshot fallback;
- correlation propagation;
- event ordering under concurrent commands.

---

## 14. AI Implementation Notes

Do not implement mutation by subscribing to an event and writing authoritative state through a hidden path.

Do not use a single unbounded broadcast channel for all events.

Do not make UI synchronization depend solely on semantic events.

Include session, sequence, and generation metadata.

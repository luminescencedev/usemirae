# 707 — Extension IPC Protocol

**Status:** Proposed  
**Audience:** SDK, IPC, runtime, security contributors  
**Canonical:** Yes  
**Required context:** `01-runtime/108-ipc-protocol.md`, `701-extension-architecture.md`, `704-sdk-api-surface.md`  
**Related ADRs:** ADR-0049

---

## 1. Purpose

The extension IPC protocol carries typed SDK control messages between extension hosts and the engine.

It is a restricted protocol layered on Mirae's process IPC infrastructure.

---

## 2. Connection Identity

Handshake includes:

- process role `ExtensionHost`;
- host instance ID;
- protocol version;
- SDK protocol versions;
- ephemeral host credential;
- assigned extension IDs;
- feature flags;
- message limits.

Per-extension identity is injected by host and verified by engine.

---

## 3. Message Families

- extension lifecycle;
- capability request/grant/revoke;
- API request/response;
- command invocation;
- query result;
- event subscription/delivery;
- operation progress;
- provider registration;
- UI contribution registration;
- storage/network/file broker;
- diagnostics;
- flow control;
- data-plane lease control.

---

## 4. Namespacing

Every message includes:

- extension ID;
- extension runtime instance ID;
- host generation;
- SDK version;
- correlation ID;
- capability token reference.

The extension process cannot select another extension namespace.

---

## 5. Bounds

Protocol limits:

- maximum frame size;
- maximum nested depth;
- maximum string/collection size;
- messages per second;
- outstanding requests;
- subscription count;
- retained event window;
- broker payload size;
- data-plane descriptors.

Large media data uses dedicated leases, not generic IPC payloads.

---

## 6. Flow Control

Priority:

1. revocation and shutdown;
2. lifecycle;
3. command results;
4. state synchronization;
5. critical provider events;
6. ordinary events;
7. diagnostics/logs;
8. low-priority metrics.

On overflow, low-priority traffic is dropped/coalesced first.

State/protocol gaps require resynchronization.

---

## 7. Compatibility

The host and engine negotiate:

- SDK major/minor;
- feature set;
- schema revisions;
- optional endpoints.

Major incompatibility prevents extension activation.

---

## 8. Security Validation

Each request validates:

- host authentication;
- extension assignment;
- runtime generation;
- capability;
- scope;
- payload schema;
- rate limit;
- lifecycle state.

Protocol violations increment abuse counters.

---

## 9. Invariants

1. Extension IPC is typed and versioned.
2. Host injects extension identity.
3. Every request validates capability.
4. Frames and rates are bounded.
5. Raw media does not use generic control messages.
6. Revocation/shutdown has priority.
7. Gaps trigger resynchronization.
8. Protocol abuse is attributable.
9. Major incompatibility fails closed.
10. Errors are structured and redacted.

---

## 10. Required Tests

- handshake;
- invalid host token;
- identity spoof;
- stale host generation;
- unsupported SDK major;
- oversized frame;
- request flood;
- capability denial;
- event gap;
- revocation priority;
- malformed payload fuzz;
- host disconnect.

---

## 11. AI Implementation Notes

Do not reuse the unrestricted engine IPC schema for extensions without capability filtering.

Do not let an extension choose its own identity field.

Do not serialize video/audio payloads through this channel.

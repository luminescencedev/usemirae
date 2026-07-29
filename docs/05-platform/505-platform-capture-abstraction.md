# 505 — Platform Capture Abstraction

**Status:** Proposed  
**Audience:** Platform, media, rendering contributors  
**Canonical:** Yes  
**Required context:** `03-media/302-capture-system.md`, `500-platform-overview.md`  
**Related ADRs:** ADR-0031, ADR-0032

---

## 1. Purpose

This document defines the cross-platform capture interfaces implemented by Windows, macOS, and Linux adapters.

---

## 2. Capture Provider

```rust
pub trait CaptureProvider: Send + Sync {
    fn provider_id(&self) -> CaptureProviderId;
    fn capabilities(&self) -> CaptureProviderCapabilities;
    fn enumerate(&self, request: CaptureEnumerationRequest)
        -> Result<CaptureEnumerationSnapshot>;
    fn create_session(&self, request: CaptureSessionRequest)
        -> Result<Box<dyn CaptureSession>>;
}
```

Exact syntax may differ. Semantics are mandatory.

---

## 3. Enumeration Snapshot

Snapshot includes:

- provider;
- generation;
- displays;
- windows/applications where allowed;
- cameras/devices;
- source-safe labels;
- thumbnails if permitted;
- permission state;
- limitations;
- stable/ephemeral identifiers.

Enumeration is bounded and cancellable.

---

## 4. Source Identity

Identity model distinguishes:

- persistent device identity;
- session-local window identity;
- user-selected portal token;
- display identity;
- fallback matching metadata.

The domain source definition stores a stable reference form appropriate to the source kind.

---

## 5. Session Request

Request includes:

- source reference;
- desired media types;
- format preferences;
- cursor policy;
- audio policy;
- latency mode;
- HDR preference;
- crop/region where backend supports it;
- permission interaction mode;
- correlation ID.

The provider returns negotiated capabilities.

---

## 6. Frame Contract

Every delivered frame includes:

- capture generation;
- source identity;
- timestamp;
- extent;
- format;
- color metadata;
- storage domain;
- cursor metadata if separate;
- damage region if available;
- protected-content status;
- flags.

---

## 7. Dynamic Reconfiguration

Session may report:

- extent changed;
- frame rate changed;
- color mode changed;
- source title changed;
- source hidden/minimized;
- source replaced;
- permission revoked;
- source ended.

Resource-incompatible changes increment capture generation.

---

## 8. Permission Mediation

Provider reports whether it can:

- query state;
- request permission;
- open system settings;
- reuse prior portal/user choice;
- require application restart.

The capture abstraction does not fake a universal permission flow.

---

## 9. Cursor

Cursor representation:

- included in frame;
- excluded;
- separate image and position;
- unsupported;
- protected by platform policy.

Separate cursor updates carry compatible timing.

---

## 10. Zero-Copy Handoff

A capture frame may expose an importable platform texture lease.

The renderer adapter validates:

- compatible device;
- handle type;
- synchronization;
- generation;
- color metadata;
- lifetime.

Fallback conversion remains explicit.

---

## 11. Health

Health includes:

- permission;
- source availability;
- frame delivery;
- current format;
- queue drops;
- backend;
- fallback;
- recovery state;
- last platform error category.

---

## 12. Invariants

1. Capture APIs are hidden behind provider interfaces.
2. Enumeration generation is explicit.
3. Source identity type matches source lifetime.
4. Frames carry timing and color metadata.
5. Permission behavior is provider-specific but normalized.
6. Zero-copy ownership is explicit.
7. Dynamic incompatible changes increment generation.
8. Protected content is respected.
9. Cursor behavior is explicit.
10. Enumeration and queues are bounded.

---

## 13. Required Tests

- provider enumeration;
- ephemeral window identity;
- persistent camera identity;
- permission required;
- portal-selected source;
- frame generation change;
- color metadata;
- separate cursor;
- protected content;
- zero-copy lease;
- fallback copy;
- source ended.

---

## 14. AI Implementation Notes

Do not force all platform source identities into one string.

Do not assume enumeration is stable across sessions.

Do not expose native capture objects to the scene graph.

Keep permission and portal behavior explicit.

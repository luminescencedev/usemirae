# 510 — Device Discovery and Hotplug

**Status:** Proposed  
**Audience:** Platform, media, audio, capture, UI contributors  
**Canonical:** Yes  
**Required context:** `03-media/301-source-system.md`, `500-platform-overview.md`  
**Related ADRs:** ADR-0035

---

## 1. Purpose

This document defines discovery, identity, capability snapshots, hotplug, replacement, and failure handling for cameras, microphones, capture cards, displays, GPUs, and related devices.

---

## 2. Device Record

A device record includes:

- Mirae device reference;
- platform identifier;
- kind;
- display label;
- vendor/product metadata;
- connection metadata;
- capability summary;
- permission state;
- availability;
- generation;
- stability class.

Sensitive serial identifiers are not exposed unnecessarily.

---

## 3. Discovery Snapshot

Enumeration produces a generation-stamped snapshot.

A diff may report:

- added;
- removed;
- changed;
- renamed;
- capability changed;
- permission changed;
- replaced identity.

Consumers must handle missed events by requesting full snapshot.

---

## 4. Stable Identity

Identity confidence may be:

- stable;
- stable per user;
- stable per session;
- heuristic;
- ephemeral.

Projects store the best stable reference and matching hints.

Automatic replacement is permitted only under configured confidence and visibility rules.

---

## 5. Hotplug

Hotplug events are debounced and coalesced.

On removal:

- invalidate runtime generation;
- mark source unavailable;
- stop device callbacks;
- release resources;
- preserve project definition;
- begin bounded recovery if policy allows.

On addition:

- update capability generation;
- attempt exact identity recovery;
- require confirmation for heuristic replacement where risk exists.

---

## 6. Capability Changes

A present device may change:

- formats;
- sample rates;
- controls;
- HDR;
- firmware mode;
- connection bandwidth;
- encoder sessions;
- device permissions.

Capability change does not necessarily mean identity change, but may require runtime generation change.

---

## 7. Multi-Client Devices

Some devices cannot be opened by multiple applications or sessions.

The platform reports:

- exclusive-use conflict;
- busy owner unknown;
- shared mode available;
- retryability.

Mirae does not spin in a tight open loop.

---

## 8. Device Controls

Optional controls may include:

- exposure;
- focus;
- white balance;
- zoom;
- gain;
- format;
- frame rate.

Control support is capability-driven.

Persisted values are validated on reconnect.

---

## 9. Invariants

1. Enumeration order is not identity.
2. Snapshots are generation-stamped.
3. Removal preserves project intent.
4. Reconnect validates identity.
5. Heuristic replacement is visible.
6. Capability change is explicit.
7. Hotplug events are coalesced.
8. Open retries are bounded.
9. Sensitive identifiers are redacted.
10. Device controls are capability-validated.

---

## 10. Required Tests

- add/remove;
- missed event and resnapshot;
- stable identity reconnect;
- heuristic replacement;
- busy device;
- capability change;
- permission change;
- rapid hotplug burst;
- format removed;
- control unavailable;
- session restart;
- redacted diagnostics.

---

## 11. AI Implementation Notes

Do not use list position as device identity.

Do not auto-replace a missing device using only a similar display name.

Do not treat every capability change as a new physical device.

Bound and debounce hotplug processing.

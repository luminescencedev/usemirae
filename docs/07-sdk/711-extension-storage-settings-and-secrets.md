# 711 — Extension Storage, Settings, and Secrets

**Status:** Proposed  
**Audience:** SDK, project, platform, security contributors  
**Canonical:** Yes  
**Required context:** `04-project/407-local-file-system.md`, `05-platform/508-secure-credential-storage.md`, `705-permission-and-capability-model.md`  
**Related ADRs:** ADR-0052, ADR-0054

---

## 1. Purpose

This document defines extension-local storage, project namespaced data, user settings, caches, temporary data, and host-mediated credentials.

---

## 2. Storage Classes

### Global extension settings

Per user/install.

Examples: preferences, account labels, feature flags.

### Project extension data

Portable project namespace.

Examples: provider configuration, automation metadata.

### Cache

Rebuildable and evictable.

### Temporary storage

Operation-scoped and crash-cleaned.

### Secure credentials

Stored only through credential broker.

---

## 3. Namespacing

All storage is scoped by:

- extension ID;
- publisher identity;
- installation/profile;
- project ID when relevant;
- schema version.

One extension cannot access another extension's storage.

---

## 4. Storage API

Storage API supports:

- get/set/delete;
- transaction or compare-and-swap where needed;
- listing with bounds;
- quota inspection;
- schema version;
- migration;
- watch limited to own namespace;
- export/delete through user action.

Arbitrary filesystem paths are not exposed.

---

## 5. Project Data

Project data:

- follows project transaction rules;
- is included in bundles according to manifest policy;
- excludes secrets;
- is size-bounded;
- has schema version;
- is preserved when extension absent;
- may support undo/redo.

---

## 6. Cache

Cache entries declare:

- key;
- bytes;
- expiration;
- rebuildability;
- source version;
- project association.

Host may evict cache at any time.

Extensions must not require cache for correctness.

---

## 7. Credentials Broker

Extension may request:

- create credential for declared service;
- use credential for a brokered request;
- rotate/revoke;
- inspect safe metadata.

By default, extension does not receive raw secret bytes.

Where raw access is unavoidable, it requires a stronger explicit capability and isolated runtime.

---

## 8. Network Broker

The broker may attach credentials to approved requests without exposing the secret.

It enforces:

- domains;
- methods;
- redirects;
- headers;
- payload size;
- rate limits;
- TLS;
- response bounds;
- logging redaction.

---

## 9. Uninstall and Data Deletion

User chooses:

- remove package only;
- remove settings/cache;
- remove credentials;
- remove project namespaces through explicit project edits.

Deletion is reported and bounded.

---

## 10. Invariants

1. Storage is extension-namespaced.
2. Project data excludes secrets.
3. Cache is non-authoritative.
4. Temporary storage is lifecycle-owned.
5. Raw filesystem access is not default.
6. Credentials use secure broker.
7. Network broker can use secrets without revealing them.
8. Quotas are enforced.
9. Extension absence preserves project data.
10. Uninstall does not silently erase project data.

---

## 11. Required Tests

- settings storage;
- project namespace;
- namespace isolation;
- quota exceeded;
- cache eviction;
- temporary cleanup;
- credential metadata;
- brokered authenticated request;
- raw-secret denial;
- uninstall choices;
- bundle inclusion;
- extension absent reopen.

---

## 12. AI Implementation Notes

Do not store extension tokens in settings or project data.

Do not expose arbitrary user filesystem paths as the storage API.

Do not make correctness depend on cache persistence.

Prefer brokered credential use over raw secret delivery.

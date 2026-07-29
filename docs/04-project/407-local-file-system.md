# 407 — Local File System

**Status:** Proposed  
**Audience:** Project, platform, security contributors  
**Canonical:** Yes  
**Required context:** `002-product-and-system-boundaries.md`, `406-asset-registry.md`

---

## 1. Purpose

This document defines filesystem abstractions, path policy, directory layout, privacy, and safe file operations.

---

## 2. Directory Classes

Mirae distinguishes:

- installation resources;
- user configuration;
- local application data;
- cache;
- logs;
- crash reports;
- project library index;
- recovery store;
- managed assets;
- temporary files;
- user project locations;
- recording locations;
- extension storage.

Each class uses platform-appropriate base directories.

---

## 3. Path Types

Use typed path wrappers:

- `ProjectPath`;
- `AssetPath`;
- `RecordingPath`;
- `CachePath`;
- `ExtensionStoragePath`;
- `TemporaryPath`.

A generic string path should not cross domain boundaries.

---

## 4. Normalization

Normalization handles:

- separators;
- relative components;
- Unicode normalization policy;
- case sensitivity awareness;
- symlinks;
- long paths;
- reserved names;
- drive/volume identity;
- network mounts;
- sandbox bookmarks or security scopes.

Normalization must not claim two paths are identical solely by string equality on all platforms.

---

## 5. Safe Writes

Safe file write API supports:

- create new;
- replace atomically;
- append where explicitly valid;
- no-follow policy;
- expected file identity;
- permission mode;
- durability level;
- cancellation before publication.

---

## 6. Symlinks and Reparse Points

Security-sensitive operations must define symlink behavior.

Bundle extraction, extension storage, and managed assets must prevent path escape through:

- `..`;
- absolute paths;
- symlinks;
- junctions/reparse points;
- alternate data streams where relevant.

---

## 7. Temporary Files

Temporary files are:

- uniquely named;
- permission-restricted;
- lifecycle-owned;
- cleaned after crash through retention policy;
- not trusted merely because created in temp directory;
- excluded from library scans.

---

## 8. File Watching

File watching may detect:

- external project modification;
- asset change;
- deletion;
- move;
- directory replacement.

Watch events are advisory and may coalesce or overflow.

The system revalidates file identity before acting.

---

## 9. Network and Removable Storage

Projects may live on network or removable storage.

The system must account for:

- weaker atomicity;
- disconnection;
- slow writes;
- stale metadata;
- locking differences;
- case behavior;
- path remount changes.

Mirae may warn when durability guarantees are reduced.

---

## 10. Privacy

Paths may reveal usernames, organizations, or project names.

Logs and diagnostic bundles should:

- redact home-directory prefixes where possible;
- hash or alias paths;
- include full paths only with explicit user consent;
- never upload path lists silently.

---

## 11. Extension Storage

Each extension receives isolated storage root.

Requirements:

- stable extension namespace;
- quota;
- no traversal;
- no access to project/user filesystem without capability;
- cleanup policy;
- migration on extension update;
- encrypted secret storage via credential APIs, not files.

---

## 12. Invariants

1. Directory classes are explicit.
2. Domain code does not use untyped path strings.
3. Atomic replacement uses platform adapter.
4. Symlink behavior is explicit.
5. Temporary files are lifecycle-owned.
6. File watchers are advisory.
7. Network-storage guarantees are reported.
8. Paths are privacy-sensitive.
9. Extension storage is isolated.
10. Secret material does not use ordinary filesystem storage.

---

## 13. Required Tests

- path normalization;
- case-sensitive/case-insensitive behavior;
- symlink escape;
- atomic replace;
- expected identity mismatch;
- temp cleanup;
- watcher overflow;
- removable disconnect;
- network path degraded guarantee;
- extension traversal;
- path redaction;
- long path.

---

## 14. AI Implementation Notes

Do not compare every path using raw string equality.

Do not trust file watcher events without revalidation.

Do not permit archive or extension paths to escape their root.

Do not log full private paths by default.

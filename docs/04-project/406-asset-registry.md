# 406 — Asset Registry

**Status:** Proposed  
**Audience:** Project, media, rendering, SDK contributors  
**Canonical:** Yes  
**Required context:** `401-project-format.md`, `407-local-file-system.md`  
**Related ADRs:** ADR-0027, ADR-0030

---

## 1. Purpose

The asset registry maps stable asset identities to local or managed resources used by a project.

It separates project references from mutable filesystem locations.

---

## 2. Asset Record

```rust
pub struct AssetRecord {
    pub id: AssetId,
    pub kind: AssetKind,
    pub location: AssetLocation,
    pub content_hash: Option<ContentHash>,
    pub byte_size: Option<u64>,
    pub media_metadata: Option<AssetMediaMetadata>,
    pub portability: AssetPortability,
    pub availability: AssetAvailability,
    pub import_metadata: AssetImportMetadata,
}
```

---

## 3. Asset Locations

Location kinds:

- project-managed;
- project-relative external;
- absolute local external;
- user-library managed;
- generated;
- extension-owned;
- embedded bundle entry.

A location is not identity.

---

## 4. Managed Assets

Managed assets are copied into Mirae-controlled project or library storage.

They are stored by content hash where practical.

Benefits:

- deduplication;
- integrity;
- portability;
- stable identity;
- recovery.

Project manifests map logical asset IDs to content-addressed blobs.

---

## 5. External Assets

External assets remain at user-selected paths.

The registry stores:

- normalized location;
- last known file identity;
- optional content hash;
- size;
- modification metadata;
- portability warning.

External files are never modified without explicit action.

---

## 6. Import

Import flow:

1. validate file and bounds;
2. detect media type;
3. compute hash if policy requires;
4. inspect metadata;
5. choose managed/external policy;
6. create asset record;
7. store or reference content;
8. commit project change.

Large imports report progress and are cancellable before commit.

---

## 7. Deduplication

Content-addressed managed storage may reuse one blob for multiple asset records.

Reference counting or reachability is maintained separately from filesystem link count.

Deletion of one asset record does not remove a blob still referenced elsewhere.

---

## 8. Availability

States:

- available;
- missing;
- changed;
- unreadable;
- unsupported;
- quarantined;
- unresolved;
- downloading for optional cloud feature.

Missing assets preserve project references.

---

## 9. Relinking

Relink operation:

- selects replacement;
- validates compatibility;
- compares hash/metadata;
- shows differences;
- updates location or asset mapping through command;
- preserves original reference in history/recovery when appropriate.

Batch relinking may use directory mapping rules with preview.

---

## 10. Metadata and Proxies

Derived metadata may include:

- dimensions;
- duration;
- codecs;
- thumbnails;
- waveform;
- proxy representation.

Derived data is cache, not canonical asset content.

---

## 11. Garbage Collection

Managed asset GC uses project/library reachability.

Rules:

- never delete currently referenced blob;
- grace period;
- exclude active imports;
- exclude recovery snapshots;
- record planned deletions;
- bounded work;
- optional dry run.

---

## 12. Security

Asset inspection treats files as untrusted.

Requirements:

- size bounds;
- parser isolation where needed;
- path traversal prevention;
- extension content restrictions;
- MIME/type validation;
- no execution;
- quarantine for suspicious content;
- diagnostics without leaking full private paths unnecessarily.

---

## 13. Invariants

1. Asset ID is independent from path.
2. Managed blobs are content-addressed.
3. External assets are not modified.
4. Missing assets preserve intent.
5. Derived proxies are rebuildable.
6. Deduplication does not break reference ownership.
7. GC respects recovery and active imports.
8. Paths are normalized at adapter boundary.
9. Asset parsing is bounded.
10. Bundle entries cannot escape extraction root.

---

## 14. Required Tests

- managed import;
- external reference;
- duplicate content;
- missing asset;
- changed external file;
- relink;
- batch relink preview;
- GC reachability;
- recovery reference protection;
- path traversal;
- malformed media;
- proxy invalidation.

---

## 15. AI Implementation Notes

Do not use the path as the stable asset ID.

Do not delete missing asset records.

Do not garbage-collect blobs without considering recovery snapshots.

Treat all imported files and bundle entries as untrusted.

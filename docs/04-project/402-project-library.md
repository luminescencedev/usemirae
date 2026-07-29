# 402 — Project Library

**Status:** Proposed  
**Audience:** Project, UI, persistence contributors  
**Canonical:** Yes  
**Required context:** `400-project-overview.md`, `407-local-file-system.md`  
**Related ADRs:** ADR-0003

---

## 1. Purpose

The project library is the local index of known projects, templates, recent items, recovery candidates, and project metadata.

It is not the project source of truth.

---

## 2. Library Records

A library record may contain:

- project ID;
- display name;
- project location;
- last opened;
- last explicit save;
- thumbnail reference;
- schema version;
- availability;
- lock state;
- recovery state;
- origin;
- tags;
- safe summary metadata.

The project file remains authoritative for project content.

---

## 3. Storage

The library index is stored separately from project files.

It may use a local embedded database or structured index selected by implementation.

The index is rebuildable from project locations and recent history where possible.

---

## 4. Discovery

Projects enter the library through:

- create;
- open;
- import;
- bundle extraction;
- template instantiation;
- explicit folder scan;
- recovery discovery.

Background scanning is bounded and excludes sensitive locations unless configured.

---

## 5. Availability States

- available;
- missing;
- moved;
- inaccessible;
- locked;
- incompatible;
- repairable;
- recovery-only;
- archived.

A missing project record is not automatically deleted.

---

## 6. Project Move Detection

Move detection may use:

- project ID;
- file identity where platform supports it;
- recent locations;
- user-selected replacement;
- bundle metadata.

Content hashing the entire project on every startup is not required.

---

## 7. Thumbnails

Thumbnails:

- are derived caches;
- may be regenerated;
- do not belong to canonical project semantics;
- are byte-bounded;
- exclude sensitive diagnostics;
- are invalidated by project generation or save metadata.

---

## 8. Templates

Templates are read-only project seeds.

Instantiation:

- copies schema content;
- generates new project ID;
- resolves template asset policy;
- removes template-only metadata;
- validates before activation.

---

## 9. Recovery Integration

Library surfaces recovery candidates separately.

It displays:

- original project;
- recovery timestamp;
- base explicit save;
- recovery generation;
- reason;
- whether original is available.

Opening recovery does not overwrite the original automatically.

---

## 10. Multi-Instance Awareness

The library may show:

- write lock owner;
- read-only availability;
- stale lock suspicion;
- active engine session on same machine.

It must not break locks automatically without recovery procedure.

---

## 11. Privacy

Library metadata remains local by default.

It must not upload project names, paths, thumbnails, or recents without explicit cloud feature policy.

---

## 12. Invariants

1. Library index is not project source of truth.
2. Missing records are preserved until user cleanup.
3. Thumbnails are derived.
4. Template instantiation creates new identity.
5. Recovery does not overwrite original automatically.
6. Index is rebuildable.
7. Scans are bounded.
8. Paths are privacy-sensitive.
9. Lock state is advisory plus validated by lock service.
10. Project content is not duplicated in index unnecessarily.

---

## 13. Required Tests

- create library record;
- missing path;
- moved project;
- duplicate project ID;
- template instantiate;
- thumbnail invalidation;
- recovery candidate;
- stale metadata;
- lock display;
- index corruption rebuild;
- privacy redaction.

---

## 14. AI Implementation Notes

Do not make the library database authoritative for project scenes or sources.

Do not delete a missing project record silently.

Do not reuse template project IDs.

Treat project paths and thumbnails as private local metadata.

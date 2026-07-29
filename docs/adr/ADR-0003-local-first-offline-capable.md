# ADR-0003 — Local-First and Offline-Capable Operation

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Live production software must remain dependable during network failure.

Mandatory cloud storage or account access would create failure modes unrelated to local production and reduce user control over projects and media.

---

## Decision

Mirae will be local-first and offline-capable.

Core capabilities will work without a Mirae account:

- create and edit projects;
- capture local sources;
- compose scenes;
- record;
- use replay;
- manage local assets;
- operate supported local outputs;
- recover projects.

Features inherently requiring a network, such as live streaming, remote ingest, collaboration, or cloud backup, may become unavailable while offline without blocking local operation.

---

## Consequences

### Positive

- production independence;
- user ownership;
- simpler local failure model;
- no mandatory service availability dependency;
- portable projects;
- privacy benefits.

### Negative

- synchronization and collaboration require later explicit design;
- conflict resolution cannot be delegated to a central database;
- local backups and recovery must be robust;
- credential and project portability require careful handling.

---

## Alternatives Considered

### Cloud-first account model

Rejected for core operation because it conflicts with production reliability and ownership goals.

### Hybrid with mandatory sign-in but local cache

Rejected because authentication failure would remain an unnecessary dependency.

---

## Implementation Notes

- Projects use a documented local schema.
- Credentials use OS secure storage.
- Cloud features must degrade gracefully.
- Project files must not contain cloud-only opaque state required for opening.
- Account identity may augment but not own local project identity.
- Offline behavior requires automated tests.

---

## Related Specifications

- `00-foundations/001-project-overview.md`
- `00-foundations/002-product-and-system-boundaries.md`
- future `04-project/401-project-format.md`
- future `04-project/404-autosave-and-recovery.md`

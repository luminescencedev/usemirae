# ADR-0030 — Portable Project Bundle Format

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Users need to transfer, back up, share, and support projects with selected assets while preserving local-first ownership.

---

## Decision

Mirae will define a separate versioned project bundle format containing the project schema, manifest, selected assets, integrity hashes, extension requirements, and portability report.

The bundle will exclude credentials.

---

## Consequences

### Positive

- portable backup and transfer;
- explicit dependency report;
- integrity checking;
- safe clone/template modes;
- no cloud requirement.

### Negative

- archive security requirements;
- potentially large files;
- asset licensing/privacy decisions;
- separate bundle-version maintenance.

---

## Alternatives Considered

### Copy project file and hope paths resolve

Rejected because external assets, fonts, and extensions would be lost or ambiguous.

### Mandatory cloud export

Rejected because it conflicts with local-first operation.

---

## Related Specifications

- `04-project/410-project-portability-and-bundles.md`
- `04-project/406-asset-registry.md`
- `04-project/401-project-format.md`

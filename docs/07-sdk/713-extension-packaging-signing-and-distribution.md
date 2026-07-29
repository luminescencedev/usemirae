# 713 — Extension Packaging, Signing, and Distribution

**Status:** Proposed  
**Audience:** SDK, security, release, marketplace/distribution contributors  
**Canonical:** Yes  
**Required context:** `702-extension-manifest.md`, `05-platform/509-updates-packaging-and-signing.md`  
**Related ADRs:** ADR-0053

---

## 1. Purpose

This document defines extension package structure, integrity, signatures, publisher identity, installation sources, updates, revocation, and trust presentation.

---

## 2. Package Contents

Conceptual package:

```text
extension.mirae-ext
├── manifest.json
├── runtime/
├── ui/
├── schemas/
├── locales/
├── assets/
├── migrations/
├── licenses/
└── signatures/
```

Every entry is declared and hashed.

---

## 3. Integrity Manifest

Includes:

- extension ID/version;
- manifest version;
- entry paths;
- byte sizes;
- hashes;
- runtime targets;
- signing metadata;
- publisher identity;
- build provenance reference;
- package-format version.

---

## 4. Signing

Signature covers canonical integrity manifest.

Verification checks:

- cryptographic validity;
- publisher identity;
- extension ID ownership;
- certificate/key validity;
- revocation;
- package contents;
- version rollback policy.

---

## 5. Trust Levels

Possible trust states:

- Mirae-signed;
- verified publisher;
- locally trusted developer;
- unsigned development package;
- invalid/revoked;
- unknown publisher.

Trust state does not auto-grant permissions.

---

## 6. Installation Sources

- official catalog;
- verified publisher repository;
- local package;
- developer directory;
- enterprise-managed source.

Source is recorded.

Catalog availability is not required for local project operation after installation.

---

## 7. Updates

Update must:

- preserve extension ID and publisher identity;
- verify new package;
- compare capabilities;
- stage;
- support rollback;
- preserve data;
- avoid auto-granting new permissions;
- respect release channel.

---

## 8. Revocation

Revocation may target:

- signing key;
- package hash;
- extension version;
- publisher.

Response may:

- block new install;
- quarantine existing package;
- disable execution;
- preserve data;
- allow user to export project without running extension.

---

## 9. Archive Security

Importer limits:

- entry count;
- compressed/uncompressed size;
- compression ratio;
- path length;
- nesting;
- symlink/reparse behavior;
- executable locations;
- duplicate paths.

Path traversal and package bombs are rejected.

---

## 10. Licensing and Notices

Package includes:

- extension license;
- third-party notices;
- privacy disclosure;
- network behavior;
- support contact;
- source disclosure where required.

---

## 11. Invariants

1. Every package entry is hashed.
2. Signature binds identity and content.
3. Trust does not grant capability.
4. Updates preserve publisher identity.
5. New permissions require review.
6. Revocation preserves project data.
7. Package extraction is bounded.
8. Unsigned packages require developer mode/explicit trust.
9. Installation source is recorded.
10. Packages include required notices.

---

## 12. Required Tests

- valid signature;
- invalid signature;
- revoked key;
- publisher mismatch;
- package tamper;
- path traversal;
- compression bomb;
- update with new permission;
- rollback;
- unsigned developer package;
- offline install;
- quarantine existing revoked version.

---

## 13. AI Implementation Notes

Do not equate catalog listing with unlimited trust.

Do not auto-grant capabilities after a signed update.

Do not extract before validating archive paths and size limits.

Preserve extension-owned project data during quarantine/uninstall.

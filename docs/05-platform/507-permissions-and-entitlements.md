# 507 — Permissions and Entitlements

**Status:** Proposed  
**Audience:** Platform, shell, security, UI contributors  
**Canonical:** Yes  
**Required context:** `500-platform-overview.md`, `03-media/302-capture-system.md`  
**Related ADRs:** ADR-0032, ADR-0035

---

## 1. Purpose

This document defines how Mirae discovers, requests, explains, and responds to protected operating-system capabilities.

---

## 2. Permission Categories

Potential categories:

- screen capture;
- camera;
- microphone;
- system audio;
- accessibility/control;
- automation;
- files and folders;
- notifications;
- network/local-network discovery;
- virtual device installation;
- device access;
- background operation.

Availability differs by platform and packaging.

---

## 3. Permission State

```rust
pub enum PermissionState {
    Unknown,
    NotRequired,
    NotRequested,
    Requestable,
    Requesting,
    Granted,
    Denied,
    Restricted,
    Revoked,
    RequiresRestart,
    BlockedByPackaging,
    Unsupported,
}
```

---

## 4. Request Principles

Mirae MUST:

- request permission only after user intent;
- explain purpose before OS prompt;
- request minimum scope;
- avoid repeated prompts;
- handle denial gracefully;
- show how to recover;
- detect restart requirements;
- not claim success until capability is verified.

---

## 5. Entitlements and Packaging

Some features require build-time or package declarations.

The capability registry distinguishes:

- OS permission granted;
- application entitlement present;
- package sandbox allowing feature;
- runtime API available.

A user cannot fix a missing entitlement through settings.

---

## 6. Permission Commands

Permission request uses a typed command.

Result includes:

- permission kind;
- previous/current state;
- whether prompt was shown;
- whether restart is required;
- settings-navigation capability;
- safe platform explanation.

---

## 7. Revocation

Permission can change while running.

On revocation:

- dependent sessions stop or degrade;
- source definitions remain;
- generations invalidate;
- health reports permission reason;
- no prompt loop begins;
- user action is offered.

---

## 8. Extension Permissions

OS permissions and extension capability grants are separate.

An extension cannot request protected OS access directly unless SDK policy routes it through the host and user-approved capability.

---

## 9. Privacy

Permission diagnostics avoid exposing:

- unrelated window lists;
- camera labels beyond need;
- full paths;
- contact/account information;
- raw OS authorization tokens.

---

## 10. Invariants

1. Permission prompts follow user intent.
2. Permission and entitlement are distinct.
3. Denial preserves project intent.
4. Revocation invalidates dependent runtimes.
5. Restart requirement is explicit.
6. Extensions cannot bypass host permission policy.
7. Packaging blocks are diagnosable.
8. Prompt loops are prohibited.
9. Granted state is verified.
10. Permission metadata is privacy-bounded.

---

## 11. Required Tests

- not requested;
- granted;
- denied;
- restricted;
- revoked at runtime;
- restart required;
- missing entitlement;
- sandbox block;
- extension request denied;
- repeated request suppression;
- settings navigation;
- source recovery after grant.

---

## 12. AI Implementation Notes

Do not request all permissions at startup.

Do not treat an OS prompt return value as proof that capture works.

Do not let extensions call native permission APIs directly.

Keep missing entitlement separate from user denial.

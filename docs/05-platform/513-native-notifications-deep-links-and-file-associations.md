# 513 — Native Notifications, Deep Links, and File Associations

**Status:** Proposed  
**Audience:** Shell, UI, platform, security contributors  
**Canonical:** Yes  
**Required context:** `501-desktop-shell.md`, `500-platform-overview.md`

---

## 1. Purpose

This document defines native integration for notifications, file associations, custom links, and external activation.

---

## 2. Notifications

Notifications may report:

- recording stopped unexpectedly;
- stream failed;
- output recovered;
- project recovery available;
- update ready;
- device permission required;
- long operation completed.

Notifications are not used for high-frequency metrics.

---

## 3. Notification Policy

A notification declares:

- category;
- severity;
- title/body;
- project/output context;
- action IDs;
- deduplication key;
- expiry;
- privacy level.

Sensitive project or endpoint names may be hidden on locked screens.

---

## 4. Deep Links

Custom deep links may support:

- open project by local library identity;
- import approved bundle reference;
- navigate to settings/help;
- OAuth callback;
- extension installation review.

Every link is parsed and validated.

Deep links never execute arbitrary commands or shell strings.

---

## 5. OAuth Callbacks

OAuth callback flow:

- validates state/nonce;
- matches pending authorization operation;
- rejects expired or unsolicited callbacks;
- passes authorization code to trusted provider adapter;
- stores resulting credentials in secure store;
- does not expose token in UI route or logs.

---

## 6. File Associations

Potential associated types:

- Mirae project;
- Mirae project bundle;
- Mirae extension package;
- diagnostic bundle.

Opening a file:

- validates type by content, not extension alone;
- routes through import/open command;
- handles second-instance forwarding;
- treats content as untrusted.

---

## 7. Action Security

Notification/deep-link actions are capability-scoped.

Examples:

- “Open diagnostics” is allowed;
- “Restart stream” requires engine session and current output validation;
- destructive actions require confirmation;
- stale actions fail safely.

---

## 8. Invariants

1. Notifications are rate-limited and deduplicated.
2. Sensitive content respects lock-screen privacy.
3. Deep links are schema-validated.
4. No arbitrary shell execution.
5. OAuth callbacks validate state.
6. File type is content-validated.
7. Stale actions are rejected.
8. External activation routes through commands.
9. Extension installation requires review.
10. Notification actions do not bypass permissions.

---

## 9. Required Tests

- notification deduplication;
- lock-screen redaction;
- valid deep link;
- malformed deep link;
- command injection attempt;
- OAuth state mismatch;
- expired callback;
- project file open;
- fake extension file;
- second-instance forwarding;
- stale notification action;
- destructive confirmation.

---

## 10. AI Implementation Notes

Do not interpolate deep-link parameters into shell commands.

Do not trust file extensions alone.

Do not put OAuth tokens in URLs or logs.

Route every activation into validated typed commands.

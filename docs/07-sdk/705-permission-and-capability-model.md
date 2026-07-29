# 705 — Permission and Capability Model

**Status:** Proposed  
**Audience:** SDK, security, UI, runtime contributors  
**Canonical:** Yes  
**Required context:** `06-quality/612-security-model.md`, `702-extension-manifest.md`  
**Related ADRs:** ADR-0048, ADR-0052

---

## 1. Purpose

This document defines requested permissions, user grants, runtime capability tokens, scopes, revocation, auditing, and permission UX.

---

## 2. Separation

- Manifest request describes desired access.
- User/policy grant authorizes access.
- Runtime token proves current scoped access.
- API endpoint validates token and context.
- Revocation invalidates current access.

These are separate layers.

---

## 3. Capability Categories

### Project

- read project metadata;
- read selected entity classes;
- read current scene state;
- write extension namespace;
- invoke approved project commands.

### Media

- register source;
- receive source-owned frames;
- register output;
- register effect;
- use bounded shared media buffers.

### Platform

- select files through broker;
- use extension storage;
- request notifications;
- use declared network domains;
- request credential broker;
- request approved device class.

### UI

- add declared panel;
- add command;
- add context action;
- show constrained dialog.

---

## 4. Scope

A grant may be scoped to:

- extension;
- extension version;
- project;
- source/output instance;
- service account;
- hostname/domain;
- file selected by user;
- directory bookmark;
- device;
- session;
- one operation.

Broad global grants should be avoided.

---

## 5. Grant States

- not requested;
- requested;
- granted;
- denied;
- partially granted;
- expired;
- revoked;
- blocked by policy;
- unsupported;
- requires OS permission.

---

## 6. User Review

Permission UI explains:

- what access is requested;
- why;
- when used;
- data involved;
- scope;
- whether required or optional;
- consequences of denial;
- publisher identity.

Permission descriptions are extension-provided but host-framed and localized safely.

---

## 7. Runtime Tokens

Tokens are:

- opaque;
- short-lived or session-scoped;
- extension-bound;
- capability-bound;
- scope-bound;
- non-persisted by extension;
- rotated on host restart;
- invalidated on revocation.

---

## 8. Revocation

Revocation:

- prevents new calls;
- cancels or drains dependent operations;
- stops affected source/output if needed;
- invalidates tokens;
- updates UI;
- preserves project configuration;
- records audit event.

---

## 9. Audit

Audit records:

- extension ID/version;
- capability;
- scope;
- grant/revoke time;
- user/policy actor;
- operation correlation;
- denied attempts.

Secrets and content are excluded.

---

## 10. Invariants

1. Request is not grant.
2. Grants are scoped.
3. Tokens are ephemeral.
4. Every endpoint checks capability.
5. Revocation takes effect during runtime.
6. Denial preserves project intent.
7. Permission UX identifies publisher.
8. OS permission and extension grant are distinct.
9. Audit excludes content/secrets.
10. Broad grants require explicit justification.

---

## 11. Required Tests

- required grant;
- optional denial;
- partial grant;
- project-scoped grant;
- domain-scoped network grant;
- user-selected file grant;
- token spoof;
- token expiry;
- revocation during operation;
- OS permission missing;
- audit record;
- host restart token rotation.

---

## 12. AI Implementation Notes

Do not authorize based only on extension ID or installed state.

Do not persist runtime tokens.

Do not create vague permissions such as “full access” when narrower scopes are possible.

Check capability at the actual resource boundary.

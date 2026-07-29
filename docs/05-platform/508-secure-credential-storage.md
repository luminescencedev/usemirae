# 508 — Secure Credential Storage

**Status:** Proposed  
**Audience:** Platform, security, project, output contributors  
**Canonical:** Yes  
**Required context:** `04-project/401-project-format.md`, `500-platform-overview.md`  
**Related ADRs:** ADR-0033

---

## 1. Purpose

This document defines secure local storage for stream keys, tokens, passwords, refresh credentials, and other secrets.

---

## 2. Credential Model

Project/configuration state stores a `CredentialRef`.

The secure store owns:

- secret bytes;
- access policy;
- creation/update time;
- provider metadata;
- optional account label;
- platform storage identity.

---

## 3. Credential Interface

```rust
pub trait CredentialStore {
    fn create(&self, request: CreateCredential) -> Result<CredentialRef>;
    fn resolve(&self, reference: &CredentialRef) -> Result<SecretLease>;
    fn update(&self, reference: &CredentialRef, secret: SecretInput) -> Result<()>;
    fn delete(&self, reference: &CredentialRef) -> Result<()>;
    fn inspect(&self, reference: &CredentialRef) -> Result<CredentialMetadata>;
}
```

Secret values do not implement ordinary debug formatting.

---

## 4. Platform Backends

Preferred backends:

- Windows secure credential facility;
- macOS Keychain;
- Linux desktop secret service or equivalent supported secure store.

No secure backend means credential-dependent features are unavailable unless a separately designed encrypted local vault is introduced.

Plaintext fallback is prohibited.

---

## 5. Secret Lease

A secret lease:

- has short lifetime;
- minimizes copies;
- zeroizes memory where practical;
- is never serialized;
- is not sent to UI;
- is redacted in errors;
- is passed only to authorized output/service adapter.

---

## 6. Credential Lifecycle

States:

- available;
- missing;
- locked;
- user interaction required;
- backend unavailable;
- access denied;
- corrupted;
- expired;
- revoked.

The project remains valid when a credential is unavailable.

---

## 7. Import and Export

Project bundles do not include credentials.

Credential import/export requires a separate encrypted workflow and explicit user action.

Copying a project to another machine leaves references unresolved.

---

## 8. Logs and Diagnostics

Redaction applies to:

- stream keys;
- tokens;
- signed URLs;
- authorization headers;
- cookies;
- password-like fields;
- secret environment variables.

Structured logging should mark secret-bearing values with dedicated types to prevent accidental formatting.

---

## 9. Rotation

Providers may support credential rotation.

Rotation:

- updates secure-store value;
- preserves or replaces reference according to policy;
- invalidates cached secret leases;
- notifies dependent outputs without exposing value;
- may require reconnect.

---

## 10. Invariants

1. Secrets never enter project files.
2. Plaintext fallback is prohibited.
3. UI never receives raw secrets after submission unless explicitly necessary for editing and securely handled.
4. Secret leases are short-lived.
5. Debug formatting is redacted.
6. Credential backend failure does not corrupt project.
7. Bundle export excludes secrets.
8. Rotation invalidates leases.
9. Extensions require explicit credential capability and never receive unrelated secrets.
10. Credential references are stable and non-secret.

---

## 11. Required Tests

- create/resolve/delete;
- backend unavailable;
- locked store;
- missing credential;
- redacted logs;
- project serialization scan;
- bundle exclusion;
- secret lease lifetime;
- rotation;
- extension denial;
- output reconnect;
- memory-zeroization best-effort test.

---

## 12. AI Implementation Notes

Do not store tokens in `.env`, registry values, project JSON, or ordinary config files for production.

Do not derive credential identity from the secret value.

Do not log full endpoint URLs containing signed query parameters.

Use secret wrapper types that prevent accidental formatting.

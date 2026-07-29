# 503 — macOS Platform

**Status:** Proposed  
**Audience:** macOS, capture, audio, GPU, packaging contributors  
**Canonical:** Yes  
**Required context:** `500-platform-overview.md`, `505-platform-capture-abstraction.md`  
**Related ADRs:** ADR-0031, ADR-0032, ADR-0033, ADR-0038

---

## 1. Purpose

This document defines macOS-specific implementation boundaries, privacy behavior, entitlements, media integration, and distribution requirements.

---

## 2. Platform Adapter Areas

macOS adapters may integrate:

- screen/window/application capture;
- camera and microphone;
- system audio where supported by platform APIs and permissions;
- Metal-backed rendering through `wgpu`;
- hardware encoding;
- Keychain credentials;
- application lifecycle;
- notifications;
- sandbox entitlements;
- signing and notarization;
- update installation.

---

## 3. Capture

The preferred screen/window capture path uses the modern system capture framework available on supported macOS versions.

The adapter exposes:

- displays;
- windows;
- applications;
- cursor configuration;
- audio capability;
- content filters;
- dynamic frame properties;
- permission state.

Older or limited systems may use a fallback only when security and capability are acceptable.

---

## 4. Privacy Permissions

Potential protected capabilities include:

- screen recording;
- camera;
- microphone;
- accessibility for optional control integrations;
- automation;
- files and folders;
- notifications.

Mirae must:

- request only when feature is invoked;
- explain why before OS prompt;
- detect denial/restriction;
- avoid prompt loops;
- guide user to system settings;
- recognize when application restart is required.

---

## 5. Audio

Audio adapters handle:

- input devices;
- output monitoring;
- device changes;
- sample-rate changes;
- aggregate/multi-output devices where supported;
- low-latency callbacks.

System-audio capture capability is reported explicitly and must not be assumed for every OS version or source type.

---

## 6. GPU and Encoder Interop

The rendering backend uses Metal through `wgpu`.

Hardware encode adapters may use system media frameworks.

Interop defines:

- pixel-buffer/texture ownership;
- synchronization;
- color attachments;
- HDR metadata;
- device compatibility;
- fallback conversion.

---

## 7. App Sandbox

If distributed in a sandboxed mode, Mirae requires explicit entitlements and user-selected access for protected paths.

Security-scoped resources or bookmarks must be wrapped by the file-system adapter and never leak into project semantics directly.

A non-sandboxed signed distribution may have different capabilities; packaging mode is observable.

---

## 8. Credentials

Secrets use the system Keychain through the secure credential interface.

Access groups, synchronization behavior, and prompts are selected conservatively.

Project files store only credential references.

---

## 9. Signing and Notarization

Release artifacts require:

- code signing;
- hardened runtime where applicable;
- entitlements;
- notarization;
- stapling/verification;
- signed update metadata.

Nested executables and helper processes must be signed consistently.

---

## 10. Application Lifecycle

Handle:

- reopen;
- dock activation;
- quit;
- sudden termination constraints;
- sleep/wake;
- display changes;
- user session changes;
- app translocation or moved installation where relevant.

Engine shutdown remains staged and bounded.

---

## 11. Diagnostics

macOS diagnostics may include:

- OS version/build;
- hardware model class;
- GPU;
- capture framework/backend;
- authorization states;
- sandbox mode;
- entitlement status;
- encoder capabilities;
- display EDR/HDR capability;
- notarization/signature status;
- active workarounds.

---

## 12. Invariants

1. macOS SDK types remain in adapters.
2. Privacy prompts are user-action-driven.
3. Sandbox access is explicit.
4. Keychain stores credentials.
5. Signed helpers match application identity.
6. Capture capability is version- and permission-aware.
7. Metal resources are generation-tracked.
8. Color metadata is preserved through pixel-buffer interop.
9. Packaging mode is observable.
10. Restart-required permissions are communicated.

---

## 13. Required Tests

- screen permission denied;
- camera permission denied;
- permission granted after settings change;
- screen capture;
- window removal;
- audio device change;
- Metal texture interop;
- hardware encoder;
- sandbox bookmark;
- sleep/wake;
- signed helper verification;
- update package verification.

---

## 14. AI Implementation Notes

Do not request every privacy permission at application launch.

Do not persist security-scoped tokens as generic project paths.

Do not assume system-audio capture is available in every supported configuration.

Keep Objective-C/Swift ownership and autorelease behavior inside adapter modules.

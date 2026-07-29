# 502 — Windows Platform

**Status:** Proposed  
**Audience:** Windows, capture, audio, GPU, packaging contributors  
**Canonical:** Yes  
**Required context:** `500-platform-overview.md`, `505-platform-capture-abstraction.md`  
**Related ADRs:** ADR-0031, ADR-0032, ADR-0033, ADR-0038

---

## 1. Purpose

This document defines Windows-specific implementation boundaries and capability expectations.

It does not lock Mirae to one API when multiple backends are required.

---

## 2. Platform Adapter Areas

Windows adapters may integrate:

- desktop/window capture;
- camera capture;
- audio capture and loopback;
- display enumeration;
- GPU interop;
- hardware encoders;
- credential storage;
- notifications;
- installer and updater;
- code signing;
- session and power events.

---

## 3. Capture Backends

The preferred screen/window capture path should use modern Windows capture APIs where supported.

Fallbacks may exist for:

- unsupported OS versions;
- protected or incompatible windows;
- driver issues;
- special full-screen applications.

Every fallback reports limitations such as:

- cursor support;
- HDR metadata;
- occlusion behavior;
- minimized-window behavior;
- border capture;
- alpha;
- frame pacing.

---

## 4. Audio

Windows audio integration may provide:

- microphone/device capture;
- system-output loopback;
- output monitoring;
- endpoint notifications;
- shared/exclusive modes where appropriate.

Device identity must survive ordinary enumeration reorder.

Audio callbacks follow the real-time rules defined in the audio architecture.

---

## 5. Camera and Capture Devices

Camera and capture-card adapters may use platform media APIs or vendor backends behind the same Mirae source contract.

Capability negotiation includes:

- formats;
- frame rates;
- color range;
- HDR;
- audio;
- device controls;
- latency.

---

## 6. GPU and Encoder Interop

Windows GPU paths may involve:

- Direct3D-compatible `wgpu` backend;
- shared textures;
- synchronization handles;
- Media Foundation or vendor hardware encoders;
- adapter LUID matching;
- multi-GPU copy fallback.

Interop must verify that capture, renderer, and encoder use compatible adapters.

---

## 7. Multiple GPUs

The platform reports:

- renderer adapter;
- capture adapter;
- encoder adapter;
- power preference;
- cross-adapter transfer requirement;
- laptop hybrid-GPU status.

Mirae should avoid hidden cross-GPU copies.

When unavoidable, diagnostics expose them.

---

## 8. Credentials

Credentials use a Windows secure credential facility through the credential-store interface.

Credential references remain stable even if display labels change.

Secrets are never placed in registry values, project files, or plain configuration.

---

## 9. Packaging

Supported packaging modes may include:

- signed installer;
- package-based distribution;
- portable development build.

Capability differences between modes are explicit.

Update behavior depends on installation mode and permissions.

---

## 10. Session and Power

Handle:

- lock/unlock;
- remote-session changes;
- display-off;
- suspend;
- resume;
- fast user switching;
- shutdown/logoff requests;
- device change notifications.

A locked session does not automatically stop outputs unless policy requires it.

---

## 11. Diagnostics

Windows diagnostics may include:

- OS build;
- GPU adapter and driver;
- capture backend;
- audio endpoints;
- encoder backend;
- package mode;
- signing state;
- active workarounds;
- protected-content errors;
- session type.

Sensitive device identifiers are redacted when exported.

---

## 12. Invariants

1. Windows API types remain inside adapters.
2. Capture backend is capability-selected.
3. Cross-GPU copies are visible.
4. Device identity is not enumeration index.
5. Credentials use secure storage.
6. Packaging-mode limitations are explicit.
7. Protected-content restrictions are respected.
8. Power/session events are serialized into platform events.
9. Hardware encoders are negotiated.
10. Driver workarounds are centralized.

---

## 13. Required Tests

- preferred capture backend;
- capture fallback;
- minimized window;
- HDR display capture;
- system audio loopback;
- endpoint hotplug;
- hybrid GPU;
- hardware encoder fallback;
- lock/unlock;
- suspend/resume;
- package-mode capability;
- credential round trip.

---

## 14. AI Implementation Notes

Do not hardcode one capture API as universally available.

Do not assume renderer and encoder are on the same GPU.

Do not store secrets in the registry or project.

Keep COM and native-handle ownership inside reviewed wrappers.

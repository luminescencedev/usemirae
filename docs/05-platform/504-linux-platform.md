# 504 — Linux Platform

**Status:** Proposed  
**Audience:** Linux, capture, audio, GPU, packaging contributors  
**Canonical:** Yes  
**Required context:** `500-platform-overview.md`, `505-platform-capture-abstraction.md`  
**Related ADRs:** ADR-0031, ADR-0032, ADR-0038

---

## 1. Purpose

This document defines Linux integration across different desktop sessions, display servers, portals, audio stacks, graphics drivers, hardware encoders, and packaging modes.

Linux capability must be discovered rather than inferred from distribution name alone.

---

## 2. Session Diversity

Important dimensions include:

- Wayland versus X11/XWayland;
- desktop portal availability;
- PipeWire availability;
- desktop environment;
- compositor behavior;
- sandbox/package mode;
- GPU driver;
- user permissions and groups.

The capability registry records these dimensions.

---

## 3. Capture

Preferred modern desktop capture should use the desktop portal and PipeWire path where available.

Other paths may support:

- X11 display/window capture;
- direct compositor-specific integration only when justified;
- camera through common Linux media interfaces;
- capture cards through device adapters.

Under Wayland, Mirae must respect portal-mediated user selection and permissions.

---

## 4. Audio

Audio integration should support the active user audio stack through an adapter.

Capabilities include:

- device input;
- output monitoring/loopback;
- endpoint changes;
- sample-rate/channel-layout negotiation;
- PipeWire graph integration where available.

Mirae should not hardcode one distribution-specific configuration.

---

## 5. Graphics

The `wgpu` backend may use Vulkan or another supported backend.

The platform reports:

- graphics API;
- driver;
- adapter;
- external-memory capabilities;
- DMA-BUF or equivalent interop availability;
- cross-device transfer;
- presentation support.

---

## 6. Hardware Encoders

Possible implementations may include:

- VA-API;
- vendor-specific adapters;
- software fallback.

The encoder registry exposes stable Mirae capabilities rather than driver-specific option names.

Device-node access and package dependencies are diagnosed.

---

## 7. Portals and Sandboxes

Flatpak or other sandboxed packaging may require portals for:

- file access;
- screen capture;
- notifications;
- opening URLs;
- device access.

Packaging mode can change feature availability.

The platform must report `BlockedByPackaging` where applicable.

---

## 8. File System

Linux adapters account for:

- case-sensitive paths;
- symbolic links;
- mount changes;
- removable media;
- XDG base directories;
- network filesystems;
- executable permission bits;
- desktop integration files.

---

## 9. Credential Storage

Credentials use a supported desktop secret service when available.

If secure storage is unavailable, Mirae must not silently fall back to plaintext.

The UI may require configuration or disable credential-dependent outputs.

---

## 10. Packaging

Potential package formats include:

- AppImage;
- Flatpak;
- distribution packages;
- development tarball.

Each mode declares:

- update mechanism;
- sandbox status;
- codec availability;
- device access;
- desktop integration;
- secure-store availability.

---

## 11. Session and Power

Handle:

- session lock;
- suspend/resume;
- compositor restart;
- PipeWire restart;
- portal service restart;
- display hotplug;
- user logout/shutdown.

Recovery is bounded and generation-aware.

---

## 12. Diagnostics

Linux diagnostics may include:

- kernel;
- distribution and desktop session;
- Wayland/X11;
- portal versions/capabilities;
- PipeWire/audio service state;
- GPU driver;
- hardware encoder backend;
- package mode;
- sandbox;
- device permissions;
- workarounds.

---

## 13. Invariants

1. Linux support is capability-driven.
2. Wayland capture respects portal mediation.
3. Secure-store absence never causes plaintext fallback.
4. Package mode limitations are explicit.
5. Driver/API types remain in adapters.
6. X11 and Wayland paths remain separate.
7. Device permissions are diagnosable.
8. Portal/service restarts increment relevant generations.
9. Hardware encoding is negotiated.
10. Distribution-specific assumptions are minimized.

---

## 14. Required Tests

- Wayland portal capture mock;
- portal denial;
- X11 capture fallback;
- PipeWire restart;
- audio-device hotplug;
- Vulkan adapter;
- external-memory unavailable;
- hardware encoder permission failure;
- Flatpak capability;
- secure-store unavailable;
- suspend/resume;
- display hotplug.

---

## 15. AI Implementation Notes

Do not assume X11 APIs work under native Wayland.

Do not bypass portals in sandboxed environments.

Do not fall back to a plaintext token file.

Model packaging and session type as capabilities, not compile-time constants.

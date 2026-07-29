# 302 — Capture System

**Status:** Proposed  
**Audience:** Platform, media, security, rendering contributors  
**Canonical:** Yes  
**Required context:** `301-source-system.md`, future `05-platform/505-platform-capture.md`  
**Related ADRs:** ADR-0016

---

## 1. Purpose

The capture system acquires live video and audio from operating-system and device APIs through platform adapters.

---

## 2. Capture Categories

- display capture;
- window capture;
- application capture;
- camera capture;
- microphone capture;
- system audio capture;
- capture card;
- mobile/remote device input;
- browser surface capture;
- extension-defined capture.

Each category has independent capability, permission, and failure behavior.

---

## 3. Capture Contract

Conceptual interface:

```rust
pub trait CaptureSession: Send {
    fn capabilities(&self) -> CaptureCapabilities;
    fn generation(&self) -> CaptureGeneration;
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn subscribe_video(&self) -> VideoFrameReceiver;
    fn subscribe_audio(&self) -> AudioBlockReceiver;
    fn health(&self) -> CaptureHealth;
}
```

Platform-specific handles remain private.

---

## 4. Permission Flow

Permission states:

- unknown;
- not requested;
- requesting;
- granted;
- denied;
- restricted;
- revoked;
- requires restart.

The UI requests permission through typed commands and receives platform-safe guidance.

Capture startup must not repeatedly trigger OS prompts without user action.

---

## 5. Frame Delivery

Capture callbacks should perform minimal work:

- validate metadata;
- attach timestamp;
- wrap or import frame;
- enqueue into bounded queue;
- update lightweight metrics.

Heavy conversion, logging, and allocation occur downstream.

---

## 6. Zero-Copy and Interop

Preferred path:

```text
platform capture surface
→ renderer-importable texture
→ GPU composition
```

Fallback path:

```text
platform capture frame
→ CPU-visible buffer
→ staged GPU upload
→ composition
```

Interop contract defines:

- device compatibility;
- ownership;
- synchronization;
- color metadata;
- frame lifetime;
- fallback behavior.

---

## 7. Dynamic Changes

Capture must handle:

- resolution change;
- orientation change;
- frame-rate change;
- color mode change;
- window resize;
- minimized/hidden window;
- device removal;
- device format renegotiation;
- display topology change.

Dynamic change increments capture generation where resource compatibility changes.

---

## 8. Window Capture Privacy

The system should preserve platform privacy semantics.

It must not:

- bypass protected-content restrictions;
- capture excluded windows when APIs prohibit it;
- hide permission state;
- expose window titles unnecessarily in diagnostics;
- persist sensitive window metadata beyond need.

---

## 9. Cursor Capture

Cursor policy is explicit:

- included by platform;
- composited separately;
- excluded;
- selectable;
- unsupported.

Separate cursor composition must preserve timestamp and position consistency.

---

## 10. Audio Capture

Audio capture adapters define:

- device clock;
- sample rate;
- channel layout;
- sample format;
- timestamp source;
- loopback semantics;
- exclusive/shared mode;
- discontinuity reporting.

Conversion to canonical audio format occurs outside the callback when possible.

---

## 11. Capture Backpressure

Capture producers generally cannot block external APIs indefinitely.

Overflow policy may be:

- drop oldest video frame;
- keep latest frame;
- mark discontinuity;
- bounded audio ring buffer with XRUN diagnostics;
- restart source if API contract is violated.

Every drop is counted and reason-coded.

---

## 12. Invariants

1. Capture callbacks are bounded.
2. Permissions are explicit.
3. Platform handles remain behind adapters.
4. Capture generation changes on incompatible resource replacement.
5. Zero-copy ownership is explicit.
6. Protected content restrictions are respected.
7. Queue overflow policy is defined.
8. Color and timing metadata accompany frames.
9. Device removal does not delete project intent.
10. Repeated permission prompts require user action.

---

## 13. Required Tests

- permission denied;
- permission revoked;
- display resize;
- window destroyed;
- camera disconnect;
- zero-copy mock;
- CPU fallback;
- cursor policy;
- protected-content behavior;
- video queue overflow;
- audio XRUN;
- capture generation change.

---

## 14. AI Implementation Notes

Do not do expensive conversion inside platform capture callbacks.

Do not bypass OS privacy restrictions.

Do not assume captured frames remain valid after callback without an ownership contract.

Keep capture permission state separate from source configuration.

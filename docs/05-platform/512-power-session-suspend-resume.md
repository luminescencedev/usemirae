# 512 — Power, Session, Suspend, and Resume

**Status:** Proposed  
**Audience:** Runtime, platform, media, output contributors  
**Canonical:** Yes  
**Required context:** `01-runtime/102-engine-lifecycle.md`, `500-platform-overview.md`

---

## 1. Purpose

This document defines behavior for sleep, wake, session lock, logout, display power changes, and related operating-system lifecycle events.

---

## 2. Event Classes

- suspend imminent;
- resumed;
- session locked;
- session unlocked;
- user logout/shutdown requested;
- display off/on;
- remote session connected/disconnected;
- power source changed;
- thermal/power pressure changed.

Not all platforms support every event.

---

## 3. Suspend Preparation

When suspend is imminent:

- persist recovery checkpoint if time permits;
- stop or pause device sessions according to backend requirements;
- flush recording according to bounded policy;
- notify network outputs;
- record lifecycle diagnostic;
- avoid long blocking refusal unless platform supports explicit delay.

Mirae cannot assume unlimited time.

---

## 4. Resume

On resume:

1. increment platform/session generation where required;
2. re-probe clocks;
3. emit media discontinuity;
4. validate GPU/device state;
5. re-enumerate affected devices;
6. restore capture/audio sessions;
7. evaluate outputs;
8. publish degraded/recovered status.

Wall-clock change does not alter master media ordering.

---

## 5. Active Outputs

Policies differ:

- local recording may segment before suspend;
- live stream may disconnect and require explicit or configured reconnect;
- replay may reset continuity;
- virtual devices may restart;
- audio monitoring may stop and resume.

Automatic external-output resume is explicit and bounded.

---

## 6. Session Lock

Locking the screen:

- does not automatically stop all production;
- may make capture sources unavailable depending on OS;
- may hide preview;
- may affect credential access;
- emits platform event;
- applies configured privacy behavior.

---

## 7. Shutdown and Logout

The shell receives request and begins bounded drain.

If OS deadline is shorter than normal shutdown:

- prioritize recording/project recovery;
- stop external outputs;
- skip nonessential cleanup;
- record incomplete shutdown stage.

---

## 8. Thermal and Power Pressure

On supported systems, pressure may affect:

- preview quality;
- background thumbnails;
- optional caches;
- diagnostics frequency;
- low-priority effects in explicitly configured adaptive mode.

Production output quality does not silently change by default.

---

## 9. Invariants

1. Suspend/resume creates explicit discontinuity.
2. Recovery work is bounded.
3. Wall-clock change does not reorder media.
4. Device and GPU generations are revalidated.
5. External output auto-resume is explicit.
6. Lock does not rewrite project intent.
7. Shutdown prioritizes user data and recording safety.
8. Thermal adaptation is visible.
9. Platform event support is capability-driven.
10. Repeated resume failures escalate.

---

## 10. Required Tests

- suspend during idle;
- suspend during recording;
- suspend during stream;
- resume clock rebase;
- GPU invalidation;
- audio-device replacement;
- session lock capture loss;
- shutdown deadline;
- thermal pressure;
- remote-session switch;
- repeated resume failure;
- recovery checkpoint.

---

## 11. AI Implementation Notes

Do not assume the process remains paused with all device handles valid after resume.

Do not automatically restart external streams without policy.

Do not use wall-clock delta as media-timeline continuation.

Prioritize bounded recovery and data safety.

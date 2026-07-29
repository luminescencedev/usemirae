# 307 — Audio Routing and Monitoring

**Status:** Proposed  
**Audience:** Audio, output, UI contributors  
**Canonical:** Yes  
**Required context:** `306-audio-architecture.md`  
**Related ADRs:** ADR-0018, ADR-0019

---

## 1. Purpose

This document defines source-to-bus routing, program mixes, monitor mixes, output-specific audio maps, mute semantics, and feedback prevention.

---

## 2. Routing Entities

- source channel set;
- input bus;
- subgroup bus;
- program bus;
- monitor bus;
- output bus;
- sidechain/control bus.

Each bus has stable identity and explicit channel layout.

---

## 3. Source Controls

Per source:

- gain;
- mute;
- solo;
- pan/balance;
- channel map;
- monitor mode;
- bus sends;
- effect chain;
- delay compensation;
- scene-ownership behavior.

---

## 4. Mute Semantics

Mute types must be distinguished:

- source mute;
- program mute;
- monitor mute;
- output-specific mute;
- temporary solo suppression;
- hardware/device mute observation.

A single boolean is insufficient for all semantics.

---

## 5. Monitoring Modes

Initial monitor modes:

- off;
- monitor only;
- monitor and program;
- program only;
- pre-fader monitor;
- post-fader monitor.

Default behavior must avoid accidental feedback.

---

## 6. Feedback Prevention

The system should detect or warn about likely loops such as:

```text
system output
→ system audio capture
→ monitor output
→ system output
```

Platform-specific loopback semantics are included in diagnostics.

Automatic prevention must be visible and reversible.

---

## 7. Output-Specific Mixes

Each output profile may select:

- program mix;
- alternate language bus;
- isolated source bus;
- custom matrix;
- multitrack recording map.

Changing one output mix does not mutate unrelated outputs.

---

## 8. Scene-Linked Audio

Sources may be:

- global;
- scene-owned;
- active when visible;
- active when scene is previewed;
- active when scene is program;
- manually armed.

Activation and transition behavior are explicit.

---

## 9. Audio Transition Policy

On scene transition:

- cut;
- crossfade;
- preserve global sources;
- fade scene-owned sources;
- custom bus automation.

The audio engine follows the same authoritative transition timeline.

---

## 10. Delay Compensation

Routes may apply bounded delay for:

- device latency;
- video alignment;
- effect latency;
- output encoder alignment.

Delay is measured in media time or sample frames and included in diagnostics.

---

## 11. Meter Routing

Meters may be exposed for:

- source pre-fader;
- source post-fader;
- buses;
- program;
- monitor;
- output tracks.

Meter subscriptions are rate-limited and capability-scoped.

---

## 12. Invariants

1. Bus identity is stable.
2. Program and monitor mixes are distinct.
3. Output-specific mix changes are isolated.
4. Mute semantics are explicit.
5. Feedback prevention is visible.
6. Scene-linked audio policy is explicit.
7. Audio transitions use master timeline.
8. Delay compensation is bounded.
9. Meter paths do not alter audio.
10. Extension routing is capability-scoped.

---

## 13. Required Tests

- source to program;
- monitor only;
- output-specific track map;
- scene-owned audio transition;
- global source preservation;
- feedback loop warning;
- delay compensation;
- solo interaction;
- monitor device failure;
- multitrack recording;
- meter subscription load.

---

## 14. AI Implementation Notes

Do not collapse all mute states into one flag.

Do not route monitor audio back into captured system output without loop detection.

Keep output-specific routing separate from the project-wide program bus.

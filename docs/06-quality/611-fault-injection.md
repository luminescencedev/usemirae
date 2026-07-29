# 611 — Fault Injection

**Status:** Proposed  
**Audience:** Runtime, media, rendering, project, platform, release contributors  
**Canonical:** Yes  
**Required context:** `609-testing-strategy.md`, `605-error-model.md`  
**Related ADRs:** ADR-0042

---

## 1. Purpose

Fault injection validates that Mirae's recovery and isolation claims hold under controlled failure.

---

## 2. Fault Categories

- allocation failure;
- disk full;
- permission denied;
- file corruption;
- process crash;
- thread/task cancellation;
- IPC disconnect;
- delayed/lost message;
- queue saturation;
- device removal;
- GPU device loss;
- encoder failure;
- network disconnect;
- clock discontinuity;
- extension timeout/crash;
- update interruption.

---

## 3. Injection Points

Fault points are named and stable.

Examples:

```text
project.save.before_publish
project.save.after_publish
render.submit.device_lost
output.network.after_auth
audio.device.callback_gap
ipc.state_patch.drop
extension.call.timeout
```

Production builds may compile out dangerous controls, while test builds expose them through secure local harnesses.

---

## 4. Determinism

Faults can trigger by:

- nth invocation;
- matching entity;
- time after start;
- probability with fixed seed;
- specific lifecycle phase;
- explicit test command.

Fixed seeds and exact conditions are recorded.

---

## 5. Safety

Fault injection must not:

- target unrelated user files;
- expose remote control;
- remain enabled in release accidentally;
- upload test data;
- break OS-wide resources;
- bypass sandbox boundaries.

---

## 6. Expected Assertions

Each fault test defines:

- affected component;
- expected error code;
- expected health transition;
- expected recovery;
- unaffected components;
- persisted data expectation;
- resource cleanup;
- diagnostics.

---

## 7. Chaos and Soak

Controlled chaos suites may combine:

- device reconnects;
- network loss;
- extension restart;
- UI disconnect;
- project autosave;
- output operation.

Combination count is bounded and reproducible.

---

## 8. Invariants

1. Fault points are named.
2. Tests are reproducible.
3. Expected unaffected components are asserted.
4. Data integrity is verified.
5. Resource cleanup is verified.
6. Dangerous controls are unavailable remotely.
7. Release builds cannot enable arbitrary injection.
8. Diagnostics identify injected fault.
9. Recovery attempts are bounded.
10. Fault tests do not modify unrelated user data.

---

## 9. Required Tests

- project save interruption;
- disk full;
- IPC disconnect;
- event gap;
- source removal;
- GPU device loss;
- encoder crash;
- network reconnect;
- extension timeout;
- updater interruption;
- multi-fault chaos;
- injection disabled in release.

---

## 10. AI Implementation Notes

Do not add hidden production-only fault toggles.

Do not consider recovery tested until unaffected components and data integrity are asserted.

Use stable injection point names and fixed seeds.

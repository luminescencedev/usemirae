# 609 — Testing Strategy

**Status:** Proposed  
**Audience:** All contributors and coding agents  
**Canonical:** Yes  
**Required context:** `600-quality-overview.md`  
**Related ADRs:** ADR-0042

---

## 1. Purpose

This document defines the layered test strategy for Mirae.

Tests verify contracts, failure behavior, compatibility, timing, and recovery.

---

## 2. Test Layers

### Unit tests

Pure logic, validation, transformations, migrations, parsers, scheduling math.

### Component tests

One subsystem with fake dependencies.

### Integration tests

Multiple real subsystems in one process or controlled process topology.

### Platform tests

Native adapters, permissions, devices, packaging, suspend/resume.

### End-to-end tests

User workflows across shell, UI, engine, and outputs.

### Performance tests

Budgets, throughput, latency, memory.

### Fault-injection tests

Crashes, disconnects, corruption, backpressure, device loss.

### Security tests

Fuzzing, permissions, archive extraction, IPC validation, secrets.

### Accessibility tests

Keyboard, semantics, focus, announcements, contrast, reduced motion.

---

## 3. Determinism

Tests should inject:

- clocks;
- random IDs;
- filesystem;
- network;
- platform capabilities;
- devices;
- renderer;
- audio backend;
- encoder;
- IPC transport.

A test that depends on wall-clock sleep should be exceptional.

---

## 4. Fixtures

Fixture categories:

- project schemas;
- migrations;
- scene graphs;
- media packets;
- color charts;
- shader graphs;
- malformed IPC;
- device snapshots;
- support bundles;
- crash metadata;
- extension manifests.

Fixtures are versioned and documented.

---

## 5. Golden Tests

Suitable for:

- canonical project serialization;
- render-plan dumps;
- render-graph dumps;
- UI accessibility trees where stable;
- color outputs under controlled renderer;
- migration reports;
- diagnostic bundles.

Golden updates require human review.

---

## 6. Fakes Versus Mocks

Prefer:

- in-memory implementations;
- deterministic fakes;
- protocol fixtures;
- real parsers with controlled data.

Avoid brittle call-order mocks unless the order itself is the contract.

---

## 7. Platform Matrix

At minimum test supported combinations of:

- Windows versions/builds;
- macOS versions and hardware classes;
- Linux Wayland/X11 and packaging modes;
- GPU vendors;
- audio backends;
- hardware/software encoders;
- light/dark UI modes;
- supported locales.

The compatibility document owns exact matrix.

---

## 8. Flaky Tests

A flaky test is a defect.

Required response:

- quarantine only with owner and deadline;
- preserve failure evidence;
- fix timing/environment dependency;
- do not simply add large sleeps;
- track flake rate.

---

## 9. Test Data Privacy

Real user projects or recordings require:

- consent;
- anonymization;
- secure storage;
- restricted access;
- deletion policy.

Synthetic fixtures are preferred.

---

## 10. Coverage

Coverage is a signal, not a goal by itself.

Critical requirements:

- error paths;
- recovery;
- migrations;
- permissions;
- concurrency;
- output isolation;
- security bounds.

---

## 11. Invariants

1. Critical contracts have automated tests.
2. Clocks and external dependencies are injectable.
3. Fixtures are versioned.
4. Golden updates are reviewed.
5. Flaky tests are tracked.
6. Real user data is protected.
7. Recovery paths are tested.
8. Platform adapters have native tests.
9. Performance and fault tests complement functional tests.
10. Test success does not replace spec compliance review.

---

## 12. Required CI Suites

- formatting/lint;
- unit;
- component;
- schema/migration;
- IPC compatibility;
- security parser/fuzz smoke;
- UI tests;
- platform smoke;
- benchmark smoke;
- documentation links;
- dependency/license checks.

---

## 13. AI Implementation Notes

Do not add sleeps as the default synchronization mechanism in tests.

Do not update golden files without explaining the semantic change.

For every bug fix, add a regression test reproducing the failure when practical.

Use deterministic fakes before brittle mocks.

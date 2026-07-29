# 002 — Product and System Boundaries

**Status:** Proposed  
**Audience:** Product, architecture, engine, UI, SDK contributors  
**Canonical:** Yes  
**Required context:** `001-project-overview.md`

---

## 1. Purpose

This document defines the boundaries of Mirae so that implementation does not drift into unrelated product categories or place responsibilities in the wrong layer.

A boundary is both a product decision and an architectural constraint.

---

## 2. Core Product Boundary

Mirae is a live-production application.

It owns:

- preparation of production scenes;
- live switching and transitions;
- real-time composition;
- audio routing and mixing;
- live capture;
- recording;
- streaming;
- replay;
- production diagnostics;
- local project management.

Mirae does not become a general-purpose nonlinear video editor, digital audio workstation, social network, cloud asset manager, or browser.

---

## 3. Included Capabilities

The following are in scope.

### 3.1 Live scene composition

- source placement and transformation;
- clipping, masks, filters, effects, and transitions;
- nested composition with bounded recursion;
- preview/program workflows;
- direct scene operation for simpler users.

### 3.2 Real-time media operation

- live capture;
- media playback;
- real-time decode and conversion;
- frame scheduling;
- audio synchronization;
- encoding;
- network output.

### 3.3 Production control

- keyboard shortcuts;
- command palette;
- stream deck or controller integration through supported APIs;
- macros and automation with explicit permissions;
- operator-safe confirmation for destructive actions.

### 3.4 Local projects

- portable project representation;
- referenced or managed assets;
- project migration;
- autosave;
- crash recovery;
- templates.

### 3.5 Diagnostics

- source health;
- encoder health;
- dropped frame classification;
- network state;
- audio clipping and timing;
- GPU timing;
- memory pressure;
- crash reports controlled by user privacy settings.

---

## 4. Explicit Non-Goals

### 4.1 Full nonlinear editing

Mirae MAY provide trimming for replay export or simple recording utilities, but it MUST NOT grow into a timeline-based editor with unrestricted offline compositing.

Reason:

- editing and live production have different state, caching, UI, and rendering models;
- forcing both into one architecture would weaken real-time guarantees.

### 4.2 General browser runtime

Browser sources MAY exist, but Mirae MUST NOT make arbitrary web content the control plane or trust boundary for the application.

Browser-source rendering must remain isolated and resource-bounded.

### 4.3 Mandatory cloud dependency

Cloud services MAY add collaboration, backup, remote control, or account synchronization later. Core production MUST NOT require them.

### 4.4 Hidden automatic production decisions

Mirae MUST NOT automatically change scenes, bitrate, audio routing, or source visibility without:

- explicit configuration;
- visible state;
- reversible behavior;
- clear diagnostics.

### 4.5 Unrestricted in-process plugins

The critical engine MUST NOT load arbitrary third-party native code into the render or audio process by default.

### 4.6 Marketplace-first architecture

The extension system is an integration surface, not the primary business model or product identity.

---

## 5. Layer Boundaries

### 5.1 Control UI

The UI owns:

- presentation;
- input interpretation;
- optimistic local interaction state when safe;
- view models;
- accessibility behavior;
- command construction;
- display of engine state and diagnostics.

The UI does not own:

- authoritative project state;
- media timing;
- capture lifetimes;
- GPU resource lifetimes;
- encoder state;
- project persistence;
- output retries.

### 5.2 Desktop shell

The shell owns:

- top-level windows;
- native menus;
- application lifecycle;
- webview creation;
- tray behavior;
- deep links;
- OS-level file open events;
- shell-to-engine startup coordination.

The shell does not implement domain logic.

### 5.3 Engine runtime

The engine owns:

- authoritative state;
- command validation;
- transactions;
- scene state;
- runtime service coordination;
- frame scheduling;
- diagnostics aggregation;
- project services;
- output orchestration.

### 5.4 Platform layer

The platform layer owns:

- native capture APIs;
- device discovery;
- hardware encoders;
- permissions;
- secure storage;
- window-system integration;
- platform packaging hooks.

Platform code MUST expose domain-oriented interfaces rather than raw OS APIs.

### 5.5 Extension host

The extension host owns:

- extension discovery;
- manifest validation;
- permission checks;
- lifecycle;
- sandboxing;
- API adaptation;
- failure isolation.

Extensions do not receive unrestricted engine memory access.

---

## 6. Process Boundaries

The architecture distinguishes logical boundaries from process boundaries.

A logical subsystem MAY initially share a process with another subsystem, but its interface and ownership must remain explicit.

The following boundaries are expected to become process boundaries:

- UI/shell versus engine;
- extensions versus engine;
- crash handler versus application;
- updater versus running application;
- unstable or privilege-sensitive media workers where isolation provides measurable value.

A process boundary MUST NOT be added solely to create architectural appearance. It must have a fault, security, privilege, or lifecycle reason.

---

## 7. Data Boundaries

### 7.1 Persisted data

Persisted data includes:

- project configuration;
- scene definitions;
- source configuration;
- output profiles;
- user preferences;
- extension configuration;
- references to managed or external assets.

Persisted data excludes:

- live GPU handles;
- active process IDs;
- mutexes;
- thread identifiers;
- native window handles;
- temporary decoder objects;
- active network sockets.

### 7.2 Credentials

Credentials are referenced by identifier in project or configuration data and stored in the operating system's secure credential facility.

Credentials MUST NOT be stored in plain project JSON or logs.

### 7.3 Diagnostics

Diagnostics are structured local events.

Telemetry upload, if introduced, MUST be opt-in or governed by an explicit privacy policy and MUST exclude credentials and media content.

---

## 8. Dependency Direction

The canonical dependency direction is:

```text
UI and adapters
      ↓
Application services
      ↓
Domain model
      ↓
Core primitives
```

Platform and infrastructure implement interfaces defined inward.

The domain model MUST NOT depend on:

- React;
- webview APIs;
- FFmpeg types;
- `wgpu` types;
- Windows, macOS, or Linux API types;
- database-specific objects;
- plugin implementation types.

---

## 9. Boundary Tests

The repository SHOULD include automated checks for:

- forbidden crate dependencies;
- schema validation;
- IPC compatibility;
- project files containing no runtime-only types;
- no credentials in serialized project fixtures;
- extension capability enforcement;
- platform module isolation.

---

## 10. Acceptance Criteria

This boundary specification is satisfied when:

- every major behavior has one authoritative owner;
- UI code can be replaced without rewriting engine state;
- platform backends can vary without changing domain semantics;
- project files are portable across supported platforms;
- extension failure cannot directly corrupt engine memory;
- live production remains usable without cloud connectivity.

---

## 11. AI Implementation Notes

Do not solve a cross-layer problem by importing an outer-layer type into the domain.

Do not place project persistence in React state.

Do not put platform conditionals throughout engine crates; add or extend a platform interface.

Do not serialize runtime implementation objects.

When an early implementation cannot support the final process split, maintain the boundary through traits, messages, or adapters so later extraction does not require a domain rewrite.

# 501 — Desktop Shell

**Status:** Proposed  
**Audience:** Shell, UI, runtime, platform contributors  
**Canonical:** Yes  
**Required context:** `01-runtime/101-process-model.md`, `500-platform-overview.md`  
**Related ADRs:** ADR-0037

---

## 1. Purpose

The desktop shell hosts Mirae's native application lifecycle and control UI.

It is intentionally thin. The shell does not become the media engine or project-state owner.

---

## 2. Responsibilities

The shell owns:

- native process entry point;
- top-level windows;
- embedded control UI;
- menus and tray;
- single-instance activation policy;
- file-open events;
- deep links;
- engine process launch and supervision;
- startup and fatal-error UI;
- native drag-and-drop handoff;
- update restart coordination;
- operating-system quit/session requests.

---

## 3. UI Hosting

The shell may embed a web-based control UI through a native webview.

Requirements:

- local packaged UI resources;
- no remote page as application shell;
- strict navigation policy;
- restricted webview permissions;
- typed bridge to shell;
- typed IPC to engine;
- no direct access to credentials;
- no media engine execution in JavaScript.

---

## 4. Navigation Security

The webview must:

- block arbitrary top-level navigation;
- open approved external links through the OS browser;
- reject custom schemes not explicitly registered;
- disable unnecessary remote debugging in production;
- use a restrictive content security policy;
- prevent untrusted browser-source content from sharing the control UI context.

---

## 5. Window Roles

Initial roles:

- main control window;
- projector/fullscreen preview;
- detached panels where supported;
- startup/recovery window;
- fatal error/report window;
- extension-owned constrained UI surfaces.

Each window has:

- stable role;
- owner;
- persisted geometry policy;
- monitor mapping;
- DPI/scale behavior;
- close behavior;
- accessibility semantics.

---

## 6. Engine Supervision

The shell:

1. creates ephemeral launch credential;
2. launches engine;
3. waits for authenticated readiness;
4. creates or activates UI;
5. reports engine crash;
6. supports bounded restart;
7. coordinates final shutdown.

The shell must not fabricate engine state while disconnected.

---

## 7. Single-Instance Behavior

The shell may enforce one application shell per user session.

A second invocation can forward:

- file open;
- project bundle import;
- deep link;
- focus request.

Project write ownership remains governed by project locks, not shell single-instance behavior alone.

---

## 8. Native Menus and Commands

Native menu items map to typed commands or UI routes.

Menu enabled state derives from engine/UI state projection.

A menu action must not mutate project state directly inside shell code.

---

## 9. Drag and Drop

Dropped content is classified:

- project;
- bundle;
- media asset;
- extension package;
- unsupported file.

The shell passes validated local references to the appropriate command flow.

It does not automatically execute or install dropped content.

---

## 10. Crash and Recovery UI

When engine is unavailable, shell may display:

- restart;
- open diagnostics;
- recover project;
- continue without reopening output;
- exit.

It must clearly distinguish UI failure from engine failure.

---

## 11. Invariants

1. Shell does not own authoritative project state.
2. UI resources are local and packaged.
3. Arbitrary navigation is blocked.
4. Engine supervision uses authenticated IPC.
5. Native menus route through commands.
6. Browser-source content is isolated from control UI.
7. Dropped files are treated as untrusted.
8. Shell restart does not imply output restart.
9. Window geometry is platform-aware.
10. Single-instance policy does not replace project locks.

---

## 12. Required Tests

- engine startup;
- engine crash;
- UI reload;
- navigation block;
- external link;
- second-instance forwarding;
- project drop;
- extension-package drop;
- menu command;
- multi-monitor window restore;
- update restart;
- fatal recovery flow.

---

## 13. AI Implementation Notes

Do not put source capture or project persistence in the shell.

Do not let the webview navigate to arbitrary remote URLs.

Do not treat the shell process as authoritative if the engine disconnects.

Keep the bridge narrow, typed, and permission-aware.

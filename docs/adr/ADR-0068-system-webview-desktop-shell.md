# ADR-0068 — System Webview for the Desktop Shell, Not a Bundled Runtime

**Status:** Accepted  
**Date:** 2026-07-30  
**Related:** ADR-0037 (native shell, replaceable web control UI), ADR-0003 (local-first)

---

## Context

`05-platform/501-desktop-shell.md` section 3 requires the shell to embed the
control UI through a native webview, over locally packaged resources, with a strict
navigation policy, restricted permissions, and a typed bridge.

`MIR-0010` built the shell process: it launches, supervises, and cleanly stops the
engine. It has no window, because the hosting technology was undecided.

`DEPENDENCY_VERSIONS.md` section 14 lists **Electron** and **Tauri** as not
approved, and the project owner has confirmed that constraint. Any other option is
a new Rust dependency and must clear section 11 first.

---

## Decision

The shell hosts the control UI in the **operating system's own webview**, driven
from Rust:

- **Windows:** WebView2 (Chromium, shipped and serviced by the OS vendor);
- **macOS:** WKWebView;
- **Linux:** WebKitGTK.

The binding layer is the `wry` webview crate with its `tao` windowing companion,
both MIT and Apache-2.0 licensed. They are bindings over the platform webview, not
a bundled browser engine and not an application framework.

The UI is loaded from **locally packaged resources through a custom protocol
handler**, never from `http://localhost` and never from a remote origin.

---

## Consequences

### Positive

- **No bundled browser runtime.** Installer size and memory stay close to the
  native process model in `101-process-model.md`, and security updates to the
  webview arrive through the operating system.
- **Not an application framework.** `wry` provides a window and a webview.
  Application lifecycle, engine supervision, and IPC stay in Mirae's own code,
  which is what ADR-0037 requires and what Electron and Tauri would blur.
- **The control UI is unchanged.** It already builds to static assets, so the same
  bundle serves the dev server and the packaged shell. `ADR-0037`'s replaceable UI
  stays replaceable.
- **A custom protocol makes the navigation policy enforceable.** Resources resolve
  from the package, so blocking arbitrary top-level navigation and rejecting
  unregistered schemes is a decision at one boundary rather than a filter.

### Negative

- **Three engines to test.** WebView2, WKWebView, and WebKitGTK differ, so the UI
  needs a compatibility baseline and platform smoke tests. This is the cost of not
  bundling one engine, and it is why `615-compatibility-policy.md` exists.
- **WebView2 is a runtime prerequisite on Windows.** Present on current Windows 11
  and serviced by the vendor; packaging must detect its absence and guide the user
  rather than fail opaquely.
- **New Rust dependencies.** `wry` and `tao` and their transitive graph must clear
  `DEPENDENCY_VERSIONS.md` section 11: justification, exact pins, committed
  `Cargo.lock`, a Rust dependency section, and license and security review. The
  graph is larger than any dependency added so far and deserves a real review, not
  a rubber stamp.
- **The bridge is a security boundary.** Anything the webview can call, a
  compromised page can call. The bridge stays narrow, typed, and permission-aware
  (`501` section 13), and credentials never cross it (`501` invariant 4).

---

## Alternatives Considered

### Electron

Rejected by `DEPENDENCY_VERSIONS.md` section 14 and by the project owner. It
bundles a browser and a Node runtime per application, owns the application
lifecycle, and would put a second runtime beside the Rust engine.

### Tauri

Rejected by the same section. Technically closer to this decision, since it also
uses the system webview, but it is an application framework with its own lifecycle,
command system, and plugin model, which would compete with the engine and the
command system Mirae already specifies.

### A native UI toolkit per platform

Rejected because the visual system in `09-ui-ux` is defined once as a web design
system with tokens, and ADR-0037 already chose a replaceable web control UI.
Reimplementing it three times contradicts both.

### No window: keep the browser dev server

Rejected as a product. It is the current state only because the decision was
pending, and it fails `501` invariant 2, which requires local packaged resources
rather than a served origin.

---

## Implementation Notes

- The window is created by the shell, which already owns supervision, so a webview
  failure is distinguishable from an engine failure (`501` section 10).
- Navigation policy, permission restrictions, and the content security policy are
  part of the implementing ticket, not follow-ups: `501` invariant 3 is a
  requirement, not a nicety.
- The typed bridge carries the IPC handshake from ADR-0067; the webview never
  reaches the engine socket directly.

---

## Related Specifications

- `05-platform/501-desktop-shell.md`
- `09-ui-ux/911-ui-library-decisions.md`
- `DEPENDENCY_VERSIONS.md` sections 11 and 14

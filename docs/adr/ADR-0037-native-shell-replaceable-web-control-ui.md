# ADR-0037 — Native Shell with Replaceable Web Control UI

**Status:** Proposed  
**Date:** 2026-07-29

---

## Context

Mirae needs a modern, fast-to-iterate control interface while preserving native engine performance, process isolation, and OS integration.

---

## Decision

Mirae will use a thin native desktop shell that hosts a local React/TypeScript control UI.

The UI remains replaceable and communicates with the engine through typed IPC.

The webview does not become the media engine.

---

## Consequences

### Positive

- productive UI development;
- modern component ecosystem;
- native shell integration;
- engine/UI process separation;
- UI reload without redesigning engine.

### Negative

- webview memory cost;
- bridge and CSP security;
- accessibility and platform behavior require care;
- UI/native styling differences.

---

## Alternatives Considered

### Electron as the entire application runtime

Rejected because it would make the browser runtime the application architecture.

### Fully native UI toolkit initially

Rejected because development cost and cross-platform UI consistency would be higher for the current team.

---

## Related Specifications

- `05-platform/501-desktop-shell.md`
- `01-runtime/109-ui-engine-synchronization.md`

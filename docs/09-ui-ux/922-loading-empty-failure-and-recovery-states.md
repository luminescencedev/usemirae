# 922 — Loading Empty Failure and Recovery States

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## Empty

Empty states explain the next meaningful action without decorative illustrations dominating the tool.

Examples:

- no project;
- no scene;
- no source;
- no output destination;
- no diagnostics incident.

## Loading

Use skeletons for structural content, progress for bounded operations, and explicit step/status for project open, migration, device initialization, and update.

## Failure

A failure state communicates:

- what failed;
- what still works;
- automatic recovery;
- next user action;
- diagnostic reference.

## Recovery

Recovery is visible as a state, not hidden spinner behavior. Repeated attempts show backoff and allow stop/diagnostics where safe.

## Toasts

Toasts are for transient confirmation. Persistent or operationally important problems use inline incidents or banners.

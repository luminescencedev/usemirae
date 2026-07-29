# 921 — Output Live and Recording Experience

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## Primary Actions

Record and Go Live are visible in the titlebar but separated by color, icon, label, and state.

## States

- idle;
- preparing;
- connecting;
- live/recording;
- reconnecting;
- stopping/finalizing;
- degraded;
- failed;
- recovered.

## Destination Cards

Each output shows:

- service/type;
- connection state;
- elapsed time;
- bitrate/health;
- dropped frames;
- retry state;
- local impact;
- stop/retry/diagnostics actions.

## Isolation

One failed destination does not visually imply that all outputs failed. Recording, streaming destinations, and replay have independent states.

## Confirmation

Stopping an active output displays exact destination and whether finalization is still in progress. Force stop is a secondary destructive action.

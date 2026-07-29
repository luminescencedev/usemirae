# 920 — Audio Mixer Experience

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## Channel Anatomy

- name and source;
- stereo/mono meter;
- fader;
- numeric gain;
- mute;
- solo;
- cue/monitor;
- route summary;
- clipping/device status;
- optional processing affordance.

## Meter Behavior

Meters update through a dedicated high-frequency store and imperative rendering where appropriate. They do not rerender the whole channel tree.

Color ranges use success, warning, and live/error with accessible text/peak indicators.

## Interaction

- pointer fader;
- keyboard arrows;
- numeric entry;
- reset to unity;
- multi-select grouping later;
- mute/solo always visible;
- cue state cannot be confused with Program routing.

## Layout

The bottom dock supports compact channels, horizontal scrolling, resizing, and a dedicated Audio workspace for advanced routing.

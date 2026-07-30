# Mirae Visual Validation

Record the following evidence in the ticket completion report.

## Rendered environments

- Browser control UI URL and viewport.
- Native Wry/WebView2 shell when relevant.
- Reference viewport: 1440 x 900 logical pixels.
- Narrow desktop viewport used.

## States observed

List each relevant state actually rendered, for example:

- initial loading;
- no project;
- connected engine;
- reconnecting engine;
- engine unavailable;
- successful project creation;
- validation or bridge error.

## Checks

- no unexpected clipping or horizontal overflow;
- correct focus order and focus return;
- keyboard access to every pointer action;
- sufficient contrast and redundant status communication;
- disabled and unavailable states remain understandable;
- reduced motion does not remove meaning;
- no console errors or unhandled rejections;
- visual hierarchy matches Obsidian Precision.

## Report format

```text
Visual validation:
- 1440 x 900: pass/fail plus observation
- narrow desktop: pass/fail plus observation
- keyboard: pass/fail plus paths exercised
- states rendered: ...
- native shell: pass/not applicable plus reason
- remaining visual debt: ...
```

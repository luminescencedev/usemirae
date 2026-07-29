# 905 — Design System Foundations

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## 1. Philosophy

```text
Headless behavior + Mirae visual language.
```

Mirae owns every visible component. A primitive library may own focus, keyboard, positioning, and accessibility behavior, but never the final aesthetic.

## 2. Layers

1. raw semantic tokens;
2. component tokens;
3. headless primitives;
4. Mirae primitives;
5. production-specific components;
6. feature compositions;
7. screens/workspaces.

## 3. Primitive Components

- Button, IconButton, ToggleButton;
- TextField, NumberField, Select, Combobox;
- Slider, Scrubber, Checkbox, Switch;
- Tooltip, Popover, Menu, ContextMenu;
- Dialog, AlertDialog, Sheet;
- Tabs, SegmentedControl;
- ScrollArea, VirtualList;
- Badge, Status, Meter, Progress;
- Panel, Dock, ResizeHandle;
- Toast and IncidentBanner.

## 4. Production Components

- SceneTree;
- SourceRow;
- PropertyInspector;
- TransformFields;
- CanvasToolbar;
- PreviewSurface;
- ProgramSurface;
- MixerChannel;
- OutputDestination;
- LiveStatus;
- CommandPalette;
- HealthPanel.

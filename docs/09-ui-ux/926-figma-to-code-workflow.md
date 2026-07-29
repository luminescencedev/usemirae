# 926 — Figma to Code Workflow

**Status:** Proposed  
**Audience:** Product design, UI engineering, desktop-shell, QA, and coding agents  
**Canonical:** Yes

---

## 1. One Token Language

Figma variables mirror semantic token names. Code generation or validation compares design tokens with repository tokens.

## 2. Component Parity

Figma and code components share:

- name;
- variants;
- sizes;
- states;
- property terminology;
- accessibility notes.

## 3. Screen Handoff

A screen handoff includes:

- target viewport;
- panel constraints;
- component instances;
- state shown;
- interaction notes;
- motion notes;
- keyboard behavior;
- data ownership;
- failure/empty/loading variants.

## 4. Priority

```text
approved Figma screen
→ canonical UI/UX docs
→ token/component contracts
→ implementation
→ generic library behavior
```

A one-off mockup cannot silently redefine the whole system.

## 5. Review Loop

Design review uses real local builds, not only static screenshots. Differences become token/component fixes before screen-specific hacks.

# 614 — Localization

**Status:** Proposed  
**Audience:** UI, product, support, release contributors  
**Canonical:** Yes  
**Required context:** `613-accessibility.md`, `05-platform/501-desktop-shell.md`

---

## 1. Purpose

This document defines localization architecture for UI text, errors, dates, numbers, file sizes, shortcuts, and support content.

---

## 2. Locale Scope

Locale affects:

- interface strings;
- pluralization;
- date/time display;
- numbers;
- percentages;
- file sizes;
- list formatting;
- keyboard shortcut display;
- sorting/collation;
- accessible names;
- notifications.

Project schema and machine-readable error codes remain locale-independent.

---

## 3. Message Keys

User-facing text uses stable message keys.

Avoid using English source strings as identifiers when long-term compatibility matters.

Messages support:

- variables;
- plurals;
- select/gender where required;
- rich text through safe structured placeholders;
- translator context.

---

## 4. No String Concatenation

Do not construct sentences from fragments.

Bad:

```text
"Recording " + status
```

Good:

```text
recording.status.running
recording.status.failed
```

This preserves grammar and accessibility.

---

## 5. Dates and Numbers

Use locale-aware formatting.

Persisted values remain canonical:

- RFC or explicit timestamps;
- rational rates;
- bytes;
- durations.

Formatting occurs in presentation layer.

---

## 6. Technical Terms

Canonical product terms may remain untranslated or use approved glossary.

The terminology document and localization glossary must agree.

---

## 7. Fallback

Fallback chain:

1. exact locale;
2. language locale;
3. default supported locale;
4. visible missing-key marker in development;
5. safe English fallback in production if necessary.

Missing keys are logged without user data.

---

## 8. Layout

UI must tolerate:

- text expansion;
- right-to-left layout;
- CJK fonts;
- long error messages;
- variable shortcut labels;
- locale-specific punctuation;
- non-Latin search.

Hard-coded widths are avoided for text controls.

---

## 9. Project Content

User-authored project names, scene names, and text sources are not translated.

Built-in template content may be localized by template variant.

---

## 10. Error Localization

Error code remains stable.

User-safe message is localized at presentation boundary using code and structured fields.

Raw platform errors are not shown directly.

---

## 11. Invariants

1. Machine contracts are locale-independent.
2. UI text uses stable keys.
3. Sentences are not assembled from fragments.
4. Formatting is locale-aware.
5. User content is not translated.
6. RTL is supported structurally.
7. Missing-key fallback is safe.
8. Accessibility labels are localized.
9. Error codes remain stable.
10. Layout supports expansion.

---

## 12. Required Tests

- pluralization;
- date/number formatting;
- long French/German text;
- RTL layout;
- CJK font fallback;
- missing key;
- error localization;
- shortcut labels;
- notification localization;
- 200% zoom with expanded text;
- user content preservation;
- locale switch.

---

## 13. AI Implementation Notes

Do not hardcode user-facing English strings in components.

Do not concatenate translated sentence fragments.

Keep error codes and project schema values locale-independent.

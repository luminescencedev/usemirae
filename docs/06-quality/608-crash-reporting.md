# 608 — Crash Reporting

**Status:** Proposed  
**Audience:** Runtime, platform, security, support contributors  
**Canonical:** Yes  
**Required context:** `01-runtime/101-process-model.md`, `606-logging-and-tracing.md`, `617-privacy-and-telemetry.md`

---

## 1. Purpose

Crash reporting captures enough information to diagnose process failures while protecting user privacy and preserving recovery data.

---

## 2. Monitored Processes

- shell/UI;
- engine;
- extension host;
- media/device workers;
- updater where supported.

The crash handler runs outside the monitored process.

---

## 3. Crash Artifact

May include:

- process role;
- build ID;
- platform;
- stack/minidump;
- thread list;
- engine session ID;
- lifecycle state;
- active project ID only as local opaque ID;
- active output/source IDs;
- recent bounded logs;
- active workarounds;
- capability summary;
- memory/exception metadata.

---

## 4. Excluded Data

By default exclude:

- credentials;
- raw media frames/audio;
- project content;
- stream keys;
- signed URLs;
- browser cookies;
- private full paths;
- extension secrets.

---

## 5. Local-First Storage

Crash artifacts are stored locally first.

Uploading requires:

- explicit opt-in or user action;
- redaction;
- size bounds;
- consent context;
- secure transport;
- retention policy.

Core crash recovery does not depend on upload.

---

## 6. Crash Loop Detection

A crash loop is detected by:

- repeated process failure;
- same build;
- bounded time window;
- same startup phase or signature.

Response may include:

- safe mode;
- disable last extension;
- disable risky hardware path;
- skip project auto-open;
- offer rollback;
- show recovery UI.

Automatic action must be reversible and visible.

---

## 7. Symbolication

Release pipeline retains symbol mapping securely.

Symbols:

- match exact build;
- are not shipped unnecessarily;
- are accessible to authorized diagnostics pipeline;
- preserve source privacy.

---

## 8. Extension Crashes

Extension-host crash reporting includes:

- extension IDs;
- active calls;
- resource usage;
- last host events;
- sandbox state.

It must not expose unrelated extension private data.

---

## 9. Invariants

1. Crash handler is out of process.
2. Artifacts are local first.
3. Upload is opt-in or explicit.
4. Secrets and raw media are excluded.
5. Build identity is exact.
6. Crash loops trigger bounded safe response.
7. Recovery data is prioritized.
8. Extension crash context is scoped.
9. Artifact retention is bounded.
10. User can inspect/delete local reports.

---

## 10. Required Tests

- engine crash;
- shell crash;
- extension-host crash;
- worker crash;
- crash-loop safe mode;
- dump redaction;
- local report retention;
- upload consent;
- symbol/build mismatch;
- project auto-open suppression;
- report deletion;
- crash during shutdown.

---

## 11. AI Implementation Notes

Do not upload crash reports automatically by default.

Do not attach full project files or media.

Do not assume in-process panic handling replaces an external crash handler.

Keep safe-mode actions reversible and documented.

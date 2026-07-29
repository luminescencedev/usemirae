# 312 — Recording

**Status:** Proposed  
**Audience:** Media, output, persistence contributors  
**Canonical:** Yes  
**Required context:** `310-output-architecture.md`, future `04-project/403-persistence.md`  
**Related ADRs:** ADR-0021

---

## 1. Purpose

The recording subsystem writes encoded media to local storage with crash resilience, segmentation, diagnostics, and explicit finalization behavior.

---

## 2. Recording Profile

Includes:

- container;
- video encoder profile;
- audio tracks;
- destination directory reference;
- file naming template;
- overwrite policy;
- segmentation policy;
- split triggers;
- metadata;
- space safety thresholds;
- finalization policy.

---

## 3. Container Strategy

Containers differ in crash behavior.

The architecture should prefer modes that preserve recoverability.

The recording subsystem may use:

- fragmented containers;
- segmented files;
- periodic index updates;
- temporary file plus final rename;
- recovery metadata.

Container choice is user-visible when it affects compatibility.

---

## 4. File Lifecycle

```text
reserve path
→ create temporary/active file
→ write headers
→ write media
→ update recovery metadata
→ flush according to policy
→ finalize container
→ fsync/close where required
→ atomically publish final name
```

Exact OS guarantees are platform-specific.

---

## 5. Segmentation

Segments may split on:

- time;
- size;
- user marker;
- scene change;
- output reconfiguration;
- encoder restart;
- discontinuity;
- manual action.

Each segment begins on a decodable boundary where possible.

---

## 6. Naming

Naming templates may use:

- project name;
- profile name;
- local date/time;
- sequence number;
- scene name;
- marker;
- unique suffix.

Sanitization is platform-aware.

Name collision behavior is explicit.

---

## 7. Disk Space

Recording monitors:

- free space;
- estimated remaining duration;
- write errors;
- throughput;
- filesystem limits;
- quota;
- path availability.

Policies:

- warn;
- split to configured fallback location;
- stop recording safely;
- continue audio-only only if explicitly configured;
- never delete unrelated files automatically.

---

## 8. Write Path

Write path uses bounded buffering.

The muxer and disk sink report:

- queue depth;
- write latency;
- bytes written;
- flush duration;
- error category.

The recording path must not block renderer or audio callbacks.

---

## 9. Crash Recovery

Recovery data may include:

- active file path;
- container;
- last known segment;
- stream parameters;
- last durable timestamp;
- finalization state;
- encoder generation.

On next startup, Mirae may:

- finalize recoverable file;
- expose repair action;
- preserve partial file;
- mark unrecoverable with diagnostics.

It must not overwrite partial data.

---

## 10. Multitrack Audio

Recording supports explicit track mapping.

Each track declares:

- bus source;
- channel layout;
- codec;
- language/label metadata;
- inclusion policy.

Track order is stable and documented.

---

## 11. Markers and Chapters

Markers may be written:

- into container metadata when supported;
- to sidecar;
- to project recording index.

Marker submission is timestamped on the master timeline.

---

## 12. Invariants

1. Recording does not block real-time callbacks.
2. Buffers are bounded.
3. Finalization state is explicit.
4. Partial data is preserved after crash.
5. Segments start on valid boundaries where possible.
6. File collisions are explicit.
7. Disk-full behavior is deterministic.
8. Credentials are irrelevant to local path storage.
9. Track mapping is explicit.
10. Active file becomes final through defined publication step.

---

## 13. Required Tests

- normal finalize;
- crash before finalize;
- disk full;
- path unavailable;
- segment by time;
- segment on encoder restart;
- name collision;
- multitrack mapping;
- marker timestamp;
- slow disk;
- recovery repair;
- forced shutdown.

---

## 14. AI Implementation Notes

Do not write directly to the final filename without a defined recovery strategy.

Do not buffer unlimited media when disk is slow.

Do not delete partial recordings on startup.

Keep container finalization and file publication separate.

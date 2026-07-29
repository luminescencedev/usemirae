# 409 — Project Locking and Multi-Instance

**Status:** Proposed  
**Audience:** Project, runtime, shell, platform contributors  
**Canonical:** Yes  
**Required context:** `403-persistence.md`, `407-local-file-system.md`  
**Related ADRs:** ADR-0029

---

## 1. Purpose

This document prevents concurrent writers from silently overwriting one project while allowing safe read-only access and recovery.

---

## 2. Lock Model

One project has at most one active writer.

Other processes may:

- open read-only;
- request handoff;
- open a duplicate copy;
- inspect metadata;
- wait for lock;
- recover after verified stale lock.

---

## 3. Lock Record

A lock record includes:

- project ID;
- file identity;
- owning engine session ID;
- process ID as advisory metadata;
- host identity;
- user identity where safe;
- creation time;
- heartbeat/lease time;
- protocol version;
- random lock token.

PID alone is insufficient.

---

## 4. Lock Acquisition

1. resolve canonical project location;
2. check existing lock;
3. validate owner liveness through local coordination if possible;
4. atomically create/claim lock;
5. verify ownership token;
6. open project for write;
7. renew lease/heartbeat.

Network filesystems may require reduced-guarantee mode.

---

## 5. Stale Locks

A lock may be stale when:

- owner process is absent;
- engine session cannot be contacted;
- heartbeat expired;
- host restarted;
- lock file exists without matching project identity.

Breaking a stale lock requires:

- revalidation;
- user-visible explanation when uncertain;
- preservation of recovery data;
- audit diagnostic;
- new random token.

---

## 6. Handoff

If the owning instance is reachable, another instance may request:

- focus existing project window;
- save and close;
- transfer ownership after close;
- open read-only.

Ownership is never transferred by merely copying the lock record.

---

## 7. Read-Only Mode

Read-only open:

- disables project mutation commands;
- permits inspection and export where safe;
- does not create autosave that could be mistaken for writable recovery;
- may allow “Save As” to new project identity/location;
- continues to resolve assets without modifying them.

---

## 8. External Tools

Projects edited by unsupported external tools are treated as external modification.

Mirae does not assume lock compliance by other applications.

Expected file identity checks remain mandatory on save.

---

## 9. Multiple Projects

One engine may own multiple project locks only if multi-project architecture explicitly supports it.

Initial canonical behavior assumes one active writable project per engine session.

---

## 10. Invariants

1. One writer per project identity/location.
2. PID alone does not prove ownership.
3. Lock token is random and session-scoped.
4. Stale-lock break is explicit.
5. Read-only mode cannot mutate canonical project.
6. Save still checks external modification.
7. Handoff requires owner cooperation or verified stale state.
8. Recovery data is preserved before break.
9. Network-storage limitations are visible.
10. Lock records contain no secrets.

---

## 11. Required Tests

- acquire/release;
- second writer rejected;
- read-only open;
- owner crash;
- stale lock;
- PID reuse;
- handoff;
- network storage degraded mode;
- save identity conflict;
- save-as from read-only;
- lock token mismatch;
- cleanup after engine stop.

---

## 12. AI Implementation Notes

Do not use “lock file exists” as the only liveness rule.

Do not break locks silently.

Do not let read-only mode write autosave into the canonical project's recovery lineage.

Keep expected-file-identity checks even with locks.

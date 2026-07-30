# ADR-0072 — Recovery Records Live in a Caller-Supplied Directory, Resolved Without a Dependency

**Status:** Accepted  
**Date:** 2026-07-30  
**Related:** ADR-0025 (autosave and recovery), ADR-0070 (state representation), ADR-0071 (project file encoding)

---

## Context

`04-project/404-autosave-and-recovery.md` section 2 requires autosave to write to
a recovery store **separate from the canonical project file**, and invariant 1
forbids autosave from ever writing over that file. It does not say where the
store lives, because that is a platform question.

`MIR-0112` builds the store, so the question has to be answered. Two things are
actually being decided: which directory holds the records, and which layer works
that directory out.

`08-development/804-dependency-rules.md` section 3 forbids `domain → platform`
and section 4 puts an interface with the layer that needs the behaviour, with the
implementation living outward. `mirae-project` needs *a directory*. It does not
need to know that Windows calls it `%LOCALAPPDATA%`.

---

## Decision

Two parts.

**The store takes a root directory as a parameter.** `mirae-project` never asks
the operating system anything. It is handed a path and owns everything below it:
layout, naming, retention, integrity.

**Resolving that path is platform work, done from environment variables, with no
new dependency.** `mirae-platform` gains a small resolver:

| Platform | Directory |
|---|---|
| Windows | `%LOCALAPPDATA%\Mirae\recovery` |
| macOS | `$HOME/Library/Application Support/Mirae/recovery` |
| Linux | `$XDG_DATA_HOME/mirae/recovery`, else `$HOME/.local/share/mirae/recovery` |

Local application data rather than roaming, config, or cache. A recovery record
is machine-local working state: roaming would copy a half-finished project
between machines, config would mix user preferences with crash debris, and cache
invites the operating system to delete exactly the thing recovery depends on.

---

## Consequences

### Positive

- **The project layer stays testable.** Every test points the store at a
  temporary directory, and none of them depends on the machine's environment.
  That is the same property the state store has, for the same reason.
- **No dependency, and no MPL crate on Windows.** The `dirs` family reaches
  `option-ext`, which is MPL-2.0. It is already in the macOS and Linux graphs
  through `wry`, and `DEPENDENCY_VERSIONS.md` section 11 records it; adding it
  directly would extend that to Windows for four lines of environment lookup.
- **The rule is inspectable.** A user asking where their recovery data went gets
  a documented path rather than "wherever the crate decided", and a support
  bundle (`411` section 8) can name it.
- **A future ticket can override the path.** Because it is a parameter, a
  packaged build, a portable install, or a test harness can point it elsewhere
  without changing anything below.

### Negative

- **Four platform rules to maintain by hand.** They are stable — these
  conventions have not moved in a decade — but they are ours now, and a fifth
  platform means editing this rather than upgrading a crate.
- **No XDG edge cases.** The `dirs` crate handles `XDG_DATA_HOME` being relative
  or empty, and other corners of the specification. The resolver here treats
  anything not an absolute path as absent and falls back, which is stricter than
  the specification and is stated in the code.
- **An environment with none of these variables has no recovery store.** The
  resolver returns nothing rather than inventing a path, and autosave is then
  unavailable and says so. Writing recovery data into the current working
  directory would be worse: it would scatter files wherever the user happened to
  launch from.

---

## Alternatives Considered

### The `dirs` crate

Rejected, narrowly. It is well maintained and handles more edge cases than this
resolver will. It costs a dependency chain ending in an MPL-2.0 crate, on every
platform, to answer a question that is four environment variables — and the
project layer would still need the path passed in, so it saves nothing there.

### Beside the project file

Rejected. `404` invariant 1 keeps recovery separate from the canonical file, and
a directory the user syncs, shares, or opens in a file manager is the wrong place
for crash debris. It also fails when the project lives somewhere read-only, which
is exactly when recovery matters most.

### A temporary directory

Rejected. The operating system may clear it between runs, and recovery data whose
lifetime is "until something cleans up" does not recover anything.

### One database file rather than a directory of records

Rejected for now. It would make retention and atomic replacement someone else's
problem, at the cost of a substantial dependency and of losing the property that
a single corrupt record cannot take the others with it (`404` invariant 10).

---

## Related Specifications

- `04-project/404-autosave-and-recovery.md` sections 2, 5 and 8
- `08-development/804-dependency-rules.md` sections 3 and 4
- `05-platform/500-platform-overview.md`
- `DEPENDENCY_VERSIONS.md` section 11

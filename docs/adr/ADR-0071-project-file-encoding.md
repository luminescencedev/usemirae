# ADR-0071 — JSON as the Canonical Project File Encoding

**Status:** Accepted  
**Date:** 2026-07-30  
**Related:** ADR-0023 (project format), ADR-0067 (control-plane serialization), ADR-0069 (identifiers)

---

## Context

`04-project/401-project-format.md` section 2 says the canonical editable project
format is a versioned, schema-validated document, that the initial human-readable
representation SHOULD use JSON or another reviewable structured encoding, and
that the choice is made by an implementation ADR. `MIR-0107` defines the schema,
so the encoding has to be settled first.

ADR-0067 chose JSON for the **control plane**. That decision does not carry over
by itself. A control-plane message is machine-to-machine, short-lived, and never
read by a person; a project file is opened years later, diffed in a review,
merged by a user who has never heard of Mirae, and hashed for integrity. The
criteria are different, so the decision is made again rather than inherited.

---

## Decision

The canonical project file is **JSON**, serialized canonically.

Canonical serialization means, per `401` section 12:

- object keys sorted by Unicode code point;
- two-space indentation and `\n` line endings, so a diff is line-oriented;
- no insignificant whitespace beyond that indentation;
- integers written without a decimal point or exponent;
- non-integral numbers written with the shortest representation that round-trips;
- NaN and infinity rejected before serialization rather than encoded;
- strings in Unicode Normalization Form C;
- the `integrity.contentHash` field excluded from the bytes it hashes.

The file carries a `.mirae.json` extension.

---

## Consequences

### Positive

- **A project is reviewable.** It opens in any editor, diffs line by line, and a
  user can see what a change did. For a tool whose configuration represents
  hours of a person's setup, that is a durable property worth paying for.
- **Sorted keys make diffs mean something.** A save that changes one scene name
  produces one changed line. Without a canonical order, an unrelated hash-map
  reordering would rewrite the file and drown the real change.
- **No new dependency.** `serde_json` is already approved for the control plane,
  already reviewed, and already parses untrusted input in this codebase.
- **Integrity is straightforward.** Deterministic bytes make a content hash
  meaningful; `401` section 11 needs exactly that, and canonicalization is what
  makes the hash reproducible rather than incidental.
- **Migrations are inspectable.** A schema version bump can be diffed against a
  fixture, which is what `MIR-0115`'s compatibility corpus depends on.

### Negative

- **Larger and slower than a binary encoding.** A project file is written on
  save and read on open — not per frame — so the cost lands where there is no
  budget pressure. If a project ever grows large enough for this to matter, the
  answer is a separate asset store, not a binary re-encoding of the intent.
- **JSON has no integer/float distinction.** The schema carries the distinction
  instead: every numeric field declares its type and bounds, and `401` section 6
  already rejects NaN and infinity. Generated decoders enforce it on both sides.
- **No comments.** A user cannot annotate a project file. Neither could they in
  any binary format, and a `notes` field is a schema decision rather than an
  encoding one.
- **Canonicalization is a rule that must be enforced, not assumed.** Serializing
  with an ordinary encoder produces valid JSON that is not canonical. The writer
  is therefore a single implementation with fixture tests, and `MIR-0109` owns
  it.

---

## Alternatives Considered

### TOML

Rejected. Excellent for flat configuration and unpleasant for what a project
actually is: deep arrays of tables, nested per-item structure, and heterogeneous
collections. The readability advantage inverts as nesting grows, which is
precisely where a project file lives.

### A binary encoding — MessagePack, CBOR, bincode

Rejected for the canonical file. Not reviewable, not diffable, not repairable by
hand when something goes wrong, and the size and speed gains land on an operation
that happens on save rather than per frame. `401` section 2 asks for a reviewable
structured encoding, and this is not one.

### SQLite

Rejected. It answers a question this project does not yet have — partial loads
and incremental writes of a project too large for memory — and gives up
diffability, plain-text review, and a trivial merge story in exchange. It also
adds a substantial C dependency to the persistence path.

### RON

Rejected. Better typed than JSON and genuinely readable, but it is a Rust
ecosystem format: a project file is read by the TypeScript side too, and by
whatever tooling users write. That would mean a second parser implementation to
own.

### Inheriting ADR-0067 without deciding

Rejected as a process, even though it reaches the same encoding. The control
plane and the project file are judged against different criteria, and recording
the reasoning here is what lets a future ticket change one without reasoning
about the other.

---

## Related Specifications

- `04-project/401-project-format.md` sections 2, 6, 11 and 12
- `04-project/403-persistence.md` section 13
- `06-quality/615-compatibility-policy.md`
- `DEPENDENCY_VERSIONS.md` section 11

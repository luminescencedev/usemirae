# Project fixtures — schema v1

Introduced by `MIR-0115`. Canonical policy:
`docs/06-quality/615-compatibility-policy.md` and
`docs/04-project/408-schema-versioning-and-migrations.md` section 11.

## Do not edit these by hand

They are generated from `crates/project/project/src/fixtures.rs`. A fixture's
entire value is that nobody adjusts it: the moment one is edited to make a test
pass, it stops recording what the format *was* and starts recording what the code
currently does.

To change one deliberately:

```bash
MIRAE_UPDATE_FIXTURES=1 cargo test -p mirae-project
```

Then read the diff. It is the compatibility change, stated plainly — that is what
these files are for.

## What each one proves

| File | Proves |
|---|---|
| `empty.mirae.json` | The smallest project the format can express, and the canonical serialization of an envelope with no content. |
| `populated.mirae.json` | One of every entity the schema models. Its scene item order is deliberately *not* identifier order, so a change that sorts composition order shows up here rather than as a silently rearranged scene. |
| `boundaries.mirae.json` | The longest accepted name, in a script where characters and bytes differ. A bound expressed in the wrong unit fails against this. |

## What is missing, and why

There is no fixture from an older schema version. `v1` is the first, so there is
nothing to migrate from. A fake `v0` would test a migration nobody wrote against
a format that never shipped, and it would pass forever without proving anything.

The corpus is stored per schema version, so a future `v2` adds its own directory
and keeps this one as history.

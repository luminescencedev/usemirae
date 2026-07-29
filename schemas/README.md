# schemas/

Canonical machine contracts. One canonical schema per contract; schemas are never
duplicated in handwritten Rust or TypeScript definitions.

Canonical documentation:
[`805-generated-contracts-and-schemas.md`](../docs/08-development/805-generated-contracts-and-schemas.md),
ADR-0057.

## Layout

```text
<domain>/v<major>/<name>.schema.json    canonical source, hand-authored
generated/                              generated output, verified in CI
```

The canonical domains from `805` section 2, each with a major-version directory:

| Domain | Contract |
|---|---|
| `ipc/` | engine, shell, and UI cross-process protocol |
| `project/` | persisted project format |
| `bundle/` | portable project bundle format |
| `diagnostics/` | diagnostic and telemetry events |
| `sdk/` | public extension protocol |
| `extension-manifest/` | extension package manifest |
| `extension-ui/` | declarative extension UI |
| `compatibility/` | compatibility and workaround database |

`801-monorepo-architecture.md` lists a shorter set of directories as an example;
the eight domains above are the full list from the schema-canonical document.

## Rules

- one directory per major version: a breaking change adds `v2`, it does not edit
  `v1`;
- every schema declares `$id` as `mirae://<domain>/v<major>/<name>` and a `title`;
- an `$id` must match the directory that holds it, so a moved file cannot silently
  change the contract it claims to define;
- no two schemas share an `$id`;
- files under `generated/` are never edited by hand.

## Generation

```bash
cargo xtask generate          # validate schemas and write generated output
cargo xtask generate --check  # fail if generated output is stale
```

The pipeline validates every schema, rejects duplicate ids, renders deterministic
output sorted by id, then writes or verifies it. `--check` runs in CI, so drift
fails the build (`805` section 5).

No schemas are defined yet. `MIR-0006` adds the first contract: the engine and
shell protocol version and readiness messages. The generated files are still
written and verified today, so the drift gate exists before there is a contract to
drift.

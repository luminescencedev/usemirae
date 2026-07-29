# schemas/

Canonical machine contracts. One canonical schema per contract; schemas are never
duplicated in handwritten Rust or TypeScript definitions.

Canonical documentation:
[`805-generated-contracts-and-schemas.md`](../docs/08-development/805-generated-contracts-and-schemas.md),
ADR-0057.

## Layout

```text
<domain>/v<major>/<name>.schema.json    canonical source, hand-authored
generated/manifest.json                 generated index of every contract
```

Language bindings are generated into the crate and package that own them, so each
generated file has one declared owner (`805` invariant 4):

```text
crates/foundation/contracts/src/generated.rs   Rust bindings   (mirae-contracts)
packages/contracts/src/generated.ts            TypeScript      (@mirae/contracts)
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

## Supported schema subset

Deliberately narrow, so a schema cannot express something the generators would
silently drop:

- a top-level `object` with `"additionalProperties": false`;
- properties of type `integer` (with a `maximum`, optionally a `const`), `string`
  (with either `enum` or a `maxLength`), or `boolean`;
- `required` naming declared properties only;
- a `description` on the document and on every property, because generated code is
  documented.

Anything else is rejected with the property named. `$ref`, composition, and nested
objects are not supported yet; the first contract that needs one extends the
generator.

## Contracts today

| Id | Type | Purpose |
|---|---|---|
| `mirae://ipc/v1/protocol-version` | `ProtocolVersion` | negotiated major and minor version |
| `mirae://ipc/v1/engine-readiness` | `EngineReadiness` | engine lifecycle state, session id, safe detail |

Both come from `docs/01-runtime/108-ipc-protocol.md` sections 6 and 8. Generated
string bounds are exported as constants so decoding can reject oversized input
before allocating for it (section 9).

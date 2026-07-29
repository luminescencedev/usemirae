# schemas/

Canonical machine contracts. One canonical location per contract; schemas are
never duplicated in handwritten Rust or TypeScript definitions.

```text
ipc/          engine/shell/UI cross-process contracts
project/      persisted project format
sdk/          public extension contracts
diagnostics/  diagnostic and telemetry event contracts
generated/    generated output, verified in CI — never hand-edited
```

Created empty by `MIR-0001`. Populated by `MIR-0005 — Create canonical schema
skeleton` and `MIR-0006 — Generate Rust and TypeScript handshake contracts`.

Canonical documentation: `docs/08-development/805-generated-contracts-and-schemas.md`,
ADR-0057.

# tools/

Repository automation. Shell scripts call `xtask` rather than duplicating logic.

```text
xtask/     the single repository automation entry point (cargo xtask ...)
codegen/   schema-to-code generation
fixtures/  fixture generation
release/   packaging and release manifests
```

Created empty by `MIR-0001`. `xtask` is implemented by `MIR-0003 — Add xtask`,
which also adds it to the Cargo workspace members.

Canonical documentation: `docs/08-development/806-build-system-and-toolchain.md`.

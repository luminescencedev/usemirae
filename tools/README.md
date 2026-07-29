# tools/

Repository automation. Shell scripts call `xtask` rather than duplicating logic.

```text
xtask/     the single repository automation entry point (cargo xtask ...)
codegen/   schema-to-code generation
fixtures/  fixture generation
release/   packaging and release manifests
```

`xtask` landed with `MIR-0003` and is a Cargo workspace member. Run
`cargo xtask help` for the command surface. It has no dependencies: an argument
parser would have to go through the Rust dependency procedure in
`DEPENDENCY_VERSIONS.md` section 11 for a surface small enough to parse by hand.

`codegen/`, `fixtures/`, and `release/` are still empty and are owned by
`MIR-0005`/`MIR-0006`, the fixture work, and the release sprint respectively.

Canonical documentation: `docs/08-development/806-build-system-and-toolchain.md`.

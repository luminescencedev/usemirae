# crates/diagnostics/

Reserved crate group from `docs/08-development/801-monorepo-architecture.md`.

Tracing, metrics, correlation, and redaction currently live in
`crates/foundation/observability` (`mirae-observability`), as specified by
`docs/08-development/802-rust-workspace-and-crates.md` §2.

This group takes diagnostics crates that are not foundation-level — diagnostic
report collection, bundle export, and local log tooling — as those tickets land
(`MIR-0008` and the quality backlog).

//! Generated Rust bindings for IPC, project, SDK, and diagnostics schemas.
//!
//! Canonical documentation:
//! - `docs/08-development/802-rust-workspace-and-crates.md`
//! - `docs/08-development/805-generated-contracts-and-schemas.md`
//! - `docs/08-development/804-dependency-rules.md`
//!
//! The contents of [`generated`] are produced by `cargo xtask generate` from the
//! schemas under `schemas/`. Do not edit them by hand: `cargo xtask generate
//! --check` fails on any difference, and CI runs it.
//!
//! Internal runtime types map to these DTOs explicitly (`805` section 7); this
//! crate does not gain behavior of its own.

pub mod generated;

pub use generated::CONTRACT_IDS;

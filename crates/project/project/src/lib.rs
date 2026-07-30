//! Project format mapping, persistence orchestration, migrations, assets, and recovery.
//!
//! Canonical documentation: `docs/04-project/400-project-overview.md`,
//! `docs/04-project/401-project-format.md`,
//! `docs/08-development/802-rust-workspace-and-crates.md`.
//! Dependency rules: `docs/08-development/804-dependency-rules.md`.
//!
//! # What exists today
//!
//! Creating an empty project through a command and a transaction, and the
//! explicit mapping from authoritative state onto the generated schema
//! (`MIR-0107`, `MIR-0108`).
//!
//! # What does not
//!
//! Everything that touches a file: atomic save (`MIR-0109`), open and validation
//! (`MIR-0110`), dirty tracking (`MIR-0111`), and the recovery store
//! (`MIR-0112`). This crate reaches nothing platform-shaped yet, and when it
//! does it will be through an interface this layer owns rather than through
//! `std::fs` sprinkled across it (`804` section 4).

pub mod create;
pub mod mapping;

pub use create::{
    CreateProject, CreatedProject, MAX_PROJECT_NAME_CHARACTERS, create_empty_project,
};
pub use mapping::{PROJECT_FORMAT, PROJECT_SCHEMA_VERSION, body_of, envelope_of};

#[cfg(test)]
mod tests;

//! Project format mapping, persistence orchestration, migrations, assets, and recovery.
//!
//! Canonical documentation: `docs/04-project/400-project-overview.md`,
//! `docs/04-project/401-project-format.md`,
//! `docs/08-development/802-rust-workspace-and-crates.md`.
//! Dependency rules: `docs/08-development/804-dependency-rules.md`.
//!
//! # What exists today
//!
//! Creating an empty project through a command and a transaction, the explicit
//! mapping from authoritative state onto the generated schema, canonical
//! serialization with an integrity hash, atomic save, opening a project with
//! layered validation, dirty tracking derived from generations, and a bounded
//! recovery store (`MIR-0107` through `MIR-0112`).
//!
//! # What does not
//!
//! Autosave scheduling, which decides *when* a record is written; `MIR-0112`
//! built the store it writes into. Filesystem access is confined to [`save`],
//! [`open`], and [`recovery`], so there is one place to look when the
//! platform-specific parts of `403` section 4 need a real adapter.

pub mod canonical;
pub mod create;
pub mod dirty;
pub mod mapping;
pub mod open;
pub mod recovery;
pub mod save;

pub use canonical::{CanonicalError, LINE_ENDING, integrity_matches, serialize_with_integrity};
pub use create::{
    CreateProject, CreatedProject, MAX_PROJECT_NAME_CHARACTERS, create_empty_project,
};
pub use dirty::{SaveState, SaveStateProjection};
pub use mapping::{PROJECT_FORMAT, PROJECT_SCHEMA_VERSION, body_of, envelope_of};
pub use open::{
    Diagnostic, MAX_PROJECT_FILE_BYTES, OpenError, OpenMode, OpenedProject, open_document,
    open_project,
};
pub use recovery::{RecoveryCandidate, RecoveryError, RecoveryStore, RetentionPolicy};
pub use save::{
    Durability, FaultPlan, FaultPoint, FileIdentity, FilesystemFailure, SaveError, SaveResult,
    clean_stale_temporaries, save_project, save_project_with_faults,
};

#[cfg(test)]
mod fault_tests;
#[cfg(test)]
mod open_tests;
#[cfg(test)]
mod recovery_tests;
#[cfg(test)]
mod save_tests;
#[cfg(test)]
mod tests;

//! Authoritative state store, generations, snapshots, and patches.
//!
//! Canonical documentation: `docs/01-runtime/106-state-store.md`,
//! `docs/01-runtime/107-transactions.md`, ADR-0070.
//! Dependency rules: `docs/08-development/804-dependency-rules.md`.
//!
//! # What exists today
//!
//! The store, immutable generation-stamped snapshots, derived indexes, and
//! bounded retention (`MIR-0102`).
//!
//! # What does not
//!
//! Transactions, patches, and the session and capability partitions from `106`
//! section 2. They arrive with `MIR-0104` and `MIR-0106`, in this crate, because
//! `106` section 4 allows only the transaction coordinator to commit and that
//! rule is enforced here by module visibility rather than by convention.

pub mod project_state;
pub mod store;

pub use project_state::{Indexes, ProjectState};
pub use store::{PROJECTION_SCHEMA_VERSION, RETAINED_SNAPSHOTS, Snapshot, StateStore};

#[cfg(test)]
mod tests;

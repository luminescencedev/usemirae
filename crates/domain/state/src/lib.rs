//! Authoritative state store, generations, snapshots, and patches.
//!
//! Canonical documentation: `docs/01-runtime/106-state-store.md`,
//! `docs/01-runtime/107-transactions.md`, ADR-0070.
//! Dependency rules: `docs/08-development/804-dependency-rules.md`.
//!
//! # What exists today
//!
//! The store, immutable generation-stamped snapshots, derived indexes, and
//! bounded retention (`MIR-0102`); the transaction coordinator that is the only
//! way to commit (`MIR-0104`); domain events published after commit through
//! bounded subscriber queues (`MIR-0105`).
//!
//! # What does not
//!
//! Patches, and the session and capability partitions from `106` section 2.
//! They arrive with `MIR-0106`, in this crate, because `106` section 4 allows
//! only the transaction coordinator to commit and that rule is enforced here by
//! module visibility rather than by convention.

pub mod events;
pub mod project_state;
pub mod store;
pub mod transaction;

pub use events::{
    DomainEvent, EventBus, EventEnvelope, EventId, EventSequence, OverflowPolicy, Subscriber,
    SubscriberId,
};
pub use project_state::{Indexes, ProjectState};
pub use store::{PROJECTION_SCHEMA_VERSION, RETAINED_SNAPSHOTS, Snapshot, StateStore};
pub use transaction::{CommitOutcome, Transaction, TransactionError, UndoRecord};

#[cfg(test)]
mod tests;

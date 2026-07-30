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
//! bounded subscriber queues (`MIR-0105`); the projection a client mirrors and
//! the patches that advance it (`MIR-0106`).
//!
//! # What does not
//!
//! The session and capability partitions from `106` section 2, and the
//! idempotency cache from `104` section 8. They arrive with the tickets that
//! first need them.

pub mod events;
pub mod patch;
pub mod project_state;
pub mod store;
pub mod transaction;

pub use events::{
    DomainEvent, EventBus, EventEnvelope, EventId, EventSequence, OverflowPolicy, Subscriber,
    SubscriberId,
};
pub use patch::{
    Mirror, MirrorError, PatchOperation, ProjectProjection, SceneItemProjection, SceneProjection,
    SourceProjection, StatePatch, diff, patch_between,
};
pub use project_state::{Indexes, ProjectState};
pub use store::{PROJECTION_SCHEMA_VERSION, RETAINED_SNAPSHOTS, Snapshot, StateStore};
pub use transaction::{CommitOutcome, Transaction, TransactionError, UndoRecord};

#[cfg(test)]
mod tests;

//! Command DTOs, validation interfaces, and transaction intents.
//!
//! Canonical documentation: `docs/01-runtime/104-command-system.md`.
//! Dependency rules: `docs/08-development/804-dependency-rules.md`.
//!
//! # What exists today
//!
//! The envelope, the actor and capability model, the acknowledgement and its
//! error categories, the stateless validation stages, and the type-keyed
//! registry (`MIR-0103`).
//!
//! # What does not
//!
//! Execution. A handler needs to read state and commit a transaction, which is
//! `MIR-0104`. Idempotency caching, long-running operations, and undo records
//! arrive with the tickets that first need them; the policy each command
//! declares is recorded here so those tickets have something to read.

pub mod envelope;
pub mod validation;

pub use envelope::{
    ActorContext, ActorKind, Capability, CommandAcknowledgement, CommandEnvelope, CommandError,
    CommandId, CommandStatus, IdempotencyKey, IdempotencyKeyError, MAX_IDEMPOTENCY_KEY_BYTES,
};
pub use validation::{
    CommandPayload, CommandRegistry, CommandRequirements, LifecyclePhase, Registration, UndoPolicy,
    validate,
};

#[cfg(test)]
mod tests;

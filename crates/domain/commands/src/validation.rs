//! The validation stages a command passes before it may commit.
//!
//! Canonical documentation: `docs/01-runtime/104-command-system.md` sections 5,
//! 7, 9 and 10.
//!
//! `104` section 5 lists ten stages in order and says that failure at any stage
//! before commit produces no authoritative state mutation. The order is not
//! decorative: checking the payload before checking the session would spend work
//! on a command addressed to an engine that no longer exists, and checking the
//! generation before checking permission would tell an actor that may not ask
//! whether it would have succeeded.
//!
//! Stages one to seven live here, because they need no state and no transaction.
//! Execution and commit are `MIR-0104`.

use std::any::{TypeId, type_name};
use std::collections::BTreeMap;

use mirae_types::StateGeneration;

use crate::envelope::{Capability, CommandEnvelope, CommandError};

/// Where the engine is, as far as a command needs to know.
///
/// A coarse projection of `102-engine-lifecycle.md`, and deliberately not that
/// enum: `804` section 4 puts the interface with the layer that needs it, and
/// the domain must not depend on the runtime to ask whether a project is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LifecyclePhase {
    /// Not yet accepting commands.
    Starting,
    /// Accepting commands, no project active (`102` section 5).
    Ready,
    /// A project is active and domain commands are accepted.
    ProjectActive,
    /// Refusing new work that would prolong shutdown.
    ShuttingDown,
}

/// What a command does to the undo history (`104` section 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum UndoPolicy {
    /// Fully reversible from a recorded inverse.
    Undoable,
    /// A runtime operation with no project-state inverse.
    NotUndoable,
    /// An external effect that undo must not pretend to reverse.
    ///
    /// Starting a stream is the example `104` section 9 gives: the bytes have
    /// left the machine, and no undo record brings them back.
    Irreversible,
}

/// What a command requires before it may run (`104` section 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandRequirements {
    /// The capability the actor must hold.
    pub capability: Capability,
    /// The phases in which this command is accepted.
    pub phases: &'static [LifecyclePhase],
    /// What this command does to undo history.
    pub undo: UndoPolicy,
}

/// A command payload, with the rules that govern it.
///
/// Registration is by type. `104` section 16 forbids the alternative — a name
/// and an arbitrary payload — because it moves dispatch from the compiler to a
/// string table, and a string table accepts whatever a caller sends.
pub trait CommandPayload: 'static {
    /// The rules for this command.
    const REQUIREMENTS: CommandRequirements;

    /// Validate the payload in isolation (`104` section 5, stage 5).
    ///
    /// Schema validation only: everything checkable without reading state. A
    /// precondition that depends on what exists is stage 6, and it belongs to
    /// the handler.
    fn validate(&self) -> Result<(), CommandError>;
}

/// Run the stateless validation stages, in the order `104` section 5 defines.
///
/// The `current_generation` argument is the engine's, not the client's. Passing
/// `None` skips the conflict check, which is correct only for a command whose
/// meaning does not depend on what it replaces.
pub fn validate<T: CommandPayload>(
    envelope: &CommandEnvelope<T>,
    engine_session_id: &str,
    phase: LifecyclePhase,
    current_generation: Option<StateGeneration>,
) -> Result<(), CommandError> {
    // 1. Envelope.
    if envelope.engine_session_id.is_empty() {
        return Err(CommandError::InvalidEnvelope);
    }

    // 2. Protocol and session.
    if envelope.engine_session_id != engine_session_id {
        return Err(CommandError::WrongSession);
    }

    // 3. Actor capability.
    if !envelope.actor.holds(T::REQUIREMENTS.capability) {
        return Err(CommandError::PermissionDenied);
    }

    // 4. Lifecycle state.
    if !T::REQUIREMENTS.phases.contains(&phase) {
        return Err(CommandError::WrongLifecycleState);
    }

    // 5. Schema.
    envelope.payload.validate()?;

    // 6 and 7. Domain preconditions belong to the handler, which can read state.
    // The generation check does not: it compares two numbers, and doing it here
    // means no handler can forget it.
    if let (Some(expected), Some(current)) = (envelope.expected_generation, current_generation)
        && expected != current
    {
        return Err(CommandError::StateConflict);
    }

    Ok(())
}

/// Which commands the engine knows about (`104` section 10).
///
/// Keyed by [`TypeId`], so a command exists because a type exists. Nothing can
/// add an entry by sending a name, and a handler cannot be shadowed by a second
/// registration under the same string, because there are no strings.
#[derive(Debug, Default)]
pub struct CommandRegistry {
    entries: BTreeMap<TypeIdKey, Registration>,
}

/// A sortable wrapper for [`TypeId`], which is `Ord` but not usefully so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TypeIdKey(TypeId);

/// What the registry knows about one command type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Registration {
    /// The payload type's name, for diagnostics only.
    pub type_name: &'static str,
    /// The rules for this command.
    pub requirements: CommandRequirements,
}

impl CommandRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a command type.
    ///
    /// Returns whether this was the first registration. A duplicate is a
    /// programming error rather than a runtime condition, so it is reported
    /// rather than silently replacing the first — `104` section 10 registers
    /// handlers by type, and two handlers for one type is ambiguous, not an
    /// override.
    pub fn register<T: CommandPayload>(&mut self) -> bool {
        self.entries
            .insert(
                TypeIdKey(TypeId::of::<T>()),
                Registration {
                    type_name: type_name::<T>(),
                    requirements: T::REQUIREMENTS,
                },
            )
            .is_none()
    }

    /// The registration for a command type, if it has one.
    #[must_use]
    pub fn registration<T: CommandPayload>(&self) -> Option<&Registration> {
        self.entries.get(&TypeIdKey(TypeId::of::<T>()))
    }

    /// Whether a command type is registered.
    #[must_use]
    pub fn contains<T: CommandPayload>(&self) -> bool {
        self.registration::<T>().is_some()
    }

    /// How many command types are registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

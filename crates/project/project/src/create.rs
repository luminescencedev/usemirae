//! Creating an empty project.
//!
//! Canonical documentation: `docs/04-project/400-project-overview.md`,
//! `docs/01-runtime/104-command-system.md`, `docs/01-runtime/102-engine-lifecycle.md`
//! section 5.
//!
//! Creation is a command and a transaction, not a constructor. That is not
//! ceremony: `005` section 7 and `104` section 1 make commands the only way
//! project state comes into being, so creation goes through the same validation,
//! the same actor check, and the same generation increment as every later edit.
//! A constructor called from the UI would be a second way to produce state, and
//! the second way is always the one that skips a rule.

use mirae_commands::{
    Capability, CommandEnvelope, CommandError, CommandPayload, CommandRequirements, LifecyclePhase,
    UndoPolicy, validate,
};
use mirae_state::{DomainEvent, StateStore};
use mirae_types::{ProjectId, StateGeneration};

/// Longest name accepted for a new project.
pub const MAX_PROJECT_NAME_CHARACTERS: usize = 256;

/// Ask the engine to create an empty project (`104` section 2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateProject {
    /// What to call it.
    ///
    /// Held for the project library and for the file name a save will suggest.
    /// It is not an identifier: two projects may share a name, and renaming one
    /// changes nothing about what it is (ADR-0069).
    pub name: String,
}

impl CommandPayload for CreateProject {
    const REQUIREMENTS: CommandRequirements = CommandRequirements {
        capability: Capability::ManageProjectLifecycle,
        // `Ready` only. `102` section 5 whitelists what the engine accepts with
        // no project open, and creating one while another is active is the
        // lifecycle rule this ticket owes: close first, deliberately, rather
        // than discovering afterwards which project the next edit landed in.
        phases: &[LifecyclePhase::Ready],
        undo: UndoPolicy::NotUndoable,
    };

    fn validate(&self) -> Result<(), CommandError> {
        if self.name.trim().is_empty() {
            return Err(CommandError::InvalidArgument);
        }

        if self.name.chars().count() > MAX_PROJECT_NAME_CHARACTERS {
            return Err(CommandError::InvalidArgument);
        }

        if self.name.chars().any(char::is_control) {
            return Err(CommandError::InvalidArgument);
        }

        Ok(())
    }
}

/// What creating a project produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedProject {
    /// The new project's stable identifier.
    pub project_id: ProjectId,
    /// The generation the empty project was committed at.
    pub generation: StateGeneration,
    /// The name it was created with.
    pub name: String,
}

/// Create an empty project, returning the store that now holds it.
///
/// The store is built here rather than mutated, because a project's identity is
/// fixed for the life of the store: `106` section 7 ties every snapshot to an
/// active project id, and swapping that underneath live snapshots would make
/// them silently describe a different project.
pub fn create_empty_project(
    command: &CommandEnvelope<CreateProject>,
    engine_session_id: &str,
    phase: LifecyclePhase,
) -> Result<(StateStore, CreatedProject), CommandError> {
    validate(command, engine_session_id, phase, None)?;

    let project_id = ProjectId::new();
    let mut store = StateStore::new(engine_session_id, project_id);

    // Empty, but still a transaction. The generation it produces is what a
    // client synchronizes against, and a project that appeared without one would
    // be the only state in the system nobody could name a version of.
    let mut transaction = store.transaction();
    transaction.emit(DomainEvent::ProjectCreated {
        project: *project_id.as_entity_id(),
    });

    let outcome = transaction
        .commit()
        .map_err(|error| error.as_command_error())?;

    let created = CreatedProject {
        project_id,
        generation: outcome.generation,
        name: command.payload.name.clone(),
    };

    Ok((store, created))
}

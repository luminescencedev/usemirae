//! Tests for the command envelope and its validation stages.
//!
//! Named against `104-command-system.md` section 15. What this ticket owes:
//! schema rejection, permission denial, lifecycle denial, generation conflict,
//! duplicate command registration, and error categories that carry no user text.
//! Transaction rollback, idempotent retry, asynchronous operations, and ordering
//! under concurrency belong to `MIR-0104` and are not faked here.

use mirae_types::StateGeneration;

use crate::envelope::{
    ActorContext, ActorKind, Capability, CommandAcknowledgement, CommandEnvelope, CommandError,
    CommandId, CommandStatus, IdempotencyKey, IdempotencyKeyError, MAX_IDEMPOTENCY_KEY_BYTES,
};
use crate::validation::{
    CommandPayload, CommandRegistry, CommandRequirements, LifecyclePhase, UndoPolicy, validate,
};

const SESSION: &str = "0000000000000000000000000000002a";

/// A mutation command, for the stages that apply to one.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RenameScene {
    name: String,
}

impl CommandPayload for RenameScene {
    const REQUIREMENTS: CommandRequirements = CommandRequirements {
        capability: Capability::MutateProject,
        phases: &[LifecyclePhase::ProjectActive],
        undo: UndoPolicy::Undoable,
    };

    fn validate(&self) -> Result<(), CommandError> {
        if self.name.is_empty() {
            return Err(CommandError::InvalidArgument);
        }

        Ok(())
    }
}

/// A lifecycle command, which is accepted in a different phase.
#[derive(Debug, Clone, PartialEq, Eq)]
struct OpenProject;

impl CommandPayload for OpenProject {
    const REQUIREMENTS: CommandRequirements = CommandRequirements {
        capability: Capability::ManageProjectLifecycle,
        phases: &[LifecyclePhase::Ready, LifecyclePhase::ProjectActive],
        undo: UndoPolicy::NotUndoable,
    };

    fn validate(&self) -> Result<(), CommandError> {
        Ok(())
    }
}

fn envelope(payload: RenameScene) -> CommandEnvelope<RenameScene> {
    CommandEnvelope {
        command_id: CommandId::new(),
        engine_session_id: SESSION.to_owned(),
        actor: ActorContext::local_ui(),
        expected_generation: None,
        idempotency_key: None,
        issued_at_millis: None,
        payload,
    }
}

fn valid() -> CommandEnvelope<RenameScene> {
    envelope(RenameScene {
        name: "Main".to_owned(),
    })
}

#[test]
fn a_well_formed_command_passes_every_stateless_stage() {
    let outcome = validate(&valid(), SESSION, LifecyclePhase::ProjectActive, None);

    assert_eq!(outcome, Ok(()));
}

#[test]
fn a_command_addressed_to_another_session_is_refused() {
    // Not stale: addressed to an engine that no longer exists. A new session may
    // hold a different project entirely.
    let outcome = validate(
        &valid(),
        "a-different-session",
        LifecyclePhase::ProjectActive,
        None,
    );

    assert_eq!(outcome, Err(CommandError::WrongSession));
}

#[test]
fn an_empty_session_is_refused_as_a_malformed_envelope() {
    let mut command = valid();
    command.engine_session_id = String::new();

    assert_eq!(
        validate(&command, SESSION, LifecyclePhase::ProjectActive, None),
        Err(CommandError::InvalidEnvelope)
    );
}

#[test]
fn an_actor_without_the_capability_is_refused() {
    // 104 invariant 4: permissions are checked before execution, not inside a
    // handler that has already begun to work.
    let mut command = valid();
    command.actor = ActorContext::new(ActorKind::Extension, [Capability::ReadState]);

    assert_eq!(
        validate(&command, SESSION, LifecyclePhase::ProjectActive, None),
        Err(CommandError::PermissionDenied)
    );
}

#[test]
fn a_command_is_refused_in_a_phase_that_does_not_allow_it() {
    // 102 section 5 whitelists what `Ready` accepts. A project mutation with no
    // project open is not a conflict, it is a category error.
    assert_eq!(
        validate(&valid(), SESSION, LifecyclePhase::Ready, None),
        Err(CommandError::WrongLifecycleState)
    );
    assert_eq!(
        validate(&valid(), SESSION, LifecyclePhase::ShuttingDown, None),
        Err(CommandError::WrongLifecycleState)
    );
}

#[test]
fn a_payload_that_fails_its_own_validation_is_refused() {
    let command = envelope(RenameScene {
        name: String::new(),
    });

    assert_eq!(
        validate(&command, SESSION, LifecyclePhase::ProjectActive, None),
        Err(CommandError::InvalidArgument)
    );
}

#[test]
fn a_stale_expected_generation_is_a_conflict_rather_than_a_rejection() {
    // 104 section 7. The client needs to tell "you may not" from "someone got
    // there first", because only one of the two is worth retrying.
    let mut command = valid();
    command.expected_generation = Some(StateGeneration::from_raw(4));

    let outcome = validate(
        &command,
        SESSION,
        LifecyclePhase::ProjectActive,
        Some(StateGeneration::from_raw(7)),
    );

    assert_eq!(outcome, Err(CommandError::StateConflict));
    assert_eq!(
        CommandError::StateConflict.status(),
        CommandStatus::Conflict
    );
    assert_eq!(
        CommandError::PermissionDenied.status(),
        CommandStatus::Rejected
    );
}

#[test]
fn a_matching_expected_generation_passes() {
    let mut command = valid();
    command.expected_generation = Some(StateGeneration::from_raw(7));

    assert_eq!(
        validate(
            &command,
            SESSION,
            LifecyclePhase::ProjectActive,
            Some(StateGeneration::from_raw(7))
        ),
        Ok(())
    );
}

#[test]
fn permission_is_checked_before_the_generation() {
    // The order in 104 section 5 is load-bearing. An actor that may not issue a
    // command must not learn from the answer whether it would have conflicted.
    let mut command = valid();
    command.actor = ActorContext::new(ActorKind::Extension, [Capability::ReadState]);
    command.expected_generation = Some(StateGeneration::from_raw(1));

    assert_eq!(
        validate(
            &command,
            SESSION,
            LifecyclePhase::ProjectActive,
            Some(StateGeneration::from_raw(9))
        ),
        Err(CommandError::PermissionDenied)
    );
}

#[test]
fn the_session_is_checked_before_the_payload() {
    // A command addressed elsewhere is not worth validating.
    let mut command = envelope(RenameScene {
        name: String::new(),
    });
    command.engine_session_id = "elsewhere".to_owned();

    assert_eq!(
        validate(&command, SESSION, LifecyclePhase::ProjectActive, None),
        Err(CommandError::WrongSession)
    );
}

#[test]
fn commands_are_registered_by_type_and_registered_twice_is_reported() {
    // 104 section 10 and section 16: dispatch belongs to the compiler. Two
    // handlers for one type is ambiguous, not an override.
    let mut registry = CommandRegistry::new();

    assert!(registry.is_empty());
    assert!(registry.register::<RenameScene>());
    assert!(!registry.register::<RenameScene>(), "already registered");
    assert!(registry.register::<OpenProject>());

    assert_eq!(registry.len(), 2);
    assert!(registry.contains::<RenameScene>());

    let registration = registry.registration::<RenameScene>();
    assert_eq!(
        registration.map(|entry| entry.requirements.undo),
        Some(UndoPolicy::Undoable)
    );
    assert_eq!(
        registry
            .registration::<OpenProject>()
            .map(|entry| entry.requirements.capability),
        Some(Capability::ManageProjectLifecycle)
    );
}

#[test]
fn an_acknowledgement_names_the_generation_it_committed() {
    // 104 invariant 2. Without it a client cannot tell which patch answers its
    // own command.
    let id = CommandId::new();
    let ack = CommandAcknowledgement::accepted(id, SESSION, StateGeneration::from_raw(3));

    assert_eq!(ack.status, CommandStatus::Accepted);
    assert!(ack.status.committed());
    assert_eq!(ack.committed_generation, Some(StateGeneration::from_raw(3)));
    assert_eq!(ack.error, None);
}

#[test]
fn a_conflict_acknowledgement_carries_the_current_generation() {
    // 104 section 7 prohibits blind retry, which means the answer has to contain
    // what the client needs in order to retry deliberately.
    let ack = CommandAcknowledgement::refused(
        CommandId::new(),
        SESSION,
        CommandError::StateConflict,
        Some(StateGeneration::from_raw(9)),
    );

    assert_eq!(ack.status, CommandStatus::Conflict);
    assert!(!ack.status.committed());
    assert_eq!(ack.committed_generation, None);
    assert_eq!(ack.current_generation, Some(StateGeneration::from_raw(9)));
}

#[test]
fn every_error_category_has_a_stable_identifier_and_no_user_text() {
    // 104 invariant 10: errors are structured and redacted. A category can be
    // matched, counted, and translated; a message built from user input is how a
    // project file reaches a log.
    for error in [
        CommandError::InvalidEnvelope,
        CommandError::WrongSession,
        CommandError::PermissionDenied,
        CommandError::WrongLifecycleState,
        CommandError::InvalidArgument,
        CommandError::EntityNotFound,
        CommandError::StateConflict,
        CommandError::UnsupportedCommand,
        CommandError::Internal,
    ] {
        assert!(!error.as_str().is_empty());
        assert!(!error.to_string().is_empty());
        assert_eq!(error.as_str(), error.as_str().to_ascii_lowercase());
    }
}

#[test]
fn an_idempotency_key_is_bounded_and_printable() {
    // 104 section 8: the cache holding these is bounded, which an unbounded key
    // would make meaningless.
    assert!(IdempotencyKey::new("retry-0001").is_ok());
    assert_eq!(IdempotencyKey::new(""), Err(IdempotencyKeyError::Empty));
    assert_eq!(
        IdempotencyKey::new("a".repeat(MAX_IDEMPOTENCY_KEY_BYTES + 1)),
        Err(IdempotencyKeyError::TooLong)
    );
    assert_eq!(
        IdempotencyKey::new("line\nbreak"),
        Err(IdempotencyKeyError::UnsupportedCharacter)
    );
}

#[test]
fn a_client_timestamp_is_carried_but_never_consulted() {
    // 104 invariant 5. Validation must not read it, because a client's clock is
    // not evidence of anything.
    let mut command = valid();
    command.issued_at_millis = Some(i64::MIN);

    assert_eq!(
        validate(&command, SESSION, LifecyclePhase::ProjectActive, None),
        Ok(())
    );
}

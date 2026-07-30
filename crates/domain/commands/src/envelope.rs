//! The command envelope and its acknowledgement.
//!
//! Canonical documentation: `docs/01-runtime/104-command-system.md` sections 3
//! to 8 and 13.
//!
//! A command expresses intent. `104` section 1 makes it the only supported way
//! to request a domain mutation, and section 16 forbids the shape that would
//! quietly undo that rule: a generic endpoint taking a command name and an
//! arbitrary payload. So the payload here is a type parameter, handlers are
//! registered by type, and there is nowhere to put a string that selects
//! behaviour.

use std::fmt;

use mirae_types::{EntityId, StateGeneration};

/// Longest idempotency key accepted.
///
/// The key comes from a client, so it is bounded before it is stored: the cache
/// that holds it is bounded too (`104` section 8), and an unbounded key would
/// make a bounded count meaningless.
pub const MAX_IDEMPOTENCY_KEY_BYTES: usize = 128;

/// Identifies one command submission (`104` section 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandId(EntityId);

impl CommandId {
    /// Mint a new command identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(EntityId::new())
    }

    /// Wrap an existing identifier, for deserialization and fixtures.
    #[must_use]
    pub const fn from_entity_id(id: EntityId) -> Self {
        Self(id)
    }

    /// The underlying identifier.
    #[must_use]
    pub const fn as_entity_id(&self) -> &EntityId {
        &self.0
    }
}

impl Default for CommandId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CommandId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Who issued a command (`104` section 4).
///
/// The kind is not decoration. `104` invariant 4 requires permissions to be
/// checked before execution, and an extension asking to delete a project is a
/// different question from a user asking the same thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ActorKind {
    /// The local control UI, driven by the person at the machine.
    LocalUi,
    /// An extension, acting within its granted capabilities.
    Extension,
    /// The engine itself, for internal maintenance work.
    Internal,
    /// The recovery process, restoring after a crash.
    Recovery,
}

/// What an actor is allowed to do.
///
/// A deliberately small set. `104` section 4 asks for capabilities; inventing a
/// permission taxonomy before there are commands to permit would produce names
/// nothing checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Capability {
    /// Read authoritative state.
    ReadState,
    /// Mutate project-domain state.
    MutateProject,
    /// Open, create, and close projects.
    ManageProjectLifecycle,
}

/// The identity and rights of a command's issuer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorContext {
    kind: ActorKind,
    capabilities: Vec<Capability>,
}

impl ActorContext {
    /// Build an actor context.
    #[must_use]
    pub fn new(kind: ActorKind, capabilities: impl IntoIterator<Item = Capability>) -> Self {
        let mut capabilities: Vec<_> = capabilities.into_iter().collect();
        capabilities.sort_unstable();
        capabilities.dedup();

        Self { kind, capabilities }
    }

    /// The local control UI, with the rights a person at the machine has.
    #[must_use]
    pub fn local_ui() -> Self {
        Self::new(
            ActorKind::LocalUi,
            [
                Capability::ReadState,
                Capability::MutateProject,
                Capability::ManageProjectLifecycle,
            ],
        )
    }

    /// What kind of actor this is.
    #[must_use]
    pub const fn kind(&self) -> ActorKind {
        self.kind
    }

    /// Whether this actor holds `capability`.
    #[must_use]
    pub fn holds(&self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }
}

/// A client-supplied key for safe retry across an interrupted connection.
///
/// `104` section 8: a command that may be retried should carry one, so a
/// reconnecting client learns the first attempt's result instead of applying it
/// twice.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Validate and wrap a key.
    pub fn new(text: impl Into<String>) -> Result<Self, IdempotencyKeyError> {
        let text = text.into();

        if text.is_empty() {
            return Err(IdempotencyKeyError::Empty);
        }

        if text.len() > MAX_IDEMPOTENCY_KEY_BYTES {
            return Err(IdempotencyKeyError::TooLong);
        }

        // The key is used as a cache key and appears in diagnostics. Restricting
        // it to printable ASCII keeps it from carrying a newline into a log
        // record or a control sequence into a terminal.
        if !text
            .bytes()
            .all(|byte| byte.is_ascii_graphic() || byte == b'-' || byte == b'_')
        {
            return Err(IdempotencyKeyError::UnsupportedCharacter);
        }

        Ok(Self(text))
    }

    /// The key as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Why an idempotency key was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdempotencyKeyError {
    /// The key had no characters.
    Empty,
    /// The key exceeded [`MAX_IDEMPOTENCY_KEY_BYTES`].
    TooLong,
    /// The key contained a character the runtime does not store.
    UnsupportedCharacter,
}

impl fmt::Display for IdempotencyKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "an idempotency key cannot be empty",
            Self::TooLong => "the idempotency key is longer than the runtime stores",
            Self::UnsupportedCharacter => {
                "an idempotency key accepts printable ASCII, hyphen, and underscore"
            }
        })
    }
}

impl std::error::Error for IdempotencyKeyError {}

/// A command, addressed and attributed (`104` section 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEnvelope<T> {
    /// Identifies this submission.
    pub command_id: CommandId,
    /// The engine session the client believes it is talking to.
    ///
    /// A command carrying a previous session's identifier is not stale, it is
    /// addressed to an engine that no longer exists.
    pub engine_session_id: String,
    /// Who is asking.
    pub actor: ActorContext,
    /// The generation the client wrote this command against (`104` section 7).
    ///
    /// `None` for a command whose meaning does not depend on what it is
    /// replacing. Present, and checked, for everything else.
    pub expected_generation: Option<StateGeneration>,
    /// A key allowing safe retry (`104` section 8).
    pub idempotency_key: Option<IdempotencyKey>,
    /// When the client believes it issued this.
    ///
    /// Informational. `104` invariant 5: client timestamps never determine
    /// authoritative ordering, because a client's clock is not evidence.
    pub issued_at_millis: Option<i64>,
    /// What is being asked for.
    pub payload: T,
}

/// The outcome of a command (`104` section 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStatus {
    /// Passed the commit point. Not "received".
    Accepted,
    /// Refused before commit, and the reason is the command's own.
    Rejected,
    /// Refused because the state moved under it.
    Conflict,
    /// Attempted and failed.
    Failed,
    /// Withdrawn before it took effect.
    Cancelled,
}

impl CommandStatus {
    /// A stable identifier for diagnostics and the wire.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Conflict => "conflict",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// Whether the authoritative state changed.
    #[must_use]
    pub const fn committed(self) -> bool {
        matches!(self, Self::Accepted)
    }
}

/// What went wrong, in a form safe for a log and for a screen (`104` section 13).
///
/// Every variant is a category, not a message. A category can be matched on,
/// counted, and translated; a message can only be printed, and a message built
/// from user input is how a project file ends up in a log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommandError {
    /// The envelope itself was malformed.
    InvalidEnvelope,
    /// The command was addressed to a different engine session.
    WrongSession,
    /// The actor lacks the capability the handler requires.
    PermissionDenied,
    /// The engine is not in a state that allows this command.
    WrongLifecycleState,
    /// The payload failed its own validation.
    InvalidArgument,
    /// A referenced entity does not exist.
    EntityNotFound,
    /// The expected generation did not match.
    StateConflict,
    /// No handler is registered for this payload type.
    UnsupportedCommand,
    /// Something failed that the caller cannot classify further.
    Internal,
}

impl CommandError {
    /// A stable identifier for diagnostics and the wire.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidEnvelope => "invalid_envelope",
            Self::WrongSession => "wrong_session",
            Self::PermissionDenied => "permission_denied",
            Self::WrongLifecycleState => "wrong_lifecycle_state",
            Self::InvalidArgument => "invalid_argument",
            Self::EntityNotFound => "entity_not_found",
            Self::StateConflict => "state_conflict",
            Self::UnsupportedCommand => "unsupported_command",
            Self::Internal => "internal",
        }
    }

    /// The status this error produces.
    ///
    /// A conflict is not a rejection: `104` section 7 requires the client to be
    /// able to tell "you may not" from "someone got there first", because only
    /// one of the two is worth retrying.
    #[must_use]
    pub const fn status(self) -> CommandStatus {
        match self {
            Self::StateConflict => CommandStatus::Conflict,
            Self::Internal => CommandStatus::Failed,
            _ => CommandStatus::Rejected,
        }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidEnvelope => "the command envelope was not well formed",
            Self::WrongSession => "the command was addressed to a different engine session",
            Self::PermissionDenied => "the actor may not issue this command",
            Self::WrongLifecycleState => {
                "the engine cannot accept this command in its current state"
            }
            Self::InvalidArgument => "the command payload was not valid",
            Self::EntityNotFound => "the command referenced something that does not exist",
            Self::StateConflict => "the state changed while the command was being written",
            Self::UnsupportedCommand => "no handler is registered for this command",
            Self::Internal => "the command failed for an internal reason",
        })
    }
}

impl std::error::Error for CommandError {}

/// The answer to a command (`104` section 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandAcknowledgement {
    /// The submission being answered.
    pub command_id: CommandId,
    /// What happened.
    pub status: CommandStatus,
    /// The session that answered.
    pub engine_session_id: String,
    /// The generation produced, when the command committed.
    ///
    /// `104` invariant 2: an accepted mutation names its resulting generation,
    /// so a client can tell which patch corresponds to its own command.
    pub committed_generation: Option<StateGeneration>,
    /// Why, when it did not commit.
    pub error: Option<CommandError>,
    /// The current generation, when the answer was a conflict.
    ///
    /// `104` section 7 requires the conflict to carry it: without it the client
    /// can only retry blindly, which the same section prohibits.
    pub current_generation: Option<StateGeneration>,
}

impl CommandAcknowledgement {
    /// Acknowledge a committed command.
    #[must_use]
    pub fn accepted(
        command_id: CommandId,
        engine_session_id: impl Into<String>,
        generation: StateGeneration,
    ) -> Self {
        Self {
            command_id,
            status: CommandStatus::Accepted,
            engine_session_id: engine_session_id.into(),
            committed_generation: Some(generation),
            error: None,
            current_generation: Some(generation),
        }
    }

    /// Acknowledge a command that did not commit.
    #[must_use]
    pub fn refused(
        command_id: CommandId,
        engine_session_id: impl Into<String>,
        error: CommandError,
        current_generation: Option<StateGeneration>,
    ) -> Self {
        Self {
            command_id,
            status: error.status(),
            engine_session_id: engine_session_id.into(),
            committed_generation: None,
            error: Some(error),
            current_generation,
        }
    }
}

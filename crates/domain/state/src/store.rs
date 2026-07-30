//! The state store: immutable snapshots in, one authoritative generation out.
//!
//! Canonical documentation: `docs/01-runtime/106-state-store.md`, ADR-0070.
//!
//! A reader gets an `Arc<Snapshot>` of a state that can no longer change, so
//! `106` invariant 2 — readers never mutate committed state — holds because
//! there is no path to a `&mut`, not because callers are careful. The commit
//! path is a pointer swap: everything expensive happens before it, which is what
//! `107-transactions.md` section 12 requires.

use std::collections::VecDeque;
use std::sync::Arc;

use mirae_types::{ProjectId, StateGeneration};

use crate::project_state::{Indexes, ProjectState};

/// The projection schema version a snapshot was built for.
///
/// A consumer that knows a different version cannot apply patches against this
/// one and must ask for a fresh snapshot (`109-ui-engine-synchronization.md`
/// section 5). It is separate from the project schema version: the wire shape
/// and the file shape evolve for different reasons.
pub const PROJECTION_SCHEMA_VERSION: u32 = 1;

/// How many superseded snapshots the store keeps.
///
/// `106` section 12 and invariant 8 require bounded retention. The number is
/// small on purpose: retained snapshots exist to let a slightly-behind consumer
/// be answered without a full rebuild, not to be a history. History is
/// `405-command-history-and-undo-redo.md`, and it is a different ticket with
/// different bounds.
pub const RETAINED_SNAPSHOTS: usize = 8;

/// An immutable, generation-stamped view of authoritative state (`106` section 7).
#[derive(Debug, PartialEq, Eq)]
pub struct Snapshot {
    engine_session_id: String,
    generation: StateGeneration,
    projection_schema_version: u32,
    state: Arc<ProjectState>,
    indexes: Indexes,
}

impl Snapshot {
    /// The engine session this snapshot belongs to.
    ///
    /// A snapshot from a previous session is not stale, it is meaningless: a new
    /// session may hold a different project entirely, so a consumer compares the
    /// session before it compares the generation.
    #[must_use]
    pub fn engine_session_id(&self) -> &str {
        &self.engine_session_id
    }

    /// The committed generation this snapshot represents.
    #[must_use]
    pub const fn generation(&self) -> StateGeneration {
        self.generation
    }

    /// The projection schema version.
    #[must_use]
    pub const fn projection_schema_version(&self) -> u32 {
        self.projection_schema_version
    }

    /// The active project.
    #[must_use]
    pub fn project_id(&self) -> ProjectId {
        self.state.project_id()
    }

    /// The immutable state.
    #[must_use]
    pub fn state(&self) -> &Arc<ProjectState> {
        &self.state
    }

    /// The derived indexes for this state.
    #[must_use]
    pub const fn indexes(&self) -> &Indexes {
        &self.indexes
    }
}

/// Owns the authoritative state for one engine session (`106` section 1).
///
/// `install` and `candidate` are crate-visible because `106` section 4 allows
/// only the transaction coordinator to commit. The coordinator is
/// [`crate::Transaction`], in this crate, so the rule is a visibility boundary
/// rather than a comment.
#[derive(Debug)]
pub struct StateStore {
    current: Arc<Snapshot>,
    superseded: VecDeque<Arc<Snapshot>>,
}

impl StateStore {
    /// Create a store holding an empty project at the initial generation.
    #[must_use]
    pub fn new(engine_session_id: impl Into<String>, project_id: ProjectId) -> Self {
        let state = ProjectState::empty(project_id);
        let indexes = Indexes::build(&state);

        Self {
            current: Arc::new(Snapshot {
                engine_session_id: engine_session_id.into(),
                generation: StateGeneration::INITIAL,
                projection_schema_version: PROJECTION_SCHEMA_VERSION,
                state: Arc::new(state),
                indexes,
            }),
            superseded: VecDeque::new(),
        }
    }

    /// The current authoritative snapshot.
    ///
    /// Cheap: an `Arc` clone. A reader may hold it for as long as it likes, and
    /// what it holds will not change underneath it.
    #[must_use]
    pub fn snapshot(&self) -> Arc<Snapshot> {
        Arc::clone(&self.current)
    }

    /// The current committed generation.
    #[must_use]
    pub fn generation(&self) -> StateGeneration {
        self.current.generation
    }

    /// A retained snapshot at `generation`, if it is still held.
    ///
    /// Returns `None` once retention has dropped it, which is the answer that
    /// sends a consumer back for a fresh snapshot rather than leaving it to
    /// guess (`106` section 8).
    #[must_use]
    pub fn retained(&self, generation: StateGeneration) -> Option<Arc<Snapshot>> {
        if self.current.generation == generation {
            return Some(Arc::clone(&self.current));
        }

        self.superseded
            .iter()
            .find(|snapshot| snapshot.generation == generation)
            .map(Arc::clone)
    }

    /// How many superseded snapshots are retained.
    #[must_use]
    pub fn retained_count(&self) -> usize {
        self.superseded.len()
    }

    /// Install a new state as the next generation.
    ///
    /// Crate-visible on purpose: `106` section 4 allows only the transaction
    /// coordinator to commit, and the coordinator lives in this crate. Making
    /// this public would make that sentence a comment rather than a rule.
    #[allow(
        dead_code,
        reason = "the transaction coordinator that calls it lands in MIR-0104"
    )]
    pub(crate) fn install(&mut self, state: ProjectState) -> Arc<Snapshot> {
        let indexes = Indexes::build(&state);
        let next = Arc::new(Snapshot {
            engine_session_id: self.current.engine_session_id.clone(),
            generation: self.current.generation.next(),
            projection_schema_version: PROJECTION_SCHEMA_VERSION,
            state: Arc::new(state),
            indexes,
        });

        let previous = std::mem::replace(&mut self.current, Arc::clone(&next));
        self.superseded.push_back(previous);

        while self.superseded.len() > RETAINED_SNAPSHOTS {
            // Front first: the oldest snapshot is the one no consumer can still
            // be waiting for.
            self.superseded.pop_front();
        }

        next
    }

    /// Produce the state a transaction should mutate.
    ///
    /// A clone of the current state: the spine and the pointers, not the
    /// entities (ADR-0070). The caller mutates its own copy and hands it back,
    /// so nothing observable changes until [`Self::install`].
    #[allow(
        dead_code,
        reason = "the transaction coordinator that calls it lands in MIR-0104"
    )]
    pub(crate) fn candidate(&self) -> ProjectState {
        (*self.current.state).clone()
    }
}

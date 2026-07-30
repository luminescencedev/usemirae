//! Transactions: related changes commit together or leave no trace.
//!
//! Canonical documentation: `docs/01-runtime/107-transactions.md`, ADR-0070.
//!
//! `107` section 3 defines the phases — begin, read, validate, prepare,
//! revalidate, commit, publish — and section 12 forbids disk, network, device,
//! encoder, GPU, and extension work while holding commit authority. Both are
//! structural here rather than advisory: a [`Transaction`] borrows the store
//! mutably, so nesting is a borrow error rather than a runtime check, and
//! preparation happens on a candidate the caller owns, so nothing the caller
//! does is visible until the swap at the end.
//!
//! What the coordinator deliberately does not do is anything external. `107`
//! section 5 gives the pattern for a side-effecting command — commit the intent,
//! do the work, commit the observed result — and the reason is section 11:
//! after a commit there is no rollback, only a new compensating transaction.

use std::sync::Arc;

use mirae_commands::{CommandError, CommandId};
use mirae_types::{EntityId, StateGeneration};

use crate::events::DomainEvent;
use crate::project_state::ProjectState;
use crate::store::{Snapshot, StateStore};

/// Why a transaction did not commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionError {
    /// The state moved between the client writing the command and the commit.
    ///
    /// Carries the current generation, because `104` section 7 prohibits blind
    /// retry and a client cannot retry deliberately without it.
    Conflict {
        /// What the caller expected.
        expected: StateGeneration,
        /// What the store actually holds.
        current: StateGeneration,
    },
    /// Preparation refused the change.
    Rejected(CommandError),
}

impl TransactionError {
    /// The command-level error this corresponds to.
    #[must_use]
    pub const fn as_command_error(&self) -> CommandError {
        match self {
            Self::Conflict { .. } => CommandError::StateConflict,
            Self::Rejected(error) => *error,
        }
    }
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conflict { expected, current } => write!(
                formatter,
                "the state moved from generation {expected} to {current} while the command was being written"
            ),
            Self::Rejected(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for TransactionError {}

/// What a committed transaction can be undone with (`107` section 8).
///
/// The prior state is an `Arc` to a snapshot that already existed, so recording
/// one costs a pointer rather than a copy (ADR-0070). That is what makes it
/// affordable to record an undo entry for every undoable commit rather than
/// deciding later which ones deserved it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoRecord {
    /// The generation this record reverses.
    pub committed_generation: StateGeneration,
    /// The command that produced it, if a command did.
    pub command_id: Option<CommandId>,
    /// The entities the transaction touched.
    pub affected: Vec<EntityId>,
    /// The state as it was immediately before the commit.
    pub prior_state: Arc<ProjectState>,
}

/// What a commit produced (`107` section 3.7).
#[derive(Debug, Clone)]
pub struct CommitOutcome {
    /// The snapshot now authoritative.
    pub snapshot: Arc<Snapshot>,
    /// The generation it carries.
    pub generation: StateGeneration,
    /// How to reverse it, when the transaction declared itself undoable.
    pub undo: Option<UndoRecord>,
    /// The domain events this commit produced, in the order they were emitted.
    ///
    /// Returned rather than published here. `105` section 4 and `107` section
    /// 3.7 require publication *after* commit, and the surest way to guarantee
    /// that is for the commit to have no way to publish: a caller cannot leak an
    /// event early if it only receives them once the swap has happened.
    pub events: Vec<DomainEvent>,
}

/// One in-memory domain transaction (`107` section 2).
///
/// Holds the store mutably for its whole life. `107` invariant 4 prohibits
/// nested commits; here a second transaction cannot be opened because the first
/// one still holds the borrow, so the rule is enforced by the compiler rather
/// than by a flag someone has to remember to check.
#[derive(Debug)]
pub struct Transaction<'store> {
    store: &'store mut StateStore,
    candidate: ProjectState,
    began_at: StateGeneration,
    expected: Option<StateGeneration>,
    command_id: Option<CommandId>,
    affected: Vec<EntityId>,
    events: Vec<DomainEvent>,
    undoable: bool,
}

impl<'store> Transaction<'store> {
    /// Begin a transaction (`107` section 3.1).
    pub(crate) fn begin(store: &'store mut StateStore) -> Self {
        let began_at = store.generation();
        let candidate = store.candidate();

        Self {
            store,
            candidate,
            began_at,
            expected: None,
            command_id: None,
            affected: Vec::new(),
            events: Vec::new(),
            undoable: false,
        }
    }

    /// Require the store to still be at `generation` when this commits.
    #[must_use]
    pub const fn expecting(mut self, generation: StateGeneration) -> Self {
        self.expected = Some(generation);
        self
    }

    /// Attribute this transaction to a command, for correlation (`107` invariant 10).
    #[must_use]
    pub const fn by_command(mut self, command_id: CommandId) -> Self {
        self.command_id = Some(command_id);
        self
    }

    /// Record an undo entry when this commits (`107` section 8).
    #[must_use]
    pub const fn undoable(mut self) -> Self {
        self.undoable = true;
        self
    }

    /// The generation this transaction began against.
    #[must_use]
    pub const fn began_at(&self) -> StateGeneration {
        self.began_at
    }

    /// Read committed state (`107` section 3.2).
    #[must_use]
    pub const fn state(&self) -> &ProjectState {
        &self.candidate
    }

    /// Build the candidate change (`107` section 3.4).
    ///
    /// The closure mutates a state the caller alone can see. Returning an error
    /// leaves the transaction usable and the store untouched, which is what
    /// `107` section 10 means by failure before commit having no effect.
    pub fn prepare<F>(&mut self, build: F) -> Result<(), TransactionError>
    where
        F: FnOnce(&mut ProjectState) -> Result<(), CommandError>,
    {
        // Prepare against a copy of the copy: a closure that fails halfway
        // through must not leave the candidate half-changed for a later
        // `prepare` on the same transaction.
        let mut attempt = self.candidate.clone();

        build(&mut attempt).map_err(TransactionError::Rejected)?;

        self.candidate = attempt;
        Ok(())
    }

    /// Queue a domain event for publication after commit (`105` section 2.1).
    ///
    /// Queued, not published. If this transaction never commits, the event never
    /// existed, which is what `107` section 10 means by a pre-commit failure
    /// publishing no committed event.
    pub fn emit(&mut self, event: DomainEvent) {
        self.events.push(event);
    }

    /// Record that this transaction touched `entity`.
    pub fn touched(&mut self, entity: EntityId) {
        if !self.affected.contains(&entity) {
            self.affected.push(entity);
        }
    }

    /// Revalidate and commit (`107` sections 3.5 and 3.6).
    ///
    /// The generation is checked here rather than at `begin`, because the whole
    /// point of the check is what happened in between.
    pub fn commit(self) -> Result<CommitOutcome, TransactionError> {
        let current = self.store.generation();

        if let Some(expected) = self.expected
            && expected != current
        {
            return Err(TransactionError::Conflict { expected, current });
        }

        // The store cannot have moved under an exclusive borrow, so this is a
        // second line of defence rather than a race: it catches a caller that
        // held a transaction across something that should not have been possible.
        if self.began_at != current {
            return Err(TransactionError::Conflict {
                expected: self.began_at,
                current,
            });
        }

        let prior_state = Arc::clone(self.store.snapshot().state());
        let snapshot = self.store.install(self.candidate);
        let generation = snapshot.generation();

        let undo = if self.undoable {
            Some(UndoRecord {
                committed_generation: generation,
                command_id: self.command_id,
                affected: self.affected,
                prior_state,
            })
        } else {
            None
        };

        Ok(CommitOutcome {
            snapshot,
            generation,
            undo,
            events: self.events,
        })
    }
}

impl StateStore {
    /// Begin a transaction against this store.
    ///
    /// The only way to commit. `106` section 4 allows writes through the
    /// transaction coordinator alone, and this is the entire surface through
    /// which one can be obtained.
    pub fn transaction(&mut self) -> Transaction<'_> {
        Transaction::begin(self)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use mirae_domain::{EntityName, Scene, SceneItem, SourceDefinition, SourceKind};
    use mirae_types::{ProjectId, SceneId, SceneItemId, SourceId};

    use super::*;

    const SESSION: &str = "0000000000000000000000000000002a";

    fn name(text: &str) -> EntityName {
        EntityName::new(text).unwrap_or_else(|_| {
            EntityName::new("unnamed").unwrap_or_else(|_| unreachable!("a literal name is valid"))
        })
    }

    fn store() -> StateStore {
        StateStore::new(SESSION, ProjectId::nil())
    }

    fn scene(id: SceneId, items: Vec<SceneItemId>) -> Scene {
        Scene {
            id,
            name: name("Scene"),
            items,
        }
    }

    fn source(id: SourceId) -> SourceDefinition {
        SourceDefinition {
            id,
            name: name("Colour"),
            kind: SourceKind::Color,
        }
    }

    fn item(id: SceneItemId, scene: SceneId, source: SourceId) -> SceneItem {
        SceneItem {
            id,
            scene,
            source,
            visible: true,
        }
    }

    #[test]
    fn a_transaction_commits_several_entities_as_one_generation() {
        // 107 invariants 1 and 2: all-or-nothing, and exactly one increment
        // however many entities were touched.
        let mut store = store();
        let scene_id = SceneId::new();
        let source_id = SourceId::new();
        let item_id = SceneItemId::new();

        let mut transaction = store.transaction();
        let prepared = transaction.prepare(|state| {
            state.put_source(source(source_id));
            state.put_scene(scene(scene_id, vec![item_id]));
            state.put_scene_item(item(item_id, scene_id, source_id));
            Ok(())
        });

        assert_eq!(prepared, Ok(()));

        let outcome = transaction.commit().ok();

        assert_eq!(
            outcome.map(|outcome| outcome.generation),
            Some(StateGeneration::from_raw(1))
        );
        assert_eq!(store.snapshot().state().entity_count(), 3);
    }

    #[test]
    fn preparation_that_fails_leaves_no_trace() {
        // 107 section 10: failure before commit does not increment the
        // generation and publishes nothing.
        let mut store = store();
        let mut transaction = store.transaction();

        let refused = transaction.prepare(|state| {
            state.put_scene(scene(SceneId::new(), Vec::new()));
            Err(CommandError::InvalidArgument)
        });

        assert_eq!(
            refused,
            Err(TransactionError::Rejected(CommandError::InvalidArgument))
        );

        // The half-built change is gone. A later prepare on the same transaction
        // starts from committed state rather than from the wreckage of the first.
        assert_eq!(transaction.state().entity_count(), 0);

        let outcome = transaction.commit().ok();

        assert_eq!(
            outcome.map(|outcome| outcome.generation),
            Some(StateGeneration::from_raw(1))
        );
        assert_eq!(store.snapshot().state().entity_count(), 0);
    }

    #[test]
    fn abandoning_a_transaction_changes_nothing() {
        let mut store = store();

        {
            let mut transaction = store.transaction();
            let _ = transaction.prepare(|state| {
                state.put_scene(scene(SceneId::new(), Vec::new()));
                Ok(())
            });
            // Dropped without commit.
        }

        assert_eq!(store.generation(), StateGeneration::INITIAL);
        assert_eq!(store.snapshot().state().entity_count(), 0);
    }

    #[test]
    fn a_stale_expectation_is_a_conflict_naming_both_generations() {
        // 104 section 7: a client cannot retry deliberately without knowing what
        // the store actually holds.
        let mut store = store();
        let _ = store.transaction().commit();

        let outcome = store
            .transaction()
            .expecting(StateGeneration::INITIAL)
            .commit();

        assert_eq!(
            outcome.err(),
            Some(TransactionError::Conflict {
                expected: StateGeneration::INITIAL,
                current: StateGeneration::from_raw(1),
            })
        );
        assert_eq!(store.generation(), StateGeneration::from_raw(1));
    }

    #[test]
    fn a_conflict_maps_to_the_state_conflict_command_error() {
        let error = TransactionError::Conflict {
            expected: StateGeneration::INITIAL,
            current: StateGeneration::from_raw(2),
        };

        assert_eq!(error.as_command_error(), CommandError::StateConflict);
        assert_eq!(
            TransactionError::Rejected(CommandError::EntityNotFound).as_command_error(),
            CommandError::EntityNotFound
        );
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn a_matching_expectation_commits() {
        let mut store = store();
        let outcome = store
            .transaction()
            .expecting(StateGeneration::INITIAL)
            .commit();

        assert_eq!(
            outcome.ok().map(|outcome| outcome.generation),
            Some(StateGeneration::from_raw(1))
        );
    }

    #[test]
    fn an_undoable_commit_records_the_prior_state_by_pointer() {
        // 107 section 8. ADR-0070 makes this affordable: the prior state is an
        // `Arc` to a snapshot that already existed, so recording one costs a
        // pointer rather than a copy of the project.
        let mut store = store();
        let scene_id = SceneId::new();

        let mut transaction = store.transaction().by_command(CommandId::new()).undoable();
        transaction.touched(*scene_id.as_entity_id());
        transaction.touched(*scene_id.as_entity_id());
        let _ = transaction.prepare(|state| {
            state.put_scene(scene(scene_id, Vec::new()));
            Ok(())
        });

        let undo = transaction.commit().ok().and_then(|outcome| outcome.undo);

        assert_eq!(
            undo.as_ref().map(|record| record.committed_generation),
            Some(StateGeneration::from_raw(1))
        );
        assert_eq!(
            undo.as_ref().map(|record| record.affected.clone()),
            Some(vec![*scene_id.as_entity_id()]),
            "an entity touched twice is recorded once"
        );
        assert_eq!(
            undo.as_ref()
                .map(|record| record.prior_state.entity_count()),
            Some(0)
        );
        assert!(undo.and_then(|record| record.command_id).is_some());
        assert_eq!(store.snapshot().state().entity_count(), 1);
    }

    #[test]
    fn a_transaction_that_does_not_declare_itself_undoable_records_nothing() {
        // 107 section 8 ties undo records to transactions that opted in. A
        // runtime operation with no project-state inverse must not leave one.
        let mut store = store();
        let outcome = store.transaction().commit().ok();

        assert!(outcome.is_some_and(|outcome| outcome.undo.is_none()));
    }

    #[test]
    fn the_committed_snapshot_carries_matching_indexes() {
        let mut store = store();
        let scene_id = SceneId::new();
        let source_id = SourceId::new();
        let item_id = SceneItemId::new();

        let mut transaction = store.transaction();
        let _ = transaction.prepare(|state| {
            state.put_source(source(source_id));
            state.put_scene(scene(scene_id, vec![item_id]));
            state.put_scene_item(item(item_id, scene_id, source_id));
            Ok(())
        });

        let snapshot = transaction.commit().ok().map(|outcome| outcome.snapshot);

        assert_eq!(
            snapshot
                .as_ref()
                .map(|snapshot| snapshot.indexes().matches(snapshot.state())),
            Some(true)
        );
        assert_eq!(
            snapshot.map(|snapshot| snapshot.indexes().items_using_source(source_id).len()),
            Some(1)
        );
    }

    #[test]
    fn reading_inside_a_transaction_sees_committed_state_plus_its_own_preparation() {
        // 107 section 3.2 and 3.4: a handler reads state, then builds on it. A
        // second prepare must see the first one's work, or a multi-step handler
        // cannot be written at all.
        let mut store = store();
        let first = SceneId::new();
        let second = SceneId::new();

        let mut transaction = store.transaction();
        let _ = transaction.prepare(|state| {
            state.put_scene(scene(first, Vec::new()));
            Ok(())
        });

        assert_eq!(transaction.state().entity_count(), 1);
        assert!(transaction.state().scene(first).is_some());

        let _ = transaction.prepare(|state| {
            assert!(state.scene(first).is_some());
            state.put_scene(scene(second, Vec::new()));
            Ok(())
        });

        let _ = transaction.commit();

        assert_eq!(store.snapshot().state().entity_count(), 2);
    }

    #[test]
    fn a_commit_against_a_large_project_stays_bounded() {
        // 107 invariant 9 and section 14: the serialized commit section is short
        // and bounded. The same loose bound as the store benchmark, for the same
        // reason — the regression worth catching is a factor of hundreds.
        const ENTITIES: usize = 10_000;
        const BUDGET_MILLIS: u128 = 20;

        let mut store = store();
        let source_id = SourceId::new();
        let mut transaction = store.transaction();
        let _ = transaction.prepare(|state| {
            state.put_source(source(source_id));

            for _ in 0..ENTITIES {
                let scene_id = SceneId::new();
                let item_id = SceneItemId::new();
                state.put_scene(scene(scene_id, vec![item_id]));
                state.put_scene_item(item(item_id, scene_id, source_id));
            }

            Ok(())
        });
        let _ = transaction.commit();

        // Best of several, not one measurement. `cargo test` runs these in
        // parallel on a machine doing other work, so a single sample measures
        // contention as much as code; the fastest achievable run is what
        // actually answers "is this copying pointers or entities".
        let best = (0..5)
            .map(|_| {
                let started = Instant::now();
                let mut transaction = store.transaction().undoable();
                let _ = transaction.prepare(|state| {
                    state.put_scene(scene(SceneId::new(), Vec::new()));
                    Ok(())
                });
                let outcome = transaction.commit();
                let elapsed = started.elapsed();

                assert!(outcome.is_ok());
                elapsed
            })
            .min()
            .unwrap_or_default();

        assert!(
            best.as_millis() < BUDGET_MILLIS,
            "the fastest of five transactions against {ENTITIES} entities took {best:?}"
        );
    }
}

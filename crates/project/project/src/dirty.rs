//! Whether the project on disk matches the project in memory.
//!
//! Canonical documentation: `docs/04-project/403-persistence.md` sections 8 and
//! 11, `docs/01-runtime/107-transactions.md` section 3.8.
//!
//! Dirtiness is derived, never set. Two generations — the one committed and the
//! one saved — answer the question by comparison, so there is no boolean that
//! can drift out of step with reality. A flag someone forgets to clear leaves a
//! user told they have unsaved work when they do not; a flag someone forgets to
//! set loses their work. Neither is reachable from here.
//!
//! The subtle case is a commit that lands while a save is running. The save
//! covers the generation it started with, not whatever arrived afterwards
//! (`403` section 8), so the project is still dirty when it finishes — and this
//! type says so rather than reporting the project clean because a save
//! completed.

use mirae_types::StateGeneration;

/// How the in-memory project relates to the file on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveState {
    committed: StateGeneration,
    saved: Option<StateGeneration>,
    saving: Option<StateGeneration>,
}

impl SaveState {
    /// A project that has never been saved.
    ///
    /// Dirty from the start when anything has been committed: a project that
    /// exists only in memory has unsaved work by definition.
    #[must_use]
    pub const fn unsaved(committed: StateGeneration) -> Self {
        Self {
            committed,
            saved: None,
            saving: None,
        }
    }

    /// A project just opened from a file.
    ///
    /// The saved generation is the one loading committed, because the file and
    /// memory agree at that instant.
    #[must_use]
    pub const fn opened(committed: StateGeneration) -> Self {
        Self {
            committed,
            saved: Some(committed),
            saving: None,
        }
    }

    /// The newest committed generation.
    #[must_use]
    pub const fn committed(self) -> StateGeneration {
        self.committed
    }

    /// The generation on disk, if the project has ever been saved.
    #[must_use]
    pub const fn saved(self) -> Option<StateGeneration> {
        self.saved
    }

    /// Whether a save is in flight, and for which generation.
    #[must_use]
    pub const fn saving(self) -> Option<StateGeneration> {
        self.saving
    }

    /// Whether memory holds work the file does not.
    #[must_use]
    pub fn is_dirty(self) -> bool {
        self.saved != Some(self.committed)
    }

    /// Record a committed transaction.
    ///
    /// Ignores a generation that is not newer. Generations only move forward
    /// (`106` invariant 4), so an older one is a caller mistake rather than a
    /// state to represent, and accepting it would let dirtiness go backwards.
    pub const fn record_commit(&mut self, generation: StateGeneration) {
        if generation.get() > self.committed.get() {
            self.committed = generation;
        }
    }

    /// Note that a save has started, covering `generation`.
    pub const fn begin_save(&mut self, generation: StateGeneration) {
        self.saving = Some(generation);
    }

    /// Note that a save finished successfully.
    ///
    /// Takes the generation the save reported rather than the current one:
    /// `403` invariant 5 has the save name what it covered, and trusting the
    /// clock instead of the report is how a save gets credit for work it did not
    /// write.
    pub const fn complete_save(&mut self, generation: StateGeneration) {
        if let Some(saved) = self.saved
            && saved.get() >= generation.get()
        {
            // An older save finishing after a newer one must not walk the saved
            // generation backwards (`403` section 8: a failure is not hidden by
            // a later background save, and neither is a success).
            self.saving = None;
            return;
        }

        self.saved = Some(generation);
        self.saving = None;
    }

    /// Note that a save failed.
    ///
    /// The saved generation is untouched: the file still holds whatever it held
    /// before, and `403` invariant 7 requires failure to preserve it.
    pub const fn fail_save(&mut self) {
        self.saving = None;
    }
}

/// The save state as a client sees it (`109` section 3.1).
///
/// Projected rather than recomputed. A client that derived dirtiness from
/// generations it happened to have would answer differently while a patch was in
/// flight, and the engine is authoritative for this as for everything else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveStateProjection {
    /// The newest committed generation.
    pub committed_generation: u64,
    /// The generation on disk, if any.
    pub saved_generation: Option<u64>,
    /// Whether memory holds unsaved work.
    pub dirty: bool,
    /// Whether a save is running.
    pub saving: bool,
}

impl From<SaveState> for SaveStateProjection {
    fn from(state: SaveState) -> Self {
        Self {
            committed_generation: state.committed().get(),
            saved_generation: state.saved().map(StateGeneration::get),
            dirty: state.is_dirty(),
            saving: state.saving().is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generation(value: u64) -> StateGeneration {
        StateGeneration::from_raw(value)
    }

    #[test]
    fn a_project_that_has_never_been_saved_is_dirty() {
        let state = SaveState::unsaved(generation(1));

        assert!(state.is_dirty());
        assert_eq!(state.saved(), None);
    }

    #[test]
    fn a_freshly_opened_project_is_clean() {
        let state = SaveState::opened(generation(4));

        assert!(!state.is_dirty());
        assert_eq!(state.saved(), Some(generation(4)));
    }

    #[test]
    fn committing_makes_a_clean_project_dirty() {
        let mut state = SaveState::opened(generation(4));
        state.record_commit(generation(5));

        assert!(state.is_dirty());
        assert_eq!(state.committed(), generation(5));
    }

    #[test]
    fn saving_the_committed_generation_makes_it_clean() {
        let mut state = SaveState::opened(generation(4));
        state.record_commit(generation(5));

        state.begin_save(generation(5));
        assert_eq!(state.saving(), Some(generation(5)));
        assert!(state.is_dirty(), "still dirty while the save runs");

        state.complete_save(generation(5));

        assert!(!state.is_dirty());
        assert_eq!(state.saving(), None);
    }

    #[test]
    fn a_commit_during_a_save_leaves_the_project_dirty_afterwards() {
        // The case this type exists for. 403 section 8: a save covers the
        // generation it started with. Reporting the project clean because a save
        // finished would tell the user their newest edit is on disk when it is
        // not.
        let mut state = SaveState::opened(generation(4));
        state.record_commit(generation(5));
        state.begin_save(generation(5));

        // The user keeps working while the save runs.
        state.record_commit(generation(6));

        state.complete_save(generation(5));

        assert!(state.is_dirty());
        assert_eq!(state.saved(), Some(generation(5)));
        assert_eq!(state.committed(), generation(6));
    }

    #[test]
    fn a_failed_save_leaves_the_saved_generation_untouched() {
        // 403 invariant 7: the previous file survives a failure, so what the
        // engine believes about disk must survive it too.
        let mut state = SaveState::opened(generation(4));
        state.record_commit(generation(5));
        state.begin_save(generation(5));
        state.fail_save();

        assert!(state.is_dirty());
        assert_eq!(state.saved(), Some(generation(4)));
        assert_eq!(state.saving(), None);
    }

    #[test]
    fn an_older_save_finishing_late_does_not_walk_the_saved_generation_backwards() {
        // Coalescing means saves can overlap (403 section 8). A stale one
        // completing afterwards must not convince the engine that disk is older
        // than it is.
        let mut state = SaveState::opened(generation(4));
        state.record_commit(generation(9));
        state.complete_save(generation(9));

        state.complete_save(generation(6));

        assert_eq!(state.saved(), Some(generation(9)));
        assert!(!state.is_dirty());
    }

    #[test]
    fn an_older_commit_is_ignored() {
        let mut state = SaveState::opened(generation(4));
        state.record_commit(generation(9));
        state.record_commit(generation(7));

        assert_eq!(state.committed(), generation(9));
    }

    #[test]
    fn the_projection_says_the_same_thing_as_the_state() {
        let mut state = SaveState::opened(generation(4));
        state.record_commit(generation(5));
        state.begin_save(generation(5));

        let projection = SaveStateProjection::from(state);

        assert_eq!(projection.committed_generation, 5);
        assert_eq!(projection.saved_generation, Some(4));
        assert!(projection.dirty);
        assert!(projection.saving);
    }

    #[test]
    fn dirtiness_is_derived_from_generations_rather_than_stored() {
        // Setting the same generation twice, or completing a save for a
        // generation already saved, changes nothing. There is no flag to get out
        // of step.
        let mut state = SaveState::opened(generation(4));
        let before = state;

        state.record_commit(generation(4));
        state.complete_save(generation(4));

        assert_eq!(state.is_dirty(), before.is_dirty());
        assert_eq!(state.committed(), before.committed());
    }
}

//! Tests for the recovery store.
//!
//! What `MIR-0112` owes from `404`: autosave never writes over the canonical
//! project, only committed generations are recorded, retention is bounded by
//! count, bytes, and age, records are integrity-checked, and an invalid record
//! does not block opening the canonical project. Timed and coalesced autosave
//! belong to the scheduler, which is not this ticket.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use mirae_types::{ProjectId, StateGeneration};

use crate::mapping::envelope_of;
use crate::recovery::{RecoveryStore, RetentionPolicy};
use mirae_contracts::generated::PersistedProjectEnvelope;
use mirae_state::ProjectState;

const APP_VERSION: &str = "0.0.0";
const RECORDED_AT: &str = "2026-07-30T12:00:00Z";

/// A scratch directory that removes itself.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("mirae-recovery-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        let _ = std::fs::create_dir_all(&path);

        Self { path }
    }

    fn store(&self, retention: RetentionPolicy) -> RecoveryStore {
        RecoveryStore::new(self.path.join("recovery"), retention)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn envelope(project: ProjectId) -> PersistedProjectEnvelope {
    envelope_of(
        &ProjectState::empty(project),
        RECORDED_AT,
        RECORDED_AT,
        APP_VERSION,
    )
}

fn write(
    store: &RecoveryStore,
    project: ProjectId,
    generation: u64,
) -> Option<crate::recovery::RecoveryCandidate> {
    store
        .write(
            &envelope(project),
            project,
            StateGeneration::from_raw(generation),
            RECORDED_AT,
            "",
            APP_VERSION,
        )
        .ok()
}

#[test]
fn a_record_is_written_into_the_store_and_read_back() {
    let scratch = Scratch::new("write");
    let store = scratch.store(RetentionPolicy::standard());
    let project = ProjectId::new();

    let written = write(&store, project, 4);

    assert!(written.is_some());

    let candidates = store.candidates(project);

    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates.first().map(|candidate| candidate.generation()),
        Some(StateGeneration::from_raw(4))
    );
}

#[test]
fn the_store_creates_its_directory_only_when_something_is_written() {
    // Constructing a store touches nothing, so a caller can hold one without
    // committing to autosave ever running.
    let scratch = Scratch::new("lazy");
    let store = scratch.store(RetentionPolicy::standard());

    assert!(!store.root().exists());

    let _ = write(&store, ProjectId::new(), 1);

    assert!(store.root().exists());
}

#[test]
fn autosave_writes_nowhere_near_the_project_file() {
    // 404 invariant 1, asserted structurally: the store is given a root and
    // never a project path, so there is nothing for it to overwrite.
    let scratch = Scratch::new("separate");
    let project_file = scratch.path.join("project.mirae.json");
    let _ = std::fs::write(&project_file, b"the canonical project");

    let store = scratch.store(RetentionPolicy::standard());
    let _ = write(&store, ProjectId::new(), 1);

    assert_eq!(
        std::fs::read_to_string(&project_file).unwrap_or_default(),
        "the canonical project"
    );
    assert!(!store.root().starts_with(&project_file));
}

#[test]
fn candidates_are_returned_newest_generation_first() {
    // Recovery offers the most recent work first; a list in filesystem order
    // would offer whatever the directory happened to yield.
    let scratch = Scratch::new("order");
    let store = scratch.store(RetentionPolicy::standard());
    let project = ProjectId::new();

    for generation in [2, 7, 5] {
        let _ = write(&store, project, generation);
    }

    let generations: Vec<u64> = store
        .candidates(project)
        .iter()
        .map(|candidate| candidate.generation().get())
        .collect();

    assert_eq!(generations, vec![7, 5, 2]);
}

#[test]
fn one_project_does_not_see_another_projects_records() {
    let scratch = Scratch::new("isolation");
    let store = scratch.store(RetentionPolicy::standard());
    let first = ProjectId::new();
    let second = ProjectId::new();

    let _ = write(&store, first, 1);
    let _ = write(&store, second, 1);
    let _ = write(&store, second, 2);

    assert_eq!(store.candidates(first).len(), 1);
    assert_eq!(store.candidates(second).len(), 2);
}

#[test]
fn an_unreadable_record_is_skipped_rather_than_hiding_the_valid_ones() {
    // 404 invariant 10: an invalid record must not block opening the canonical
    // project, and it must not take the records beside it down either.
    let scratch = Scratch::new("corrupt");
    let store = scratch.store(RetentionPolicy::standard());
    let project = ProjectId::new();

    let _ = write(&store, project, 3);
    let _ = std::fs::write(
        store.root().join(format!("{project}.broken.recovery.json")),
        b"{ not a record",
    );

    let candidates = store.candidates(project);

    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates.first().map(|candidate| candidate.generation()),
        Some(StateGeneration::from_raw(3))
    );
}

#[test]
fn a_file_that_is_not_a_record_is_ignored() {
    let scratch = Scratch::new("stranger");
    let store = scratch.store(RetentionPolicy::standard());
    let project = ProjectId::new();

    let _ = write(&store, project, 1);
    let _ = std::fs::write(store.root().join("notes.txt"), b"unrelated");

    assert_eq!(store.candidates(project).len(), 1);
}

#[test]
fn retention_bounds_the_number_of_records() {
    // 404 invariant 4 and section 5.
    let scratch = Scratch::new("count");
    let store = scratch.store(RetentionPolicy {
        max_records: 3,
        ..RetentionPolicy::standard()
    });
    let project = ProjectId::new();

    for generation in 1..=8 {
        let _ = write(&store, project, generation);
    }

    let removed = store.prune(SystemTime::now());

    assert!(removed > 0);
    assert!(store.candidates(project).len() <= 3);
}

#[test]
fn pruning_keeps_the_newest_record() {
    // 404 section 5: at least one recent valid candidate survives. Pruning is
    // ordered so that stopping halfway still leaves the useful record behind.
    let scratch = Scratch::new("newest");
    let store = scratch.store(RetentionPolicy {
        max_records: 2,
        ..RetentionPolicy::standard()
    });
    let project = ProjectId::new();

    for generation in 1..=6 {
        let _ = write(&store, project, generation);
    }

    let _ = store.prune(SystemTime::now());
    let candidates = store.candidates(project);

    assert!(!candidates.is_empty());
    assert_eq!(
        candidates.first().map(|candidate| candidate.generation()),
        Some(StateGeneration::from_raw(6)),
        "the newest record is never the one pruned"
    );
}

#[test]
fn retention_bounds_the_total_size() {
    // A count bound alone lets a few enormous records fill a disk.
    let scratch = Scratch::new("bytes");
    let store = scratch.store(RetentionPolicy {
        max_records: 100,
        max_bytes: 1,
        ..RetentionPolicy::standard()
    });
    let project = ProjectId::new();

    for generation in 1..=5 {
        let _ = write(&store, project, generation);
    }

    let _ = store.prune(SystemTime::now());

    assert_eq!(
        store.candidates(project).len(),
        1,
        "only the newest fits in the budget"
    );
}

#[test]
fn retention_removes_records_older_than_the_policy() {
    // A byte bound alone never notices a record from last year.
    let scratch = Scratch::new("age");
    let store = scratch.store(RetentionPolicy {
        max_records: 100,
        max_bytes: u64::MAX,
        max_age: Duration::from_secs(1),
    });
    let project = ProjectId::new();

    for generation in 1..=4 {
        let _ = write(&store, project, generation);
    }

    // Ask as though a great deal of time has passed, rather than waiting for it.
    let later = SystemTime::now() + Duration::from_secs(60 * 60);
    let removed = store.prune(later);

    assert_eq!(removed, 3, "everything but the newest has expired");
    assert_eq!(store.candidates(project).len(), 1);
}

#[test]
fn pruning_an_empty_store_removes_nothing() {
    let scratch = Scratch::new("empty");
    let store = scratch.store(RetentionPolicy::standard());

    assert_eq!(store.prune(SystemTime::now()), 0);
}

#[test]
fn a_candidate_knows_which_explicit_save_it_builds_on() {
    // 404 section 6 compares base identity before generations: a record built on
    // a save the user has since replaced describes a different history.
    let scratch = Scratch::new("base");
    let store = scratch.store(RetentionPolicy::standard());
    let project = ProjectId::new();

    let written = store.write(
        &envelope(project),
        project,
        StateGeneration::from_raw(2),
        RECORDED_AT,
        "abc123",
        APP_VERSION,
    );

    let Ok(candidate) = written else {
        unreachable!("the write should have succeeded");
    };

    assert!(candidate.builds_on("abc123"));
    assert!(!candidate.builds_on("something-else"));
}

#[test]
fn a_record_carries_the_project_it_belongs_to_and_the_version_that_wrote_it() {
    let scratch = Scratch::new("metadata");
    let store = scratch.store(RetentionPolicy::standard());
    let project = ProjectId::new();

    let Some(candidate) = write(&store, project, 9) else {
        unreachable!("the write should have succeeded");
    };

    assert_eq!(candidate.record.project_id, project.to_string());
    assert_eq!(candidate.record.app_version, APP_VERSION);
    assert_eq!(candidate.record.state_generation, 9);
    assert_eq!(candidate.record.recorded_at, RECORDED_AT);
}

#[test]
fn discarding_removes_only_the_named_projects_records() {
    // 404 section 8: cleanup after a clean close, and only for what closed.
    let scratch = Scratch::new("discard");
    let store = scratch.store(RetentionPolicy::standard());
    let closed = ProjectId::new();
    let other = ProjectId::new();

    let _ = write(&store, closed, 1);
    let _ = write(&store, closed, 2);
    let _ = write(&store, other, 1);

    let removed = store.discard(closed);

    assert_eq!(removed, 2);
    assert!(store.candidates(closed).is_empty());
    assert_eq!(store.candidates(other).len(), 1);
}

#[test]
fn candidates_of_a_store_that_does_not_exist_is_empty_rather_than_an_error() {
    // A machine that has never autosaved is the ordinary case, not a failure.
    let scratch = Scratch::new("absent");
    let store = scratch.store(RetentionPolicy::standard());

    assert!(store.candidates(ProjectId::new()).is_empty());
}

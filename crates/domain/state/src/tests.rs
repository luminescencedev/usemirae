//! Tests for the state store.
//!
//! Named against `106-state-store.md` section 14, which lists what this ticket
//! owes: concurrent immutable reads, index consistency, bounded snapshot
//! retention, runtime-handle exclusion, and a large-project mutation benchmark.
//! Atomic commit, generation conflict, and patch equivalence belong to
//! `MIR-0104` and `MIR-0106` and are not faked here.

use std::sync::Arc;
use std::time::Instant;

use mirae_domain::{EntityName, Scene, SceneItem, SourceDefinition, SourceKind};
use mirae_types::{ProjectId, SceneId, SceneItemId, SourceId, StateGeneration};

use crate::project_state::{Indexes, ProjectState};
use crate::store::{RETAINED_SNAPSHOTS, Snapshot, StateStore};

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
fn a_new_store_holds_an_empty_project_at_the_initial_generation() {
    let store = store();
    let snapshot = store.snapshot();

    assert_eq!(snapshot.generation(), StateGeneration::INITIAL);
    assert_eq!(snapshot.state().entity_count(), 0);
    assert_eq!(snapshot.engine_session_id(), SESSION);
    assert_eq!(snapshot.project_id(), ProjectId::nil());
}

#[test]
fn installing_a_state_advances_the_generation_exactly_once() {
    // 106 invariant 4. One install, one increment — not one per entity touched.
    let mut store = store();
    let mut candidate = store.candidate();

    candidate.put_scene(scene(SceneId::nil(), Vec::new()));
    candidate.put_source(source(SourceId::nil()));

    let installed = store.install(candidate);

    assert_eq!(installed.generation(), StateGeneration::from_raw(1));
    assert_eq!(store.generation(), StateGeneration::from_raw(1));
    assert_eq!(installed.state().entity_count(), 2);
}

#[test]
fn a_snapshot_taken_before_a_commit_does_not_change_afterwards() {
    // The property the whole design exists for: a reader holding a snapshot is
    // holding a value, not a view.
    let mut store = store();
    let before = store.snapshot();

    let mut candidate = store.candidate();
    candidate.put_scene(scene(SceneId::new(), Vec::new()));
    store.install(candidate);

    assert_eq!(before.state().entity_count(), 0);
    assert_eq!(before.generation(), StateGeneration::INITIAL);
    assert_eq!(store.snapshot().state().entity_count(), 1);
}

#[test]
fn many_readers_hold_snapshots_concurrently_while_writes_continue() {
    // 106 section 14: concurrent immutable reads. Real threads, because the
    // claim is about `Send` and `Sync`, and a single-threaded test would prove
    // nothing about either.
    let mut store = store();
    let mut candidate = store.candidate();
    candidate.put_source(source(SourceId::nil()));
    store.install(candidate);

    let held = store.snapshot();
    let readers: Vec<_> = (0..8)
        .map(|_| {
            let snapshot = Arc::clone(&held);
            std::thread::spawn(move || {
                (0..1_000)
                    .map(|_| snapshot.state().entity_count())
                    .sum::<usize>()
            })
        })
        .collect();

    for _ in 0..16 {
        let mut candidate = store.candidate();
        candidate.put_scene(scene(SceneId::new(), Vec::new()));
        store.install(candidate);
    }

    for reader in readers {
        assert_eq!(reader.join().ok(), Some(1_000));
    }

    assert_eq!(held.state().entity_count(), 1);
    assert_eq!(store.snapshot().state().entity_count(), 17);
}

#[test]
fn indexes_match_the_entities_they_are_derived_from() {
    // 106 invariant 6. Asserted after a commit, because drift between a
    // maintained index and its entities is the failure this rebuild avoids.
    let mut store = store();
    let scene_id = SceneId::new();
    let source_id = SourceId::new();
    let first = SceneItemId::new();
    let second = SceneItemId::new();

    let mut candidate = store.candidate();
    candidate.put_source(source(source_id));
    candidate.put_scene(scene(scene_id, vec![first, second]));
    candidate.put_scene_item(item(first, scene_id, source_id));
    candidate.put_scene_item(item(second, scene_id, source_id));

    let snapshot = store.install(candidate);
    let indexes = snapshot.indexes();

    assert!(indexes.matches(snapshot.state()));
    assert_eq!(indexes.items_in_scene(scene_id).len(), 2);
    assert_eq!(indexes.items_using_source(source_id).len(), 2);
    assert!(indexes.items_in_scene(SceneId::new()).is_empty());
}

#[test]
fn removing_an_entity_removes_it_from_the_indexes() {
    let mut store = store();
    let scene_id = SceneId::new();
    let source_id = SourceId::new();
    let item_id = SceneItemId::new();

    let mut candidate = store.candidate();
    candidate.put_source(source(source_id));
    candidate.put_scene(scene(scene_id, vec![item_id]));
    candidate.put_scene_item(item(item_id, scene_id, source_id));
    store.install(candidate);

    let mut candidate = store.candidate();
    assert!(candidate.remove_scene_item(item_id));
    assert!(!candidate.remove_scene_item(item_id), "already gone");
    let snapshot = store.install(candidate);

    assert!(snapshot.indexes().matches(snapshot.state()));
    assert!(snapshot.indexes().items_using_source(source_id).is_empty());
    assert!(snapshot.state().scene_item(item_id).is_none());

    // The scene and the source outlive the item that referenced them. Removing
    // an item is not removing what it pointed at, and the store does not decide
    // otherwise on the caller's behalf.
    assert!(snapshot.state().scene(scene_id).is_some());
    assert!(snapshot.state().source(source_id).is_some());

    let mut candidate = store.candidate();
    assert!(candidate.remove_scene(scene_id));
    assert!(candidate.remove_source(source_id));
    let snapshot = store.install(candidate);

    assert_eq!(snapshot.state().entity_count(), 0);
    assert!(snapshot.indexes().matches(snapshot.state()));
}

#[test]
fn an_index_built_from_a_stale_state_is_reported_as_not_matching() {
    // `matches` has to be able to fail, or asserting it proves nothing.
    let mut state = ProjectState::empty(ProjectId::nil());
    let indexes = Indexes::build(&state);

    state.put_scene_item(item(SceneItemId::new(), SceneId::new(), SourceId::new()));

    assert!(!indexes.matches(&state));
}

#[test]
fn snapshot_retention_is_bounded() {
    // 106 invariant 8. Without a bound the store is a memory leak shaped like a
    // history: every commit would retain every entity that commit replaced.
    let mut store = store();

    for _ in 0..(RETAINED_SNAPSHOTS * 4) {
        let mut candidate = store.candidate();
        candidate.put_scene(scene(SceneId::new(), Vec::new()));
        store.install(candidate);
    }

    assert_eq!(store.retained_count(), RETAINED_SNAPSHOTS);
}

#[test]
fn a_dropped_generation_is_reported_as_absent_rather_than_guessed() {
    // The honest answer sends the consumer back for a fresh snapshot. A wrong
    // answer would have it apply patches against the wrong base.
    let mut store = store();

    for _ in 0..(RETAINED_SNAPSHOTS + 3) {
        let mut candidate = store.candidate();
        candidate.put_scene(scene(SceneId::new(), Vec::new()));
        store.install(candidate);
    }

    assert!(store.retained(StateGeneration::INITIAL).is_none());
    assert!(store.retained(store.generation()).is_some());
    assert!(
        store
            .retained(StateGeneration::from_raw(store.generation().get() - 1))
            .is_some()
    );
}

#[test]
fn a_snapshot_can_be_shared_across_threads() {
    // 106 invariant 5 and section 10, asserted at compile time: a store holding
    // a capture handle, a socket, or a GPU resource would not satisfy these
    // bounds, so the type system enforces what the document asks for.
    fn assert_shareable<T: Send + Sync + 'static>() {}

    assert_shareable::<Arc<Snapshot>>();
    assert_shareable::<ProjectState>();
    assert_shareable::<Indexes>();
}

#[test]
fn committing_a_large_project_stays_far_inside_the_acknowledgement_budget() {
    // ADR-0070 accepts an O(entities) commit and owes a measurement for it.
    // 601-performance-budgets.md allows 100 ms for the whole command
    // acknowledgement at p95; the state clone is one step inside that, so a
    // millisecond is already two orders of magnitude of headroom.
    //
    // The assertion is deliberately loose. A tight bound on a shared CI machine
    // fails for reasons that have nothing to do with this code; what must fail
    // is the regression that turns pointer copying into entity copying, and that
    // is a factor of hundreds, not percent.
    const ENTITIES: usize = 10_000;
    const BUDGET_MILLIS: u128 = 20;

    let mut store = store();
    let mut candidate = store.candidate();
    let source_id = SourceId::new();
    candidate.put_source(source(source_id));

    for _ in 0..ENTITIES {
        let scene_id = SceneId::new();
        let item_id = SceneItemId::new();
        candidate.put_scene(scene(scene_id, vec![item_id]));
        candidate.put_scene_item(item(item_id, scene_id, source_id));
    }

    store.install(candidate);
    assert!(store.snapshot().state().entity_count() > ENTITIES);

    // One more single-entity commit against the large project: this is the
    // shape of an ordinary user edit, and its cost is what the ADR claims is
    // proportional to the project rather than to the edit — but with a constant
    // small enough that the difference does not reach a person.
    //
    // Best of several, not one measurement. `cargo test` runs these in parallel
    // on a machine doing other work, so a single sample measures contention as
    // much as code.
    let best = (0..5)
        .map(|_| {
            let started = Instant::now();
            let mut candidate = store.candidate();
            candidate.put_scene(scene(SceneId::new(), Vec::new()));
            store.install(candidate);
            started.elapsed()
        })
        .min()
        .unwrap_or_default();

    assert!(
        best.as_millis() < BUDGET_MILLIS,
        "the fastest of five edits against {ENTITIES} entities took {best:?}, \
         which means the commit is copying entities rather than pointers"
    );
}

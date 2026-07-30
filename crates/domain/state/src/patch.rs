//! The projection a client mirrors, and the patches that advance it.
//!
//! Canonical documentation: `docs/01-runtime/106-state-store.md` section 8,
//! `docs/01-runtime/109-ui-engine-synchronization.md` sections 4 and 5.
//!
//! A client does not receive domain entities. It receives a *projection* — a
//! flat, serializable view built for replication — and then patches that carry
//! it forward one generation at a time. The separation is what lets the domain
//! model change without changing the wire, and it is why `109` section 5 can
//! list four conditions for applying a patch: same session, same projection
//! version, matching from-generation, and operations that validate.
//!
//! Every failure here has the same remedy, and it is deliberately the only one:
//! ask for a fresh snapshot. `109` section 5 marks the mirror stale, stops
//! applying dependent patches, and resynchronizes. A mirror that guessed instead
//! would be confidently wrong, which is worse than being visibly behind.

use std::collections::BTreeMap;

use mirae_types::{SceneId, SceneItemId, SourceId, StateGeneration};

use crate::project_state::ProjectState;
use crate::store::{PROJECTION_SCHEMA_VERSION, Snapshot};

/// A scene as a client sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneProjection {
    /// Display name.
    pub name: String,
    /// Scene items in composition order.
    pub items: Vec<SceneItemId>,
}

/// A source definition as a client sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProjection {
    /// Display name.
    pub name: String,
    /// The source kind, as a stable identifier rather than a domain enum.
    ///
    /// A string on the wire, because adding a source kind must not force every
    /// client to be rebuilt before it can display a project containing one.
    pub kind: String,
}

/// A scene item as a client sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneItemProjection {
    /// The scene this item belongs to.
    pub scene: SceneId,
    /// The source this item places.
    pub source: SourceId,
    /// Whether the item participates in composition.
    pub visible: bool,
}

/// The whole project, as a client mirrors it (`109` section 3.1).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectProjection {
    /// Scenes by identifier.
    pub scenes: BTreeMap<SceneId, SceneProjection>,
    /// Source definitions by identifier.
    pub sources: BTreeMap<SourceId, SourceProjection>,
    /// Scene items by identifier.
    pub scene_items: BTreeMap<SceneItemId, SceneItemProjection>,
}

impl ProjectProjection {
    /// Project authoritative state.
    #[must_use]
    pub fn of(state: &ProjectState) -> Self {
        Self {
            scenes: state
                .scenes()
                .map(|scene| {
                    (
                        scene.id,
                        SceneProjection {
                            name: scene.name.as_str().to_owned(),
                            items: scene.items.clone(),
                        },
                    )
                })
                .collect(),
            sources: state
                .sources()
                .map(|source| {
                    (
                        source.id,
                        SourceProjection {
                            name: source.name.as_str().to_owned(),
                            kind: match source.kind {
                                mirae_domain::SourceKind::Color => "color".to_owned(),
                                _ => "unknown".to_owned(),
                            },
                        },
                    )
                })
                .collect(),
            scene_items: state
                .scene_items()
                .map(|item| {
                    (
                        item.id,
                        SceneItemProjection {
                            scene: item.scene,
                            source: item.source,
                            visible: item.visible,
                        },
                    )
                })
                .collect(),
        }
    }
}

/// One change to a projection.
///
/// Upserts rather than field-level edits. A field-level operation set would be
/// smaller on the wire and would require the mirror to understand every field's
/// merge semantics; at the size of these entities that trade is not worth
/// making, and `106` section 8 asks for operations that validate, not for the
/// smallest possible ones.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PatchOperation {
    /// A scene was created or changed.
    UpsertScene(SceneId, SceneProjection),
    /// A scene no longer exists.
    RemoveScene(SceneId),
    /// A source definition was created or changed.
    UpsertSource(SourceId, SourceProjection),
    /// A source definition no longer exists.
    RemoveSource(SourceId),
    /// A scene item was created or changed.
    UpsertSceneItem(SceneItemId, SceneItemProjection),
    /// A scene item no longer exists.
    RemoveSceneItem(SceneItemId),
}

/// A patch advancing a mirror exactly one generation (`106` section 8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatePatch {
    /// The session that produced it.
    pub engine_session_id: String,
    /// The generation a mirror must be at to apply this.
    pub from_generation: StateGeneration,
    /// The generation it will be at afterwards.
    pub to_generation: StateGeneration,
    /// The projection schema this patch speaks.
    pub projection_schema_version: u32,
    /// What changed.
    pub operations: Vec<PatchOperation>,
}

/// Compute the patch between two projections.
///
/// Removals first, then upserts. Order matters for a consumer that indexes as it
/// applies: removing before adding means an index never briefly holds two
/// entries claiming the same thing.
#[must_use]
pub fn diff(previous: &ProjectProjection, next: &ProjectProjection) -> Vec<PatchOperation> {
    let mut operations = Vec::new();

    for id in previous.scene_items.keys() {
        if !next.scene_items.contains_key(id) {
            operations.push(PatchOperation::RemoveSceneItem(*id));
        }
    }

    for id in previous.scenes.keys() {
        if !next.scenes.contains_key(id) {
            operations.push(PatchOperation::RemoveScene(*id));
        }
    }

    for id in previous.sources.keys() {
        if !next.sources.contains_key(id) {
            operations.push(PatchOperation::RemoveSource(*id));
        }
    }

    for (id, source) in &next.sources {
        if previous.sources.get(id) != Some(source) {
            operations.push(PatchOperation::UpsertSource(*id, source.clone()));
        }
    }

    for (id, scene) in &next.scenes {
        if previous.scenes.get(id) != Some(scene) {
            operations.push(PatchOperation::UpsertScene(*id, scene.clone()));
        }
    }

    for (id, item) in &next.scene_items {
        if previous.scene_items.get(id) != Some(item) {
            operations.push(PatchOperation::UpsertSceneItem(*id, item.clone()));
        }
    }

    operations
}

/// Build the patch between two snapshots.
///
/// Returns `None` when the snapshots are not adjacent or come from different
/// sessions. A patch spanning more than one generation would defeat the gap
/// detection it exists to support.
#[must_use]
pub fn patch_between(previous: &Snapshot, next: &Snapshot) -> Option<StatePatch> {
    if previous.engine_session_id() != next.engine_session_id() {
        return None;
    }

    if !next.generation().immediately_follows(previous.generation()) {
        return None;
    }

    Some(StatePatch {
        engine_session_id: next.engine_session_id().to_owned(),
        from_generation: previous.generation(),
        to_generation: next.generation(),
        projection_schema_version: next.projection_schema_version(),
        operations: diff(
            &ProjectProjection::of(previous.state()),
            &ProjectProjection::of(next.state()),
        ),
    })
}

/// Why a mirror refused a patch (`109` section 5).
///
/// Every variant has the same remedy — request a fresh snapshot — but they are
/// distinct because they are distinct problems, and a diagnostic that says
/// "resynchronized" without saying why is a diagnostic nobody can act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirrorError {
    /// The patch came from a different engine session.
    WrongSession,
    /// The patch speaks a projection schema this mirror does not.
    WrongProjectionVersion,
    /// The patch does not start where this mirror is.
    ///
    /// A gap, a duplicate, and an out-of-order patch all land here: from the
    /// mirror's side they are the same fact, that the next patch is not the one
    /// that arrived.
    GenerationMismatch,
}

impl std::fmt::Display for MirrorError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::WrongSession => "the patch came from a different engine session",
            Self::WrongProjectionVersion => {
                "the patch uses a projection schema this client does not"
            }
            Self::GenerationMismatch => "the patch does not continue from this client's generation",
        })
    }
}

impl std::error::Error for MirrorError {}

/// A client-side replica of engine state (`109` section 3.1).
#[derive(Debug, Clone)]
pub struct Mirror {
    engine_session_id: String,
    generation: StateGeneration,
    projection_schema_version: u32,
    projection: ProjectProjection,
    stale: bool,
}

impl Mirror {
    /// Install a snapshot, replacing whatever the mirror held (`109` section 4).
    #[must_use]
    pub fn from_snapshot(snapshot: &Snapshot) -> Self {
        Self {
            engine_session_id: snapshot.engine_session_id().to_owned(),
            generation: snapshot.generation(),
            projection_schema_version: snapshot.projection_schema_version(),
            projection: ProjectProjection::of(snapshot.state()),
            stale: false,
        }
    }

    /// The generation this mirror is at.
    #[must_use]
    pub const fn generation(&self) -> StateGeneration {
        self.generation
    }

    /// Whether this mirror is known to be behind and must resynchronize.
    ///
    /// `109` section 5: a mirror that failed to apply a patch stops applying
    /// dependent ones. Staleness is sticky until a fresh snapshot arrives,
    /// because the patches after a missed one are exactly as unusable.
    #[must_use]
    pub const fn stale(&self) -> bool {
        self.stale
    }

    /// What the mirror currently believes.
    #[must_use]
    pub const fn projection(&self) -> &ProjectProjection {
        &self.projection
    }

    /// Apply a patch (`109` section 5).
    pub fn apply(&mut self, patch: &StatePatch) -> Result<(), MirrorError> {
        if self.stale {
            return Err(MirrorError::GenerationMismatch);
        }

        if patch.engine_session_id != self.engine_session_id {
            self.stale = true;
            return Err(MirrorError::WrongSession);
        }

        if patch.projection_schema_version != self.projection_schema_version {
            self.stale = true;
            return Err(MirrorError::WrongProjectionVersion);
        }

        if patch.from_generation != self.generation {
            self.stale = true;
            return Err(MirrorError::GenerationMismatch);
        }

        for operation in &patch.operations {
            match operation {
                PatchOperation::UpsertScene(id, scene) => {
                    self.projection.scenes.insert(*id, scene.clone());
                }
                PatchOperation::RemoveScene(id) => {
                    self.projection.scenes.remove(id);
                }
                PatchOperation::UpsertSource(id, source) => {
                    self.projection.sources.insert(*id, source.clone());
                }
                PatchOperation::RemoveSource(id) => {
                    self.projection.sources.remove(id);
                }
                PatchOperation::UpsertSceneItem(id, item) => {
                    self.projection.scene_items.insert(*id, item.clone());
                }
                PatchOperation::RemoveSceneItem(id) => {
                    self.projection.scene_items.remove(id);
                }
            }
        }

        self.generation = patch.to_generation;
        Ok(())
    }
}

/// The projection schema version this build speaks.
#[must_use]
pub const fn projection_schema_version() -> u32 {
    PROJECTION_SCHEMA_VERSION
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mirae_domain::{EntityName, Scene, SceneItem, SourceDefinition, SourceKind};
    use mirae_types::ProjectId;

    use super::*;
    use crate::store::StateStore;

    const SESSION: &str = "0000000000000000000000000000002a";

    fn name(text: &str) -> EntityName {
        EntityName::new(text).unwrap_or_else(|_| {
            EntityName::new("unnamed").unwrap_or_else(|_| unreachable!("a literal name is valid"))
        })
    }

    fn store() -> StateStore {
        StateStore::new(SESSION, ProjectId::nil())
    }

    /// Commit one change and return the snapshots either side of it.
    fn commit(
        store: &mut StateStore,
        build: impl FnOnce(&mut ProjectState),
    ) -> (Arc<Snapshot>, Arc<Snapshot>) {
        let before = store.snapshot();
        let mut transaction = store.transaction();
        let _ = transaction.prepare(|state| {
            build(state);
            Ok(())
        });
        let after = transaction
            .commit()
            .map(|outcome| outcome.snapshot)
            .unwrap_or_else(|_| store.snapshot());

        (before, after)
    }

    #[test]
    fn a_mirror_built_from_a_snapshot_matches_that_snapshot() {
        let mut store = store();
        let scene_id = SceneId::new();

        let (_, after) = commit(&mut store, |state| {
            state.put_scene(Scene {
                id: scene_id,
                name: name("Main"),
                items: Vec::new(),
            });
        });

        let mirror = Mirror::from_snapshot(&after);

        assert_eq!(mirror.generation(), StateGeneration::from_raw(1));
        assert!(!mirror.stale());
        assert_eq!(
            mirror
                .projection()
                .scenes
                .get(&scene_id)
                .map(|scene| scene.name.clone()),
            Some("Main".to_owned())
        );
    }

    #[test]
    fn a_patched_mirror_equals_a_mirror_built_from_the_later_snapshot() {
        // 106 section 14: snapshot and patch equivalence. This is the property
        // the whole protocol rests on — if it does not hold, a client that
        // stayed connected diverges from one that reconnected.
        let mut store = store();
        let scene_id = SceneId::new();
        let source_id = SourceId::new();
        let item_id = SceneItemId::new();

        let (_, first) = commit(&mut store, |state| {
            state.put_source(SourceDefinition {
                id: source_id,
                name: name("Colour"),
                kind: SourceKind::Color,
            });
        });

        let mut mirror = Mirror::from_snapshot(&first);

        let (before, after) = commit(&mut store, |state| {
            state.put_scene(Scene {
                id: scene_id,
                name: name("Main"),
                items: vec![item_id],
            });
            state.put_scene_item(SceneItem {
                id: item_id,
                scene: scene_id,
                source: source_id,
                visible: true,
            });
        });

        let patch = patch_between(&before, &after);

        assert!(patch.is_some());

        if let Some(patch) = patch {
            assert_eq!(mirror.apply(&patch), Ok(()));
        }

        assert_eq!(
            mirror.projection(),
            Mirror::from_snapshot(&after).projection()
        );
        assert_eq!(mirror.generation(), after.generation());
    }

    #[test]
    fn a_patch_carries_removals_before_upserts() {
        // A consumer that indexes as it applies must never briefly hold two
        // entries claiming the same thing.
        let mut previous = ProjectProjection::default();
        let doomed = SceneId::new();
        previous.scenes.insert(
            doomed,
            SceneProjection {
                name: "Old".to_owned(),
                items: Vec::new(),
            },
        );

        let mut next = ProjectProjection::default();
        next.scenes.insert(
            SceneId::new(),
            SceneProjection {
                name: "New".to_owned(),
                items: Vec::new(),
            },
        );

        let operations = diff(&previous, &next);

        assert!(matches!(
            operations.first(),
            Some(PatchOperation::RemoveScene(id)) if *id == doomed
        ));
        assert!(matches!(
            operations.last(),
            Some(PatchOperation::UpsertScene(..))
        ));
    }

    #[test]
    fn an_unchanged_entity_produces_no_operation() {
        // A patch that restates unchanged entities is a snapshot with extra
        // steps, and it would make the gap detection meaningless by making every
        // patch applicable.
        let mut store = store();
        let scene_id = SceneId::new();

        let (_, first) = commit(&mut store, |state| {
            state.put_scene(Scene {
                id: scene_id,
                name: name("Main"),
                items: Vec::new(),
            });
        });

        let (before, after) = commit(&mut store, |state| {
            state.put_source(SourceDefinition {
                id: SourceId::new(),
                name: name("Colour"),
                kind: SourceKind::Color,
            });
        });

        let _ = first;
        let operations = patch_between(&before, &after).map(|patch| patch.operations);

        assert_eq!(operations.as_ref().map(Vec::len), Some(1));
        assert!(matches!(
            operations.and_then(|operations| operations.into_iter().next()),
            Some(PatchOperation::UpsertSource(..))
        ));
    }

    #[test]
    fn a_gap_is_refused_and_makes_the_mirror_stale() {
        // 109 section 5: a mirror that cannot apply a patch stops applying
        // dependent ones and asks for a snapshot. Guessing would leave it
        // confidently wrong.
        let mut store = store();
        let (_, first) = commit(&mut store, |state| {
            state.put_scene(Scene {
                id: SceneId::new(),
                name: name("One"),
                items: Vec::new(),
            });
        });

        let mut mirror = Mirror::from_snapshot(&first);

        let (second_before, second_after) = commit(&mut store, |state| {
            state.put_scene(Scene {
                id: SceneId::new(),
                name: name("Two"),
                items: Vec::new(),
            });
        });
        let (_, third_after) = commit(&mut store, |state| {
            state.put_scene(Scene {
                id: SceneId::new(),
                name: name("Three"),
                items: Vec::new(),
            });
        });

        // Skip the second patch entirely.
        let skipped = patch_between(&second_after, &third_after);

        assert_eq!(
            skipped.as_ref().map(|patch| mirror.apply(patch)),
            Some(Err(MirrorError::GenerationMismatch))
        );
        assert!(mirror.stale());

        // The patch it actually needed is now refused too: after a gap, later
        // patches are exactly as unusable.
        let needed = patch_between(&second_before, &second_after);

        assert_eq!(
            needed.as_ref().map(|patch| mirror.apply(patch)),
            Some(Err(MirrorError::GenerationMismatch))
        );
    }

    #[test]
    fn a_duplicate_patch_is_refused_rather_than_applied_twice() {
        // 105 section 5 allows duplicates after a reconnect. Applying an upsert
        // twice would be harmless; applying a removal twice would not, and a
        // mirror should not have to reason about which.
        let mut store = store();
        let (before, after) = commit(&mut store, |state| {
            state.put_scene(Scene {
                id: SceneId::new(),
                name: name("Main"),
                items: Vec::new(),
            });
        });

        let mut mirror = Mirror::from_snapshot(&before);
        let patch = patch_between(&before, &after);

        assert_eq!(
            patch.as_ref().map(|patch| mirror.apply(patch)),
            Some(Ok(()))
        );
        assert_eq!(
            patch.as_ref().map(|patch| mirror.apply(patch)),
            Some(Err(MirrorError::GenerationMismatch))
        );
    }

    #[test]
    fn a_patch_from_another_session_is_refused() {
        // 109 invariant 8: a new engine session invalidates the old mirror. The
        // generations may even line up, which is exactly why the session is
        // checked first.
        let mut store = store();
        let (before, after) = commit(&mut store, |state| {
            state.put_scene(Scene {
                id: SceneId::new(),
                name: name("Main"),
                items: Vec::new(),
            });
        });

        let mut mirror = Mirror::from_snapshot(&before);
        let mut patch = patch_between(&before, &after).unwrap_or_else(|| StatePatch {
            engine_session_id: SESSION.to_owned(),
            from_generation: StateGeneration::INITIAL,
            to_generation: StateGeneration::from_raw(1),
            projection_schema_version: projection_schema_version(),
            operations: Vec::new(),
        });
        patch.engine_session_id = "a-different-session".to_owned();

        assert_eq!(mirror.apply(&patch), Err(MirrorError::WrongSession));
        assert!(mirror.stale());
    }

    #[test]
    fn a_patch_speaking_another_projection_version_is_refused() {
        let mut store = store();
        let (before, after) = commit(&mut store, |state| {
            state.put_scene(Scene {
                id: SceneId::new(),
                name: name("Main"),
                items: Vec::new(),
            });
        });

        let mut mirror = Mirror::from_snapshot(&before);
        let mut patch = patch_between(&before, &after).unwrap_or_else(|| StatePatch {
            engine_session_id: SESSION.to_owned(),
            from_generation: StateGeneration::INITIAL,
            to_generation: StateGeneration::from_raw(1),
            projection_schema_version: projection_schema_version(),
            operations: Vec::new(),
        });
        patch.projection_schema_version = projection_schema_version() + 1;

        assert_eq!(
            mirror.apply(&patch),
            Err(MirrorError::WrongProjectionVersion)
        );
        assert!(mirror.stale());
    }

    #[test]
    fn a_stale_mirror_recovers_only_through_a_fresh_snapshot() {
        let mut store = store();
        let (before, after) = commit(&mut store, |state| {
            state.put_scene(Scene {
                id: SceneId::new(),
                name: name("Main"),
                items: Vec::new(),
            });
        });

        let mut mirror = Mirror::from_snapshot(&before);
        let mut broken = patch_between(&before, &after).unwrap_or_else(|| StatePatch {
            engine_session_id: SESSION.to_owned(),
            from_generation: StateGeneration::INITIAL,
            to_generation: StateGeneration::from_raw(1),
            projection_schema_version: projection_schema_version(),
            operations: Vec::new(),
        });
        broken.from_generation = StateGeneration::from_raw(99);

        let _ = mirror.apply(&broken);
        assert!(mirror.stale());

        let recovered = Mirror::from_snapshot(&after);

        assert!(!recovered.stale());
        assert_eq!(recovered.generation(), after.generation());
    }

    #[test]
    fn non_adjacent_snapshots_produce_no_patch() {
        // A patch spanning more than one generation would defeat the gap
        // detection it exists to support.
        let mut store = store();
        let (initial, _) = commit(&mut store, |state| {
            state.put_scene(Scene {
                id: SceneId::new(),
                name: name("One"),
                items: Vec::new(),
            });
        });
        let (_, third) = commit(&mut store, |state| {
            state.put_scene(Scene {
                id: SceneId::new(),
                name: name("Two"),
                items: Vec::new(),
            });
        });

        assert!(patch_between(&initial, &third).is_none());
        assert!(
            patch_between(&third, &initial).is_none(),
            "and not backwards"
        );
    }

    #[test]
    fn a_removal_reaches_the_mirror() {
        let mut store = store();
        let scene_id = SceneId::new();

        let (_, first) = commit(&mut store, |state| {
            state.put_scene(Scene {
                id: scene_id,
                name: name("Main"),
                items: Vec::new(),
            });
        });

        let mut mirror = Mirror::from_snapshot(&first);

        let (before, after) = commit(&mut store, |state| {
            state.remove_scene(scene_id);
        });

        let patch = patch_between(&before, &after);

        assert_eq!(
            patch.as_ref().map(|patch| mirror.apply(patch)),
            Some(Ok(()))
        );
        assert!(mirror.projection().scenes.is_empty());
    }
}

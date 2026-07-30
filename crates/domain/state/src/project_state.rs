//! The authoritative project state and its derived indexes.
//!
//! Canonical documentation: `docs/01-runtime/106-state-store.md` sections 2, 6
//! and 10, ADR-0070.
//!
//! State is a `BTreeMap` per entity kind holding `Arc` entities. A commit clones
//! the maps' spines and the pointers, never the entities, which is the
//! structural sharing `106` section 5 asks for at the granularity where it pays:
//! renaming one scene copies `N` pointers and one scene.
//!
//! `BTreeMap` rather than `HashMap` because iteration ordered by identifier is
//! what `401-project-format.md` section 12 needs for deterministic
//! serialization. Getting it from the data structure is cheaper and harder to
//! forget than sorting at save time.

use std::collections::BTreeMap;
use std::sync::Arc;

use mirae_domain::{Scene, SceneItem, SourceDefinition};
use mirae_types::{ProjectId, SceneId, SceneItemId, SourceId};

/// The authoritative project-domain state (`106` section 2.1).
///
/// Persistable intent only. Nothing here is a handle, a socket, a device, or a
/// metric: `106` invariant 5 and section 11 keep runtime state out so that a
/// snapshot can be serialized and retained without dragging a thread-affine
/// object along with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectState {
    project_id: ProjectId,
    scenes: BTreeMap<SceneId, Arc<Scene>>,
    sources: BTreeMap<SourceId, Arc<SourceDefinition>>,
    scene_items: BTreeMap<SceneItemId, Arc<SceneItem>>,
}

impl ProjectState {
    /// An empty project.
    #[must_use]
    pub fn empty(project_id: ProjectId) -> Self {
        Self {
            project_id,
            scenes: BTreeMap::new(),
            sources: BTreeMap::new(),
            scene_items: BTreeMap::new(),
        }
    }

    /// The project this state belongs to.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Look up a scene.
    #[must_use]
    pub fn scene(&self, id: SceneId) -> Option<&Arc<Scene>> {
        self.scenes.get(&id)
    }

    /// Look up a source definition.
    #[must_use]
    pub fn source(&self, id: SourceId) -> Option<&Arc<SourceDefinition>> {
        self.sources.get(&id)
    }

    /// Look up a scene item.
    #[must_use]
    pub fn scene_item(&self, id: SceneItemId) -> Option<&Arc<SceneItem>> {
        self.scene_items.get(&id)
    }

    /// Every scene, ordered by identifier.
    pub fn scenes(&self) -> impl Iterator<Item = &Arc<Scene>> {
        self.scenes.values()
    }

    /// Every source definition, ordered by identifier.
    pub fn sources(&self) -> impl Iterator<Item = &Arc<SourceDefinition>> {
        self.sources.values()
    }

    /// Every scene item, ordered by identifier.
    pub fn scene_items(&self) -> impl Iterator<Item = &Arc<SceneItem>> {
        self.scene_items.values()
    }

    /// How many entities this state holds, for retention and diagnostics.
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.scenes.len() + self.sources.len() + self.scene_items.len()
    }

    /// Insert or replace a scene.
    ///
    /// Mutates a candidate, never authoritative state. `106` section 4 reserves
    /// commits to the transaction coordinator, and that rule is enforced on
    /// [`crate::StateStore::install`] rather than here: a caller holding a
    /// `ProjectState` by value is holding a proposal, and a proposal nobody
    /// installs changes nothing.
    pub fn put_scene(&mut self, scene: Scene) {
        self.scenes.insert(scene.id, Arc::new(scene));
    }

    /// Insert or replace a source definition.
    pub fn put_source(&mut self, source: SourceDefinition) {
        self.sources.insert(source.id, Arc::new(source));
    }

    /// Insert or replace a scene item.
    pub fn put_scene_item(&mut self, item: SceneItem) {
        self.scene_items.insert(item.id, Arc::new(item));
    }

    /// Remove a scene, returning whether it was there.
    pub fn remove_scene(&mut self, id: SceneId) -> bool {
        self.scenes.remove(&id).is_some()
    }

    /// Remove a scene item, returning whether it was there.
    pub fn remove_scene_item(&mut self, id: SceneItemId) -> bool {
        self.scene_items.remove(&id).is_some()
    }

    /// Remove a source definition, returning whether it was there.
    pub fn remove_source(&mut self, id: SourceId) -> bool {
        self.sources.remove(&id).is_some()
    }
}

/// Derived lookups over a [`ProjectState`] (`106` section 6).
///
/// Indexes are derived, never persisted, and never authoritative. They exist so
/// that "which items use this source" is not a scan of every scene item, and
/// they are rebuilt from the canonical collections rather than maintained
/// incrementally — an incremental index that drifts is a bug that reports itself
/// as missing data much later.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Indexes {
    items_by_scene: BTreeMap<SceneId, Vec<SceneItemId>>,
    items_by_source: BTreeMap<SourceId, Vec<SceneItemId>>,
}

impl Indexes {
    /// Build the indexes for `state`.
    #[must_use]
    pub fn build(state: &ProjectState) -> Self {
        let mut items_by_scene: BTreeMap<SceneId, Vec<SceneItemId>> = BTreeMap::new();
        let mut items_by_source: BTreeMap<SourceId, Vec<SceneItemId>> = BTreeMap::new();

        for item in state.scene_items() {
            items_by_scene.entry(item.scene).or_default().push(item.id);
            items_by_source
                .entry(item.source)
                .or_default()
                .push(item.id);
        }

        Self {
            items_by_scene,
            items_by_source,
        }
    }

    /// The scene items belonging to a scene, ordered by identifier.
    ///
    /// This is not composition order. Composition order is the scene's own item
    /// list, which is authoritative; this index answers membership.
    #[must_use]
    pub fn items_in_scene(&self, scene: SceneId) -> &[SceneItemId] {
        self.items_by_scene
            .get(&scene)
            .map_or(&[], |items| items.as_slice())
    }

    /// The scene items referencing a source definition.
    ///
    /// The question asked before a source is deleted, and the reason deleting one
    /// is not a matter of removing a map entry.
    #[must_use]
    pub fn items_using_source(&self, source: SourceId) -> &[SceneItemId] {
        self.items_by_source
            .get(&source)
            .map_or(&[], |items| items.as_slice())
    }

    /// Whether these indexes match `state`.
    ///
    /// `106` invariant 6 requires derived indexes to match the canonical
    /// entities. Stating it as a checkable predicate rather than a comment means
    /// a test can assert it after every commit, which is where drift would
    /// otherwise appear.
    #[must_use]
    pub fn matches(&self, state: &ProjectState) -> bool {
        *self == Self::build(state)
    }
}

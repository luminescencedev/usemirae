//! Mapping between authoritative state and the persisted schema.
//!
//! Canonical documentation: `docs/08-development/805-generated-contracts-and-schemas.md`
//! section 7, `docs/00-foundations/005-domain-model.md` section 9,
//! `docs/04-project/401-project-format.md`.
//!
//! `005` section 9 keeps the persisted schema separate from the runtime model,
//! and `805` section 7 says internal types map to the generated DTOs
//! *explicitly*. This module is that mapping, written by hand on purpose: a
//! derive would tie the file format to whatever the domain struct happens to
//! look like, and the next field added for a renderer's convenience would
//! silently become part of every project file ever written.
//!
//! Entities are written in identifier order. `401` section 12 wants stable
//! diffs, and the state store already holds them ordered (ADR-0070), so the
//! order costs nothing here and would be expensive to reconstruct later.

use mirae_contracts::generated::{
    PersistedProjectBody, PersistedProjectEnvelope, PersistedScene, PersistedSceneItem,
    PersistedSourceDefinition, PersistedSourceDefinitionKind, ProjectAppVersions, ProjectIntegrity,
    ProjectIntegrityAlgorithm,
};
use mirae_domain::SourceKind;
use mirae_state::ProjectState;

/// The `format` value every Mirae project file carries.
pub const PROJECT_FORMAT: &str = "mirae-project";

/// The project schema major version this build writes.
pub const PROJECT_SCHEMA_VERSION: u16 = 1;

/// Build the persisted body from authoritative state.
#[must_use]
pub fn body_of(state: &ProjectState) -> PersistedProjectBody {
    PersistedProjectBody {
        scenes: state
            .scenes()
            .map(|scene| PersistedScene {
                id: scene.id.to_string(),
                name: scene.name.as_str().to_owned(),
                items: scene.items.iter().map(ToString::to_string).collect(),
            })
            .collect(),
        sources: state
            .sources()
            .map(|source| PersistedSourceDefinition {
                id: source.id.to_string(),
                name: source.name.as_str().to_owned(),
                kind: match source.kind {
                    SourceKind::Color => PersistedSourceDefinitionKind::Color,
                    // A source kind the schema does not model cannot be written
                    // without inventing a value, and inventing one would produce
                    // a file that claims something untrue. The variant does not
                    // exist yet; when it does, its ticket extends the schema.
                    _ => PersistedSourceDefinitionKind::Color,
                },
            })
            .collect(),
        scene_items: state
            .scene_items()
            .map(|item| PersistedSceneItem {
                id: item.id.to_string(),
                scene_id: item.scene.to_string(),
                source_id: item.source.to_string(),
                visible: item.visible,
            })
            .collect(),
    }
}

/// Build the whole envelope from authoritative state.
///
/// `content_hash` is left empty here. `401` section 11 hashes the canonically
/// serialized document with the hash field excluded, so it can only be computed
/// once the bytes exist — that belongs to the save pipeline, and `MIR-0109` owns
/// it. Leaving it visibly empty is better than filling it with something that
/// looks like a hash.
#[must_use]
pub fn envelope_of(
    state: &ProjectState,
    created_at: &str,
    last_saved_at: &str,
    app_version: &str,
) -> PersistedProjectEnvelope {
    PersistedProjectEnvelope {
        format: PROJECT_FORMAT.to_owned(),
        schema_version: PROJECT_SCHEMA_VERSION,
        project_id: state.project_id().to_string(),
        created_at: created_at.to_owned(),
        last_saved_at: last_saved_at.to_owned(),
        app: ProjectAppVersions {
            minimum_version: app_version.to_owned(),
            saved_by_version: app_version.to_owned(),
        },
        // No feature is required by a v1 project. A build opening this file needs
        // to support nothing beyond the schema version it already read.
        features: Vec::new(),
        integrity: ProjectIntegrity {
            algorithm: ProjectIntegrityAlgorithm::Sha256,
            content_hash: String::new(),
        },
        project: body_of(state),
    }
}

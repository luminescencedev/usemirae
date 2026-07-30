//! Tests for opening and validating a project file.
//!
//! What `MIR-0110` owes: schema validation before semantic validation, an
//! unsupported required feature refused or opened read-only rather than ignored,
//! unresolved references preserved with diagnostics rather than deleted,
//! integrity mismatch reported, and a malformed file that cannot panic the
//! engine.

use mirae_state::StateStore;
use mirae_types::{ProjectId, SceneId, SceneItemId, SourceId, StateGeneration};

use crate::canonical::serialize_with_integrity;
use crate::mapping::envelope_of;
use crate::open::{Diagnostic, OpenError, OpenMode, open_document};
use mirae_contracts::generated::{
    PersistedProjectEnvelope, PersistedScene, PersistedSceneItem, PersistedSourceDefinition,
    PersistedSourceDefinitionKind,
};
use mirae_state::ProjectState;

const SESSION: &str = "0000000000000000000000000000002a";
const APP_VERSION: &str = "0.0.0";
const CREATED_AT: &str = "2026-07-30T12:00:00Z";

fn envelope() -> PersistedProjectEnvelope {
    envelope_of(
        &ProjectState::empty(ProjectId::nil()),
        CREATED_AT,
        CREATED_AT,
        APP_VERSION,
    )
}

/// Serialize an envelope the way a save would, hash included.
fn document(envelope: &PersistedProjectEnvelope) -> String {
    serialize_with_integrity(envelope)
        .map(|(text, _)| text)
        .unwrap_or_default()
}

fn open(text: &str) -> Result<crate::open::OpenedProject, OpenError> {
    open_document(text, SESSION)
}

#[test]
fn an_empty_project_opens_cleanly() {
    let opened = open(&document(&envelope()));

    assert!(opened.is_ok());

    let Ok(opened) = opened else { return };

    assert_eq!(opened.mode, OpenMode::ReadWrite);
    assert!(opened.diagnostics.is_empty());
    assert_eq!(opened.store.snapshot().project_id(), ProjectId::nil());
}

#[test]
fn opening_commits_a_generation_a_client_can_synchronize_against() {
    // An opened project sitting at the initial generation would be
    // indistinguishable from one where nothing has happened yet.
    let opened = open(&document(&envelope()));

    assert_eq!(
        opened.ok().map(|opened| opened.store.generation()),
        Some(StateGeneration::from_raw(1))
    );
}

#[test]
fn a_populated_project_round_trips_through_save_and_open() {
    // The property that matters across MIR-0109 and MIR-0110 together: what was
    // written comes back as what it was.
    let source_id = SourceId::new();
    let scene_id = SceneId::new();
    let item_id = SceneItemId::new();

    let mut envelope = envelope();
    envelope.project.sources.push(PersistedSourceDefinition {
        id: source_id.to_string(),
        name: "Background".to_owned(),
        kind: PersistedSourceDefinitionKind::Color,
    });
    envelope.project.scenes.push(PersistedScene {
        id: scene_id.to_string(),
        name: "Main".to_owned(),
        items: vec![item_id.to_string()],
    });
    envelope.project.scene_items.push(PersistedSceneItem {
        id: item_id.to_string(),
        scene_id: scene_id.to_string(),
        source_id: source_id.to_string(),
        visible: true,
    });

    let opened = open(&document(&envelope));

    let Ok(opened) = opened else {
        unreachable!("a well-formed project should open");
    };

    assert!(opened.diagnostics.is_empty());

    let snapshot = opened.store.snapshot();
    let state = snapshot.state();

    assert_eq!(state.entity_count(), 3);
    assert_eq!(
        state
            .scene(scene_id)
            .map(|scene| scene.name.as_str().to_owned()),
        Some("Main".to_owned())
    );
    assert_eq!(
        state.scene_item(item_id).map(|item| item.source),
        Some(source_id)
    );
    assert!(snapshot.indexes().matches(state));
}

#[test]
fn a_file_that_is_not_json_is_refused_without_panicking() {
    for text in [
        "",
        "not json at all",
        "{",
        "[]",
        "null",
        "{\"format\":",
        "\u{0}\u{1}\u{2}",
    ] {
        let outcome = open(text);

        assert!(outcome.is_err(), "{text:?} should be refused");
    }
}

#[test]
fn a_json_document_that_is_not_a_mirae_project_is_refused() {
    // Refused by the envelope layer, before anything asks about scenes.
    assert_eq!(open("{}").err(), Some(OpenError::NotAProject));
    assert_eq!(
        open("{\"format\": \"something-else\"}").err(),
        Some(OpenError::NotAProject)
    );
}

#[test]
fn a_future_schema_version_is_refused_by_version_rather_than_by_shape() {
    // 401 invariant 7 and 411 section 2.1. A file from a later build differs in
    // its fields *because* of its version, so reporting the field differences
    // would tell the user the wrong thing.
    let text = document(&envelope()).replace("\"schemaVersion\": 1", "\"schemaVersion\": 2");

    assert_eq!(
        open(&text).err(),
        Some(OpenError::UnsupportedSchemaVersion {
            found: 2,
            supported: 1
        })
    );
}

#[test]
fn a_missing_required_field_is_refused_as_malformed() {
    let text = document(&envelope()).replace("  \"createdAt\": \"2026-07-30T12:00:00Z\",\n", "");

    assert_eq!(open(&text).err(), Some(OpenError::Malformed));
}

#[test]
fn the_envelope_is_checked_before_the_schema() {
    // A document that is both a non-project and structurally wrong must be
    // reported as a non-project: 411 section 2 runs 2.1 before 2.2, so a user is
    // told the useful thing rather than the first thing.
    let text = "{\"format\": \"not-mirae\", \"schemaVersion\": \"nonsense\"}";

    assert_eq!(open(text).err(), Some(OpenError::NotAProject));
}

#[test]
fn an_altered_document_opens_with_an_integrity_diagnostic() {
    // 411 section 6: unrepairable does not imply refusing to open. The project
    // may be perfectly usable; the user is told what was noticed and decides.
    let text = document(&envelope()).replace(
        "\"savedByVersion\": \"0.0.0\"",
        "\"savedByVersion\": \"9.9.9\"",
    );

    let opened = open(&text);

    assert!(opened.is_ok(), "an altered file still opens");
    assert_eq!(
        opened.map(|opened| opened.diagnostics),
        Ok(vec![Diagnostic::IntegrityMismatch])
    );
}

#[test]
fn a_declared_feature_this_build_does_not_implement_opens_read_only() {
    // 401 invariant 7 and section 10. Read-only rather than refused: the user
    // can look at the project, and cannot silently save away what this build
    // could not represent.
    let mut envelope = envelope();
    envelope.features.push("multi-track-audio".to_owned());

    let opened = open(&document(&envelope));

    let Ok(opened) = opened else {
        unreachable!("a project declaring a feature should still open");
    };

    assert_eq!(opened.mode, OpenMode::ReadOnly);
    assert_eq!(
        opened.diagnostics,
        vec![Diagnostic::UnsupportedFeature {
            feature: "multi-track-audio".to_owned()
        }]
    );
    assert!(opened.diagnostics.iter().any(Diagnostic::forces_read_only));
}

#[test]
fn a_scene_item_pointing_at_a_missing_source_is_kept_with_a_diagnostic() {
    // The rule this ticket exists to get right. 411 section 6 and 005 section
    // 12: a dangling reference is a repair the user can make; a deletion is not
    // something they can undo, and they were never told about it.
    let scene_id = SceneId::new();
    let item_id = SceneItemId::new();
    let absent_source = SourceId::new();

    let mut envelope = envelope();
    envelope.project.scenes.push(PersistedScene {
        id: scene_id.to_string(),
        name: "Main".to_owned(),
        items: vec![item_id.to_string()],
    });
    envelope.project.scene_items.push(PersistedSceneItem {
        id: item_id.to_string(),
        scene_id: scene_id.to_string(),
        source_id: absent_source.to_string(),
        visible: true,
    });

    let opened = open(&document(&envelope));

    let Ok(opened) = opened else {
        unreachable!("an unresolved reference must not prevent opening");
    };

    assert_eq!(
        opened.diagnostics,
        vec![Diagnostic::UnresolvedSourceReference {
            item: item_id,
            source: absent_source,
        }]
    );
    assert!(
        opened
            .store
            .snapshot()
            .state()
            .scene_item(item_id)
            .is_some(),
        "the item survives so the user can repair it"
    );
    assert_eq!(opened.mode, OpenMode::ReadWrite);
}

#[test]
fn a_scene_listing_an_item_that_is_not_in_the_file_is_reported() {
    let scene_id = SceneId::new();
    let absent_item = SceneItemId::new();

    let mut envelope = envelope();
    envelope.project.scenes.push(PersistedScene {
        id: scene_id.to_string(),
        name: "Main".to_owned(),
        items: vec![absent_item.to_string()],
    });

    let opened = open(&document(&envelope));

    assert_eq!(
        opened.map(|opened| opened.diagnostics),
        Ok(vec![Diagnostic::MissingSceneItem {
            scene: scene_id,
            item: absent_item,
        }])
    );
}

#[test]
fn an_item_belonging_to_a_missing_scene_is_reported() {
    let item_id = SceneItemId::new();
    let source_id = SourceId::new();
    let absent_scene = SceneId::new();

    let mut envelope = envelope();
    envelope.project.sources.push(PersistedSourceDefinition {
        id: source_id.to_string(),
        name: "Background".to_owned(),
        kind: PersistedSourceDefinitionKind::Color,
    });
    envelope.project.scene_items.push(PersistedSceneItem {
        id: item_id.to_string(),
        scene_id: absent_scene.to_string(),
        source_id: source_id.to_string(),
        visible: true,
    });

    let opened = open(&document(&envelope));

    assert_eq!(
        opened.map(|opened| opened.diagnostics),
        Ok(vec![Diagnostic::UnresolvedSceneReference {
            item: item_id,
            scene: absent_scene,
        }])
    );
}

#[test]
fn an_unreadable_identifier_is_reported_and_the_rest_of_the_file_still_loads() {
    // One bad entity does not cost the user the other nine hundred.
    let good_source = SourceId::new();

    let mut envelope = envelope();
    envelope.project.sources.push(PersistedSourceDefinition {
        id: "not-a-uuid".to_owned(),
        name: "Broken".to_owned(),
        kind: PersistedSourceDefinitionKind::Color,
    });
    envelope.project.sources.push(PersistedSourceDefinition {
        id: good_source.to_string(),
        name: "Fine".to_owned(),
        kind: PersistedSourceDefinitionKind::Color,
    });

    let opened = open(&document(&envelope));

    let Ok(opened) = opened else {
        unreachable!("one unreadable entity should not refuse the file");
    };

    assert_eq!(
        opened.diagnostics,
        vec![Diagnostic::UnreadableIdentifier {
            collection: "sources"
        }]
    );
    assert!(
        opened
            .store
            .snapshot()
            .state()
            .source(good_source)
            .is_some()
    );
}

#[test]
fn a_duplicate_identifier_keeps_the_first_and_reports_the_second() {
    // 005 invariant 1: identifiers are unique within a namespace. Keeping the
    // first is arbitrary but deterministic, and the diagnostic says a choice was
    // made rather than hiding it.
    let repeated = SourceId::new();

    let mut envelope = envelope();
    for name in ["First", "Second"] {
        envelope.project.sources.push(PersistedSourceDefinition {
            id: repeated.to_string(),
            name: name.to_owned(),
            kind: PersistedSourceDefinitionKind::Color,
        });
    }

    let opened = open(&document(&envelope));

    let Ok(opened) = opened else {
        unreachable!("a duplicate should not refuse the file");
    };

    assert_eq!(
        opened.diagnostics,
        vec![Diagnostic::DuplicateIdentifier {
            collection: "sources"
        }]
    );
    assert_eq!(
        opened
            .store
            .snapshot()
            .state()
            .source(repeated)
            .map(|source| source.name.as_str().to_owned()),
        Some("First".to_owned())
    );
}

#[test]
fn a_name_that_the_domain_refuses_is_reported_rather_than_silently_trimmed() {
    let mut envelope = envelope();
    envelope.project.sources.push(PersistedSourceDefinition {
        id: SourceId::new().to_string(),
        name: "line\nbreak".to_owned(),
        kind: PersistedSourceDefinitionKind::Color,
    });

    let opened = open(&document(&envelope));

    assert_eq!(
        opened.map(|opened| opened.diagnostics),
        Ok(vec![Diagnostic::UnusableName {
            collection: "sources"
        }])
    );
}

#[test]
fn every_diagnostic_has_a_stable_lowercase_code() {
    // 411 section 3 wants a stable issue code, because a code can be counted,
    // matched, and translated. A message can only be printed.
    for diagnostic in [
        Diagnostic::IntegrityMismatch,
        Diagnostic::UnsupportedFeature {
            feature: "x".to_owned(),
        },
        Diagnostic::UnreadableIdentifier {
            collection: "sources",
        },
        Diagnostic::UnusableName {
            collection: "scenes",
        },
        Diagnostic::DuplicateIdentifier {
            collection: "sceneItems",
        },
        Diagnostic::UnresolvedSourceReference {
            item: SceneItemId::nil(),
            source: SourceId::nil(),
        },
        Diagnostic::UnresolvedSceneReference {
            item: SceneItemId::nil(),
            scene: SceneId::nil(),
        },
        Diagnostic::MissingSceneItem {
            scene: SceneId::nil(),
            item: SceneItemId::nil(),
        },
    ] {
        assert!(!diagnostic.code().is_empty());
        assert_eq!(diagnostic.code(), diagnostic.code().to_ascii_lowercase());
    }
}

#[test]
fn a_project_identifier_that_is_not_an_identifier_is_refused() {
    // The project id is the one identifier nothing can carry on without. An
    // entity can be reported and skipped; a project with no identity cannot be
    // opened at all.
    let text = document(&envelope()).replace(
        "\"projectId\": \"00000000-0000-0000-0000-000000000000\"",
        "\"projectId\": \"not-a-uuid\"",
    );

    assert_eq!(open(&text).err(), Some(OpenError::Malformed));
}

#[test]
fn opening_does_not_disturb_an_existing_store() {
    // Opening builds its own store. A caller holding one for another project
    // keeps it, which is what makes "close first" a decision rather than a race.
    let existing = StateStore::new(SESSION, ProjectId::new());
    let generation = existing.generation();

    let _ = open(&document(&envelope()));

    assert_eq!(existing.generation(), generation);
}

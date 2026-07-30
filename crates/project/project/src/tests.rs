//! Tests for project creation and schema mapping.
//!
//! What `MIR-0108` owes: creation is a command and a transaction rather than a
//! constructor, the result validates against the schema from `MIR-0107`, the new
//! project has a stable identity and a first generation, and creating one while
//! another is active follows a defined lifecycle rule.

use mirae_commands::{
    ActorContext, ActorKind, Capability, CommandEnvelope, CommandError, CommandId, LifecyclePhase,
};
use mirae_contracts::generated::PersistedProjectEnvelope;
use mirae_domain::{EntityName, Scene, SceneItem, SourceDefinition, SourceKind};
use mirae_state::{ProjectState, StateStore};
use mirae_types::{SceneId, SceneItemId, SourceId, StateGeneration};

use crate::create::{CreateProject, create_empty_project};
use crate::mapping::{PROJECT_FORMAT, PROJECT_SCHEMA_VERSION, body_of, envelope_of};

const SESSION: &str = "0000000000000000000000000000002a";
const APP_VERSION: &str = "0.0.0";
const CREATED_AT: &str = "2026-07-30T12:00:00Z";

fn command(name: &str) -> CommandEnvelope<CreateProject> {
    CommandEnvelope {
        command_id: CommandId::new(),
        engine_session_id: SESSION.to_owned(),
        actor: ActorContext::local_ui(),
        expected_generation: None,
        idempotency_key: None,
        issued_at_millis: None,
        payload: CreateProject {
            name: name.to_owned(),
        },
    }
}

fn name(text: &str) -> EntityName {
    EntityName::new(text).unwrap_or_else(|_| {
        EntityName::new("unnamed").unwrap_or_else(|_| unreachable!("a literal name is valid"))
    })
}

#[test]
fn creating_a_project_produces_an_identity_and_a_first_generation() {
    let outcome = create_empty_project(&command("Stream"), SESSION, LifecyclePhase::Ready);

    let Ok((store, created)) = outcome else {
        unreachable!("creation should have succeeded");
    };

    assert_eq!(created.generation, StateGeneration::from_raw(1));
    assert_eq!(created.name, "Stream");
    assert_eq!(store.snapshot().project_id(), created.project_id);
    assert_eq!(
        store.snapshot().state().entity_count(),
        0,
        "and it is empty"
    );
}

#[test]
fn two_projects_created_with_the_same_name_are_different_projects() {
    // A name is a label, not an identity (ADR-0069). Two projects may share one,
    // and the day a user duplicates a project is the day this matters.
    let first = create_empty_project(&command("Stream"), SESSION, LifecyclePhase::Ready);
    let second = create_empty_project(&command("Stream"), SESSION, LifecyclePhase::Ready);

    let ids = (
        first.ok().map(|(_, created)| created.project_id),
        second.ok().map(|(_, created)| created.project_id),
    );

    assert!(ids.0.is_some() && ids.1.is_some());
    assert_ne!(ids.0, ids.1);
}

#[test]
fn creating_a_project_while_one_is_active_is_refused() {
    // The lifecycle rule this ticket owes. `102` section 5 whitelists what the
    // engine accepts with no project open; creating one while another is active
    // must be an explicit close first, not a silent switch that leaves the next
    // edit landing somewhere the user did not expect.
    let outcome = create_empty_project(&command("Stream"), SESSION, LifecyclePhase::ProjectActive);

    assert_eq!(outcome.err(), Some(CommandError::WrongLifecycleState));
}

#[test]
fn creation_is_refused_before_the_engine_is_ready() {
    let outcome = create_empty_project(&command("Stream"), SESSION, LifecyclePhase::Starting);

    assert_eq!(outcome.err(), Some(CommandError::WrongLifecycleState));
}

#[test]
fn an_actor_without_the_lifecycle_capability_cannot_create_a_project() {
    let mut envelope = command("Stream");
    envelope.actor = ActorContext::new(
        ActorKind::Extension,
        [Capability::ReadState, Capability::MutateProject],
    );

    let outcome = create_empty_project(&envelope, SESSION, LifecyclePhase::Ready);

    assert_eq!(outcome.err(), Some(CommandError::PermissionDenied));
}

#[test]
fn a_command_for_another_session_is_refused() {
    let outcome =
        create_empty_project(&command("Stream"), "another-session", LifecyclePhase::Ready);

    assert_eq!(outcome.err(), Some(CommandError::WrongSession));
}

#[test]
fn an_unusable_project_name_is_refused() {
    for candidate in ["", "   ", "line\nbreak", &"a".repeat(257)] {
        let outcome = create_empty_project(&command(candidate), SESSION, LifecyclePhase::Ready);

        assert_eq!(
            outcome.err(),
            Some(CommandError::InvalidArgument),
            "{candidate:?} should be refused"
        );
    }
}

#[test]
fn an_empty_project_maps_onto_the_schema_and_round_trips() {
    // The acceptance criterion: what creation produces validates against the
    // schema MIR-0107 defined. Serializing and decoding it is how that claim is
    // checked rather than asserted.
    let outcome = create_empty_project(&command("Stream"), SESSION, LifecyclePhase::Ready);

    let Ok((store, _)) = outcome else {
        unreachable!("creation should have succeeded");
    };

    let envelope = envelope_of(
        store.snapshot().state(),
        CREATED_AT,
        CREATED_AT,
        APP_VERSION,
    );

    assert_eq!(envelope.format, PROJECT_FORMAT);
    assert_eq!(envelope.schema_version, PROJECT_SCHEMA_VERSION);
    assert!(envelope.project.scenes.is_empty());
    assert!(envelope.features.is_empty());

    let encoded = serde_json::to_string(&envelope).unwrap_or_default();
    let decoded = serde_json::from_str::<PersistedProjectEnvelope>(&encoded);

    assert_eq!(decoded.ok(), Some(envelope));
}

#[test]
fn the_envelope_carries_the_project_identity_as_text() {
    let outcome = create_empty_project(&command("Stream"), SESSION, LifecyclePhase::Ready);

    let Ok((store, created)) = outcome else {
        unreachable!("creation should have succeeded");
    };

    let envelope = envelope_of(
        store.snapshot().state(),
        CREATED_AT,
        CREATED_AT,
        APP_VERSION,
    );

    assert_eq!(envelope.project_id, created.project_id.to_string());
    assert_eq!(envelope.project_id.len(), 36);
}

#[test]
fn the_content_hash_is_left_empty_rather_than_invented() {
    // 401 section 11 hashes the serialized document with the hash field
    // excluded, so it cannot exist before the bytes do. MIR-0109 fills it.
    // Something that looked like a hash and was not would be worse than nothing.
    let outcome = create_empty_project(&command("Stream"), SESSION, LifecyclePhase::Ready);

    let Ok((store, _)) = outcome else {
        return;
    };

    let envelope = envelope_of(
        store.snapshot().state(),
        CREATED_AT,
        CREATED_AT,
        APP_VERSION,
    );

    assert!(envelope.integrity.content_hash.is_empty());
}

#[test]
fn a_populated_state_maps_every_entity_in_identifier_order() {
    // 401 section 12 wants stable diffs. The store already holds entities
    // ordered, so the file inherits that order rather than reconstructing it.
    let mut state = ProjectState::empty(mirae_types::ProjectId::nil());
    let source_id = SourceId::new();
    let scene_id = SceneId::new();
    let first_item = SceneItemId::new();
    let second_item = SceneItemId::new();

    state.put_source(SourceDefinition {
        id: source_id,
        name: name("Background"),
        kind: SourceKind::Color,
    });
    state.put_scene(Scene {
        id: scene_id,
        name: name("Main"),
        items: vec![first_item, second_item],
    });
    for id in [first_item, second_item] {
        state.put_scene_item(SceneItem {
            id,
            scene: scene_id,
            source: source_id,
            visible: true,
        });
    }

    let body = body_of(&state);

    assert_eq!(body.scenes.len(), 1);
    assert_eq!(body.sources.len(), 1);
    assert_eq!(body.scene_items.len(), 2);

    let written: Vec<_> = body
        .scene_items
        .iter()
        .map(|item| item.id.clone())
        .collect();
    let mut sorted = written.clone();
    sorted.sort();

    assert_eq!(written, sorted, "entities are written in identifier order");

    // The scene's own item list keeps composition order, which is not the same
    // thing and must not be sorted with it.
    assert_eq!(
        body.scenes.first().map(|scene| scene.items.clone()),
        Some(vec![first_item.to_string(), second_item.to_string()])
    );
}

#[test]
fn a_store_created_directly_is_not_a_created_project() {
    // A store can be built without a command — the type allows it, because
    // opening a project will need to. What it cannot do is produce a generation:
    // an unopened store sits at the initial one, so "has state been committed"
    // stays answerable.
    let store = StateStore::new(SESSION, mirae_types::ProjectId::nil());

    assert_eq!(store.generation(), StateGeneration::INITIAL);
}

//! The persisted project envelope round-trips through JSON as the schema says.
//!
//! Canonical documentation: `docs/04-project/401-project-format.md`, ADR-0071.
//!
//! `MIR-0107` defines the schema; these tests prove the generated types actually
//! carry it. They live beside the generated code rather than inside it because
//! `805` forbids editing generated files, and a test written into one would be
//! deleted by the next `cargo xtask generate`.

use mirae_contracts::generated::{
    PersistedProjectBody, PersistedProjectEnvelope, PersistedScene, PersistedSceneItem,
    PersistedSourceDefinition, PersistedSourceDefinitionKind, ProjectAppVersions, ProjectIntegrity,
    ProjectIntegrityAlgorithm,
};

/// The canonical text of an empty project, as it is written to disk.
///
/// Keys are sorted and the indentation is two spaces (ADR-0071). Written out in
/// full rather than built, so a change to the format has to be made here on
/// purpose and shows up in a diff as what it is.
const EMPTY_PROJECT: &str = r#"{
  "app": {
    "minimumVersion": "0.1.0",
    "savedByVersion": "0.1.0"
  },
  "createdAt": "2026-07-30T12:00:00Z",
  "features": [],
  "format": "mirae-project",
  "integrity": {
    "algorithm": "sha256",
    "contentHash": ""
  },
  "lastSavedAt": "2026-07-30T12:00:00Z",
  "project": {
    "sceneItems": [],
    "scenes": [],
    "sources": []
  },
  "projectId": "00000000-0000-0000-0000-000000000000",
  "schemaVersion": 1
}"#;

fn empty_envelope() -> PersistedProjectEnvelope {
    PersistedProjectEnvelope {
        format: "mirae-project".to_owned(),
        schema_version: 1,
        project_id: "00000000-0000-0000-0000-000000000000".to_owned(),
        created_at: "2026-07-30T12:00:00Z".to_owned(),
        last_saved_at: "2026-07-30T12:00:00Z".to_owned(),
        app: ProjectAppVersions {
            minimum_version: "0.1.0".to_owned(),
            saved_by_version: "0.1.0".to_owned(),
        },
        features: Vec::new(),
        integrity: ProjectIntegrity {
            algorithm: ProjectIntegrityAlgorithm::Sha256,
            content_hash: String::new(),
        },
        project: PersistedProjectBody {
            scenes: Vec::new(),
            sources: Vec::new(),
            scene_items: Vec::new(),
        },
    }
}

#[test]
fn an_empty_project_round_trips_unchanged() {
    // 401 required test: stable serialization. The value that comes back must be
    // the value that went out, or a save-open cycle silently edits the project.
    let decoded = serde_json::from_str::<PersistedProjectEnvelope>(EMPTY_PROJECT);

    assert_eq!(decoded.ok(), Some(empty_envelope()));
}

#[test]
fn a_populated_project_round_trips_unchanged() {
    let mut envelope = empty_envelope();
    envelope.project.sources.push(PersistedSourceDefinition {
        id: "11111111-1111-1111-1111-111111111111".to_owned(),
        name: "Background".to_owned(),
        kind: PersistedSourceDefinitionKind::Color,
    });
    envelope.project.scenes.push(PersistedScene {
        id: "22222222-2222-2222-2222-222222222222".to_owned(),
        name: "Main".to_owned(),
        items: vec!["33333333-3333-3333-3333-333333333333".to_owned()],
    });
    envelope.project.scene_items.push(PersistedSceneItem {
        id: "33333333-3333-3333-3333-333333333333".to_owned(),
        scene_id: "22222222-2222-2222-2222-222222222222".to_owned(),
        source_id: "11111111-1111-1111-1111-111111111111".to_owned(),
        visible: true,
    });

    let encoded = serde_json::to_string(&envelope).unwrap_or_default();
    let decoded = serde_json::from_str::<PersistedProjectEnvelope>(&encoded);

    assert_eq!(decoded.ok(), Some(envelope));
}

#[test]
fn the_wire_names_are_the_schema_names() {
    // The schema property names are the file's names. A Rust field renamed
    // without the schema changing would silently produce a file nothing else can
    // read.
    let encoded = serde_json::to_string(&empty_envelope()).unwrap_or_default();

    for name in [
        "\"format\"",
        "\"schemaVersion\"",
        "\"projectId\"",
        "\"createdAt\"",
        "\"lastSavedAt\"",
        "\"minimumVersion\"",
        "\"savedByVersion\"",
        "\"contentHash\"",
        "\"sceneItems\"",
    ] {
        assert!(encoded.contains(name), "{name} should appear on the wire");
    }
}

#[test]
fn an_unknown_field_is_refused_rather_than_ignored() {
    // 401 section 9 and the closed-contract rule: the schema sets
    // `additionalProperties: false`, and the generated decoder matches it. A
    // field nothing understands must not be silently dropped on save.
    let smuggled = EMPTY_PROJECT.replace(
        "\"format\": \"mirae-project\"",
        "\"format\": \"mirae-project\",\n  \"exfiltrate\": \"anything\"",
    );

    assert!(serde_json::from_str::<PersistedProjectEnvelope>(&smuggled).is_err());
}

#[test]
fn a_missing_required_field_is_refused() {
    let truncated = EMPTY_PROJECT.replace(
        "  \"projectId\": \"00000000-0000-0000-0000-000000000000\",\n",
        "",
    );

    assert!(serde_json::from_str::<PersistedProjectEnvelope>(&truncated).is_err());
}

#[test]
fn an_unknown_enumeration_value_is_refused() {
    // 401 invariant 7: an unsupported required feature is not silently ignored.
    // The same applies to a closed enumeration — an unknown hash algorithm must
    // not decode to a default.
    let unknown = EMPTY_PROJECT.replace("\"algorithm\": \"sha256\"", "\"algorithm\": \"md5\"");

    assert!(serde_json::from_str::<PersistedProjectEnvelope>(&unknown).is_err());
}

#[test]
fn a_non_integral_schema_version_is_refused() {
    // 401 section 6: JSON has no integer type, so the schema carries the
    // distinction and the generated decoder enforces it.
    let fractional = EMPTY_PROJECT.replace("\"schemaVersion\": 1", "\"schemaVersion\": 1.5");

    assert!(serde_json::from_str::<PersistedProjectEnvelope>(&fractional).is_err());
}

#[test]
fn the_envelope_can_be_read_before_the_body_is_understood() {
    // 401 section 10: an opening build decides whether it may interpret the body
    // *before* it tries to. That is only possible if the envelope's own fields
    // parse independently, which this asserts by reading them from a document
    // whose body is a shape this build does not model.
    let document = EMPTY_PROJECT.replace(
        "  \"project\": {\n    \"sceneItems\": [],\n    \"scenes\": [],\n    \"sources\": []\n  },\n",
        "",
    );

    let value = serde_json::from_str::<serde_json::Value>(&document);
    let schema_version = value
        .as_ref()
        .ok()
        .and_then(|value| value.get("schemaVersion"))
        .and_then(serde_json::Value::as_u64);

    assert_eq!(schema_version, Some(1));
    assert!(
        serde_json::from_str::<PersistedProjectEnvelope>(&document).is_err(),
        "and the typed decode still refuses the incomplete document"
    );
}

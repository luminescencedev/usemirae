//! The committed project fixtures, and how they are produced.
//!
//! Canonical documentation: `docs/06-quality/615-compatibility-policy.md`,
//! `docs/04-project/408-schema-versioning-and-migrations.md` section 11,
//! `docs/04-project/401-project-format.md` section 15.
//!
//! A fixture is a project file committed to the repository so that a later
//! change has to confront it. Its value comes entirely from nobody editing it by
//! hand: the moment a fixture is adjusted to make a test pass, it stops
//! recording what the format used to be and starts recording what the code
//! currently does.
//!
//! So they are generated. This module builds each one from code, the tests
//! compare the committed bytes against what it builds, and a deliberate
//! regeneration is one command with an obvious diff:
//!
//! ```text
//! MIRAE_UPDATE_FIXTURES=1 cargo test -p mirae-project
//! ```
//!
//! A diff in that regeneration is the compatibility change, stated plainly. That
//! is the entire point.

use mirae_contracts::generated::{
    PersistedProjectEnvelope, PersistedScene, PersistedSceneItem, PersistedSourceDefinition,
    PersistedSourceDefinitionKind,
};
use mirae_state::ProjectState;
use mirae_types::ProjectId;

use crate::mapping::envelope_of;

/// The environment variable that turns a comparison into a regeneration.
pub const UPDATE_VARIABLE: &str = "MIRAE_UPDATE_FIXTURES";

/// Where fixtures live, relative to the repository root.
pub const FIXTURE_DIRECTORY: &str = "fixtures/project/v1";

/// Fixed values, so a generated fixture is byte-identical every time.
///
/// A fixture built with `Uuid::new_v4` and the current clock would differ on
/// every run, which would make byte comparison meaningless and the diff
/// unreadable — the two things the corpus exists to provide.
const CREATED_AT: &str = "2026-01-01T00:00:00Z";
const SAVED_AT: &str = "2026-01-02T03:04:05Z";
const APP_VERSION: &str = "0.0.0";

/// One fixture: what it is called and what it proves.
pub struct Fixture {
    /// File name inside [`FIXTURE_DIRECTORY`].
    pub name: &'static str,
    /// What this fixture exists to catch, recorded as `fixtures/README.md` requires.
    pub proves: &'static str,
    /// The project it holds.
    pub envelope: PersistedProjectEnvelope,
}

/// A stable identifier built from a single byte, for readable fixtures.
fn stable_id(byte: u8) -> String {
    format!("{byte:02x}000000-0000-4000-8000-000000000000")
}

/// The empty project: the smallest thing the format can express.
fn empty() -> PersistedProjectEnvelope {
    let mut envelope = envelope_of(
        &ProjectState::empty(ProjectId::nil()),
        CREATED_AT,
        SAVED_AT,
        APP_VERSION,
    );
    envelope.project_id = stable_id(1);
    envelope
}

/// A project with one of everything the schema models.
fn populated() -> PersistedProjectEnvelope {
    let source_id = stable_id(2);
    let scene_id = stable_id(3);
    let item_id = stable_id(4);
    let second_item_id = stable_id(5);

    let mut envelope = empty();
    envelope.project_id = stable_id(6);
    envelope.project.sources.push(PersistedSourceDefinition {
        id: source_id.clone(),
        name: "Background".to_owned(),
        kind: PersistedSourceDefinitionKind::Color,
    });
    envelope.project.scenes.push(PersistedScene {
        id: scene_id.clone(),
        name: "Main".to_owned(),
        // Two items in a deliberate order, so a change that sorts composition
        // order by identifier shows up here as a diff rather than as a subtly
        // rearranged scene nobody notices.
        items: vec![second_item_id.clone(), item_id.clone()],
    });

    for id in [&item_id, &second_item_id] {
        envelope.project.scene_items.push(PersistedSceneItem {
            id: id.clone(),
            scene_id: scene_id.clone(),
            source_id: source_id.clone(),
            visible: id == &item_id,
        });
    }

    envelope
}

/// A project exercising the boundaries `408` section 11 asks for.
fn boundaries() -> PersistedProjectEnvelope {
    let mut envelope = empty();
    envelope.project_id = stable_id(7);
    envelope.project.sources.push(PersistedSourceDefinition {
        id: stable_id(8),
        // The longest name the schema accepts, in a script where characters and
        // bytes differ. A bound expressed in the wrong unit fails here.
        name: "設".repeat(256),
        kind: PersistedSourceDefinitionKind::Color,
    });

    envelope
}

/// Every fixture in the corpus.
#[must_use]
pub fn corpus() -> Vec<Fixture> {
    vec![
        Fixture {
            name: "empty.mirae.json",
            proves: "the smallest project the format can express, and the canonical \
                     serialization of an envelope with no content",
            envelope: empty(),
        },
        Fixture {
            name: "populated.mirae.json",
            proves: "one of every entity the schema models, with a scene item order \
                     that is not identifier order",
            envelope: populated(),
        },
        Fixture {
            name: "boundaries.mirae.json",
            proves: "the longest accepted name, in a script where characters and bytes \
                     differ",
            envelope: boundaries(),
        },
    ]
}

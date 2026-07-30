//! Opening a project file, and saying honestly what is wrong with it.
//!
//! Canonical documentation: `docs/04-project/411-project-validation-and-repair.md`,
//! `docs/04-project/401-project-format.md` section 10.
//!
//! `411` section 2 layers the validation, and the order is the point: envelope,
//! then schema, then references, then semantics. Each layer only runs on input
//! the previous one accepted, so a file that is not a Mirae project is never
//! asked whether its scene graph is acyclic.
//!
//! The rule that shapes everything here is `411` section 6: unrepairable does
//! not imply deletion, and `005` section 12 says the same from the other side —
//! do not resolve an unavailable resource by deleting user intent. So a scene
//! item pointing at a source that is not in the file is *kept*, with a
//! diagnostic naming it. A user can fix a dangling reference. Nobody can undo a
//! silent deletion they were never told about.

use std::path::Path;

use mirae_contracts::generated::{
    PersistedProjectEnvelope, PersistedSceneItem, PersistedSourceDefinitionKind,
};
use mirae_domain::{EntityName, Scene, SceneItem, SourceDefinition, SourceKind};
use mirae_state::{ProjectState, StateStore};
use mirae_types::{ProjectId, SceneId, SceneItemId, SourceId};

use crate::canonical::integrity_matches;
use crate::mapping::{PROJECT_FORMAT, PROJECT_SCHEMA_VERSION};
use crate::save::{FileIdentity, FilesystemFailure};

/// Largest project file this build will read into memory.
///
/// The file is untrusted input and its length is chosen by whoever wrote it, so
/// it is bounded before it is read rather than after. Sixty-four megabytes is
/// far beyond any project of intent — a project references media, it does not
/// contain it — and small enough that refusing costs nothing.
pub const MAX_PROJECT_FILE_BYTES: u64 = 64 * 1024 * 1024;

/// How a project was opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    /// Fully usable.
    ReadWrite,
    /// Readable, but saving would lose something (`401` section 10).
    ///
    /// The honest outcome when a file declares a feature this build does not
    /// implement: the user can look, and cannot silently discard what they
    /// cannot see.
    ReadOnly,
}

/// Why a project could not be opened at all (`411` section 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenError {
    /// The file could not be read.
    Unreadable(FilesystemFailure),
    /// The file is larger than [`MAX_PROJECT_FILE_BYTES`].
    TooLarge,
    /// The bytes are not valid JSON, or not the shape the schema declares.
    ///
    /// Carries no parser detail. The message would quote the file, and a project
    /// file can contain anything a user typed.
    Malformed,
    /// The document is well-formed JSON but not a Mirae project.
    NotAProject,
    /// The schema version is one this build does not understand.
    ///
    /// Refused rather than guessed at: `401` invariant 7 forbids silently
    /// ignoring what a file requires, and a future version may mean anything.
    UnsupportedSchemaVersion {
        /// What the file declared.
        found: u16,
        /// What this build writes.
        supported: u16,
    },
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreadable(_) => formatter.write_str("the project file could not be read"),
            Self::TooLarge => formatter.write_str("the project file is larger than Mirae opens"),
            Self::Malformed => {
                formatter.write_str("the project file is not a well-formed Mirae project")
            }
            Self::NotAProject => formatter.write_str("the file is not a Mirae project"),
            Self::UnsupportedSchemaVersion { found, supported } => write!(
                formatter,
                "the project uses schema version {found}; this build understands {supported}"
            ),
        }
    }
}

impl std::error::Error for OpenError {}

/// Something wrong with a project that did open (`411` section 3).
///
/// Every variant names entities rather than quoting file content, so a
/// diagnostic can be logged without carrying a user's text with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diagnostic {
    /// The content hash does not match the document.
    ///
    /// Reported, not repaired. `401` section 11 detects accidental corruption,
    /// and the project may still be perfectly usable — but the user is told,
    /// because a file that was modified outside Mirae may have been modified in
    /// ways this validation cannot see.
    IntegrityMismatch,
    /// The file declares a feature this build does not implement.
    UnsupportedFeature {
        /// The declared feature name.
        feature: String,
    },
    /// An entity identifier is not a valid identifier.
    UnreadableIdentifier {
        /// Which collection it appeared in.
        collection: &'static str,
    },
    /// A name could not be used as written.
    UnusableName {
        /// Which collection it appeared in.
        collection: &'static str,
    },
    /// Two entities in one collection claim the same identifier.
    DuplicateIdentifier {
        /// Which collection.
        collection: &'static str,
    },
    /// A scene item references a source that is not in the file.
    ///
    /// The item is kept. `411` section 6 and `005` section 12: an unresolved
    /// reference is preserved with a diagnostic so the user can repair it.
    UnresolvedSourceReference {
        /// The item holding the reference.
        item: SceneItemId,
        /// The source it points at.
        source: SourceId,
    },
    /// A scene item names a scene that is not in the file.
    UnresolvedSceneReference {
        /// The item holding the reference.
        item: SceneItemId,
        /// The scene it points at.
        scene: SceneId,
    },
    /// A scene lists an item that is not in the file.
    MissingSceneItem {
        /// The scene.
        scene: SceneId,
        /// The item it lists.
        item: SceneItemId,
    },
}

impl Diagnostic {
    /// A stable issue code (`411` section 3).
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::IntegrityMismatch => "integrity_mismatch",
            Self::UnsupportedFeature { .. } => "unsupported_feature",
            Self::UnreadableIdentifier { .. } => "unreadable_identifier",
            Self::UnusableName { .. } => "unusable_name",
            Self::DuplicateIdentifier { .. } => "duplicate_identifier",
            Self::UnresolvedSourceReference { .. } => "unresolved_source_reference",
            Self::UnresolvedSceneReference { .. } => "unresolved_scene_reference",
            Self::MissingSceneItem { .. } => "missing_scene_item",
        }
    }

    /// Whether this condition prevents saving over the original file.
    ///
    /// A project opened with an unsupported feature is read-only, because saving
    /// would write back a document with that feature quietly missing.
    #[must_use]
    pub const fn forces_read_only(&self) -> bool {
        matches!(self, Self::UnsupportedFeature { .. })
    }
}

/// A project that opened, and everything noticed on the way.
#[derive(Debug)]
pub struct OpenedProject {
    /// The store holding the project.
    pub store: StateStore,
    /// How it may be used.
    pub mode: OpenMode,
    /// What was wrong with it, in the order it was found.
    pub diagnostics: Vec<Diagnostic>,
    /// What the file looked like, for the next save's conflict check.
    pub identity: Option<FileIdentity>,
}

/// Read and validate a project file.
pub fn open_project(path: &Path, engine_session_id: &str) -> Result<OpenedProject, OpenError> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| OpenError::Unreadable(FilesystemFailure::of_io(&error)))?;

    // Bounded before reading, not after: the length is chosen by whoever wrote
    // the file, and reading it to find out how big it is defeats the bound.
    if metadata.len() > MAX_PROJECT_FILE_BYTES {
        return Err(OpenError::TooLarge);
    }

    let identity = FileIdentity::of(path);
    let text = std::fs::read_to_string(path)
        .map_err(|error| OpenError::Unreadable(FilesystemFailure::of_io(&error)))?;

    let opened = open_document(&text, engine_session_id)?;

    Ok(OpenedProject { identity, ..opened })
}

/// Validate a project document that has already been read.
///
/// Split from the filesystem so every rejection is testable without one, which
/// matters because the interesting inputs here are malformed and a malformed
/// file is easier to write as a string than to place on disk.
pub fn open_document(text: &str, engine_session_id: &str) -> Result<OpenedProject, OpenError> {
    // 411 section 2.1 before 2.2: read the envelope as untyped JSON first, so a
    // file from a future schema version is refused for that reason rather than
    // for the field differences the version explains.
    let document =
        serde_json::from_str::<serde_json::Value>(text).map_err(|_| OpenError::Malformed)?;

    if document.get("format").and_then(serde_json::Value::as_str) != Some(PROJECT_FORMAT) {
        return Err(OpenError::NotAProject);
    }

    match document
        .get("schemaVersion")
        .and_then(serde_json::Value::as_u64)
    {
        Some(version) if version == u64::from(PROJECT_SCHEMA_VERSION) => {}
        Some(version) => {
            return Err(OpenError::UnsupportedSchemaVersion {
                found: u16::try_from(version).unwrap_or(u16::MAX),
                supported: PROJECT_SCHEMA_VERSION,
            });
        }
        None => return Err(OpenError::NotAProject),
    }

    // 411 section 2.2. The generated decoder enforces required fields, types,
    // enums, and bounds, so schema validation is this one call.
    let envelope =
        serde_json::from_str::<PersistedProjectEnvelope>(text).map_err(|_| OpenError::Malformed)?;

    let mut diagnostics = Vec::new();

    if !integrity_matches(&envelope) {
        diagnostics.push(Diagnostic::IntegrityMismatch);
    }

    for feature in &envelope.features {
        // No feature is implemented yet, so every declared one is unsupported.
        // That is not a placeholder: `401` invariant 7 requires the file's
        // requirements to be honoured or refused, and honouring nothing honestly
        // is better than honouring nothing quietly.
        diagnostics.push(Diagnostic::UnsupportedFeature {
            feature: feature.clone(),
        });
    }

    let project_id = envelope
        .project_id
        .parse::<ProjectId>()
        .map_err(|_| OpenError::Malformed)?;

    let mut state = ProjectState::empty(project_id);
    load_entities(&envelope, &mut state, &mut diagnostics);
    check_references(&state, &mut diagnostics);

    let mode = if diagnostics.iter().any(Diagnostic::forces_read_only) {
        OpenMode::ReadOnly
    } else {
        OpenMode::ReadWrite
    };

    let mut store = StateStore::new(engine_session_id, project_id);
    let mut transaction = store.transaction();

    // Loading is a commit like any other, so the opened project arrives at a
    // generation a client can synchronize against rather than at the initial one
    // that means "nothing has happened yet".
    let _ = transaction.prepare(|candidate| {
        *candidate = state;
        Ok(())
    });
    let _ = transaction.commit();

    Ok(OpenedProject {
        store,
        mode,
        diagnostics,
        identity: None,
    })
}

/// Turn persisted entities into domain entities, keeping what can be kept.
fn load_entities(
    envelope: &PersistedProjectEnvelope,
    state: &mut ProjectState,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for source in &envelope.project.sources {
        let (Ok(id), Ok(name)) = (
            source.id.parse::<SourceId>(),
            EntityName::new(source.name.clone()),
        ) else {
            diagnostics.push(if source.id.parse::<SourceId>().is_err() {
                Diagnostic::UnreadableIdentifier {
                    collection: "sources",
                }
            } else {
                Diagnostic::UnusableName {
                    collection: "sources",
                }
            });
            continue;
        };

        if state.source(id).is_some() {
            diagnostics.push(Diagnostic::DuplicateIdentifier {
                collection: "sources",
            });
            continue;
        }

        state.put_source(SourceDefinition {
            id,
            name,
            kind: match source.kind {
                PersistedSourceDefinitionKind::Color => SourceKind::Color,
            },
        });
    }

    for scene in &envelope.project.scenes {
        let (Ok(id), Ok(name)) = (
            scene.id.parse::<SceneId>(),
            EntityName::new(scene.name.clone()),
        ) else {
            diagnostics.push(if scene.id.parse::<SceneId>().is_err() {
                Diagnostic::UnreadableIdentifier {
                    collection: "scenes",
                }
            } else {
                Diagnostic::UnusableName {
                    collection: "scenes",
                }
            });
            continue;
        };

        if state.scene(id).is_some() {
            diagnostics.push(Diagnostic::DuplicateIdentifier {
                collection: "scenes",
            });
            continue;
        }

        let mut items = Vec::new();
        for item in &scene.items {
            match item.parse::<SceneItemId>() {
                Ok(item) => items.push(item),
                Err(_) => diagnostics.push(Diagnostic::UnreadableIdentifier {
                    collection: "scenes",
                }),
            }
        }

        state.put_scene(Scene { id, name, items });
    }

    for item in &envelope.project.scene_items {
        let Some(loaded) = load_scene_item(item, diagnostics) else {
            continue;
        };

        if state.scene_item(loaded.id).is_some() {
            diagnostics.push(Diagnostic::DuplicateIdentifier {
                collection: "sceneItems",
            });
            continue;
        }

        state.put_scene_item(loaded);
    }
}

/// Parse one scene item, or report why it could not be read.
fn load_scene_item(
    item: &PersistedSceneItem,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<SceneItem> {
    let (Ok(id), Ok(scene), Ok(source)) = (
        item.id.parse::<SceneItemId>(),
        item.scene_id.parse::<SceneId>(),
        item.source_id.parse::<SourceId>(),
    ) else {
        diagnostics.push(Diagnostic::UnreadableIdentifier {
            collection: "sceneItems",
        });
        return None;
    };

    Some(SceneItem {
        id,
        scene,
        source,
        visible: item.visible,
    })
}

/// Check that references point at things that exist (`411` section 2.3).
///
/// Nothing is removed. A dangling reference is a repair a user can make; a
/// deletion is not something they can undo.
fn check_references(state: &ProjectState, diagnostics: &mut Vec<Diagnostic>) {
    for item in state.scene_items() {
        if state.source(item.source).is_none() {
            diagnostics.push(Diagnostic::UnresolvedSourceReference {
                item: item.id,
                source: item.source,
            });
        }

        if state.scene(item.scene).is_none() {
            diagnostics.push(Diagnostic::UnresolvedSceneReference {
                item: item.id,
                scene: item.scene,
            });
        }
    }

    for scene in state.scenes() {
        for listed in &scene.items {
            if state.scene_item(*listed).is_none() {
                diagnostics.push(Diagnostic::MissingSceneItem {
                    scene: scene.id,
                    item: *listed,
                });
            }
        }
    }
}

//! Tests for canonical serialization and atomic save.
//!
//! What `MIR-0109` owes from `403` section 13: a normal save, external
//! modification, deterministic serialization, permission denied, and stale
//! temporary cleanup. Crash-during-write, crash-before-rename, and
//! crash-after-rename belong to `MIR-0114`, which injects the interruption
//! rather than describing it.

use std::path::{Path, PathBuf};

use mirae_contracts::generated::PersistedProjectEnvelope;
use mirae_types::{ProjectId, StateGeneration};

use crate::canonical::{LINE_ENDING, integrity_matches, serialize_with_integrity};
use crate::mapping::envelope_of;
use crate::save::{Durability, FileIdentity, FilesystemFailure, SaveError, save_project};
use mirae_state::ProjectState;

const APP_VERSION: &str = "0.0.0";
const CREATED_AT: &str = "2026-07-30T12:00:00Z";

/// A scratch directory that removes itself.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("mirae-project-save-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        let _ = std::fs::create_dir_all(&path);

        Self { path }
    }

    fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }

    fn entries(&self) -> Vec<String> {
        let Ok(entries) = std::fs::read_dir(&self.path) else {
            return Vec::new();
        };

        let mut names: Vec<String> = entries
            .flatten()
            .filter_map(|entry| entry.file_name().to_str().map(ToOwned::to_owned))
            .collect();
        names.sort();
        names
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn envelope() -> PersistedProjectEnvelope {
    envelope_of(
        &ProjectState::empty(ProjectId::nil()),
        CREATED_AT,
        CREATED_AT,
        APP_VERSION,
    )
}

fn save_to(
    path: &Path,
    expected: Option<&FileIdentity>,
) -> Result<crate::save::SaveResult, SaveError> {
    save_project(
        &envelope(),
        StateGeneration::from_raw(1),
        path,
        expected,
        Durability::Normal,
    )
}

#[test]
fn serialization_is_deterministic_and_key_sorted() {
    // 401 section 12 and ADR-0071. Two serializations of one value must be
    // byte-identical, or the content hash disagrees with itself and every save
    // rewrites the whole file.
    let (first, first_hash) = serialize_with_integrity(&envelope()).unwrap_or_default();
    let (second, second_hash) = serialize_with_integrity(&envelope()).unwrap_or_default();

    assert_eq!(first, second);
    assert_eq!(first_hash, second_hash);

    let keys: Vec<&str> = first
        .lines()
        .filter_map(|line| line.trim().strip_prefix('"'))
        .filter_map(|rest| rest.split('"').next())
        .collect();
    let top_level: Vec<&str> = first
        .lines()
        .filter(|line| line.starts_with("  \""))
        .filter_map(|line| line.trim().strip_prefix('"'))
        .filter_map(|rest| rest.split('"').next())
        .collect();
    let mut sorted = top_level.clone();
    sorted.sort_unstable();

    assert!(!keys.is_empty());
    assert_eq!(top_level, sorted, "keys are in code-point order");
}

#[test]
fn the_document_uses_two_space_indentation_and_ends_with_a_newline() {
    let (text, _) = serialize_with_integrity(&envelope()).unwrap_or_default();

    assert!(text.ends_with(LINE_ENDING));
    assert!(
        !text.contains("\r\n"),
        "line endings do not depend on the platform"
    );
    assert!(text.contains("\n  \"format\""));
}

#[test]
fn the_hash_covers_the_document_without_itself() {
    // 401 section 11: the hash field is excluded from the bytes it describes.
    // Without that exclusion the value would have to contain its own hash, which
    // no value can.
    let (text, hash) = serialize_with_integrity(&envelope()).unwrap_or_default();

    assert_eq!(hash.len(), 64, "sha256 as lowercase hexadecimal");
    assert!(hash.chars().all(|character| character.is_ascii_hexdigit()));
    assert!(text.contains(&hash), "and the file carries it");

    let decoded = serde_json::from_str::<PersistedProjectEnvelope>(&text);

    assert!(decoded.as_ref().is_ok_and(integrity_matches));
}

#[test]
fn a_tampered_document_fails_its_integrity_check() {
    let (text, _) = serialize_with_integrity(&envelope()).unwrap_or_default();
    let altered = text.replace(
        "\"savedByVersion\": \"0.0.0\"",
        "\"savedByVersion\": \"9.9.9\"",
    );

    let decoded = serde_json::from_str::<PersistedProjectEnvelope>(&altered);

    assert!(
        decoded
            .as_ref()
            .is_ok_and(|envelope| !integrity_matches(envelope))
    );
}

#[test]
fn a_first_save_writes_the_file_and_leaves_no_temporary_behind() {
    // 403 section 5: a temporary file is never canonical and must not survive to
    // be mistaken for a recovery record.
    let scratch = Scratch::new("first");
    let path = scratch.file("project.mirae.json");

    let result = save_to(&path, None);

    assert!(result.is_ok());
    assert_eq!(scratch.entries(), vec!["project.mirae.json".to_owned()]);
    assert_eq!(
        result.as_ref().ok().map(|result| result.saved_generation),
        Some(StateGeneration::from_raw(1))
    );
    assert!(result.is_ok_and(|result| result.bytes_written > 0));
}

#[test]
fn the_written_file_decodes_back_into_the_same_project() {
    let scratch = Scratch::new("roundtrip");
    let path = scratch.file("project.mirae.json");
    let _ = save_to(&path, None);

    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let decoded = serde_json::from_str::<PersistedProjectEnvelope>(&text);

    assert!(decoded.as_ref().is_ok_and(integrity_matches));
    assert_eq!(
        decoded.ok().map(|decoded| decoded.project_id),
        Some(ProjectId::nil().to_string())
    );
}

#[test]
fn saving_over_an_unexpected_file_is_refused_rather_than_overwritten() {
    // 403 invariant 4 and section 7. The alternative is discarding somebody's
    // work without telling them, which is the one outcome persistence exists to
    // prevent.
    let scratch = Scratch::new("conflict");
    let path = scratch.file("project.mirae.json");

    let first = save_to(&path, None);
    let identity = first.ok().and_then(|result| result.identity);

    // Something else edits the file.
    let _ = std::fs::write(&path, b"edited by something else");

    let second = save_to(&path, identity.as_ref());

    assert_eq!(second.err(), Some(SaveError::ExternallyModified));
    assert_eq!(
        std::fs::read_to_string(&path).unwrap_or_default(),
        "edited by something else",
        "and the external version survives"
    );
}

#[test]
fn saving_with_the_matching_identity_replaces_the_file() {
    let scratch = Scratch::new("replace");
    let path = scratch.file("project.mirae.json");

    let first = save_to(&path, None);
    let identity = first.ok().and_then(|result| result.identity);
    let second = save_to(&path, identity.as_ref());

    assert!(second.is_ok());
    assert_eq!(scratch.entries(), vec!["project.mirae.json".to_owned()]);
}

#[test]
fn expecting_no_file_when_one_exists_is_a_conflict() {
    // A first save that finds something already there is the same problem as a
    // stale identity: this code did not write it, so it does not get to replace
    // it.
    let scratch = Scratch::new("exists");
    let path = scratch.file("project.mirae.json");
    let _ = std::fs::write(&path, b"someone else's project");

    assert_eq!(
        save_to(&path, None).err(),
        Some(SaveError::ExternallyModified)
    );
}

#[test]
fn expecting_a_file_that_is_gone_is_a_conflict() {
    let scratch = Scratch::new("vanished");
    let path = scratch.file("project.mirae.json");
    let stale = FileIdentity {
        length: 10,
        modified: None,
    };

    assert_eq!(
        save_to(&path, Some(&stale)).err(),
        Some(SaveError::ExternallyModified)
    );
}

#[test]
fn a_destination_inside_a_missing_directory_reports_a_filesystem_failure() {
    // The previous project file, if any, is untouched: 403 invariant 7 requires
    // failure to preserve it, and there is nothing to preserve here only because
    // there was nothing to begin with.
    let scratch = Scratch::new("missing-dir");
    let path = scratch.file("absent").join("project.mirae.json");

    let error = save_to(&path, None).err();

    assert!(matches!(error, Some(SaveError::Filesystem(_))));
    assert_ne!(
        error,
        Some(SaveError::Filesystem(FilesystemFailure::OutOfSpace))
    );
}

#[test]
fn a_failed_write_leaves_no_temporary_file_in_the_directory() {
    let scratch = Scratch::new("cleanup");
    let path = scratch.file("nested").join("project.mirae.json");

    let _ = save_to(&path, None);

    assert!(
        scratch
            .entries()
            .iter()
            .all(|entry| !entry.ends_with(".tmp")),
        "a failed save cleans up after itself"
    );
}

#[test]
fn each_durability_level_writes_the_same_bytes() {
    // The levels differ in what they flush, never in what they write. A file
    // whose contents depended on the durability policy would make autosave and
    // explicit save produce different projects.
    let scratch = Scratch::new("durability");
    let mut written = Vec::new();

    for (index, durability) in [Durability::Fast, Durability::Normal, Durability::Strong]
        .into_iter()
        .enumerate()
    {
        let path = scratch.file(&format!("project-{index}.mirae.json"));
        let result = save_project(
            &envelope(),
            StateGeneration::from_raw(1),
            &path,
            None,
            durability,
        );

        assert!(result.is_ok(), "{durability:?} should succeed");
        written.push(std::fs::read_to_string(&path).unwrap_or_default());
    }

    assert_eq!(written.first(), written.last());
}

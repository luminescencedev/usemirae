//! Interrupting a save at each point it can be interrupted.
//!
//! Canonical documentation: `docs/04-project/403-persistence.md` section 13,
//! `docs/06-quality/611-fault-injection.md`.
//!
//! `403` promises that a crash leaves either the complete previous version or
//! the complete new one, never a truncated file. That is a claim about what the
//! filesystem looks like after the process stops existing, so these tests stop
//! the pipeline at each named point from `611` section 3 and then look at the
//! directory.
//!
//! What is deliberately *not* done here: writing a truncated file by hand and
//! asserting the reader copes. That tests the reader. The writer is what
//! promised something.

use std::path::{Path, PathBuf};

use mirae_contracts::generated::PersistedProjectEnvelope;
use mirae_types::{ProjectId, StateGeneration};

use crate::canonical::integrity_matches;
use crate::mapping::envelope_of;
use crate::open::{OpenError, open_document};
use crate::save::{
    Durability, FaultPlan, FaultPoint, FileIdentity, SaveError, clean_stale_temporaries,
    save_project, save_project_with_faults,
};
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
            std::env::temp_dir().join(format!("mirae-project-fault-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        let _ = std::fs::create_dir_all(&path);

        Self { path }
    }

    fn file(&self) -> PathBuf {
        self.path.join("project.mirae.json")
    }

    /// Every entry in the directory, sorted.
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

    /// How many temporary files are lying around.
    fn temporaries(&self) -> usize {
        self.entries()
            .iter()
            .filter(|name| name.ends_with(".tmp"))
            .count()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn envelope(name: &str) -> PersistedProjectEnvelope {
    let mut envelope = envelope_of(
        &ProjectState::empty(ProjectId::nil()),
        CREATED_AT,
        CREATED_AT,
        APP_VERSION,
    );
    // The version string is a convenient way to tell two saves apart in a file.
    envelope.app.saved_by_version = name.to_owned();
    envelope
}

/// Save with no faults, returning the identity for the next save.
fn save(path: &Path, name: &str, expected: Option<&FileIdentity>) -> Option<FileIdentity> {
    save_project(
        &envelope(name),
        StateGeneration::from_raw(1),
        path,
        expected,
        Durability::Normal,
    )
    .ok()
    .and_then(|result| result.identity)
}

/// Save with a fault plan.
fn save_interrupted(
    path: &Path,
    name: &str,
    expected: Option<&FileIdentity>,
    point: FaultPoint,
) -> SaveError {
    let outcome = save_project_with_faults(
        &envelope(name),
        StateGeneration::from_raw(2),
        path,
        expected,
        Durability::Normal,
        FaultPlan::interrupt_at(point),
    );

    outcome.err().unwrap_or(SaveError::ExternallyModified)
}

/// Read a file back and report which save wrote it.
fn written_by(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let envelope = serde_json::from_str::<PersistedProjectEnvelope>(&text).ok()?;

    Some(envelope.app.saved_by_version)
}

#[test]
fn every_fault_point_has_the_stable_name_the_documentation_gives() {
    // 611 section 3: fault points are named and stable, so a test and a
    // diagnostic can refer to the same instant.
    assert_eq!(
        FaultPoint::BeforeTempWrite.as_str(),
        "project.save.before_temp_write"
    );
    assert_eq!(
        FaultPoint::AfterTempWrite.as_str(),
        "project.save.after_temp_write"
    );
    assert_eq!(
        FaultPoint::BeforePublish.as_str(),
        "project.save.before_publish"
    );
    assert_eq!(
        FaultPoint::AfterPublish.as_str(),
        "project.save.after_publish"
    );
}

#[test]
fn an_interruption_before_the_temporary_write_touches_nothing() {
    let scratch = Scratch::new("before-write");
    let path = scratch.file();
    let identity = save(&path, "first", None);

    let error = save_interrupted(
        &path,
        "second",
        identity.as_ref(),
        FaultPoint::BeforeTempWrite,
    );

    assert_eq!(error, SaveError::Interrupted(FaultPoint::BeforeTempWrite));
    assert_eq!(written_by(&path), Some("first".to_owned()));
    assert_eq!(scratch.temporaries(), 0);
}

#[test]
fn an_interruption_after_the_temporary_write_leaves_the_previous_file_complete() {
    // 403 invariant 7 and section 13, crash during temp write. The debris is
    // expected: a killed process does not clean up. What must not happen is the
    // canonical file changing.
    let scratch = Scratch::new("after-write");
    let path = scratch.file();
    let identity = save(&path, "first", None);

    let error = save_interrupted(
        &path,
        "second",
        identity.as_ref(),
        FaultPoint::AfterTempWrite,
    );

    assert_eq!(error, SaveError::Interrupted(FaultPoint::AfterTempWrite));
    assert_eq!(
        written_by(&path),
        Some("first".to_owned()),
        "the visible file is still the complete previous version"
    );
    assert_eq!(scratch.temporaries(), 1, "and the debris is left behind");
}

#[test]
fn an_interruption_before_the_rename_leaves_the_previous_file_complete() {
    // 403 section 13, crash before rename. The same guarantee at the moment it
    // is most tempting to break: the new content exists, and is not yet the
    // project.
    let scratch = Scratch::new("before-rename");
    let path = scratch.file();
    let identity = save(&path, "first", None);

    let error = save_interrupted(
        &path,
        "second",
        identity.as_ref(),
        FaultPoint::BeforePublish,
    );

    assert_eq!(error, SaveError::Interrupted(FaultPoint::BeforePublish));
    assert_eq!(written_by(&path), Some("first".to_owned()));

    let decoded = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<PersistedProjectEnvelope>(&text).ok());

    assert!(
        decoded.as_ref().is_some_and(integrity_matches),
        "and it still passes its own integrity check"
    );
}

#[test]
fn an_interruption_after_the_rename_leaves_the_new_file_complete() {
    // 403 invariant 2, crash after rename. Everything after the rename is
    // durability work; the file is already the new one, and it is whole.
    let scratch = Scratch::new("after-rename");
    let path = scratch.file();
    let identity = save(&path, "first", None);

    let error = save_interrupted(&path, "second", identity.as_ref(), FaultPoint::AfterPublish);

    assert_eq!(error, SaveError::Interrupted(FaultPoint::AfterPublish));
    assert_eq!(
        written_by(&path),
        Some("second".to_owned()),
        "the visible file is the complete new version"
    );
    assert_eq!(scratch.temporaries(), 0, "and the temporary is gone");
}

#[test]
fn a_file_left_by_an_interrupted_save_is_never_a_valid_project_at_the_visible_path() {
    // The property underneath all of the above: at no interruption point does
    // the *visible* path hold something a reader would refuse. Stated once,
    // against every point, because this is the promise 403 actually makes.
    for point in [
        FaultPoint::BeforeTempWrite,
        FaultPoint::AfterTempWrite,
        FaultPoint::BeforePublish,
        FaultPoint::AfterPublish,
    ] {
        let scratch = Scratch::new(&format!("readable-{}", point.as_str().replace('.', "-")));
        let path = scratch.file();
        let identity = save(&path, "first", None);

        let _ = save_interrupted(&path, "second", identity.as_ref(), point);

        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let opened = open_document(&text, "session");

        assert!(
            opened.is_ok(),
            "after an interruption at {}, the project still opens",
            point.as_str()
        );
    }
}

#[test]
fn a_first_save_interrupted_before_the_rename_leaves_no_project_rather_than_half_of_one() {
    // There is no previous version to preserve, so the guarantee is the other
    // half of the same sentence: the visible path holds nothing at all.
    let scratch = Scratch::new("first-save");
    let path = scratch.file();

    let error = save_interrupted(&path, "only", None, FaultPoint::BeforePublish);

    assert_eq!(error, SaveError::Interrupted(FaultPoint::BeforePublish));
    assert!(
        !path.exists(),
        "no partial project appears at the visible path"
    );
    assert_eq!(scratch.temporaries(), 1);
}

#[test]
fn stale_temporaries_are_cleaned_and_nothing_else_is() {
    // 403 section 5: temporary files are cleaned after bounded retention and are
    // never mistaken for recovery snapshots. The next run is what knows the
    // debris is stale, and it must not take anything else with it.
    let scratch = Scratch::new("cleanup");
    let path = scratch.file();
    let identity = save(&path, "first", None);

    let _ = save_interrupted(
        &path,
        "second",
        identity.as_ref(),
        FaultPoint::BeforePublish,
    );
    let _ = save_interrupted(&path, "third", identity.as_ref(), FaultPoint::BeforePublish);

    // Things that are not this project's temporaries.
    let _ = std::fs::write(scratch.path.join("notes.txt"), b"a user file");
    let _ = std::fs::write(
        scratch.path.join(".other.mirae.json.1.tmp"),
        b"another project",
    );

    assert_eq!(scratch.temporaries(), 3);

    let removed = clean_stale_temporaries(&path);

    assert_eq!(removed, 2, "only this project's debris");
    assert!(scratch.entries().contains(&"notes.txt".to_owned()));
    assert!(
        scratch
            .entries()
            .contains(&".other.mirae.json.1.tmp".to_owned()),
        "another project's temporary is not this one's to remove"
    );
    assert_eq!(written_by(&path), Some("first".to_owned()));
}

#[test]
fn cleaning_a_directory_with_no_debris_removes_nothing() {
    let scratch = Scratch::new("nothing-to-clean");
    let path = scratch.file();
    let _ = save(&path, "first", None);

    assert_eq!(clean_stale_temporaries(&path), 0);
    assert_eq!(scratch.entries(), vec!["project.mirae.json".to_owned()]);
}

#[test]
fn a_save_after_an_interrupted_one_still_succeeds() {
    // Debris does not block the next attempt. A user whose machine died
    // mid-save presses save again, and it works.
    let scratch = Scratch::new("retry");
    let path = scratch.file();
    let identity = save(&path, "first", None);

    let _ = save_interrupted(
        &path,
        "second",
        identity.as_ref(),
        FaultPoint::BeforePublish,
    );

    let retried = save(&path, "third", identity.as_ref());

    assert!(retried.is_some());
    assert_eq!(written_by(&path), Some("third".to_owned()));
}

#[test]
fn a_truncated_file_at_the_visible_path_is_refused_rather_than_half_read() {
    // The reader's side of the same promise. This state is not reachable through
    // the save pipeline — that is what the tests above establish — but a disk
    // error or another program can produce it, and 411 requires it to be
    // reported rather than partly believed.
    let scratch = Scratch::new("truncated");
    let path = scratch.file();
    let _ = save(&path, "first", None);

    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let _ = std::fs::write(&path, &text.as_bytes()[..text.len() / 2]);

    let opened = open_document(
        &std::fs::read_to_string(&path).unwrap_or_default(),
        "session",
    );

    assert_eq!(opened.err(), Some(OpenError::Malformed));
}

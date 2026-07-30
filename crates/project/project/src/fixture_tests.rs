//! The compatibility corpus, checked against the code that reads and writes it.
//!
//! Canonical documentation: `docs/06-quality/615-compatibility-policy.md`,
//! `docs/04-project/408-schema-versioning-and-migrations.md` sections 11 and 13,
//! `docs/04-project/401-project-format.md` section 15.
//!
//! What `MIR-0115` owes: a committed fixture opens, saves, and compares
//! byte-identically; an older fixture opens through migration; an unknown
//! optional field survives a round trip or produces a diagnostic; fixtures are
//! generated rather than hand-edited.
//!
//! One of those cannot be satisfied honestly yet, and is recorded rather than
//! faked — see `an_older_schema_version_has_no_fixture_yet_and_says_so`.

use std::path::PathBuf;

use mirae_contracts::generated::PersistedProjectEnvelope;

use crate::canonical::{integrity_matches, serialize_with_integrity};
use crate::fixtures::{FIXTURE_DIRECTORY, UPDATE_VARIABLE, corpus};
use crate::mapping::PROJECT_SCHEMA_VERSION;
use crate::open::{OpenError, open_document};

/// The repository root, found from this crate's manifest.
fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

/// The directory holding the corpus.
fn fixture_directory() -> PathBuf {
    repository_root().join(FIXTURE_DIRECTORY)
}

/// Whether this run regenerates rather than compares.
fn updating() -> bool {
    std::env::var(UPDATE_VARIABLE).is_ok_and(|value| !value.is_empty())
}

#[test]
fn every_fixture_matches_what_the_current_code_would_write() {
    // The corpus is generated, so this is both the check and the regeneration.
    // A diff here is a compatibility change, stated plainly, which is the whole
    // reason the files are committed.
    let directory = fixture_directory();
    let update = updating();

    if update {
        let _ = std::fs::create_dir_all(&directory);
    }

    for fixture in corpus() {
        let Ok((expected, _)) = serialize_with_integrity(&fixture.envelope) else {
            unreachable!("a fixture must be serializable");
        };
        let path = directory.join(fixture.name);

        if update {
            let _ = std::fs::write(&path, expected.as_bytes());
            continue;
        }

        let committed = std::fs::read_to_string(&path).unwrap_or_default();

        assert_eq!(
            committed, expected,
            "{} differs from what this build writes. If that is intended, \
             regenerate with `{UPDATE_VARIABLE}=1 cargo test -p mirae-project` \
             and review the diff: it is the compatibility change. ({})",
            fixture.name, fixture.proves
        );
    }
}

#[test]
fn every_fixture_opens_and_saves_back_byte_identically() {
    // 401 section 15 and 615: the round trip is where a serialization change
    // shows itself. A build that reads a project and writes back different bytes
    // has changed the format, whether or not it meant to.
    for fixture in corpus() {
        let path = fixture_directory().join(fixture.name);
        let Ok(committed) = std::fs::read_to_string(&path) else {
            unreachable!("the fixture {} should be committed", fixture.name);
        };

        let opened = open_document(&committed, "session");

        assert!(
            opened.is_ok(),
            "{} should open: {}",
            fixture.name,
            fixture.proves
        );

        let Ok(opened) = opened else { continue };

        assert!(
            opened.diagnostics.is_empty(),
            "{} should open without diagnostics, found {:?}",
            fixture.name,
            opened.diagnostics
        );

        // Re-serialize from the state that was loaded, not from the fixture
        // value: this is the path a real save takes, so it is the path that has
        // to produce the same bytes.
        let snapshot = opened.store.snapshot();
        let rewritten = crate::mapping::envelope_of(
            snapshot.state(),
            &envelope_of_fixture(&committed).created_at,
            &envelope_of_fixture(&committed).last_saved_at,
            &envelope_of_fixture(&committed).app.saved_by_version,
        );

        let Ok((written, _)) = serialize_with_integrity(&rewritten) else {
            unreachable!("a loaded project must be serializable");
        };

        assert_eq!(
            written, committed,
            "{} did not survive a load and save unchanged",
            fixture.name
        );
    }
}

/// Decode a fixture, for the envelope fields the state does not carry.
fn envelope_of_fixture(text: &str) -> PersistedProjectEnvelope {
    serde_json::from_str(text).unwrap_or_else(|_| unreachable!("a fixture must decode"))
}

#[test]
fn every_fixture_passes_its_own_integrity_check() {
    for fixture in corpus() {
        let path = fixture_directory().join(fixture.name);
        let committed = std::fs::read_to_string(&path).unwrap_or_default();
        let decoded = serde_json::from_str::<PersistedProjectEnvelope>(&committed);

        assert!(
            decoded.as_ref().is_ok_and(integrity_matches),
            "{} should carry a hash of itself",
            fixture.name
        );
    }
}

#[test]
fn an_unknown_core_field_is_reported_rather_than_silently_dropped() {
    // 401 section 9: a core unknown field may be preserved only when the parser
    // and serializer can do so safely, and otherwise triggers a compatibility
    // diagnostic. This build cannot preserve one — the contract is closed — so
    // it must refuse rather than drop it, because dropping it means the next
    // save writes a file missing something the previous build put there.
    let path = fixture_directory().join("empty.mirae.json");
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    let smuggled = committed.replace(
        "\"format\": \"mirae-project\"",
        "\"format\": \"mirae-project\",\n  \"unknownFutureField\": 42",
    );

    assert_eq!(
        open_document(&smuggled, "session").err(),
        Some(OpenError::Malformed)
    );
}

#[test]
fn a_fixture_from_a_newer_schema_version_is_refused_by_version() {
    // 408 section 6: an unknown newer schema is refused as such. The fields will
    // also be wrong, and reporting *those* would send a user looking for a
    // corrupt file instead of a newer Mirae.
    let path = fixture_directory().join("empty.mirae.json");
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    let newer = committed.replace(
        "\"schemaVersion\": 1",
        &format!("\"schemaVersion\": {}", PROJECT_SCHEMA_VERSION + 1),
    );

    assert_eq!(
        open_document(&newer, "session").err(),
        Some(OpenError::UnsupportedSchemaVersion {
            found: PROJECT_SCHEMA_VERSION + 1,
            supported: PROJECT_SCHEMA_VERSION,
        })
    );
}

#[test]
fn an_older_schema_version_has_no_fixture_yet_and_says_so() {
    // MIR-0115 asks for an older fixture that opens through migration. There is
    // no older version: v1 is the first, and 408 section 3 migrates forward from
    // whatever existed. Writing a fake v0 fixture would test a migration nobody
    // wrote against a format that never shipped, and it would pass forever
    // without proving anything.
    //
    // What is asserted instead is the thing that will make the real fixture
    // possible: the corpus is per-version, so a v2 build adds
    // `fixtures/project/v1` files as its historical corpus without moving what
    // is here.
    assert_eq!(PROJECT_SCHEMA_VERSION, 1, "still the first schema version");
    assert!(
        FIXTURE_DIRECTORY.ends_with("/v1"),
        "the corpus is stored per schema version, so the next one has somewhere to go"
    );
}

#[test]
fn the_corpus_covers_what_the_test_matrix_asks_for() {
    // 408 section 11 lists what the corpus should hold. This asserts the ones
    // that exist today rather than the whole list, so the gap is visible in the
    // failure message rather than in nobody's memory.
    let names: Vec<&str> = corpus().iter().map(|fixture| fixture.name).collect();

    assert!(
        names.contains(&"empty.mirae.json"),
        "boundary: nothing at all"
    );
    assert!(
        names.contains(&"populated.mirae.json"),
        "one of every entity"
    );
    assert!(names.contains(&"boundaries.mirae.json"), "boundary values");

    // Every fixture records what it proves, as `fixtures/README.md` requires.
    for fixture in corpus() {
        assert!(
            !fixture.proves.is_empty(),
            "{} must record what it proves",
            fixture.name
        );
    }
}

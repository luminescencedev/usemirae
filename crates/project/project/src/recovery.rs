//! The recovery store: bounded autosave records, kept away from the project file.
//!
//! Canonical documentation: `docs/04-project/404-autosave-and-recovery.md`,
//! ADR-0072.
//!
//! `404` invariant 1 is the shape of this module: autosave never writes over the
//! canonical project. It writes here, into a directory it owns, and recovery
//! reads from here and offers the user a choice. Nothing in this file can reach
//! a project file, because nothing in it is given the path of one.
//!
//! The root directory is a parameter (ADR-0072). Resolving the platform's data
//! directory is `mirae-platform`'s job, so every test here points at a temporary
//! directory and none of them depends on the machine.
//!
//! Retention is bounded three ways — count, bytes, and age — because any single
//! bound fails on its own: a count bound lets a few enormous records fill a disk,
//! a byte bound lets a burst of tiny ones drown the useful history, and neither
//! notices a record from last year. `404` section 5 also requires that at least
//! one valid candidate survive an interrupted cleanup, so pruning removes the
//! oldest first and never the newest.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use mirae_contracts::generated::{PersistedProjectEnvelope, RecoveryRecord};
use mirae_types::{ProjectId, StateGeneration};

use crate::save::FilesystemFailure;

/// The extension every recovery record carries.
const RECORD_EXTENSION: &str = "recovery.json";

/// How much recovery history to keep (`404` section 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionPolicy {
    /// Most records to keep per project.
    pub max_records: usize,
    /// Most bytes to keep across all records.
    pub max_bytes: u64,
    /// Oldest record to keep.
    pub max_age: Duration,
}

impl RetentionPolicy {
    /// The default policy.
    ///
    /// Eight records, thirty-two megabytes, seven days. Generous enough that a
    /// user who crashes repeatedly still has yesterday's work, small enough that
    /// a forgotten install does not accumulate without limit.
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            max_records: 8,
            max_bytes: 32 * 1024 * 1024,
            max_age: Duration::from_secs(7 * 24 * 60 * 60),
        }
    }
}

/// Why a recovery operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryError {
    /// The store directory could not be created or written to.
    Filesystem(FilesystemFailure),
    /// The record could not be serialized.
    NotRepresentable,
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Filesystem(_) => "the recovery store could not be written to",
            Self::NotRepresentable => "the recovery record could not be written",
        })
    }
}

impl std::error::Error for RecoveryError {}

/// A recovery record found on disk, with what the caller needs to choose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryCandidate {
    /// Where the record is.
    pub path: PathBuf,
    /// The record itself.
    pub record: RecoveryRecord,
    /// Its size on disk.
    pub bytes: u64,
}

impl RecoveryCandidate {
    /// The generation this candidate holds.
    #[must_use]
    pub const fn generation(&self) -> StateGeneration {
        StateGeneration::from_raw(self.record.state_generation)
    }

    /// Whether this candidate was based on the explicit save identified by `hash`.
    ///
    /// `404` section 6 compares base identity before generations: a record built
    /// on a save the user has since replaced describes a different history, and
    /// offering it as "newer" would be misleading.
    #[must_use]
    pub fn builds_on(&self, hash: &str) -> bool {
        self.record.base_save_hash == hash
    }
}

/// Bounded storage for autosave records.
#[derive(Debug, Clone)]
pub struct RecoveryStore {
    root: PathBuf,
    retention: RetentionPolicy,
}

impl RecoveryStore {
    /// Open a store rooted at `root`.
    ///
    /// The directory is created on first write rather than here, so constructing
    /// a store touches nothing and a caller can hold one without committing to
    /// autosave ever running.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, retention: RetentionPolicy) -> Self {
        Self {
            root: root.into(),
            retention,
        }
    }

    /// Where this store keeps its records.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Write a record for a committed generation.
    ///
    /// Takes the generation explicitly rather than reading one, because `404`
    /// invariant 2 records committed generations only and the caller is the one
    /// that knows a commit happened.
    pub fn write(
        &self,
        project: &PersistedProjectEnvelope,
        project_id: ProjectId,
        generation: StateGeneration,
        recorded_at: &str,
        base_save_hash: &str,
        app_version: &str,
    ) -> Result<RecoveryCandidate, RecoveryError> {
        std::fs::create_dir_all(&self.root)
            .map_err(|error| RecoveryError::Filesystem(FilesystemFailure::of_io(&error)))?;

        let recovery_id = mirae_types::EntityId::new();
        let record = RecoveryRecord {
            recovery_id: recovery_id.to_string(),
            project_id: project_id.to_string(),
            state_generation: generation.get(),
            recorded_at: recorded_at.to_owned(),
            base_save_hash: base_save_hash.to_owned(),
            app_version: app_version.to_owned(),
            project: project.clone(),
        };

        let text =
            serde_json::to_string_pretty(&record).map_err(|_| RecoveryError::NotRepresentable)?;

        // The name carries the project so candidates can be found without
        // reading every record, and the recovery id so two writes cannot collide.
        let path = self
            .root
            .join(format!("{project_id}.{recovery_id}.{RECORD_EXTENSION}"));

        std::fs::write(&path, text.as_bytes())
            .map_err(|error| RecoveryError::Filesystem(FilesystemFailure::of_io(&error)))?;

        let bytes = std::fs::metadata(&path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);

        Ok(RecoveryCandidate {
            path,
            record,
            bytes,
        })
    }

    /// Every readable record for a project, newest generation first.
    ///
    /// A record that will not parse is skipped rather than reported as an error.
    /// `404` invariant 10: an invalid record must not block opening the
    /// canonical project, and it must not hide the valid records beside it
    /// either.
    #[must_use]
    pub fn candidates(&self, project_id: ProjectId) -> Vec<RecoveryCandidate> {
        let prefix = format!("{project_id}.");
        let mut found = self.all_candidates();

        found.retain(|candidate| {
            candidate
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&prefix))
        });
        found.sort_by(|left, right| {
            right
                .record
                .state_generation
                .cmp(&left.record.state_generation)
        });

        found
    }

    /// Every readable record in the store, in filesystem order.
    fn all_candidates(&self) -> Vec<RecoveryCandidate> {
        let Ok(entries) = std::fs::read_dir(&self.root) else {
            return Vec::new();
        };

        let mut candidates = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();

            if !path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(RECORD_EXTENSION))
            {
                continue;
            }

            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(record) = serde_json::from_str::<RecoveryRecord>(&text) else {
                continue;
            };

            candidates.push(RecoveryCandidate {
                bytes: entry.metadata().map(|metadata| metadata.len()).unwrap_or(0),
                path,
                record,
            });
        }

        candidates
    }

    /// Apply the retention policy, returning how many records were removed.
    ///
    /// Oldest first, and never the newest for a project. `404` section 5 asks
    /// that a recent valid candidate survive an interrupted cleanup, which means
    /// pruning has to be ordered so that stopping halfway still leaves the useful
    /// record behind.
    pub fn prune(&self, now: SystemTime) -> usize {
        let mut candidates = self.all_candidates();

        // Newest last, so draining from the front removes the least valuable.
        candidates.sort_by(|left, right| {
            left.record
                .state_generation
                .cmp(&right.record.state_generation)
        });

        let newest = candidates.pop();
        let mut removed = 0;
        let mut kept: Vec<&RecoveryCandidate> = Vec::new();
        let mut kept_bytes = newest
            .as_ref()
            .map(|candidate| candidate.bytes)
            .unwrap_or(0);

        // Walk newest to oldest over what remains, keeping while there is room.
        for candidate in candidates.iter().rev() {
            let too_many = kept.len() + 1 >= self.retention.max_records;
            let too_large = kept_bytes + candidate.bytes > self.retention.max_bytes;
            let too_old = self.is_expired(candidate, now);

            if too_many || too_large || too_old {
                if std::fs::remove_file(&candidate.path).is_ok() {
                    removed += 1;
                }
                continue;
            }

            kept_bytes += candidate.bytes;
            kept.push(candidate);
        }

        removed
    }

    /// Whether a record is older than the policy allows.
    ///
    /// Uses the file's modification time rather than the timestamp inside it: a
    /// record whose contents claim any date at all is still a file, and the
    /// filesystem is the harder of the two to get wrong.
    fn is_expired(&self, candidate: &RecoveryCandidate, now: SystemTime) -> bool {
        let Ok(metadata) = std::fs::metadata(&candidate.path) else {
            return false;
        };
        let Ok(modified) = metadata.modified() else {
            return false;
        };

        now.duration_since(modified)
            .is_ok_and(|age| age > self.retention.max_age)
    }

    /// Remove every record for a project after a clean close (`404` section 8).
    ///
    /// Returns how many were removed. Called after an explicit save has been
    /// confirmed durable, never before: the whole point of a recovery record is
    /// that it outlives the moment the user thought they were finished.
    pub fn discard(&self, project_id: ProjectId) -> usize {
        self.candidates(project_id)
            .iter()
            .filter(|candidate| std::fs::remove_file(&candidate.path).is_ok())
            .count()
    }
}

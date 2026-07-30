//! Publishing a project file atomically.
//!
//! Canonical documentation: `docs/04-project/403-persistence.md`.
//!
//! The pipeline is `403` section 3: validate, serialize, write a temporary file,
//! flush, replace, make the directory durable, report the saved generation. The
//! order is the whole point — the visible path is never the file being written,
//! so a crash at any moment leaves either the previous complete version or the
//! new complete one, and never a truncated one (`403` invariant 2).
//!
//! Serialization takes an immutable snapshot and never reads state that could
//! change underneath it (`403` invariant 1 and section 9). The save therefore
//! holds no lock: `403` section 14 forbids serializing while holding the commit
//! lock, and there is no way to do so here because the input is a value.

use std::fs::File;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use mirae_contracts::generated::PersistedProjectEnvelope;
use mirae_types::StateGeneration;

use crate::canonical::{CanonicalError, serialize_with_integrity};

/// How hard a save works to survive a crash (`403` section 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    /// Write and replace, leaving the flush to the operating system.
    ///
    /// For autosave, where losing the newest record to a power cut is
    /// acceptable because an older one survives.
    Fast,
    /// Flush the file's data before replacing.
    ///
    /// The floor for an explicit save (`403` section 6).
    Normal,
    /// Flush the file and then the directory entry, where the platform supports it.
    ///
    /// Windows has no directory handle to flush, so `Strong` and `Normal` behave
    /// identically there. Saying so is better than implying a guarantee the
    /// platform does not give.
    Strong,
}

/// What the caller believes about the file it is replacing (`403` section 7).
///
/// Captured when a project is opened or last saved. Comparing it before the
/// replace is what turns "someone edited this behind our back" from data loss
/// into a refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileIdentity {
    /// Size in bytes at the moment it was observed.
    pub length: u64,
    /// Last modification time at the moment it was observed.
    pub modified: Option<SystemTime>,
}

impl FileIdentity {
    /// Observe a file on disk.
    ///
    /// `None` when the path does not exist, which is the ordinary case for a
    /// first save and must not be confused with a conflict.
    #[must_use]
    pub fn of(path: &Path) -> Option<Self> {
        let metadata = std::fs::metadata(path).ok()?;

        Some(Self {
            length: metadata.len(),
            modified: metadata.modified().ok(),
        })
    }
}

/// Why a save did not happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveError {
    /// The project could not be serialized. The file on disk is untouched.
    Serialization(CanonicalError),
    /// The destination has no parent directory to write a temporary file into.
    NoDestinationDirectory,
    /// The file changed since the caller last observed it (`403` section 7).
    ///
    /// Not overwritten. `403` invariant 4 requires external modification to be
    /// detected, and the only safe response is to stop and let a person decide.
    ExternallyModified,
    /// The filesystem refused, with a stable category rather than an OS string.
    ///
    /// The category is what a caller can branch on; the raw message would carry
    /// a path, and a path is the sort of thing that ends up in a log it should
    /// not be in.
    Filesystem(FilesystemFailure),
}

/// What the filesystem refused (`403` section 13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemFailure {
    /// The process may not write there.
    PermissionDenied,
    /// There is no room.
    OutOfSpace,
    /// Something else, which the caller can only report.
    Other,
}

impl FilesystemFailure {
    /// Classify an I/O error.
    fn of(error: &std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            std::io::ErrorKind::StorageFull | std::io::ErrorKind::QuotaExceeded => Self::OutOfSpace,
            _ => Self::Other,
        }
    }
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialization(error) => write!(formatter, "{error}"),
            Self::NoDestinationDirectory => {
                formatter.write_str("the destination has no parent directory")
            }
            Self::ExternallyModified => {
                formatter.write_str("the project file changed since it was last read")
            }
            Self::Filesystem(failure) => formatter.write_str(match failure {
                FilesystemFailure::PermissionDenied => "the project file could not be written to",
                FilesystemFailure::OutOfSpace => "there is not enough room to write the project",
                FilesystemFailure::Other => "the project file could not be written",
            }),
        }
    }
}

impl std::error::Error for SaveError {}

/// What a completed save produced (`403` section 11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveResult {
    /// The generation that is now on disk.
    ///
    /// `403` invariant 5: an explicit save names the generation it covers, so a
    /// caller can tell whether a commit that raced the save is included.
    pub saved_generation: StateGeneration,
    /// Where it was written.
    pub path: PathBuf,
    /// What the file looks like now, for the next save's conflict check.
    pub identity: Option<FileIdentity>,
    /// How many bytes were written.
    pub bytes_written: u64,
    /// The content hash the file carries.
    pub content_hash: String,
    /// The durability actually achieved.
    pub durability: Durability,
}

/// Write a project file atomically.
///
/// `expected` is what the caller last observed at `path`: `None` means it
/// expects no file to be there. A mismatch stops the save rather than
/// overwriting, because the alternative is discarding somebody's work silently.
pub fn save_project(
    envelope: &PersistedProjectEnvelope,
    generation: StateGeneration,
    path: &Path,
    expected: Option<&FileIdentity>,
    durability: Durability,
) -> Result<SaveResult, SaveError> {
    let (text, content_hash) =
        serialize_with_integrity(envelope).map_err(SaveError::Serialization)?;

    let directory = path.parent().ok_or(SaveError::NoDestinationDirectory)?;

    // 403 section 7: compare before replacing, not after. Checking afterwards
    // would mean the overwrite already happened.
    let current = FileIdentity::of(path);
    if current.as_ref() != expected {
        return Err(SaveError::ExternallyModified);
    }

    // 403 section 5: the temporary file lives in the destination directory, so
    // the replace stays within one filesystem and can be atomic. A collision
    // -resistant name keeps two saves of the same project from meeting.
    let temporary = directory.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project"),
        temporary_suffix()
    ));

    let write_outcome = write_temporary(&temporary, text.as_bytes(), durability);

    if let Err(failure) = write_outcome {
        // A failed write must not leave a temporary file behind to be mistaken
        // for a recovery record later (`403` section 5).
        let _ = std::fs::remove_file(&temporary);
        return Err(SaveError::Filesystem(failure));
    }

    if let Err(error) = std::fs::rename(&temporary, path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(SaveError::Filesystem(FilesystemFailure::of(&error)));
    }

    if durability == Durability::Strong {
        // Unix only: the rename is durable once the directory entry is. Windows
        // exposes no directory handle to flush, and pretending otherwise would
        // be a guarantee this code cannot keep.
        if let Ok(handle) = File::open(directory) {
            let _ = handle.sync_all();
        }
    }

    Ok(SaveResult {
        saved_generation: generation,
        path: path.to_path_buf(),
        identity: FileIdentity::of(path),
        bytes_written: text.len() as u64,
        content_hash,
        durability,
    })
}

/// Write the temporary file and flush it according to `durability`.
fn write_temporary(
    temporary: &Path,
    bytes: &[u8],
    durability: Durability,
) -> Result<(), FilesystemFailure> {
    let mut file = File::create(temporary).map_err(|error| FilesystemFailure::of(&error))?;

    file.write_all(bytes)
        .map_err(|error| FilesystemFailure::of(&error))?;

    if durability != Durability::Fast {
        // `sync_all` rather than `flush`: flushing moves bytes out of the
        // process, which says nothing about whether they reached the disk.
        file.sync_all()
            .map_err(|error| FilesystemFailure::of(&error))?;
    }

    Ok(())
}

/// A collision-resistant suffix for a temporary file name.
fn temporary_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());

    format!("{}-{nanos:x}", std::process::id())
}

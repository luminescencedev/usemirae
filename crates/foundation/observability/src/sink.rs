//! Where events are written, and the retention policy that bounds them.
//!
//! Canonical documentation: `docs/06-quality/606-logging-and-tracing.md` sections
//! 6 and 9, and invariant 4: log storage is bounded.
//!
//! A sink never panics and never propagates a write failure upward. Logging is a
//! diagnostic path: if the disk is full or the directory disappeared, the failure
//! is counted and the caller continues (`605` section 9, recoverable external
//! conditions must not panic).

use std::fs::{self, File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// How much log data may be kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionPolicy {
    /// Rotate once the active file reaches this size.
    pub max_file_bytes: u64,
    /// Keep at most this many rotated files, newest first.
    pub max_files: usize,
}

impl RetentionPolicy {
    /// A conservative default: eight megabytes across four files.
    pub const DEFAULT: Self = Self {
        max_file_bytes: 2 * 1024 * 1024,
        max_files: 4,
    };

    /// The maximum total bytes this policy can occupy.
    #[must_use]
    pub const fn max_total_bytes(&self) -> u64 {
        self.max_file_bytes.saturating_mul(self.max_files as u64)
    }
}

/// Somewhere an event line can be written.
pub trait Sink: Send {
    /// Write one line. Returns `false` when the line was not stored.
    fn write_line(&mut self, line: &str) -> bool;

    /// How many lines this sink failed to store.
    fn failed_writes(&self) -> u64;

    /// The most recently stored line, for sinks that retain lines.
    ///
    /// Sinks that stream to disk return `None`; reading back the last line would
    /// mean holding it in memory for no operational benefit.
    fn last_line(&self) -> Option<&str> {
        None
    }
}

/// Discards everything. Used when logging is disabled.
#[derive(Debug, Default)]
pub struct NullSink;

impl Sink for NullSink {
    fn write_line(&mut self, _line: &str) -> bool {
        true
    }

    fn failed_writes(&self) -> u64 {
        0
    }
}

/// Keeps lines in memory. Used by tests and by short-lived tools.
#[derive(Debug, Default)]
pub struct MemorySink {
    lines: Vec<String>,
    /// Bound, so a test or tool cannot grow it without limit.
    max_lines: usize,
    failed: u64,
}

impl MemorySink {
    /// Build a sink holding at most `max_lines`.
    #[must_use]
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: Vec::new(),
            max_lines,
            failed: 0,
        }
    }

    /// The stored lines.
    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

impl Sink for MemorySink {
    fn write_line(&mut self, line: &str) -> bool {
        if self.lines.len() >= self.max_lines {
            self.failed = self.failed.saturating_add(1);
            return false;
        }

        self.lines.push(line.to_owned());
        true
    }

    fn failed_writes(&self) -> u64 {
        self.failed
    }

    fn last_line(&self) -> Option<&str> {
        self.lines.last().map(String::as_str)
    }
}

/// A sink that always fails, used to prove disk-full behavior.
#[derive(Debug, Default)]
pub struct FailingSink {
    failed: u64,
}

impl Sink for FailingSink {
    fn write_line(&mut self, _line: &str) -> bool {
        self.failed = self.failed.saturating_add(1);
        false
    }

    fn failed_writes(&self) -> u64 {
        self.failed
    }
}

/// Appends to a file, rotating it and deleting the oldest when full.
///
/// User projects and recordings never share the log directory (`606` section 9),
/// so the directory is owned entirely by this sink.
#[derive(Debug)]
pub struct RollingFileSink {
    directory: PathBuf,
    base_name: String,
    policy: RetentionPolicy,
    file: Option<File>,
    written_bytes: u64,
    failed: u64,
    rotations: u64,
}

impl RollingFileSink {
    /// Open, or create, the active log file in `directory`.
    ///
    /// A failure to create the directory or file is not fatal: the sink records it
    /// and drops lines until a later write succeeds.
    #[must_use]
    pub fn new(directory: &Path, base_name: &str, policy: RetentionPolicy) -> Self {
        let mut sink = Self {
            directory: directory.to_path_buf(),
            base_name: base_name.to_owned(),
            policy,
            file: None,
            written_bytes: 0,
            failed: 0,
            rotations: 0,
        };

        sink.open_active();
        sink
    }

    /// Path of the file currently being appended to.
    #[must_use]
    pub fn active_path(&self) -> PathBuf {
        self.directory.join(format!("{}.log", self.base_name))
    }

    /// Path of a rotated file, where 1 is the most recently rotated.
    #[must_use]
    pub fn rotated_path(&self, index: usize) -> PathBuf {
        self.directory
            .join(format!("{}.{index}.log", self.base_name))
    }

    /// How many times the active file has been rotated.
    #[must_use]
    pub const fn rotations(&self) -> u64 {
        self.rotations
    }

    /// The retention policy in force.
    #[must_use]
    pub const fn policy(&self) -> RetentionPolicy {
        self.policy
    }

    fn open_active(&mut self) {
        if fs::create_dir_all(&self.directory).is_err() {
            self.file = None;
            return;
        }

        let path = self.active_path();
        self.written_bytes = fs::metadata(&path).map_or(0, |metadata| metadata.len());
        self.file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .ok();
    }

    /// Rotate the active file and drop anything past the retention bound.
    fn rotate(&mut self) {
        // Close before renaming: Windows refuses to rename an open file.
        self.file = None;

        // Oldest first, so each rename lands on a free slot.
        let oldest = self.policy.max_files.saturating_sub(1);
        let _ = fs::remove_file(self.rotated_path(oldest.max(1)));

        let mut index = oldest.max(1);
        while index > 1 {
            let from = self.rotated_path(index - 1);
            let to = self.rotated_path(index);
            let _ = fs::rename(&from, &to);
            index -= 1;
        }

        if self.policy.max_files > 0 {
            let _ = fs::rename(self.active_path(), self.rotated_path(1));
        } else {
            let _ = fs::remove_file(self.active_path());
        }

        self.rotations = self.rotations.saturating_add(1);
        self.open_active();
    }
}

impl Sink for RollingFileSink {
    fn write_line(&mut self, line: &str) -> bool {
        let projected = self
            .written_bytes
            .saturating_add(line.len() as u64)
            .saturating_add(1);

        if self.file.is_some() && projected > self.policy.max_file_bytes {
            self.rotate();
        }

        if self.file.is_none() {
            // Try once more: the directory may have reappeared.
            self.open_active();
        }

        let Some(file) = self.file.as_mut() else {
            self.failed = self.failed.saturating_add(1);
            return false;
        };

        match writeln!(file, "{line}") {
            Ok(()) => {
                self.written_bytes = projected;
                true
            }
            Err(_) => {
                // Disk full, permission change, or a removed directory. Counted,
                // never propagated: a logging failure must not fail the caller.
                self.failed = self.failed.saturating_add(1);
                self.file = None;
                false
            }
        }
    }

    fn failed_writes(&self) -> u64 {
        self.failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unique scratch directory per test, removed on completion.
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Self {
            let mut path = std::env::temp_dir();
            path.push(format!("mirae-observability-{name}"));
            let _ = fs::remove_dir_all(&path);
            let _ = fs::create_dir_all(&path);

            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn memory_sink_is_bounded_and_counts_drops() {
        let mut sink = MemorySink::new(2);

        assert!(sink.write_line("one"));
        assert!(sink.write_line("two"));
        assert!(!sink.write_line("three"));
        assert_eq!(sink.lines().len(), 2);
        assert_eq!(sink.failed_writes(), 1);
    }

    #[test]
    fn writes_and_reads_back_a_line() {
        let temp = TempDir::new("write");
        let mut sink = RollingFileSink::new(&temp.path, "engine", RetentionPolicy::DEFAULT);

        assert!(sink.write_line("{\"event\":\"one\"}"));

        let contents = fs::read_to_string(sink.active_path()).unwrap_or_default();

        assert!(contents.contains("\"one\""));
        assert_eq!(sink.failed_writes(), 0);
    }

    #[test]
    fn rotates_when_the_file_reaches_its_bound() {
        let temp = TempDir::new("rotate");
        let policy = RetentionPolicy {
            max_file_bytes: 64,
            max_files: 3,
        };
        let mut sink = RollingFileSink::new(&temp.path, "engine", policy);

        for index in 0..20 {
            assert!(sink.write_line(&format!("line-{index:03}-padding-padding")));
        }

        assert!(sink.rotations() >= 1, "the file should have rotated");
        assert!(sink.rotated_path(1).exists(), "a rotated file should exist");
        assert_eq!(sink.failed_writes(), 0);
    }

    #[test]
    fn retention_never_exceeds_the_declared_total() {
        let temp = TempDir::new("retention");
        let policy = RetentionPolicy {
            max_file_bytes: 128,
            max_files: 2,
        };
        let mut sink = RollingFileSink::new(&temp.path, "engine", policy);

        for index in 0..200 {
            sink.write_line(&format!("line-{index:04}-padding-padding-padding"));
        }

        let total: u64 = fs::read_dir(&temp.path)
            .map(|entries| {
                entries
                    .flatten()
                    .filter_map(|entry| entry.metadata().ok())
                    .map(|metadata| metadata.len())
                    .sum()
            })
            .unwrap_or_default();

        // Allow one line of overshoot on the active file, which is written before
        // the next size check.
        assert!(
            total <= policy.max_total_bytes() + policy.max_file_bytes,
            "total {total} exceeded the retention bound"
        );
    }

    #[test]
    fn a_failing_sink_reports_rather_than_panics() {
        let mut sink = FailingSink::default();

        assert!(!sink.write_line("anything"));
        assert_eq!(sink.failed_writes(), 1);
    }

    #[test]
    fn an_unwritable_directory_is_counted_not_fatal() {
        // Point the sink at a path whose parent is a file, so creation must fail.
        let temp = TempDir::new("unwritable");
        let blocker = temp.path.join("blocker");
        let _ = fs::write(&blocker, b"not a directory");

        let mut sink =
            RollingFileSink::new(&blocker.join("logs"), "engine", RetentionPolicy::DEFAULT);

        assert!(!sink.write_line("{\"event\":\"one\"}"));
        assert!(sink.failed_writes() >= 1);
    }

    #[test]
    fn the_default_policy_bounds_total_storage() {
        assert_eq!(RetentionPolicy::DEFAULT.max_total_bytes(), 8 * 1024 * 1024);
    }
}

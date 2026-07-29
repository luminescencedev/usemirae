//! Process and file helpers.
//!
//! Every failure is returned, never panicked: a missing tool or unreadable file
//! is an operator problem to report, not a crash
//! (`docs/08-development/807-code-conventions.md`).

use std::fmt;
use std::path::Path;
use std::process::Command;

use crate::toolchain;

/// A command that did not succeed.
#[derive(Debug)]
pub(crate) struct StepError {
    /// The command line as an operator would retype it.
    pub(crate) command: String,
    pub(crate) reason: String,
}

impl fmt::Display for StepError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "`{}` {}", self.command, self.reason)
    }
}

/// Read a file, treating any failure as absence.
pub(crate) fn read_file(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// Split a command line into program and arguments.
///
/// Only whitespace splitting is needed: every caller passes a literal.
fn split_command(command_line: &str) -> Option<(&str, Vec<&str>)> {
    let mut parts = command_line.split_whitespace();
    let program = parts.next()?;

    Some((program, parts.collect()))
}

/// Build a command, going through the shell on Windows so `.cmd` shims resolve.
fn build(command_line: &str) -> Option<Command> {
    let (program, args) = split_command(command_line)?;

    // pnpm and other Node tools are `.cmd` shims on Windows, which
    // `CreateProcess` cannot execute directly.
    if cfg!(windows) {
        let mut command = Command::new("cmd");
        command.arg("/C").arg(program).args(&args);
        return Some(command);
    }

    let mut command = Command::new(program);
    command.args(&args);
    Some(command)
}

/// Probe a tool's version, or `None` when it cannot be run.
pub(crate) fn probe_version(command_line: &str) -> Option<String> {
    let output = build(command_line)?.output().ok()?;
    if !output.status.success() {
        return None;
    }

    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));

    toolchain::extract_version(&combined)
}

/// Run a step, streaming its output, and fail with the command that broke.
pub(crate) fn run(command_line: &str, working_directory: &Path) -> Result<(), StepError> {
    println!("+ {command_line}");

    let Some(mut command) = build(command_line) else {
        return Err(StepError {
            command: command_line.to_owned(),
            reason: "is not a runnable command line".to_owned(),
        });
    };

    let status = command.current_dir(working_directory).status();

    match status {
        Err(error) => Err(StepError {
            command: command_line.to_owned(),
            reason: format!("could not start: {error}"),
        }),
        Ok(status) if !status.success() => Err(StepError {
            command: command_line.to_owned(),
            reason: match status.code() {
                Some(code) => format!("failed with exit code {code}"),
                None => "was terminated by a signal".to_owned(),
            },
        }),
        Ok(_) => Ok(()),
    }
}

/// Run every step in order, stopping at the first failure.
pub(crate) fn run_all(steps: &[&str], working_directory: &Path) -> Result<(), StepError> {
    for step in steps {
        run(step, working_directory)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_a_command_line() {
        let split = split_command("cargo fmt --all -- --check");

        assert_eq!(
            split,
            Some(("cargo", vec!["fmt", "--all", "--", "--check"]))
        );
    }

    #[test]
    fn rejects_an_empty_command_line() {
        assert_eq!(split_command("   "), None);
    }

    #[test]
    fn reports_a_missing_program_instead_of_panicking() {
        let error = run(
            "mirae-command-that-does-not-exist --version",
            Path::new("."),
        );

        assert!(error.is_err());
    }

    #[test]
    fn treats_an_unreadable_file_as_absent() {
        assert_eq!(read_file(Path::new("does-not-exist.toml")), None);
    }

    #[test]
    fn formats_a_step_error_with_its_command() {
        let error = StepError {
            command: "cargo lint".to_owned(),
            reason: "failed with exit code 101".to_owned(),
        };

        assert_eq!(error.to_string(), "`cargo lint` failed with exit code 101");
    }
}

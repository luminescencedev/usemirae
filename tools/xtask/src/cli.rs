//! Command-line parsing.
//!
//! Kept free of I/O so every accepted and rejected invocation is unit tested.

use std::fmt;

/// A parsed `cargo xtask` invocation.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    /// Verify the toolchain and report anything an operator must install.
    Bootstrap,
    /// Run code generation, or verify that generated output is current.
    Generate { check: bool },
    /// Format, or verify formatting.
    Fmt { check: bool },
    /// Lint with warnings denied.
    Lint,
    /// Run tests.
    Test { scope: TestScope },
    /// Validate documentation structure.
    Docs { check: bool },
    /// Enforce repository policy: secrets, local paths, dependency direction, pins.
    Policy,
    /// Run the full baseline: generate --check, fmt --check, lint, test.
    Check,
    /// Print help, either general or for one command.
    Help { topic: Option<String> },
}

/// Which tests to run.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TestScope {
    /// Every test in both workspaces.
    All,
    /// The tests affected by the working tree, when detection exists.
    Affected,
    /// Cross-cutting tests under `tests/`.
    Integration,
}

/// Why an invocation could not be understood.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ParseError {
    /// No subcommand was given.
    MissingCommand,
    /// The subcommand is not known.
    UnknownCommand(String),
    /// The subcommand does not accept that flag.
    UnknownFlag { command: String, flag: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCommand => write!(formatter, "no command given"),
            Self::UnknownCommand(command) => write!(formatter, "unknown command `{command}`"),
            Self::UnknownFlag { command, flag } => {
                write!(formatter, "`{command}` does not accept `{flag}`")
            }
        }
    }
}

/// Accepted flags, so an unknown flag fails instead of being ignored.
fn only_check_flag(command: &str, args: &[String]) -> Result<bool, ParseError> {
    let mut check = false;

    for arg in args {
        if arg == "--check" {
            check = true;
        } else {
            return Err(ParseError::UnknownFlag {
                command: command.to_owned(),
                flag: arg.clone(),
            });
        }
    }

    Ok(check)
}

/// Reject every flag for commands that take none.
fn no_flags(command: &str, args: &[String]) -> Result<(), ParseError> {
    match args.first() {
        Some(flag) => Err(ParseError::UnknownFlag {
            command: command.to_owned(),
            flag: flag.clone(),
        }),
        None => Ok(()),
    }
}

/// Parse arguments that follow `cargo xtask`.
pub(crate) fn parse(args: &[String]) -> Result<Command, ParseError> {
    let (name, rest) = match args.split_first() {
        Some((name, rest)) => (name.as_str(), rest),
        None => return Err(ParseError::MissingCommand),
    };

    match name {
        "bootstrap" => {
            no_flags("bootstrap", rest)?;
            Ok(Command::Bootstrap)
        }
        "generate" => Ok(Command::Generate {
            check: only_check_flag("generate", rest)?,
        }),
        "fmt" => Ok(Command::Fmt {
            check: only_check_flag("fmt", rest)?,
        }),
        "lint" => {
            no_flags("lint", rest)?;
            Ok(Command::Lint)
        }
        "test" => {
            no_flags("test", rest)?;
            Ok(Command::Test {
                scope: TestScope::All,
            })
        }
        "test-affected" => {
            no_flags("test-affected", rest)?;
            Ok(Command::Test {
                scope: TestScope::Affected,
            })
        }
        "test-integration" => {
            no_flags("test-integration", rest)?;
            Ok(Command::Test {
                scope: TestScope::Integration,
            })
        }
        "docs" => Ok(Command::Docs {
            check: only_check_flag("docs", rest)?,
        }),
        "policy" => {
            no_flags("policy", rest)?;
            Ok(Command::Policy)
        }
        "check" => {
            no_flags("check", rest)?;
            Ok(Command::Check)
        }
        "help" | "--help" | "-h" => Ok(Command::Help {
            topic: rest.first().cloned(),
        }),
        other => Err(ParseError::UnknownCommand(other.to_owned())),
    }
}

/// General help text.
pub(crate) fn general_help() -> String {
    "\
cargo xtask — Mirae repository automation

Usage:
  cargo xtask <command> [flags]

Commands:
  bootstrap           Verify the pinned toolchain and report what to install
  generate [--check]  Run code generation, or fail if generated output is stale
  fmt [--check]       Format sources, or fail if formatting is needed
  lint                Lint with warnings denied
  test                Run every test in both workspaces
  test-affected       Run the tests affected by the working tree
  test-integration    Run the cross-cutting tests under tests/
  docs [--check]      Validate documentation structure
  policy              Enforce secrets, local paths, dependency direction, pins
  check               Run policy, generate --check, fmt --check, lint, and test
  help [command]      Show help for a command

Documentation:
  docs/08-development/806-build-system-and-toolchain.md
  docs/08-development/809-testing-and-validation-workflow.md
"
    .to_owned()
}

/// Help text for one command, or `None` when the command is unknown.
pub(crate) fn command_help(topic: &str) -> Option<String> {
    let text = match topic {
        "bootstrap" => {
            "\
cargo xtask bootstrap

Verifies that the installed Rust, Node, and pnpm versions match the pins in
.node-version, package.json, and rust-toolchain.toml, that those files agree with
each other, and that both lockfiles are committed. Also notes when no C compiler
is visible on PATH, which is advisory only: on Windows rustc links through MSVC
discovery, so a working machine often has none.

Reports the expected version, the found version, and the command that fixes each
problem. Makes no changes, so it is safe to run repeatedly.

Also runs as the root package.json preinstall hook, so a wrong version fails
before any dependency is fetched.
"
        }
        "generate" => {
            "\
cargo xtask generate [--check]

Runs every registered generator. With --check, fails if generated output differs
from what the current schemas produce, which is how CI detects drift.

No generators are registered yet; the canonical schema pipeline arrives with
MIR-0005 and MIR-0006.
"
        }
        "fmt" => {
            "\
cargo xtask fmt [--check]

Formats Rust sources with rustfmt. With --check, fails instead of writing.

TypeScript formatting is not wired up yet: prettier is pinned by
DEPENDENCY_VERSIONS.md but its configuration belongs to MIR-0004.
"
        }
        "lint" => {
            "\
cargo xtask lint

Runs clippy across the workspace with all targets and features, denying warnings.

TypeScript linting is not wired up yet: the ESLint flat configuration belongs to
MIR-0004.
"
        }
        "test" => {
            "\
cargo xtask test

Runs cargo test across the Rust workspace and pnpm -r test across the frontend
workspace.
"
        }
        "test-affected" => {
            "\
cargo xtask test-affected

Intended to run only the tests affected by the working tree. Change detection is
not implemented, so this currently runs the full suite and says so rather than
implying narrower coverage. No ticket owns it yet.
"
        }
        "test-integration" => {
            "\
cargo xtask test-integration

Runs the cross-cutting tests under tests/. The first harness arrives with
MIR-0015.
"
        }
        "docs" => {
            "\
cargo xtask docs [--check]

Validates that every link in docs/SUMMARY.md resolves and that every ADR file is
indexed exactly once.

Validation never writes, so --check is accepted for symmetry with the baseline in
809 section 2 and behaves identically.

Header, duplicate-document-id, and ADR reference validation belong to MIR-0014,
which extends this command.
"
        }
        "policy" => {
            "\
cargo xtask policy

Enforces the repository policies that can be checked mechanically:

  - committed secrets: private key blocks, provider token prefixes, and
    secret-named fields assigned a credential-shaped literal
  - machine-local absolute paths
  - committed environment files (.env, but not .env.example)
  - dependency direction from 804 section 3, over crate path groups
  - npm dependency pins: only an exact version, `catalog:`, or `workspace:`

Cycles are not checked here because cargo and pnpm already reject them. Import-
level dependency rules for TypeScript live in eslint.config.js.
"
        }
        "check" => {
            "\
cargo xtask check

Runs the baseline from docs/08-development/809 section 2 in order: policy,
generate --check, fmt --check, lint, test, then docs. Stops at the first failure.
"
        }
        _ => return None,
    };

    Some(text.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn parses_every_documented_command() {
        assert_eq!(parse(&args(&["bootstrap"])), Ok(Command::Bootstrap));
        assert_eq!(parse(&args(&["lint"])), Ok(Command::Lint));
        assert_eq!(parse(&args(&["check"])), Ok(Command::Check));
        assert_eq!(
            parse(&args(&["generate"])),
            Ok(Command::Generate { check: false })
        );
        assert_eq!(
            parse(&args(&["generate", "--check"])),
            Ok(Command::Generate { check: true })
        );
        assert_eq!(
            parse(&args(&["fmt", "--check"])),
            Ok(Command::Fmt { check: true })
        );
        assert_eq!(
            parse(&args(&["docs", "--check"])),
            Ok(Command::Docs { check: true })
        );
    }

    #[test]
    fn parses_each_test_scope() {
        assert_eq!(
            parse(&args(&["test"])),
            Ok(Command::Test {
                scope: TestScope::All
            })
        );
        assert_eq!(
            parse(&args(&["test-affected"])),
            Ok(Command::Test {
                scope: TestScope::Affected
            })
        );
        assert_eq!(
            parse(&args(&["test-integration"])),
            Ok(Command::Test {
                scope: TestScope::Integration
            })
        );
    }

    #[test]
    fn treats_help_flags_as_the_help_command() {
        assert_eq!(parse(&args(&["help"])), Ok(Command::Help { topic: None }));
        assert_eq!(parse(&args(&["--help"])), Ok(Command::Help { topic: None }));
        assert_eq!(parse(&args(&["-h"])), Ok(Command::Help { topic: None }));
        assert_eq!(
            parse(&args(&["help", "lint"])),
            Ok(Command::Help {
                topic: Some("lint".to_owned())
            })
        );
    }

    #[test]
    fn rejects_a_missing_command() {
        assert_eq!(parse(&[]), Err(ParseError::MissingCommand));
    }

    #[test]
    fn rejects_an_unknown_command() {
        assert_eq!(
            parse(&args(&["deploy"])),
            Err(ParseError::UnknownCommand("deploy".to_owned()))
        );
    }

    #[test]
    fn rejects_an_unknown_flag_instead_of_ignoring_it() {
        assert_eq!(
            parse(&args(&["fmt", "--fix"])),
            Err(ParseError::UnknownFlag {
                command: "fmt".to_owned(),
                flag: "--fix".to_owned(),
            })
        );
        assert_eq!(
            parse(&args(&["lint", "--check"])),
            Err(ParseError::UnknownFlag {
                command: "lint".to_owned(),
                flag: "--check".to_owned(),
            })
        );
    }

    #[test]
    fn every_command_has_help_text() {
        for command in [
            "bootstrap",
            "generate",
            "fmt",
            "lint",
            "test",
            "test-affected",
            "test-integration",
            "docs",
            "policy",
            "check",
        ] {
            let help = command_help(command);
            assert!(help.is_some(), "`{command}` has no help text");
            assert!(
                help.unwrap_or_default().contains(command),
                "`{command}` help does not name the command"
            );
        }
    }

    #[test]
    fn general_help_lists_every_command() {
        let help = general_help();
        for command in [
            "bootstrap",
            "generate",
            "fmt",
            "lint",
            "test-affected",
            "test-integration",
            "docs",
            "policy",
            "check",
        ] {
            assert!(help.contains(command), "general help omits `{command}`");
        }
    }

    #[test]
    fn unknown_help_topic_has_no_text() {
        assert_eq!(command_help("deploy"), None);
    }
}

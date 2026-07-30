//! `cargo xtask` — the single repository automation entry point.
//!
//! Canonical documentation:
//! - `docs/08-development/806-build-system-and-toolchain.md` (sections 3 and 4)
//! - `docs/08-development/809-testing-and-validation-workflow.md` (section 2)
//! - `docs/08-development/808-local-development-environment.md`
//!
//! Shell scripts and CI call these commands instead of duplicating their logic,
//! so local and CI validation cannot drift.

mod cli;
mod docs;
mod json;
mod naming;
mod policy;
mod runner;
mod schema;
mod toolchain;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cli::{Command, TestScope};
use runner::StepError;

/// Locate the repository root from this crate's compile-time location.
///
/// `CARGO_MANIFEST_DIR` is `tools/xtask`, so the root is two levels up. This
/// works regardless of the directory `cargo xtask` was invoked from.
fn repository_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    manifest_dir
        .ancestors()
        .nth(2)
        .unwrap_or(manifest_dir)
        .to_path_buf()
}

/// Run code generation, or verify that generated output is current.
///
/// The pipeline follows `805` section 4: discover and validate schemas, reject
/// duplicate ids, render deterministic outputs, then write them or compare them.
/// With `check`, any difference fails, which is how CI detects drift.
fn generate(check: bool, root: &Path) -> Result<(), StepError> {
    let (schemas, errors) = schema::discover(root);

    if !errors.is_empty() {
        eprintln!("{} schema problem(s):", errors.len());
        for error in &errors {
            eprintln!("  - {}: {}", error.path, error.detail);
        }

        return Err(StepError {
            command: "cargo xtask generate".to_owned(),
            reason: format!("found {} invalid schema(s)", errors.len()),
        });
    }

    println!(
        "Validated {} schema(s) across {} canonical domains.",
        schemas.len(),
        schema::DOMAINS.len()
    );

    let mut stale = Vec::new();
    let mut written = Vec::new();

    for artifact in schema::artifacts(&schemas) {
        let absolute = root.join(artifact.path);
        let current = runner::read_file(&absolute);

        if current.as_deref() == Some(artifact.contents.as_str()) {
            continue;
        }

        if check {
            stale.push(artifact.path.to_owned());
            continue;
        }

        if let Some(parent) = absolute.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            return Err(StepError {
                command: "cargo xtask generate".to_owned(),
                reason: format!("could not create {}: {error}", parent.display()),
            });
        }

        if let Err(error) = std::fs::write(&absolute, &artifact.contents) {
            return Err(StepError {
                command: "cargo xtask generate".to_owned(),
                reason: format!("could not write {}: {error}", absolute.display()),
            });
        }

        written.push(artifact.path.to_owned());
    }

    if check {
        if stale.is_empty() {
            println!("Generated output is current.");
            return Ok(());
        }

        eprintln!("{} generated file(s) differ from the schemas:", stale.len());
        for path in &stale {
            eprintln!("  - {path}");
        }
        eprintln!("Run `cargo xtask generate` and commit the result.");

        return Err(StepError {
            command: "cargo xtask generate --check".to_owned(),
            reason: format!("{} generated file(s) are stale", stale.len()),
        });
    }

    // 805 section 4 point 7: report changed contracts.
    if written.is_empty() {
        println!("Generated output was already current; nothing was rewritten.");
    } else {
        println!("Wrote {} generated file(s):", written.len());
        for path in &written {
            println!("  - {path}");
        }
    }

    if schemas.is_empty() {
        println!(
            "note: no schemas are defined yet, so the outputs are empty. The gate still \
             runs, so drift is caught from the first contract onward (MIR-0006)."
        );
    }

    Ok(())
}

/// Format Rust and TypeScript sources, or verify formatting.
fn fmt(check: bool, root: &Path) -> Result<(), StepError> {
    let steps: [&str; 2] = if check {
        ["cargo fmt --all -- --check", "pnpm exec prettier --check ."]
    } else {
        ["cargo fmt --all", "pnpm exec prettier --write ."]
    };

    runner::run_all(&steps, root)
}

/// Lint both workspaces with warnings denied.
fn lint(root: &Path) -> Result<(), StepError> {
    runner::run_all(
        &[
            "cargo clippy --workspace --all-targets --all-features -- -D warnings",
            "pnpm exec eslint . --max-warnings 0",
        ],
        root,
    )
}

/// Enforce the repository policies that can be checked mechanically.
fn enforce_policy(root: &Path) -> Result<(), StepError> {
    let files = policy::collect_text_files(root);
    let mut violations = Vec::new();
    let mut relative_paths = Vec::new();

    let mut exemptions = Vec::new();

    for file in &files {
        let relative = file
            .strip_prefix(root)
            .unwrap_or(file)
            .to_string_lossy()
            .replace('\\', "/");
        relative_paths.push(relative.clone());

        let Some(contents) = runner::read_file(file) else {
            continue;
        };

        // A file may exempt itself from a rule, with a reason. Every exemption is
        // listed in the report, so it stays a visible decision.
        let mut allowed: Vec<String> = Vec::new();
        for exemption in policy::declared_exemptions(&contents) {
            if exemption.reason.is_empty() {
                violations.push(policy::Violation {
                    rule: "policy-exemption".to_owned(),
                    location: relative.clone(),
                    detail: format!(
                        "exempts `{}` without a reason; write `policy-allow: {} - why`",
                        exemption.rule, exemption.rule
                    ),
                });
                continue;
            }

            allowed.push(exemption.rule.clone());
            exemptions.push((relative.clone(), exemption));
        }

        let allows = |rule: &str| allowed.iter().any(|allowed| allowed == rule);

        if !allows("secret") {
            violations.extend(policy::scan_secrets(&relative, &contents));
        }

        if !allows("local-path") {
            violations.extend(policy::scan_local_paths(&relative, &contents));
        }

        if relative.ends_with("Cargo.toml") {
            violations.extend(policy::check_dependency_direction(&relative, &contents));
        }

        if relative.ends_with("package.json") {
            violations.extend(policy::check_npm_pins(&relative, &contents));
        }
    }

    violations.extend(policy::check_env_files(&relative_paths));

    println!("Scanned {} text files.", files.len());

    if !exemptions.is_empty() {
        println!("{} declared exemption(s):", exemptions.len());
        for (path, exemption) in &exemptions {
            println!("  {path} allows `{}`: {}", exemption.rule, exemption.reason);
        }
    }

    if violations.is_empty() {
        println!("No policy violations.");
        println!(
            "note: secrets, local paths, environment files, dependency direction, and \
             npm pins are enforced here; TypeScript import rules live in \
             eslint.config.js, and cycles are rejected by cargo and pnpm."
        );
        return Ok(());
    }

    eprintln!("{} policy violation(s):", violations.len());
    for violation in &violations {
        eprintln!(
            "  [{}] {}: {}",
            violation.rule, violation.location, violation.detail
        );
    }

    Err(StepError {
        command: "cargo xtask policy".to_owned(),
        reason: format!("found {} violation(s)", violations.len()),
    })
}

/// Run tests for the requested scope.
fn test(scope: &TestScope, root: &Path) -> Result<(), StepError> {
    match scope {
        TestScope::All => runner::run_all(&["cargo test --workspace", "pnpm -r test"], root),
        TestScope::Affected => {
            println!(
                "note: affected-set detection is not implemented, so the full suite runs \
                 rather than implying narrower coverage. No ticket owns it yet."
            );
            runner::run_all(&["cargo test --workspace", "pnpm -r test"], root)
        }
        TestScope::Integration => {
            let tests_dir = root.join("tests");
            let has_tests = std::fs::read_dir(&tests_dir).is_ok_and(|entries| {
                entries.flatten().any(|entry| {
                    entry
                        .path()
                        .extension()
                        .is_some_and(|extension| extension == "rs" || extension == "ts")
                })
            });

            if has_tests {
                return runner::run("cargo test --workspace --tests", root);
            }

            println!(
                "No cross-cutting tests exist under tests/ yet; the first harness \
                 arrives with MIR-0015."
            );
            Ok(())
        }
    }
}

/// Validate documentation structure.
fn validate_docs(root: &Path) -> Result<(), StepError> {
    let docs_dir = root.join("docs");
    let summary_path = docs_dir.join("SUMMARY.md");

    let Some(summary) = runner::read_file(&summary_path) else {
        return Err(StepError {
            command: "cargo xtask docs".to_owned(),
            reason: format!("could not read {}", summary_path.display()),
        });
    };

    let findings = docs::validate(&summary, &docs_dir);
    let link_count = docs::extract_links(&summary).len();

    if findings.is_empty() {
        let documents = docs::collect_documents(&docs_dir).len();
        println!(
            "docs: {link_count} SUMMARY links resolve, every ADR indexed once, \
             {documents} documents carry their headers, ids are unique, and every \
             ADR reference exists."
        );
        return Ok(());
    }

    eprintln!("{} documentation problem(s):", findings.len());
    for finding in &findings {
        eprintln!("  - {}", finding.detail);
    }

    Err(StepError {
        command: "cargo xtask docs".to_owned(),
        reason: format!("found {} problem(s)", findings.len()),
    })
}

/// Run the baseline validation from `809` section 2, stopping at the first failure.
fn check(root: &Path) -> Result<(), StepError> {
    enforce_policy(root)?;
    generate(true, root)?;
    fmt(true, root)?;
    lint(root)?;
    test(&TestScope::All, root)?;
    validate_docs(root)?;

    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = repository_root();

    let command = match cli::parse(&args) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("error: {error}\n");
            eprint!("{}", cli::general_help());
            return ExitCode::FAILURE;
        }
    };

    let outcome = match command {
        Command::Help { topic } => {
            match topic {
                None => print!("{}", cli::general_help()),
                Some(topic) => match cli::command_help(&topic) {
                    Some(help) => print!("{help}"),
                    None => {
                        eprintln!("error: unknown command `{topic}`\n");
                        eprint!("{}", cli::general_help());
                        return ExitCode::FAILURE;
                    }
                },
            }
            Ok(())
        }
        Command::Bootstrap => {
            if toolchain::bootstrap(&root) {
                Ok(())
            } else {
                Err(StepError {
                    command: "cargo xtask bootstrap".to_owned(),
                    reason: "found toolchain problems".to_owned(),
                })
            }
        }
        Command::Generate { check } => generate(check, &root),
        Command::Fmt { check } => fmt(check, &root),
        Command::Lint => lint(&root),
        Command::Test { scope } => test(&scope, &root),
        Command::Docs { .. } => validate_docs(&root),
        Command::Policy => enforce_policy(&root),
        Command::Check => check(&root),
    };

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("\nxtask failed: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_root_holds_the_canonical_files() {
        let root = repository_root();

        assert!(root.join("Cargo.toml").is_file());
        assert!(root.join("DEPENDENCY_VERSIONS.md").is_file());
        assert!(root.join("docs").join("SUMMARY.md").is_file());
    }

    #[test]
    fn the_repository_summary_and_adr_index_are_valid() {
        let root = repository_root();
        let docs_dir = root.join("docs");
        let summary = runner::read_file(&docs_dir.join("SUMMARY.md")).unwrap_or_default();

        assert!(!summary.is_empty(), "docs/SUMMARY.md is empty");
        assert_eq!(docs::validate(&summary, &docs_dir), Vec::new());
    }
}

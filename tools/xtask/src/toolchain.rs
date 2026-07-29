//! Toolchain verification for `cargo xtask bootstrap`.
//!
//! Ported from the temporary `@mirae/toolchain-check` package that `MIR-0002`
//! added, which this crate replaces so the repository keeps one automation entry
//! point (`docs/08-development/806-build-system-and-toolchain.md` section 4).
//!
//! Parsing and comparison are pure functions; probing and file reads live in
//! `run`.

use std::fmt;
use std::path::Path;

use crate::json;
use crate::runner;

/// Which tool a pin or finding refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tool {
    Node,
    Pnpm,
    Rust,
}

impl fmt::Display for Tool {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Node => "node",
            Self::Pnpm => "pnpm",
            Self::Rust => "rust",
        };
        formatter.write_str(name)
    }
}

/// One version requirement resolved from a canonical file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pin {
    pub(crate) tool: Tool,
    pub(crate) version: String,
    /// The file and field the pin came from, shown verbatim to operators.
    pub(crate) source: String,
}

/// A problem an operator has to act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Finding {
    pub(crate) subject: String,
    pub(crate) detail: String,
    pub(crate) remediation: String,
}

/// Strip a leading `v` so `v24.18.1` and `24.18.1` compare equal.
pub(crate) fn normalize_version(raw: &str) -> String {
    raw.trim().trim_start_matches('v').to_owned()
}

/// True when the version is exactly `x.y.z`.
fn is_exact_version(version: &str) -> bool {
    let mut parts = version.split('.');
    let exact = [parts.next(), parts.next(), parts.next()]
        .iter()
        .all(|part| match part {
            Some(value) => !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()),
            None => false,
        });

    exact && parts.next().is_none()
}

/// Range operators and release tags the version lock forbids.
fn forbidden_pin(version: &str) -> bool {
    const TAGS: [&str; 5] = ["latest", "next", "canary", "beta", "rc"];

    version.starts_with(['^', '~', '>', '<', '=', '*'])
        || version.contains('*')
        || TAGS.iter().any(|tag| version.contains(tag))
}

/// Read the first meaningful line of a `.node-version` file.
pub(crate) fn parse_node_version_file(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .map(normalize_version)
}

/// Read `channel` from a `rust-toolchain.toml` file.
pub(crate) fn parse_rust_channel(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("channel"))
        .filter_map(|rest| rest.trim_start().strip_prefix('='))
        .map(|rest| rest.trim().trim_matches(['"', '\'']).to_owned())
        .find(|value| !value.is_empty())
}

/// Read the pnpm version from a `packageManager` field such as `pnpm@11.17.0`.
///
/// Returns the manager name alongside the version so a foreign manager is
/// reported rather than silently accepted.
pub(crate) fn parse_package_manager(field: &str) -> Option<(String, String)> {
    let (name, rest) = field.trim().split_once('@')?;
    if name.is_empty() {
        return None;
    }

    // Corepack allows a `+sha224.<hash>` suffix; the hash is not a version.
    let version = rest.split('+').next().unwrap_or(rest).trim();
    if version.is_empty() {
        return None;
    }

    Some((name.to_owned(), version.to_owned()))
}

/// Extract the first `x.y.z` from `--version` output.
pub(crate) fn extract_version(output: &str) -> Option<String> {
    for token in output.split(|character: char| {
        !character.is_ascii_digit() && character != '.' && character != '-'
    }) {
        let candidate = token.split('-').next().unwrap_or(token);
        if is_exact_version(candidate) {
            return Some(candidate.to_owned());
        }
    }

    None
}

/// Verify the canonical files agree with each other and use exact pins.
///
/// A self-contradictory lock is reported before any machine comparison, because
/// every later check would inherit the ambiguity.
pub(crate) fn check_pin_consistency(pins: &[Pin]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for tool in [Tool::Node, Tool::Pnpm, Tool::Rust] {
        let for_tool: Vec<&Pin> = pins.iter().filter(|pin| pin.tool == tool).collect();
        let mut versions: Vec<&str> = for_tool.iter().map(|pin| pin.version.as_str()).collect();
        versions.sort_unstable();
        versions.dedup();

        if versions.len() > 1 {
            let detail = for_tool
                .iter()
                .map(|pin| format!("{} = {}", pin.source, pin.version))
                .collect::<Vec<_>>()
                .join(", ");
            findings.push(Finding {
                subject: tool.to_string(),
                detail: format!("pin files disagree ({detail})"),
                remediation: format!(
                    "Make every {tool} pin identical and record the change in \
                     DEPENDENCY_VERSIONS.md in the same commit."
                ),
            });
        }
    }

    for pin in pins {
        if forbidden_pin(&pin.version) {
            findings.push(Finding {
                subject: pin.tool.to_string(),
                detail: format!(
                    "{} uses a range or release tag ({})",
                    pin.source, pin.version
                ),
                remediation: "DEPENDENCY_VERSIONS.md section 2 forbids ^, ~, *, >=, latest, \
                              next, canary, beta, and rc. Use one exact version."
                    .to_owned(),
            });
        } else if !is_exact_version(&pin.version) {
            findings.push(Finding {
                subject: pin.tool.to_string(),
                detail: format!(
                    "{} is not an exact x.y.z version ({})",
                    pin.source, pin.version
                ),
                remediation: format!("Pin {} to an exact version such as 1.2.3.", pin.tool),
            });
        }
    }

    findings
}

/// What is actually installed. `None` means the tool could not be run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Installed {
    pub(crate) node: Option<String>,
    pub(crate) pnpm: Option<String>,
    pub(crate) rustc: Option<String>,
    pub(crate) cargo: Option<String>,
}

/// How to install each tool, kept beside the comparison that needs it.
fn remediation(subject: &str, expected: &str) -> String {
    match subject {
        "node" => format!(
            "Install Node {expected} and activate it (nvm install {expected} && \
             nvm use {expected}). The pin lives in .node-version."
        ),
        "pnpm" => format!(
            "Activate pnpm {expected} (corepack enable && corepack use pnpm@{expected}). \
             npm, Yarn, Bun, and Deno are not project package managers."
        ),
        "rustc" | "cargo" => format!(
            "Install the pinned Rust toolchain (rustup toolchain install {expected}). \
             rustup reads rust-toolchain.toml automatically inside the repository."
        ),
        other => format!("Install {other} {expected}."),
    }
}

/// Compare the resolved pins against what is installed.
pub(crate) fn compare_installed(pins: &[Pin], installed: &Installed) -> Vec<Finding> {
    let mut findings = Vec::new();

    let checks: [(&str, Tool, &Option<String>); 4] = [
        ("node", Tool::Node, &installed.node),
        ("pnpm", Tool::Pnpm, &installed.pnpm),
        ("rustc", Tool::Rust, &installed.rustc),
        ("cargo", Tool::Rust, &installed.cargo),
    ];

    for (subject, tool, actual) in checks {
        let expected = pins.iter().find(|pin| pin.tool == tool);

        let Some(expected) = expected else {
            findings.push(Finding {
                subject: subject.to_owned(),
                detail: format!("no {tool} pin found in the canonical files"),
                remediation: format!("Declare the {tool} version, then re-run bootstrap."),
            });
            continue;
        };

        match actual {
            None => findings.push(Finding {
                subject: subject.to_owned(),
                detail: format!("not found on PATH (expected {})", expected.version),
                remediation: remediation(subject, &expected.version),
            }),
            Some(found) if *found != expected.version => findings.push(Finding {
                subject: subject.to_owned(),
                detail: format!("found {found}, expected {}", expected.version),
                remediation: remediation(subject, &expected.version),
            }),
            Some(_) => {}
        }
    }

    findings
}

/// Report a missing committed lockfile, which breaks reproducible installs.
pub(crate) fn check_lockfiles(pnpm_present: bool, cargo_present: bool) -> Vec<Finding> {
    let mut findings = Vec::new();

    if !pnpm_present {
        findings.push(Finding {
            subject: "pnpm-lock.yaml".to_owned(),
            detail: "missing".to_owned(),
            remediation: "Run pnpm install and commit pnpm-lock.yaml; \
                          pnpm install --frozen-lockfile cannot work without it."
                .to_owned(),
        });
    }

    if !cargo_present {
        findings.push(Finding {
            subject: "Cargo.lock".to_owned(),
            detail: "missing".to_owned(),
            remediation: "Run cargo check --workspace and commit Cargo.lock; deployable \
                          applications require a committed lockfile."
                .to_owned(),
        });
    }

    findings
}

/// Reject package managers that would write a second lockfile.
pub(crate) fn check_package_manager_agent(user_agent: Option<&str>) -> Vec<Finding> {
    let Some(agent) = user_agent.map(str::trim).filter(|agent| !agent.is_empty()) else {
        return Vec::new();
    };

    let name = agent.split('/').next().unwrap_or(agent);
    if name == "pnpm" {
        return Vec::new();
    }

    vec![Finding {
        subject: "package manager".to_owned(),
        detail: format!("invoked through {name}"),
        remediation: "Use pnpm. DEPENDENCY_VERSIONS.md section 2 forbids npm, Yarn, Bun, and \
                      Deno as project package managers; another manager would write a second \
                      lockfile and break reproducible installs."
            .to_owned(),
    }]
}

/// Resolve every declared pin from the canonical files.
fn resolve_pins(root: &Path) -> (Vec<Pin>, Vec<Finding>) {
    let mut pins = Vec::new();
    let mut findings = Vec::new();

    match runner::read_file(&root.join(".node-version"))
        .as_deref()
        .and_then(parse_node_version_file)
    {
        Some(version) => pins.push(Pin {
            tool: Tool::Node,
            version,
            source: ".node-version".to_owned(),
        }),
        None => findings.push(Finding {
            subject: "node".to_owned(),
            detail: ".node-version is missing or empty".to_owned(),
            remediation: "Create .node-version containing the exact Node version from \
                          DEPENDENCY_VERSIONS.md section 3."
                .to_owned(),
        }),
    }

    match runner::read_file(&root.join("package.json")) {
        None => findings.push(Finding {
            subject: "package.json".to_owned(),
            detail: "missing at the repository root".to_owned(),
            remediation: "Restore the root package.json.".to_owned(),
        }),
        Some(manifest) => {
            if let Some(node) = json::nested_string_field(&manifest, "engines", "node") {
                pins.push(Pin {
                    tool: Tool::Node,
                    version: normalize_version(node),
                    source: "package.json#engines.node".to_owned(),
                });
            }

            if let Some(pnpm) = json::nested_string_field(&manifest, "engines", "pnpm") {
                pins.push(Pin {
                    tool: Tool::Pnpm,
                    version: normalize_version(pnpm),
                    source: "package.json#engines.pnpm".to_owned(),
                });
            }

            match json::string_field(&manifest, "packageManager").and_then(parse_package_manager) {
                None => findings.push(Finding {
                    subject: "pnpm".to_owned(),
                    detail: "package.json#packageManager is missing or unparsable".to_owned(),
                    remediation: "Set \"packageManager\": \"pnpm@<exact version>\" so corepack \
                                  activates the pinned pnpm."
                        .to_owned(),
                }),
                Some((name, _)) if name != "pnpm" => findings.push(Finding {
                    subject: "package manager".to_owned(),
                    detail: format!("package.json#packageManager declares {name}"),
                    remediation: "Mirae uses pnpm. npm, Yarn, Bun, and Deno are not project \
                                  package managers."
                        .to_owned(),
                }),
                Some((_, version)) => pins.push(Pin {
                    tool: Tool::Pnpm,
                    version,
                    source: "package.json#packageManager".to_owned(),
                }),
            }
        }
    }

    match runner::read_file(&root.join("rust-toolchain.toml"))
        .as_deref()
        .and_then(parse_rust_channel)
    {
        Some(channel) => pins.push(Pin {
            tool: Tool::Rust,
            version: channel,
            source: "rust-toolchain.toml#channel".to_owned(),
        }),
        None => findings.push(Finding {
            subject: "rust".to_owned(),
            detail: "rust-toolchain.toml is missing or declares no channel".to_owned(),
            remediation: "Create rust-toolchain.toml with the exact channel from \
                          DEPENDENCY_VERSIONS.md section 11."
                .to_owned(),
        }),
    }

    (pins, findings)
}

/// Advise about the native compiler when none is visible on PATH.
///
/// Deliberately advisory rather than a failure. On Windows, `rustc` locates
/// `link.exe` through MSVC installation discovery, so a working machine has no
/// `cl` on PATH unless the shell is a Developer Command Prompt. Treating that as
/// an error fails machines that build correctly; only a real link attempt could
/// decide, and that is too expensive for a preinstall gate.
fn native_compiler_advice() -> Option<String> {
    let probes: &[&str] = if cfg!(windows) {
        &["cl --version", "gcc --version", "clang --version"]
    } else {
        &["cc --version", "gcc --version", "clang --version"]
    };

    if probes
        .iter()
        .any(|probe| runner::probe_version(probe).is_some())
    {
        return None;
    }

    let advice = if cfg!(windows) {
        "no C compiler on PATH. rustc can still link through MSVC discovery; if a \
         build fails with a linker error, install the Visual Studio Build Tools \
         with the C++ workload."
    } else {
        "no C compiler on PATH. If a build fails with a linker error, install a C \
         toolchain (build-essential on Debian and Ubuntu, Xcode command line tools \
         on macOS)."
    };

    Some(advice.to_owned())
}

/// Verify the toolchain and print an operator-facing report.
///
/// Returns `true` when everything matches. Makes no changes, so it is safe to run
/// repeatedly (`808` invariant 6).
pub(crate) fn bootstrap(root: &Path) -> bool {
    let (pins, mut findings) = resolve_pins(root);

    let installed = Installed {
        node: runner::probe_version("node --version"),
        pnpm: runner::probe_version("pnpm --version"),
        rustc: runner::probe_version("rustc --version"),
        cargo: runner::probe_version("cargo --version"),
    };

    findings.splice(
        0..0,
        check_package_manager_agent(std::env::var("npm_config_user_agent").ok().as_deref()),
    );
    findings.extend(check_pin_consistency(&pins));
    findings.extend(compare_installed(&pins, &installed));
    findings.extend(check_lockfiles(
        root.join("pnpm-lock.yaml").is_file(),
        root.join("Cargo.lock").is_file(),
    ));

    let expected_for = |tool: Tool| -> String {
        pins.iter()
            .find(|pin| pin.tool == tool)
            .map_or_else(|| "unpinned".to_owned(), |pin| pin.version.clone())
    };

    println!("Mirae toolchain check");
    for (subject, expected, actual) in [
        ("node", expected_for(Tool::Node), installed.node.clone()),
        ("pnpm", expected_for(Tool::Pnpm), installed.pnpm.clone()),
        ("rustc", expected_for(Tool::Rust), installed.rustc.clone()),
        ("cargo", expected_for(Tool::Rust), installed.cargo.clone()),
    ] {
        let found = actual.unwrap_or_else(|| "not found".to_owned());
        let status = if found == expected { "ok  " } else { "FAIL" };
        println!("  {status} {subject:<6} expected {expected:<10} found {found}");
    }

    if let Some(advice) = native_compiler_advice() {
        println!("  note   {advice}");
    }

    if findings.is_empty() {
        println!("\nToolchain matches DEPENDENCY_VERSIONS.md.");
        return true;
    }

    eprintln!("\n{} problem(s) to fix:", findings.len());
    for (index, finding) in findings.iter().enumerate() {
        eprintln!(
            "\n{}. {}: {}\n   -> {}",
            index + 1,
            finding.subject,
            finding.detail,
            finding.remediation
        );
    }
    eprintln!("\nAuthoritative lock: DEPENDENCY_VERSIONS.md");
    eprintln!("To install dependencies without this gate: pnpm install --ignore-scripts");

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pin(tool: Tool, version: &str, source: &str) -> Pin {
        Pin {
            tool,
            version: version.to_owned(),
            source: source.to_owned(),
        }
    }

    fn all_pins() -> Vec<Pin> {
        vec![
            pin(Tool::Node, "24.18.1", ".node-version"),
            pin(Tool::Pnpm, "11.17.0", "package.json#packageManager"),
            pin(Tool::Rust, "1.97.1", "rust-toolchain.toml#channel"),
        ]
    }

    fn matching() -> Installed {
        Installed {
            node: Some("24.18.1".to_owned()),
            pnpm: Some("11.17.0".to_owned()),
            rustc: Some("1.97.1".to_owned()),
            cargo: Some("1.97.1".to_owned()),
        }
    }

    #[test]
    fn reads_the_node_version_file() {
        assert_eq!(
            parse_node_version_file("v24.18.1\n").as_deref(),
            Some("24.18.1")
        );
        assert_eq!(
            parse_node_version_file("\n# pinned by MIR-0002\n24.18.1\n").as_deref(),
            Some("24.18.1")
        );
        assert_eq!(parse_node_version_file("  \n\n"), None);
    }

    #[test]
    fn reads_the_rust_channel() {
        let toml = "[toolchain]\nchannel = \"1.97.1\"\nprofile = \"minimal\"\n";
        assert_eq!(parse_rust_channel(toml).as_deref(), Some("1.97.1"));
        assert_eq!(
            parse_rust_channel("[toolchain]\nprofile = \"minimal\"\n"),
            None
        );
    }

    #[test]
    fn reads_the_package_manager_field() {
        assert_eq!(
            parse_package_manager("pnpm@11.17.0"),
            Some(("pnpm".to_owned(), "11.17.0".to_owned()))
        );
        assert_eq!(
            parse_package_manager("pnpm@11.17.0+sha224.abcdef").map(|(_, version)| version),
            Some("11.17.0".to_owned())
        );
        assert_eq!(
            parse_package_manager("yarn@4.0.0").map(|(name, _)| name),
            Some("yarn".to_owned())
        );
        assert_eq!(parse_package_manager("pnpm"), None);
    }

    #[test]
    fn extracts_versions_from_tool_output() {
        assert_eq!(
            extract_version("rustc 1.97.1 (8bab26f4f 2026-07-14)").as_deref(),
            Some("1.97.1")
        );
        assert_eq!(extract_version("11.17.0\n").as_deref(), Some("11.17.0"));
        assert_eq!(extract_version("v24.18.1").as_deref(), Some("24.18.1"));
        assert_eq!(extract_version("command not found"), None);
    }

    #[test]
    fn accepts_pins_that_agree() {
        assert_eq!(check_pin_consistency(&all_pins()), Vec::new());
    }

    #[test]
    fn rejects_pin_files_that_disagree() {
        let pins = vec![
            pin(Tool::Node, "24.18.1", ".node-version"),
            pin(Tool::Node, "24.18.0", "package.json#engines.node"),
        ];
        let findings = check_pin_consistency(&pins);

        assert_eq!(findings.len(), 1);
        assert!(findings[0].detail.contains("disagree"));
        assert!(findings[0].detail.contains("24.18.0"));
    }

    #[test]
    fn rejects_ranges_and_release_tags() {
        for version in [
            "^24.18.1",
            "~24.18.1",
            ">=24.18.1",
            "*",
            "latest",
            "24.1.0-beta",
        ] {
            let findings = check_pin_consistency(&[pin(Tool::Node, version, ".node-version")]);
            assert_eq!(findings.len(), 1, "`{version}` was accepted");
            assert!(findings[0].remediation.contains("exact"));
        }
    }

    #[test]
    fn rejects_a_non_exact_version() {
        let findings = check_pin_consistency(&[pin(Tool::Node, "24.18", ".node-version")]);
        assert!(findings[0].detail.contains("not an exact"));
    }

    #[test]
    fn passes_when_every_tool_matches() {
        assert_eq!(compare_installed(&all_pins(), &matching()), Vec::new());
    }

    #[test]
    fn reports_a_mismatch_with_both_versions_and_a_fix() {
        let installed = Installed {
            node: Some("24.15.0".to_owned()),
            ..matching()
        };
        let findings = compare_installed(&all_pins(), &installed);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].detail, "found 24.15.0, expected 24.18.1");
        assert!(findings[0].remediation.contains("nvm install 24.18.1"));
    }

    #[test]
    fn reports_a_missing_tool_instead_of_failing() {
        let installed = Installed {
            rustc: None,
            ..matching()
        };
        let findings = compare_installed(&all_pins(), &installed);

        assert!(findings[0].detail.contains("not found on PATH"));
        assert!(
            findings[0]
                .remediation
                .contains("rustup toolchain install 1.97.1")
        );
    }

    #[test]
    fn checks_cargo_separately_from_rustc() {
        let installed = Installed {
            cargo: Some("1.96.0".to_owned()),
            ..matching()
        };
        let findings = compare_installed(&all_pins(), &installed);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].subject, "cargo");
    }

    #[test]
    fn reports_an_undeclared_pin() {
        let pins = vec![pin(Tool::Node, "24.18.1", ".node-version")];
        let findings = compare_installed(&pins, &matching());

        // pnpm, rustc, and cargo have no pin to compare against.
        assert_eq!(findings.len(), 3);
        assert!(findings[0].detail.contains("no pnpm pin"));
    }

    #[test]
    fn accepts_pnpm_as_the_invoking_manager() {
        assert_eq!(
            check_package_manager_agent(Some("pnpm/11.17.0 npm/? node/v24.18.1")),
            Vec::new()
        );
        assert_eq!(check_package_manager_agent(None), Vec::new());
        assert_eq!(check_package_manager_agent(Some("  ")), Vec::new());
    }

    #[test]
    fn rejects_foreign_package_managers() {
        for agent in ["npm/10.9.0 node/v24.18.1", "yarn/4.0.0", "bun/1.1.0"] {
            let findings = check_package_manager_agent(Some(agent));
            assert_eq!(findings.len(), 1, "`{agent}` was accepted");
            assert!(findings[0].remediation.contains("Use pnpm"));
        }
    }

    #[test]
    fn reports_each_missing_lockfile() {
        assert_eq!(check_lockfiles(true, true), Vec::new());

        let findings = check_lockfiles(false, false);
        let subjects: Vec<&str> = findings
            .iter()
            .map(|finding| finding.subject.as_str())
            .collect();

        assert_eq!(subjects, vec!["pnpm-lock.yaml", "Cargo.lock"]);
    }
}

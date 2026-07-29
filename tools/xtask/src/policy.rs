//! Repository policy checks for `cargo xtask policy`.
//!
//! Canonical documentation:
//! - `docs/08-development/804-dependency-rules.md` (forbidden dependencies)
//! - `docs/08-development/805-generated-contracts-and-schemas.md` (drift)
//! - `DEPENDENCY_VERSIONS.md` section 2 (pin syntax)
//! - `CLAUDE.md` (secrets never enter project files, logs, or config)
//!
//! Every matcher is a pure function over text so its true and false positives are
//! unit tested. Matching is hand-written: a regular-expression crate would have to
//! clear the Rust dependency procedure in `DEPENDENCY_VERSIONS.md` section 11.

use std::path::{Path, PathBuf};

/// A policy violation, reported with the file and line that triggered it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Violation {
    pub(crate) rule: String,
    pub(crate) location: String,
    pub(crate) detail: String,
}

/// Directories that never contain reviewable source.
const SKIPPED_DIRECTORIES: [&str; 8] = [
    ".git",
    "node_modules",
    "target",
    "dist",
    ".vite",
    "coverage",
    "playwright-report",
    "test-results",
];

/// Extensions worth scanning as text.
const TEXT_EXTENSIONS: [&str; 16] = [
    "rs", "ts", "tsx", "js", "mjs", "cjs", "json", "toml", "yaml", "yml", "md", "css", "html",
    "sh", "ps1", "env",
];

/// Collect every text file under `root`, skipping build output and vendor trees.
pub(crate) fn collect_text_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();

            if path.is_dir() {
                if !SKIPPED_DIRECTORIES.contains(&name.as_ref()) {
                    pending.push(path);
                }
                continue;
            }

            let is_text = path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| TEXT_EXTENSIONS.contains(&extension))
                || name.starts_with(".env");

            if is_text {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

/// Placeholders that look like credentials but carry no secret.
const PLACEHOLDERS: [&str; 10] = [
    "example",
    "changeme",
    "placeholder",
    "your-",
    "<",
    "xxx",
    "process.env",
    "${",
    "redacted",
    "dummy",
];

/// True when a value is a placeholder rather than a real credential.
fn is_placeholder(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    PLACEHOLDERS
        .iter()
        .any(|placeholder| lowered.contains(placeholder))
}

/// Credential-shaped prefixes that are unambiguous on sight.
const TOKEN_PREFIXES: [&str; 6] = [
    "-----BEGIN ",
    "AKIA",
    "ghp_",
    "github_pat_",
    "xoxb-",
    "sk_live_",
];

/// Field name endings whose assigned literal must not look like a credential.
///
/// Matched against the assignment's key reduced to lowercase alphanumerics, and
/// only at the end, so `tokens.css` and `"path": "tokens/design-tokens.css"` do
/// not masquerade as a `token` field.
const SECRET_FIELDS: [&str; 8] = [
    "apikey",
    "secret",
    "password",
    "passwd",
    "token",
    "privatekey",
    "credential",
    "accesskey",
];

/// Split an assignment into its key and quoted value.
fn assignment(line: &str) -> Option<(&str, &str)> {
    let separator = line.find([':', '='])?;
    let key = line[..separator].trim();
    let rest = &line[separator + 1..];

    let open = rest.find(['"', '\''])?;
    let quote = rest.as_bytes().get(open).copied()?;
    let value_start = open + 1;
    let close = rest[value_start..].find(quote as char)?;

    Some((key, &rest[value_start..value_start + close]))
}

/// Reduce a key to lowercase alphanumerics for comparison.
fn normalized_key(key: &str) -> String {
    key.chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>()
        .to_ascii_lowercase()
}

/// True when the key names a secret, judged on the final word of the key.
fn names_a_secret(key: &str) -> bool {
    let normalized = normalized_key(key);

    SECRET_FIELDS
        .iter()
        .any(|field| normalized.ends_with(field))
}

/// True when a value is a path or module specifier rather than a credential.
fn looks_like_path(value: &str) -> bool {
    value.contains('/')
        || value.contains('\\')
        || value.starts_with('.')
        || value.ends_with(".css")
        || value.ends_with(".json")
        || value.ends_with(".ts")
}

/// True when a literal is long and mixed enough to be a real credential.
fn looks_like_credential(value: &str) -> bool {
    if value.len() < 20 || is_placeholder(value) || looks_like_path(value) {
        return false;
    }

    let has_lower = value.bytes().any(|byte| byte.is_ascii_lowercase());
    let has_upper_or_digit = value
        .bytes()
        .any(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit());
    let mostly_credential_charset = value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b"-_+/=.".contains(&byte));

    has_lower && has_upper_or_digit && mostly_credential_charset
}

/// Scan a file's text for committed secrets.
pub(crate) fn scan_secrets(path: &str, contents: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    for (number, line) in contents.lines().enumerate() {
        let location = format!("{path}:{}", number + 1);

        if let Some(prefix) = TOKEN_PREFIXES.iter().find(|prefix| line.contains(*prefix)) {
            violations.push(Violation {
                rule: "secret".to_owned(),
                location: location.clone(),
                detail: format!("line contains a credential marker (`{prefix}`)"),
            });
            continue;
        }

        let Some((key, value)) = assignment(line) else {
            continue;
        };

        if names_a_secret(key) && looks_like_credential(value) {
            violations.push(Violation {
                rule: "secret".to_owned(),
                location,
                detail: format!(
                    "the secret-named field `{}` is assigned a credential-shaped literal",
                    key.trim_matches(['"', '\'', ' '])
                ),
            });
        }
    }

    violations
}

/// Machine-local path prefixes that must never be committed.
///
/// Both the raw and the source-escaped Windows forms are listed: a path inside a
/// JavaScript or JSON string literal appears as `C:\\Users\\`, which the raw
/// marker alone would miss.
const LOCAL_PATH_MARKERS: [&str; 5] = [
    "C:\\Users\\",
    "C:\\\\Users\\\\",
    "C:/Users/",
    "/home/",
    "/Users/",
];

/// Scan a file's text for machine-local absolute paths.
pub(crate) fn scan_local_paths(path: &str, contents: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    for (number, line) in contents.lines().enumerate() {
        if let Some(marker) = LOCAL_PATH_MARKERS
            .iter()
            .find(|marker| line.contains(*marker))
        {
            violations.push(Violation {
                rule: "local-path".to_owned(),
                location: format!("{path}:{}", number + 1),
                detail: format!("machine-local path (`{marker}`) would be committed"),
            });
        }
    }

    violations
}

/// A dependency-direction rule: crates under `group` may not depend on `forbidden`.
struct DirectionRule {
    group: &'static str,
    forbidden: &'static [&'static str],
    reason: &'static str,
}

/// Rules from `804` section 3, expressed over crate path groups and crate names.
const DIRECTION_RULES: [DirectionRule; 4] = [
    DirectionRule {
        group: "crates/foundation/",
        forbidden: &[
            "mirae-domain",
            "mirae-runtime",
            "mirae-platform",
            "mirae-renderer",
            "wgpu",
            "ffmpeg",
        ],
        reason: "foundation is the lowest layer and may not depend on anything above it",
    },
    DirectionRule {
        group: "crates/domain/",
        forbidden: &[
            "mirae-platform",
            "mirae-renderer",
            "mirae-media-ffmpeg",
            "mirae-runtime",
            "wgpu",
            "ffmpeg",
        ],
        reason: "domain must stay independent of platform, GPU, media, and framework code",
    },
    DirectionRule {
        group: "crates/",
        forbidden: &["mirae-engine", "mirae-shell", "mirae-extension-host"],
        reason: "a shared library must not depend on a deployable application",
    },
    DirectionRule {
        group: "crates/sdk/",
        forbidden: &["mirae-runtime", "mirae-state", "mirae-project"],
        reason: "the public extension SDK must not depend on engine internals",
    },
];

/// Read dependency names from a `Cargo.toml`.
///
/// Recognises `name = ...` and `name.workspace = true` inside any
/// `[dependencies]`-like table, which is every form this repository uses.
pub(crate) fn cargo_dependency_names(manifest: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_dependencies = false;

    for line in manifest.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            in_dependencies = line.contains("dependencies");
            continue;
        }

        if !in_dependencies || line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((name, _)) = line.split_once('=') {
            let name = name.trim().split('.').next().unwrap_or("").trim();
            if !name.is_empty() {
                names.push(name.to_owned());
            }
        }
    }

    names
}

/// Check one crate manifest against the dependency-direction rules.
///
/// `relative_path` is the manifest path relative to the repository root, using
/// forward slashes.
pub(crate) fn check_dependency_direction(relative_path: &str, manifest: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let dependencies = cargo_dependency_names(manifest);

    for rule in &DIRECTION_RULES {
        if !relative_path.starts_with(rule.group) {
            continue;
        }

        for dependency in &dependencies {
            // `mirae-platform` also forbids `mirae-platform-windows`.
            if rule
                .forbidden
                .iter()
                .any(|forbidden| dependency.starts_with(forbidden))
            {
                violations.push(Violation {
                    rule: "dependency-direction".to_owned(),
                    location: relative_path.to_owned(),
                    detail: format!("depends on `{dependency}`: {} (804 section 3)", rule.reason),
                });
            }
        }
    }

    violations
}

/// Check a `package.json` dependency version for an allowed form.
///
/// The version lock permits only `catalog:`, `workspace:` references, and exact
/// versions; ranges and release tags are forbidden.
pub(crate) fn is_allowed_npm_version(version: &str) -> bool {
    if version.starts_with("catalog:") || version.starts_with("workspace:") {
        return true;
    }

    let mut parts = version.split('.');
    let exact = [parts.next(), parts.next(), parts.next()]
        .iter()
        .all(|part| match part {
            Some(value) => !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()),
            None => false,
        });

    exact && parts.next().is_none()
}

/// Dependency tables in a `package.json`.
const NPM_DEPENDENCY_TABLES: [&str; 4] = [
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
];

/// Check every dependency version in a `package.json` for an allowed form.
pub(crate) fn check_npm_pins(relative_path: &str, manifest: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut current_table: Option<&str> = None;

    for line in manifest.lines() {
        let trimmed = line.trim();

        if let Some(table) = NPM_DEPENDENCY_TABLES
            .iter()
            .find(|table| trimmed.starts_with(&format!("\"{table}\"")))
        {
            current_table = Some(table);
            continue;
        }

        if trimmed == "}" || trimmed == "}," {
            current_table = None;
            continue;
        }

        let Some(table) = current_table else {
            continue;
        };

        let Some((name, value)) = trimmed.split_once(':') else {
            continue;
        };

        let name = name.trim().trim_matches('"');
        let version = value.trim().trim_end_matches(',').trim().trim_matches('"');

        if version.is_empty() || is_allowed_npm_version(version) {
            continue;
        }

        violations.push(Violation {
            rule: "npm-pin".to_owned(),
            location: relative_path.to_owned(),
            detail: format!(
                "{table}.{name} is `{version}`: DEPENDENCY_VERSIONS.md section 2 allows only \
                 an exact version, `catalog:`, or `workspace:`"
            ),
        });
    }

    violations
}

/// Report a committed environment file, which is where secrets leak first.
pub(crate) fn check_env_files(paths: &[String]) -> Vec<Violation> {
    paths
        .iter()
        .filter(|path| {
            let name = path.rsplit('/').next().unwrap_or(path);
            name.starts_with(".env") && !name.ends_with(".example")
        })
        .map(|path| Violation {
            rule: "secret".to_owned(),
            location: path.clone(),
            detail: "an environment file is committed; use a .env.example template instead"
                .to_owned(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_a_private_key_block() {
        let violations = scan_secrets("a.pem", "-----BEGIN RSA PRIVATE KEY-----");

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].location, "a.pem:1");
    }

    #[test]
    fn flags_provider_token_prefixes() {
        for line in [
            "aws = \"AKIAIOSFODNN7EXAMPLE\"",
            "gh = \"ghp_16CharactersOrMoreHere\"",
            "slack = \"xoxb-1234-5678-abcdefg\"",
        ] {
            assert_eq!(
                scan_secrets("config.toml", line).len(),
                1,
                "missed `{line}`"
            );
        }
    }

    #[test]
    fn flags_a_secret_field_with_a_credential_value() {
        let line = "  api_key: \"A1b2C3d4E5f6G7h8I9j0KlMn\"";

        assert_eq!(scan_secrets("config.ts", line).len(), 1);
    }

    #[test]
    fn does_not_flag_placeholders_or_environment_lookups() {
        for line in [
            "  apiKey: \"your-api-key-here\"",
            "  token: process.env.MIRAE_TOKEN",
            "  password: \"<redacted>\"",
            "  secret: \"changeme\"",
            "  token: \"${INPUT_TOKEN}\"",
        ] {
            assert_eq!(scan_secrets("a.ts", line), Vec::new(), "flagged `{line}`");
        }
    }

    #[test]
    fn does_not_flag_token_shaped_paths() {
        // Regression: `tokens.css` and token file paths matched `token` as a
        // substring of the whole line, so a design-token path read as a secret.
        for line in [
            "    \"./styles/tokens.css\": \"./src/styles/tokens.css\"",
            "      \"path\": \"tokens/design-tokens-v1.css\",",
            "      \"path\": \"packages/ui-kit/tokens/design-tokens.v1.json\",",
            "  \"sha256\": \"9a4fa20ab0c6ad9851b52de6f595013883375e5e6daa06b93a573252c2e6743f\"",
        ] {
            assert_eq!(
                scan_secrets("package.json", line),
                Vec::new(),
                "flagged `{line}`"
            );
        }
    }

    #[test]
    fn matches_secret_names_only_at_the_end_of_a_key() {
        assert!(names_a_secret("api_key"));
        assert!(names_a_secret("\"accessToken\""));
        assert!(names_a_secret("MIRAE_PASSWORD"));
        assert!(!names_a_secret("\"./styles/tokens.css\""));
        assert!(!names_a_secret("tokenCount"));
        assert!(!names_a_secret("\"path\""));
    }

    #[test]
    fn does_not_flag_prose_or_field_names_without_values() {
        for line in [
            "Secrets never enter project files, logs, telemetry, or bundles.",
            "  /// The credential broker mediates every token request.",
            "  pub struct Password { hash: Hash }",
        ] {
            assert_eq!(scan_secrets("a.rs", line), Vec::new(), "flagged `{line}`");
        }
    }

    #[test]
    fn flags_machine_local_paths() {
        let contents = "const root = \"C:\\\\Users\\\\Arthur\\\\dev\";";
        let violations = scan_local_paths("a.ts", contents);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "local-path");
    }

    #[test]
    fn accepts_relative_paths() {
        assert_eq!(
            scan_local_paths("a.ts", "const root = \"./packages/ui-kit\";"),
            Vec::new()
        );
    }

    #[test]
    fn reads_cargo_dependency_names() {
        let manifest = "\
[package]
name = \"mirae-domain\"

[dependencies]
mirae-types = { path = \"../../foundation/types\" }
serde.workspace = true
# commented = \"1.0\"

[dev-dependencies]
mirae-test-support = { path = \"../../test-support/test-support\" }
";

        assert_eq!(
            cargo_dependency_names(manifest),
            vec!["mirae-types", "serde", "mirae-test-support"]
        );
    }

    #[test]
    fn flags_domain_depending_on_platform() {
        let manifest = "[dependencies]\nmirae-platform-windows = { path = \"x\" }\n";
        let violations = check_dependency_direction("crates/domain/domain/Cargo.toml", manifest);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].detail.contains("mirae-platform-windows"));
    }

    #[test]
    fn flags_domain_depending_on_wgpu() {
        let manifest = "[dependencies]\nwgpu = \"=0.1.0\"\n";
        let violations = check_dependency_direction("crates/domain/state/Cargo.toml", manifest);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn flags_a_library_depending_on_an_application() {
        let manifest = "[dependencies]\nmirae-engine = { path = \"x\" }\n";
        let violations = check_dependency_direction("crates/media/audio/Cargo.toml", manifest);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].detail.contains("deployable application"));
    }

    #[test]
    fn flags_the_sdk_depending_on_engine_internals() {
        let manifest = "[dependencies]\nmirae-state = { path = \"x\" }\n";
        let violations = check_dependency_direction("crates/sdk/sdk-protocol/Cargo.toml", manifest);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].detail.contains("engine internals"));
    }

    #[test]
    fn allows_dependencies_that_flow_downward() {
        let manifest = "[dependencies]\nmirae-types = { path = \"x\" }\n";

        assert_eq!(
            check_dependency_direction("crates/domain/domain/Cargo.toml", manifest),
            Vec::new()
        );
        assert_eq!(
            check_dependency_direction(
                "crates/rendering/renderer-wgpu/Cargo.toml",
                "[dependencies]\nwgpu = \"=0.1.0\"\n"
            ),
            Vec::new()
        );
    }

    #[test]
    fn accepts_allowed_npm_version_forms() {
        for version in ["catalog:", "workspace:*", "19.2.8"] {
            assert!(is_allowed_npm_version(version), "rejected `{version}`");
        }
    }

    #[test]
    fn rejects_ranges_and_tags_in_npm_versions() {
        for version in ["^19.2.8", "~19.2.8", "*", "latest", "19", "19.2"] {
            assert!(!is_allowed_npm_version(version), "accepted `{version}`");
        }
    }

    #[test]
    fn flags_a_ranged_dependency_in_a_manifest() {
        let manifest = "\
{
  \"dependencies\": {
    \"react\": \"catalog:\",
    \"left-pad\": \"^1.3.0\"
  }
}";
        let violations = check_npm_pins("apps/control-ui/package.json", manifest);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].detail.contains("left-pad"));
    }

    #[test]
    fn ignores_non_dependency_fields_in_a_manifest() {
        let manifest = "\
{
  \"name\": \"mirae\",
  \"scripts\": {
    \"lint\": \"eslint .\"
  }
}";

        assert_eq!(check_npm_pins("package.json", manifest), Vec::new());
    }

    #[test]
    fn flags_a_committed_environment_file() {
        let paths = vec![
            "apps/control-ui/.env".to_owned(),
            "apps/control-ui/.env.example".to_owned(),
        ];
        let violations = check_env_files(&paths);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].location, "apps/control-ui/.env");
    }
}

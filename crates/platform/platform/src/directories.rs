//! Where Mirae keeps machine-local data.
//!
//! Canonical documentation: `docs/05-platform/500-platform-overview.md`,
//! ADR-0072.
//!
//! ADR-0072 resolves these from environment variables rather than through a
//! crate, and states the rules it enforces. This module is the whole
//! implementation of that decision: everything above it takes a directory as a
//! parameter, so nothing but this file knows what `%LOCALAPPDATA%` is.
//!
//! Every function returns `Option`. A machine with none of the expected
//! variables set gets `None`, and the caller reports that the feature is
//! unavailable. Inventing a path — the working directory, a temporary
//! directory, next to the executable — would scatter a user's data wherever they
//! happened to launch from, which is worse than not having the feature.

use std::path::PathBuf;

/// The directory name Mirae uses under a platform's data root.
///
/// Capitalized on Windows and macOS, lowercase on Linux, because that is what
/// each platform's users expect to see when they look.
const APPLICATION_DIRECTORY: &str = if cfg!(target_os = "linux") {
    "mirae"
} else {
    "Mirae"
};

/// Read an environment variable that must name an absolute path.
///
/// A relative value is treated as absent. ADR-0072 records this as stricter than
/// the XDG specification allows: a relative data directory would resolve against
/// whatever the process happened to start in, which is not a location anybody
/// chose.
fn absolute_from_environment(variable: &str) -> Option<PathBuf> {
    let value = std::env::var(variable).ok()?;

    if value.is_empty() {
        return None;
    }

    let path = PathBuf::from(value);
    path.is_absolute().then_some(path)
}

/// The machine-local data directory for this platform.
///
/// Local rather than roaming, config, or cache (ADR-0072): roaming would copy
/// working state between machines, config would mix it with preferences, and
/// cache invites the operating system to delete it.
#[must_use]
pub fn local_data_directory() -> Option<PathBuf> {
    let root = if cfg!(windows) {
        absolute_from_environment("LOCALAPPDATA")?
    } else if cfg!(target_os = "macos") {
        absolute_from_environment("HOME")?
            .join("Library")
            .join("Application Support")
    } else {
        absolute_from_environment("XDG_DATA_HOME").unwrap_or(
            absolute_from_environment("HOME")?
                .join(".local")
                .join("share"),
        )
    };

    Some(root.join(APPLICATION_DIRECTORY))
}

/// Where recovery records live (`404` section 2).
///
/// A directory of its own under the data root, so retention can clear it without
/// reasoning about what else might be in there.
#[must_use]
pub fn recovery_directory() -> Option<PathBuf> {
    Some(local_data_directory()?.join("recovery"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_relative_environment_value_is_treated_as_absent() {
        // The rule ADR-0072 records: a relative data directory resolves against
        // wherever the process started, which is not a location anybody chose.
        // SAFETY: the variable is Mirae's own and is not read by anything else.
        unsafe {
            std::env::set_var("MIRAE_TEST_RELATIVE", "some/relative/path");
        }

        assert_eq!(absolute_from_environment("MIRAE_TEST_RELATIVE"), None);

        unsafe {
            std::env::remove_var("MIRAE_TEST_RELATIVE");
        }
    }

    #[test]
    fn an_empty_environment_value_is_treated_as_absent() {
        // SAFETY: as above.
        unsafe {
            std::env::set_var("MIRAE_TEST_EMPTY", "");
        }

        assert_eq!(absolute_from_environment("MIRAE_TEST_EMPTY"), None);

        unsafe {
            std::env::remove_var("MIRAE_TEST_EMPTY");
        }
    }

    #[test]
    fn an_unset_variable_is_absent() {
        assert_eq!(
            absolute_from_environment("MIRAE_TEST_DEFINITELY_UNSET"),
            None
        );
    }

    #[test]
    fn the_recovery_directory_sits_under_the_data_directory() {
        // Skipped rather than failed on a machine with no data directory: the
        // point is the relationship between the two paths, and a build machine
        // without `HOME` is a real thing rather than a bug in this code.
        let (Some(data), Some(recovery)) = (local_data_directory(), recovery_directory()) else {
            return;
        };

        assert!(recovery.starts_with(&data));
        assert!(recovery.ends_with("recovery"));
        assert!(recovery.is_absolute());
    }

    #[test]
    fn the_data_directory_ends_with_the_application_name() {
        let Some(data) = local_data_directory() else {
            return;
        };

        assert!(data.ends_with(APPLICATION_DIRECTORY));
    }
}

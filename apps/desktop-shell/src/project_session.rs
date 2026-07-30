//! The active project, as the shell holds it.
//!
//! Canonical documentation: `docs/04-project/403-persistence.md`,
//! `docs/05-platform/501-desktop-shell.md` section 2.
//!
//! The shell owns the *session*: which project is open, where it lives, and
//! whether it has unsaved work. It does not own the project. Every mutation goes
//! through `mirae-project`, which goes through a command and a transaction, so
//! the shell has no path to change state that a command could not.
//!
//! `501` invariant 1 says the shell does not own authoritative project state.
//! That holds here because the shell holds a `StateStore` it can only read and a
//! `SaveState` derived from generations — there is nothing to keep in sync,
//! because nothing is duplicated.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use mirae_commands::{ActorContext, CommandEnvelope, CommandError, CommandId, LifecyclePhase};
use mirae_project::{
    CreateProject, Durability, FileIdentity, SaveState, clean_stale_temporaries,
    create_empty_project, envelope_of, save_project,
};
use mirae_state::StateStore;

/// The version this build writes into a project file.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A project the shell currently has open.
pub(crate) struct ProjectSession {
    store: StateStore,
    save_state: SaveState,
    name: String,
    path: Option<PathBuf>,
    identity: Option<FileIdentity>,
    created_at: String,
}

impl ProjectSession {
    /// Create an empty project through the command path.
    pub(crate) fn create(name: &str, engine_session_id: &str) -> Result<Self, CommandError> {
        let command = CommandEnvelope {
            command_id: CommandId::new(),
            engine_session_id: engine_session_id.to_owned(),
            actor: ActorContext::local_ui(),
            expected_generation: None,
            idempotency_key: None,
            issued_at_millis: None,
            payload: CreateProject {
                name: name.to_owned(),
            },
        };

        // `Ready`: no project is open at the moment this runs, which is the
        // phase `CreateProject` accepts. Replacing an open project is a close
        // followed by a create, deliberately, rather than a silent switch.
        let (store, created) =
            create_empty_project(&command, engine_session_id, LifecyclePhase::Ready)?;
        let save_state = SaveState::unsaved(created.generation);

        Ok(Self {
            store,
            save_state,
            name: created.name,
            path: None,
            identity: None,
            created_at: timestamp(),
        })
    }

    /// The project's name.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// Where it was last saved, if anywhere.
    pub(crate) fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    /// How memory and disk relate.
    pub(crate) const fn save_state(&self) -> SaveState {
        self.save_state
    }

    /// Save to `path`, or to wherever it was last saved.
    ///
    /// Returns the generation written. The save is explicit, so it uses at least
    /// `Normal` durability (`403` section 6).
    pub(crate) fn save(&mut self, path: Option<PathBuf>) -> Result<(), SaveFailure> {
        let destination = path
            .or_else(|| self.path.clone())
            .ok_or(SaveFailure::NoDestination)?;

        let snapshot = self.store.snapshot();
        let generation = snapshot.generation();
        let envelope = envelope_of(
            snapshot.state(),
            &self.created_at,
            &timestamp(),
            APP_VERSION,
        );

        self.save_state.begin_save(generation);

        let result = save_project(
            &envelope,
            generation,
            &destination,
            self.identity.as_ref(),
            Durability::Normal,
        );

        match result {
            Ok(result) => {
                self.save_state.complete_save(result.saved_generation);
                self.identity = result.identity;

                // A previous run that died between writing a temporary file and
                // renaming it left debris (`403` section 5). The run that
                // succeeds afterwards is the one that knows it is stale.
                let removed = clean_stale_temporaries(&result.path);

                if removed > 0 {
                    let mut out = std::io::stdout().lock();
                    let _ = std::io::Write::write_fmt(
                        &mut out,
                        format_args!(
                            "cleaned {removed} stale save file(s)
"
                        ),
                    );
                }

                self.path = Some(result.path);
                Ok(())
            }
            Err(error) => {
                self.save_state.fail_save();
                Err(SaveFailure::Refused(error))
            }
        }
    }
}

/// Why a save did not happen.
#[derive(Debug)]
pub(crate) enum SaveFailure {
    /// The project has never been saved and no path was given.
    NoDestination,
    /// The save pipeline refused.
    Refused(mirae_project::SaveError),
}

impl SaveFailure {
    /// A stable code for the bridge.
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::NoDestination => "no_save_destination",
            Self::Refused(error) => match error {
                mirae_project::SaveError::ExternallyModified => "externally_modified",
                mirae_project::SaveError::Serialization(_) => "not_representable",
                mirae_project::SaveError::NoDestinationDirectory => "no_save_destination",
                mirae_project::SaveError::Filesystem(_) => "filesystem_refused",
                // Only reachable through a fault plan, which production never
                // builds with anything in it. Reported rather than ignored, so a
                // harness that injects one sees it arrive at the page.
                mirae_project::SaveError::Interrupted(_) => "save_interrupted",
            },
        }
    }
}

/// The current time as an RFC 3339 timestamp.
///
/// Built from the system clock without a date library: the format is fixed and
/// the arithmetic is calendar arithmetic, which is worth a dependency only when
/// something needs to parse or localize it. Nothing here does — the value is
/// written and read back as an opaque string.
fn timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());

    // Days since the epoch, converted with the civil-from-days algorithm. The
    // shifted era arithmetic is the standard formulation; it is written out
    // rather than pulled in because it is fifteen lines and has no edge cases
    // left to discover.
    let days = i64::try_from(seconds / 86_400).unwrap_or(0);
    let time_of_day = seconds % 86_400;

    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = if month <= 2 { year + 1 } else { year };

    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time_of_day / 3_600,
        (time_of_day % 3_600) / 60,
        time_of_day % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: &str = "0000000000000000000000000000002a";

    #[test]
    fn creating_a_session_produces_an_unsaved_project() {
        let session = ProjectSession::create("Stream", SESSION);

        let Ok(session) = session else {
            unreachable!("creation should have succeeded");
        };

        assert_eq!(session.name(), "Stream");
        assert!(session.path().is_none());
        assert!(session.save_state().is_dirty(), "never saved is dirty");
        assert_eq!(session.save_state().saved(), None);
    }

    #[test]
    fn an_unusable_name_is_refused_by_the_command_rather_than_the_shell() {
        // The shell does not validate names. `CreateProject` does, and the shell
        // reports what it said — otherwise there would be two rules.
        let session = ProjectSession::create("", SESSION);

        assert!(matches!(session, Err(CommandError::InvalidArgument)));
    }

    #[test]
    fn saving_without_a_destination_is_refused() {
        let Ok(mut session) = ProjectSession::create("Stream", SESSION) else {
            unreachable!("creation should have succeeded");
        };

        let failure = session.save(None);

        assert!(matches!(failure, Err(SaveFailure::NoDestination)));
        assert!(session.save_state().is_dirty());
    }

    #[test]
    fn saving_writes_the_file_and_makes_the_project_clean() {
        let directory =
            std::env::temp_dir().join(format!("mirae-shell-session-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&directory);
        let path = directory.join("project.mirae.json");

        let Ok(mut session) = ProjectSession::create("Stream", SESSION) else {
            unreachable!("creation should have succeeded");
        };

        let saved = session.save(Some(path.clone()));

        assert!(saved.is_ok());
        assert!(!session.save_state().is_dirty());
        assert_eq!(session.path(), Some(&path));
        assert!(path.is_file());

        // And saving again to the same place succeeds, because the session kept
        // the file identity the next save compares against.
        assert!(session.save(None).is_ok());

        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_failed_save_leaves_the_project_dirty() {
        let Ok(mut session) = ProjectSession::create("Stream", SESSION) else {
            unreachable!("creation should have succeeded");
        };

        let missing = std::env::temp_dir()
            .join("mirae-shell-absent-directory")
            .join("project.mirae.json");

        let failure = session.save(Some(missing));

        assert!(failure.is_err());
        assert!(
            session.save_state().is_dirty(),
            "403 invariant 7: a failure does not make the project look saved"
        );
        assert!(session.path().is_none());
    }

    #[test]
    fn the_timestamp_looks_like_an_rfc_3339_instant() {
        let now = timestamp();

        assert_eq!(now.len(), 20, "YYYY-MM-DDTHH:MM:SSZ");
        assert!(now.ends_with('Z'));
        assert!(now.contains('T'));
        assert!(now.starts_with("20"), "this century, at least");
    }

    #[test]
    fn every_save_failure_has_a_stable_lowercase_code() {
        for failure in [
            SaveFailure::NoDestination,
            SaveFailure::Refused(mirae_project::SaveError::ExternallyModified),
            SaveFailure::Refused(mirae_project::SaveError::NoDestinationDirectory),
        ] {
            assert!(!failure.as_str().is_empty());
            assert_eq!(failure.as_str(), failure.as_str().to_ascii_lowercase());
        }
    }
}

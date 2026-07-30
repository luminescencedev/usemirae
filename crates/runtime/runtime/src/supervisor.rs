//! Engine process supervision for the desktop shell.
//!
//! Canonical documentation: `docs/05-platform/501-desktop-shell.md` section 6,
//! `docs/01-runtime/101-process-model.md`, ADR-0037.
//!
//! The shell creates an ephemeral launch credential, launches the engine, waits
//! for readiness, and supports bounded restart. Two rules shape this module:
//!
//! - the shell never fabricates engine state while disconnected (section 6 and
//!   section 13), so the observed readiness is cleared the moment the engine goes
//!   away rather than being remembered;
//! - restart is bounded (section 6 point 6), so a crash loop gives up with a
//!   reason instead of restarting forever.
//!
//! Process spawning sits behind [`EngineLauncher`] so supervision is tested
//! without launching real processes.

use core::fmt;
use std::time::{Duration, Instant};

use mirae_contracts::generated::{EngineReadiness, EngineReadinessState};

/// Maximum length of a launch credential, in bytes.
pub const MAX_CREDENTIAL_BYTES: usize = 64;

/// An ephemeral secret proving to the engine that this shell launched it.
///
/// The value is never logged, never included in diagnostics, and its `Debug` is
/// redacted, so it cannot leak through a formatted error.
///
/// # Not yet cryptographic
///
/// `MIR-0010` defines the type and the handoff. Generating the value from a
/// cryptographic source, and verifying it during the handshake, belong to
/// `MIR-0012` together with the platform entropy adapter. [`Self::placeholder`]
/// is named for what it is so no caller mistakes it for a real secret.
#[derive(Clone, PartialEq, Eq)]
pub struct LaunchCredential {
    bytes: Vec<u8>,
}

impl LaunchCredential {
    /// Wrap credential bytes, rejecting an empty or oversized value.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() || bytes.len() > MAX_CREDENTIAL_BYTES {
            return None;
        }

        Some(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// A predictable stand-in until `MIR-0012` provides real entropy.
    ///
    /// Deliberately named `placeholder`: it is derived from the caller's seed, so
    /// it proves the handoff works and proves nothing about identity.
    #[must_use]
    pub fn placeholder(seed: u128) -> Self {
        Self {
            bytes: seed.to_be_bytes().to_vec(),
        }
    }

    /// The bytes, for the launcher that must hand them to the engine.
    #[must_use]
    pub fn expose(&self) -> &[u8] {
        &self.bytes
    }

    /// How many bytes the credential holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Whether the credential holds no bytes. Always false for a built value.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl fmt::Debug for LaunchCredential {
    /// Redacted: a credential must never reach a log through a formatted value.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "LaunchCredential({} bytes, redacted)",
            self.bytes.len()
        )
    }
}

/// What the supervisor knows about the engine right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisionState {
    /// No engine has been launched yet.
    Idle,
    /// Launched, waiting for a readiness report.
    Launching,
    /// The engine reported readiness.
    Connected,
    /// The engine exited or stopped reporting.
    Disconnected,
    /// A restart is pending within the budget.
    Restarting,
    /// The restart budget is exhausted; no further restart will be attempted.
    GaveUp,
    /// Shutdown was requested by the shell.
    Stopped,
}

impl SupervisionState {
    /// A stable identifier for diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Launching => "launching",
            Self::Connected => "connected",
            Self::Disconnected => "disconnected",
            Self::Restarting => "restarting",
            Self::GaveUp => "gave_up",
            Self::Stopped => "stopped",
        }
    }
}

impl fmt::Display for SupervisionState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// How often the engine may be restarted before the shell gives up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestartPolicy {
    /// Restarts allowed inside one window.
    pub max_restarts: u32,
    /// The window the budget applies to.
    pub window: Duration,
}

impl RestartPolicy {
    /// Three restarts per minute, then give up.
    pub const DEFAULT: Self = Self {
        max_restarts: 3,
        window: Duration::from_secs(60),
    };
}

/// Spawns and observes an engine process.
///
/// Implemented by the shell with a real process, and by tests with a fake, so
/// supervision logic is exercised without spawning anything.
pub trait EngineLauncher {
    /// Launch the engine with the credential. Returns whether it started.
    fn launch(&mut self, credential: &LaunchCredential) -> bool;

    /// Whether the launched process is still running.
    fn is_running(&mut self) -> bool;

    /// Take the readiness the engine reported, if it has reported one.
    fn take_readiness(&mut self) -> Option<EngineReadiness>;

    /// Ask the process to stop.
    fn stop(&mut self);
}

/// Supervises one engine process on behalf of the shell.
pub struct Supervisor<L: EngineLauncher> {
    launcher: L,
    policy: RestartPolicy,
    state: SupervisionState,
    /// Cleared whenever the engine goes away, so the shell cannot report stale
    /// state as current (`501` section 6).
    observed: Option<EngineReadiness>,
    restarts: Vec<Instant>,
    launches: u64,
    last_reason: Option<String>,
}

impl<L: EngineLauncher> Supervisor<L> {
    /// Build a supervisor.
    pub fn new(launcher: L, policy: RestartPolicy) -> Self {
        Self {
            launcher,
            policy,
            state: SupervisionState::Idle,
            observed: None,
            restarts: Vec::new(),
            launches: 0,
            last_reason: None,
        }
    }

    /// Access the launcher, for a caller that must speak the protocol directly.
    pub fn launcher_mut(&mut self) -> &mut L {
        &mut self.launcher
    }

    /// The supervision state.
    #[must_use]
    pub const fn state(&self) -> SupervisionState {
        self.state
    }

    /// The readiness the engine last reported, if it is still connected.
    ///
    /// `None` while disconnected: the shell reports what it knows, not what it
    /// last saw.
    #[must_use]
    pub const fn observed_readiness(&self) -> Option<&EngineReadiness> {
        self.observed.as_ref()
    }

    /// How many times an engine process has been launched.
    #[must_use]
    pub const fn launches(&self) -> u64 {
        self.launches
    }

    /// Why the supervisor is in its current state, when there is a reason.
    #[must_use]
    pub fn last_reason(&self) -> Option<&str> {
        self.last_reason.as_deref()
    }

    /// Launch the engine.
    pub fn start(&mut self, credential: &LaunchCredential, now: Instant) -> bool {
        if !self.launcher.launch(credential) {
            self.state = SupervisionState::Disconnected;
            self.last_reason = Some("the engine process could not be launched".to_owned());
            return false;
        }

        let _ = now;
        self.launches = self.launches.saturating_add(1);
        self.state = SupervisionState::Launching;
        self.observed = None;
        true
    }

    /// Poll the engine once.
    ///
    /// Returns the state after polling. A crash inside the restart budget schedules
    /// a restart; past the budget the supervisor gives up with a reason.
    pub fn poll(&mut self, credential: &LaunchCredential, now: Instant) -> SupervisionState {
        if matches!(
            self.state,
            SupervisionState::Stopped | SupervisionState::GaveUp | SupervisionState::Idle
        ) {
            return self.state;
        }

        if let Some(readiness) = self.launcher.take_readiness() {
            // A stopped engine is not something to report as usable state.
            if readiness.state == EngineReadinessState::Stopped {
                self.observed = None;
            } else {
                self.observed = Some(readiness);
                self.state = SupervisionState::Connected;
            }
        }

        if self.launcher.is_running() {
            return self.state;
        }

        // The process is gone: whatever it last said is no longer true.
        self.observed = None;
        self.state = SupervisionState::Disconnected;

        self.prune_restarts(now);

        if self.restarts.len() as u32 >= self.policy.max_restarts {
            self.state = SupervisionState::GaveUp;
            self.last_reason = Some(format!(
                "the engine exited {} times within {} seconds",
                self.restarts.len(),
                self.policy.window.as_secs()
            ));
            return self.state;
        }

        self.restarts.push(now);
        self.state = SupervisionState::Restarting;
        self.last_reason = Some("the engine exited; restarting".to_owned());

        if self.launcher.launch(credential) {
            self.launches = self.launches.saturating_add(1);
            self.state = SupervisionState::Launching;
        } else {
            self.state = SupervisionState::GaveUp;
            self.last_reason = Some("the engine could not be relaunched".to_owned());
        }

        self.state
    }

    /// Stop the engine and end supervision.
    pub fn stop(&mut self) {
        self.launcher.stop();
        self.observed = None;
        self.state = SupervisionState::Stopped;
    }

    /// Forget restarts that fell out of the window.
    fn prune_restarts(&mut self, now: Instant) {
        let window = self.policy.window;
        self.restarts
            .retain(|at| now.saturating_duration_since(*at) < window);
    }
}

/// Read the readiness contract out of one line the engine printed.
///
/// A deliberate stop-gap: the engine prints one JSON line because no wire format
/// exists yet. `MIR-ADR-0001` chooses the real serialization, and `MIR-0012`
/// replaces this with the authenticated handshake. Kept small and total: anything
/// unrecognised returns `None` rather than a partly filled contract.
#[must_use]
pub fn parse_readiness_line(line: &str) -> Option<EngineReadiness> {
    let state = match string_field(line, "state")?.as_str() {
        "starting" => EngineReadinessState::Starting,
        "ready" => EngineReadinessState::Ready,
        "degraded" => EngineReadinessState::Degraded,
        "stopping" => EngineReadinessState::Stopping,
        "stopped" => EngineReadinessState::Stopped,
        _ => return None,
    };

    Some(EngineReadiness {
        state,
        protocol_major: number_field(line, "protocolMajor")?,
        protocol_minor: number_field(line, "protocolMinor")?,
        engine_session_id: string_field(line, "engineSessionId")?,
        detail: string_field(line, "detail"),
    })
}

/// Read a quoted string field.
fn string_field(line: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let start = line.find(&needle)? + needle.len();
    let rest = line.get(start..)?;
    let end = rest.find('"')?;

    rest.get(..end).map(str::to_owned)
}

/// Read an unquoted numeric field.
fn number_field(line: &str, key: &str) -> Option<u16> {
    let needle = format!("\"{key}\":");
    let start = line.find(&needle)? + needle.len();
    let rest = line.get(start..)?;
    let end = rest
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(rest.len());

    rest.get(..end)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A launcher that never touches the operating system.
    #[derive(Debug, Default)]
    struct FakeLauncher {
        running: bool,
        pending_readiness: Option<EngineReadiness>,
        launches: u32,
        /// Launch attempts that should fail, counted down.
        refuse_launches: u32,
        stopped: bool,
        seen_credential: Option<Vec<u8>>,
    }

    impl FakeLauncher {
        fn ready(state: EngineReadinessState) -> EngineReadiness {
            EngineReadiness {
                state,
                protocol_major: 1,
                protocol_minor: 0,
                engine_session_id: "0000000000000000000000000000000a".to_owned(),
                detail: None,
            }
        }

        /// Simulate the engine exiting.
        fn crash(&mut self) {
            self.running = false;
        }
    }

    impl EngineLauncher for FakeLauncher {
        fn launch(&mut self, credential: &LaunchCredential) -> bool {
            self.seen_credential = Some(credential.expose().to_vec());

            if self.refuse_launches > 0 {
                self.refuse_launches -= 1;
                return false;
            }

            self.launches += 1;
            self.running = true;
            self.pending_readiness = Some(Self::ready(EngineReadinessState::Ready));
            true
        }

        fn is_running(&mut self) -> bool {
            self.running
        }

        fn take_readiness(&mut self) -> Option<EngineReadiness> {
            self.pending_readiness.take()
        }

        fn stop(&mut self) {
            self.running = false;
            self.stopped = true;
        }
    }

    fn credential() -> LaunchCredential {
        LaunchCredential::placeholder(0x1234)
    }

    #[test]
    fn a_credential_is_bounded_and_redacted_in_debug() {
        assert!(LaunchCredential::from_bytes(&[]).is_none());
        assert!(LaunchCredential::from_bytes(&[0_u8; MAX_CREDENTIAL_BYTES + 1]).is_none());

        let credential = credential();
        let debug = format!("{credential:?}");

        assert!(debug.contains("redacted"));
        assert!(!debug.contains("1234"));
        assert_eq!(credential.len(), 16);
        assert!(!credential.is_empty());
    }

    #[test]
    fn the_credential_reaches_the_launcher() {
        let mut supervisor = Supervisor::new(FakeLauncher::default(), RestartPolicy::DEFAULT);

        assert!(supervisor.start(&credential(), Instant::now()));
        assert_eq!(
            supervisor.launcher.seen_credential.as_deref(),
            Some(credential().expose())
        );
    }

    #[test]
    fn a_started_engine_becomes_connected_after_reporting() {
        let mut supervisor = Supervisor::new(FakeLauncher::default(), RestartPolicy::DEFAULT);
        let now = Instant::now();

        supervisor.start(&credential(), now);

        assert_eq!(supervisor.state(), SupervisionState::Launching);
        assert!(supervisor.observed_readiness().is_none());

        supervisor.poll(&credential(), now);

        assert_eq!(supervisor.state(), SupervisionState::Connected);
        assert_eq!(
            supervisor.observed_readiness().map(|ready| ready.state),
            Some(EngineReadinessState::Ready)
        );
    }

    #[test]
    fn a_crash_clears_the_observed_state_rather_than_remembering_it() {
        // 501 section 6: the shell must not fabricate engine state while
        // disconnected.
        let mut supervisor = Supervisor::new(FakeLauncher::default(), RestartPolicy::DEFAULT);
        let now = Instant::now();

        supervisor.start(&credential(), now);
        supervisor.poll(&credential(), now);
        assert!(supervisor.observed_readiness().is_some());

        supervisor.launcher.crash();
        supervisor.poll(&credential(), now);

        assert!(
            supervisor.observed_readiness().is_none(),
            "stale readiness must not survive a disconnect"
        );
    }

    #[test]
    fn a_crash_inside_the_budget_restarts_the_engine() {
        let mut supervisor = Supervisor::new(FakeLauncher::default(), RestartPolicy::DEFAULT);
        let now = Instant::now();

        supervisor.start(&credential(), now);
        supervisor.poll(&credential(), now);
        supervisor.launcher.crash();

        let state = supervisor.poll(&credential(), now);

        assert_eq!(state, SupervisionState::Launching);
        assert_eq!(supervisor.launches(), 2);
        assert!(
            supervisor
                .last_reason()
                .is_some_and(|reason| reason.contains("restarting"))
        );
    }

    #[test]
    fn restarts_are_bounded_and_the_supervisor_gives_up_with_a_reason() {
        let policy = RestartPolicy {
            max_restarts: 2,
            window: Duration::from_secs(60),
        };
        let mut supervisor = Supervisor::new(FakeLauncher::default(), policy);
        let now = Instant::now();

        supervisor.start(&credential(), now);

        for _ in 0..5 {
            supervisor.launcher.crash();
            supervisor.poll(&credential(), now);
        }

        assert_eq!(supervisor.state(), SupervisionState::GaveUp);
        assert!(
            supervisor
                .last_reason()
                .is_some_and(|reason| reason.contains("within 60 seconds"))
        );
        // The budget is 2, so at most the initial launch plus 2 restarts.
        assert!(supervisor.launches() <= 3);
    }

    #[test]
    fn the_budget_refills_once_the_window_passes() {
        let policy = RestartPolicy {
            max_restarts: 1,
            window: Duration::from_secs(60),
        };
        let mut supervisor = Supervisor::new(FakeLauncher::default(), policy);
        let start = Instant::now();

        supervisor.start(&credential(), start);
        supervisor.launcher.crash();
        supervisor.poll(&credential(), start);

        assert_eq!(supervisor.state(), SupervisionState::Launching);

        let later = start + Duration::from_secs(61);
        supervisor.launcher.crash();
        let state = supervisor.poll(&credential(), later);

        assert_eq!(
            state,
            SupervisionState::Launching,
            "an old crash should not count against a later one"
        );
    }

    #[test]
    fn a_launch_that_never_starts_is_reported() {
        let launcher = FakeLauncher {
            refuse_launches: 1,
            ..FakeLauncher::default()
        };
        let mut supervisor = Supervisor::new(launcher, RestartPolicy::DEFAULT);

        assert!(!supervisor.start(&credential(), Instant::now()));
        assert_eq!(supervisor.state(), SupervisionState::Disconnected);
        assert!(
            supervisor
                .last_reason()
                .is_some_and(|reason| reason.contains("could not be launched"))
        );
    }

    #[test]
    fn stopping_ends_supervision_and_clears_state() {
        let mut supervisor = Supervisor::new(FakeLauncher::default(), RestartPolicy::DEFAULT);
        let now = Instant::now();

        supervisor.start(&credential(), now);
        supervisor.poll(&credential(), now);
        supervisor.stop();

        assert_eq!(supervisor.state(), SupervisionState::Stopped);
        assert!(supervisor.observed_readiness().is_none());
        assert!(supervisor.launcher.stopped);

        // Polling after a stop does not resurrect anything.
        assert_eq!(
            supervisor.poll(&credential(), now),
            SupervisionState::Stopped
        );
    }

    #[test]
    fn a_stopped_engine_is_not_reported_as_usable_state() {
        let mut supervisor = Supervisor::new(FakeLauncher::default(), RestartPolicy::DEFAULT);
        let now = Instant::now();

        supervisor.start(&credential(), now);
        supervisor.launcher.pending_readiness =
            Some(FakeLauncher::ready(EngineReadinessState::Stopped));
        supervisor.poll(&credential(), now);

        assert!(supervisor.observed_readiness().is_none());
    }

    #[test]
    fn a_readiness_line_round_trips() {
        let line = "{\"state\":\"ready\",\"protocolMajor\":1,\"protocolMinor\":0,\
                    \"engineSessionId\":\"0000000000000000000000000000002a\"}";
        let readiness = parse_readiness_line(line);

        assert_eq!(
            readiness.as_ref().map(|ready| ready.state),
            Some(EngineReadinessState::Ready)
        );
        assert_eq!(
            readiness.as_ref().map(|ready| ready.protocol_major),
            Some(1)
        );
        assert_eq!(
            readiness.as_ref().and_then(|ready| ready.detail.clone()),
            None
        );
        assert_eq!(
            readiness.map(|ready| ready.engine_session_id),
            Some("0000000000000000000000000000002a".to_owned())
        );
    }

    #[test]
    fn a_readiness_line_carries_the_optional_detail() {
        let line = "{\"state\":\"degraded\",\"protocolMajor\":1,\"protocolMinor\":0,\
                    \"engineSessionId\":\"a\",\"detail\":\"no GPU adapter\"}";

        assert_eq!(
            parse_readiness_line(line).and_then(|ready| ready.detail),
            Some("no GPU adapter".to_owned())
        );
    }

    #[test]
    fn a_malformed_line_yields_nothing_rather_than_a_partial_contract() {
        for line in [
            "",
            "not json",
            "{\"state\":\"exploded\",\"protocolMajor\":1,\"protocolMinor\":0,\"engineSessionId\":\"a\"}",
            "{\"state\":\"ready\",\"protocolMinor\":0,\"engineSessionId\":\"a\"}",
            "{\"state\":\"ready\",\"protocolMajor\":1,\"protocolMinor\":0}",
        ] {
            assert!(
                parse_readiness_line(line).is_none(),
                "accepted malformed line `{line}`"
            );
        }
    }
}

//! Process and session identity shared by every Mirae process.
//!
//! Canonical documentation: `docs/06-quality/606-logging-and-tracing.md` sections
//! 2 and 7. Every process in one run stamps the same engine session id and build
//! id, so separate log files can be merged by tooling afterwards.

use core::fmt;
use std::time::Instant;

/// Which process emitted an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcessRole {
    /// The engine process.
    Engine,
    /// The native desktop shell that supervises the engine.
    Shell,
    /// The operator interface.
    ControlUi,
    /// The sandboxed extension host.
    ExtensionHost,
    /// A test harness standing in for a real process.
    Test,
}

impl ProcessRole {
    /// A stable identifier used in logs and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Engine => "engine",
            Self::Shell => "shell",
            Self::ControlUi => "control_ui",
            Self::ExtensionHost => "extension_host",
            Self::Test => "test",
        }
    }
}

impl fmt::Display for ProcessRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Identifies one engine run across every process that took part in it.
///
/// Generated once by the process that starts the engine and passed to the others,
/// so this crate only carries a value and never invents one: entropy belongs to
/// the platform layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EngineSessionId(u128);

impl EngineSessionId {
    /// The id used before a session exists, such as during early startup.
    pub const NONE: Self = Self(0);

    /// Wrap a raw value.
    #[must_use]
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }

    /// The raw value.
    #[must_use]
    pub const fn get(self) -> u128 {
        self.0
    }

    /// Whether a session has been established.
    #[must_use]
    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for EngineSessionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:032x}", self.0)
    }
}

/// The identity stamped on every event this process emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessIdentity {
    session: EngineSessionId,
    role: ProcessRole,
    /// Build identity, so merged logs from mismatched builds are detectable.
    build_id: &'static str,
}

impl ProcessIdentity {
    /// Build an identity.
    #[must_use]
    pub const fn new(session: EngineSessionId, role: ProcessRole, build_id: &'static str) -> Self {
        Self {
            session,
            role,
            build_id,
        }
    }

    /// The engine session id.
    #[must_use]
    pub const fn session(&self) -> EngineSessionId {
        self.session
    }

    /// The process role.
    #[must_use]
    pub const fn role(&self) -> ProcessRole {
        self.role
    }

    /// The build id.
    #[must_use]
    pub const fn build_id(&self) -> &'static str {
        self.build_id
    }

    /// Attach a session id learned after startup, such as from a handshake.
    #[must_use]
    pub const fn with_session(mut self, session: EngineSessionId) -> Self {
        self.session = session;
        self
    }
}

/// Maps monotonic time to the wall clock for one process.
///
/// Events carry both: the wall clock so a human can read them, and monotonic
/// nanoseconds since process start so ordering survives a clock adjustment
/// (`606` section 2). Tooling merges processes by session id and wall clock, then
/// orders within a process by the monotonic value.
#[derive(Debug, Clone, Copy)]
pub struct ClockOrigin {
    started_at_unix_millis: u64,
    started_at: Instant,
}

impl ClockOrigin {
    /// Capture the current wall clock and monotonic origin.
    #[must_use]
    pub fn now() -> Self {
        Self {
            started_at_unix_millis: unix_millis_now(),
            started_at: Instant::now(),
        }
    }

    /// Build an origin from explicit values, for tests and for replaying logs.
    #[must_use]
    pub const fn from_parts(started_at_unix_millis: u64, started_at: Instant) -> Self {
        Self {
            started_at_unix_millis,
            started_at,
        }
    }

    /// Wall-clock milliseconds for an instant, derived from the origin.
    #[must_use]
    pub fn unix_millis_at(&self, instant: Instant) -> u64 {
        let elapsed = instant
            .saturating_duration_since(self.started_at)
            .as_millis();

        // Saturating: a clock far in the future is a diagnostic problem, not a
        // reason to panic in a logging path.
        self.started_at_unix_millis
            .saturating_add(u64::try_from(elapsed).unwrap_or(u64::MAX))
    }

    /// Monotonic nanoseconds since this process started.
    #[must_use]
    pub fn monotonic_nanos_at(&self, instant: Instant) -> u64 {
        u64::try_from(
            instant
                .saturating_duration_since(self.started_at)
                .as_nanos(),
        )
        .unwrap_or(u64::MAX)
    }

    /// The wall clock at process start.
    #[must_use]
    pub const fn started_at_unix_millis(&self) -> u64 {
        self.started_at_unix_millis
    }
}

/// Milliseconds since the Unix epoch, or `0` when the clock is before it.
fn unix_millis_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn roles_have_distinct_stable_identifiers() {
        let roles = [
            ProcessRole::Engine,
            ProcessRole::Shell,
            ProcessRole::ControlUi,
            ProcessRole::ExtensionHost,
            ProcessRole::Test,
        ];
        let mut identifiers: Vec<&str> = roles.iter().map(|role| role.as_str()).collect();
        let count = identifiers.len();
        identifiers.sort_unstable();
        identifiers.dedup();

        assert_eq!(identifiers.len(), count);
        assert_eq!(ProcessRole::ControlUi.to_string(), "control_ui");
    }

    #[test]
    fn session_ids_round_trip_and_format_as_hex() {
        let session = EngineSessionId::from_u128(0xdead_beef);

        assert_eq!(session.get(), 0xdead_beef);
        assert_eq!(session.to_string(), "000000000000000000000000deadbeef");
        assert!(!session.is_none());
        assert!(EngineSessionId::NONE.is_none());
    }

    #[test]
    fn identity_can_learn_its_session_after_startup() {
        let identity = ProcessIdentity::new(EngineSessionId::NONE, ProcessRole::Shell, "test");

        assert!(identity.session().is_none());

        let joined = identity.with_session(EngineSessionId::from_u128(9));

        assert_eq!(joined.session().get(), 9);
        assert_eq!(joined.role(), ProcessRole::Shell);
        assert_eq!(joined.build_id(), "test");
    }

    #[test]
    fn monotonic_time_advances_from_the_origin() {
        let start = Instant::now();
        let origin = ClockOrigin::from_parts(1_000, start);
        let later = start + Duration::from_millis(250);

        assert_eq!(origin.monotonic_nanos_at(start), 0);
        assert_eq!(origin.monotonic_nanos_at(later), 250_000_000);
        assert_eq!(origin.unix_millis_at(later), 1_250);
        assert_eq!(origin.started_at_unix_millis(), 1_000);
    }

    #[test]
    fn an_instant_before_the_origin_does_not_underflow() {
        // Logging must not panic because a caller passed an earlier instant.
        let start = Instant::now();
        let origin = ClockOrigin::from_parts(1_000, start + Duration::from_secs(5));

        assert_eq!(origin.monotonic_nanos_at(start), 0);
        assert_eq!(origin.unix_millis_at(start), 1_000);
    }
}

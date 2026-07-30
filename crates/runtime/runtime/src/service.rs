//! Services, their initialization order, and their health outcomes.
//!
//! Canonical documentation: `docs/01-runtime/102-engine-lifecycle.md` sections 5,
//! 9, 10, and 11.
//!
//! A service reports health; it never transitions the engine itself (section 16).
//! An optional service that is unavailable degrades the engine rather than failing
//! it (invariant 5), and a mandatory one that fails ends startup.

use core::fmt;
use std::time::Duration;

use mirae_errors::MiraeError;

/// What a service reported after being asked to start.
#[derive(Debug)]
pub enum ServiceOutcome {
    /// Fully operational.
    Ready,
    /// Operational with an impaired capability. The reason is safe to log.
    Degraded(&'static str),
    /// Not operational. Optional services may report this without failing the
    /// engine; a mandatory one that does is a startup failure.
    Unavailable(&'static str),
    /// The service could not start and reported a structured error.
    Failed(Box<MiraeError>),
}

impl ServiceOutcome {
    /// Whether the service is usable at all.
    #[must_use]
    pub const fn is_usable(&self) -> bool {
        matches!(self, Self::Ready | Self::Degraded(_))
    }

    /// Whether the engine should be marked degraded because of this outcome.
    #[must_use]
    pub const fn degrades_engine(&self) -> bool {
        matches!(self, Self::Degraded(_) | Self::Unavailable(_))
    }

    /// A stable identifier for diagnostics.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Degraded(_) => "degraded",
            Self::Unavailable(_) => "unavailable",
            Self::Failed(_) => "failed",
        }
    }

    /// The safe reason, when there is one.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Ready => None,
            Self::Degraded(reason) | Self::Unavailable(reason) => Some(reason),
            Self::Failed(error) => Some(error.safe_message()),
        }
    }
}

impl fmt::Display for ServiceOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reason() {
            Some(reason) => write!(formatter, "{}: {reason}", self.as_str()),
            None => formatter.write_str(self.as_str()),
        }
    }
}

/// How much the engine depends on a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    /// The engine cannot serve without it.
    Mandatory,
    /// The engine serves in a degraded state without it.
    Optional,
}

/// One engine subsystem.
///
/// Implementations are constructed before initialization begins, so a constructor
/// must not do work that can fail; failure belongs in [`Service::initialize`],
/// where it can be reported as a structured outcome.
pub trait Service: Send {
    /// A stable name used in diagnostics and ordering.
    fn name(&self) -> &'static str;

    /// Whether the engine can serve without this service.
    fn requirement(&self) -> Requirement;

    /// Start the service.
    fn initialize(&mut self) -> ServiceOutcome;

    /// Stop the service.
    ///
    /// Called in reverse initialization order, and only for services that were
    /// initialized. Failure is recorded, never propagated: shutdown continues so
    /// the remaining services still get a chance to stop.
    fn shutdown(&mut self) -> Result<(), MiraeError> {
        Ok(())
    }

    /// How long this service may take to stop before it is abandoned.
    ///
    /// `102` section 11: no stage may wait forever.
    fn shutdown_deadline(&self) -> Duration {
        Duration::from_secs(5)
    }
}

/// What happened to one service during startup.
#[derive(Debug)]
pub struct ServiceReport {
    /// The service name.
    pub name: &'static str,
    /// Whether it was mandatory.
    pub requirement: Requirement,
    /// What it reported.
    pub outcome: ServiceOutcome,
}

/// What happened to one service during shutdown.
#[derive(Debug)]
pub struct ShutdownReport {
    /// The service name.
    pub name: &'static str,
    /// Whether stopping succeeded.
    pub stopped: bool,
    /// How long stopping took.
    pub elapsed: Duration,
    /// Whether it exceeded its deadline.
    pub overran_deadline: bool,
    /// The safe reason it failed, when it did.
    pub failure: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirae_errors::{ErrorCategory, ErrorCode, SubsystemId};

    const FALLBACK: ErrorCode = match ErrorCode::new("RUNTIME_SERVICE_FAILED") {
        Some(code) => code,
        None => panic!("the literal above is a valid error code"),
    };

    fn error() -> MiraeError {
        MiraeError::new(
            FALLBACK,
            ErrorCategory::PersistentInfrastructure,
            SubsystemId::Runtime,
            "the service could not start",
        )
    }

    #[test]
    fn ready_and_degraded_services_are_usable() {
        assert!(ServiceOutcome::Ready.is_usable());
        assert!(ServiceOutcome::Degraded("fallback adapter").is_usable());
        assert!(!ServiceOutcome::Unavailable("no device").is_usable());
        assert!(!ServiceOutcome::Failed(Box::new(error())).is_usable());
    }

    #[test]
    fn anything_short_of_ready_degrades_the_engine() {
        assert!(!ServiceOutcome::Ready.degrades_engine());
        assert!(ServiceOutcome::Degraded("fallback").degrades_engine());
        assert!(ServiceOutcome::Unavailable("absent").degrades_engine());
    }

    #[test]
    fn outcomes_carry_a_safe_reason() {
        assert_eq!(ServiceOutcome::Ready.reason(), None);
        assert_eq!(
            ServiceOutcome::Degraded("renderer fallback active").reason(),
            Some("renderer fallback active")
        );
        assert_eq!(
            ServiceOutcome::Failed(Box::new(error())).reason(),
            Some("the service could not start")
        );
        assert_eq!(
            ServiceOutcome::Unavailable("no adapter").to_string(),
            "unavailable: no adapter"
        );
    }
}

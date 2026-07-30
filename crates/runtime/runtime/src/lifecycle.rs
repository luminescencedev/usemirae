//! The engine lifecycle state machine.
//!
//! Canonical documentation: `docs/01-runtime/102-engine-lifecycle.md` sections 2,
//! 3, and 6, and ADR-0005.
//!
//! One serialized authority owns the state (section 16). Services report health
//! and request transitions; they never set the state themselves. Every accepted
//! transition is observable, and every rejected one is an error rather than a
//! silent no-op (invariants 6 and 7).

use core::fmt;

use mirae_contracts::generated::EngineReadinessState;

/// Where the engine is in its life (`102` section 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EngineLifecycleState {
    /// Constructed, nothing started.
    Created,
    /// Minimal process-safe setup: clock, session, crash context, logging.
    Bootstrapping,
    /// Services are initializing in dependency order.
    Initializing,
    /// Connections may be accepted; no project is necessarily active.
    Ready,
    /// A project is being activated transactionally.
    ActivatingProject,
    /// A project is active and the scheduler runs.
    Running,
    /// Core operation continues with an impaired capability.
    Degraded,
    /// New work that would prolong shutdown is refused.
    Draining,
    /// Services are stopping in reverse order.
    Stopping,
    /// Terminal for this engine session.
    Stopped,
    /// Startup or activation failed; only shutdown follows.
    Failed,
}

impl EngineLifecycleState {
    /// A stable identifier for diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Bootstrapping => "bootstrapping",
            Self::Initializing => "initializing",
            Self::Ready => "ready",
            Self::ActivatingProject => "activating_project",
            Self::Running => "running",
            Self::Degraded => "degraded",
            Self::Draining => "draining",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        }
    }

    /// Whether the engine accepts protocol connections in this state.
    ///
    /// `102` section 6: connections may be accepted from `Ready` onward, and a
    /// degraded engine still serves.
    #[must_use]
    pub const fn accepts_connections(self) -> bool {
        matches!(
            self,
            Self::Ready | Self::ActivatingProject | Self::Running | Self::Degraded
        )
    }

    /// Whether this state is terminal for the session (invariant 3).
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Stopped)
    }

    /// The readiness a peer should be told about.
    ///
    /// Maps the internal machine onto the generated `mirae://ipc/v1/engine-readiness`
    /// contract, so the wire vocabulary stays smaller than the internal one.
    #[must_use]
    pub const fn readiness(self) -> EngineReadinessState {
        match self {
            Self::Created | Self::Bootstrapping | Self::Initializing => {
                EngineReadinessState::Starting
            }
            Self::Ready | Self::ActivatingProject | Self::Running => EngineReadinessState::Ready,
            Self::Degraded => EngineReadinessState::Degraded,
            Self::Draining | Self::Stopping => EngineReadinessState::Stopping,
            // A failed engine is not serving, and only shutdown follows, so peers
            // are told it is stopped rather than degraded.
            Self::Stopped | Self::Failed => EngineReadinessState::Stopped,
        }
    }

    /// Whether `next` may follow this state (`102` section 3).
    #[must_use]
    pub const fn may_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Created, Self::Bootstrapping)
                | (Self::Bootstrapping, Self::Initializing | Self::Failed)
                | (Self::Initializing, Self::Ready | Self::Failed)
                | (Self::Ready, Self::ActivatingProject | Self::Draining)
                | (Self::ActivatingProject, Self::Running | Self::Failed)
                | (Self::Running, Self::Degraded | Self::Draining)
                | (Self::Degraded, Self::Running | Self::Draining)
                | (Self::Draining, Self::Stopping)
                | (Self::Failed, Self::Stopping)
                | (Self::Stopping, Self::Stopped)
        )
    }
}

impl fmt::Display for EngineLifecycleState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A transition that was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTransition {
    /// The state the engine was in.
    pub from: EngineLifecycleState,
    /// The state that was requested.
    pub to: EngineLifecycleState,
}

impl fmt::Display for InvalidTransition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "`{}` may not transition to `{}`",
            self.from, self.to
        )
    }
}

impl std::error::Error for InvalidTransition {}

/// Maximum transitions retained for crash context (`102` section 13).
pub const MAX_RETAINED_TRANSITIONS: usize = 16;

/// The single authority over lifecycle state.
#[derive(Debug)]
pub struct LifecycleCoordinator {
    state: EngineLifecycleState,
    /// Recent transitions, oldest first, bounded for crash context.
    history: Vec<EngineLifecycleState>,
    transitions: u64,
    rejected: u64,
}

impl LifecycleCoordinator {
    /// A coordinator in [`EngineLifecycleState::Created`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: EngineLifecycleState::Created,
            history: vec![EngineLifecycleState::Created],
            transitions: 0,
            rejected: 0,
        }
    }

    /// The current state.
    #[must_use]
    pub const fn state(&self) -> EngineLifecycleState {
        self.state
    }

    /// Recent states, oldest first.
    #[must_use]
    pub fn history(&self) -> &[EngineLifecycleState] {
        &self.history
    }

    /// How many transitions were accepted.
    #[must_use]
    pub const fn transitions(&self) -> u64 {
        self.transitions
    }

    /// How many transitions were refused.
    #[must_use]
    pub const fn rejected(&self) -> u64 {
        self.rejected
    }

    /// Request a transition.
    ///
    /// Returns the previous state on success. An invalid request is refused and
    /// counted: a caller that guessed wrong learns rather than corrupting the
    /// machine.
    pub fn transition_to(
        &mut self,
        next: EngineLifecycleState,
    ) -> Result<EngineLifecycleState, InvalidTransition> {
        if !self.state.may_transition_to(next) {
            self.rejected = self.rejected.saturating_add(1);
            return Err(InvalidTransition {
                from: self.state,
                to: next,
            });
        }

        let previous = self.state;
        self.state = next;
        self.transitions = self.transitions.saturating_add(1);

        if self.history.len() >= MAX_RETAINED_TRANSITIONS {
            self.history.remove(0);
        }
        self.history.push(next);

        Ok(previous)
    }
}

impl Default for LifecycleCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every state, so adding one forces the tests to be revisited.
    const ALL: [EngineLifecycleState; 11] = [
        EngineLifecycleState::Created,
        EngineLifecycleState::Bootstrapping,
        EngineLifecycleState::Initializing,
        EngineLifecycleState::Ready,
        EngineLifecycleState::ActivatingProject,
        EngineLifecycleState::Running,
        EngineLifecycleState::Degraded,
        EngineLifecycleState::Draining,
        EngineLifecycleState::Stopping,
        EngineLifecycleState::Stopped,
        EngineLifecycleState::Failed,
    ];

    #[test]
    fn the_documented_happy_path_is_permitted() {
        let mut coordinator = LifecycleCoordinator::new();

        for next in [
            EngineLifecycleState::Bootstrapping,
            EngineLifecycleState::Initializing,
            EngineLifecycleState::Ready,
            EngineLifecycleState::ActivatingProject,
            EngineLifecycleState::Running,
            EngineLifecycleState::Draining,
            EngineLifecycleState::Stopping,
            EngineLifecycleState::Stopped,
        ] {
            assert!(
                coordinator.transition_to(next).is_ok(),
                "the documented path rejected `{next}`"
            );
        }

        assert!(coordinator.state().is_terminal());
        assert_eq!(coordinator.transitions(), 8);
    }

    #[test]
    fn degraded_and_running_recover_in_both_directions() {
        // 102 section 3: Running -> Degraded -> Running.
        let mut coordinator = LifecycleCoordinator::new();
        for next in [
            EngineLifecycleState::Bootstrapping,
            EngineLifecycleState::Initializing,
            EngineLifecycleState::Ready,
            EngineLifecycleState::ActivatingProject,
            EngineLifecycleState::Running,
        ] {
            let _ = coordinator.transition_to(next);
        }

        assert!(
            coordinator
                .transition_to(EngineLifecycleState::Degraded)
                .is_ok()
        );
        assert!(
            coordinator
                .transition_to(EngineLifecycleState::Running)
                .is_ok()
        );
    }

    #[test]
    fn an_illegal_transition_is_refused_and_counted() {
        let mut coordinator = LifecycleCoordinator::new();

        // Created may not jump straight to Ready.
        let error = coordinator.transition_to(EngineLifecycleState::Ready);

        assert_eq!(
            error,
            Err(InvalidTransition {
                from: EngineLifecycleState::Created,
                to: EngineLifecycleState::Ready,
            })
        );
        assert_eq!(coordinator.state(), EngineLifecycleState::Created);
        assert_eq!(coordinator.rejected(), 1);
        assert!(
            error
                .err()
                .is_some_and(|error| error.to_string().contains("may not transition"))
        );
    }

    #[test]
    fn stopped_is_terminal() {
        let mut coordinator = LifecycleCoordinator::new();
        for next in [
            EngineLifecycleState::Bootstrapping,
            EngineLifecycleState::Failed,
            EngineLifecycleState::Stopping,
            EngineLifecycleState::Stopped,
        ] {
            let _ = coordinator.transition_to(next);
        }

        for next in ALL {
            assert!(
                coordinator.transition_to(next).is_err(),
                "`stopped` accepted `{next}`"
            );
        }
    }

    #[test]
    fn failure_may_only_be_followed_by_shutdown() {
        for next in ALL {
            let permitted = EngineLifecycleState::Failed.may_transition_to(next);

            assert_eq!(
                permitted,
                next == EngineLifecycleState::Stopping,
                "`failed` -> `{next}` should be {}",
                next == EngineLifecycleState::Stopping
            );
        }
    }

    #[test]
    fn connections_are_accepted_only_from_ready_onward() {
        for state in ALL {
            let expected = matches!(
                state,
                EngineLifecycleState::Ready
                    | EngineLifecycleState::ActivatingProject
                    | EngineLifecycleState::Running
                    | EngineLifecycleState::Degraded
            );

            assert_eq!(state.accepts_connections(), expected, "state `{state}`");
        }
    }

    #[test]
    fn every_state_maps_onto_the_readiness_contract() {
        assert_eq!(
            EngineLifecycleState::Bootstrapping.readiness(),
            EngineReadinessState::Starting
        );
        assert_eq!(
            EngineLifecycleState::Ready.readiness(),
            EngineReadinessState::Ready
        );
        assert_eq!(
            EngineLifecycleState::Running.readiness(),
            EngineReadinessState::Ready
        );
        assert_eq!(
            EngineLifecycleState::Degraded.readiness(),
            EngineReadinessState::Degraded
        );
        assert_eq!(
            EngineLifecycleState::Draining.readiness(),
            EngineReadinessState::Stopping
        );
        // A failed engine is not serving; peers must not read that as degraded.
        assert_eq!(
            EngineLifecycleState::Failed.readiness(),
            EngineReadinessState::Stopped
        );
    }

    #[test]
    fn history_is_bounded() {
        let mut coordinator = LifecycleCoordinator::new();

        // Bounce between Running and Degraded well past the retention bound.
        for next in [
            EngineLifecycleState::Bootstrapping,
            EngineLifecycleState::Initializing,
            EngineLifecycleState::Ready,
            EngineLifecycleState::ActivatingProject,
            EngineLifecycleState::Running,
        ] {
            let _ = coordinator.transition_to(next);
        }

        for _ in 0..MAX_RETAINED_TRANSITIONS * 2 {
            let _ = coordinator.transition_to(EngineLifecycleState::Degraded);
            let _ = coordinator.transition_to(EngineLifecycleState::Running);
        }

        assert!(coordinator.history().len() <= MAX_RETAINED_TRANSITIONS);
        assert!(coordinator.transitions() > MAX_RETAINED_TRANSITIONS as u64);
    }

    #[test]
    fn state_identifiers_are_unique() {
        let mut identifiers: Vec<&str> = ALL.iter().map(|state| state.as_str()).collect();
        let count = identifiers.len();
        identifiers.sort_unstable();
        identifiers.dedup();

        assert_eq!(identifiers.len(), count);
    }
}

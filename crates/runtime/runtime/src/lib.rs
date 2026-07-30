//! Engine lifecycle, service orchestration, and process coordination.
//!
//! Canonical documentation: `docs/01-runtime/102-engine-lifecycle.md`,
//! `docs/01-runtime/101-process-model.md`, ADR-0005, ADR-0010.
//! Dependency rules: `docs/08-development/804-dependency-rules.md`.
//!
//! # Scope today
//!
//! `MIR-0009` covers launch, service initialization, readiness, and clean
//! shutdown. Project activation, the scheduler, and the IPC server are named in
//! the lifecycle but owned by later tickets; their states exist here so the
//! machine is complete and the gaps stay visible rather than invented.
//!
//! # Structure
//!
//! [`LifecycleCoordinator`] is the single serialized authority over state.
//! [`Engine`] drives it: it bootstraps, initializes services in registration
//! order, publishes readiness through the generated
//! `mirae://ipc/v1/engine-readiness` contract, then drains, stops, and reports.
//! Services report health and never transition the engine themselves
//! (`102` section 16).

pub mod ipc;
pub mod lifecycle;
pub mod service;
pub mod supervisor;

pub use lifecycle::{
    EngineLifecycleState, InvalidTransition, LifecycleCoordinator, MAX_RETAINED_TRANSITIONS,
};
pub use service::{Requirement, Service, ServiceOutcome, ServiceReport, ShutdownReport};
pub use supervisor::{
    EngineLauncher, LaunchCredential, RestartPolicy, SupervisionState, Supervisor,
    parse_readiness_line,
};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use mirae_contracts::generated::{
    EngineReadiness, EngineReadinessState, PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR,
};
use mirae_errors::{ErrorCategory, ErrorCode, MiraeError, SubsystemId};
use mirae_observability::{EngineSessionId, Field, FieldValue, Level, RedactionClass, Tracer};

/// Maximum services one engine may register.
///
/// The lifecycle names thirteen (`102` section 5); the bound leaves room without
/// letting a defect register without limit.
pub const MAX_SERVICES: usize = 32;

/// Maximum diagnostics retained for crash context (`102` section 13).
pub const MAX_CRASH_DIAGNOSTICS: usize = 16;

/// Cooperative cancellation for long-running work (`102` sections 4 and 12).
///
/// The root token is created during bootstrapping and handed to services. It is
/// observed, never enforced: a service that ignores it is caught by its shutdown
/// deadline instead.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// A token that has not been cancelled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Request cancellation. Idempotent.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Whether cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// Minimal state preserved for crash reporting (`102` section 13).
///
/// Excludes secrets and raw media by construction: it holds only identifiers,
/// versions, lifecycle states, and short safe labels.
#[derive(Debug, Clone)]
pub struct CrashContext {
    /// The engine session.
    pub session: EngineSessionId,
    /// The lifecycle state at capture time.
    pub state: EngineLifecycleState,
    /// Recent lifecycle states, oldest first.
    pub recent_states: Vec<EngineLifecycleState>,
    /// Protocol major version this build speaks.
    pub protocol_major: u16,
    /// Protocol minor version this build speaks.
    pub protocol_minor: u16,
    /// The build identity.
    pub build_id: &'static str,
    /// Recent safe diagnostics, oldest first and bounded.
    pub diagnostics: Vec<String>,
}

/// Why an engine could not start.
///
/// The error is boxed: startup succeeds far more often than it fails, and an
/// unboxed `MiraeError` would make every `Result` from [`Engine::start`] carry its
/// full size.
#[derive(Debug)]
pub struct StartupFailure {
    /// The service that failed, when a service was responsible.
    pub service: Option<&'static str>,
    /// The structured error.
    pub error: Box<MiraeError>,
}

/// Drives the lifecycle and owns the services.
pub struct Engine {
    coordinator: LifecycleCoordinator,
    services: Vec<Box<dyn Service>>,
    /// Indices of services that initialized, in the order they did.
    started: Vec<usize>,
    tracer: Tracer,
    cancellation: CancellationToken,
    session: EngineSessionId,
    degraded_reasons: Vec<String>,
    diagnostics: Vec<String>,
}

impl Engine {
    /// Build an engine that reports as `session`.
    ///
    /// A new process creates a new session (`102` invariant 8); this type accepts
    /// one rather than generating it, because entropy belongs to the platform.
    #[must_use]
    pub fn new(tracer: Tracer, session: EngineSessionId) -> Self {
        Self {
            coordinator: LifecycleCoordinator::new(),
            services: Vec::new(),
            started: Vec::new(),
            tracer,
            cancellation: CancellationToken::new(),
            session,
            degraded_reasons: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Register a service. Initialization order is registration order.
    ///
    /// `102` section 5 fixes the dependency order; the caller registers in that
    /// order, and shutdown reverses it.
    pub fn register(&mut self, service: Box<dyn Service>) -> bool {
        if self.services.len() >= MAX_SERVICES {
            self.record_diagnostic(&format!(
                "service `{}` was not registered: the registry is full",
                service.name()
            ));
            return false;
        }

        self.services.push(service);
        true
    }

    /// The current lifecycle state.
    #[must_use]
    pub const fn state(&self) -> EngineLifecycleState {
        self.coordinator.state()
    }

    /// The root cancellation token.
    #[must_use]
    pub fn cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    /// Recent safe diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    /// Whether an optional service left the engine impaired.
    #[must_use]
    pub fn is_impaired(&self) -> bool {
        !self.degraded_reasons.is_empty()
    }

    /// The safe reasons the engine is impaired.
    #[must_use]
    pub fn degraded_reasons(&self) -> &[String] {
        &self.degraded_reasons
    }

    /// The readiness to publish to peers.
    ///
    /// Built from the generated contract, so the engine and any client agree on
    /// the vocabulary by construction.
    /// The detail is populated whenever a capability is impaired, not only when
    /// the lifecycle itself is `Degraded`. The documented state graph only allows
    /// `Degraded` from `Running`, so a `Ready` engine missing an optional service
    /// would otherwise report a bare `ready` and hide the impairment.
    #[must_use]
    pub fn readiness(&self) -> EngineReadiness {
        let state = self.coordinator.state().readiness();
        let detail = match state {
            EngineReadinessState::Stopped | EngineReadinessState::Stopping => None,
            _ => self.degraded_reasons.first().cloned(),
        };

        EngineReadiness {
            state,
            protocol_major: PROTOCOL_VERSION_MAJOR,
            protocol_minor: PROTOCOL_VERSION_MINOR,
            engine_session_id: self.session.to_string(),
            detail,
        }
    }

    /// Capture crash context (`102` section 13).
    #[must_use]
    pub fn crash_context(&self) -> CrashContext {
        CrashContext {
            session: self.session,
            state: self.coordinator.state(),
            recent_states: self.coordinator.history().to_vec(),
            protocol_major: PROTOCOL_VERSION_MAJOR,
            protocol_minor: PROTOCOL_VERSION_MINOR,
            build_id: self.tracer.identity().build_id(),
            diagnostics: self.diagnostics.clone(),
        }
    }

    /// Bootstrap and initialize every registered service.
    ///
    /// On success the engine is [`EngineLifecycleState::Ready`]; an impaired
    /// optional service is reported through [`Engine::is_impaired`] and the
    /// readiness detail. A mandatory failure leaves the engine in
    /// [`EngineLifecycleState::Failed`]; the caller still calls
    /// [`Engine::shutdown`], which stops whatever did start.
    pub fn start(&mut self) -> Result<Vec<ServiceReport>, StartupFailure> {
        self.force_transition(EngineLifecycleState::Bootstrapping);
        // Bootstrapping is deliberately minimal: identity, clock, cancellation,
        // and logging already exist by construction (102 section 4).
        self.record_diagnostic("bootstrapped");

        self.force_transition(EngineLifecycleState::Initializing);

        let mut reports = Vec::with_capacity(self.services.len());

        for index in 0..self.services.len() {
            let Some(service) = self.services.get_mut(index) else {
                continue;
            };

            let name = service.name();
            let requirement = service.requirement();
            let outcome = service.initialize();

            self.tracer
                .event(Level::Info, SubsystemId::Runtime, "engine.service_started")
                .field(Field::internal("service", FieldValue::Label(name)))
                .field(Field::public(
                    "outcome",
                    FieldValue::Label(outcome.as_str()),
                ))
                .emit();

            if outcome.is_usable() {
                self.started.push(index);
            }

            if let Some(reason) = outcome.reason() {
                let line = format!("{name}: {reason}");
                self.record_diagnostic(&line);
            }

            if requirement == Requirement::Mandatory && !outcome.is_usable() {
                let error = match outcome {
                    ServiceOutcome::Failed(error) => error,
                    ref other => Box::new(runtime_error(
                        "ENGINE_MANDATORY_SERVICE_UNAVAILABLE",
                        &format!(
                            "the mandatory service `{name}` is unavailable: {}",
                            other.reason().unwrap_or("no reason given")
                        ),
                    )),
                };

                self.force_transition(EngineLifecycleState::Failed);
                self.tracer
                    .event(Level::Error, SubsystemId::Runtime, "engine.start_failed")
                    .field(Field::internal("service", FieldValue::Label(name)))
                    .field(Field::text(
                        "reason",
                        RedactionClass::Internal,
                        error.safe_message(),
                    ))
                    .emit();

                return Err(StartupFailure {
                    service: Some(name),
                    error,
                });
            }

            if outcome.degrades_engine()
                && let Some(reason) = outcome.reason()
            {
                self.degraded_reasons.push(format!("{name}: {reason}"));
            }

            reports.push(ServiceReport {
                name,
                requirement,
                outcome,
            });
        }

        self.force_transition(EngineLifecycleState::Ready);
        self.tracer
            .event(Level::Info, SubsystemId::Runtime, "engine.ready")
            .field(Field::public(
                "services",
                FieldValue::Unsigned(self.started.len() as u64),
            ))
            .field(Field::public(
                "degraded",
                FieldValue::Unsigned(self.degraded_reasons.len() as u64),
            ))
            .emit();

        Ok(reports)
    }

    /// Drain and stop, in reverse initialization order.
    ///
    /// Every started service is asked to stop even if an earlier one failed or
    /// overran, so one bad service cannot strand the rest. Deadlines are recorded
    /// rather than enforced by preemption: this crate has no executor that could
    /// cancel a blocking call, so an overrun is reported and shutdown continues,
    /// which is the forced cleanup `102` section 11 describes.
    pub fn shutdown(&mut self) -> Vec<ShutdownReport> {
        self.cancellation.cancel();

        if self.coordinator.state() != EngineLifecycleState::Failed {
            self.force_transition(EngineLifecycleState::Draining);
        }
        self.force_transition(EngineLifecycleState::Stopping);

        let order: Vec<usize> = self.started.iter().rev().copied().collect();
        let mut reports = Vec::with_capacity(order.len());

        for index in order {
            let Some(service) = self.services.get_mut(index) else {
                continue;
            };

            let name = service.name();
            let deadline = service.shutdown_deadline();
            let started_at = Instant::now();
            let result = service.shutdown();
            let elapsed = started_at.elapsed();
            let overran = elapsed > deadline;
            let failure = result.err().map(|error| error.safe_message().to_owned());

            if overran {
                self.record_diagnostic(&format!("service `{name}` exceeded its shutdown deadline"));
            }
            if let Some(reason) = failure.clone() {
                self.record_diagnostic(&format!("service `{name}` failed to stop: {reason}"));
            }

            self.tracer
                .event(Level::Info, SubsystemId::Runtime, "engine.service_stopped")
                .field(Field::internal("service", FieldValue::Label(name)))
                .field(Field::public(
                    "stopped",
                    FieldValue::Flag(failure.is_none()),
                ))
                .field(Field::public("overran_deadline", FieldValue::Flag(overran)))
                .emit();

            reports.push(ShutdownReport {
                name,
                stopped: failure.is_none(),
                elapsed,
                overran_deadline: overran,
                failure,
            });
        }

        self.started.clear();
        self.force_transition(EngineLifecycleState::Stopped);
        self.tracer
            .event(Level::Info, SubsystemId::Runtime, "engine.stopped")
            .field(Field::public(
                "services_stopped",
                FieldValue::Unsigned(reports.len() as u64),
            ))
            .emit();

        reports
    }

    /// Record a safe diagnostic, keeping the buffer bounded.
    fn record_diagnostic(&mut self, message: &str) {
        if self.diagnostics.len() >= MAX_CRASH_DIAGNOSTICS {
            self.diagnostics.remove(0);
        }

        self.diagnostics.push(message.to_owned());
    }

    /// Apply a transition, recording a diagnostic if the machine refuses it.
    ///
    /// The engine drives the documented path, so a refusal means the engine has a
    /// defect. It is recorded rather than panicked: a lifecycle bug must not take
    /// the process down before shutdown has run.
    fn force_transition(&mut self, next: EngineLifecycleState) {
        match self.coordinator.transition_to(next) {
            Ok(previous) => {
                self.tracer
                    .event(
                        Level::Info,
                        SubsystemId::Runtime,
                        "engine.lifecycle_transition",
                    )
                    .field(Field::public("from", FieldValue::Label(previous.as_str())))
                    .field(Field::public("to", FieldValue::Label(next.as_str())))
                    .emit();
            }
            Err(invalid) => {
                self.record_diagnostic(&format!("refused transition: {invalid}"));
                self.tracer
                    .event(
                        Level::Error,
                        SubsystemId::Runtime,
                        "engine.lifecycle_transition_refused",
                    )
                    .field(Field::public(
                        "from",
                        FieldValue::Label(invalid.from.as_str()),
                    ))
                    .field(Field::public("to", FieldValue::Label(invalid.to.as_str())))
                    .emit();
            }
        }
    }
}

/// Build a runtime error, falling back to a valid code if a literal is malformed.
fn runtime_error(code: &'static str, message: &str) -> MiraeError {
    const FALLBACK: ErrorCode = match ErrorCode::new("ENGINE_INTERNAL") {
        Some(code) => code,
        None => panic!("the literal above is a valid error code"),
    };

    MiraeError::new(
        ErrorCode::new(code).unwrap_or(FALLBACK),
        ErrorCategory::PersistentInfrastructure,
        SubsystemId::Runtime,
        message,
    )
}

/// A service that reports a fixed outcome.
///
/// Used by tests, and by the engine binary until the real subsystems from
/// `102` section 5 exist.
#[derive(Debug)]
pub struct StubService {
    name: &'static str,
    requirement: Requirement,
    outcome: Option<ServiceOutcome>,
    shutdown_error: Option<&'static str>,
    shutdown_delay: Duration,
    deadline: Duration,
}

impl StubService {
    /// A service that starts and stops cleanly.
    #[must_use]
    pub fn ready(name: &'static str, requirement: Requirement) -> Self {
        Self {
            name,
            requirement,
            outcome: Some(ServiceOutcome::Ready),
            shutdown_error: None,
            shutdown_delay: Duration::ZERO,
            deadline: Duration::from_secs(5),
        }
    }

    /// A service that reports the given outcome when it starts.
    #[must_use]
    pub fn with_outcome(
        name: &'static str,
        requirement: Requirement,
        outcome: ServiceOutcome,
    ) -> Self {
        Self {
            name,
            requirement,
            outcome: Some(outcome),
            shutdown_error: None,
            shutdown_delay: Duration::ZERO,
            deadline: Duration::from_secs(5),
        }
    }

    /// Make this service fail to stop.
    #[must_use]
    pub fn failing_shutdown(mut self, reason: &'static str) -> Self {
        self.shutdown_error = Some(reason);
        self
    }

    /// Make this service overrun its shutdown deadline.
    #[must_use]
    pub fn slow_shutdown(mut self, delay: Duration, deadline: Duration) -> Self {
        self.shutdown_delay = delay;
        self.deadline = deadline;
        self
    }
}

impl Service for StubService {
    fn name(&self) -> &'static str {
        self.name
    }

    fn requirement(&self) -> Requirement {
        self.requirement
    }

    fn initialize(&mut self) -> ServiceOutcome {
        self.outcome.take().unwrap_or(ServiceOutcome::Ready)
    }

    fn shutdown(&mut self) -> Result<(), MiraeError> {
        if !self.shutdown_delay.is_zero() {
            std::thread::sleep(self.shutdown_delay);
        }

        match self.shutdown_error {
            Some(reason) => Err(runtime_error("ENGINE_SERVICE_STOP_FAILED", reason)),
            None => Ok(()),
        }
    }

    fn shutdown_deadline(&self) -> Duration {
        self.deadline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirae_observability::{
        ClockOrigin, MemorySink, ProcessIdentity, ProcessRole, VolumeControl,
    };

    fn engine() -> Engine {
        let tracer = Tracer::new(
            ProcessIdentity::new(
                EngineSessionId::from_u128(7),
                ProcessRole::Engine,
                "test-build",
            ),
            ClockOrigin::from_parts(0, Instant::now()),
            Level::Debug,
            Box::new(MemorySink::new(256)),
            VolumeControl::new(64, Duration::from_secs(60)),
        );

        Engine::new(tracer, EngineSessionId::from_u128(7))
    }

    #[test]
    fn a_complete_startup_reaches_ready() {
        let mut engine = engine();
        engine.register(Box::new(StubService::ready(
            "config",
            Requirement::Mandatory,
        )));
        engine.register(Box::new(StubService::ready("ipc", Requirement::Mandatory)));

        let reports = engine.start();

        assert_eq!(reports.map(|reports| reports.len()).ok(), Some(2));
        assert_eq!(engine.state(), EngineLifecycleState::Ready);
        assert!(engine.state().accepts_connections());
        assert!(!engine.is_impaired());
    }

    #[test]
    fn an_unavailable_optional_service_impairs_but_does_not_fail() {
        // 102 invariant 5: optional capability failure does not fail the engine.
        let mut engine = engine();
        engine.register(Box::new(StubService::ready(
            "config",
            Requirement::Mandatory,
        )));
        engine.register(Box::new(StubService::with_outcome(
            "extension_host",
            Requirement::Optional,
            ServiceOutcome::Unavailable("the extension host is not installed"),
        )));

        assert!(engine.start().is_ok());
        assert_eq!(engine.state(), EngineLifecycleState::Ready);
        assert!(engine.is_impaired());
        assert!(
            engine.degraded_reasons()[0].contains("extension_host"),
            "the reason should name the service"
        );

        // The documented graph only allows Degraded from Running, so a Ready
        // engine reports `ready` with the impairment in the detail rather than
        // hiding it.
        let readiness = engine.readiness();

        assert_eq!(readiness.state, EngineReadinessState::Ready);
        assert!(
            readiness
                .detail
                .as_deref()
                .is_some_and(|detail| detail.contains("extension_host")),
            "an impaired engine must say so in its readiness detail"
        );
    }

    #[test]
    fn a_mandatory_failure_fails_startup_and_names_the_service() {
        let mut engine = engine();
        engine.register(Box::new(StubService::ready(
            "config",
            Requirement::Mandatory,
        )));
        engine.register(Box::new(StubService::with_outcome(
            "renderer",
            Requirement::Mandatory,
            ServiceOutcome::Unavailable("no compatible GPU adapter"),
        )));
        engine.register(Box::new(StubService::ready("late", Requirement::Mandatory)));

        let failure = engine.start().err();

        assert_eq!(engine.state(), EngineLifecycleState::Failed);
        assert_eq!(
            failure.as_ref().and_then(|failure| failure.service),
            Some("renderer")
        );
        assert!(failure.is_some_and(|failure| failure.error.safe_message().contains("renderer")));
    }

    #[test]
    fn services_stop_in_reverse_order() {
        let mut engine = engine();
        for name in ["first", "second", "third"] {
            engine.register(Box::new(StubService::ready(name, Requirement::Mandatory)));
        }
        let _ = engine.start();

        let reports = engine.shutdown();
        let order: Vec<&str> = reports.iter().map(|report| report.name).collect();

        assert_eq!(order, vec!["third", "second", "first"]);
        assert_eq!(engine.state(), EngineLifecycleState::Stopped);
        assert!(engine.state().is_terminal());
    }

    #[test]
    fn shutdown_continues_after_a_service_fails_to_stop() {
        let mut engine = engine();
        engine.register(Box::new(StubService::ready(
            "first",
            Requirement::Mandatory,
        )));
        engine.register(Box::new(
            StubService::ready("second", Requirement::Mandatory)
                .failing_shutdown("the worker did not respond"),
        ));
        let _ = engine.start();

        let reports = engine.shutdown();

        assert_eq!(reports.len(), 2, "every started service must be asked");
        assert!(!reports[0].stopped, "`second` failed to stop");
        assert!(reports[1].stopped, "`first` still got its chance");
        assert_eq!(engine.state(), EngineLifecycleState::Stopped);
    }

    #[test]
    fn a_shutdown_overrun_is_recorded_and_does_not_block_the_rest() {
        let mut engine = engine();
        engine.register(Box::new(StubService::ready(
            "first",
            Requirement::Mandatory,
        )));
        engine.register(Box::new(
            StubService::ready("slow", Requirement::Mandatory)
                .slow_shutdown(Duration::from_millis(30), Duration::from_millis(1)),
        ));
        let _ = engine.start();

        let reports = engine.shutdown();

        assert!(reports[0].overran_deadline, "`slow` should overrun");
        assert!(!reports[1].overran_deadline);
        assert!(
            engine
                .diagnostics()
                .iter()
                .any(|line| line.contains("exceeded its shutdown deadline"))
        );
        assert_eq!(engine.state(), EngineLifecycleState::Stopped);
    }

    #[test]
    fn a_failed_engine_still_stops_what_started() {
        let mut engine = engine();
        engine.register(Box::new(StubService::ready(
            "config",
            Requirement::Mandatory,
        )));
        engine.register(Box::new(StubService::with_outcome(
            "renderer",
            Requirement::Mandatory,
            ServiceOutcome::Unavailable("no adapter"),
        )));
        let _ = engine.start();

        let reports = engine.shutdown();

        assert_eq!(reports.len(), 1, "only `config` had started");
        assert_eq!(reports[0].name, "config");
        assert_eq!(engine.state(), EngineLifecycleState::Stopped);
    }

    #[test]
    fn shutdown_cancels_the_root_token() {
        let mut engine = engine();
        let token = engine.cancellation();

        assert!(!token.is_cancelled());

        let _ = engine.start();
        engine.shutdown();

        assert!(token.is_cancelled(), "services observe cancellation");
    }

    #[test]
    fn readiness_uses_the_generated_contract() {
        let mut engine = engine();
        engine.register(Box::new(StubService::ready(
            "config",
            Requirement::Mandatory,
        )));

        assert_eq!(
            engine.readiness().state,
            EngineReadinessState::Starting,
            "before start the engine is starting"
        );

        let _ = engine.start();
        let ready = engine.readiness();

        assert_eq!(ready.state, EngineReadinessState::Ready);
        assert_eq!(ready.protocol_major, PROTOCOL_VERSION_MAJOR);
        assert_eq!(ready.protocol_minor, PROTOCOL_VERSION_MINOR);
        assert_eq!(ready.engine_session_id, "00000000000000000000000000000007");
        assert_eq!(ready.detail, None);

        engine.shutdown();

        assert_eq!(engine.readiness().state, EngineReadinessState::Stopped);
    }

    #[test]
    fn crash_context_carries_identity_without_secrets() {
        let mut engine = engine();
        engine.register(Box::new(StubService::with_outcome(
            "credentials",
            Requirement::Optional,
            ServiceOutcome::Degraded("the secure store is locked"),
        )));
        let _ = engine.start();

        let context = engine.crash_context();

        assert_eq!(context.session.get(), 7);
        assert_eq!(context.state, EngineLifecycleState::Ready);
        assert_eq!(context.protocol_major, PROTOCOL_VERSION_MAJOR);
        assert_eq!(context.build_id, "test-build");
        assert!(context.recent_states.contains(&EngineLifecycleState::Ready));
        // 102 invariant 9: crash context excludes credentials. Only safe reasons
        // reach it; the reason above says the store is locked, not what is in it.
        assert!(
            context
                .diagnostics
                .iter()
                .all(|line| !line.contains("password") && !line.contains("secret_value"))
        );
    }

    #[test]
    fn diagnostics_stay_bounded() {
        let mut engine = engine();

        for _ in 0..(MAX_CRASH_DIAGNOSTICS * 3) {
            engine.record_diagnostic("something happened");
        }

        assert_eq!(engine.diagnostics().len(), MAX_CRASH_DIAGNOSTICS);
    }

    #[test]
    fn the_service_registry_is_bounded() {
        let mut engine = engine();

        for _ in 0..MAX_SERVICES {
            assert!(engine.register(Box::new(StubService::ready(
                "service",
                Requirement::Optional
            ))));
        }

        assert!(
            !engine.register(Box::new(StubService::ready(
                "one-too-many",
                Requirement::Optional
            ))),
            "the registry must refuse past its bound"
        );
    }
}

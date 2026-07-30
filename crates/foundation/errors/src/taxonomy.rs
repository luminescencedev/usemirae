//! Error classification: category, severity, retryability, subsystem, and the
//! action a user can take.
//!
//! Canonical documentation: `docs/06-quality/605-error-model.md` sections 2, 5, 6,
//! and 8, and ADR-0040.

use core::fmt;

/// What kind of failure occurred, from `605` section 2.
///
/// The category drives default severity and retryability, so classifying an error
/// once gives consistent recovery behavior across subsystems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorCategory {
    /// Input did not match its schema or could not be decoded.
    InputSchema,
    /// Input was well formed but violates a domain rule.
    DomainValidation,
    /// The operation lost a race or targets state that moved on.
    Conflict,
    /// The caller is not permitted, by the OS or by Mirae policy.
    Permission,
    /// An external resource is currently unavailable.
    ExternalResourceUnavailable,
    /// The platform or device cannot do what was asked.
    CapabilityCompatibility,
    /// Infrastructure failed in a way that is expected to pass later.
    TransientInfrastructure,
    /// Infrastructure failed in a way that will not pass without a change.
    PersistentInfrastructure,
    /// Stored data is unreadable or internally inconsistent.
    DataCorruption,
    /// An internal invariant was violated. This is a defect, not a user error.
    InternalInvariant,
    /// The operation was cancelled, by a user or by a supervising component.
    Cancellation,
    /// The operation exceeded its deadline.
    Timeout,
}

impl ErrorCategory {
    /// The severity to use unless a subsystem has a reason to differ.
    ///
    /// Severity reflects user and system impact, not developer embarrassment
    /// (`605` section 5).
    #[must_use]
    pub const fn default_severity(self) -> ErrorSeverity {
        match self {
            Self::Cancellation => ErrorSeverity::Info,
            Self::Conflict | Self::CapabilityCompatibility | Self::DomainValidation => {
                ErrorSeverity::Warning
            }
            Self::InputSchema
            | Self::Permission
            | Self::ExternalResourceUnavailable
            | Self::TransientInfrastructure
            | Self::PersistentInfrastructure
            | Self::Timeout => ErrorSeverity::Error,
            Self::DataCorruption | Self::InternalInvariant => ErrorSeverity::Critical,
        }
    }

    /// The retry classification to use unless the owning subsystem knows better.
    ///
    /// Retry policy belongs to the owning subsystem, not a generic retry loop
    /// (`605` section 6); this is the starting classification, not a schedule.
    #[must_use]
    pub const fn default_retryability(self) -> Retryability {
        match self {
            Self::InputSchema
            | Self::DomainValidation
            | Self::DataCorruption
            | Self::InternalInvariant
            | Self::Cancellation => Retryability::NotRetryable,
            Self::Conflict => Retryability::RetryImmediatelyOnce,
            Self::ExternalResourceUnavailable | Self::TransientInfrastructure | Self::Timeout => {
                Retryability::RetryWithBackoff
            }
            Self::Permission => Retryability::RetryAfterUserAction,
            Self::CapabilityCompatibility | Self::PersistentInfrastructure => {
                Retryability::RetryAfterEnvironmentChange
            }
        }
    }

    /// Whether this category counts as a failure for reporting.
    ///
    /// Cancellation is not logged as a failure by default (`605` invariant 8).
    #[must_use]
    pub const fn is_failure(self) -> bool {
        !matches!(self, Self::Cancellation)
    }

    /// A stable identifier for logs and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputSchema => "input_schema",
            Self::DomainValidation => "domain_validation",
            Self::Conflict => "conflict",
            Self::Permission => "permission",
            Self::ExternalResourceUnavailable => "external_resource_unavailable",
            Self::CapabilityCompatibility => "capability_compatibility",
            Self::TransientInfrastructure => "transient_infrastructure",
            Self::PersistentInfrastructure => "persistent_infrastructure",
            Self::DataCorruption => "data_corruption",
            Self::InternalInvariant => "internal_invariant",
            Self::Cancellation => "cancellation",
            Self::Timeout => "timeout",
        }
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// How much the failure affects the user or the system (`605` section 5).
///
/// Ordered, so a supervisor can compare and escalate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorSeverity {
    /// Expected and not a problem, such as a cancelled operation.
    Info,
    /// The user should know, but the operation or production continues.
    Warning,
    /// The operation failed. A recoverable source failure is `Error` without
    /// being engine-fatal.
    Error,
    /// Production or data integrity is at risk.
    Critical,
}

impl ErrorSeverity {
    /// A stable identifier for logs and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Whether and how an operation may be retried (`605` section 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Retryability {
    /// Retrying will fail the same way.
    NotRetryable,
    /// One immediate retry is worthwhile, such as after losing a race.
    RetryImmediatelyOnce,
    /// Retry on the owning subsystem's backoff schedule.
    RetryWithBackoff,
    /// Retry only after the user does something, such as granting permission.
    RetryAfterUserAction,
    /// Retry only after the environment changes, such as a driver update.
    RetryAfterEnvironmentChange,
    /// Classification is not known. Treated as not automatically retryable.
    Unknown,
}

impl Retryability {
    /// Whether a supervisor may retry without asking anyone.
    ///
    /// `Unknown` is deliberately false: an unclassified failure must not become a
    /// silent retry loop.
    #[must_use]
    pub const fn allows_automatic_retry(self) -> bool {
        matches!(self, Self::RetryImmediatelyOnce | Self::RetryWithBackoff)
    }

    /// Whether progress depends on someone or something outside Mirae.
    #[must_use]
    pub const fn needs_external_change(self) -> bool {
        matches!(
            self,
            Self::RetryAfterUserAction | Self::RetryAfterEnvironmentChange
        )
    }

    /// A stable identifier for logs and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotRetryable => "not_retryable",
            Self::RetryImmediatelyOnce => "retry_immediately_once",
            Self::RetryWithBackoff => "retry_with_backoff",
            Self::RetryAfterUserAction => "retry_after_user_action",
            Self::RetryAfterEnvironmentChange => "retry_after_environment_change",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for Retryability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Which part of Mirae reported the error.
///
/// Used for routing, metrics, and diagnostics. The list follows the crate groups
/// in `docs/08-development/802-rust-workspace-and-crates.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SubsystemId {
    /// Foundation types, errors, and generated contracts.
    Foundation,
    /// Engine lifecycle and service orchestration.
    Runtime,
    /// Project domain, commands, and state.
    Project,
    /// Scene graph, frame compilation, and rendering.
    Rendering,
    /// Capture, decode, and source runtime.
    Media,
    /// Audio graph and routing.
    Audio,
    /// Encoding, streaming, and recording outputs.
    Output,
    /// Platform adapters and capabilities.
    Platform,
    /// Cross-process protocol.
    Ipc,
    /// Extension host and public SDK.
    Sdk,
    /// Diagnostics, logging, and telemetry.
    Diagnostics,
    /// Operator interface.
    Ui,
}

impl SubsystemId {
    /// A stable identifier for logs and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Foundation => "foundation",
            Self::Runtime => "runtime",
            Self::Project => "project",
            Self::Rendering => "rendering",
            Self::Media => "media",
            Self::Audio => "audio",
            Self::Output => "output",
            Self::Platform => "platform",
            Self::Ipc => "ipc",
            Self::Sdk => "sdk",
            Self::Diagnostics => "diagnostics",
            Self::Ui => "ui",
        }
    }
}

impl fmt::Display for SubsystemId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// What the user can do about the failure (`605` section 8).
///
/// A hint, not a sentence: the UI owns the wording and its localization, so this
/// carries no user-visible text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UserActionHint {
    /// Nothing to do; Mirae will retry on its own.
    WaitForAutomaticRetry,
    /// Try the operation again.
    RetryManually,
    /// Grant a permission the operating system is withholding.
    GrantPermission,
    /// Reconnect or re-select a device.
    ReconnectDevice,
    /// Free disk space.
    FreeDiskSpace,
    /// Correct a setting or credential.
    CheckConfiguration,
    /// Update Mirae, a driver, or the operating system.
    UpdateSoftware,
    /// Report the problem with the diagnostic reference.
    ContactSupport,
}

impl UserActionHint {
    /// A stable identifier the UI maps to localized guidance.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WaitForAutomaticRetry => "wait_for_automatic_retry",
            Self::RetryManually => "retry_manually",
            Self::GrantPermission => "grant_permission",
            Self::ReconnectDevice => "reconnect_device",
            Self::FreeDiskSpace => "free_disk_space",
            Self::CheckConfiguration => "check_configuration",
            Self::UpdateSoftware => "update_software",
            Self::ContactSupport => "contact_support",
        }
    }
}

impl fmt::Display for UserActionHint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Correlates an error with the request, command, or frame that caused it.
///
/// The width matches the IPC frame header in
/// `docs/01-runtime/108-ipc-protocol.md` section 4, so a correlation id survives a
/// process hop unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CorrelationId(u128);

impl CorrelationId {
    /// The id used when no correlation is available.
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

    /// Whether this id actually correlates anything.
    #[must_use]
    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Hexadecimal keeps the id compact and greppable across processes.
        write!(formatter, "{:032x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every category, so a new one cannot be added without revisiting the tests.
    const ALL_CATEGORIES: [ErrorCategory; 12] = [
        ErrorCategory::InputSchema,
        ErrorCategory::DomainValidation,
        ErrorCategory::Conflict,
        ErrorCategory::Permission,
        ErrorCategory::ExternalResourceUnavailable,
        ErrorCategory::CapabilityCompatibility,
        ErrorCategory::TransientInfrastructure,
        ErrorCategory::PersistentInfrastructure,
        ErrorCategory::DataCorruption,
        ErrorCategory::InternalInvariant,
        ErrorCategory::Cancellation,
        ErrorCategory::Timeout,
    ];

    #[test]
    fn every_category_has_distinct_stable_identifiers() {
        let mut identifiers: Vec<&str> = ALL_CATEGORIES
            .iter()
            .map(|category| category.as_str())
            .collect();
        let count = identifiers.len();
        identifiers.sort_unstable();
        identifiers.dedup();

        assert_eq!(
            identifiers.len(),
            count,
            "category identifiers must be unique"
        );
    }

    #[test]
    fn severity_orders_by_impact() {
        assert!(ErrorSeverity::Info < ErrorSeverity::Warning);
        assert!(ErrorSeverity::Warning < ErrorSeverity::Error);
        assert!(ErrorSeverity::Error < ErrorSeverity::Critical);
    }

    #[test]
    fn defects_and_corruption_are_critical() {
        assert_eq!(
            ErrorCategory::InternalInvariant.default_severity(),
            ErrorSeverity::Critical
        );
        assert_eq!(
            ErrorCategory::DataCorruption.default_severity(),
            ErrorSeverity::Critical
        );
    }

    #[test]
    fn cancellation_is_informational_and_not_a_failure() {
        // 605 invariant 8: cancellation is not logged as a failure by default.
        assert_eq!(
            ErrorCategory::Cancellation.default_severity(),
            ErrorSeverity::Info
        );
        assert!(!ErrorCategory::Cancellation.is_failure());
        assert!(
            ALL_CATEGORIES
                .iter()
                .filter(|category| !category.is_failure())
                .count()
                == 1
        );
    }

    #[test]
    fn transient_conditions_retry_and_permanent_ones_do_not() {
        assert!(
            ErrorCategory::TransientInfrastructure
                .default_retryability()
                .allows_automatic_retry()
        );
        assert!(
            ErrorCategory::Timeout
                .default_retryability()
                .allows_automatic_retry()
        );
        assert!(
            !ErrorCategory::DomainValidation
                .default_retryability()
                .allows_automatic_retry()
        );
        assert!(
            !ErrorCategory::DataCorruption
                .default_retryability()
                .allows_automatic_retry()
        );
    }

    #[test]
    fn a_defect_is_never_automatically_retried() {
        // Retrying an internal invariant violation would just repeat the defect.
        assert_eq!(
            ErrorCategory::InternalInvariant.default_retryability(),
            Retryability::NotRetryable
        );
    }

    #[test]
    fn unknown_retryability_never_starts_a_retry_loop() {
        assert!(!Retryability::Unknown.allows_automatic_retry());
        assert!(!Retryability::Unknown.needs_external_change());
    }

    #[test]
    fn permission_and_capability_need_someone_else_to_act() {
        assert!(
            ErrorCategory::Permission
                .default_retryability()
                .needs_external_change()
        );
        assert!(
            ErrorCategory::CapabilityCompatibility
                .default_retryability()
                .needs_external_change()
        );
    }

    #[test]
    fn every_category_has_a_defined_default_pair() {
        for category in ALL_CATEGORIES {
            // Exercises both const matches for every variant, so adding a variant
            // without handling it fails to compile rather than at runtime.
            let severity = category.default_severity();
            let retryability = category.default_retryability();

            assert!(!severity.as_str().is_empty());
            assert!(!retryability.as_str().is_empty());
        }
    }

    #[test]
    fn correlation_ids_round_trip_and_format_as_hex() {
        let id = CorrelationId::from_u128(0x0123_4567_89ab_cdef);

        assert_eq!(id.get(), 0x0123_4567_89ab_cdef);
        assert_eq!(id.to_string(), "00000000000000000123456789abcdef");
        assert!(!id.is_none());
        assert!(CorrelationId::NONE.is_none());
    }

    #[test]
    fn subsystem_and_hint_identifiers_are_stable_and_unique() {
        let subsystems = [
            SubsystemId::Foundation,
            SubsystemId::Runtime,
            SubsystemId::Project,
            SubsystemId::Rendering,
            SubsystemId::Media,
            SubsystemId::Audio,
            SubsystemId::Output,
            SubsystemId::Platform,
            SubsystemId::Ipc,
            SubsystemId::Sdk,
            SubsystemId::Diagnostics,
            SubsystemId::Ui,
        ];
        let mut identifiers: Vec<&str> = subsystems
            .iter()
            .map(|subsystem| subsystem.as_str())
            .collect();
        let count = identifiers.len();
        identifiers.sort_unstable();
        identifiers.dedup();

        assert_eq!(identifiers.len(), count);
        assert_eq!(SubsystemId::Ipc.to_string(), "ipc");
        assert_eq!(
            UserActionHint::GrantPermission.to_string(),
            "grant_permission"
        );
    }
}

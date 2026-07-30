//! Structured error taxonomy and safe error context.
//!
//! Canonical documentation: `docs/06-quality/605-error-model.md` and ADR-0040.
//! Dependency rules: `docs/08-development/804-dependency-rules.md`.
//!
//! This crate is the foundation layer: it depends on `std` only, so every other
//! crate can report errors without pulling in a framework.
//!
//! # What this crate provides
//!
//! [`MiraeError`] carries a stable [`ErrorCode`], an [`ErrorCategory`], an
//! [`ErrorSeverity`], the reporting [`SubsystemId`], a [`Retryability`]
//! classification, an optional [`UserActionHint`], a [`CorrelationId`], a safe
//! message, bounded [`ErrorContext`], and an optional source error.
//!
//! # What it does not do
//!
//! It defines no concrete error codes: each subsystem owns its own and keeps them
//! stable. It performs no retries; retry policy belongs to the owning subsystem
//! (`605` section 6). Its redaction removes absolute paths, not credentials, so a
//! caller must still keep secrets out of a message.
//!
//! policy-allow: local-path - tests prove that an absolute path in a message or a
//! source error never reaches the safe message or a diagnostic line

mod code;
mod context;
pub mod redaction;
mod taxonomy;

pub use code::ErrorCode;
pub use context::{ContextValue, ErrorContext, MAX_CONTEXT_ENTRIES, MAX_TEXT_VALUE_CHARACTERS};
pub use taxonomy::{
    CorrelationId, ErrorCategory, ErrorSeverity, Retryability, SubsystemId, UserActionHint,
};

use core::fmt;
use std::error::Error;

/// Maximum length of a safe message, in characters.
///
/// Long enough for one clear sentence, short enough that an error cannot become a
/// transport for a payload.
pub const MAX_SAFE_MESSAGE_CHARACTERS: usize = 240;

/// A structured, user-safe error.
///
/// Built through [`MiraeError::new`], which applies the category defaults and makes
/// the message safe. Every `with_*` method overrides one field, so a subsystem
/// states only what differs from its category.
///
/// # Examples
///
/// ```
/// use mirae_errors::{CorrelationId, ErrorCategory, ErrorCode, MiraeError, SubsystemId};
///
/// let code = ErrorCode::new("IPC_PROTOCOL_MISMATCH").expect("valid code");
/// let error = MiraeError::new(
///     code,
///     ErrorCategory::CapabilityCompatibility,
///     SubsystemId::Ipc,
///     "the engine speaks a different protocol major version",
/// )
/// .with_correlation_id(CorrelationId::from_u128(7));
///
/// assert_eq!(error.code().as_str(), "IPC_PROTOCOL_MISMATCH");
/// assert!(!error.retryability().allows_automatic_retry());
/// ```
#[derive(Debug)]
pub struct MiraeError {
    code: ErrorCode,
    category: ErrorCategory,
    severity: ErrorSeverity,
    subsystem: SubsystemId,
    retryability: Retryability,
    user_action: Option<UserActionHint>,
    correlation_id: CorrelationId,
    safe_message: String,
    context: ErrorContext,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl MiraeError {
    /// Build an error, taking severity and retryability from the category.
    ///
    /// The message is redacted, whitespace-normalized, and truncated, so it is safe
    /// to show a user and safe to log.
    #[must_use]
    pub fn new(
        code: ErrorCode,
        category: ErrorCategory,
        subsystem: SubsystemId,
        safe_message: &str,
    ) -> Self {
        Self {
            code,
            category,
            severity: category.default_severity(),
            subsystem,
            retryability: category.default_retryability(),
            user_action: None,
            correlation_id: CorrelationId::NONE,
            safe_message: sanitize_message(safe_message),
            context: ErrorContext::new(),
            source: None,
        }
    }

    /// Override the severity when impact differs from the category default.
    #[must_use]
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Override the retry classification.
    #[must_use]
    pub fn with_retryability(mut self, retryability: Retryability) -> Self {
        self.retryability = retryability;
        self
    }

    /// State what the user can do about it.
    #[must_use]
    pub fn with_user_action(mut self, hint: UserActionHint) -> Self {
        self.user_action = Some(hint);
        self
    }

    /// Attach the correlation id of the request, command, or frame.
    #[must_use]
    pub fn with_correlation_id(mut self, correlation_id: CorrelationId) -> Self {
        self.correlation_id = correlation_id;
        self
    }

    /// Replace the whole context.
    #[must_use]
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }

    /// Add one context value at a boundary.
    #[must_use]
    pub fn with_context_value(mut self, key: &'static str, value: ContextValue) -> Self {
        self.context.insert(key, value);
        self
    }

    /// Preserve the underlying cause (`605` invariant 6).
    ///
    /// The source is kept for diagnostics and never formatted into the safe
    /// message: internal source text is not automatically user-safe.
    #[must_use]
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    /// The stable code.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// The category.
    #[must_use]
    pub const fn category(&self) -> ErrorCategory {
        self.category
    }

    /// The severity.
    #[must_use]
    pub const fn severity(&self) -> ErrorSeverity {
        self.severity
    }

    /// The reporting subsystem.
    #[must_use]
    pub const fn subsystem(&self) -> SubsystemId {
        self.subsystem
    }

    /// The retry classification.
    #[must_use]
    pub const fn retryability(&self) -> Retryability {
        self.retryability
    }

    /// The suggested user action, if any.
    #[must_use]
    pub const fn user_action(&self) -> Option<UserActionHint> {
        self.user_action
    }

    /// The correlation id.
    #[must_use]
    pub const fn correlation_id(&self) -> CorrelationId {
        self.correlation_id
    }

    /// The user-safe message.
    #[must_use]
    pub fn safe_message(&self) -> &str {
        &self.safe_message
    }

    /// The diagnostic context.
    #[must_use]
    pub const fn context(&self) -> &ErrorContext {
        &self.context
    }

    /// Whether this counts as a failure for reporting.
    ///
    /// Cancellation does not (`605` invariant 8).
    #[must_use]
    pub const fn is_failure(&self) -> bool {
        self.category.is_failure()
    }

    /// The deepest error in the source chain, or this error when it has no source.
    ///
    /// Used for aggregation: several related failures report one root cause rather
    /// than one notification per layer (`605` section 10).
    #[must_use]
    pub fn root_cause(&self) -> &(dyn Error + 'static) {
        let mut current: &(dyn Error + 'static) = self;

        while let Some(source) = current.source() {
            current = source;
        }

        current
    }

    /// How deep the source chain runs, counting this error as one.
    #[must_use]
    pub fn chain_length(&self) -> usize {
        let mut length = 1;
        let mut current: &(dyn Error + 'static) = self;

        while let Some(source) = current.source() {
            current = source;
            length += 1;
        }

        length
    }

    /// A one-line diagnostic summary.
    ///
    /// Contains the code, classification, correlation id, safe message, and
    /// context. It is safe to log: it never includes the source chain, because
    /// source text may carry vendor detail that has not been redacted.
    #[must_use]
    pub fn diagnostic_line(&self) -> String {
        let mut line = format!(
            "{code} severity={severity} category={category} subsystem={subsystem} \
             retryability={retryability} correlation={correlation}",
            code = self.code,
            severity = self.severity,
            category = self.category,
            subsystem = self.subsystem,
            retryability = self.retryability,
            correlation = self.correlation_id,
        );

        if let Some(hint) = self.user_action {
            line.push_str(&format!(" user_action={hint}"));
        }

        line.push_str(&format!(" message=\"{}\"", self.safe_message));

        if !self.context.is_empty() || self.context.dropped() != 0 {
            line.push_str(&format!(" context[{}]", self.context));
        }

        line
    }
}

/// Make caller-supplied text safe to show and to log.
fn sanitize_message(message: &str) -> String {
    redaction::truncate(
        &redaction::normalize_whitespace(&redaction::redact_paths(message)),
        MAX_SAFE_MESSAGE_CHARACTERS,
    )
}

impl fmt::Display for MiraeError {
    /// Shows only the safe message, so formatting an error can never leak.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.safe_message)
    }
}

impl Error for MiraeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FALLBACK: ErrorCode = match ErrorCode::new("FALLBACK") {
        Some(code) => code,
        None => panic!("the literal above is a valid error code"),
    };

    fn code(value: &'static str) -> ErrorCode {
        ErrorCode::new(value).unwrap_or(FALLBACK)
    }

    fn sample() -> MiraeError {
        MiraeError::new(
            code("CAPTURE_PERMISSION_DENIED"),
            ErrorCategory::Permission,
            SubsystemId::Media,
            "screen capture permission was denied",
        )
    }

    #[test]
    fn takes_severity_and_retryability_from_the_category() {
        let error = sample();

        assert_eq!(error.severity(), ErrorSeverity::Error);
        assert_eq!(error.retryability(), Retryability::RetryAfterUserAction);
        assert!(!error.retryability().allows_automatic_retry());
        assert!(error.retryability().needs_external_change());
    }

    #[test]
    fn overrides_apply_only_to_what_was_set() {
        let error = sample()
            .with_severity(ErrorSeverity::Critical)
            .with_user_action(UserActionHint::GrantPermission);

        assert_eq!(error.severity(), ErrorSeverity::Critical);
        assert_eq!(error.category(), ErrorCategory::Permission);
        assert_eq!(error.user_action(), Some(UserActionHint::GrantPermission));
        assert_eq!(error.retryability(), Retryability::RetryAfterUserAction);
    }

    #[test]
    fn redacts_and_bounds_the_safe_message() {
        let error = MiraeError::new(
            code("PROJECT_OPEN_FAILED"),
            ErrorCategory::PersistentInfrastructure,
            SubsystemId::Project,
            "could not open C:\\Users\\arthur\\projects\\live.mirae",
        );

        assert_eq!(error.safe_message(), "could not open <path>");
        assert!(!error.safe_message().contains("arthur"));

        let long = MiraeError::new(
            code("PROJECT_OPEN_FAILED"),
            ErrorCategory::PersistentInfrastructure,
            SubsystemId::Project,
            &"x".repeat(MAX_SAFE_MESSAGE_CHARACTERS * 2),
        );

        assert_eq!(
            long.safe_message().chars().count(),
            MAX_SAFE_MESSAGE_CHARACTERS
        );
    }

    #[test]
    fn display_shows_only_the_safe_message() {
        let error = sample().with_source(std::io::Error::other("device /home/arthur/x busy"));

        assert_eq!(error.to_string(), "screen capture permission was denied");
        assert!(!error.to_string().contains("arthur"));
    }

    #[test]
    fn preserves_the_root_cause_through_the_chain() {
        let inner = std::io::Error::other("tls certificate rejected");
        let error = sample().with_source(inner);

        assert_eq!(error.chain_length(), 2);
        assert_eq!(error.root_cause().to_string(), "tls certificate rejected");
        assert!(error.source().is_some());
    }

    #[test]
    fn an_error_without_a_source_is_its_own_root_cause() {
        let error = sample();

        assert_eq!(error.chain_length(), 1);
        assert_eq!(error.root_cause().to_string(), error.safe_message());
    }

    #[test]
    fn cancellation_is_not_reported_as_a_failure() {
        let cancelled = MiraeError::new(
            code("OPERATION_CANCELLED"),
            ErrorCategory::Cancellation,
            SubsystemId::Runtime,
            "the operation was cancelled",
        );

        assert!(!cancelled.is_failure());
        assert_eq!(cancelled.severity(), ErrorSeverity::Info);
        assert!(sample().is_failure());
    }

    #[test]
    fn carries_bounded_context() {
        let mut context = ErrorContext::new();
        context
            .insert_id("source_id", 12)
            .insert_label("protocol_phase", "hello");

        let error = sample()
            .with_context(context)
            .with_context_value("state_generation", ContextValue::Generation(9));

        assert_eq!(error.context().len(), 3);
        assert_eq!(
            error.context().get("state_generation"),
            Some(&ContextValue::Generation(9))
        );
    }

    #[test]
    fn diagnostic_line_is_complete_and_safe() {
        let error = sample()
            .with_correlation_id(CorrelationId::from_u128(255))
            .with_user_action(UserActionHint::GrantPermission)
            .with_context_value("source_id", ContextValue::Id(3))
            .with_source(std::io::Error::other("denied by /home/arthur/policy"));

        let line = error.diagnostic_line();

        assert!(line.contains("CAPTURE_PERMISSION_DENIED"));
        assert!(line.contains("severity=error"));
        assert!(line.contains("category=permission"));
        assert!(line.contains("subsystem=media"));
        assert!(line.contains("retryability=retry_after_user_action"));
        assert!(line.contains("correlation=000000000000000000000000000000ff"));
        assert!(line.contains("user_action=grant_permission"));
        assert!(line.contains("source_id=3"));
        // The source chain is never formatted into a loggable line.
        assert!(!line.contains("arthur"));
        assert!(!line.contains("denied by"));
    }

    #[test]
    fn correlation_defaults_to_none() {
        assert!(sample().correlation_id().is_none());
    }
}

//! Structured fields and their redaction classes.
//!
//! Canonical documentation: `docs/06-quality/606-logging-and-tracing.md` sections
//! 2 and 5, and `617-privacy-and-telemetry.md`.
//!
//! The redaction class is part of the field, not a decision made at the sink, so a
//! field cannot be logged safely in one place and unsafely in another.
//!
//! policy-allow: local-path - a test fixture proves that an absolute path in a
//! text field is redacted when the field is constructed

use core::fmt;

use mirae_errors::redaction;

/// Maximum length of a text field value, in characters.
pub const MAX_TEXT_FIELD_CHARACTERS: usize = 128;

/// How a field may be handled (`606` section 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RedactionClass {
    /// Safe anywhere, including telemetry: ids, counts, states.
    Public,
    /// Safe in local logs and support exports: internal names and phases.
    Internal,
    /// Identifies a person or their content. Hashed before it is written, so
    /// occurrences can still be correlated without revealing the value.
    Private,
    /// Credentials and keys. Never written, in any mode.
    Secret,
    /// Frame, sample, or project payload. Never written, in any mode.
    MediaContent,
}

impl RedactionClass {
    /// Whether a field of this class may appear in a normal log.
    ///
    /// `Secret` and `MediaContent` never may (`606` invariant 3).
    #[must_use]
    pub const fn is_loggable(self) -> bool {
        matches!(self, Self::Public | Self::Internal | Self::Private)
    }

    /// Whether the value must be hashed rather than written as-is.
    #[must_use]
    pub const fn requires_hashing(self) -> bool {
        matches!(self, Self::Private)
    }

    /// A stable identifier for tooling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Internal => "internal",
            Self::Private => "private",
            Self::Secret => "secret",
            Self::MediaContent => "media_content",
        }
    }
}

impl fmt::Display for RedactionClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A structured field value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldValue {
    /// A signed count or measurement.
    Integer(i64),
    /// An identifier, generation, or unsigned measurement.
    Unsigned(u64),
    /// A flag.
    Flag(bool),
    /// A fixed label chosen by the code, such as a state or phase.
    Label(&'static str),
    /// Text, redacted and truncated when the field is built.
    Text(String),
}

impl fmt::Display for FieldValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(formatter, "{value}"),
            Self::Unsigned(value) => write!(formatter, "{value}"),
            Self::Flag(value) => write!(formatter, "{value}"),
            Self::Label(value) => formatter.write_str(value),
            Self::Text(value) => formatter.write_str(value),
        }
    }
}

/// A named value with a declared redaction class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    name: &'static str,
    value: FieldValue,
    class: RedactionClass,
}

impl Field {
    /// A public field, safe anywhere.
    #[must_use]
    pub const fn public(name: &'static str, value: FieldValue) -> Self {
        Self {
            name,
            value,
            class: RedactionClass::Public,
        }
    }

    /// An internal field, safe in local logs and support exports.
    #[must_use]
    pub const fn internal(name: &'static str, value: FieldValue) -> Self {
        Self {
            name,
            value,
            class: RedactionClass::Internal,
        }
    }

    /// A private field. Its value is hashed when the event is written.
    #[must_use]
    pub const fn private(name: &'static str, value: FieldValue) -> Self {
        Self {
            name,
            value,
            class: RedactionClass::Private,
        }
    }

    /// A secret field. Constructing one is allowed so that call sites can be
    /// explicit; writing one is not, and the tracer drops and counts it.
    #[must_use]
    pub const fn secret(name: &'static str, value: FieldValue) -> Self {
        Self {
            name,
            value,
            class: RedactionClass::Secret,
        }
    }

    /// A media-content field. Never written, like [`Field::secret`].
    #[must_use]
    pub const fn media_content(name: &'static str, value: FieldValue) -> Self {
        Self {
            name,
            value,
            class: RedactionClass::MediaContent,
        }
    }

    /// Build a text field, redacting and truncating the value immediately.
    ///
    /// Redaction happens here rather than at the sink, so an unredacted value is
    /// never held in memory longer than the call.
    #[must_use]
    pub fn text(name: &'static str, class: RedactionClass, value: &str) -> Self {
        let safe = redaction::truncate(
            &redaction::normalize_whitespace(&redaction::redact_paths(value)),
            MAX_TEXT_FIELD_CHARACTERS,
        );

        Self {
            name,
            value: FieldValue::Text(safe),
            class,
        }
    }

    /// The field name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// The value as stored.
    #[must_use]
    pub const fn value(&self) -> &FieldValue {
        &self.value
    }

    /// The redaction class.
    #[must_use]
    pub const fn class(&self) -> RedactionClass {
        self.class
    }

    /// The value as it should be written, hashing private values.
    ///
    /// Returns `None` when the field must not be written at all.
    #[must_use]
    pub fn rendered_value(&self) -> Option<String> {
        if !self.class.is_loggable() {
            return None;
        }

        if self.class.requires_hashing() {
            return Some(format!(
                "hashed:{:016x}",
                stable_hash(&self.value.to_string())
            ));
        }

        Some(self.value.to_string())
    }
}

/// A stable, non-cryptographic hash used to correlate private values.
///
/// FNV-1a: small, dependency-free, and stable across runs and processes, which is
/// what correlation needs. It is deliberately not a security primitive; a private
/// field is hashed to avoid writing the value, not to protect a secret. Secrets
/// are never written at all.
#[must_use]
pub fn stable_hash(value: &str) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = OFFSET_BASIS;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_and_media_fields_are_never_loggable() {
        assert!(!RedactionClass::Secret.is_loggable());
        assert!(!RedactionClass::MediaContent.is_loggable());
        assert!(RedactionClass::Public.is_loggable());
        assert!(RedactionClass::Internal.is_loggable());
        assert!(RedactionClass::Private.is_loggable());
    }

    #[test]
    fn a_secret_field_renders_nothing() {
        let field = Field::secret("stream_key", FieldValue::Label("live_abc123"));

        assert_eq!(field.rendered_value(), None);
        assert_eq!(
            Field::media_content("frame", FieldValue::Unsigned(1)).rendered_value(),
            None
        );
    }

    #[test]
    fn a_private_field_is_hashed_and_correlatable() {
        let first = Field::private("account", FieldValue::Label("arthur"));
        let second = Field::private("other_account", FieldValue::Label("arthur"));
        let different = Field::private("account", FieldValue::Label("someone-else"));

        let rendered = first.rendered_value().unwrap_or_default();

        assert!(rendered.starts_with("hashed:"));
        assert!(!rendered.contains("arthur"));
        // The same value hashes the same way, so occurrences correlate.
        assert_eq!(rendered, second.rendered_value().unwrap_or_default());
        assert_ne!(rendered, different.rendered_value().unwrap_or_default());
    }

    #[test]
    fn public_and_internal_fields_render_their_value() {
        assert_eq!(
            Field::public("source_id", FieldValue::Unsigned(7)).rendered_value(),
            Some("7".to_owned())
        );
        assert_eq!(
            Field::internal("phase", FieldValue::Label("handshake")).rendered_value(),
            Some("handshake".to_owned())
        );
    }

    #[test]
    fn text_fields_are_redacted_and_bounded_on_construction() {
        let field = Field::text(
            "detail",
            RedactionClass::Internal,
            "open C:\\Users\\arthur\\p.mirae failed",
        );

        assert_eq!(
            field.value(),
            &FieldValue::Text("open <path> failed".to_owned())
        );

        let long = Field::text(
            "detail",
            RedactionClass::Internal,
            &"x".repeat(MAX_TEXT_FIELD_CHARACTERS * 3),
        );
        let stored = match long.value() {
            FieldValue::Text(text) => text.chars().count(),
            _ => 0,
        };

        assert_eq!(stored, MAX_TEXT_FIELD_CHARACTERS);
    }

    #[test]
    fn the_hash_is_stable_and_distinguishes_values() {
        assert_eq!(stable_hash("mirae"), stable_hash("mirae"));
        assert_ne!(stable_hash("mirae"), stable_hash("mirae "));
        // Pinned so a change to the algorithm cannot silently break correlation
        // across builds.
        assert_eq!(stable_hash(""), 0xcbf2_9ce4_8422_2325);
    }
}

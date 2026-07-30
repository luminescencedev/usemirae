//! Bounded, safe diagnostic context.
//!
//! Canonical documentation: `docs/06-quality/605-error-model.md` section 7.
//!
//! Context is added at boundaries and uses ids and generations rather than
//! free-form prose, so the same error is not restated at every layer. The store is
//! bounded: `CLAUDE.md` requires every queue, cache, and collection to have a
//! limit, and diagnostic context attached in a retry loop is exactly where an
//! unbounded collection would grow.
//!
//! policy-allow: local-path - a test fixture proves that an absolute path in a
//! context value is redacted before it is stored

use core::fmt;

use crate::redaction;

/// Maximum number of entries kept on one error.
pub const MAX_CONTEXT_ENTRIES: usize = 16;

/// Maximum length of a text value, in characters.
pub const MAX_TEXT_VALUE_CHARACTERS: usize = 128;

/// A single safe context value.
///
/// Deliberately narrow: ids, generations, and short labels are safe to log, while
/// arbitrary payloads are not. Text is redacted and truncated on the way in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextValue {
    /// A numeric identifier, such as a source or output id.
    Id(u64),
    /// A generation counter, used to spot stale replicas.
    Generation(u64),
    /// A fixed label chosen by the code, such as a protocol phase.
    Label(&'static str),
    /// Redacted, truncated text.
    Text(String),
}

impl fmt::Display for ContextValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(value) | Self::Generation(value) => write!(formatter, "{value}"),
            Self::Label(value) => formatter.write_str(value),
            Self::Text(value) => formatter.write_str(value),
        }
    }
}

/// Bounded key and value pairs attached to an error.
///
/// Keys are `&'static str`, so a key can never be built from user input.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ErrorContext {
    entries: Vec<(&'static str, ContextValue)>,
    /// How many entries were dropped because the bound was reached.
    dropped: usize,
}

impl ErrorContext {
    /// An empty context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            dropped: 0,
        }
    }

    /// Add a value, dropping it if the bound is already reached.
    ///
    /// Dropping is counted and reported by [`Self::dropped`] rather than silently
    /// ignored, so a truncated context is visible in diagnostics.
    pub fn insert(&mut self, key: &'static str, value: ContextValue) -> &mut Self {
        if self.entries.len() >= MAX_CONTEXT_ENTRIES {
            self.dropped = self.dropped.saturating_add(1);
            return self;
        }

        // A repeated key replaces the earlier value: the innermost boundary that
        // set it is the most specific.
        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|(existing_key, _)| *existing_key == key)
        {
            existing.1 = value;
            return self;
        }

        self.entries.push((key, value));
        self
    }

    /// Add a numeric identifier.
    pub fn insert_id(&mut self, key: &'static str, id: u64) -> &mut Self {
        self.insert(key, ContextValue::Id(id))
    }

    /// Add a generation counter.
    pub fn insert_generation(&mut self, key: &'static str, generation: u64) -> &mut Self {
        self.insert(key, ContextValue::Generation(generation))
    }

    /// Add a fixed label.
    pub fn insert_label(&mut self, key: &'static str, label: &'static str) -> &mut Self {
        self.insert(key, ContextValue::Label(label))
    }

    /// Add text, redacted and truncated first.
    ///
    /// Redaction removes absolute paths only; it cannot make a credential safe, so
    /// callers still must not pass one.
    pub fn insert_text(&mut self, key: &'static str, text: &str) -> &mut Self {
        let safe = redaction::truncate(
            &redaction::normalize_whitespace(&redaction::redact_paths(text)),
            MAX_TEXT_VALUE_CHARACTERS,
        );

        self.insert(key, ContextValue::Text(safe))
    }

    /// The entries, in insertion order.
    #[must_use]
    pub fn entries(&self) -> &[(&'static str, ContextValue)] {
        &self.entries
    }

    /// Look up one value.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&ContextValue> {
        self.entries
            .iter()
            .find(|(existing, _)| *existing == key)
            .map(|(_, value)| value)
    }

    /// How many entries were dropped because the bound was reached.
    #[must_use]
    pub const fn dropped(&self) -> usize {
        self.dropped
    }

    /// Whether any context was recorded.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// How many entries are stored.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, (key, value)) in self.entries.iter().enumerate() {
            if index != 0 {
                formatter.write_str(" ")?;
            }
            write!(formatter, "{key}={value}")?;
        }

        if self.dropped != 0 {
            if !self.entries.is_empty() {
                formatter.write_str(" ")?;
            }
            write!(formatter, "dropped={}", self.dropped)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_ids_generations_and_labels() {
        let mut context = ErrorContext::new();
        context
            .insert_id("source_id", 7)
            .insert_generation("state_generation", 42)
            .insert_label("protocol_phase", "handshake");

        assert_eq!(context.get("source_id"), Some(&ContextValue::Id(7)));
        assert_eq!(
            context.get("state_generation"),
            Some(&ContextValue::Generation(42))
        );
        assert_eq!(
            context.get("protocol_phase"),
            Some(&ContextValue::Label("handshake"))
        );
        assert_eq!(context.len(), 3);
        assert!(!context.is_empty());
    }

    #[test]
    fn redacts_and_truncates_text_values() {
        let mut context = ErrorContext::new();
        context.insert_text("detail", "open C:\\Users\\arthur\\p.mirae failed");

        assert_eq!(
            context.get("detail"),
            Some(&ContextValue::Text("open <path> failed".to_owned()))
        );

        let long = "x".repeat(MAX_TEXT_VALUE_CHARACTERS * 2);
        context.insert_text("long", &long);

        let stored = match context.get("long") {
            Some(ContextValue::Text(text)) => text.chars().count(),
            _ => 0,
        };
        assert_eq!(stored, MAX_TEXT_VALUE_CHARACTERS);
    }

    #[test]
    fn is_bounded_and_counts_what_it_dropped() {
        let mut context = ErrorContext::new();

        for index in 0..(MAX_CONTEXT_ENTRIES as u64 + 5) {
            // Distinct static keys are not available in a loop, so vary the value
            // and use one key per bucket to fill the store.
            context.insert(
                match index % 4 {
                    0 => "a",
                    1 => "b",
                    2 => "c",
                    _ => "d",
                },
                ContextValue::Id(index),
            );
        }

        // Four distinct keys, repeatedly replaced: still bounded, nothing dropped.
        assert_eq!(context.len(), 4);
        assert_eq!(context.dropped(), 0);
    }

    #[test]
    fn drops_entries_past_the_bound_and_reports_the_count() {
        let mut context = ErrorContext::new();
        let keys: [&'static str; 20] = [
            "k00", "k01", "k02", "k03", "k04", "k05", "k06", "k07", "k08", "k09", "k10", "k11",
            "k12", "k13", "k14", "k15", "k16", "k17", "k18", "k19",
        ];

        for (index, key) in keys.iter().enumerate() {
            context.insert_id(key, index as u64);
        }

        assert_eq!(context.len(), MAX_CONTEXT_ENTRIES);
        assert_eq!(context.dropped(), keys.len() - MAX_CONTEXT_ENTRIES);
        assert!(context.to_string().contains("dropped=4"));
    }

    #[test]
    fn a_repeated_key_takes_the_most_specific_value() {
        let mut context = ErrorContext::new();
        context.insert_label("protocol_phase", "handshake");
        context.insert_label("protocol_phase", "capability");

        assert_eq!(
            context.get("protocol_phase"),
            Some(&ContextValue::Label("capability"))
        );
        assert_eq!(context.len(), 1);
    }

    #[test]
    fn formats_as_key_value_pairs() {
        let mut context = ErrorContext::new();
        context
            .insert_id("source_id", 7)
            .insert_label("phase", "hello");

        assert_eq!(context.to_string(), "source_id=7 phase=hello");
        assert_eq!(ErrorContext::new().to_string(), "");
    }
}

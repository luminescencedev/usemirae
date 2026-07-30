//! The project-domain entities the state store holds.
//!
//! Canonical documentation: `docs/00-foundations/005-domain-model.md` section 3.
//!
//! These are the in-memory domain types. They deliberately do not derive
//! serialization: `005` section 9 keeps the persisted schema separate from the
//! runtime model so that a field can be added here without silently changing a
//! file format, and so that a file format can evolve without dragging the domain
//! behind it. The schema types arrive with `MIR-0107`.
//!
//! Only what the kernel needs today is modelled. Transforms, crops, effects,
//! groups, and output profiles are named by `005` and arrive with the tickets
//! that first act on them, because a field nobody reads is a field nobody
//! validates.

use mirae_types::{SceneId, SceneItemId, SourceId};

/// Longest name accepted for any entity.
///
/// `005` section 8 requires bounded user text. The bound is on characters rather
/// than bytes so the limit means the same thing in every script.
pub const MAX_NAME_CHARACTERS: usize = 256;

/// A semantic composition (`005` section 3.2).
///
/// A scene is not a rendered image. It names what should be composed and in what
/// order; what that looks like is the renderer's problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    /// Stable identity.
    pub id: SceneId,
    /// Display name, bounded by [`MAX_NAME_CHARACTERS`].
    pub name: EntityName,
    /// Root scene items, in composition order.
    ///
    /// Order is the z-order (`005` section 3.4), which is why it lives in a
    /// sequence rather than being derived from a field on each item.
    pub items: Vec<SceneItemId>,
}

/// A reusable source definition (`005` section 3.3).
///
/// It describes configuration. It never holds a capture session, a decoder, or a
/// socket: those belong to a source runtime, and `106` section 10 keeps them out
/// of anything a snapshot can retain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDefinition {
    /// Stable identity.
    pub id: SourceId,
    /// Display name, bounded by [`MAX_NAME_CHARACTERS`].
    pub name: EntityName,
    /// What kind of source this is.
    pub kind: SourceKind,
}

/// The kinds of source the kernel knows about today.
///
/// One variant, because one is enough to build and test the kernel and because
/// `005` section 3.3 lists a dozen more that each need their own configuration,
/// capability checks, and platform work. They arrive with their tickets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceKind {
    /// A generated solid colour: no device, no file, no permission prompt.
    Color,
}

/// The placement of a source inside a scene (`005` section 3.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneItem {
    /// Stable identity.
    pub id: SceneItemId,
    /// The scene this item belongs to.
    pub scene: SceneId,
    /// The source this item places.
    ///
    /// Several items may reference one source definition, which is why the
    /// reference points at an identifier rather than owning the definition.
    pub source: SourceId,
    /// Whether the item participates in composition.
    pub visible: bool,
}

/// A validated, bounded entity name.
///
/// A newtype rather than a `String`, so that the bound is checked once at the
/// edge instead of everywhere a name is used, and so an over-long name from an
/// untrusted project file cannot reach the store at all.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityName(String);

impl EntityName {
    /// Validate and wrap a name.
    pub fn new(text: impl Into<String>) -> Result<Self, NameError> {
        let text = text.into();

        if text.is_empty() {
            return Err(NameError::Empty);
        }

        if text.chars().count() > MAX_NAME_CHARACTERS {
            return Err(NameError::TooLong);
        }

        // Control characters would let a name rewrite a terminal diagnostic or
        // break a single-line log record. A name is user text, not markup.
        if text.chars().any(char::is_control) {
            return Err(NameError::ControlCharacter);
        }

        Ok(Self(text))
    }

    /// The name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EntityName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Why a name was refused.
///
/// Each variant names the rule, never the rejected text: the text came from
/// outside and must not travel into a log through an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameError {
    /// The name had no characters.
    Empty,
    /// The name exceeded [`MAX_NAME_CHARACTERS`].
    TooLong,
    /// The name contained a control character.
    ControlCharacter,
}

impl std::fmt::Display for NameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "a name cannot be empty",
            Self::TooLong => "the name is longer than the project format accepts",
            Self::ControlCharacter => "a name cannot contain control characters",
        })
    }
}

impl std::error::Error for NameError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_bounded_and_refuses_control_characters() {
        // 005 section 8: bounded names and user text.
        assert!(EntityName::new("Main scene").is_ok());
        assert_eq!(EntityName::new(""), Err(NameError::Empty));
        assert_eq!(
            EntityName::new("a".repeat(MAX_NAME_CHARACTERS + 1)),
            Err(NameError::TooLong)
        );
        assert_eq!(
            EntityName::new("line\nbreak"),
            Err(NameError::ControlCharacter)
        );
        assert_eq!(
            EntityName::new("bell\u{7}"),
            Err(NameError::ControlCharacter)
        );
    }

    #[test]
    fn the_bound_counts_characters_rather_than_bytes() {
        // A limit expressed in bytes would let a Latin name be twice as long as
        // a Japanese one, which is a bug that only some users ever see.
        let japanese = "設".repeat(MAX_NAME_CHARACTERS);

        assert!(
            japanese.len() > MAX_NAME_CHARACTERS,
            "three bytes per character"
        );
        assert!(EntityName::new(japanese).is_ok());
    }

    #[test]
    fn a_rejected_name_is_not_echoed_by_its_error() {
        let secret = "\u{1}super-secret-stream-key";
        let error = EntityName::new(secret).err();

        assert_eq!(error, Some(NameError::ControlCharacter));
        assert!(!NameError::ControlCharacter.to_string().contains("secret"));
    }
}

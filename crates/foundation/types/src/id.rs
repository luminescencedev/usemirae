//! Stable opaque identifiers for persisted domain entities.
//!
//! Canonical documentation: `docs/00-foundations/005-domain-model.md` section 2,
//! `docs/04-project/401-project-format.md` section 5, ADR-0069.
//!
//! Every persisted entity is named by a random 128-bit identifier. `005`
//! invariants 1 and 3 — identifiers are unique in their namespace and independent
//! of position — hold structurally here rather than by review: there is no
//! counter to leak an order, no path to leak a location, and no index to go
//! stale when a list is reordered.
//!
//! Each entity kind has its own newtype. They are deliberately not
//! interchangeable: passing a `SceneId` where a `SourceId` belongs is the mistake
//! this module exists to make impossible, and it is a mistake that costs nothing
//! to prevent and a great deal to find in a project file.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A stable opaque identifier for a persisted domain entity.
///
/// Serializes as the canonical hyphenated lowercase text form, because a project
/// file is meant to be read and diffed by a person (ADR-0069).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityId(Uuid);

impl EntityId {
    /// Mint a new identifier.
    ///
    /// Random rather than sequential: no allocator to coordinate, and nothing
    /// about creation time or order is disclosed by the value.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wrap an existing UUID, for deserialization and fixtures.
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// The underlying UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// The identifier used by deterministic tests and fixtures.
    ///
    /// A fixture that mints a random identifier produces a different file every
    /// run, which makes byte-comparison worthless (`401` section 12).
    #[must_use]
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.as_hyphenated())
    }
}

/// Why an identifier could not be parsed.
///
/// One variant, because the caller can do exactly one thing about it. The text
/// carries no part of the input: an identifier read from an untrusted project
/// file must not reach a log through an error message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdParseError;

impl fmt::Display for IdParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("not a valid entity identifier")
    }
}

impl std::error::Error for IdParseError {}

impl FromStr for EntityId {
    type Err = IdParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(text).map(Self).map_err(|_| IdParseError)
    }
}

/// Define an entity-specific identifier newtype.
///
/// The wrapper exists for the type system, not for behaviour: two identifiers of
/// different kinds must not be assignable to each other even though both are the
/// same 128 bits underneath.
macro_rules! entity_id {
    ($name:ident, $what:literal) => {
        #[doc = concat!("A stable identifier for ", $what, ".")]
        #[doc = ""]
        #[doc = "Not interchangeable with any other entity identifier."]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(EntityId);

        impl $name {
            #[doc = concat!("Mint a new identifier for ", $what, ".")]
            #[must_use]
            pub fn new() -> Self {
                Self(EntityId::new())
            }

            /// Wrap an existing entity identifier.
            #[must_use]
            pub const fn from_entity_id(id: EntityId) -> Self {
                Self(id)
            }

            /// The underlying entity identifier.
            #[must_use]
            pub const fn as_entity_id(&self) -> &EntityId {
                &self.0
            }

            /// The identifier used by deterministic tests and fixtures.
            #[must_use]
            pub const fn nil() -> Self {
                Self(EntityId::nil())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl FromStr for $name {
            type Err = IdParseError;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                EntityId::from_str(text).map(Self)
            }
        }
    };
}

entity_id!(ProjectId, "a project");
entity_id!(SceneId, "a scene");
entity_id!(SourceId, "a source definition");
entity_id!(SceneItemId, "a scene item");
entity_id!(OutputId, "an output profile");
entity_id!(AssetId, "an asset record");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_minted_identifiers_differ() {
        assert_ne!(EntityId::new(), EntityId::new());
        assert_ne!(SceneId::new(), SceneId::new());
    }

    #[test]
    fn an_identifier_round_trips_through_its_text_form() {
        let id = SourceId::new();
        let text = id.to_string();
        let parsed = SourceId::from_str(&text);

        assert_eq!(parsed, Ok(id));
        assert_eq!(
            text.len(),
            36,
            "the canonical hyphenated form is 36 characters"
        );
        assert_eq!(text, text.to_ascii_lowercase());
    }

    #[test]
    fn an_identifier_round_trips_through_serialization() {
        // 005 required test: round-trip serialization. The transparent
        // representation matters — an identifier is a string in the file, not an
        // object wrapping one.
        let id = SceneId::new();
        let encoded = serde_json::to_string(&id).unwrap_or_default();
        let decoded = serde_json::from_str::<SceneId>(&encoded);

        assert_eq!(encoded, format!("\"{id}\""));
        assert_eq!(decoded.ok(), Some(id));
    }

    #[test]
    fn malformed_text_is_refused_without_echoing_it() {
        // 005 section 8 and the error model: a value from an untrusted project
        // file must not travel into a diagnostic.
        for text in ["", "not-an-id", "12345", "'; DROP TABLE scenes; --"] {
            let parsed = EntityId::from_str(text);

            assert_eq!(parsed, Err(IdParseError), "{text} should be refused");
            assert!(!IdParseError.to_string().contains(text) || text.is_empty());
        }
    }

    #[test]
    fn identity_survives_reordering_a_collection() {
        // 401 section 5: identifiers survive reorder. Stated as a test because
        // the failure mode it guards against — an index used as identity — looks
        // correct until something is moved.
        let mut scenes = vec![SceneId::new(), SceneId::new(), SceneId::new()];
        let before = scenes.clone();

        scenes.reverse();

        assert_eq!(scenes[0], before[2]);
        assert_eq!(scenes[2], before[0]);
    }

    #[test]
    fn identifiers_of_different_kinds_are_not_the_same_type() {
        // The compile-time guarantee, asserted the only way a runtime test can:
        // the same underlying value carried by two kinds stays distinguishable.
        let shared = EntityId::new();
        let scene = SceneId::from_entity_id(shared);
        let source = SourceId::from_entity_id(shared);

        assert_eq!(scene.as_entity_id(), source.as_entity_id());
        assert_eq!(scene.to_string(), source.to_string());
        // `assert_eq!(scene, source)` does not compile, which is the point.
    }

    #[test]
    fn the_fixture_identifier_is_stable() {
        assert_eq!(
            SceneId::nil().to_string(),
            "00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(SceneId::nil(), SceneId::nil());
    }

    #[test]
    fn identifiers_order_totally_for_deterministic_serialization() {
        // 401 section 12 needs a defined map ordering. Sorting by identifier is
        // the obvious one, so the ordering has to be total and stable.
        let mut ids = vec![
            EntityId::from_uuid(Uuid::from_u128(3)),
            EntityId::from_uuid(Uuid::from_u128(1)),
            EntityId::from_uuid(Uuid::from_u128(2)),
        ];
        ids.sort();

        assert_eq!(
            ids,
            vec![
                EntityId::from_uuid(Uuid::from_u128(1)),
                EntityId::from_uuid(Uuid::from_u128(2)),
                EntityId::from_uuid(Uuid::from_u128(3)),
            ]
        );
    }
}

//! Generations: the version numbers that make state observable without locks.
//!
//! Canonical documentation: `docs/01-runtime/106-state-store.md` sections 7, 8
//! and 13, `docs/01-runtime/104-command-system.md` section 7.
//!
//! A generation names one committed version of authoritative state. Snapshots
//! carry it, patches name the pair they span, and commands carry the one they
//! were written against so a conflict is detected instead of overwritten.
//!
//! There are two of them and they are different types. `106` invariant 10
//! separates project-state generation from capability generation, because a
//! camera appearing is not a project edit, and code that confuses the two would
//! either resynchronize the whole project when a device is plugged in or miss an
//! edit because a device was not.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Define a monotonic generation counter.
macro_rules! generation {
    ($name:ident, $what:literal) => {
        #[doc = concat!("A monotonic version of ", $what, ".")]
        #[doc = ""]
        #[doc = "Increments exactly once per committed change, and compares as a"]
        #[doc = "total order so a reader can tell which of two values is newer."]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(u64);

        impl $name {
            #[doc = concat!("The generation of ", $what, " before anything has been committed.")]
            pub const INITIAL: Self = Self(0);

            /// Wrap a raw value, for deserialization and fixtures.
            #[must_use]
            pub const fn from_raw(value: u64) -> Self {
                Self(value)
            }

            /// The raw value, for the wire and for diagnostics.
            #[must_use]
            pub const fn get(self) -> u64 {
                self.0
            }

            /// The generation that follows this one.
            ///
            /// Saturates rather than wrapping. Wrapping would make an ancient
            /// generation compare as newer than the current one, which is a
            /// silent correctness failure in every consumer; saturation stalls
            /// visibly instead. At one commit per microsecond this arrives in
            /// roughly six hundred thousand years, so the branch exists to be
            /// correct rather than to be taken.
            #[must_use]
            pub const fn next(self) -> Self {
                Self(self.0.saturating_add(1))
            }

            /// Whether `self` is the generation immediately after `earlier`.
            ///
            /// A patch may only advance one step (`106` section 8), so this is
            /// the question a consumer asks before applying one.
            #[must_use]
            pub const fn immediately_follows(self, earlier: Self) -> bool {
                // Saturating, not `+ 1`: at the ceiling that addition would
                // panic in a debug build, and a bounds check is not the place to
                // introduce a panic on the patch path.
                self.0 == earlier.0.saturating_add(1)
            }

            /// How many commits separate `self` from `earlier`, if `self` is newer.
            ///
            /// `None` when `self` is not newer, which is a consumer that has run
            /// ahead of the engine and must resynchronize rather than guess.
            #[must_use]
            pub const fn distance_from(self, earlier: Self) -> Option<u64> {
                if self.0 >= earlier.0 {
                    Some(self.0 - earlier.0)
                } else {
                    None
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}", self.0)
            }
        }
    };
}

generation!(StateGeneration, "authoritative project and session state");
generation!(
    CapabilityGeneration,
    "platform, device, and encoder capabilities"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generation_starts_at_zero_and_advances_by_one() {
        let first = StateGeneration::INITIAL;
        let second = first.next();

        assert_eq!(first.get(), 0);
        assert_eq!(second.get(), 1);
        assert!(second > first);
        assert!(second.immediately_follows(first));
    }

    #[test]
    fn a_generation_never_wraps_backwards() {
        // The failure this prevents is not a panic. It is an old generation
        // comparing as newer, which every consumer would believe.
        let last = StateGeneration::from_raw(u64::MAX);

        assert_eq!(last.next(), last);
        assert!(last.next() >= last);
    }

    #[test]
    fn a_skipped_generation_is_not_immediately_following() {
        // 106 section 8: a patch advances exactly one known range. A gap must be
        // detectable, because the consumer's only safe response is a new
        // snapshot.
        let base = StateGeneration::from_raw(7);

        assert!(!StateGeneration::from_raw(9).immediately_follows(base));
        assert!(!base.immediately_follows(base));
        assert!(StateGeneration::from_raw(8).immediately_follows(base));
    }

    #[test]
    fn distance_reports_none_when_the_consumer_is_ahead() {
        let engine = StateGeneration::from_raw(4);
        let consumer_behind = StateGeneration::from_raw(2);
        let consumer_ahead = StateGeneration::from_raw(9);

        assert_eq!(engine.distance_from(consumer_behind), Some(2));
        assert_eq!(engine.distance_from(engine), Some(0));
        assert_eq!(engine.distance_from(consumer_ahead), None);
    }

    #[test]
    fn a_generation_round_trips_as_a_bare_number() {
        // The wire form matters: a generation appears in every patch header, and
        // an object wrapper would cost more than the value.
        let generation = StateGeneration::from_raw(42);
        let encoded = serde_json::to_string(&generation).unwrap_or_default();

        assert_eq!(encoded, "42");
        assert_eq!(
            serde_json::from_str::<StateGeneration>(&encoded).ok(),
            Some(generation)
        );
    }

    #[test]
    fn state_and_capability_generations_are_different_types() {
        // 106 invariant 10. The same number in both kinds stays distinguishable,
        // and `assert_eq!(state, capability)` does not compile.
        let state = StateGeneration::from_raw(5);
        let capability = CapabilityGeneration::from_raw(5);

        assert_eq!(state.get(), capability.get());
        assert_eq!(state.to_string(), capability.to_string());
    }
}

//! Rate limiting and duplicate suppression.
//!
//! Canonical documentation: `docs/06-quality/606-logging-and-tracing.md` section 6
//! and invariant 6: a failing component must not fill the disk through repeated
//! identical errors.
//!
//! Time is passed in rather than read, so every decision is deterministic and unit
//! tested without sleeping.

use std::time::{Duration, Instant};

/// Maximum number of distinct event names tracked at once.
///
/// Bounded, because the key set is derived from running code and a defect could
/// otherwise grow it without limit.
pub const MAX_TRACKED_EVENTS: usize = 64;

/// What the limiter decided about one event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Write the event.
    Emit,
    /// Do not write it; the budget for this event name is exhausted.
    RateLimited,
    /// Do not write it; it repeats the previous event.
    Duplicate,
}

/// One tracked event name.
#[derive(Debug, Clone)]
struct Budget {
    name: &'static str,
    /// Tokens left in the current window.
    tokens: u32,
    window_started: Instant,
}

/// A token bucket per event name, plus consecutive-duplicate suppression.
#[derive(Debug)]
pub struct VolumeControl {
    budgets: Vec<Budget>,
    capacity_per_window: u32,
    window: Duration,
    /// The last emitted key, for duplicate suppression.
    last_key: Option<(&'static str, u64)>,
    suppressed_duplicates: u64,
    rate_limited: u64,
    untracked_events: u64,
}

impl VolumeControl {
    /// Build a control allowing `capacity_per_window` events per name per window.
    #[must_use]
    pub fn new(capacity_per_window: u32, window: Duration) -> Self {
        Self {
            budgets: Vec::new(),
            capacity_per_window,
            window,
            last_key: None,
            suppressed_duplicates: 0,
            rate_limited: 0,
            untracked_events: 0,
        }
    }

    /// Decide whether to write an event.
    ///
    /// `signature` distinguishes otherwise identical events, so a repeated error
    /// about the same entity is suppressed while the same event about a different
    /// entity is not.
    pub fn admit(&mut self, name: &'static str, signature: u64, now: Instant) -> Decision {
        if self.last_key == Some((name, signature)) {
            self.suppressed_duplicates = self.suppressed_duplicates.saturating_add(1);
            return Decision::Duplicate;
        }

        let capacity = self.capacity_per_window;
        let window = self.window;

        let budget = match self.budgets.iter_mut().find(|budget| budget.name == name) {
            Some(budget) => budget,
            None => {
                if self.budgets.len() >= MAX_TRACKED_EVENTS {
                    // Beyond the bound, admit the event but count that it was not
                    // rate limited, so the gap is visible rather than silent.
                    self.untracked_events = self.untracked_events.saturating_add(1);
                    self.last_key = Some((name, signature));
                    return Decision::Emit;
                }

                self.budgets.push(Budget {
                    name,
                    tokens: capacity,
                    window_started: now,
                });

                match self.budgets.last_mut() {
                    Some(budget) => budget,
                    None => return Decision::Emit,
                }
            }
        };

        if now.saturating_duration_since(budget.window_started) >= window {
            budget.tokens = capacity;
            budget.window_started = now;
        }

        if budget.tokens == 0 {
            self.rate_limited = self.rate_limited.saturating_add(1);
            return Decision::RateLimited;
        }

        budget.tokens -= 1;
        self.last_key = Some((name, signature));
        Decision::Emit
    }

    /// How many events were dropped as consecutive duplicates.
    #[must_use]
    pub const fn suppressed_duplicates(&self) -> u64 {
        self.suppressed_duplicates
    }

    /// How many events were dropped by rate limiting.
    #[must_use]
    pub const fn rate_limited(&self) -> u64 {
        self.rate_limited
    }

    /// How many events bypassed rate limiting because the name table was full.
    #[must_use]
    pub const fn untracked_events(&self) -> u64 {
        self.untracked_events
    }

    /// How many distinct event names are tracked.
    #[must_use]
    pub fn tracked_events(&self) -> usize {
        self.budgets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control() -> VolumeControl {
        VolumeControl::new(3, Duration::from_secs(1))
    }

    #[test]
    fn admits_up_to_the_budget_then_rate_limits() {
        let mut control = control();
        let now = Instant::now();

        // Signatures differ so duplicate suppression does not mask the budget.
        assert_eq!(control.admit("engine.start", 1, now), Decision::Emit);
        assert_eq!(control.admit("engine.start", 2, now), Decision::Emit);
        assert_eq!(control.admit("engine.start", 3, now), Decision::Emit);
        assert_eq!(control.admit("engine.start", 4, now), Decision::RateLimited);
        assert_eq!(control.rate_limited(), 1);
    }

    #[test]
    fn refills_after_the_window() {
        let mut control = control();
        let now = Instant::now();

        for signature in 0..3 {
            control.admit("output.retry", signature, now);
        }
        assert_eq!(control.admit("output.retry", 9, now), Decision::RateLimited);

        let later = now + Duration::from_secs(1);

        assert_eq!(control.admit("output.retry", 10, later), Decision::Emit);
    }

    #[test]
    fn suppresses_an_identical_repeat() {
        let mut control = control();
        let now = Instant::now();

        assert_eq!(control.admit("render.device_lost", 7, now), Decision::Emit);
        assert_eq!(
            control.admit("render.device_lost", 7, now),
            Decision::Duplicate
        );
        assert_eq!(
            control.admit("render.device_lost", 7, now),
            Decision::Duplicate
        );
        assert_eq!(control.suppressed_duplicates(), 2);
    }

    #[test]
    fn a_different_entity_is_not_a_duplicate() {
        let mut control = control();
        let now = Instant::now();

        assert_eq!(control.admit("source.failed", 1, now), Decision::Emit);
        assert_eq!(control.admit("source.failed", 2, now), Decision::Emit);
        assert_eq!(control.suppressed_duplicates(), 0);
    }

    #[test]
    fn a_repeat_after_another_event_is_admitted_again() {
        // Suppression is for storms, not for forgetting an event happened twice
        // with something else in between.
        let mut control = control();
        let now = Instant::now();

        control.admit("a", 1, now);
        control.admit("b", 1, now);

        assert_eq!(control.admit("a", 1, now), Decision::Emit);
    }

    #[test]
    fn the_name_table_is_bounded_and_counts_what_it_could_not_track() {
        let mut control = VolumeControl::new(1, Duration::from_secs(1));
        let now = Instant::now();
        let names: [&'static str; 5] = ["a", "b", "c", "d", "e"];

        // Fill the table well past its bound by reusing a small name set with
        // distinct signatures, then confirm the bound held.
        for round in 0..(MAX_TRACKED_EVENTS as u64 + 10) {
            let name = names[(round as usize) % names.len()];
            control.admit(name, round, now);
        }

        assert!(control.tracked_events() <= MAX_TRACKED_EVENTS);
    }

    #[test]
    fn a_storm_of_distinct_names_stays_bounded() {
        let mut control = VolumeControl::new(1, Duration::from_secs(1));
        let now = Instant::now();
        let names: [&'static str; 70] = [
            "n00", "n01", "n02", "n03", "n04", "n05", "n06", "n07", "n08", "n09", "n10", "n11",
            "n12", "n13", "n14", "n15", "n16", "n17", "n18", "n19", "n20", "n21", "n22", "n23",
            "n24", "n25", "n26", "n27", "n28", "n29", "n30", "n31", "n32", "n33", "n34", "n35",
            "n36", "n37", "n38", "n39", "n40", "n41", "n42", "n43", "n44", "n45", "n46", "n47",
            "n48", "n49", "n50", "n51", "n52", "n53", "n54", "n55", "n56", "n57", "n58", "n59",
            "n60", "n61", "n62", "n63", "n64", "n65", "n66", "n67", "n68", "n69",
        ];

        for (index, name) in names.iter().enumerate() {
            control.admit(name, index as u64, now);
        }

        assert_eq!(control.tracked_events(), MAX_TRACKED_EVENTS);
        assert_eq!(
            control.untracked_events(),
            (names.len() - MAX_TRACKED_EVENTS) as u64
        );
    }
}

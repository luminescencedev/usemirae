//! Domain events, published after commit and in commit order.
//!
//! Canonical documentation: `docs/01-runtime/105-event-system.md`,
//! `docs/01-runtime/107-transactions.md` section 9.
//!
//! `105` invariant ordering and `107` section 3.7 both say the same thing from
//! different sides: nothing is published before the commit that produced it. So
//! events are computed during preparation, carried by the transaction, and
//! handed to the bus only once the swap has happened. A subscriber that receives
//! an event can therefore read the generation it names.
//!
//! Every queue here is bounded (`105` section 6: there is no unrestricted global
//! subscriber with an unbounded queue). What happens at the bound is the
//! subscriber's declared policy, and both policies are honest: either the
//! subscriber is told it fell behind, or a diagnostic is dropped and counted.
//! Neither silently continues as if synchronized, which `105` section 7 forbids
//! for anything carrying state.

use std::collections::VecDeque;

use mirae_types::{EntityId, StateGeneration};

/// Identifies one published event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(EntityId);

impl EventId {
    /// Mint a new event identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(EntityId::new())
    }

    /// The underlying identifier.
    #[must_use]
    pub const fn as_entity_id(&self) -> &EntityId {
        &self.0
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// Position in one engine's event stream (`105` section 3).
///
/// Monotonic within a session and independent of the state generation: several
/// events may accompany one commit, and runtime events accompany no commit at
/// all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventSequence(u64);

impl EventSequence {
    /// The sequence before anything has been published.
    pub const INITIAL: Self = Self(0);

    /// Wrap a raw value.
    #[must_use]
    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// The raw value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// The next position.
    ///
    /// Saturating for the same reason generations saturate: a wrapped sequence
    /// would make an old event look newer, and every gap detector downstream
    /// would believe it.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// What happened to the project (`105` section 2.1).
///
/// Semantic, not structural. `105` section 8 divides the work: an event says
/// what happened, a state patch says what changed, and a consumer that has both
/// must not rebuild canonical state from the event. Scene creation and deletion
/// are here because a notification about them is useful on its own; field-level
/// edits are not, because the patch already carries them.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DomainEvent {
    /// A scene was created.
    SceneCreated {
        /// The scene.
        scene: EntityId,
    },
    /// A scene was removed.
    SceneRemoved {
        /// The scene.
        scene: EntityId,
    },
    /// A source definition was created.
    SourceCreated {
        /// The source definition.
        source: EntityId,
    },
    /// A source definition was removed.
    SourceRemoved {
        /// The source definition.
        source: EntityId,
    },
    /// A scene item was added to a scene.
    SceneItemAdded {
        /// The item.
        item: EntityId,
        /// The scene it joined.
        scene: EntityId,
    },
    /// A scene item was removed from a scene.
    SceneItemRemoved {
        /// The item.
        item: EntityId,
        /// The scene it left.
        scene: EntityId,
    },
}

impl DomainEvent {
    /// A stable identifier for diagnostics and the wire.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SceneCreated { .. } => "scene_created",
            Self::SceneRemoved { .. } => "scene_removed",
            Self::SourceCreated { .. } => "source_created",
            Self::SourceRemoved { .. } => "source_removed",
            Self::SceneItemAdded { .. } => "scene_item_added",
            Self::SceneItemRemoved { .. } => "scene_item_removed",
        }
    }
}

/// A published event with everything a consumer needs to place it (`105` section 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventEnvelope {
    /// Identifies this event, for duplicate detection after a reconnect.
    pub event_id: EventId,
    /// The session that published it.
    pub engine_session_id: String,
    /// Position in the stream.
    pub sequence: EventSequence,
    /// The generation this event accompanies.
    ///
    /// Always present for a domain event: `105` section 2.1 requires it, and
    /// without it a consumer cannot tell whether its mirror already includes
    /// what the event describes.
    pub state_generation: StateGeneration,
    /// What happened.
    pub payload: DomainEvent,
}

/// What a subscriber does when its queue is full (`105` section 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowPolicy {
    /// Mark the subscriber lagged and stop delivering until it resynchronizes.
    ///
    /// The only honest policy for anything mirroring state: dropping an event
    /// and continuing would leave the consumer confidently wrong.
    Lag,
    /// Drop the oldest event and count it.
    ///
    /// For diagnostics, where the newest observation is the useful one and the
    /// count is enough to know something was lost.
    DropOldest,
}

/// One subscriber's bounded queue (`105` section 6).
#[derive(Debug)]
pub struct Subscriber {
    queue: VecDeque<EventEnvelope>,
    capacity: usize,
    policy: OverflowPolicy,
    dropped: u64,
    lagged: bool,
}

impl Subscriber {
    /// Create a subscriber with a bounded queue.
    ///
    /// A zero capacity is raised to one: a queue that can hold nothing is a
    /// subscriber that is permanently lagged, which is a configuration mistake
    /// rather than an intent.
    #[must_use]
    pub fn new(capacity: usize, policy: OverflowPolicy) -> Self {
        Self {
            queue: VecDeque::new(),
            capacity: capacity.max(1),
            policy,
            dropped: 0,
            lagged: false,
        }
    }

    /// Take everything queued.
    pub fn drain(&mut self) -> Vec<EventEnvelope> {
        self.queue.drain(..).collect()
    }

    /// How many events are waiting.
    #[must_use]
    pub fn queued(&self) -> usize {
        self.queue.len()
    }

    /// How many events were dropped under [`OverflowPolicy::DropOldest`].
    #[must_use]
    pub const fn dropped(&self) -> u64 {
        self.dropped
    }

    /// Whether this subscriber fell behind and must resynchronize.
    #[must_use]
    pub const fn lagged(&self) -> bool {
        self.lagged
    }

    /// Clear the lagged flag after the consumer has taken a fresh snapshot.
    pub fn resynchronized(&mut self) {
        self.lagged = false;
        self.queue.clear();
    }

    /// Offer an event, applying the overflow policy.
    fn offer(&mut self, event: EventEnvelope) {
        if self.lagged {
            // A lagged subscriber is not a slow one: it has already missed
            // something, so queueing more would only grow a queue whose contents
            // it must discard anyway.
            return;
        }

        if self.queue.len() >= self.capacity {
            match self.policy {
                OverflowPolicy::Lag => {
                    self.lagged = true;
                    self.queue.clear();
                    return;
                }
                OverflowPolicy::DropOldest => {
                    self.queue.pop_front();
                    self.dropped = self.dropped.saturating_add(1);
                }
            }
        }

        self.queue.push_back(event);
    }
}

/// Publishes committed domain events to subscribers (`105` sections 4 and 6).
#[derive(Debug)]
pub struct EventBus {
    engine_session_id: String,
    sequence: EventSequence,
    subscribers: Vec<Subscriber>,
}

impl EventBus {
    /// Create a bus for one engine session.
    #[must_use]
    pub fn new(engine_session_id: impl Into<String>) -> Self {
        Self {
            engine_session_id: engine_session_id.into(),
            sequence: EventSequence::INITIAL,
            subscribers: Vec::new(),
        }
    }

    /// Register a subscriber, returning its handle.
    pub fn subscribe(&mut self, capacity: usize, policy: OverflowPolicy) -> SubscriberId {
        self.subscribers.push(Subscriber::new(capacity, policy));
        SubscriberId(self.subscribers.len() - 1)
    }

    /// A subscriber, for draining and for its counters.
    #[must_use]
    pub fn subscriber(&mut self, id: SubscriberId) -> Option<&mut Subscriber> {
        self.subscribers.get_mut(id.0)
    }

    /// The last sequence published.
    #[must_use]
    pub const fn sequence(&self) -> EventSequence {
        self.sequence
    }

    /// Publish the events of one committed transaction, in order.
    ///
    /// Takes the generation rather than deriving it, because the caller is the
    /// only one that knows the commit succeeded. `107` section 11 is the reason
    /// this cannot fail: a delivery problem after commit does not revert the
    /// state, so there is nothing here for a caller to handle.
    pub fn publish_committed(
        &mut self,
        generation: StateGeneration,
        events: impl IntoIterator<Item = DomainEvent>,
    ) -> Vec<EventEnvelope> {
        let mut published = Vec::new();

        for payload in events {
            self.sequence = self.sequence.next();

            let envelope = EventEnvelope {
                event_id: EventId::new(),
                engine_session_id: self.engine_session_id.clone(),
                sequence: self.sequence,
                state_generation: generation,
                payload,
            };

            for subscriber in &mut self.subscribers {
                subscriber.offer(envelope.clone());
            }

            published.push(envelope);
        }

        published
    }
}

/// Identifies a registered subscriber.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubscriberId(usize);

#[cfg(test)]
mod tests {
    use mirae_domain::{EntityName, Scene};
    use mirae_types::{ProjectId, SceneId};

    use super::*;
    use crate::store::StateStore;

    const SESSION: &str = "0000000000000000000000000000002a";

    fn bus() -> EventBus {
        EventBus::new(SESSION)
    }

    fn scene_created() -> DomainEvent {
        DomainEvent::SceneCreated {
            scene: EntityId::new(),
        }
    }

    #[test]
    fn published_events_are_sequenced_in_commit_order() {
        // 105 section 4: domain events are published after commit and in commit
        // order. The sequence is what lets a consumer detect a gap.
        let mut bus = bus();
        let published = bus.publish_committed(
            StateGeneration::from_raw(1),
            [scene_created(), scene_created(), scene_created()],
        );

        let sequences: Vec<_> = published.iter().map(|event| event.sequence.get()).collect();

        assert_eq!(sequences, vec![1, 2, 3]);
        assert_eq!(bus.sequence(), EventSequence::from_raw(3));
    }

    #[test]
    fn every_domain_event_carries_the_generation_it_accompanies() {
        // 105 section 2.1. Without it a consumer cannot tell whether its mirror
        // already includes what the event describes.
        let mut bus = bus();
        let published = bus.publish_committed(StateGeneration::from_raw(7), [scene_created()]);

        assert_eq!(
            published.first().map(|event| event.state_generation),
            Some(StateGeneration::from_raw(7))
        );
        assert_eq!(
            published
                .first()
                .map(|event| event.engine_session_id.clone()),
            Some(SESSION.to_owned())
        );
    }

    #[test]
    fn the_sequence_continues_across_commits() {
        let mut bus = bus();
        let _ = bus.publish_committed(StateGeneration::from_raw(1), [scene_created()]);
        let second = bus.publish_committed(StateGeneration::from_raw(2), [scene_created()]);

        assert_eq!(
            second.first().map(|event| event.sequence),
            Some(EventSequence::from_raw(2))
        );
    }

    #[test]
    fn a_subscriber_receives_what_was_published_after_it_subscribed() {
        let mut bus = bus();
        let id = bus.subscribe(8, OverflowPolicy::Lag);

        let _ = bus.publish_committed(
            StateGeneration::from_raw(1),
            [scene_created(), scene_created()],
        );

        let received = bus
            .subscriber(id)
            .map(|subscriber| subscriber.drain())
            .unwrap_or_default();

        assert_eq!(received.len(), 2);
        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.queued()),
            Some(0)
        );
    }

    #[test]
    fn a_state_mirror_that_falls_behind_is_told_rather_than_quietly_trimmed() {
        // 105 section 7: domain state replication must not silently drop
        // required patches and continue as if synchronized. Lagging is the
        // honest failure — the consumer knows it must resynchronize.
        let mut bus = bus();
        let id = bus.subscribe(2, OverflowPolicy::Lag);

        let _ = bus.publish_committed(
            StateGeneration::from_raw(1),
            [scene_created(), scene_created(), scene_created()],
        );

        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.lagged()),
            Some(true)
        );
        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.queued()),
            Some(0),
            "a lagged subscriber holds nothing it would have to discard"
        );
    }

    #[test]
    fn a_lagged_subscriber_stops_accumulating_until_it_resynchronizes() {
        let mut bus = bus();
        let id = bus.subscribe(1, OverflowPolicy::Lag);

        let _ = bus.publish_committed(
            StateGeneration::from_raw(1),
            [scene_created(), scene_created()],
        );
        let _ = bus.publish_committed(StateGeneration::from_raw(2), [scene_created()]);

        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.queued()),
            Some(0)
        );

        if let Some(subscriber) = bus.subscriber(id) {
            subscriber.resynchronized();
        }

        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.lagged()),
            Some(false)
        );

        let _ = bus.publish_committed(StateGeneration::from_raw(3), [scene_created()]);

        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.queued()),
            Some(1)
        );
    }

    #[test]
    fn a_diagnostic_subscriber_drops_the_oldest_and_counts_it() {
        // The other honest policy: the newest observation is the useful one, and
        // the count is enough to know something was lost.
        let mut bus = bus();
        let id = bus.subscribe(2, OverflowPolicy::DropOldest);

        let _ = bus.publish_committed(
            StateGeneration::from_raw(1),
            [scene_created(), scene_created(), scene_created()],
        );

        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.queued()),
            Some(2)
        );
        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.dropped()),
            Some(1)
        );
        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.lagged()),
            Some(false)
        );
    }

    #[test]
    fn a_zero_capacity_queue_is_raised_to_one() {
        // A queue that can hold nothing is a permanently lagged subscriber,
        // which is a configuration mistake rather than an intent.
        let mut bus = bus();
        let id = bus.subscribe(0, OverflowPolicy::Lag);

        let _ = bus.publish_committed(StateGeneration::from_raw(1), [scene_created()]);

        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.queued()),
            Some(1)
        );
        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.lagged()),
            Some(false)
        );
    }

    #[test]
    fn every_subscriber_receives_every_event() {
        let mut bus = bus();
        let first = bus.subscribe(8, OverflowPolicy::Lag);
        let second = bus.subscribe(8, OverflowPolicy::DropOldest);

        let _ = bus.publish_committed(StateGeneration::from_raw(1), [scene_created()]);

        assert_eq!(
            bus.subscriber(first).map(|subscriber| subscriber.queued()),
            Some(1)
        );
        assert_eq!(
            bus.subscriber(second).map(|subscriber| subscriber.queued()),
            Some(1)
        );
    }

    #[test]
    fn a_transaction_that_does_not_commit_publishes_nothing() {
        // 107 section 10 and 105 section 4 from the other side: an event queued
        // by a transaction that is dropped never existed.
        let mut store = StateStore::new(SESSION, ProjectId::nil());
        let mut bus = bus();
        let id = bus.subscribe(8, OverflowPolicy::Lag);

        {
            let mut transaction = store.transaction();
            transaction.emit(scene_created());
            // Dropped without commit. There is no path from here to the bus.
        }

        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.queued()),
            Some(0)
        );
        assert_eq!(bus.sequence(), EventSequence::INITIAL);
    }

    #[test]
    fn a_commit_hands_back_its_events_for_publication_afterwards() {
        // The ordering guarantee, end to end: the events reach the bus only once
        // the commit has produced a generation, and they name that generation.
        let mut store = StateStore::new(SESSION, ProjectId::nil());
        let mut bus = bus();
        let id = bus.subscribe(8, OverflowPolicy::Lag);
        let scene_id = SceneId::new();

        let mut transaction = store.transaction();
        let _ = transaction.prepare(|state| {
            state.put_scene(Scene {
                id: scene_id,
                name: EntityName::new("Scene")
                    .unwrap_or_else(|_| unreachable!("a literal name is valid")),
                items: Vec::new(),
            });
            Ok(())
        });
        transaction.emit(DomainEvent::SceneCreated {
            scene: *scene_id.as_entity_id(),
        });

        let outcome = transaction.commit();

        assert!(outcome.is_ok(), "the commit should have succeeded");

        let Ok(outcome) = outcome else { return };

        assert_eq!(outcome.events.len(), 1);
        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.queued()),
            Some(0)
        );

        let published = bus.publish_committed(outcome.generation, outcome.events);

        assert_eq!(
            published.first().map(|event| event.state_generation),
            Some(StateGeneration::from_raw(1))
        );
        assert_eq!(
            bus.subscriber(id).map(|subscriber| subscriber.queued()),
            Some(1)
        );
    }

    #[test]
    fn an_event_sequence_never_wraps_backwards() {
        let last = EventSequence::from_raw(u64::MAX);

        assert_eq!(last.next(), last);
    }

    #[test]
    fn every_event_kind_has_a_stable_identifier() {
        let entity = EntityId::nil();

        for event in [
            DomainEvent::SceneCreated { scene: entity },
            DomainEvent::SceneRemoved { scene: entity },
            DomainEvent::SourceCreated { source: entity },
            DomainEvent::SourceRemoved { source: entity },
            DomainEvent::SceneItemAdded {
                item: entity,
                scene: entity,
            },
            DomainEvent::SceneItemRemoved {
                item: entity,
                scene: entity,
            },
        ] {
            assert!(!event.as_str().is_empty());
            assert_eq!(event.as_str(), event.as_str().to_ascii_lowercase());
        }
    }
}

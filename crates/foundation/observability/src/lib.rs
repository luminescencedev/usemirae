//! Structured tracing: identity, fields, volume control, and bounded local logs.
//!
//! Canonical documentation: `docs/06-quality/606-logging-and-tracing.md` and
//! ADR-0041. Dependency rules: `docs/08-development/804-dependency-rules.md`.
//!
//! Foundation layer: `std` and `mirae-errors` only, so any crate can emit events.
//!
//! # Shape of an event
//!
//! Every event carries the wall clock, monotonic nanoseconds since process start,
//! severity, subsystem, a stable event name, the engine session id, the process
//! role, the build id, the thread name, an optional correlation id, and typed
//! fields with declared redaction classes (`606` section 2). Events are written as
//! one JSON object per line, so separate per-process files can be merged by
//! tooling (`606` section 7).
//!
//! # What is enforced here
//!
//! - secret and media-content fields are never written, and each attempt is
//!   counted ([`Tracer::rejected_fields`]);
//! - private fields are hashed rather than written;
//! - repeated identical events are suppressed and rate limited (`606` section 6);
//! - log files are size- and count-bounded, and a write failure is counted rather
//!   than propagated (`606` section 9).
//!
//! # Real-time paths
//!
//! Audio and capture callbacks must not format or write synchronously
//! (`606` section 8). They use [`RealtimeCounters`], which only touches atomics; a
//! later, non-real-time tick emits one event carrying the totals.

pub mod field;
pub mod session;
pub mod sink;
pub mod volume;

pub use field::{Field, FieldValue, RedactionClass};
pub use session::{ClockOrigin, EngineSessionId, ProcessIdentity, ProcessRole};
pub use sink::{FailingSink, MemorySink, NullSink, RetentionPolicy, RollingFileSink, Sink};
pub use volume::{Decision, VolumeControl};

use core::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use mirae_errors::{CorrelationId, SubsystemId};

use field::stable_hash;

/// Maximum number of fields kept on one event.
pub const MAX_EVENT_FIELDS: usize = 24;

/// Severity of an event (`606` section 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// High-volume detail, off unless a bounded diagnostic mode is enabled.
    Trace,
    /// Developer-facing detail, off by default in production.
    Debug,
    /// Normal lifecycle milestones.
    Info,
    /// Something unexpected that did not fail the operation.
    Warn,
    /// The operation failed.
    Error,
}

impl Level {
    /// A stable identifier for tooling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Lock-free counters for real-time callbacks.
///
/// Audio and capture callbacks increment these and return; a later tick reads them
/// and emits one ordinary event. Nothing here formats, allocates, or writes.
#[derive(Debug, Default)]
pub struct RealtimeCounters {
    processed: AtomicU64,
    dropped: AtomicU64,
    underruns: AtomicU64,
}

impl RealtimeCounters {
    /// New counters, all zero.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            processed: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
            underruns: AtomicU64::new(0),
        }
    }

    /// Record processed units. Safe to call from a real-time callback.
    pub fn record_processed(&self, count: u64) {
        self.processed.fetch_add(count, Ordering::Relaxed);
    }

    /// Record dropped units. Safe to call from a real-time callback.
    pub fn record_dropped(&self, count: u64) {
        self.dropped.fetch_add(count, Ordering::Relaxed);
    }

    /// Record an underrun. Safe to call from a real-time callback.
    pub fn record_underrun(&self) {
        self.underruns.fetch_add(1, Ordering::Relaxed);
    }

    /// Read and reset every counter, for the tick that emits them.
    #[must_use]
    pub fn take(&self) -> RealtimeSnapshot {
        RealtimeSnapshot {
            processed: self.processed.swap(0, Ordering::Relaxed),
            dropped: self.dropped.swap(0, Ordering::Relaxed),
            underruns: self.underruns.swap(0, Ordering::Relaxed),
        }
    }
}

/// Counter values read by a non-real-time tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RealtimeSnapshot {
    /// Units processed since the last read.
    pub processed: u64,
    /// Units dropped since the last read.
    pub dropped: u64,
    /// Underruns since the last read.
    pub underruns: u64,
}

impl RealtimeSnapshot {
    /// Whether anything happened worth emitting.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.processed == 0 && self.dropped == 0 && self.underruns == 0
    }
}

/// Writes structured events to a sink, subject to volume control.
pub struct Tracer {
    identity: ProcessIdentity,
    clock: ClockOrigin,
    minimum_level: Level,
    sink: Box<dyn Sink>,
    volume: VolumeControl,
    emitted: u64,
    rejected_fields: u64,
    dropped_fields: u64,
}

impl fmt::Debug for Tracer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Tracer")
            .field("identity", &self.identity)
            .field("minimum_level", &self.minimum_level)
            .field("emitted", &self.emitted)
            .field("rejected_fields", &self.rejected_fields)
            .finish_non_exhaustive()
    }
}

impl Tracer {
    /// Build a tracer.
    #[must_use]
    pub fn new(
        identity: ProcessIdentity,
        clock: ClockOrigin,
        minimum_level: Level,
        sink: Box<dyn Sink>,
        volume: VolumeControl,
    ) -> Self {
        Self {
            identity,
            clock,
            minimum_level,
            sink,
            volume,
            emitted: 0,
            rejected_fields: 0,
            dropped_fields: 0,
        }
    }

    /// The identity stamped on every event.
    #[must_use]
    pub const fn identity(&self) -> ProcessIdentity {
        self.identity
    }

    /// Adopt a session id learned after startup, such as from a handshake.
    pub fn adopt_session(&mut self, session: EngineSessionId) {
        self.identity = self.identity.with_session(session);
    }

    /// How many events were written.
    #[must_use]
    pub const fn emitted(&self) -> u64 {
        self.emitted
    }

    /// How many fields were refused because their class forbids logging.
    #[must_use]
    pub const fn rejected_fields(&self) -> u64 {
        self.rejected_fields
    }

    /// How many fields were dropped because the per-event bound was reached.
    #[must_use]
    pub const fn dropped_fields(&self) -> u64 {
        self.dropped_fields
    }

    /// How many lines the sink failed to store.
    #[must_use]
    pub fn failed_writes(&self) -> u64 {
        self.sink.failed_writes()
    }

    /// The most recently written line, when the sink retains lines.
    #[must_use]
    pub fn last_line(&self) -> Option<&str> {
        self.sink.last_line()
    }

    /// Volume-control counters.
    #[must_use]
    pub const fn volume(&self) -> &VolumeControl {
        &self.volume
    }

    /// Start an event. Nothing is written until [`EventBuilder::emit`].
    #[must_use]
    pub fn event(
        &mut self,
        level: Level,
        subsystem: SubsystemId,
        name: &'static str,
    ) -> EventBuilder<'_> {
        EventBuilder {
            tracer: self,
            level,
            subsystem,
            name,
            correlation: CorrelationId::NONE,
            fields: Vec::new(),
            rejected: 0,
            dropped: 0,
        }
    }

    /// Emit the counters a real-time callback accumulated.
    ///
    /// Does nothing when no activity was recorded, so an idle pipeline is silent.
    pub fn emit_realtime(
        &mut self,
        subsystem: SubsystemId,
        name: &'static str,
        snapshot: RealtimeSnapshot,
    ) -> bool {
        if snapshot.is_empty() {
            return false;
        }

        self.event(Level::Debug, subsystem, name)
            .field(Field::public(
                "processed",
                FieldValue::Unsigned(snapshot.processed),
            ))
            .field(Field::public(
                "dropped",
                FieldValue::Unsigned(snapshot.dropped),
            ))
            .field(Field::public(
                "underruns",
                FieldValue::Unsigned(snapshot.underruns),
            ))
            .emit()
    }
}

/// Builds one event.
pub struct EventBuilder<'tracer> {
    tracer: &'tracer mut Tracer,
    level: Level,
    subsystem: SubsystemId,
    name: &'static str,
    correlation: CorrelationId,
    fields: Vec<(&'static str, String)>,
    rejected: u64,
    dropped: u64,
}

impl EventBuilder<'_> {
    /// Attach the correlation id of the request, command, or frame.
    #[must_use]
    pub const fn correlation(mut self, correlation: CorrelationId) -> Self {
        self.correlation = correlation;
        self
    }

    /// Add a field.
    ///
    /// A secret or media-content field is refused and counted; a field past the
    /// per-event bound is dropped and counted.
    #[must_use]
    pub fn field(mut self, field: Field) -> Self {
        let Some(value) = field.rendered_value() else {
            self.rejected = self.rejected.saturating_add(1);
            return self;
        };

        if self.fields.len() >= MAX_EVENT_FIELDS {
            self.dropped = self.dropped.saturating_add(1);
            return self;
        }

        self.fields.push((field.name(), value));
        self
    }

    /// Write the event. Returns whether it reached the sink.
    pub fn emit(self) -> bool {
        let Self {
            tracer,
            level,
            subsystem,
            name,
            correlation,
            fields,
            rejected,
            dropped,
        } = self;

        tracer.rejected_fields = tracer.rejected_fields.saturating_add(rejected);
        tracer.dropped_fields = tracer.dropped_fields.saturating_add(dropped);

        if level < tracer.minimum_level {
            return false;
        }

        // The signature distinguishes an identical repeat from the same event
        // about a different entity.
        let mut signature = u64::from(level as u8);
        for (key, value) in &fields {
            signature ^= stable_hash(key) ^ stable_hash(value);
        }

        let now = Instant::now();
        if tracer.volume.admit(name, signature, now) != Decision::Emit {
            return false;
        }

        let line = render_line(
            tracer,
            now,
            level,
            subsystem,
            name,
            correlation,
            &fields,
            dropped,
        );

        if tracer.sink.write_line(&line) {
            tracer.emitted = tracer.emitted.saturating_add(1);
            return true;
        }

        false
    }
}

/// Escape a string for embedding in a JSON value.
fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other if (other as u32) < 0x20 => escaped.push(' '),
            other => escaped.push(other),
        }
    }

    escaped
}

/// Render one event as a single JSON object.
#[allow(clippy::too_many_arguments)]
fn render_line(
    tracer: &Tracer,
    now: Instant,
    level: Level,
    subsystem: SubsystemId,
    name: &'static str,
    correlation: CorrelationId,
    fields: &[(&'static str, String)],
    dropped_fields: u64,
) -> String {
    let identity = tracer.identity;
    let mut line = String::with_capacity(256);

    line.push('{');
    line.push_str(&format!(
        "\"timestamp_unix_ms\":{},\"monotonic_ns\":{},",
        tracer.clock.unix_millis_at(now),
        tracer.clock.monotonic_nanos_at(now)
    ));
    line.push_str(&format!(
        "\"level\":\"{}\",\"subsystem\":\"{}\",\"event\":\"{}\",",
        level,
        subsystem,
        escape(name)
    ));
    line.push_str(&format!(
        "\"session\":\"{}\",\"role\":\"{}\",\"build\":\"{}\",",
        identity.session(),
        identity.role(),
        escape(identity.build_id())
    ));
    line.push_str(&format!(
        "\"thread\":\"{}\"",
        escape(std::thread::current().name().unwrap_or("unnamed"))
    ));

    if !correlation.is_none() {
        line.push_str(&format!(",\"correlation\":\"{correlation}\""));
    }

    if dropped_fields != 0 {
        line.push_str(&format!(",\"dropped_fields\":{dropped_fields}"));
    }

    if !fields.is_empty() {
        line.push_str(",\"fields\":{");
        for (index, (key, value)) in fields.iter().enumerate() {
            if index != 0 {
                line.push(',');
            }
            line.push_str(&format!("\"{}\":\"{}\"", escape(key), escape(value)));
        }
        line.push('}');
    }

    line.push('}');
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn tracer(sink: Box<dyn Sink>) -> Tracer {
        Tracer::new(
            ProcessIdentity::new(
                EngineSessionId::from_u128(0x2a),
                ProcessRole::Engine,
                "test-build",
            ),
            ClockOrigin::from_parts(1_700_000_000_000, Instant::now()),
            Level::Debug,
            sink,
            VolumeControl::new(16, Duration::from_secs(1)),
        )
    }

    fn emit_one(tracer: &mut Tracer) -> bool {
        tracer
            .event(Level::Info, SubsystemId::Runtime, "engine.started")
            .field(Field::public("attempt", FieldValue::Unsigned(1)))
            .emit()
    }

    /// Read a JSON string value out of a rendered line.
    fn value_of<'line>(line: &'line str, key: &str) -> Option<&'line str> {
        let needle = format!("\"{key}\":\"");
        let start = line.find(&needle)? + needle.len();
        let rest = &line[start..];
        let end = rest.find('"')?;

        Some(&rest[..end])
    }

    #[test]
    fn an_event_carries_the_documented_structure() {
        let mut tracer = tracer(Box::new(MemorySink::new(8)));

        assert!(
            tracer
                .event(Level::Warn, SubsystemId::Ipc, "ipc.handshake_rejected")
                .correlation(CorrelationId::from_u128(255))
                .field(Field::public("protocol_major", FieldValue::Unsigned(1)))
                .field(Field::internal("phase", FieldValue::Label("hello")))
                .emit()
        );

        let line = tracer.last_line().unwrap_or_default().to_owned();

        assert!(line.starts_with('{') && line.ends_with('}'));
        assert_eq!(value_of(&line, "level"), Some("warn"));
        assert_eq!(value_of(&line, "subsystem"), Some("ipc"));
        assert_eq!(value_of(&line, "event"), Some("ipc.handshake_rejected"));
        assert_eq!(
            value_of(&line, "session"),
            Some("0000000000000000000000000000002a")
        );
        assert_eq!(value_of(&line, "role"), Some("engine"));
        assert_eq!(value_of(&line, "build"), Some("test-build"));
        assert_eq!(
            value_of(&line, "correlation"),
            Some("000000000000000000000000000000ff")
        );
        assert!(line.contains("\"timestamp_unix_ms\":"));
        assert!(line.contains("\"monotonic_ns\":"));
        assert!(line.contains("\"protocol_major\":\"1\""));
    }

    #[test]
    fn a_secret_field_is_refused_and_counted() {
        let mut tracer = tracer(Box::new(MemorySink::new(8)));

        assert!(
            tracer
                .event(Level::Error, SubsystemId::Output, "output.auth_failed")
                .field(Field::secret(
                    "stream_key",
                    FieldValue::Label("live_super_secret")
                ))
                .field(Field::media_content("frame", FieldValue::Unsigned(9)))
                .field(Field::public("attempt", FieldValue::Unsigned(2)))
                .emit()
        );

        assert_eq!(tracer.rejected_fields(), 2);
        assert!(
            !tracer
                .last_line()
                .unwrap_or_default()
                .contains("live_super_secret")
        );
        assert!(
            tracer
                .last_line()
                .unwrap_or_default()
                .contains("\"attempt\":\"2\"")
        );
    }

    #[test]
    fn a_private_field_is_hashed_in_the_line() {
        let mut tracer = tracer(Box::new(MemorySink::new(8)));

        tracer
            .event(Level::Info, SubsystemId::Project, "project.opened")
            .field(Field::private("account", FieldValue::Label("arthur")))
            .emit();

        let line = tracer.last_line().unwrap_or_default();

        assert!(line.contains("hashed:"));
        assert!(!line.contains("arthur"));
    }

    #[test]
    fn fields_are_bounded_per_event() {
        let mut tracer = tracer(Box::new(MemorySink::new(8)));
        let mut builder = tracer.event(Level::Info, SubsystemId::Runtime, "engine.bulk");

        for _ in 0..(MAX_EVENT_FIELDS + 6) {
            builder = builder.field(Field::public("f", FieldValue::Unsigned(1)));
        }
        builder.emit();

        assert_eq!(tracer.dropped_fields(), 6);
        assert!(
            tracer
                .last_line()
                .unwrap_or_default()
                .contains("\"dropped_fields\":6")
        );
    }

    #[test]
    fn levels_below_the_minimum_are_not_written() {
        let mut tracer = tracer(Box::new(MemorySink::new(8)));

        assert!(
            !tracer
                .event(Level::Trace, SubsystemId::Runtime, "engine.tick")
                .emit()
        );
        assert_eq!(tracer.emitted(), 0);
    }

    #[test]
    fn an_identical_repeat_is_suppressed() {
        let mut tracer = tracer(Box::new(MemorySink::new(32)));

        assert!(emit_one(&mut tracer));
        assert!(!emit_one(&mut tracer));
        assert_eq!(tracer.emitted(), 1);
        assert_eq!(tracer.volume().suppressed_duplicates(), 1);
    }

    #[test]
    fn a_storm_is_rate_limited() {
        let mut tracer = Tracer::new(
            ProcessIdentity::new(EngineSessionId::NONE, ProcessRole::Test, "test"),
            ClockOrigin::from_parts(0, Instant::now()),
            Level::Debug,
            Box::new(MemorySink::new(1_000)),
            VolumeControl::new(3, Duration::from_secs(60)),
        );

        for attempt in 0..50 {
            tracer
                .event(Level::Error, SubsystemId::Output, "output.retry")
                .field(Field::public("attempt", FieldValue::Unsigned(attempt)))
                .emit();
        }

        assert_eq!(tracer.emitted(), 3, "the budget must cap a storm");
        assert!(tracer.volume().rate_limited() >= 40);
    }

    #[test]
    fn a_failing_sink_is_counted_and_never_panics() {
        let mut tracer = tracer(Box::new(FailingSink::default()));

        assert!(!emit_one(&mut tracer));
        assert_eq!(tracer.emitted(), 0);
        assert_eq!(tracer.failed_writes(), 1);
    }

    #[test]
    fn realtime_counters_do_not_write_until_a_tick_emits_them() {
        let counters = RealtimeCounters::new();
        counters.record_processed(480);
        counters.record_dropped(2);
        counters.record_underrun();

        let mut tracer = tracer(Box::new(MemorySink::new(8)));

        assert_eq!(tracer.emitted(), 0, "callbacks must not write");

        let snapshot = counters.take();

        assert_eq!(snapshot.processed, 480);
        assert_eq!(snapshot.dropped, 2);
        assert_eq!(snapshot.underruns, 1);
        assert!(tracer.emit_realtime(SubsystemId::Audio, "audio.tick", snapshot));
        assert_eq!(tracer.emitted(), 1);

        // Taking resets, so an idle pipeline emits nothing.
        assert!(counters.take().is_empty());
        assert!(!tracer.emit_realtime(SubsystemId::Audio, "audio.tick", counters.take()));
    }

    #[test]
    fn a_session_learned_later_appears_on_subsequent_events() {
        let mut tracer = Tracer::new(
            ProcessIdentity::new(EngineSessionId::NONE, ProcessRole::Shell, "test"),
            ClockOrigin::from_parts(0, Instant::now()),
            Level::Debug,
            Box::new(MemorySink::new(8)),
            VolumeControl::new(8, Duration::from_secs(1)),
        );

        tracer.adopt_session(EngineSessionId::from_u128(7));
        tracer
            .event(Level::Info, SubsystemId::Ipc, "ipc.connected")
            .emit();

        assert_eq!(
            value_of(tracer.last_line().unwrap_or_default(), "session"),
            Some("00000000000000000000000000000007")
        );
    }

    #[test]
    fn lines_from_two_processes_merge_by_session() {
        let build = |role: ProcessRole| {
            let mut tracer = Tracer::new(
                ProcessIdentity::new(EngineSessionId::from_u128(5), role, "test"),
                ClockOrigin::from_parts(1_000, Instant::now()),
                Level::Debug,
                Box::new(MemorySink::new(4)),
                VolumeControl::new(4, Duration::from_secs(1)),
            );
            tracer
                .event(Level::Info, SubsystemId::Runtime, "process.started")
                .emit();
            tracer.last_line().unwrap_or_default().to_owned()
        };

        let engine = build(ProcessRole::Engine);
        let shell = build(ProcessRole::Shell);

        // Same session, different roles: tooling can merge them and tell them
        // apart (606 section 7).
        assert_eq!(value_of(&engine, "session"), value_of(&shell, "session"));
        assert_eq!(value_of(&engine, "role"), Some("engine"));
        assert_eq!(value_of(&shell, "role"), Some("shell"));
    }

    #[test]
    fn control_characters_cannot_break_the_line_format() {
        let mut tracer = tracer(Box::new(MemorySink::new(8)));

        tracer
            .event(Level::Info, SubsystemId::Runtime, "engine.note")
            .field(Field::text(
                "detail",
                RedactionClass::Internal,
                "line\nbreak\tand \"quotes\"",
            ))
            .emit();

        let line = tracer.last_line().unwrap_or_default();

        assert_eq!(line.lines().count(), 1, "an event must stay on one line");
        assert!(line.contains("\\\"quotes\\\""));
    }
}

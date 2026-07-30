//! Minimal engine process entry point.
//!
//! Canonical documentation: `docs/01-runtime/102-engine-lifecycle.md`,
//! `docs/01-runtime/101-process-model.md`,
//! `docs/08-development/802-rust-workspace-and-crates.md`.
//!
//! This binary assembles crates and contains no domain logic (`801` section 4).
//! It bootstraps identity and logging, registers the services that exist today,
//! runs the lifecycle to `Ready`, publishes readiness, then shuts down cleanly.
//!
//! No IPC server is listening yet, so the process starts, reports, and stops
//! rather than waiting for connections. `MIR-0012` adds the authenticated
//! handshake and makes the process wait; `MIR-0010` adds the shell that supervises
//! it.

use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mirae_contracts::generated::{EngineReadinessState, Hello, Reject, RejectReason};
use mirae_observability::{
    ClockOrigin, EngineSessionId, Level, ProcessIdentity, ProcessRole, RetentionPolicy,
    RollingFileSink, Tracer, VolumeControl,
};
use mirae_runtime::ipc::{self, HandshakeOutcome};
use mirae_runtime::{Engine, Requirement, ServiceOutcome, StubService};

/// Process role reported to diagnostics and to peers.
const ROLE: ProcessRole = ProcessRole::Engine;

/// Build identity stamped on every event.
const BUILD_ID: &str = concat!("mirae-engine@", env!("CARGO_PKG_VERSION"));

/// How many events one name may emit per window before it is rate limited.
const EVENT_BUDGET: u32 = 64;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();

    if arguments
        .iter()
        .any(|argument| argument == "--version" || argument == "-V")
    {
        let mut out = std::io::stdout().lock();
        let _ = writeln!(
            out,
            "{name} {version} role={role} protocol={major}.{minor}",
            name = env!("CARGO_PKG_NAME"),
            version = env!("CARGO_PKG_VERSION"),
            role = ROLE,
            major = mirae_contracts::generated::PROTOCOL_VERSION_MAJOR,
            minor = mirae_contracts::generated::PROTOCOL_VERSION_MINOR,
        );
        return ExitCode::SUCCESS;
    }

    let supervised = arguments.iter().any(|argument| argument == SUPERVISED_FLAG);

    if let Some(unknown) = arguments
        .iter()
        .find(|argument| argument.starts_with('-') && *argument != SUPERVISED_FLAG)
    {
        let mut err = std::io::stderr().lock();
        let _ = writeln!(err, "unknown argument `{unknown}`; try --version");
        return ExitCode::FAILURE;
    }

    run(supervised)
}

/// Environment variable carrying the launch credential from the shell.
const CREDENTIAL_VARIABLE: &str = "MIRAE_LAUNCH_CREDENTIAL";

/// Flag that makes the engine stay alive until its parent closes stdin.
const SUPERVISED_FLAG: &str = "--supervised";

/// Bootstrap, initialize, report readiness, and shut down.
fn run(supervised: bool) -> ExitCode {
    // 102 invariant 8: a new engine process creates a new session.
    let session = new_session_id();
    let identity = ProcessIdentity::new(session, ROLE, BUILD_ID);
    let sink = RollingFileSink::new(&log_directory(), "engine", RetentionPolicy::DEFAULT);
    let tracer = Tracer::new(
        identity,
        ClockOrigin::now(),
        Level::Info,
        Box::new(sink),
        VolumeControl::new(EVENT_BUDGET, Duration::from_secs(10)),
    );

    let mut engine = Engine::new(tracer, session);
    register_services(&mut engine);

    let startup = engine.start();

    if let Err(failure) = startup {
        // The failure is already an event; stderr carries the safe message so a
        // supervising shell sees it without reading the log file.
        let mut err = std::io::stderr().lock();
        let _ = writeln!(
            err,
            "engine failed to start: {} ({})",
            failure.error.safe_message(),
            failure.error.code()
        );

        engine.shutdown();
        return ExitCode::FAILURE;
    }

    if supervised {
        // Supervised: the handshake is how readiness is reported, so stdout
        // carries frames only. An unsupervised run still prints the line, which
        // keeps the binary inspectable by hand.
        // The inherited stdio pair is the transport (`108` section 3 lists an
        // inherited secure channel). No command is served before the handshake
        // completes, and today the handshake is the whole protocol.
        serve_handshake(&engine);

        // The shell owns the lifetime: it closes stdin to request shutdown, which
        // is the cooperative half of `501` section 6 point 7. Killing the process
        // is the supervisor's fallback, not its first move.
        wait_for_parent_shutdown();
    } else {
        publish_readiness(&engine);
    }

    engine.shutdown();

    ExitCode::SUCCESS
}

/// Answer exactly one `Hello` on the inherited channel.
///
/// The credential comes from the environment variable the shell set. It is read
/// once and removed from the environment, so it cannot reach a child process or a
/// crash dump of this one.
fn serve_handshake(engine: &Engine) {
    let expected = std::env::var(CREDENTIAL_VARIABLE).unwrap_or_default();
    // SAFETY-adjacent: removing it narrows the window in which the secret exists.
    unsafe { std::env::remove_var(CREDENTIAL_VARIABLE) };

    let Some(expected) = ipc::decode_hex(&expected) else {
        let mut err = std::io::stderr().lock();
        let _ = writeln!(err, "no usable launch credential; refusing connections");
        return;
    };

    let mut input = std::io::stdin().lock();
    let Ok((header, payload)) = ipc::read_frame(&mut input, ipc::DEFAULT_MAX_FRAME_BYTES) else {
        // A peer that cannot frame a message gets no answer and no detail.
        return;
    };

    if header.message_type != ipc::MessageType::Hello {
        return;
    }

    let session = engine.readiness().engine_session_id;
    let ready = engine.state().accepts_connections();

    let (message_type, body) = match serde_json::from_slice::<Hello>(&payload) {
        Ok(hello) => match ipc::evaluate_hello(&hello, &expected, &session, ready) {
            HandshakeOutcome::Accepted(welcome) => (
                ipc::MessageType::Welcome,
                serde_json::to_vec(&welcome).unwrap_or_default(),
            ),
            HandshakeOutcome::Refused(reject) => (
                ipc::MessageType::Reject,
                serde_json::to_vec(&reject).unwrap_or_default(),
            ),
        },
        Err(_) => (
            ipc::MessageType::Reject,
            serde_json::to_vec(&Reject {
                reason: RejectReason::MalformedHello,
                detail: "the hello message did not match its contract".to_owned(),
                protocol_major: mirae_contracts::generated::PROTOCOL_VERSION_MAJOR,
            })
            .unwrap_or_default(),
        ),
    };

    let response = ipc::FrameHeader {
        protocol_major: mirae_contracts::generated::PROTOCOL_VERSION_MAJOR,
        protocol_minor: mirae_contracts::generated::PROTOCOL_VERSION_MINOR,
        message_type,
        flags: 0,
        payload_length: u32::try_from(body.len()).unwrap_or(u32::MAX),
        correlation_id: header.correlation_id,
    };

    let mut output = std::io::stdout().lock();
    let _ = ipc::write_frame(&mut output, &response, &body);
}

/// Block until stdin reaches end of file.
///
/// Reading, rather than sleeping, means the engine notices immediately when the
/// shell exits or closes the pipe, including when the shell dies unexpectedly.
fn wait_for_parent_shutdown() {
    let mut discard = String::new();
    let stdin = std::io::stdin();

    loop {
        discard.clear();
        match stdin.read_line(&mut discard) {
            // End of file: the shell closed the pipe or went away.
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
    }
}

/// Register the services that exist today, in the order `102` section 5 fixes.
///
/// The remaining subsystems from that list are absent rather than stubbed as
/// working: an optional service that reports `Unavailable` says so honestly and
/// leaves the engine impaired, which is what a missing capability means.
fn register_services(engine: &mut Engine) {
    engine.register(Box::new(StubService::ready(
        "configuration",
        Requirement::Mandatory,
    )));
    engine.register(Box::new(StubService::ready(
        "diagnostics",
        Requirement::Mandatory,
    )));
    engine.register(Box::new(StubService::ready(
        "ipc_server",
        Requirement::Mandatory,
    )));
    engine.register(Box::new(StubService::with_outcome(
        "platform_capabilities",
        Requirement::Optional,
        ServiceOutcome::Unavailable("the capability probe arrives with the platform work"),
    )));
}

/// Write the readiness contract to stdout as one JSON line.
///
/// A supervising shell reads this until the IPC handshake exists. The field names
/// come from the generated contract, so they cannot drift from the schema.
fn publish_readiness(engine: &Engine) {
    let readiness = engine.readiness();
    let state = match readiness.state {
        EngineReadinessState::Starting => "starting",
        EngineReadinessState::Ready => "ready",
        EngineReadinessState::Degraded => "degraded",
        EngineReadinessState::Stopping => "stopping",
        EngineReadinessState::Stopped => "stopped",
    };

    let mut line = format!(
        "{{\"state\":\"{state}\",\"protocolMajor\":{},\"protocolMinor\":{},\
         \"engineSessionId\":\"{}\"",
        readiness.protocol_major, readiness.protocol_minor, readiness.engine_session_id
    );

    if let Some(detail) = readiness.detail.as_deref() {
        line.push_str(&format!(",\"detail\":\"{}\"", detail.replace('"', "'")));
    }
    line.push('}');

    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{line}");
}

/// Where rolling logs are written.
///
/// A machine-local directory, never shared with user projects or recordings
/// (`606` section 9). The platform layer will own this choice; until then the
/// system temporary directory keeps it off the user's project paths.
fn log_directory() -> PathBuf {
    let mut directory = std::env::temp_dir();
    directory.push("mirae");
    directory.push("logs");
    directory
}

/// Derive a session id without a random-number dependency.
///
/// Mixes the wall clock with the process id, which is enough to distinguish
/// concurrent runs on one machine. A cryptographically random id belongs to the
/// platform layer, alongside the launch credentials in `MIR-0012`.
fn new_session_id() -> EngineSessionId {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    let process_id = u128::from(std::process::id());

    EngineSessionId::from_u128(nanos.wrapping_mul(0x1_0000_0001).wrapping_add(process_id))
}

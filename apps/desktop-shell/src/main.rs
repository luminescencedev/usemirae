//! Native shell entry point and UI host.
//!
//! Canonical documentation: `docs/05-platform/501-desktop-shell.md`,
//! `docs/01-runtime/101-process-model.md`, ADR-0037.
//!
//! The shell is intentionally thin (`501` section 1): it launches and supervises
//! the engine and hosts the control UI. It never owns project state and never
//! reports engine state it cannot currently observe.
//!
//! # What exists today
//!
//! Launch, readiness handoff, supervision, and bounded restart, and the main
//! control window hosting the packaged control UI in the operating system's own
//! webview (ADR-0068). The supervision logic lives in `mirae-runtime` so it is
//! testable without spawning processes; this binary provides the real process
//! launcher, the command line, and the window.
//!
//! # What does not
//!
//! The other window roles in `501` section 5 — projector, detached panels,
//! startup and recovery windows — and the typed bridge that will carry commands
//! between the page and the engine. The webview today loads packaged resources
//! and nothing else reaches it.
//!
//! Authentication of the handoff is `MIR-0012`: the credential is created and
//! passed, but the engine does not verify it yet.

mod assets;
mod bridge;
mod external;
mod navigation;
mod ui_host;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::assets::UiResources;
use crate::bridge::EngineView;
use crate::ui_host::{EngineHealth, FatalError};

use mirae_contracts::generated::{
    EngineReadiness, Hello, HelloRole, PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR, Reject,
    Welcome,
};
use mirae_runtime::ipc;
use mirae_runtime::supervisor::{
    EngineLauncher, LaunchCredential, RestartPolicy, SupervisionState, Supervisor,
};

/// How long to wait for the engine to report readiness before giving up.
const READINESS_TIMEOUT: Duration = Duration::from_secs(10);

/// How often the supervisor polls the engine.
const POLL_INTERVAL: Duration = Duration::from_millis(25);

/// How long the engine may take to stop after being asked, before it is killed.
const SHUTDOWN_DEADLINE: Duration = Duration::from_secs(5);

/// Environment variable naming the engine executable.
const ENGINE_PATH_VARIABLE: &str = "MIRAE_ENGINE_PATH";

/// Environment variable carrying the launch credential to the engine.
///
/// An environment variable keeps the secret off the command line, where other
/// users can read it from the process table. `MIR-0012` replaces this with an
/// inherited channel, which is better still.
const CREDENTIAL_VARIABLE: &str = "MIRAE_LAUNCH_CREDENTIAL";

fn main() -> std::process::ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();

    if arguments
        .iter()
        .any(|argument| argument == "--version" || argument == "-V")
    {
        let mut out = std::io::stdout().lock();
        let _ = writeln!(
            out,
            "{name} {version} role=shell",
            name = env!("CARGO_PKG_NAME"),
            version = env!("CARGO_PKG_VERSION"),
        );
        return std::process::ExitCode::SUCCESS;
    }

    if let Some(unknown) = arguments.iter().find(|argument| argument.starts_with('-')) {
        let mut err = std::io::stderr().lock();
        let _ = writeln!(err, "unknown argument `{unknown}`; try --version");
        return std::process::ExitCode::FAILURE;
    }

    run()
}

/// Launch, supervise, host the control UI, and stop.
fn run() -> std::process::ExitCode {
    match session() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(failure) => {
            let mut err = std::io::stderr().lock();
            let _ = writeln!(err, "{}", failure.report());
            std::process::ExitCode::FAILURE
        }
    }
}

/// One shell session, from engine launch to window close.
///
/// The order is `501` section 6: credential, launch, authenticated readiness,
/// then the UI. The packaged resources are located first because failing to find
/// them is a UI failure that should not cost the user an engine launch.
fn session() -> Result<(), FatalError> {
    let resources = UiResources::locate().map_err(FatalError::Ui)?;

    {
        // Which directory is being served answers most "why is the UI stale"
        // questions before they are asked. It is a path, not a secret.
        let mut out = std::io::stdout().lock();
        let _ = writeln!(out, "control UI served from {}", resources.root().display());
    }

    let Some(engine_path) = engine_path() else {
        return Err(FatalError::Engine(format!(
            "could not locate the engine executable; set {ENGINE_PATH_VARIABLE}"
        )));
    };

    let credential = LaunchCredential::placeholder(seed());
    let mut supervisor = Supervisor::new(ProcessLauncher::new(engine_path), RestartPolicy::DEFAULT);

    if !supervisor.start(&credential, Instant::now()) {
        return Err(FatalError::Engine(format!(
            "could not launch the engine: {}",
            supervisor.last_reason().unwrap_or("no reason given")
        )));
    }

    // 501 section 6 point 3: wait for *authenticated* readiness. The handshake is
    // both: the engine answers only once it is accepting connections, and the
    // Welcome proves this shell is allowed to talk to it.
    let deadline = Instant::now() + READINESS_TIMEOUT;
    let mut outcome: Result<Welcome, String> = Err("the engine never answered".to_owned());

    while Instant::now() < deadline {
        let state = supervisor.poll(&credential, Instant::now());

        if matches!(state, SupervisionState::GaveUp | SupervisionState::Stopped) {
            break;
        }

        outcome = supervisor.launcher_mut().handshake(&credential);

        match outcome {
            Ok(_) => break,
            // A closed channel means the engine has not opened it yet; anything
            // else is a real refusal and must not be retried into a loop.
            Err(ref reason) if reason.contains("frame ended early") => {
                std::thread::sleep(POLL_INTERVAL);
            }
            Err(_) => break,
        }
    }

    let (session_id, protocol_major, protocol_minor) = match outcome {
        Ok(welcome) => {
            let mut out = std::io::stdout().lock();
            let _ = writeln!(
                out,
                "handshake accepted: protocol={}.{} session={} max_frame={} launches={}",
                welcome.protocol_major,
                welcome.protocol_minor,
                welcome.engine_session_id,
                welcome.max_frame_bytes,
                supervisor.launches()
            );

            (
                welcome.engine_session_id,
                welcome.protocol_major,
                welcome.protocol_minor,
            )
        }
        Err(reason) => {
            let state = supervisor.state();
            supervisor.stop();

            return Err(FatalError::Engine(format!(
                "handshake failed: {reason} (supervision state {state})"
            )));
        }
    };

    // What the bridge will report to the page. Built from the handshake the
    // shell actually completed, so `501` section 6 holds: the shell never
    // reports engine state it did not observe.
    let engine = EngineView::Connected {
        session_id: session_id.clone(),
        protocol_major,
        protocol_minor,
        state_generation: 0,
    };

    // 501 section 6 point 4: the UI is created once the engine has answered.
    // Supervision keeps running underneath the window, so a crash reaches the
    // user as an engine failure rather than as a window that stops responding.
    let hosted = ui_host::run(resources, engine, || {
        match supervisor.poll(&credential, Instant::now()) {
            SupervisionState::GaveUp => EngineHealth::Failed(format!(
                "the engine stopped and the restart budget is exhausted after {} launches: {}",
                supervisor.launches(),
                supervisor.last_reason().unwrap_or("no reason given")
            )),
            _ => EngineHealth::Running,
        }
    });

    // Whatever ended the session, the engine is asked to stop rather than left
    // behind: `501` section 6 point 7 makes shutdown the shell's job.
    supervisor.stop();

    hosted
}

/// Locate the engine executable.
///
/// The environment variable wins, so a packaged build can point at its own layout.
/// Otherwise the engine is expected beside this executable, which is how both are
/// built and how they will be packaged.
fn engine_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var(ENGINE_PATH_VARIABLE) {
        let path = PathBuf::from(path);
        return path.is_file().then_some(path);
    }

    let executable = std::env::current_exe().ok()?;
    let directory = executable.parent()?;
    let candidate = directory.join(if cfg!(windows) {
        "mirae-engine.exe"
    } else {
        "mirae-engine"
    });

    candidate.is_file().then_some(candidate)
}

/// A seed for the placeholder credential.
fn seed() -> u128 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());

    nanos ^ u128::from(std::process::id())
}

/// Launches the engine as a real child process.
struct ProcessLauncher {
    engine_path: PathBuf,
    child: Option<Child>,
    /// Readiness learned from the handshake, if it has completed.
    readiness: Option<EngineReadiness>,
}

impl ProcessLauncher {
    /// Build a launcher for the engine at `engine_path`.
    fn new(engine_path: PathBuf) -> Self {
        Self {
            engine_path,
            child: None,
            readiness: None,
        }
    }

    /// Send `Hello` over the inherited channel and read the engine's answer.
    ///
    /// Returns the `Welcome` on success, or a safe reason on refusal. The
    /// credential is written into the frame and never logged.
    fn handshake(&mut self, credential: &LaunchCredential) -> Result<Welcome, String> {
        let child = self.child.as_mut().ok_or("the engine is not running")?;
        let stdin = child.stdin.as_mut().ok_or("the engine channel is closed")?;

        let hello = Hello {
            role: HelloRole::Shell,
            protocol_major_min: PROTOCOL_VERSION_MAJOR,
            protocol_major_max: PROTOCOL_VERSION_MAJOR,
            protocol_minor_max: PROTOCOL_VERSION_MINOR,
            credential: ipc::encode_hex(credential.expose()),
            build_id: concat!("mirae-shell@", env!("CARGO_PKG_VERSION")).to_owned(),
        };

        let body = serde_json::to_vec(&hello).map_err(|_| "could not encode hello".to_owned())?;
        let header = ipc::FrameHeader {
            protocol_major: PROTOCOL_VERSION_MAJOR,
            protocol_minor: PROTOCOL_VERSION_MINOR,
            message_type: ipc::MessageType::Hello,
            flags: 0,
            payload_length: u32::try_from(body.len()).unwrap_or(u32::MAX),
            correlation_id: 1,
        };

        ipc::write_frame(stdin, &header, &body).map_err(|error| error.to_string())?;

        let stdout = child
            .stdout
            .as_mut()
            .ok_or("the engine channel is closed")?;
        let (response, payload) = ipc::read_frame(stdout, ipc::DEFAULT_MAX_FRAME_BYTES)
            .map_err(|error| error.to_string())?;

        match response.message_type {
            ipc::MessageType::Welcome => serde_json::from_slice::<Welcome>(&payload)
                .map_err(|_| "the welcome did not match its contract".to_owned()),
            ipc::MessageType::Reject => {
                let reject = serde_json::from_slice::<Reject>(&payload)
                    .map_err(|_| "the rejection did not match its contract".to_owned())?;
                Err(format!("{:?}: {}", reject.reason, reject.detail))
            }
            ipc::MessageType::Hello => Err("the engine answered with a hello".to_owned()),
        }
    }
}

impl EngineLauncher for ProcessLauncher {
    fn launch(&mut self, credential: &LaunchCredential) -> bool {
        // Hex, because an environment variable carries text. MIR-0012 replaces the
        // whole handoff with an inherited channel.
        let encoded: String = credential
            .expose()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();

        // `--supervised` keeps the engine alive until this shell closes its stdin,
        // so a clean exit is something the shell asked for rather than something
        // the supervisor has to interpret as a crash.
        let spawned = Command::new(&self.engine_path)
            .arg("--supervised")
            .env(CREDENTIAL_VARIABLE, encoded)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn();

        let Ok(child) = spawned else {
            return false;
        };

        // stdout stays with the child: under `--supervised` it carries protocol
        // frames, and the handshake reads them directly. No reader thread, because
        // nothing may consume bytes the framing needs.
        self.child = Some(child);
        self.readiness = None;
        true
    }

    fn is_running(&mut self) -> bool {
        let Some(child) = self.child.as_mut() else {
            return false;
        };

        // `try_wait` reaps without blocking; an error means the child is unusable.
        matches!(child.try_wait(), Ok(None))
    }

    fn take_readiness(&mut self) -> Option<EngineReadiness> {
        // Readiness now comes from the authenticated handshake, which the shell
        // performs explicitly. Nothing arrives asynchronously.
        self.readiness.take()
    }

    fn stop(&mut self) {
        let Some(child) = self.child.as_mut() else {
            self.readiness = None;
            return;
        };

        // Ask first: closing stdin is the engine's shutdown signal, so it stops
        // through its own lifecycle and flushes its logs.
        drop(child.stdin.take());

        let deadline = Instant::now() + SHUTDOWN_DEADLINE;
        let mut exited = false;

        while Instant::now() < deadline {
            match child.try_wait() {
                Ok(Some(_)) => {
                    exited = true;
                    break;
                }
                Ok(None) => std::thread::sleep(POLL_INTERVAL),
                Err(_) => break,
            }
        }

        if !exited {
            // Killing is the fallback after the deadline, never the first move
            // (`102` section 11: no stage may wait forever).
            let _ = child.kill();
            let _ = child.wait();
        }

        self.child = None;
        self.readiness = None;
    }
}

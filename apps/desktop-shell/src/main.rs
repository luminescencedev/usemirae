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
//! Launch, readiness handoff, supervision, and bounded restart. The supervision
//! logic lives in `mirae-runtime` so it is testable without spawning processes;
//! this binary provides the real process launcher and the command line.
//!
//! # What does not
//!
//! No window and no embedded webview. `501` section 3 requires a native webview
//! hosting locally packaged resources, and every candidate is either unapproved by
//! `DEPENDENCY_VERSIONS.md` section 14 or a new Rust dependency that must clear
//! section 11 first. Choosing one is a ticket with an ADR, not a decision to make
//! quietly here, so the shell currently reports to the terminal and the control UI
//! runs through its own dev server.
//!
//! Authentication of the handoff is `MIR-0012`: the credential is created and
//! passed, but the engine does not verify it yet.

use std::io::{BufRead as _, BufReader, Write as _};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mirae_contracts::generated::EngineReadiness;
use mirae_runtime::supervisor::{
    EngineLauncher, LaunchCredential, RestartPolicy, SupervisionState, Supervisor,
    parse_readiness_line,
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

/// Launch, supervise, report, and stop.
fn run() -> std::process::ExitCode {
    let Some(engine_path) = engine_path() else {
        let mut err = std::io::stderr().lock();
        let _ = writeln!(
            err,
            "could not locate the engine executable; set {ENGINE_PATH_VARIABLE}"
        );
        return std::process::ExitCode::FAILURE;
    };

    let credential = LaunchCredential::placeholder(seed());
    let mut supervisor = Supervisor::new(ProcessLauncher::new(engine_path), RestartPolicy::DEFAULT);

    if !supervisor.start(&credential, Instant::now()) {
        report(&supervisor, None);
        return std::process::ExitCode::FAILURE;
    }

    let deadline = Instant::now() + READINESS_TIMEOUT;
    let mut readiness: Option<EngineReadiness> = None;

    while Instant::now() < deadline {
        let state = supervisor.poll(&credential, Instant::now());

        if let Some(observed) = supervisor.observed_readiness() {
            readiness = Some(observed.clone());
            break;
        }

        if matches!(state, SupervisionState::GaveUp | SupervisionState::Stopped) {
            break;
        }

        std::thread::sleep(POLL_INTERVAL);
    }

    report(&supervisor, readiness.as_ref());

    // No window to keep open, so the shell stops the engine and exits rather than
    // idling. The wait loop arrives with the UI host.
    supervisor.stop();

    if readiness.is_some() {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

/// Print what the shell observed. Never prints the credential.
fn report(supervisor: &Supervisor<ProcessLauncher>, readiness: Option<&EngineReadiness>) {
    let mut out = std::io::stdout().lock();

    match readiness {
        Some(readiness) => {
            let _ = writeln!(
                out,
                "engine connected: session={} protocol={}.{} launches={}",
                readiness.engine_session_id,
                readiness.protocol_major,
                readiness.protocol_minor,
                supervisor.launches()
            );

            if let Some(detail) = readiness.detail.as_deref() {
                let _ = writeln!(out, "engine reports an impairment: {detail}");
            }
        }
        None => {
            let mut err = std::io::stderr().lock();
            let _ = writeln!(
                err,
                "engine did not report readiness: state={} reason={}",
                supervisor.state(),
                supervisor.last_reason().unwrap_or("none given")
            );
        }
    }
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
    /// Readiness lines the reader thread has parsed.
    readiness: Option<Receiver<EngineReadiness>>,
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

        let Ok(mut child) = spawned else {
            return false;
        };

        let (sender, receiver) = channel();

        if let Some(stdout) = child.stdout.take() {
            // A dedicated thread, so a silent engine cannot block the supervisor.
            std::thread::spawn(move || {
                for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                    if let Some(readiness) = parse_readiness_line(&line)
                        && sender.send(readiness).is_err()
                    {
                        // The supervisor is gone; stop reading.
                        break;
                    }
                }
            });
        }

        self.child = Some(child);
        self.readiness = Some(receiver);
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
        // Empty and Disconnected both mean "nothing to report right now"; a
        // disconnected channel is the reader thread finishing, which the running
        // check already covers.
        self.readiness.as_ref()?.try_recv().ok()
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

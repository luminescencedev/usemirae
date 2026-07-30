//! The shell detects a real engine failure and restarts it within a bound.
//!
//! Canonical documentation: `docs/05-platform/501-desktop-shell.md` section 6,
//! `docs/01-runtime/102-engine-lifecycle.md`.
//!
//! The supervisor's own unit tests use a fake launcher. This suite spawns the
//! actual engine binary and kills it, so the parts a fake cannot prove are
//! covered: that a killed process is observed as gone, that a restart produces a
//! genuinely new engine session, and that the budget stops a crash loop.
//!
//! `CARGO_BIN_EXE_mirae-engine` is provided by cargo for this package's tests, so
//! the test runs against the binary the workspace just built.

use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use mirae_runtime::supervisor::{
    EngineLauncher, LaunchCredential, RestartPolicy, SupervisionState, Supervisor,
};

/// Spawns the real engine binary.
struct RealEngineLauncher {
    child: Option<Child>,
    /// Sessions observed across launches, to prove a restart is a new process.
    process_ids: Vec<u32>,
}

impl RealEngineLauncher {
    fn new() -> Self {
        Self {
            child: None,
            process_ids: Vec::new(),
        }
    }

    /// Kill the running engine, simulating a crash.
    fn kill_engine(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl EngineLauncher for RealEngineLauncher {
    fn launch(&mut self, credential: &LaunchCredential) -> bool {
        let encoded: String = credential
            .expose()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();

        // `--supervised` keeps the engine alive until its stdin closes, which is
        // what makes a kill distinguishable from a normal exit.
        let spawned = Command::new(env!("CARGO_BIN_EXE_mirae-engine"))
            .arg("--supervised")
            .env("MIRAE_LAUNCH_CREDENTIAL", encoded)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        match spawned {
            Ok(child) => {
                self.process_ids.push(child.id());
                self.child = Some(child);
                true
            }
            Err(_) => false,
        }
    }

    fn is_running(&mut self) -> bool {
        self.child
            .as_mut()
            .is_some_and(|child| matches!(child.try_wait(), Ok(None)))
    }

    fn take_readiness(&mut self) -> Option<mirae_contracts::generated::EngineReadiness> {
        None
    }

    fn stop(&mut self) {
        self.kill_engine();
        self.child = None;
    }
}

/// Give a freshly spawned process a moment to exist before it is inspected.
fn settle() {
    std::thread::sleep(Duration::from_millis(150));
}

#[test]
fn the_shell_launches_a_real_engine() {
    let mut supervisor = Supervisor::new(RealEngineLauncher::new(), RestartPolicy::DEFAULT);
    let credential = LaunchCredential::placeholder(1);

    assert!(supervisor.start(&credential, Instant::now()));
    settle();

    assert_eq!(supervisor.state(), SupervisionState::Launching);
    assert_eq!(supervisor.launches(), 1);

    supervisor.stop();
    assert_eq!(supervisor.state(), SupervisionState::Stopped);
}

#[test]
fn a_killed_engine_is_detected_and_restarted() {
    let mut supervisor = Supervisor::new(RealEngineLauncher::new(), RestartPolicy::DEFAULT);
    let credential = LaunchCredential::placeholder(2);

    assert!(supervisor.start(&credential, Instant::now()));
    settle();

    supervisor.launcher_mut().kill_engine();
    settle();

    let state = supervisor.poll(&credential, Instant::now());

    assert_eq!(
        state,
        SupervisionState::Launching,
        "a killed engine must be relaunched"
    );
    assert_eq!(supervisor.launches(), 2);

    // A restart is a new process, not a resurrected one: 102 invariant 8 says a
    // new engine process creates a new session.
    let ids = supervisor.launcher_mut().process_ids.clone();
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0], ids[1], "the restart should be a different process");

    supervisor.stop();
}

#[test]
fn a_crash_loop_stops_at_the_restart_budget() {
    let policy = RestartPolicy {
        max_restarts: 2,
        window: Duration::from_secs(60),
    };
    let mut supervisor = Supervisor::new(RealEngineLauncher::new(), policy);
    let credential = LaunchCredential::placeholder(3);

    assert!(supervisor.start(&credential, Instant::now()));
    settle();

    for _ in 0..5 {
        supervisor.launcher_mut().kill_engine();
        settle();
        supervisor.poll(&credential, Instant::now());
    }

    assert_eq!(
        supervisor.state(),
        SupervisionState::GaveUp,
        "the budget must stop a crash loop"
    );
    assert!(
        supervisor
            .last_reason()
            .is_some_and(|reason| reason.contains("within 60 seconds")),
        "giving up must say why"
    );
    // The initial launch plus at most the budget.
    assert!(supervisor.launches() <= 3);

    supervisor.stop();
}

#[test]
fn the_shell_reports_no_engine_state_while_disconnected() {
    // 501 section 6: the shell must not fabricate engine state while disconnected.
    let mut supervisor = Supervisor::new(RealEngineLauncher::new(), RestartPolicy::DEFAULT);
    let credential = LaunchCredential::placeholder(4);

    supervisor.start(&credential, Instant::now());
    settle();
    supervisor.launcher_mut().kill_engine();
    settle();
    supervisor.poll(&credential, Instant::now());

    assert!(supervisor.observed_readiness().is_none());

    supervisor.stop();
}

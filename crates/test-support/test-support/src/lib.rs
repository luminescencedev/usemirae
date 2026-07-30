//! Shared harness for cross-process integration tests.
//!
//! Canonical documentation:
//! `docs/08-development/809-testing-and-validation-workflow.md`,
//! `docs/08-development/801-monorepo-architecture.md` section 7,
//! `docs/06-quality/609-testing-strategy.md`.
//!
//! Integration tests launch real processes, so every wait here is bounded and
//! every failure names what it was waiting for. A test that hangs is worse than
//! one that fails: CI cannot tell a hang from a slow machine.
//!
//! The harness owns process lifetime. Dropping it stops the engine, so a failing
//! assertion cannot leak a process into the rest of the suite.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use mirae_contracts::generated::{
    Hello, HelloRole, PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR, Reject, Welcome,
};
use mirae_runtime::ipc;
use mirae_runtime::supervisor::LaunchCredential;

/// How long the harness waits for the engine to answer before failing.
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// How long the harness waits for the engine to exit after being asked.
pub const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// How often a bounded wait re-checks.
const POLL_INTERVAL: Duration = Duration::from_millis(25);

/// Why the harness could not do what a test asked.
#[derive(Debug, PartialEq, Eq)]
pub enum HarnessError {
    /// The engine executable could not be located.
    EngineNotFound(String),
    /// The engine process could not be launched.
    LaunchFailed(String),
    /// The engine did not answer the handshake within the deadline.
    HandshakeTimedOut,
    /// The engine refused the handshake.
    HandshakeRefused(String),
    /// The engine did not exit within the deadline and had to be killed.
    ShutdownTimedOut,
    /// The channel to the engine failed.
    ChannelFailed(String),
}

impl std::fmt::Display for HarnessError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EngineNotFound(detail) => write!(formatter, "engine not found: {detail}"),
            Self::LaunchFailed(detail) => write!(formatter, "launch failed: {detail}"),
            Self::HandshakeTimedOut => {
                formatter.write_str("the engine never answered the handshake")
            }
            Self::HandshakeRefused(reason) => write!(formatter, "handshake refused: {reason}"),
            Self::ShutdownTimedOut => formatter.write_str("the engine did not exit in time"),
            Self::ChannelFailed(detail) => write!(formatter, "channel failed: {detail}"),
        }
    }
}

impl std::error::Error for HarnessError {}

/// Locate the engine executable next to the running test binary.
///
/// `MIRAE_ENGINE_PATH` wins, so a packaged or cross-compiled layout can point at
/// its own. Otherwise the binary sits above `deps/`, which is where cargo puts it
/// for every test in the workspace.
pub fn locate_engine() -> Result<PathBuf, HarnessError> {
    let name = if cfg!(windows) {
        "mirae-engine.exe"
    } else {
        "mirae-engine"
    };

    if let Ok(path) = std::env::var("MIRAE_ENGINE_PATH") {
        let path = PathBuf::from(path);
        return if path.is_file() {
            Ok(path)
        } else {
            Err(HarnessError::EngineNotFound(format!(
                "MIRAE_ENGINE_PATH points at {}, which is not a file",
                path.display()
            )))
        };
    }

    let executable =
        std::env::current_exe().map_err(|error| HarnessError::EngineNotFound(error.to_string()))?;

    // target/debug/deps/<test> -> target/debug/<engine>
    for ancestor in executable.ancestors().skip(1).take(3) {
        let candidate = ancestor.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(HarnessError::EngineNotFound(format!(
        "no {name} beside {}",
        executable.display()
    )))
}

/// A running engine process, with the channel a test speaks over.
#[derive(Debug)]
pub struct EngineHarness {
    child: Option<Child>,
    credential: LaunchCredential,
    /// The welcome, once the handshake has completed.
    welcome: Option<Welcome>,
}

impl EngineHarness {
    /// Launch an engine with a deterministic credential.
    ///
    /// The credential is derived from a fixed seed rather than randomness, so a
    /// failing run can be reproduced exactly.
    pub fn launch(credential_seed: u128) -> Result<Self, HarnessError> {
        let engine = locate_engine()?;
        let credential = LaunchCredential::placeholder(credential_seed);

        let child = Command::new(&engine)
            .arg("--supervised")
            .env(
                "MIRAE_LAUNCH_CREDENTIAL",
                ipc::encode_hex(credential.expose()),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| HarnessError::LaunchFailed(error.to_string()))?;

        Ok(Self {
            child: Some(child),
            credential,
            welcome: None,
        })
    }

    /// The credential this engine was launched with, hex encoded.
    #[must_use]
    pub fn credential_hex(&self) -> String {
        ipc::encode_hex(self.credential.expose())
    }

    /// Complete the authenticated handshake, retrying until the deadline.
    pub fn handshake(&mut self) -> Result<Welcome, HarnessError> {
        let credential = self.credential_hex();
        self.handshake_as(HelloRole::Shell, &credential)
    }

    /// Handshake with an explicit role and credential, for negative tests.
    ///
    /// Retries only while the channel is not yet open. A refusal returns
    /// immediately, because retrying a rejection would turn a clear failure into a
    /// timeout.
    pub fn handshake_as(
        &mut self,
        role: HelloRole,
        credential_hex: &str,
    ) -> Result<Welcome, HarnessError> {
        let deadline = Instant::now() + HANDSHAKE_TIMEOUT;

        loop {
            match self.try_handshake(role, credential_hex) {
                Ok(welcome) => {
                    self.welcome = Some(welcome.clone());
                    return Ok(welcome);
                }
                Err(HarnessError::ChannelFailed(ref detail))
                    if detail.contains("frame ended early") && Instant::now() < deadline =>
                {
                    std::thread::sleep(POLL_INTERVAL);
                }
                Err(HarnessError::ChannelFailed(_)) => return Err(HarnessError::HandshakeTimedOut),
                Err(error) => return Err(error),
            }
        }
    }

    fn try_handshake(
        &mut self,
        role: HelloRole,
        credential_hex: &str,
    ) -> Result<Welcome, HarnessError> {
        let child = self
            .child
            .as_mut()
            .ok_or_else(|| HarnessError::ChannelFailed("the engine is not running".to_owned()))?;

        let hello = Hello {
            role,
            protocol_major_min: PROTOCOL_VERSION_MAJOR,
            protocol_major_max: PROTOCOL_VERSION_MAJOR,
            protocol_minor_max: PROTOCOL_VERSION_MINOR,
            credential: credential_hex.to_owned(),
            build_id: "integration-test".to_owned(),
        };

        let body = serde_json::to_vec(&hello)
            .map_err(|error| HarnessError::ChannelFailed(error.to_string()))?;
        let header = ipc::FrameHeader {
            protocol_major: PROTOCOL_VERSION_MAJOR,
            protocol_minor: PROTOCOL_VERSION_MINOR,
            message_type: ipc::MessageType::Hello,
            flags: 0,
            payload_length: u32::try_from(body.len()).unwrap_or(u32::MAX),
            correlation_id: 42,
        };

        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| HarnessError::ChannelFailed("stdin is closed".to_owned()))?;
        ipc::write_frame(stdin, &header, &body)
            .map_err(|error| HarnessError::ChannelFailed(error.to_string()))?;

        let stdout = child
            .stdout
            .as_mut()
            .ok_or_else(|| HarnessError::ChannelFailed("stdout is closed".to_owned()))?;
        let (response, payload) = ipc::read_frame(stdout, ipc::DEFAULT_MAX_FRAME_BYTES)
            .map_err(|error| HarnessError::ChannelFailed(error.to_string()))?;

        // The answer must belong to the request it was sent for.
        if response.correlation_id != header.correlation_id {
            return Err(HarnessError::ChannelFailed(
                "the answer carried a different correlation id".to_owned(),
            ));
        }

        match response.message_type {
            ipc::MessageType::Welcome => serde_json::from_slice::<Welcome>(&payload)
                .map_err(|error| HarnessError::ChannelFailed(error.to_string())),
            ipc::MessageType::Reject => {
                let reject: Reject = serde_json::from_slice(&payload)
                    .map_err(|error| HarnessError::ChannelFailed(error.to_string()))?;
                Err(HarnessError::HandshakeRefused(format!(
                    "{:?}: {}",
                    reject.reason, reject.detail
                )))
            }
            ipc::MessageType::Hello => Err(HarnessError::ChannelFailed(
                "the engine answered with a hello".to_owned(),
            )),
        }
    }

    /// The welcome from a completed handshake.
    #[must_use]
    pub fn welcome(&self) -> Option<&Welcome> {
        self.welcome.as_ref()
    }

    /// Whether the engine process is still running.
    pub fn is_running(&mut self) -> bool {
        self.child
            .as_mut()
            .is_some_and(|child| matches!(child.try_wait(), Ok(None)))
    }

    /// Ask the engine to stop, and wait for it within the deadline.
    ///
    /// Closing stdin is the cooperative shutdown request. An overrun returns an
    /// error rather than killing silently, so a test can assert that the clean
    /// path actually worked.
    pub fn shutdown(&mut self) -> Result<(), HarnessError> {
        let Some(child) = self.child.as_mut() else {
            return Ok(());
        };

        drop(child.stdin.take());

        let deadline = Instant::now() + SHUTDOWN_TIMEOUT;
        while Instant::now() < deadline {
            match child.try_wait() {
                Ok(Some(_)) => {
                    self.child = None;
                    return Ok(());
                }
                Ok(None) => std::thread::sleep(POLL_INTERVAL),
                Err(error) => return Err(HarnessError::ChannelFailed(error.to_string())),
            }
        }

        let _ = child.kill();
        let _ = child.wait();
        self.child = None;

        Err(HarnessError::ShutdownTimedOut)
    }
}

impl Drop for EngineHarness {
    /// Stop the engine, so a failing test cannot leak a process.
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            drop(child.stdin.take());
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_describe_what_the_harness_was_waiting_for() {
        assert_eq!(
            HarnessError::HandshakeTimedOut.to_string(),
            "the engine never answered the handshake"
        );
        assert!(
            HarnessError::HandshakeRefused("unauthenticated".to_owned())
                .to_string()
                .contains("unauthenticated")
        );
        assert_eq!(
            HarnessError::ShutdownTimedOut.to_string(),
            "the engine did not exit in time"
        );
    }

    #[test]
    fn an_explicit_engine_path_that_is_not_a_file_is_reported() {
        // This sets a process-wide variable and restores it immediately.
        unsafe { std::env::set_var("MIRAE_ENGINE_PATH", "definitely-not-a-file") };
        let located = locate_engine();
        unsafe { std::env::remove_var("MIRAE_ENGINE_PATH") };

        assert!(matches!(located, Err(HarnessError::EngineNotFound(_))));
    }
}

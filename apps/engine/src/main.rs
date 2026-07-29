//! Minimal engine process entry point.
//!
//! Placeholder binary created by `MIR-0001 — Initialize monorepo`. Service
//! initialization, readiness reporting, and clean shutdown arrive with
//! `MIR-0009`; the authenticated IPC handshake arrives with `MIR-0012`.
//!
//! Canonical documentation: `docs/08-development/802-rust-workspace-and-crates.md`.

use std::io::Write as _;
use std::process::ExitCode;

/// Process role this application reports to the shell and to diagnostics.
const ROLE: &str = "engine";

/// Ticket that replaces this placeholder with real process lifecycle.
const IMPLEMENTED_BY: &str = "MIR-0009";

fn main() -> ExitCode {
    // Only the version metadata contract from `801-monorepo-architecture.md` §4
    // exists yet. Any other invocation fails loudly rather than reporting a
    // started engine that does not exist.
    let wants_version = std::env::args().any(|arg| arg == "--version" || arg == "-V");

    if wants_version {
        let mut out = std::io::stdout().lock();
        let _ = writeln!(
            out,
            "{name} {version} role={ROLE} state=placeholder implemented-by={IMPLEMENTED_BY}",
            name = env!("CARGO_PKG_NAME"),
            version = env!("CARGO_PKG_VERSION"),
        );
        return ExitCode::SUCCESS;
    }

    let mut err = std::io::stderr().lock();
    let _ = writeln!(
        err,
        "{name}: placeholder process, no runtime yet (see {IMPLEMENTED_BY}). Try --version.",
        name = env!("CARGO_PKG_NAME"),
    );
    ExitCode::FAILURE
}

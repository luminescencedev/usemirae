//! Extension-host process entry point.
//!
//! Placeholder binary created by `MIR-0001 — Initialize monorepo`. Sandboxed
//! extension execution, quotas, and broker mediation arrive with the SDK
//! backlog; third-party extensions never execute in the engine process.
//!
//! Canonical documentation: `docs/08-development/802-rust-workspace-and-crates.md`,
//! `docs/07-sdk/`.

use std::io::Write as _;
use std::process::ExitCode;

/// Process role this application reports to diagnostics.
const ROLE: &str = "extension-host";

/// Ticket that replaces this placeholder with real process lifecycle.
const IMPLEMENTED_BY: &str = "MIR-0012";

fn main() -> ExitCode {
    // Only the version metadata contract from `801-monorepo-architecture.md` §4
    // exists yet. Any other invocation fails loudly rather than claiming to host
    // extensions.
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

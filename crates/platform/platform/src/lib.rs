//! Cross-platform interfaces and capability model.
//!
//! Canonical documentation: `docs/05-platform/500-platform-overview.md`,
//! `docs/08-development/802-rust-workspace-and-crates.md`.
//! Dependency rules: `docs/08-development/804-dependency-rules.md`.
//!
//! # What exists today
//!
//! Machine-local directory resolution (`MIR-0112`, ADR-0072). It lives here
//! rather than in the project layer because `804` section 3 forbids a domain or
//! application crate from reaching platform knowledge, and section 4 puts the
//! implementation outward from the interface that needs it.
//!
//! # What does not
//!
//! The capability model this crate is named for. It arrives with the first
//! subsystem that has a capability to report.

pub mod directories;

pub use directories::{local_data_directory, recovery_directory};

//! Stable IDs, generations, rational time, bounded primitive wrappers, and common enums.
//!
//! Canonical documentation: `docs/00-foundations/005-domain-model.md`,
//! `docs/01-runtime/106-state-store.md`, `docs/08-development/802-rust-workspace-and-crates.md`.
//! Dependency rules: `docs/08-development/804-dependency-rules.md`.
//!
//! This is the innermost layer. It depends on serialization and identity and on
//! nothing else — no runtime, no platform, no UI — so every layer above may
//! depend on it without acquiring anything (804 section 2).
//!
//! # What exists today
//!
//! Entity identifiers and generations, added by `MIR-0101` because every ticket
//! in the project kernel needs to name an entity and to say which version of the
//! state it is talking about.
//!
//! # What does not
//!
//! Rational time, bounded primitive wrappers, and the common enums this crate is
//! named for. They arrive with the first subsystem that needs them, which keeps
//! the foundation from filling with types invented ahead of a caller.

pub mod generation;
pub mod id;

pub use generation::{CapabilityGeneration, StateGeneration};
pub use id::{
    AssetId, EntityId, IdParseError, OutputId, ProjectId, SceneId, SceneItemId, SourceId,
};

//! Project-domain entities and validation. No OS, GPU, FFmpeg, UI, or network dependencies.
//!
//! Canonical documentation: `docs/00-foundations/005-domain-model.md`,
//! `docs/08-development/802-rust-workspace-and-crates.md`.
//! Dependency rules: `docs/08-development/804-dependency-rules.md`.
//!
//! The domain depends on identity and on nothing else. That is not a stylistic
//! preference: `804` section 3 lists `domain → platform`, `domain → wgpu`,
//! `domain → FFmpeg`, and `domain → React` as forbidden precisely because a
//! domain that can reach a device stops being testable and starts being a
//! description of one machine.

pub mod entity;

pub use entity::{
    EntityName, MAX_NAME_CHARACTERS, NameError, Scene, SceneItem, SourceDefinition, SourceKind,
};

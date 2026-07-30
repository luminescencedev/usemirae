//! Canonical serialization and the integrity hash.
//!
//! Canonical documentation: `docs/04-project/401-project-format.md` sections 11
//! and 12, ADR-0071.
//!
//! ADR-0071 fixed the rules: keys sorted, two-space indentation, `\n` line
//! endings, and the hash computed over the document with the hash field
//! excluded. This module is the only place those rules are implemented, because
//! a second implementation is a second answer, and the whole point of
//! canonicalization is that there is one.
//!
//! Sorting is not enforced by hand here. `serde_json` builds objects on a
//! `BTreeMap` unless its `preserve_order` feature is on, so key order is already
//! by code point; the test that asserts it exists so that turning that feature
//! on somewhere else fails loudly rather than silently changing every file
//! Mirae writes.

use mirae_contracts::generated::PersistedProjectEnvelope;
use sha2::{Digest, Sha256};

/// The line ending every project file uses.
///
/// Fixed rather than platform-dependent. A project written on Windows and read
/// on Linux must be byte-identical, or the content hash disagrees with itself
/// across machines and every diff is whole-file.
pub const LINE_ENDING: &str = "\n";

/// Why a document could not be serialized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalError {
    /// The document could not be represented as JSON.
    ///
    /// Reachable only through a value the schema should have refused — a
    /// non-finite number is the realistic case — so it is a bug report rather
    /// than something a user can act on.
    NotRepresentable,
}

impl std::fmt::Display for CanonicalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::NotRepresentable => "the project could not be represented as JSON",
        })
    }
}

impl std::error::Error for CanonicalError {}

/// Serialize an envelope canonically.
fn serialize(envelope: &PersistedProjectEnvelope) -> Result<String, CanonicalError> {
    // Through `Value` rather than straight to a string: that is what puts the
    // keys in a `BTreeMap` and therefore in code-point order, whatever order the
    // Rust struct happens to declare them in.
    let value = serde_json::to_value(envelope).map_err(|_| CanonicalError::NotRepresentable)?;

    let mut text =
        serde_json::to_string_pretty(&value).map_err(|_| CanonicalError::NotRepresentable)?;
    text.push_str(LINE_ENDING);

    Ok(text)
}

/// The hexadecimal SHA-256 of `bytes`.
fn hash_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);

    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }

    out
}

/// Serialize an envelope and fill in its integrity hash.
///
/// The hash covers the document with `contentHash` empty, which is how `401`
/// section 11 excludes the field from the bytes it describes. Doing it in two
/// passes is the cost of that exclusion, and both passes are over a document
/// small enough that it does not matter.
pub fn serialize_with_integrity(
    envelope: &PersistedProjectEnvelope,
) -> Result<(String, String), CanonicalError> {
    let mut without_hash = envelope.clone();
    without_hash.integrity.content_hash = String::new();

    let hash = hash_hex(serialize(&without_hash)?.as_bytes());

    let mut with_hash = without_hash;
    with_hash.integrity.content_hash = hash.clone();

    Ok((serialize(&with_hash)?, hash))
}

/// Recompute the hash of a decoded envelope and compare it with the one it carries.
///
/// Returns `false` when the file was altered after it was written. `401` section
/// 11 is explicit that this detects accidental corruption rather than tampering:
/// anyone who can edit the file can recompute the hash, so a match means intact,
/// not authentic.
#[must_use]
pub fn integrity_matches(envelope: &PersistedProjectEnvelope) -> bool {
    let mut without_hash = envelope.clone();
    without_hash.integrity.content_hash = String::new();

    serialize(&without_hash)
        .map(|text| hash_hex(text.as_bytes()) == envelope.integrity.content_hash)
        .unwrap_or(false)
}

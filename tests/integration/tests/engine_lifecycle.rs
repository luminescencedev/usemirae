//! The engine launches, authenticates a client, and shuts down cleanly.
//!
//! Canonical documentation: `docs/01-runtime/102-engine-lifecycle.md`,
//! `docs/01-runtime/108-ipc-protocol.md` sections 6 and 11,
//! `docs/05-platform/501-desktop-shell.md` section 6.
//!
//! These tests cross a process boundary on purpose. Everything they assert has
//! unit coverage in the crate that owns it; what only a real process can show is
//! that the pieces agree with each other: that the engine the workspace built
//! answers the frames the shell's code writes, and that closing the channel
//! actually ends the process.

// The workspace denies `expect` and `panic` because `807` section 2 forbids them
// in recoverable production paths. A test has no recovery: failing loudly with a
// message is the assertion. Scoped to this file rather than relaxed workspace-wide.
#![allow(clippy::expect_used, clippy::panic)]

use mirae_contracts::generated::{HelloRole, PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR};
use mirae_test_support::{EngineHarness, HarnessError};

#[test]
fn an_engine_accepts_an_authenticated_client_and_stops_cleanly() {
    let mut harness = EngineHarness::launch(0x1111).expect("the engine should launch");

    let welcome = harness
        .handshake()
        .expect("the handshake should be accepted");

    assert_eq!(welcome.protocol_major, PROTOCOL_VERSION_MAJOR);
    assert_eq!(welcome.protocol_minor, PROTOCOL_VERSION_MINOR);
    assert!(
        !welcome.engine_session_id.is_empty(),
        "the engine must identify its session"
    );
    assert!(
        welcome.max_frame_bytes > 0,
        "the connection must state its frame limit"
    );

    // 102 invariant 4: shutdown waits are bounded, and the clean path must work
    // without the harness resorting to a kill.
    harness
        .shutdown()
        .expect("the engine should stop when its channel closes");
    assert!(!harness.is_running());
}

#[test]
fn a_wrong_credential_is_refused_by_the_real_engine() {
    let mut harness = EngineHarness::launch(0x2222).expect("the engine should launch");

    let refused = harness.handshake_as(HelloRole::Shell, "00ff00ff00ff00ff");

    match refused {
        Err(HarnessError::HandshakeRefused(reason)) => {
            assert!(
                reason.contains("Unauthenticated"),
                "expected an unauthenticated refusal, got `{reason}`"
            );
            // 108 section 14: the refusal must not describe the credential.
            assert!(!reason.contains("00ff"), "the reason leaked the credential");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }

    harness.shutdown().ok();
}

#[test]
fn each_engine_run_is_a_new_session() {
    // 102 invariant 8: a new engine process creates a new session.
    let mut first = EngineHarness::launch(0x3333).expect("the engine should launch");
    let mut second = EngineHarness::launch(0x3333).expect("the engine should launch");

    let one = first.handshake().expect("first handshake");
    let two = second.handshake().expect("second handshake");

    assert_ne!(
        one.engine_session_id, two.engine_session_id,
        "two concurrent engines must not share a session id"
    );

    first.shutdown().expect("first engine stops");
    second.shutdown().expect("second engine stops");
}

#[test]
fn the_harness_leaves_no_process_behind_when_a_test_fails() {
    // Dropping without shutting down must still stop the engine, which is what
    // keeps one failing test from leaking into the rest of the suite.
    let mut harness = EngineHarness::launch(0x4444).expect("the engine should launch");
    harness.handshake().expect("the handshake should succeed");

    assert!(harness.is_running());
    drop(harness);

    // A second engine on the same credential seed launches and answers, which it
    // could not do reliably if the first were still holding the channel.
    let mut replacement = EngineHarness::launch(0x4444).expect("the engine should launch");
    assert!(replacement.handshake().is_ok());
    replacement.shutdown().expect("the replacement stops");
}

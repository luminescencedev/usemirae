//! The generated handshake contracts are usable from Rust.
//!
//! These tests are hand-written and live outside the generated file, so
//! regeneration never overwrites them. They assert the parts other crates will
//! depend on: the negotiated protocol version, the readiness states, and the
//! bounds that keep decoding bounded (`docs/01-runtime/108-ipc-protocol.md`).

use mirae_contracts::generated::{
    CONTRACT_IDS, ENGINE_READINESS_DETAIL_MAX_LENGTH,
    ENGINE_READINESS_ENGINE_SESSION_ID_MAX_LENGTH, EngineReadiness, EngineReadinessState,
    PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR, ProtocolVersion,
};

#[test]
fn protocol_version_constants_match_the_schema() {
    assert_eq!(PROTOCOL_VERSION_MAJOR, 1);
    assert_eq!(PROTOCOL_VERSION_MINOR, 0);
}

#[test]
fn protocol_version_can_be_constructed_from_its_constants() {
    let version = ProtocolVersion {
        major: PROTOCOL_VERSION_MAJOR,
        minor: PROTOCOL_VERSION_MINOR,
    };

    assert_eq!(version, ProtocolVersion { major: 1, minor: 0 });
}

#[test]
fn every_readiness_state_has_a_stable_wire_value() {
    let states = [
        (EngineReadinessState::Starting, "starting"),
        (EngineReadinessState::Ready, "ready"),
        (EngineReadinessState::Degraded, "degraded"),
        (EngineReadinessState::Stopping, "stopping"),
        (EngineReadinessState::Stopped, "stopped"),
    ];

    for (state, wire) in states {
        assert_eq!(state.as_wire_str(), wire);
    }
}

#[test]
fn readiness_carries_an_optional_detail() {
    let ready = EngineReadiness {
        state: EngineReadinessState::Ready,
        protocol_major: PROTOCOL_VERSION_MAJOR,
        protocol_minor: PROTOCOL_VERSION_MINOR,
        engine_session_id: "session-1".to_owned(),
        detail: None,
    };

    let degraded = EngineReadiness {
        state: EngineReadinessState::Degraded,
        detail: Some("no GPU adapter".to_owned()),
        ..ready.clone()
    };

    assert_eq!(ready.detail, None);
    assert_eq!(degraded.detail.as_deref(), Some("no GPU adapter"));
    assert_ne!(ready.state, degraded.state);
}

#[test]
fn string_bounds_are_exposed_for_bounded_decoding() {
    // 108 section 9: oversized input is rejected before a large allocation, which
    // requires the limit to be available to the decoder.
    assert_eq!(ENGINE_READINESS_ENGINE_SESSION_ID_MAX_LENGTH, 64);
    assert_eq!(ENGINE_READINESS_DETAIL_MAX_LENGTH, 256);
}

#[test]
fn contract_ids_are_sorted_and_unique() {
    let mut sorted = CONTRACT_IDS;
    sorted.sort_unstable();

    assert_eq!(CONTRACT_IDS, sorted, "generated ids must already be sorted");

    let mut deduplicated = sorted.to_vec();
    deduplicated.dedup();

    assert_eq!(deduplicated.len(), CONTRACT_IDS.len());
    assert!(CONTRACT_IDS.contains(&"mirae://ipc/v1/protocol-version"));
    assert!(CONTRACT_IDS.contains(&"mirae://ipc/v1/engine-readiness"));
}

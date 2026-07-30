//! Control-plane framing and the authenticated handshake.
//!
//! Canonical documentation: `docs/01-runtime/108-ipc-protocol.md` sections 4, 6,
//! 8, 9, 11, and 14; ADR-0006 and ADR-0067.
//!
//! # Framing
//!
//! A fixed-size binary header precedes a JSON payload, exactly as ADR-0067
//! decided. The header is validated before anything is allocated for the payload,
//! so an oversized or malformed frame is refused rather than trusted
//! (section 9, and section 18: never allocate on an untrusted length).
//!
//! # Authentication and authorization are separate
//!
//! A correct credential proves the peer was launched by this engine's parent. It
//! does not decide what that peer may then do (section 11). This module performs
//! the first half only.

use core::fmt;
use std::io::{Read, Write};

use mirae_contracts::generated::{
    Hello, HelloRole, PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR, Reject, RejectReason, Welcome,
};

/// Frame magic, so a stray byte stream is refused immediately.
pub const FRAME_MAGIC: [u8; 4] = *b"MIRA";

/// Bytes in the fixed header: magic, major, minor, type, flags, length,
/// correlation id.
pub const HEADER_BYTES: usize = 4 + 2 + 2 + 2 + 2 + 4 + 16;

/// Largest payload this build accepts, before a connection negotiates its own.
///
/// One megabyte is generous for control-plane messages and small enough that a
/// hostile length cannot exhaust memory (`108` section 9).
pub const DEFAULT_MAX_FRAME_BYTES: u32 = 1024 * 1024;

/// Which message a frame carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// A client's opening message.
    Hello,
    /// The engine accepting a client.
    Welcome,
    /// The engine refusing a client.
    Reject,
}

impl MessageType {
    /// The stable numeric identity carried on the wire (`108` section 7).
    #[must_use]
    pub const fn code(self) -> u16 {
        match self {
            Self::Hello => 1,
            Self::Welcome => 2,
            Self::Reject => 3,
        }
    }

    /// Read a message type from its wire code.
    #[must_use]
    pub const fn from_code(code: u16) -> Option<Self> {
        match code {
            1 => Some(Self::Hello),
            2 => Some(Self::Welcome),
            3 => Some(Self::Reject),
            _ => None,
        }
    }
}

/// The fixed-size frame header from `108` section 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    /// Protocol major version of the sender.
    pub protocol_major: u16,
    /// Protocol minor version of the sender.
    pub protocol_minor: u16,
    /// Which message the payload holds.
    pub message_type: MessageType,
    /// Reserved for future use; must be zero today.
    pub flags: u16,
    /// Payload length in bytes.
    pub payload_length: u32,
    /// Correlates a response with its request.
    pub correlation_id: u128,
}

/// Why a frame could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    /// The stream ended before a whole frame arrived.
    Truncated,
    /// The magic bytes did not match, so this is not a Mirae stream.
    BadMagic,
    /// The message type is not one this build knows.
    UnknownMessageType(u16),
    /// The declared payload length exceeds the limit for this connection.
    TooLarge {
        /// The length the header declared.
        declared: u32,
        /// The largest length allowed.
        limit: u32,
    },
    /// The payload was not valid JSON for the expected contract.
    MalformedPayload,
    /// The underlying stream failed.
    Io,
}

impl fmt::Display for FrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => formatter.write_str("the frame ended early"),
            Self::BadMagic => formatter.write_str("the stream is not a Mirae protocol stream"),
            Self::UnknownMessageType(code) => {
                write!(formatter, "unknown message type {code}")
            }
            Self::TooLarge { declared, limit } => {
                write!(
                    formatter,
                    "frame of {declared} bytes exceeds the {limit} byte limit"
                )
            }
            Self::MalformedPayload => formatter.write_str("the payload did not match its contract"),
            Self::Io => formatter.write_str("the connection failed"),
        }
    }
}

impl std::error::Error for FrameError {}

/// Encode a header and payload into one frame.
#[must_use]
pub fn encode_frame(header: &FrameHeader, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(HEADER_BYTES + payload.len());

    frame.extend_from_slice(&FRAME_MAGIC);
    frame.extend_from_slice(&header.protocol_major.to_be_bytes());
    frame.extend_from_slice(&header.protocol_minor.to_be_bytes());
    frame.extend_from_slice(&header.message_type.code().to_be_bytes());
    frame.extend_from_slice(&header.flags.to_be_bytes());
    frame.extend_from_slice(
        &u32::try_from(payload.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    frame.extend_from_slice(&header.correlation_id.to_be_bytes());
    frame.extend_from_slice(payload);

    frame
}

/// Parse a header, refusing anything unusable before a payload is read.
pub fn parse_header(bytes: &[u8], max_frame_bytes: u32) -> Result<FrameHeader, FrameError> {
    let header = bytes.get(..HEADER_BYTES).ok_or(FrameError::Truncated)?;

    if header.get(..4) != Some(&FRAME_MAGIC[..]) {
        return Err(FrameError::BadMagic);
    }

    let read_u16 = |at: usize| -> u16 {
        let mut value = [0_u8; 2];
        value.copy_from_slice(header.get(at..at + 2).unwrap_or(&[0, 0]));
        u16::from_be_bytes(value)
    };

    let protocol_major = read_u16(4);
    let protocol_minor = read_u16(6);
    let type_code = read_u16(8);
    let flags = read_u16(10);

    let mut length = [0_u8; 4];
    length.copy_from_slice(header.get(12..16).unwrap_or(&[0; 4]));
    let payload_length = u32::from_be_bytes(length);

    let mut correlation = [0_u8; 16];
    correlation.copy_from_slice(header.get(16..32).unwrap_or(&[0; 16]));

    let message_type =
        MessageType::from_code(type_code).ok_or(FrameError::UnknownMessageType(type_code))?;

    // The bound is checked here, before any caller allocates for the payload.
    if payload_length > max_frame_bytes {
        return Err(FrameError::TooLarge {
            declared: payload_length,
            limit: max_frame_bytes,
        });
    }

    Ok(FrameHeader {
        protocol_major,
        protocol_minor,
        message_type,
        flags,
        payload_length,
        correlation_id: u128::from_be_bytes(correlation),
    })
}

/// Read one frame from a stream, allocating only after the header is trusted.
pub fn read_frame<R: Read>(
    reader: &mut R,
    max_frame_bytes: u32,
) -> Result<(FrameHeader, Vec<u8>), FrameError> {
    let mut header_bytes = [0_u8; HEADER_BYTES];
    read_exact(reader, &mut header_bytes)?;

    let header = parse_header(&header_bytes, max_frame_bytes)?;
    let mut payload = vec![0_u8; header.payload_length as usize];
    read_exact(reader, &mut payload)?;

    Ok((header, payload))
}

/// Fill a buffer, distinguishing a short stream from a broken one.
fn read_exact<R: Read>(reader: &mut R, buffer: &mut [u8]) -> Result<(), FrameError> {
    let mut filled = 0;

    while filled < buffer.len() {
        let Some(slice) = buffer.get_mut(filled..) else {
            return Err(FrameError::Truncated);
        };

        match reader.read(slice) {
            Ok(0) => return Err(FrameError::Truncated),
            Ok(count) => filled += count,
            Err(_) => return Err(FrameError::Io),
        }
    }

    Ok(())
}

/// Write one frame and flush it.
pub fn write_frame<W: Write>(
    writer: &mut W,
    header: &FrameHeader,
    payload: &[u8],
) -> Result<(), FrameError> {
    let frame = encode_frame(header, payload);

    writer.write_all(&frame).map_err(|_| FrameError::Io)?;
    writer.flush().map_err(|_| FrameError::Io)
}

/// Compare two credentials without leaking their difference through timing.
///
/// A byte-by-byte early return would let a caller learn the credential one byte
/// at a time. This is not a cryptographic primitive; it is the minimum a
/// comparison of secret material owes (`612-security-model.md`).
#[must_use]
pub fn credentials_match(expected: &[u8], presented: &[u8]) -> bool {
    if expected.len() != presented.len() {
        return false;
    }

    let mut difference = 0_u8;
    for (left, right) in expected.iter().zip(presented.iter()) {
        difference |= left ^ right;
    }

    difference == 0
}

/// What the engine decided about a `Hello`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeOutcome {
    /// The client is accepted on the negotiated version.
    Accepted(Welcome),
    /// The client is refused, with a stable reason.
    Refused(Reject),
}

/// Decide whether to accept a `Hello` (`108` sections 6, 8, and 11).
///
/// `expected_credential` is the ephemeral value this engine was launched with.
#[must_use]
pub fn evaluate_hello(
    hello: &Hello,
    expected_credential: &[u8],
    engine_session_id: &str,
    engine_ready: bool,
) -> HandshakeOutcome {
    // Authentication first: an unauthenticated peer learns nothing else, not even
    // whether its protocol version would have been acceptable.
    let presented = decode_hex(&hello.credential);
    let authenticated = presented
        .as_deref()
        .is_some_and(|presented| credentials_match(expected_credential, presented));

    if !authenticated {
        return HandshakeOutcome::Refused(Reject {
            reason: RejectReason::Unauthenticated,
            // Deliberately identical for a missing and a wrong credential.
            detail: "the launch credential was not accepted".to_owned(),
            protocol_major: PROTOCOL_VERSION_MAJOR,
        });
    }

    if hello.protocol_major_min > hello.protocol_major_max {
        return HandshakeOutcome::Refused(Reject {
            reason: RejectReason::MalformedHello,
            detail: "the supported major version range is inverted".to_owned(),
            protocol_major: PROTOCOL_VERSION_MAJOR,
        });
    }

    // 108 section 8.1: a major mismatch is an incompatible protocol.
    if PROTOCOL_VERSION_MAJOR < hello.protocol_major_min
        || PROTOCOL_VERSION_MAJOR > hello.protocol_major_max
    {
        return HandshakeOutcome::Refused(Reject {
            reason: RejectReason::ProtocolMajorMismatch,
            detail: format!("this engine speaks protocol major {PROTOCOL_VERSION_MAJOR}"),
            protocol_major: PROTOCOL_VERSION_MAJOR,
        });
    }

    // Only the roles that may open a control connection today.
    if !matches!(
        hello.role,
        HelloRole::Shell | HelloRole::ControlUi | HelloRole::Test
    ) {
        return HandshakeOutcome::Refused(Reject {
            reason: RejectReason::RoleNotPermitted,
            detail: "this role may not open a control connection".to_owned(),
            protocol_major: PROTOCOL_VERSION_MAJOR,
        });
    }

    if !engine_ready {
        return HandshakeOutcome::Refused(Reject {
            reason: RejectReason::EngineNotReady,
            detail: "the engine is not accepting connections yet".to_owned(),
            protocol_major: PROTOCOL_VERSION_MAJOR,
        });
    }

    // 108 section 8.2: the highest mutually supported minor version. The lint
    // fires only because this build's minor is currently 0, which makes the
    // comparison provably a no-op today; it stops being one at minor 1.
    #[allow(clippy::unnecessary_min_or_max)]
    let minor = PROTOCOL_VERSION_MINOR.min(hello.protocol_minor_max);

    HandshakeOutcome::Accepted(Welcome {
        protocol_major: PROTOCOL_VERSION_MAJOR,
        protocol_minor: minor,
        engine_session_id: engine_session_id.to_owned(),
        max_frame_bytes: DEFAULT_MAX_FRAME_BYTES,
    })
}

/// Encode bytes as lowercase hex.
#[must_use]
pub fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Decode lowercase or uppercase hex, rejecting anything malformed.
#[must_use]
pub fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }

    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(text.len() / 2);
    let mut index = 0;

    while index < bytes.len() {
        let high = (*bytes.get(index)? as char).to_digit(16)?;
        let low = (*bytes.get(index + 1)? as char).to_digit(16)?;
        out.push(u8::try_from(high * 16 + low).ok()?);
        index += 2;
    }

    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: &str = "0000000000000000000000000000002a";

    fn hello(credential: &str) -> Hello {
        Hello {
            role: HelloRole::Shell,
            protocol_major_min: 1,
            protocol_major_max: 1,
            protocol_minor_max: 0,
            credential: credential.to_owned(),
            build_id: "test".to_owned(),
        }
    }

    fn header(message_type: MessageType, payload_length: u32) -> FrameHeader {
        FrameHeader {
            protocol_major: PROTOCOL_VERSION_MAJOR,
            protocol_minor: PROTOCOL_VERSION_MINOR,
            message_type,
            flags: 0,
            payload_length,
            correlation_id: 7,
        }
    }

    #[test]
    fn a_frame_round_trips_through_a_stream() {
        let payload = br#"{"role":"shell"}"#;
        let frame = encode_frame(&header(MessageType::Hello, payload.len() as u32), payload);
        let mut cursor = std::io::Cursor::new(frame);

        let read = read_frame(&mut cursor, DEFAULT_MAX_FRAME_BYTES);
        let (parsed, body) = read.unwrap_or((header(MessageType::Reject, 0), Vec::new()));

        assert_eq!(parsed.message_type, MessageType::Hello);
        assert_eq!(parsed.correlation_id, 7);
        assert_eq!(body, payload);
    }

    #[test]
    fn an_oversized_frame_is_refused_before_it_is_allocated_for() {
        // 108 section 9 and section 18: never allocate on an untrusted length.
        let mut bytes = encode_frame(&header(MessageType::Hello, 0), &[]);
        bytes.splice(12..16, 0x00FF_FFFF_u32.to_be_bytes());

        let parsed = parse_header(&bytes, DEFAULT_MAX_FRAME_BYTES);

        assert_eq!(
            parsed,
            Err(FrameError::TooLarge {
                declared: 0x00FF_FFFF,
                limit: DEFAULT_MAX_FRAME_BYTES,
            })
        );
    }

    #[test]
    fn a_foreign_stream_is_refused() {
        let mut bytes = encode_frame(&header(MessageType::Hello, 0), &[]);
        bytes.splice(0..4, *b"HTTP");

        assert_eq!(
            parse_header(&bytes, DEFAULT_MAX_FRAME_BYTES),
            Err(FrameError::BadMagic)
        );
    }

    #[test]
    fn an_unknown_message_type_is_refused() {
        let mut bytes = encode_frame(&header(MessageType::Hello, 0), &[]);
        bytes.splice(8..10, 9999_u16.to_be_bytes());

        assert_eq!(
            parse_header(&bytes, DEFAULT_MAX_FRAME_BYTES),
            Err(FrameError::UnknownMessageType(9999))
        );
    }

    #[test]
    fn a_truncated_stream_is_refused() {
        let payload = b"{}";
        let frame = encode_frame(&header(MessageType::Hello, payload.len() as u32), payload);
        let mut cursor = std::io::Cursor::new(frame[..HEADER_BYTES + 1].to_vec());

        assert_eq!(
            read_frame(&mut cursor, DEFAULT_MAX_FRAME_BYTES).err(),
            Some(FrameError::Truncated)
        );
        assert_eq!(
            parse_header(&[0_u8; 4], DEFAULT_MAX_FRAME_BYTES),
            Err(FrameError::Truncated)
        );
    }

    #[test]
    fn hex_round_trips_and_rejects_malformed_input() {
        assert_eq!(encode_hex(&[0x0a, 0xff]), "0aff");
        assert_eq!(decode_hex("0aff"), Some(vec![0x0a, 0xff]));
        assert_eq!(decode_hex("0af"), None);
        assert_eq!(decode_hex("zz"), None);
    }

    #[test]
    fn credential_comparison_is_length_safe_and_constant_time() {
        assert!(credentials_match(b"secret", b"secret"));
        assert!(!credentials_match(b"secret", b"secrey"));
        assert!(!credentials_match(b"secret", b"secre"));
        assert!(!credentials_match(b"", b"x"));
    }

    #[test]
    fn a_correct_credential_is_accepted_and_negotiates_the_minor_version() {
        let outcome = evaluate_hello(&hello("0a0b"), &[0x0a, 0x0b], SESSION, true);

        match outcome {
            HandshakeOutcome::Accepted(welcome) => {
                assert_eq!(welcome.protocol_major, PROTOCOL_VERSION_MAJOR);
                assert_eq!(welcome.protocol_minor, PROTOCOL_VERSION_MINOR);
                assert_eq!(welcome.engine_session_id, SESSION);
                assert_eq!(welcome.max_frame_bytes, DEFAULT_MAX_FRAME_BYTES);
            }
            HandshakeOutcome::Refused(reject) => {
                assert_eq!(
                    Some(reject.reason),
                    None,
                    "expected acceptance, got a refusal"
                );
            }
        }
    }

    #[test]
    fn a_client_supporting_a_lower_minor_gets_that_minor() {
        // 108 section 8.2: the highest mutually supported minor version.
        let mut hello = hello("0a0b");
        hello.protocol_minor_max = 0;

        let outcome = evaluate_hello(&hello, &[0x0a, 0x0b], SESSION, true);

        assert!(matches!(
            outcome,
            HandshakeOutcome::Accepted(ref welcome) if welcome.protocol_minor == 0
        ));
    }

    #[test]
    fn a_wrong_credential_is_refused_without_saying_why() {
        let wrong = evaluate_hello(&hello("0a0c"), &[0x0a, 0x0b], SESSION, true);
        let missing = evaluate_hello(&hello(""), &[0x0a, 0x0b], SESSION, true);

        let detail_of = |outcome: HandshakeOutcome| match outcome {
            HandshakeOutcome::Refused(reject) => {
                (reject.reason, reject.detail, reject.protocol_major)
            }
            HandshakeOutcome::Accepted(_) => {
                (RejectReason::EngineNotReady, "accepted".to_owned(), 0)
            }
        };

        let wrong = detail_of(wrong);
        let missing = detail_of(missing);

        assert_eq!(wrong.0, RejectReason::Unauthenticated);
        assert_eq!(missing.0, RejectReason::Unauthenticated);
        // A wrong credential and a missing one must be indistinguishable.
        assert_eq!(wrong.1, missing.1);
        assert!(!wrong.1.contains("0a0"));
    }

    #[test]
    fn a_major_mismatch_is_refused_and_says_what_the_engine_speaks() {
        let mut hello = hello("0a0b");
        hello.protocol_major_min = 2;
        hello.protocol_major_max = 3;

        match evaluate_hello(&hello, &[0x0a, 0x0b], SESSION, true) {
            HandshakeOutcome::Refused(reject) => {
                assert_eq!(reject.reason, RejectReason::ProtocolMajorMismatch);
                assert_eq!(reject.protocol_major, PROTOCOL_VERSION_MAJOR);
            }
            HandshakeOutcome::Accepted(_) => unreachable!("a major mismatch must be refused"),
        }
    }

    #[test]
    fn authentication_is_checked_before_the_protocol_version() {
        // An unauthenticated peer must not learn whether its version would fit.
        let mut hello = hello("dead");
        hello.protocol_major_min = 9;
        hello.protocol_major_max = 9;

        match evaluate_hello(&hello, &[0x0a, 0x0b], SESSION, true) {
            HandshakeOutcome::Refused(reject) => {
                assert_eq!(reject.reason, RejectReason::Unauthenticated);
            }
            HandshakeOutcome::Accepted(_) => unreachable!("expected refusal"),
        }
    }

    #[test]
    fn an_inverted_version_range_is_malformed() {
        let mut hello = hello("0a0b");
        hello.protocol_major_min = 5;
        hello.protocol_major_max = 1;

        match evaluate_hello(&hello, &[0x0a, 0x0b], SESSION, true) {
            HandshakeOutcome::Refused(reject) => {
                assert_eq!(reject.reason, RejectReason::MalformedHello);
            }
            HandshakeOutcome::Accepted(_) => unreachable!("expected refusal"),
        }
    }

    #[test]
    fn an_engine_that_is_not_ready_refuses_rather_than_pretending() {
        match evaluate_hello(&hello("0a0b"), &[0x0a, 0x0b], SESSION, false) {
            HandshakeOutcome::Refused(reject) => {
                assert_eq!(reject.reason, RejectReason::EngineNotReady);
            }
            HandshakeOutcome::Accepted(_) => unreachable!("expected refusal"),
        }
    }

    #[test]
    fn an_extension_host_may_not_open_a_control_connection() {
        let mut hello = hello("0a0b");
        hello.role = HelloRole::ExtensionHost;

        match evaluate_hello(&hello, &[0x0a, 0x0b], SESSION, true) {
            HandshakeOutcome::Refused(reject) => {
                assert_eq!(reject.reason, RejectReason::RoleNotPermitted);
            }
            HandshakeOutcome::Accepted(_) => unreachable!("expected refusal"),
        }
    }

    #[test]
    fn contracts_round_trip_as_json() {
        // ADR-0067: the payload is JSON generated from the canonical schema.
        let welcome = Welcome {
            protocol_major: 1,
            protocol_minor: 0,
            engine_session_id: SESSION.to_owned(),
            max_frame_bytes: DEFAULT_MAX_FRAME_BYTES,
        };

        let encoded = serde_json::to_string(&welcome).unwrap_or_default();

        assert!(
            encoded.contains("\"engineSessionId\""),
            "wire names are camelCase"
        );
        assert_eq!(
            serde_json::from_str::<Welcome>(&encoded).ok(),
            Some(welcome)
        );
    }

    #[test]
    fn an_unknown_field_is_rejected_rather_than_ignored() {
        // The schema says additionalProperties: false, so the decoder must agree.
        let payload = r#"{"protocolMajor":1,"protocolMinor":0,
            "engineSessionId":"a","maxFrameBytes":1024,"extra":true}"#;

        assert!(serde_json::from_str::<Welcome>(payload).is_err());
    }
}

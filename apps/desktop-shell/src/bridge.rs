//! The typed bridge between the control UI and the shell.
//!
//! Canonical documentation: `docs/05-platform/501-desktop-shell.md` sections 3
//! and 13, `docs/01-runtime/109-ui-engine-synchronization.md`, ADR-0068.
//!
//! # The shape of this, and why
//!
//! There were two ways to build it. The shell could relay engine frames
//! verbatim — a transparent pipe between the page and the engine socket — or it
//! could terminate every message, validate it against a closed set of typed
//! requests, and speak to the engine itself.
//!
//! Relaying loses on three counts, and each one is a rule rather than a
//! preference. A page that can put bytes on the engine socket can send *any*
//! frame, including a `Hello` carrying a credential it chose, which is not the
//! narrow, permission-aware bridge `501` section 13 asks for. The launch
//! credential must never cross the bridge (`501` invariant 4), so the shell
//! would have to inject it — and injecting into an opaque frame means parsing
//! the frame, at which point it is not a relay. And ADR-0068 says the webview
//! never reaches the engine socket directly; a pipe is a longer socket.
//!
//! So the bridge terminates. The page names one of a closed set of requests, the
//! shell answers from what it knows or asks the engine on the page's behalf, and
//! nothing the page sends is ever forwarded as protocol.
//!
//! Everything here is pure. The transport is [`crate::ui_host`]'s problem, which
//! means every rejection below is tested without a window.

use mirae_contracts::generated::{BridgeRequest, BridgeRequestKind, BridgeResponse};

use crate::project_session::ProjectSession;

/// Longest message accepted from the page.
///
/// The page chooses the length, so it is bounded before it is parsed. A request
/// is a request id and an enumerated kind; anything approaching this is not a
/// request that got large, it is something else.
pub(crate) const MAX_REQUEST_BYTES: usize = 4096;

/// Why a request was refused.
///
/// Categories, never text taken from the message. A page under someone else's
/// control chooses what it sends, and echoing that into a diagnostic is how it
/// reaches a log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BridgeError {
    /// The message was longer than [`MAX_REQUEST_BYTES`].
    TooLarge,
    /// The message was not a well-formed request.
    Malformed,
    /// The engine is not connected, and the answer would require it.
    EngineUnavailable,
    /// No project is open, and the request needs one.
    NoProjectOpen,
    /// A project is already open, and the request would replace it.
    ///
    /// `102` section 5 and `CreateProject`'s own rule: replacing an open project
    /// is a close followed by a create, deliberately. Doing it silently would
    /// leave a user's next edit in a project they did not mean.
    ProjectAlreadyOpen,
    /// The command refused the request.
    ///
    /// Carries the command's own category, so the page can show it against the
    /// control that caused it (`109` section 8).
    Command(&'static str),
}

impl BridgeError {
    /// A stable code for the wire and for diagnostics.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::TooLarge => "request_too_large",
            Self::Malformed => "malformed_request",
            Self::EngineUnavailable => "engine_unavailable",
            Self::NoProjectOpen => "no_project_open",
            Self::ProjectAlreadyOpen => "project_already_open",
            Self::Command(code) => code,
        }
    }
}

/// What the shell knows about the engine right now.
///
/// `501` section 6 forbids the shell fabricating engine state while
/// disconnected, so this is an enum rather than a struct with a `connected`
/// flag: there is no way to hold session details and claim to be disconnected,
/// or to claim connection without them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EngineView {
    /// No authenticated connection.
    Disconnected,
    /// Connected, with what the handshake established.
    Connected {
        /// The engine session.
        session_id: String,
        /// Negotiated major version.
        protocol_major: u16,
        /// Negotiated minor version.
        protocol_minor: u16,
        /// The newest committed generation the shell has observed.
        state_generation: u64,
    },
}

/// Parse a message from the page.
pub(crate) fn parse_request(message: &str) -> Result<BridgeRequest, BridgeError> {
    if message.len() > MAX_REQUEST_BYTES {
        return Err(BridgeError::TooLarge);
    }

    // `deny_unknown_fields` comes from the schema's closed contract, so a
    // request carrying an extra field is refused rather than partially honoured.
    serde_json::from_str::<BridgeRequest>(message).map_err(|_| BridgeError::Malformed)
}

/// Answer a request, acting on the session where the request asks for it.
pub(crate) fn answer(
    request: &BridgeRequest,
    engine: &EngineView,
    session: &mut Option<ProjectSession>,
) -> BridgeResponse {
    match request.kind {
        BridgeRequestKind::EngineStatus | BridgeRequestKind::ProjectState => {
            status_response(&request.request_id, engine, session.as_ref())
        }
        BridgeRequestKind::CreateProject => create_project(request, engine, session),
        BridgeRequestKind::SaveProject => save_open_project(request, engine, session),
    }
}

/// Create a project, or say why not.
fn create_project(
    request: &BridgeRequest,
    engine: &EngineView,
    session: &mut Option<ProjectSession>,
) -> BridgeResponse {
    if session.is_some() {
        return refusal(
            &request.request_id,
            BridgeError::ProjectAlreadyOpen,
            engine,
            session.as_ref(),
        );
    }

    let EngineView::Connected { session_id, .. } = engine else {
        return refusal(
            &request.request_id,
            BridgeError::EngineUnavailable,
            engine,
            session.as_ref(),
        );
    };

    // The name belongs to the page, so it is validated by the command rather
    // than here. Two rules would eventually disagree, and the command is the one
    // that has to be right.
    let name = request.name.clone().unwrap_or_default();

    match ProjectSession::create(&name, session_id) {
        Ok(created) => {
            *session = Some(created);
            status_response(&request.request_id, engine, session.as_ref())
        }
        Err(error) => refusal(
            &request.request_id,
            BridgeError::Command(error.as_str()),
            engine,
            session.as_ref(),
        ),
    }
}

/// Save the open project, or say why not.
fn save_open_project(
    request: &BridgeRequest,
    engine: &EngineView,
    session: &mut Option<ProjectSession>,
) -> BridgeResponse {
    let Some(open) = session.as_mut() else {
        return refusal(
            &request.request_id,
            BridgeError::NoProjectOpen,
            engine,
            None,
        );
    };

    // A first save needs somewhere to go. There is no file chooser yet, so it
    // goes to a real location under the application data directory rather than
    // to a prompt that does not exist. A chooser is a later ticket and changes
    // this one call.
    let destination = open.path().cloned().or_else(default_save_path);
    let outcome = open.save(destination);

    match outcome {
        Ok(()) => status_response(&request.request_id, engine, session.as_ref()),
        Err(failure) => {
            let code = failure.as_str();
            refusal(
                &request.request_id,
                BridgeError::Command(code),
                engine,
                session.as_ref(),
            )
        }
    }
}

/// Where a project that has never been saved is written.
///
/// The shell invents this path, so the shell makes it exist. `save_project`
/// deliberately refuses a destination whose directory is missing — for a path a
/// user chose, a missing directory is a mistake worth reporting — but a location
/// nobody has seen yet is this function's responsibility to create.
fn default_save_path() -> Option<std::path::PathBuf> {
    let directory = mirae_platform::local_data_directory()?.join("projects");

    // A failure here leaves the path pointing at a directory that does not
    // exist, and the save reports that rather than pretending it wrote
    // something.
    let _ = std::fs::create_dir_all(&directory);

    Some(directory.join("untitled.mirae.json"))
}

/// Build an answer describing where the engine and the project stand.
fn status_response(
    request_id: &str,
    engine: &EngineView,
    session: Option<&ProjectSession>,
) -> BridgeResponse {
    // Empty rather than the last session seen: a stale session id would let a
    // client mistake a previous engine for the current one.
    let (engine_connected, engine_session_id, protocol_major, protocol_minor, state_generation) =
        match engine {
            EngineView::Disconnected => (false, String::new(), 0, 0, 0),
            EngineView::Connected {
                session_id,
                protocol_major,
                protocol_minor,
                state_generation,
            } => (
                true,
                session_id.clone(),
                *protocol_major,
                *protocol_minor,
                *state_generation,
            ),
        };

    // Every project field is empty when none is open, for the same reason. A
    // closed project is not a project with stale values.
    let (project_open, project_name, project_path, project_dirty, saved_generation) = match session
    {
        None => (false, String::new(), String::new(), false, 0),
        Some(session) => (
            true,
            session.name().to_owned(),
            session
                .path()
                .map(|path| path.display().to_string())
                .unwrap_or_default(),
            session.save_state().is_dirty(),
            session
                .save_state()
                .saved()
                .map(mirae_types::StateGeneration::get)
                .unwrap_or(0),
        ),
    };

    BridgeResponse {
        request_id: request_id.to_owned(),
        ok: true,
        error_code: String::new(),
        engine_connected,
        engine_session_id,
        protocol_major,
        protocol_minor,
        project_open,
        project_name,
        project_path,
        project_dirty,
        saved_generation,
        state_generation,
    }
}

/// Build a refusal that still tells the page where the engine stands.
///
/// A refused request is not a reason to withhold connection state: the UI needs
/// it in order to show why the request failed, and `501` section 10 asks for a
/// UI failure to be distinguishable from an engine failure.
pub(crate) fn refusal(
    request_id: &str,
    error: BridgeError,
    engine: &EngineView,
    session: Option<&ProjectSession>,
) -> BridgeResponse {
    let mut response = status_response(request_id, engine, session);
    response.ok = false;
    response.error_code = error.as_str().to_owned();
    response
}

/// Serialize a response for delivery to the page.
///
/// Falls back to a minimal refusal that cannot itself fail to serialize. The
/// page is waiting on a request id; leaving it waiting because the answer would
/// not encode is worse than telling it the shell went wrong.
pub(crate) fn encode_response(response: &BridgeResponse) -> String {
    serde_json::to_string(response).unwrap_or_else(|_| {
        format!(
            r#"{{"requestId":"","ok":false,"errorCode":"{}","engineConnected":false,"engineSessionId":"","protocolMajor":0,"protocolMinor":0,"projectOpen":false,"projectName":"","projectPath":"","projectDirty":false,"savedGeneration":0,"stateGeneration":0}}"#,
            BridgeError::Malformed.as_str()
        )
    })
}

/// The script that delivers a response to the page.
///
/// The response is embedded as a JSON string literal and parsed on the other
/// side, rather than interpolated as an object. That is the difference between
/// data and code: a value that somehow contained a quote would, spliced
/// directly, become script running in the control UI's own context — the exact
/// thing the content security policy in [`crate::navigation`] exists to prevent.
pub(crate) fn delivery_script(response: &BridgeResponse) -> String {
    let payload =
        serde_json::to_string(&encode_response(response)).unwrap_or_else(|_| "\"{}\"".to_owned());

    // Optional chaining: a page that has not installed a listener — or has been
    // replaced by a reload mid-request — must not throw into the webview.
    format!("window.__mirae?.receive?.({payload});")
}

/// Handle one message end to end.
pub(crate) fn handle(
    message: &str,
    engine: &EngineView,
    session: &mut Option<ProjectSession>,
) -> BridgeResponse {
    match parse_request(message) {
        Ok(request) => answer(&request, engine, session),
        // A malformed message has no request id to echo, so the page correlates
        // by the empty one and knows the failure was not about a request it can
        // name.
        Err(error) => refusal("", error, engine, session.as_ref()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: &str = "0000000000000000000000000000002a";

    fn connected() -> EngineView {
        EngineView::Connected {
            session_id: SESSION.to_owned(),
            protocol_major: 1,
            protocol_minor: 0,
            state_generation: 7,
        }
    }

    fn request(kind: &str) -> String {
        format!(r#"{{"requestId":"r-1","kind":"{kind}"}}"#)
    }

    #[test]
    fn a_status_request_is_answered_from_what_the_shell_knows() {
        let response = handle(&request("engineStatus"), &connected(), &mut None);

        assert!(response.ok);
        assert_eq!(response.request_id, "r-1");
        assert!(response.engine_connected);
        assert_eq!(response.engine_session_id, SESSION);
        assert_eq!(response.protocol_major, 1);
        assert_eq!(response.state_generation, 7);
    }

    #[test]
    fn a_disconnected_shell_reports_disconnected_rather_than_stale_detail() {
        // 501 section 6: the shell must not fabricate engine state while
        // disconnected. Reporting the last session it saw would be exactly that.
        let response = handle(
            &request("engineStatus"),
            &EngineView::Disconnected,
            &mut None,
        );

        assert!(response.ok, "the shell can answer this without the engine");
        assert!(!response.engine_connected);
        assert!(response.engine_session_id.is_empty());
        assert_eq!(response.protocol_major, 0);
        assert_eq!(response.state_generation, 0);
    }

    #[test]
    fn an_unknown_request_kind_is_refused() {
        // The closed set is the bridge's narrowness. A page that can name only
        // these cannot ask for anything else, whatever it is running.
        for kind in ["deleteEverything", "engine_status", "", "ENGINESTATUS"] {
            let response = handle(&request(kind), &connected(), &mut None);

            assert!(!response.ok, "{kind} should be refused");
            assert_eq!(response.error_code, BridgeError::Malformed.as_str());
        }
    }

    #[test]
    fn a_request_with_an_extra_field_is_refused() {
        // The schema is a closed contract, so the generated decoder denies
        // unknown fields. A request that is partly understood must not be partly
        // honoured.
        let smuggled = r#"{"requestId":"r-1","kind":"engineStatus","credential":"stolen"}"#;
        let response = handle(smuggled, &connected(), &mut None);

        assert!(!response.ok);
        assert_eq!(response.error_code, BridgeError::Malformed.as_str());
    }

    #[test]
    fn a_malformed_message_is_refused_without_echoing_it() {
        for message in ["", "not json", "{", "[]", "null", "{\"requestId\":1}"] {
            let response = handle(message, &connected(), &mut None);

            assert!(!response.ok, "{message:?} should be refused");
            assert!(
                !encode_response(&response).contains("not json"),
                "no part of the message reaches the answer"
            );
        }
    }

    #[test]
    fn an_oversized_message_is_refused_before_it_is_parsed() {
        // The page chooses the length, so the bound comes first. Parsing to find
        // out how big something is defeats the bound.
        let huge = format!(
            r#"{{"requestId":"{}","kind":"engineStatus"}}"#,
            "a".repeat(MAX_REQUEST_BYTES)
        );

        assert_eq!(parse_request(&huge), Err(BridgeError::TooLarge));
        assert_eq!(
            handle(&huge, &connected(), &mut None).error_code,
            BridgeError::TooLarge.as_str()
        );
    }

    #[test]
    fn a_refusal_still_reports_where_the_engine_stands() {
        // 501 section 10: the UI has to tell a bridge failure from an engine
        // failure, which it cannot do if a refusal withholds connection state.
        let response = handle("nonsense", &connected(), &mut None);

        assert!(!response.ok);
        assert!(
            response.engine_connected,
            "the engine is fine; the request was not"
        );
    }

    #[test]
    fn no_response_can_carry_a_credential() {
        // 501 invariant 4. Asserted against the serialized form, because that is
        // what actually crosses: the response type has no field for one, and
        // this fails loudly if a later ticket adds one.
        let encoded = encode_response(&handle(&request("engineStatus"), &connected(), &mut None));

        for forbidden in ["credential", "secret", "token", "password"] {
            assert!(
                !encoded.to_ascii_lowercase().contains(forbidden),
                "{forbidden} must not appear in a bridge response"
            );
        }
    }

    #[test]
    fn project_state_reports_no_project_rather_than_an_empty_one() {
        // Returning an empty project would be a fabrication. 109 invariant 1
        // makes the engine authoritative so the UI never invents what it cannot
        // see, and "no project is open" is a thing the UI can show.
        let response = handle(&request("projectState"), &connected(), &mut None);

        assert!(response.ok);
        assert!(!response.project_open);
        assert!(response.project_name.is_empty());
        assert!(response.project_path.is_empty());
        assert!(!response.project_dirty);
        assert_eq!(response.saved_generation, 0);
    }

    // -----------------------------------------------------------------------
    // MIR-0113 — the project commands the control window drives.
    // -----------------------------------------------------------------------

    /// A create request carrying a name.
    fn create(name: &str) -> String {
        format!(r#"{{"requestId":"r-1","kind":"createProject","name":"{name}"}}"#)
    }

    #[test]
    fn creating_a_project_opens_one_and_reports_it() {
        let mut session = None;
        let response = handle(&create("Stream"), &connected(), &mut session);

        assert!(response.ok);
        assert!(response.project_open);
        assert_eq!(response.project_name, "Stream");
        assert!(response.project_dirty, "a project never saved is dirty");
        assert!(response.project_path.is_empty(), "and has no path yet");
        assert!(session.is_some());
    }

    #[test]
    fn creating_a_second_project_is_refused_rather_than_replacing_the_first() {
        // 102 section 5: replacing an open project is a close followed by a
        // create, deliberately. Doing it silently would leave a user's next edit
        // in a project they did not mean.
        let mut session = None;
        let _ = handle(&create("First"), &connected(), &mut session);
        let response = handle(&create("Second"), &connected(), &mut session);

        assert!(!response.ok);
        assert_eq!(
            response.error_code,
            BridgeError::ProjectAlreadyOpen.as_str()
        );
        assert_eq!(
            response.project_name, "First",
            "and the first project is still the open one"
        );
    }

    #[test]
    fn a_name_the_command_refuses_is_reported_with_the_commands_own_category() {
        // The shell does not validate names; `CreateProject` does. Two rules
        // would eventually disagree, and the page needs the one that decided.
        let mut session = None;
        let response = handle(&create(""), &connected(), &mut session);

        assert!(!response.ok);
        assert_eq!(response.error_code, "invalid_argument");
        assert!(session.is_none(), "and nothing was opened");
    }

    #[test]
    fn creating_a_project_without_an_engine_is_refused() {
        let mut session = None;
        let response = handle(&create("Stream"), &EngineView::Disconnected, &mut session);

        assert!(!response.ok);
        assert_eq!(response.error_code, BridgeError::EngineUnavailable.as_str());
        assert!(session.is_none());
    }

    #[test]
    fn saving_without_a_project_is_refused() {
        let mut session = None;
        let response = handle(&request("saveProject"), &connected(), &mut session);

        assert!(!response.ok);
        assert_eq!(response.error_code, BridgeError::NoProjectOpen.as_str());
    }

    #[test]
    fn a_created_project_survives_across_requests() {
        // The session is owned by the window loop, so a status request after a
        // create sees the same project rather than a fresh view.
        let mut session = None;
        let _ = handle(&create("Stream"), &connected(), &mut session);
        let response = handle(&request("engineStatus"), &connected(), &mut session);

        assert!(response.project_open);
        assert_eq!(response.project_name, "Stream");
    }

    #[test]
    fn the_request_id_is_echoed_verbatim_rather_than_interpreted() {
        let odd = r#"{"requestId":"../../etc/passwd","kind":"engineStatus"}"#;
        let response = handle(odd, &connected(), &mut None);

        assert!(response.ok);
        assert_eq!(response.request_id, "../../etc/passwd");
    }

    #[test]
    fn every_error_has_a_stable_lowercase_code() {
        for error in [
            BridgeError::TooLarge,
            BridgeError::Malformed,
            BridgeError::EngineUnavailable,
        ] {
            assert!(!error.as_str().is_empty());
            assert_eq!(error.as_str(), error.as_str().to_ascii_lowercase());
        }
    }
}

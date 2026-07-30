//! The main control window and the system webview that hosts the control UI.
//!
//! Canonical documentation: `docs/05-platform/501-desktop-shell.md` sections 3,
//! 4, 5, and 10, ADR-0068.
//!
//! This is the only module that touches an operating-system SDK. It owns the
//! window and the webview and nothing else: the decisions it enforces come from
//! [`crate::assets`] and [`crate::navigation`], which are pure and tested, so the
//! untestable part stays as thin as a window can be.
//!
//! `501` section 10 requires the shell to tell a UI failure from an engine
//! failure. Both are [`FatalError`], and each one names which half of the
//! application stopped, because "Mirae stopped working" is not something a user
//! can act on.

use std::borrow::Cow;
use std::time::{Duration, Instant};

use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::keyboard::KeyCode;
use tao::platform::run_return::EventLoopExtRunReturn as _;
use tao::window::WindowBuilder;
use wry::http::{Request, Response, header};
use wry::{NewWindowResponse, WebViewBuilder};

use crate::assets::{Resolution, UiResources};
use crate::bridge::{self, EngineView};
use crate::external::open_in_browser;
use crate::navigation::{APP_SCHEME, CONTENT_SECURITY_POLICY, Decision, START_URL, decide};

/// Title of the main control window (`501` section 5: every window has a role).
const WINDOW_TITLE: &str = "Mirae";

/// Initial size of the main control window.
const WINDOW_SIZE: (f64, f64) = (1440.0, 900.0);

/// Smallest size the control UI is laid out for.
const MINIMUM_WINDOW_SIZE: (f64, f64) = (960.0, 600.0);

/// How often the window loop asks whether the engine is still there.
///
/// The loop otherwise sleeps, so this is the longest a crash can go unreported
/// while the window is idle.
const ENGINE_POLL_INTERVAL: Duration = Duration::from_millis(250);

/// A message the window loop has to act on.
///
/// The IPC handler runs off the loop and cannot touch the webview, so it sends
/// one of these through the event-loop proxy and the loop does the talking. That
/// is not ceremony: a webview is not `Send`, and the alternative would be
/// sharing it across threads by force.
#[derive(Debug)]
pub(crate) enum HostEvent {
    /// The page sent a bridge message.
    BridgeRequest(String),
}

/// What the shell believes about the engine, asked from the window loop.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EngineHealth {
    /// The engine is running, or is inside a bounded restart.
    Running,
    /// The engine is gone and supervision has given up.
    Failed(String),
}

/// A failure that ends the session, and which half of the application it came from.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FatalError {
    /// The window or the webview failed. The engine may well be healthy.
    Ui(String),
    /// The engine failed. The window may well be healthy.
    Engine(String),
}

impl FatalError {
    /// A message that names the failing half before it names the cause.
    ///
    /// `501` section 10: a user deciding whether to restart, recover, or report
    /// needs to know which process stopped.
    pub(crate) fn report(&self) -> String {
        match self {
            Self::Ui(reason) => format!(
                "the control UI failed and the engine was stopped: {reason}\n\
                 the engine itself did not report a failure"
            ),
            Self::Engine(reason) => format!(
                "the engine failed and the control UI was closed: {reason}\n\
                 the control UI itself did not report a failure"
            ),
        }
    }
}

/// Open the control window and run until it closes or something fails.
///
/// `engine_health` is polled from the loop rather than owned here, so this
/// module never learns how the engine is supervised and supervision never learns
/// that there is a window.
pub(crate) fn run(
    resources: UiResources,
    engine: EngineView,
    mut engine_health: impl FnMut() -> EngineHealth,
) -> Result<(), FatalError> {
    let mut event_loop = EventLoopBuilder::<HostEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(tao::dpi::LogicalSize::new(WINDOW_SIZE.0, WINDOW_SIZE.1))
        .with_min_inner_size(tao::dpi::LogicalSize::new(
            MINIMUM_WINDOW_SIZE.0,
            MINIMUM_WINDOW_SIZE.1,
        ))
        .build(&event_loop)
        .map_err(|error| {
            FatalError::Ui(format!("the control window could not be created: {error}"))
        })?;

    let webview = WebViewBuilder::new()
        .with_url(START_URL)
        .with_custom_protocol(APP_SCHEME.to_owned(), move |_id, request| {
            serve(&resources, &request)
        })
        // 501 section 4 and invariant 3.
        .with_navigation_handler(|url| match decide(&url) {
            Decision::Allow => true,
            Decision::OpenExternally => {
                let _ = open_in_browser(&url);
                false
            }
            Decision::Block(_) => false,
        })
        // 501 section 13: the bridge is the only thing the page can call, and
        // it is typed. The handler forwards the raw message and decides nothing:
        // parsing, bounding, and refusing all happen in `bridge`, where they are
        // tested without a window.
        .with_ipc_handler(move |request| {
            let _ = proxy.send_event(HostEvent::BridgeRequest(request.into_body()));
        })
        // `window.open` is the same decision reached by another route.
        .with_new_window_req_handler(|url, _features| match decide(&url) {
            Decision::OpenExternally => {
                let _ = open_in_browser(&url);
                NewWindowResponse::Deny
            }
            _ => NewWindowResponse::Deny,
        })
        // Restricted permissions (`501` section 3). The control UI reads and
        // writes through the bridge, so it needs none of these.
        .with_clipboard(false)
        .with_autoplay(false)
        .with_hotkeys_zoom(false)
        .with_back_forward_navigation_gestures(false)
        .with_download_started_handler(|_url, _path| false)
        // Dropped content is classified by the shell (`501` section 9 and
        // invariant 7), never opened by the page that received it.
        .with_drag_drop_handler(|_event| true)
        // 501 section 4: no remote debugging surface in a release build.
        .with_devtools(cfg!(debug_assertions))
        .build(&window)
        .map_err(|error| {
            FatalError::Ui(format!("the system webview could not be created: {error}"))
        })?;

    let mut failure: Option<FatalError> = None;
    let mut engine = engine;

    event_loop.run_return(|event, _target, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + ENGINE_POLL_INTERVAL);

        match event {
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                if let EngineHealth::Failed(reason) = engine_health() {
                    // Report disconnected before exiting. The window is closing,
                    // but a bridge request already in flight still gets answered,
                    // and `501` section 6 forbids answering it with session
                    // details the shell can no longer observe.
                    engine = EngineView::Disconnected;
                    failure = Some(FatalError::Engine(reason));
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::UserEvent(HostEvent::BridgeRequest(message)) => {
                let response = bridge::handle(&message, &engine);
                let script = bridge::delivery_script(&response);
                let _ = webview.evaluate_script(&script);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            // 501 section 12 requires a UI reload. Assets are read from disk per
            // request, so a reload always shows what is packaged now.
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } if event.physical_key == KeyCode::F5 => {
                let _ = webview.reload();
            }
            _ => {}
        }
    });

    match failure {
        Some(failure) => Err(failure),
        None => Ok(()),
    }
}

/// Answer one custom-protocol request from the packaged resources.
fn serve(resources: &UiResources, request: &Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    let path = request.uri().path().to_owned();

    match resources.resolve(&path) {
        Resolution::Found {
            bytes,
            content_type,
        } => respond(200, content_type, Cow::Owned(bytes)),
        Resolution::NotFound => respond(
            404,
            "text/plain; charset=utf-8",
            Cow::Borrowed(b"not found".as_slice()),
        ),
        Resolution::Refused(_) => {
            // The reason stays out of the response: the page asked for something
            // it may not have, and telling it why is telling it how.
            respond(
                403,
                "text/plain; charset=utf-8",
                Cow::Borrowed(b"refused".as_slice()),
            )
        }
    }
}

/// Build a response carrying the security headers every packaged reply needs.
fn respond(
    status: u16,
    content_type: &str,
    body: Cow<'static, [u8]>,
) -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_SECURITY_POLICY, CONTENT_SECURITY_POLICY)
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .header(header::REFERRER_POLICY, "no-referrer")
        .body(body)
        .unwrap_or_else(|_| Response::new(Cow::Borrowed(b"".as_slice())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ui_failure_and_an_engine_failure_read_differently() {
        // 501 section 10: the report must let a user tell which half stopped.
        let ui = FatalError::Ui("the system webview could not be created".to_owned());
        let engine = FatalError::Engine("the engine exited three times".to_owned());

        assert!(ui.report().contains("the control UI failed"));
        assert!(
            ui.report()
                .contains("the engine itself did not report a failure")
        );
        assert!(engine.report().contains("the engine failed"));
        assert!(
            engine
                .report()
                .contains("the control UI itself did not report a failure")
        );
        assert_ne!(ui.report(), engine.report());
    }

    #[test]
    fn every_packaged_response_carries_the_security_headers() {
        let response = respond(
            200,
            "text/html; charset=utf-8",
            Cow::Borrowed(b"<!doctype html>".as_slice()),
        );

        assert_eq!(
            response.headers().get(header::CONTENT_SECURITY_POLICY),
            Some(
                &CONTENT_SECURITY_POLICY
                    .parse()
                    .unwrap_or(header::HeaderValue::from_static(""))
            )
        );
        assert_eq!(
            response
                .headers()
                .get(header::X_CONTENT_TYPE_OPTIONS)
                .and_then(|value| value.to_str().ok()),
            Some("nosniff")
        );
    }

    #[test]
    fn a_refused_path_answers_without_saying_why() {
        let root = std::env::temp_dir().join(format!("mirae-shell-serve-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&root);
        let resources = UiResources::rooted_at(root.clone());

        let request = Request::builder()
            .uri("mirae://localhost/../secret")
            .body(Vec::new())
            .unwrap_or_else(|_| Request::new(Vec::new()));
        let response = serve(&resources, &request);

        assert_eq!(response.status(), 403);
        assert_eq!(response.body().as_ref(), b"refused");

        let _ = std::fs::remove_dir_all(&root);
    }
}

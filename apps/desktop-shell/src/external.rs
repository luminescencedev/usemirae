//! Handing an approved external address to the operating system browser.
//!
//! Canonical documentation: `docs/05-platform/501-desktop-shell.md` section 4.
//!
//! The control UI never navigates away from the package, so an external link
//! leaves through the browser the user already trusts. The address reaches a
//! process launcher, so it is validated by [`crate::navigation::decide`] first
//! and passed as one argument to a program that takes no shell: nothing here
//! goes through `cmd`, `sh`, or any string a shell would re-split.

use std::process::{Command, Stdio};

use crate::navigation::{Decision, decide};

/// The program and arguments that open `url` on this platform.
///
/// Split out from [`open_in_browser`] so the argument shape is testable without
/// launching a browser during `cargo test`.
fn command_for(url: &str) -> (&'static str, Vec<String>) {
    if cfg!(windows) {
        // `rundll32 url.dll,FileProtocolHandler` opens the default handler
        // without a shell. `cmd /C start` would work too, and would also let
        // `&` in a query string split the command line.
        (
            "rundll32.exe",
            vec!["url.dll,FileProtocolHandler".to_owned(), url.to_owned()],
        )
    } else if cfg!(target_os = "macos") {
        ("open", vec![url.to_owned()])
    } else {
        ("xdg-open", vec![url.to_owned()])
    }
}

/// Open `url` in the operating system browser.
///
/// Refuses anything the navigation policy would not hand outward, so a page
/// cannot use this as a way to launch an address the webview itself is not
/// allowed to reach.
pub(crate) fn open_in_browser(url: &str) -> Result<(), &'static str> {
    if decide(url) != Decision::OpenExternally {
        return Err("the shell only opens approved external addresses");
    }

    let (program, arguments) = command_for(url);

    Command::new(program)
        .args(&arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|_| "the operating system browser could not be launched")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_address_is_one_argument_to_a_program_that_takes_no_shell() {
        let (program, arguments) = command_for("https://mirae.example/docs?a=1&b=2");

        assert!(!program.contains("cmd") && !program.contains("sh"));
        assert!(
            arguments
                .iter()
                .any(|argument| argument == "https://mirae.example/docs?a=1&b=2"),
            "the address must survive as a single argument"
        );
    }

    #[test]
    fn only_an_approved_external_address_is_opened() {
        // Anything the navigation policy blocks or keeps inside the package must
        // not reach a process launcher.
        for url in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "http://example.test/",
            "mirae://localhost/index.html",
            "https://example.test/\r\nmalicious",
        ] {
            assert_eq!(
                open_in_browser(url),
                Err("the shell only opens approved external addresses"),
                "{url} must not be launched"
            );
        }
    }
}

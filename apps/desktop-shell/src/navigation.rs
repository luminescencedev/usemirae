//! Navigation policy and content security policy for the control-UI webview.
//!
//! Canonical documentation: `docs/05-platform/501-desktop-shell.md` section 4 and
//! invariant 3, ADR-0068.
//!
//! `501` invariant 3 is a requirement, not a nicety: the webview may reach the
//! packaged application and nothing else. Every navigation the page attempts —
//! a link, a redirect, `window.open`, a form submission — passes through
//! [`decide`], which is pure so the block can be tested without a window.

/// Scheme registered for the packaged control UI.
///
/// This is the only scheme the shell registers. `501` section 4 requires
/// unregistered custom schemes to be rejected, and since the handler exists for
/// exactly one name, an unregistered scheme never reaches a handler at all.
pub(crate) const APP_SCHEME: &str = "mirae";

/// Host inside the application scheme.
///
/// `wry` rewrites `mirae://localhost/...` to `http://mirae.localhost/...` on
/// Windows, because WebView2 cannot own a scheme outright. Both spellings name
/// the same packaged resources, so both are accepted here and nowhere else.
const APP_HOST: &str = "localhost";

/// The document the window opens on.
pub(crate) const START_URL: &str = "mirae://localhost/index.html";

/// Longest URL considered before it is blocked unparsed.
const MAX_URL_BYTES: usize = 2048;

/// The content security policy sent with every packaged response.
///
/// `default-src 'none'` means each capability below is granted deliberately.
/// Scripts and styles come from the package only: no inline script is allowed,
/// which is why the bundle must not emit one. `connect-src 'none'` matters most
/// — the UI talks to the engine through the shell bridge, never over the
/// network, so a compromised page has nowhere to send what it reads.
pub(crate) const CONTENT_SECURITY_POLICY: &str = "default-src 'none'; \
     script-src 'self'; \
     style-src 'self'; \
     img-src 'self' data:; \
     font-src 'self'; \
     media-src 'self'; \
     connect-src 'none'; \
     form-action 'none'; \
     frame-src 'none'; \
     frame-ancestors 'none'; \
     base-uri 'none'; \
     object-src 'none'";

/// What the shell does with a navigation the page asked for.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Decision {
    /// Stay in the webview: the target is the packaged application.
    Allow,
    /// Leave the webview: hand the address to the operating system browser.
    OpenExternally,
    /// Refuse, with a fixed reason safe to log.
    Block(&'static str),
}

/// Decide what happens to `url`.
pub(crate) fn decide(url: &str) -> Decision {
    if url.len() > MAX_URL_BYTES {
        return Decision::Block("the address is longer than the shell accepts");
    }

    if url.bytes().any(|byte| byte.is_ascii_control()) {
        return Decision::Block("the address contains a control character");
    }

    let lowercase = url.to_ascii_lowercase();

    if is_packaged(&lowercase) {
        return Decision::Allow;
    }

    // An `https` address is the only thing worth handing to the browser. `http`
    // is not: it is either an attempt to impersonate the packaged origin on
    // Windows or a downgrade, and neither deserves a click.
    if lowercase.starts_with("https://") && has_host(&lowercase) {
        return Decision::OpenExternally;
    }

    if lowercase.starts_with("javascript:") || lowercase.starts_with("data:") {
        return Decision::Block("the address would execute in the control UI context");
    }

    if lowercase.starts_with("file://") {
        return Decision::Block("the control UI does not navigate to local files");
    }

    Decision::Block("the address is outside the packaged application")
}

/// Whether `url` names the packaged application, in either platform spelling.
fn is_packaged(lowercase: &str) -> bool {
    let native = format!("{APP_SCHEME}://{APP_HOST}");
    let windows = format!("http://{APP_SCHEME}.{APP_HOST}");

    for prefix in [native, windows] {
        if lowercase == prefix {
            return true;
        }

        if let Some(rest) = lowercase.strip_prefix(&prefix)
            && rest.starts_with('/')
        {
            return true;
        }
    }

    false
}

/// Whether an `https` address actually carries a host.
fn has_host(lowercase: &str) -> bool {
    lowercase
        .trim_start_matches("https://")
        .split(['/', '?', '#'])
        .next()
        .is_some_and(|host| !host.is_empty() && !host.contains(char::is_whitespace))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_packaged_application_is_allowed_in_both_platform_spellings() {
        for url in [
            "mirae://localhost/index.html",
            "mirae://localhost/assets/main.js",
            "mirae://localhost",
            "http://mirae.localhost/index.html",
            "HTTP://MIRAE.LOCALHOST/index.html",
        ] {
            assert_eq!(decide(url), Decision::Allow, "{url} should be allowed");
        }
    }

    #[test]
    fn arbitrary_top_level_navigation_is_blocked() {
        // 501 invariant 3. The webview may not leave the package, whatever the
        // page asks for.
        for url in [
            "http://example.test/",
            "http://mirae.localhost.example.test/",
            "file:///etc/passwd",
            "javascript:fetch('https://example.test')",
            "data:text/html,<script>1</script>",
            "ftp://example.test/",
            "mirae-extension://localhost/index.html",
            "chrome://settings",
        ] {
            assert!(
                matches!(decide(url), Decision::Block(_)),
                "{url} should be blocked"
            );
        }
    }

    #[test]
    fn an_https_link_leaves_through_the_operating_system_browser() {
        // 501 section 4: approved external links open in the OS browser rather
        // than replacing the control UI.
        assert_eq!(
            decide("https://mirae.example/docs"),
            Decision::OpenExternally
        );
    }

    #[test]
    fn a_host_that_only_looks_like_the_package_never_loads_in_the_webview() {
        // `https://mirae.localhost.example.test/` is somebody else's domain. It
        // is not blocked outright — it is an https address like any other — but
        // it must never be treated as the packaged origin, which is the whole
        // point of matching the prefix and then the separator.
        assert_eq!(
            decide("https://mirae.localhost.example.test/"),
            Decision::OpenExternally
        );
        assert!(matches!(
            decide("http://mirae.localhost.example.test/"),
            Decision::Block(_)
        ));
    }

    #[test]
    fn an_https_address_without_a_host_is_blocked() {
        assert!(matches!(decide("https:///docs"), Decision::Block(_)));
    }

    #[test]
    fn an_overlong_or_smuggled_address_is_blocked_unparsed() {
        let long = format!("https://example.test/{}", "a".repeat(MAX_URL_BYTES));

        assert!(matches!(decide(&long), Decision::Block(_)));
        assert!(matches!(
            decide("https://example.test/\r\nHost: elsewhere"),
            Decision::Block(_)
        ));
    }

    #[test]
    fn the_content_security_policy_denies_by_default_and_allows_no_inline_script() {
        assert!(CONTENT_SECURITY_POLICY.starts_with("default-src 'none';"));
        assert!(CONTENT_SECURITY_POLICY.contains("frame-ancestors 'none'"));
        assert!(CONTENT_SECURITY_POLICY.contains("connect-src 'none'"));
        assert!(!CONTENT_SECURITY_POLICY.contains("unsafe-inline"));
        assert!(!CONTENT_SECURITY_POLICY.contains("unsafe-eval"));
    }

    #[test]
    fn the_start_url_is_inside_the_packaged_application() {
        assert_eq!(decide(START_URL), Decision::Allow);
    }
}

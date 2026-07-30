//! The locally packaged control-UI resources served over the custom protocol.
//!
//! Canonical documentation: `docs/05-platform/501-desktop-shell.md` sections 3
//! and 4, ADR-0068.
//!
//! `501` invariant 2 requires the UI to come from local packaged resources, so
//! this module is the only place the shell reads them, and it is the only place
//! a webview-supplied string reaches the filesystem. Everything arriving here is
//! untrusted: the request path is chosen by the page, and a compromised page
//! would use it to walk out of the package. The resolution is therefore total —
//! every path either names a file inside the resource root or is refused — and
//! it is pure, so the refusals are testable without a window.

use std::path::{Path, PathBuf};

/// Environment variable naming the directory holding the built control UI.
///
/// A packaged build points this at its own layout; a developer points it at
/// `apps/control-ui/dist`. There is no compiled-in fallback to a machine-local
/// path, which `cargo xtask policy` forbids and which would ship a developer's
/// directory to users.
pub(crate) const UI_PATH_VARIABLE: &str = "MIRAE_UI_PATH";

/// Directory beside the executable holding the packaged UI.
const PACKAGED_DIRECTORY: &str = "ui";

/// Served when the request names the root of the package.
const ENTRY_DOCUMENT: &str = "index.html";

/// Longest request path accepted before it is refused unread.
///
/// Bounded because the page chooses the length (architecture rule: every input
/// is bounded). Real bundle paths are far below this.
const MAX_REQUEST_PATH_BYTES: usize = 1024;

/// What a request path resolves to.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Resolution {
    /// The file exists inside the package.
    Found {
        /// File contents.
        bytes: Vec<u8>,
        /// The media type derived from the extension.
        content_type: &'static str,
    },
    /// The path was acceptable but names nothing.
    NotFound,
    /// The path was refused before it reached the filesystem.
    ///
    /// The reason is a fixed string chosen here, never a caller-supplied one, so
    /// nothing a page controls can be echoed into a diagnostic.
    Refused(&'static str),
}

/// The root of the packaged control UI.
pub(crate) struct UiResources {
    root: PathBuf,
}

impl UiResources {
    /// Locate the packaged UI, preferring the environment override.
    pub(crate) fn locate() -> Result<Self, String> {
        if let Ok(configured) = std::env::var(UI_PATH_VARIABLE) {
            let root = PathBuf::from(configured);

            return if root.is_dir() {
                Ok(Self::rooted_at(root))
            } else {
                Err(format!(
                    "{UI_PATH_VARIABLE} does not name a directory: {}",
                    root.display()
                ))
            };
        }

        let executable = std::env::current_exe()
            .map_err(|_| "could not locate this executable to find the packaged UI".to_owned())?;
        let directory = executable
            .parent()
            .ok_or_else(|| "this executable has no parent directory".to_owned())?
            .join(PACKAGED_DIRECTORY);

        if directory.is_dir() {
            Ok(Self::rooted_at(directory))
        } else {
            Err(format!(
                "no control UI beside the executable at {} and {UI_PATH_VARIABLE} is not set; \
                 build it with `pnpm --filter @mirae/control-ui build`",
                directory.display()
            ))
        }
    }

    /// Build a resource root at `root`.
    pub(crate) fn rooted_at(root: PathBuf) -> Self {
        Self { root }
    }

    /// The directory being served.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// Resolve one request path from the webview.
    ///
    /// The file is read on every call rather than cached, so reloading the
    /// window shows what is on disk (`501` section 12, UI reload) and so a stale
    /// bundle can never outlive the file it came from.
    pub(crate) fn resolve(&self, request_path: &str) -> Resolution {
        let relative = match normalize(request_path) {
            Ok(relative) => relative,
            Err(reason) => return Resolution::Refused(reason),
        };

        let candidate = self.root.join(&relative);

        // Canonicalizing both sides is the last defence: the segment rules above
        // already reject `..`, and this additionally refuses a symbolic link
        // inside the package that points outside it.
        let (Ok(resolved), Ok(root)) = (candidate.canonicalize(), self.root.canonicalize()) else {
            return Resolution::NotFound;
        };

        if !resolved.starts_with(&root) {
            return Resolution::Refused("the path escapes the packaged resources");
        }

        if !resolved.is_file() {
            return Resolution::NotFound;
        }

        match std::fs::read(&resolved) {
            Ok(bytes) => Resolution::Found {
                bytes,
                content_type: content_type_for(&resolved),
            },
            Err(_) => Resolution::NotFound,
        }
    }
}

/// Turn a request path into a relative path inside the package, or refuse it.
fn normalize(request_path: &str) -> Result<PathBuf, &'static str> {
    if request_path.len() > MAX_REQUEST_PATH_BYTES {
        return Err("the request path is longer than the shell accepts");
    }

    let trimmed = request_path.trim_start_matches('/');
    let trimmed = trimmed.split(['?', '#']).next().unwrap_or("");

    if trimmed.is_empty() {
        return Ok(PathBuf::from(ENTRY_DOCUMENT));
    }

    let decoded = percent_decode(trimmed)?;
    let mut relative = PathBuf::new();

    for segment in decoded.split('/') {
        if segment.is_empty() {
            return Err("the request path has an empty segment");
        }

        if segment == "." || segment == ".." {
            return Err("the request path tries to leave the packaged resources");
        }

        if segment.starts_with('.') {
            return Err("the request path names a hidden entry");
        }

        if !segment.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
        }) {
            return Err("the request path contains a character the shell does not serve");
        }

        relative.push(segment);
    }

    Ok(relative)
}

/// Decode `%XX` escapes, refusing anything that is not a well-formed escape.
///
/// Decoding happens before the segment rules above, so `%2e%2e` is rejected as
/// the `..` it decodes to rather than looked up as a literal filename.
fn percent_decode(path: &str) -> Result<String, &'static str> {
    if !path.contains('%') {
        return Ok(path.to_owned());
    }

    let bytes = path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'%' {
            decoded.push(bytes[index]);
            index += 1;
            continue;
        }

        let Some(escape) = bytes.get(index + 1..index + 3) else {
            return Err("the request path ends inside an escape");
        };

        let high = (escape[0] as char)
            .to_digit(16)
            .ok_or("the request path contains a malformed escape")?;
        let low = (escape[1] as char)
            .to_digit(16)
            .ok_or("the request path contains a malformed escape")?;

        decoded.push(u8::try_from(high * 16 + low).unwrap_or(0));
        index += 3;
    }

    String::from_utf8(decoded).map_err(|_| "the request path decodes to invalid text")
}

/// The media type for a packaged file.
///
/// A type the shell does not know is served as an opaque download rather than
/// guessed, so a mistyped asset can never be interpreted as a script.
fn content_type_for(path: &Path) -> &'static str {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch directory that removes itself.
    struct Scratch {
        path: PathBuf,
    }

    impl Scratch {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("mirae-shell-assets-{}-{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            let _ = std::fs::create_dir_all(&path);
            Self { path }
        }

        fn write(&self, relative: &str, contents: &[u8]) {
            let target = self.path.join(relative);

            if let Some(parent) = target.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            let _ = std::fs::write(target, contents);
        }

        fn resources(&self) -> UiResources {
            UiResources::rooted_at(self.path.clone())
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn the_root_serves_the_entry_document() {
        let scratch = Scratch::new("entry");
        scratch.write("index.html", b"<!doctype html>");

        let resolution = scratch.resources().resolve("/");

        assert_eq!(
            resolution,
            Resolution::Found {
                bytes: b"<!doctype html>".to_vec(),
                content_type: "text/html; charset=utf-8",
            }
        );
    }

    #[test]
    fn a_nested_asset_resolves_with_its_media_type() {
        let scratch = Scratch::new("nested");
        scratch.write("assets/main.js", b"export {};");

        let resolution = scratch.resources().resolve("/assets/main.js");

        assert_eq!(
            resolution,
            Resolution::Found {
                bytes: b"export {};".to_vec(),
                content_type: "text/javascript; charset=utf-8",
            }
        );
    }

    #[test]
    fn a_query_string_and_a_fragment_do_not_change_the_file() {
        let scratch = Scratch::new("query");
        scratch.write("assets/main.css", b"body{}");

        let resolution = scratch.resources().resolve("/assets/main.css?v=1#top");

        assert_eq!(
            resolution,
            Resolution::Found {
                bytes: b"body{}".to_vec(),
                content_type: "text/css; charset=utf-8",
            }
        );
    }

    #[test]
    fn a_missing_file_is_not_found_rather_than_refused() {
        let scratch = Scratch::new("missing");

        assert_eq!(
            scratch.resources().resolve("/absent.js"),
            Resolution::NotFound
        );
    }

    #[test]
    fn a_traversal_is_refused_before_the_filesystem_is_touched() {
        let scratch = Scratch::new("traversal");
        scratch.write("index.html", b"<!doctype html>");

        for path in [
            "/../secret",
            "/assets/../../secret",
            "/%2e%2e/secret",
            "/%2E%2E%2Fsecret",
        ] {
            assert!(
                matches!(scratch.resources().resolve(path), Resolution::Refused(_)),
                "{path} should be refused"
            );
        }
    }

    #[test]
    fn a_windows_style_path_is_refused() {
        let scratch = Scratch::new("backslash");

        assert!(matches!(
            scratch.resources().resolve("/assets\\..\\secret"),
            Resolution::Refused(_)
        ));
    }

    #[test]
    fn a_hidden_entry_is_refused() {
        let scratch = Scratch::new("hidden");
        scratch.write(".env", b"SECRET=1");

        assert!(matches!(
            scratch.resources().resolve("/.env"),
            Resolution::Refused(_)
        ));
    }

    #[test]
    fn an_overlong_path_is_refused_unread() {
        let scratch = Scratch::new("overlong");
        let path = format!("/{}", "a".repeat(MAX_REQUEST_PATH_BYTES + 1));

        assert!(matches!(
            scratch.resources().resolve(&path),
            Resolution::Refused(_)
        ));
    }

    #[test]
    fn a_malformed_escape_is_refused() {
        let scratch = Scratch::new("escape");

        for path in ["/main%zz.js", "/main%", "/main%2"] {
            assert!(
                matches!(scratch.resources().resolve(path), Resolution::Refused(_)),
                "{path} should be refused"
            );
        }
    }

    #[test]
    fn a_reload_reads_the_file_again_rather_than_a_cached_copy() {
        // 501 section 12 requires a UI reload test. A reload re-requests every
        // asset, so the shell must not answer from a copy taken at startup.
        let scratch = Scratch::new("reload");
        scratch.write("index.html", b"first");
        let resources = scratch.resources();

        let before = resources.resolve("/index.html");
        scratch.write("index.html", b"second");
        let after = resources.resolve("/index.html");

        assert_eq!(
            before,
            Resolution::Found {
                bytes: b"first".to_vec(),
                content_type: "text/html; charset=utf-8",
            }
        );
        assert_eq!(
            after,
            Resolution::Found {
                bytes: b"second".to_vec(),
                content_type: "text/html; charset=utf-8",
            }
        );
    }

    #[test]
    fn an_unknown_extension_is_served_as_an_opaque_type() {
        let scratch = Scratch::new("opaque");
        scratch.write("data.bin", b"\x00\x01");

        assert_eq!(
            scratch.resources().resolve("/data.bin"),
            Resolution::Found {
                bytes: vec![0, 1],
                content_type: "application/octet-stream",
            }
        );
    }

    #[test]
    fn a_directory_is_not_served_as_a_file() {
        let scratch = Scratch::new("directory");
        scratch.write("assets/main.js", b"export {};");

        assert_eq!(scratch.resources().resolve("/assets"), Resolution::NotFound);
    }
}

//! Redaction for text that reaches users, logs, or telemetry.
//!
//! Canonical documentation: `docs/06-quality/605-error-model.md` sections 7 and 8,
//! and `CLAUDE.md` ("secrets never enter project files, logs, telemetry, bundles,
//! or ordinary config").
//!
//! Scope is deliberately narrow and stated rather than implied: this removes
//! absolute filesystem paths, which is the leak that happens by accident when a
//! platform error is formatted into a message. It is not a general secret scanner,
//! and it cannot make an unsafe message safe. Callers still must not put
//! credentials in a message.
//!
//! policy-allow: local-path - this module recognizes absolute paths, so its
//! markers and its test fixtures must contain them

/// Replacement for a redacted absolute path.
const PATH_PLACEHOLDER: &str = "<path>";

/// True at the start of an absolute path: a Windows drive root such as `C:\`.
fn windows_root_at(bytes: &[u8], index: usize) -> bool {
    let Some(drive) = bytes.get(index) else {
        return false;
    };

    drive.is_ascii_alphabetic()
        && bytes.get(index + 1) == Some(&b':')
        && matches!(bytes.get(index + 2), Some(b'\\') | Some(b'/'))
}

/// True at the start of a POSIX home or user path.
fn posix_root_at(text: &str, index: usize) -> bool {
    let rest = &text[index..];

    rest.starts_with("/home/") || rest.starts_with("/Users/") || rest.starts_with("/root/")
}

/// Whether a byte can continue a path segment.
fn is_path_byte(byte: u8) -> bool {
    !matches!(
        byte,
        b' ' | b'\t' | b'\n' | b'\r' | b'"' | b'\'' | b')' | b']' | b','
    )
}

/// Replace absolute filesystem paths with a placeholder.
///
/// A relative path is left alone: it names a file role rather than a private
/// location, which is what `605` section 7 asks for.
#[must_use]
pub fn redact_paths(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut index = 0;

    while index < bytes.len() {
        if !text.is_char_boundary(index) {
            // Inside a multi-byte character: copy the byte and move on.
            if let Some(slice) = text.get(index..=index) {
                out.push_str(slice);
            }
            index += 1;
            continue;
        }

        if windows_root_at(bytes, index) || posix_root_at(text, index) {
            let mut end = index;
            while end < bytes.len() && is_path_byte(bytes[end]) {
                end += 1;
            }

            // Trailing sentence punctuation is not part of the path.
            while end > index && matches!(bytes.get(end - 1), Some(b'.') | Some(b';') | Some(b':'))
            {
                end -= 1;
            }

            out.push_str(PATH_PLACEHOLDER);
            index = end;
            continue;
        }

        let character_end = text[index..]
            .chars()
            .next()
            .map_or(index + 1, |character| index + character.len_utf8());
        if let Some(slice) = text.get(index..character_end) {
            out.push_str(slice);
        }
        index = character_end;
    }

    out
}

/// Collapse whitespace and trim, so a message stays one readable line.
#[must_use]
pub fn normalize_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;

    for character in text.chars() {
        if character.is_whitespace() {
            pending_space = !out.is_empty();
            continue;
        }

        if pending_space {
            out.push(' ');
            pending_space = false;
        }

        out.push(character);
    }

    out
}

/// Truncate to at most `max_characters`, marking that text was removed.
///
/// Truncation counts characters, never bytes, so a multi-byte character cannot be
/// split into invalid UTF-8.
#[must_use]
pub fn truncate(text: &str, max_characters: usize) -> String {
    if text.chars().count() <= max_characters {
        return text.to_owned();
    }

    const ELLIPSIS: char = '…';

    let keep = max_characters.saturating_sub(1);
    let mut out: String = text.chars().take(keep).collect();
    out.push(ELLIPSIS);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_windows_user_paths() {
        let redacted = redact_paths("failed to open C:\\Users\\arthur\\project.mirae");

        assert_eq!(redacted, "failed to open <path>");
        assert!(!redacted.contains("arthur"));
    }

    #[test]
    fn redacts_posix_home_paths() {
        for text in [
            "failed to open /home/arthur/project.mirae",
            "failed to open /Users/arthur/project.mirae",
            "failed to open /root/project.mirae",
        ] {
            let redacted = redact_paths(text);

            assert_eq!(redacted, "failed to open <path>");
            assert!(!redacted.contains("arthur"));
        }
    }

    #[test]
    fn redacts_a_path_inside_a_sentence_and_keeps_punctuation() {
        let redacted = redact_paths("open C:\\Users\\a\\p.mirae failed, retrying");

        assert_eq!(redacted, "open <path> failed, retrying");
    }

    #[test]
    fn redacts_every_path_in_a_message() {
        let redacted = redact_paths("copy /home/a/one to /home/b/two");

        assert_eq!(redacted, "copy <path> to <path>");
    }

    #[test]
    fn keeps_relative_paths_and_file_roles() {
        // A file role without a private location is what 605 section 7 wants kept.
        for text in [
            "failed to open the project autosave file",
            "failed to open ./recovery/autosave.mirae",
            "failed to open recovery/autosave.mirae",
        ] {
            assert_eq!(redact_paths(text), text);
        }
    }

    #[test]
    fn leaves_text_without_paths_untouched() {
        let text = "the capture device was removed";

        assert_eq!(redact_paths(text), text);
    }

    #[test]
    fn survives_multi_byte_characters() {
        let text = "échec: ouverture de /home/arthur/projet.mirae — réessai";
        let redacted = redact_paths(text);

        assert!(redacted.starts_with("échec: ouverture de <path>"));
        assert!(redacted.ends_with("réessai"));
        assert!(!redacted.contains("arthur"));
    }

    #[test]
    fn normalizes_whitespace_to_one_line() {
        assert_eq!(
            normalize_whitespace("  failed\n  to   open\tthe file \n"),
            "failed to open the file"
        );
        assert_eq!(normalize_whitespace(""), "");
        assert_eq!(normalize_whitespace("   "), "");
    }

    #[test]
    fn truncates_on_character_boundaries() {
        assert_eq!(truncate("abcdef", 10), "abcdef");
        assert_eq!(truncate("abcdef", 6), "abcdef");
        assert_eq!(truncate("abcdef", 4), "abc…");
        // A multi-byte character must never be split.
        assert_eq!(truncate("ééééé", 3), "éé…");
    }

    #[test]
    fn truncation_is_never_longer_than_requested() {
        for max in 1..12_usize {
            let truncated = truncate("the capture device was removed", max);

            assert!(truncated.chars().count() <= max, "max {max} exceeded");
        }
    }
}

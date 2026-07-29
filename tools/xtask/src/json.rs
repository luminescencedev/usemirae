//! Minimal string-field reader for the few `package.json` fields xtask needs.
//!
//! A full JSON parser would be a new dependency for three lookups. This reader
//! is deliberately narrow: it finds string fields and one level of nesting, and
//! returns `None` for anything it does not understand rather than guessing.

/// Find the object body that follows `"key":`, including nested braces.
fn object_body<'input>(source: &'input str, key: &str) -> Option<&'input str> {
    let needle = format!("\"{key}\"");
    let after_key = source.find(&needle)? + needle.len();
    let start = source[after_key..].find('{')? + after_key + 1;

    let mut depth = 1_usize;
    for (offset, character) in source[start..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&source[start..start + offset]);
                }
            }
            _ => {}
        }
    }

    None
}

/// Read a top-level string field such as `packageManager`.
///
/// Only the first match is considered, so a field nested in an earlier object
/// cannot shadow the lookup for well-formed manifests.
pub(crate) fn string_field<'input>(source: &'input str, key: &str) -> Option<&'input str> {
    let needle = format!("\"{key}\"");
    let after_key = source.find(&needle)? + needle.len();
    let rest = source[after_key..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;

    Some(&rest[..end])
}

/// Read a string field from inside a nested object, such as `engines.node`.
pub(crate) fn nested_string_field<'input>(
    source: &'input str,
    object: &str,
    key: &str,
) -> Option<&'input str> {
    string_field(object_body(source, object)?, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"{
  "name": "mirae",
  "private": true,
  "packageManager": "pnpm@11.17.0",
  "engines": {
    "node": "24.18.1",
    "pnpm": "11.17.0"
  },
  "scripts": {
    "preinstall": "cargo xtask bootstrap"
  }
}"#;

    #[test]
    fn reads_a_top_level_string_field() {
        assert_eq!(
            string_field(MANIFEST, "packageManager"),
            Some("pnpm@11.17.0")
        );
        assert_eq!(string_field(MANIFEST, "name"), Some("mirae"));
    }

    #[test]
    fn reads_nested_string_fields() {
        assert_eq!(
            nested_string_field(MANIFEST, "engines", "node"),
            Some("24.18.1")
        );
        assert_eq!(
            nested_string_field(MANIFEST, "engines", "pnpm"),
            Some("11.17.0")
        );
    }

    #[test]
    fn does_not_confuse_a_later_object_for_the_requested_one() {
        assert_eq!(
            nested_string_field(MANIFEST, "scripts", "preinstall"),
            Some("cargo xtask bootstrap")
        );
    }

    #[test]
    fn returns_none_for_absent_fields() {
        assert_eq!(string_field(MANIFEST, "license"), None);
        assert_eq!(nested_string_field(MANIFEST, "engines", "bun"), None);
        assert_eq!(nested_string_field(MANIFEST, "absent", "node"), None);
    }

    #[test]
    fn returns_none_for_a_non_string_value() {
        assert_eq!(string_field(MANIFEST, "private"), None);
    }

    #[test]
    fn handles_nested_braces_inside_the_object() {
        let source = r#"{"engines":{"extra":{"deep":"x"},"node":"24.18.1"}}"#;
        assert_eq!(
            nested_string_field(source, "engines", "node"),
            Some("24.18.1")
        );
    }
}

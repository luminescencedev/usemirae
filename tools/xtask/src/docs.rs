//! Documentation structure validation for `cargo xtask docs`.
//!
//! Covers what `MIR-0003` needs to make the command real: every `docs/SUMMARY.md`
//! link resolves, and every ADR is indexed exactly once. Header validation,
//! duplicate document ids, and ADR cross-references belong to `MIR-0014`, which
//! extends this module.

use std::path::Path;

/// Extract every markdown link target from a document.
///
/// Returns targets in document order, including duplicates, so the caller can
/// detect a document indexed twice.
pub(crate) fn extract_links(markdown: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let bytes = markdown.as_bytes();
    let mut index = 0_usize;

    while index < bytes.len() {
        // A link target is the parenthesised part of `[text](target)`.
        let Some(open) = markdown[index..].find("](") else {
            break;
        };
        let start = index + open + 2;

        let Some(close) = markdown[start..].find(')') else {
            break;
        };
        let target = markdown[start..start + close].trim();

        if !target.is_empty() {
            targets.push(target.to_owned());
        }

        index = start + close + 1;
    }

    targets
}

/// True when a link points outside the repository or inside the same page.
pub(crate) fn is_external(target: &str) -> bool {
    target.starts_with("http://") || target.starts_with("https://") || target.starts_with('#')
}

/// Strip a `#fragment` so the file part can be resolved on disk.
pub(crate) fn file_part(target: &str) -> &str {
    target.split('#').next().unwrap_or(target)
}

/// Extract the four-digit id of an ADR file name, if it is one.
pub(crate) fn adr_id(file_name: &str) -> Option<&str> {
    let rest = file_name.strip_prefix("ADR-")?;
    let id = rest.get(..4)?;

    if id.bytes().all(|byte| byte.is_ascii_digit()) {
        Some(id)
    } else {
        None
    }
}

/// A documentation problem.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DocFinding {
    pub(crate) detail: String,
}

/// Validate the summary against the documents on disk.
///
/// `docs_dir` is the directory holding `SUMMARY.md`; link targets resolve
/// relative to it.
pub(crate) fn validate(summary: &str, docs_dir: &Path) -> Vec<DocFinding> {
    let mut findings = Vec::new();
    let links = extract_links(summary);

    for target in &links {
        if is_external(target) {
            continue;
        }

        if !docs_dir.join(file_part(target)).exists() {
            findings.push(DocFinding {
                detail: format!("SUMMARY.md links to `{target}`, which does not exist"),
            });
        }
    }

    let indexed: Vec<&str> = links
        .iter()
        .filter_map(|target| {
            Path::new(file_part(target))
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(adr_id)
        })
        .collect();

    let adr_dir = docs_dir.join("adr");
    let mut on_disk: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&adr_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if let Some(id) = name.to_str().and_then(adr_id) {
                on_disk.push(id.to_owned());
            }
        }
    }
    on_disk.sort_unstable();

    for id in &on_disk {
        let count = indexed.iter().filter(|candidate| *candidate == id).count();

        match count {
            0 => findings.push(DocFinding {
                detail: format!("ADR-{id} exists but is not indexed in SUMMARY.md"),
            }),
            1 => {}
            more => findings.push(DocFinding {
                detail: format!("ADR-{id} is indexed {more} times in SUMMARY.md"),
            }),
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_link_targets_in_order() {
        let markdown = "- [One](a/one.md)\n- [Two](b/two.md#section)\n";

        assert_eq!(
            extract_links(markdown),
            vec!["a/one.md", "b/two.md#section"]
        );
    }

    #[test]
    fn ignores_text_without_links() {
        assert_eq!(extract_links("no links here"), Vec::<String>::new());
    }

    #[test]
    fn survives_an_unterminated_link() {
        assert_eq!(extract_links("- [Broken](a/one.md"), Vec::<String>::new());
    }

    #[test]
    fn recognizes_external_and_anchor_targets() {
        assert!(is_external("https://example.com"));
        assert!(is_external("http://example.com"));
        assert!(is_external("#section"));
        assert!(!is_external("adr/ADR-0001-native-rust-core.md"));
    }

    #[test]
    fn strips_a_fragment_from_a_target() {
        assert_eq!(file_part("b/two.md#section"), "b/two.md");
        assert_eq!(file_part("b/two.md"), "b/two.md");
    }

    #[test]
    fn reads_adr_ids_only_from_adr_file_names() {
        assert_eq!(adr_id("ADR-0061-css-variables.md"), Some("0061"));
        assert_eq!(adr_id("ADR-abcd-not-an-id.md"), None);
        assert_eq!(adr_id("900-ui-ux-overview.md"), None);
        assert_eq!(adr_id("ADR-12.md"), None);
    }
}

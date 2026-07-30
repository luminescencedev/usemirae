//! Documentation structure validation for `cargo xtask docs`.
//!
//! Validates that every `docs/SUMMARY.md` link resolves, that every ADR is
//! indexed exactly once, that each document declares the header block its kind
//! requires, that no two numbered documents claim the same id, and that every
//! ADR referenced in prose exists (MIR-0003, extended by MIR-0014).

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

/// Headers every numbered specification must declare.
const SPECIFICATION_HEADERS: [&str; 3] = ["**Status:**", "**Audience:**", "**Canonical:**"];

/// Headers every ADR must declare.
const ADR_HEADERS: [&str; 2] = ["**Status:**", "**Date:**"];

/// Read the numeric id of a numbered document, such as `605` in `605-error-model.md`.
pub(crate) fn document_id(file_name: &str) -> Option<&str> {
    let digits = file_name.split('-').next()?;

    if digits.len() >= 3 && digits.bytes().all(|byte| byte.is_ascii_digit()) {
        Some(digits)
    } else {
        None
    }
}

/// Check the header block a document must declare.
///
/// A missing header is why `docs --check` exists: a specification without a
/// status or an audience cannot be reviewed against its own rules.
pub(crate) fn check_headers(relative_path: &str, contents: &str) -> Vec<DocFinding> {
    let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);

    let required: &[&str] = if file_name.starts_with("ADR-") {
        &ADR_HEADERS
    } else if document_id(file_name).is_some() {
        &SPECIFICATION_HEADERS
    } else {
        // README and index pages carry no header block.
        return Vec::new();
    };

    // Only the header block, so a header named in prose further down does not
    // count as a declaration.
    let header_block: String = contents.lines().take(12).collect::<Vec<_>>().join("\n");

    required
        .iter()
        .filter(|header| !header_block.contains(*header))
        .map(|header| DocFinding {
            detail: format!("{relative_path} is missing the `{header}` header"),
        })
        .collect()
}

/// Find two numbered documents claiming the same id.
///
/// `documents` is a list of paths relative to `docs/`.
pub(crate) fn find_duplicate_ids(documents: &[String]) -> Vec<DocFinding> {
    let mut findings = Vec::new();

    for (index, path) in documents.iter().enumerate() {
        let Some(name) = path.rsplit('/').next() else {
            continue;
        };
        let Some(id) = document_id(name) else {
            continue;
        };

        if let Some(earlier) = documents[..index].iter().find(|candidate| {
            candidate
                .rsplit('/')
                .next()
                .and_then(document_id)
                .is_some_and(|other| other == id)
        }) {
            findings.push(DocFinding {
                detail: format!("`{path}` reuses document id {id}, already used by `{earlier}`"),
            });
        }
    }

    findings
}

/// Extract every `ADR-NNNN` referenced in prose.
pub(crate) fn adr_references(contents: &str) -> Vec<String> {
    let mut references = Vec::new();
    let bytes = contents.as_bytes();
    let mut index = 0;

    while let Some(found) = contents.get(index..).and_then(|rest| rest.find("ADR-")) {
        let start = index + found + 4;

        if let Some(id) = contents.get(start..start + 4)
            && id.bytes().all(|byte| byte.is_ascii_digit())
            && !references.iter().any(|existing| existing == id)
        {
            references.push(id.to_owned());
        }

        index = start.min(bytes.len());
    }

    references
}

/// Report a reference to an ADR that does not exist.
pub(crate) fn check_adr_references(
    relative_path: &str,
    contents: &str,
    known_adrs: &[String],
) -> Vec<DocFinding> {
    adr_references(contents)
        .into_iter()
        .filter(|id| !known_adrs.contains(id))
        .map(|id| DocFinding {
            detail: format!("{relative_path} references ADR-{id}, which does not exist"),
        })
        .collect()
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

    // Every markdown document, so headers, ids, and references are checked
    // whether or not the summary happens to link them.
    let documents = collect_documents(docs_dir);
    findings.extend(find_duplicate_ids(&documents));

    for relative in &documents {
        let Ok(contents) = std::fs::read_to_string(docs_dir.join(relative)) else {
            continue;
        };

        findings.extend(check_headers(relative, &contents));
        findings.extend(check_adr_references(relative, &contents, &on_disk));
    }

    findings
}

/// Every markdown file under `docs/`, relative and sorted.
pub(crate) fn collect_documents(docs_dir: &Path) -> Vec<String> {
    let mut documents = Vec::new();
    let mut pending = vec![docs_dir.to_path_buf()];

    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                pending.push(path);
                continue;
            }

            if path.extension().is_some_and(|extension| extension == "md") {
                let relative = path
                    .strip_prefix(docs_dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                documents.push(relative);
            }
        }
    }

    documents.sort();
    documents
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
    fn reads_document_ids_from_numbered_names() {
        assert_eq!(document_id("605-error-model.md"), Some("605"));
        assert_eq!(document_id("1001-future.md"), Some("1001"));
        assert_eq!(document_id("SUMMARY.md"), None);
        assert_eq!(document_id("ADR-0001-native-rust-core.md"), None);
    }

    #[test]
    fn a_specification_must_declare_status_audience_and_canonical() {
        let complete = "# 605 — Error Model\n\n**Status:** Proposed  \n\
                        **Audience:** All  \n**Canonical:** Yes  \n";

        assert_eq!(
            check_headers("06-quality/605-error-model.md", complete),
            Vec::new()
        );

        let findings = check_headers("06-quality/605-error-model.md", "# 605 — Error Model\n");

        assert_eq!(findings.len(), 3);
    }

    #[test]
    fn an_adr_must_declare_status_and_date() {
        let complete = "# ADR-0001 — X\n\n**Status:** Accepted  \n**Date:** 2026-07-30\n";

        assert_eq!(check_headers("adr/ADR-0001-x.md", complete), Vec::new());
        assert_eq!(
            check_headers("adr/ADR-0001-x.md", "# ADR-0001 — X\n").len(),
            2
        );
    }

    #[test]
    fn an_index_page_needs_no_header_block() {
        assert_eq!(check_headers("README.md", "# Docs\n"), Vec::new());
        assert_eq!(check_headers("SUMMARY.md", "# Summary\n"), Vec::new());
    }

    #[test]
    fn a_header_named_further_down_does_not_count() {
        // Only the header block counts, so prose about a status cannot satisfy it.
        let body =
            "# 605 — Error Model\n".to_owned() + &"\nfiller\n".repeat(20) + "**Status:** x\n";

        assert!(!check_headers("06-quality/605-error-model.md", &body).is_empty());
    }

    #[test]
    fn two_documents_may_not_claim_the_same_id() {
        let documents = vec![
            "06-quality/605-error-model.md".to_owned(),
            "01-runtime/605-something-else.md".to_owned(),
            "01-runtime/108-ipc-protocol.md".to_owned(),
        ];
        let findings = find_duplicate_ids(&documents);

        assert_eq!(findings.len(), 1);
        assert!(findings[0].detail.contains("605"));
    }

    #[test]
    fn extracts_adr_references_from_prose() {
        let contents = "Related ADRs: ADR-0006, ADR-0057. See also ADR-0006 again.";

        assert_eq!(adr_references(contents), vec!["0006", "0057"]);
        assert_eq!(adr_references("no references here"), Vec::<String>::new());
    }

    #[test]
    fn a_reference_to_a_missing_adr_is_reported() {
        let known = vec!["0006".to_owned()];

        assert_eq!(
            check_adr_references("a.md", "See ADR-0006.", &known),
            Vec::new()
        );
        assert_eq!(
            check_adr_references("a.md", "See ADR-9999.", &known).len(),
            1
        );
    }

    #[test]
    fn reads_adr_ids_only_from_adr_file_names() {
        assert_eq!(adr_id("ADR-0061-css-variables.md"), Some("0061"));
        assert_eq!(adr_id("ADR-abcd-not-an-id.md"), None);
        assert_eq!(adr_id("900-ui-ux-overview.md"), None);
        assert_eq!(adr_id("ADR-12.md"), None);
    }
}

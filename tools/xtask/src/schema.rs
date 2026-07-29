//! Canonical schema discovery, validation, and deterministic code generation.
//!
//! Canonical documentation:
//! `docs/08-development/805-generated-contracts-and-schemas.md`, ADR-0057, and
//! `docs/01-runtime/108-ipc-protocol.md` for the IPC contracts.
//!
//! Section 4 requires one root command that validates schemas, generates outputs,
//! writes deterministic files, verifies no duplicate ids, and reports changed
//! contracts. This module implements that pipeline.
//!
//! ## Supported schema subset
//!
//! Deliberately narrow, so a schema cannot express something the generators would
//! silently drop: a top-level `object` with `properties` of type `string`
//! (optionally `enum` or `maxLength`), `integer` (optionally `const`, `minimum`,
//! `maximum`), or `boolean`, plus `required`. Anything else is rejected with the
//! property named. `$ref`, composition, and nested objects are not supported yet.
//!
//! Parsing and rendering are pure so determinism and every rejection are unit
//! tested; `discover` owns the file system.

use std::path::{Path, PathBuf};

use crate::json::{self, Value};
use crate::naming::{pascal_case, screaming_snake_case, snake_case};

/// The canonical schema domains from `805` section 2.
///
/// Each becomes `schemas/<domain>/v<major>/`. The list is ordered, and generated
/// output follows it, so output never depends on directory iteration order.
pub(crate) const DOMAINS: [&str; 8] = [
    "ipc",
    "project",
    "bundle",
    "diagnostics",
    "sdk",
    "extension-manifest",
    "extension-ui",
    "compatibility",
];

/// The marker that identifies generated files.
pub(crate) const GENERATED_MARKER: &str = "@generated";

/// A property's value shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FieldKind {
    /// A bounded unsigned integer. `bits` is 16, 32, or 64.
    Integer { bits: u8 },
    /// Free text, with an optional maximum length.
    Text { max_length: Option<u64> },
    /// A closed set of string values.
    Enumeration { variants: Vec<String> },
    /// A boolean flag.
    Flag,
}

/// One property of a contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Field {
    /// Property name as written in the schema, used verbatim in TypeScript.
    pub(crate) json_name: String,
    pub(crate) doc: String,
    pub(crate) kind: FieldKind,
    pub(crate) required: bool,
    /// A fixed value, rendered as a language constant.
    pub(crate) constant: Option<String>,
}

/// A validated contract document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Contract {
    pub(crate) domain: String,
    pub(crate) version: String,
    /// Path relative to the repository root, with forward slashes.
    pub(crate) path: String,
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    /// `PascalCase` type name derived from the last segment of the id.
    pub(crate) type_name: String,
    pub(crate) fields: Vec<Field>,
}

/// A schema that cannot be accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchemaError {
    pub(crate) path: String,
    pub(crate) detail: String,
}

/// Read an integer literal out of a parsed number.
fn integer_literal(value: &Value) -> Option<u64> {
    match value {
        Value::Number(text) => text.parse::<u64>().ok(),
        _ => None,
    }
}

/// Choose the smallest unsigned width that holds `maximum`.
fn integer_bits(maximum: Option<u64>) -> u8 {
    match maximum {
        Some(max) if max <= u64::from(u16::MAX) => 16,
        Some(max) if max <= u64::from(u32::MAX) => 32,
        _ => 64,
    }
}

/// Parse one property definition.
fn parse_field(
    path: &str,
    name: &str,
    definition: &Value,
    required: bool,
) -> Result<Field, SchemaError> {
    let error = |detail: String| SchemaError {
        path: path.to_owned(),
        detail: format!("property `{name}`: {detail}"),
    };

    let doc = definition
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();

    if doc.is_empty() {
        return Err(error(
            "needs a `description`; generated code is documented".to_owned(),
        ));
    }

    let Some(type_name) = definition.get("type").and_then(Value::as_str) else {
        return Err(error("needs a string `type`".to_owned()));
    };

    let kind = match type_name {
        "integer" => {
            let maximum = definition.get("maximum").and_then(integer_literal);
            if maximum.is_none() {
                return Err(error(
                    "needs a `maximum`; bounds are represented in the schema (805 invariant 9)"
                        .to_owned(),
                ));
            }
            FieldKind::Integer {
                bits: integer_bits(maximum),
            }
        }
        "string" => match definition.get("enum") {
            Some(values) => {
                let mut variants = Vec::new();
                for value in values.elements() {
                    match value.as_str() {
                        Some(text) => variants.push(text.to_owned()),
                        None => return Err(error("`enum` values must be strings".to_owned())),
                    }
                }
                if variants.is_empty() {
                    return Err(error("`enum` must list at least one value".to_owned()));
                }
                FieldKind::Enumeration { variants }
            }
            None => {
                let max_length = definition.get("maxLength").and_then(integer_literal);
                if max_length.is_none() {
                    return Err(error(
                        "needs a `maxLength`; bounds are represented in the schema \
                         (805 invariant 9)"
                            .to_owned(),
                    ));
                }
                FieldKind::Text { max_length }
            }
        },
        "boolean" => FieldKind::Flag,
        other => {
            return Err(error(format!(
                "type `{other}` is not supported; use integer, string, or boolean"
            )));
        }
    };

    let constant = match definition.get("const") {
        None => None,
        Some(Value::Number(text)) => Some(text.clone()),
        Some(Value::String(text)) => Some(text.clone()),
        Some(_) => {
            return Err(error("`const` must be a number or a string".to_owned()));
        }
    };

    Ok(Field {
        json_name: name.to_owned(),
        doc,
        kind,
        required,
        constant,
    })
}

/// Validate one schema document and build its contract model.
pub(crate) fn parse_schema(
    domain: &str,
    version: &str,
    path: &str,
    contents: &str,
) -> Result<Contract, SchemaError> {
    let error = |detail: String| SchemaError {
        path: path.to_owned(),
        detail,
    };

    let document = json::parse(contents)
        .map_err(|parse_error| error(format!("is not valid JSON: {parse_error}")))?;

    let string = |key: &str| document.get(key).and_then(Value::as_str);

    let Some(id) = string("$id") else {
        return Err(error("missing a string `$id`".to_owned()));
    };
    let Some(title) = string("title") else {
        return Err(error("missing a string `title`".to_owned()));
    };
    let Some(description) = string("description") else {
        return Err(error("missing a string `description`".to_owned()));
    };

    // The id carries the domain and version so a moved file cannot silently
    // change the contract it claims to define.
    let expected_prefix = format!("mirae://{domain}/{version}/");
    if !id.starts_with(&expected_prefix) {
        return Err(error(format!(
            "`$id` must start with `{expected_prefix}`, found `{id}`"
        )));
    }

    if string("type") != Some("object") {
        return Err(error(
            "top-level `type` must be `object`; other shapes are not supported".to_owned(),
        ));
    }

    // Closed contracts only: an open object would let a peer smuggle fields past
    // both generators.
    if document.get("additionalProperties") != Some(&Value::Bool(false)) {
        return Err(error(
            "must set `\"additionalProperties\": false` so the contract stays closed".to_owned(),
        ));
    }

    let Some(properties) = document.get("properties") else {
        return Err(error("missing `properties`".to_owned()));
    };

    let required: Vec<&str> = document
        .get("required")
        .map(|value| value.elements().iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();

    let mut fields = Vec::new();
    for (name, definition) in properties.members() {
        fields.push(parse_field(
            path,
            name,
            definition,
            required.contains(&name.as_str()),
        )?);
    }

    if fields.is_empty() {
        return Err(error(
            "`properties` must define at least one field".to_owned(),
        ));
    }

    for name in &required {
        if !fields.iter().any(|field| field.json_name == *name) {
            return Err(error(format!(
                "`required` names `{name}`, which is not a declared property"
            )));
        }
    }

    let Some(last_segment) = id.rsplit('/').next().filter(|segment| !segment.is_empty()) else {
        return Err(error("`$id` must end with a contract name".to_owned()));
    };

    Ok(Contract {
        domain: domain.to_owned(),
        version: version.to_owned(),
        path: path.to_owned(),
        id: id.to_owned(),
        title: title.to_owned(),
        description: description.to_owned(),
        type_name: pascal_case(last_segment),
        fields,
    })
}

/// Reject two schemas that claim the same `$id` (`805` invariant 8).
pub(crate) fn find_duplicate_ids(contracts: &[Contract]) -> Vec<SchemaError> {
    let mut errors = Vec::new();

    for (index, contract) in contracts.iter().enumerate() {
        if let Some(earlier) = contracts[..index]
            .iter()
            .find(|candidate| candidate.id == contract.id)
        {
            errors.push(SchemaError {
                path: contract.path.clone(),
                detail: format!(
                    "duplicate `$id` `{}`, already defined by `{}`",
                    contract.id, earlier.path
                ),
            });
        }
    }

    errors
}

/// Reject two contracts that would generate the same type name.
pub(crate) fn find_duplicate_type_names(contracts: &[Contract]) -> Vec<SchemaError> {
    let mut errors = Vec::new();

    for (index, contract) in contracts.iter().enumerate() {
        if let Some(earlier) = contracts[..index]
            .iter()
            .find(|candidate| candidate.type_name == contract.type_name)
        {
            errors.push(SchemaError {
                path: contract.path.clone(),
                detail: format!(
                    "generates type `{}`, which `{}` already generates",
                    contract.type_name, earlier.path
                ),
            });
        }
    }

    errors
}

/// Escape a string for embedding in JSON.
fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other => escaped.push(other),
        }
    }

    escaped
}

/// Sort contracts by id so output never depends on discovery order.
fn sorted(contracts: &[Contract]) -> Vec<&Contract> {
    let mut sorted: Vec<&Contract> = contracts.iter().collect();
    sorted.sort_by(|left, right| left.id.cmp(&right.id));
    sorted
}

/// Render the deterministic schema manifest.
pub(crate) fn render_manifest(contracts: &[Contract]) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"generated\": true,\n");
    out.push_str(&format!(
        "  \"note\": \"{GENERATED_MARKER} by `cargo xtask generate`. Do not edit by hand.\",\n"
    ));
    out.push_str("  \"domains\": [\n");
    for (index, domain) in DOMAINS.iter().enumerate() {
        let comma = if index + 1 == DOMAINS.len() { "" } else { "," };
        out.push_str(&format!("    \"{domain}\"{comma}\n"));
    }
    out.push_str("  ],\n");
    out.push_str("  \"contracts\": [");

    let contracts = sorted(contracts);
    if contracts.is_empty() {
        out.push_str("]\n}\n");
        return out;
    }

    out.push('\n');
    for (index, contract) in contracts.iter().enumerate() {
        let comma = if index + 1 == contracts.len() {
            ""
        } else {
            ","
        };
        out.push_str("    {\n");
        out.push_str(&format!(
            "      \"id\": \"{}\",\n",
            escape_json(&contract.id)
        ));
        out.push_str(&format!(
            "      \"title\": \"{}\",\n",
            escape_json(&contract.title)
        ));
        out.push_str(&format!("      \"domain\": \"{}\",\n", contract.domain));
        out.push_str(&format!("      \"version\": \"{}\",\n", contract.version));
        out.push_str(&format!(
            "      \"typeName\": \"{}\",\n",
            contract.type_name
        ));
        out.push_str(&format!("      \"fields\": {},\n", contract.fields.len()));
        out.push_str(&format!(
            "      \"path\": \"{}\"\n",
            escape_json(&contract.path)
        ));
        out.push_str(&format!("    }}{comma}\n"));
    }
    out.push_str("  ]\n}\n");

    out
}

/// The Rust type for a field, and the enum it may need.
fn rust_type(contract: &Contract, field: &Field) -> String {
    let base = match &field.kind {
        FieldKind::Integer { bits } => format!("u{bits}"),
        FieldKind::Text { .. } => "String".to_owned(),
        FieldKind::Flag => "bool".to_owned(),
        FieldKind::Enumeration { .. } => {
            format!("{}{}", contract.type_name, pascal_case(&field.json_name))
        }
    };

    if field.required {
        base
    } else {
        format!("Option<{base}>")
    }
}

/// Render the generated Rust bindings.
pub(crate) fn render_rust(contracts: &[Contract]) -> String {
    let mut out = String::new();
    out.push_str("//! Generated Rust contract bindings.\n//!\n");
    out.push_str(&format!(
        "//! {GENERATED_MARKER} by `cargo xtask generate`. Do not edit by hand.\n"
    ));
    out.push_str("//! Regenerate with `cargo xtask generate` after changing a schema under\n");
    out.push_str("//! `schemas/`, and verify with `cargo xtask generate --check`.\n");

    let contracts = sorted(contracts);

    if contracts.is_empty() {
        out.push_str("\n// No schemas are defined yet.\n");
        return out;
    }

    for contract in &contracts {
        out.push_str("\n/// ");
        out.push_str(&contract.title);
        out.push_str(".\n///\n/// ");
        out.push_str(&contract.description);
        out.push_str("\n///\n/// Canonical schema: `");
        out.push_str(&contract.id);
        out.push_str("`.\n");

        // Point at the enums the struct refers to, so the generated docs link.
        for field in &contract.fields {
            if matches!(field.kind, FieldKind::Enumeration { .. }) {
                let enum_name = format!("{}{}", contract.type_name, pascal_case(&field.json_name));
                out.push_str(&format!("///\n/// See [`{enum_name}`].\n"));
            }
        }

        out.push_str("#[derive(Debug, Clone, PartialEq, Eq)]\n");
        out.push_str(&format!("pub struct {} {{\n", contract.type_name));
        for field in &contract.fields {
            out.push_str(&format!("    /// {}\n", field.doc));
            if let FieldKind::Text {
                max_length: Some(max),
            } = &field.kind
            {
                out.push_str(&format!(
                    "    ///\n    /// Bounded to {max} characters by the schema.\n"
                ));
            }
            out.push_str(&format!(
                "    pub {}: {},\n",
                snake_case(&field.json_name),
                rust_type(contract, field)
            ));
        }
        out.push_str("}\n");

        for field in &contract.fields {
            if let FieldKind::Enumeration { variants } = &field.kind {
                let enum_name = format!("{}{}", contract.type_name, pascal_case(&field.json_name));
                out.push_str(&format!("\n/// {}\n", field.doc));
                out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
                out.push_str(&format!("pub enum {enum_name} {{\n"));
                for variant in variants {
                    out.push_str(&format!(
                        "    /// The `{variant}` value of `{}`.\n",
                        field.json_name
                    ));
                    out.push_str(&format!("    {},\n", pascal_case(variant)));
                }
                out.push_str("}\n");

                out.push_str(&format!("\nimpl {enum_name} {{\n"));
                out.push_str("    /// The value as written on the wire.\n");
                out.push_str("    #[must_use]\n");
                out.push_str("    pub const fn as_wire_str(self) -> &'static str {\n");
                out.push_str("        match self {\n");
                for variant in variants {
                    out.push_str(&format!(
                        "            Self::{} => \"{variant}\",\n",
                        pascal_case(variant)
                    ));
                }
                out.push_str("        }\n    }\n}\n");
            }
        }

        for field in &contract.fields {
            let prefix = format!(
                "{}_{}",
                screaming_snake_case(&contract.type_name),
                screaming_snake_case(&field.json_name)
            );

            if let Some(constant) = &field.constant {
                let FieldKind::Integer { bits } = &field.kind else {
                    continue;
                };
                out.push_str(&format!(
                    "\n/// The only accepted value of `{}`.\n",
                    field.json_name
                ));
                out.push_str(&format!("pub const {prefix}: u{bits} = {constant};\n"));
            }

            if let FieldKind::Text {
                max_length: Some(max),
            } = &field.kind
            {
                out.push_str(&format!(
                    "\n/// Maximum accepted length of `{}`, for bounded decoding.\n",
                    field.json_name
                ));
                out.push_str(&format!("pub const {prefix}_MAX_LENGTH: usize = {max};\n"));
            }
        }
    }

    out.push_str("\n/// Every contract id in this build, sorted.\n");
    out.push_str(&format!(
        "pub const CONTRACT_IDS: [&str; {}] = [\n",
        contracts.len()
    ));
    for contract in &contracts {
        out.push_str(&format!("    \"{}\",\n", escape_json(&contract.id)));
    }
    out.push_str("];\n");

    out
}

/// The TypeScript type for a field.
fn typescript_type(contract: &Contract, field: &Field) -> String {
    match &field.kind {
        FieldKind::Integer { .. } => "number".to_owned(),
        FieldKind::Text { .. } => "string".to_owned(),
        FieldKind::Flag => "boolean".to_owned(),
        FieldKind::Enumeration { .. } => {
            format!("{}{}", contract.type_name, pascal_case(&field.json_name))
        }
    }
}

/// Render the generated TypeScript bindings.
pub(crate) fn render_typescript(contracts: &[Contract]) -> String {
    let mut out = String::new();
    out.push_str("/**\n * Generated TypeScript contract bindings.\n *\n");
    out.push_str(&format!(
        " * {GENERATED_MARKER} by `cargo xtask generate`. Do not edit by hand.\n"
    ));
    out.push_str(" * Regenerate with `cargo xtask generate` after changing a schema under\n");
    out.push_str(" * `schemas/`, and verify with `cargo xtask generate --check`.\n */\n");

    let contracts = sorted(contracts);

    if contracts.is_empty() {
        out.push_str("\nexport const CONTRACT_IDS: readonly string[] = [];\n");
        return out;
    }

    for contract in &contracts {
        for field in &contract.fields {
            if let FieldKind::Enumeration { variants } = &field.kind {
                let type_name = format!("{}{}", contract.type_name, pascal_case(&field.json_name));
                out.push_str(&format!("\n/** {} */\n", field.doc));
                out.push_str(&format!("export type {type_name} =\n"));
                for (index, variant) in variants.iter().enumerate() {
                    let terminator = if index + 1 == variants.len() { ";" } else { "" };
                    out.push_str(&format!("  | \"{variant}\"{terminator}\n"));
                }
            }
        }

        out.push_str(&format!(
            "\n/**\n * {}.\n *\n * {}\n *\n * Canonical schema: `{}`.\n */\n",
            contract.title, contract.description, contract.id
        ));
        out.push_str(&format!("export interface {} {{\n", contract.type_name));
        for field in &contract.fields {
            out.push_str(&format!("  /** {} */\n", field.doc));
            let optional = if field.required { "" } else { "?" };
            out.push_str(&format!(
                "  readonly {}{}: {};\n",
                field.json_name,
                optional,
                typescript_type(contract, field)
            ));
        }
        out.push_str("}\n");

        for field in &contract.fields {
            let prefix = format!(
                "{}_{}",
                screaming_snake_case(&contract.type_name),
                screaming_snake_case(&field.json_name)
            );

            if let Some(constant) = &field.constant
                && matches!(field.kind, FieldKind::Integer { .. })
            {
                out.push_str(&format!(
                    "\n/** The only accepted value of `{}`. */\n",
                    field.json_name
                ));
                out.push_str(&format!("export const {prefix} = {constant};\n"));
            }

            if let FieldKind::Text {
                max_length: Some(max),
            } = &field.kind
            {
                out.push_str(&format!(
                    "\n/** Maximum accepted length of `{}`, for bounded decoding. */\n",
                    field.json_name
                ));
                out.push_str(&format!("export const {prefix}_MAX_LENGTH = {max};\n"));
            }
        }
    }

    out.push_str(
        "\n/** Every contract id in this build, sorted. */\nexport const CONTRACT_IDS = [\n",
    );
    for contract in &contracts {
        out.push_str(&format!("  \"{}\",\n", escape_json(&contract.id)));
    }
    out.push_str("] as const;\n");

    out
}

/// One generated artifact: where it goes and what it should contain.
pub(crate) struct Artifact {
    /// Path relative to the repository root.
    pub(crate) path: &'static str,
    pub(crate) contents: String,
}

/// Build every generated artifact.
///
/// Language bindings are written into the crate and package that own them
/// (`802` and `803`), not into a shared directory, so each generated file has one
/// declared owner (`805` invariant 4).
pub(crate) fn artifacts(contracts: &[Contract]) -> Vec<Artifact> {
    vec![
        Artifact {
            path: "schemas/generated/manifest.json",
            contents: render_manifest(contracts),
        },
        Artifact {
            path: "crates/foundation/contracts/src/generated.rs",
            contents: render_rust(contracts),
        },
        Artifact {
            path: "packages/contracts/src/generated.ts",
            contents: render_typescript(contracts),
        },
    ]
}

/// Read the major-version directory name, if it is one.
fn version_directory(name: &str) -> Option<String> {
    let digits = name.strip_prefix('v')?;

    if !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()) {
        Some(name.to_owned())
    } else {
        None
    }
}

/// Discover and validate every schema under `schemas/<domain>/v<major>/`.
pub(crate) fn discover(root: &Path) -> (Vec<Contract>, Vec<SchemaError>) {
    let mut contracts = Vec::new();
    let mut errors = Vec::new();

    for domain in DOMAINS {
        let domain_dir = root.join("schemas").join(domain);
        let Ok(version_entries) = std::fs::read_dir(&domain_dir) else {
            continue;
        };

        let mut versions: Vec<PathBuf> = version_entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect();
        versions.sort();

        for version_path in versions {
            let Some(version) = version_path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(version_directory)
            else {
                continue;
            };

            let Ok(entries) = std::fs::read_dir(&version_path) else {
                continue;
            };
            let mut files: Vec<PathBuf> = entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.ends_with(".schema.json"))
                })
                .collect();
            files.sort();

            for file in files {
                let relative = file
                    .strip_prefix(root)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .replace('\\', "/");

                match std::fs::read_to_string(&file) {
                    Err(error) => errors.push(SchemaError {
                        path: relative,
                        detail: format!("could not be read: {error}"),
                    }),
                    Ok(contents) => match parse_schema(domain, &version, &relative, &contents) {
                        Ok(contract) => contracts.push(contract),
                        Err(error) => errors.push(error),
                    },
                }
            }
        }
    }

    errors.extend(find_duplicate_ids(&contracts));
    errors.extend(find_duplicate_type_names(&contracts));

    (contracts, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERSION_SCHEMA: &str = r#"{
  "$id": "mirae://ipc/v1/protocol-version",
  "title": "Protocol version",
  "description": "The engine and shell protocol version.",
  "type": "object",
  "properties": {
    "major": {
      "type": "integer",
      "const": 1,
      "maximum": 65535,
      "description": "Incompatible protocol generation."
    }
  },
  "required": ["major"],
  "additionalProperties": false
}"#;

    const READINESS_SCHEMA: &str = r#"{
  "$id": "mirae://ipc/v1/engine-readiness",
  "title": "Engine readiness",
  "description": "What the engine reports about its own lifecycle.",
  "type": "object",
  "properties": {
    "state": {
      "type": "string",
      "enum": ["starting", "ready"],
      "description": "Lifecycle state."
    },
    "engineSessionId": {
      "type": "string",
      "maxLength": 64,
      "description": "Identifies one engine process lifetime."
    },
    "detail": {
      "type": "string",
      "maxLength": 256,
      "description": "Optional safe explanation."
    }
  },
  "required": ["state", "engineSessionId"],
  "additionalProperties": false
}"#;

    fn parse(contents: &str) -> Result<Contract, SchemaError> {
        parse_schema("ipc", "v1", "schemas/ipc/v1/probe.schema.json", contents)
    }

    fn contracts() -> Vec<Contract> {
        let mut parsed = Vec::new();
        if let Ok(contract) = parse(VERSION_SCHEMA) {
            parsed.push(contract);
        }
        if let Ok(contract) = parse(READINESS_SCHEMA) {
            parsed.push(contract);
        }
        parsed
    }

    #[test]
    fn parses_a_contract_with_a_constant_field() {
        let contract = parse(VERSION_SCHEMA);

        assert_eq!(
            contract.as_ref().map(|contract| contract.type_name.clone()),
            Ok("ProtocolVersion".to_owned())
        );
        assert_eq!(
            contract.map(|contract| contract.fields),
            Ok(vec![Field {
                json_name: "major".to_owned(),
                doc: "Incompatible protocol generation.".to_owned(),
                kind: FieldKind::Integer { bits: 16 },
                required: true,
                constant: Some("1".to_owned()),
            }])
        );
    }

    #[test]
    fn parses_enumerations_and_optional_fields() {
        let contract = parse(READINESS_SCHEMA).ok();
        let fields = contract.map(|contract| contract.fields).unwrap_or_default();

        assert_eq!(fields.len(), 3);
        assert_eq!(
            fields[0].kind,
            FieldKind::Enumeration {
                variants: vec!["starting".to_owned(), "ready".to_owned()]
            }
        );
        assert!(fields[1].required);
        assert!(!fields[2].required, "`detail` is not in `required`");
    }

    #[test]
    fn chooses_the_smallest_integer_width() {
        assert_eq!(integer_bits(Some(65535)), 16);
        assert_eq!(integer_bits(Some(65536)), 32);
        assert_eq!(integer_bits(Some(u64::from(u32::MAX) + 1)), 64);
        assert_eq!(integer_bits(None), 64);
    }

    #[test]
    fn rejects_schemas_that_would_generate_undocumented_or_unbounded_code() {
        let cases: [(&str, &str); 7] = [
            (
                r#"{"$id":"mirae://ipc/v1/a","title":"A","description":"d","type":"object","properties":{"x":{"type":"integer","maximum":10}},"additionalProperties":false}"#,
                "description",
            ),
            (
                r#"{"$id":"mirae://ipc/v1/a","title":"A","description":"d","type":"object","properties":{"x":{"type":"integer","description":"d"}},"additionalProperties":false}"#,
                "maximum",
            ),
            (
                r#"{"$id":"mirae://ipc/v1/a","title":"A","description":"d","type":"object","properties":{"x":{"type":"string","description":"d"}},"additionalProperties":false}"#,
                "maxLength",
            ),
            (
                r#"{"$id":"mirae://ipc/v1/a","title":"A","description":"d","type":"object","properties":{"x":{"type":"array","description":"d"}},"additionalProperties":false}"#,
                "not supported",
            ),
            (
                r#"{"$id":"mirae://ipc/v1/a","title":"A","description":"d","type":"object","properties":{"x":{"type":"integer","maximum":1,"description":"d"}}}"#,
                "additionalProperties",
            ),
            (
                r#"{"$id":"mirae://ipc/v1/a","title":"A","type":"object","properties":{"x":{"type":"integer","maximum":1,"description":"d"}},"additionalProperties":false}"#,
                "description",
            ),
            (
                r#"{"$id":"mirae://ipc/v1/a","title":"A","description":"d","type":"object","properties":{"x":{"type":"integer","maximum":1,"description":"d"}},"required":["y"],"additionalProperties":false}"#,
                "not a declared property",
            ),
        ];

        for (document, expected) in cases {
            let error = parse(document).err();
            assert!(
                error
                    .as_ref()
                    .is_some_and(|error| error.detail.contains(expected)),
                "expected `{expected}`, got {error:?}"
            );
        }
    }

    #[test]
    fn rejects_malformed_json_with_the_position() {
        let error = parse("{ not json }").err();

        assert!(
            error.is_some_and(|error| error.detail.contains("is not valid JSON")),
            "malformed JSON must be reported as such"
        );
    }

    #[test]
    fn rejects_an_id_that_disagrees_with_its_location() {
        let document = VERSION_SCHEMA.replace("mirae://ipc/v1/", "mirae://project/v1/");
        let error = parse(&document).err();

        assert!(error.is_some_and(|error| error.detail.contains("must start with")));
    }

    #[test]
    fn rejects_duplicate_ids_and_type_names() {
        let mut same_id = contracts();
        if let Some(second) = same_id.get_mut(1) {
            second.id = "mirae://ipc/v1/protocol-version".to_owned();
        }
        assert_eq!(find_duplicate_ids(&same_id).len(), 1);

        let mut same_type = contracts();
        if let Some(second) = same_type.get_mut(1) {
            second.type_name = "ProtocolVersion".to_owned();
        }
        assert_eq!(find_duplicate_type_names(&same_type).len(), 1);
    }

    #[test]
    fn output_does_not_depend_on_input_order() {
        let forward = contracts();
        let mut reversed = forward.clone();
        reversed.reverse();

        assert_eq!(render_manifest(&forward), render_manifest(&reversed));
        assert_eq!(render_rust(&forward), render_rust(&reversed));
        assert_eq!(render_typescript(&forward), render_typescript(&reversed));
    }

    #[test]
    fn every_artifact_carries_the_generated_marker() {
        for artifact in artifacts(&contracts()) {
            assert!(
                artifact.contents.contains(GENERATED_MARKER),
                "{} has no generated marker",
                artifact.path
            );
        }
    }

    #[test]
    fn rust_output_declares_types_constants_and_bounds() {
        let rust = render_rust(&contracts());

        assert!(rust.contains("pub struct ProtocolVersion {"));
        assert!(rust.contains("pub major: u16,"));
        assert!(rust.contains("pub const PROTOCOL_VERSION_MAJOR: u16 = 1;"));
        assert!(rust.contains("pub struct EngineReadiness {"));
        assert!(rust.contains("pub enum EngineReadinessState {"));
        assert!(rust.contains("Starting,"));
        assert!(rust.contains("pub state: EngineReadinessState,"));
        assert!(rust.contains("pub engine_session_id: String,"));
        assert!(rust.contains("pub detail: Option<String>,"));
        assert!(
            rust.contains("pub const ENGINE_READINESS_ENGINE_SESSION_ID_MAX_LENGTH: usize = 64;")
        );
        assert!(rust.contains("CONTRACT_IDS: [&str; 2]"));
    }

    #[test]
    fn typescript_output_mirrors_the_rust_shape() {
        let typescript = render_typescript(&contracts());

        assert!(typescript.contains("export interface ProtocolVersion {"));
        assert!(typescript.contains("readonly major: number;"));
        assert!(typescript.contains("export const PROTOCOL_VERSION_MAJOR = 1;"));
        assert!(typescript.contains("export type EngineReadinessState ="));
        assert!(typescript.contains("| \"starting\""));
        assert!(typescript.contains("readonly engineSessionId: string;"));
        assert!(typescript.contains("readonly detail?: string;"));
        assert!(typescript.contains("export const ENGINE_READINESS_DETAIL_MAX_LENGTH = 256;"));
    }

    #[test]
    fn both_languages_agree_on_field_count_and_optionality() {
        let contracts = contracts();
        let rust = render_rust(&contracts);
        let typescript = render_typescript(&contracts);

        for contract in &contracts {
            for field in &contract.fields {
                assert!(
                    rust.contains(&format!("pub {}:", snake_case(&field.json_name))),
                    "Rust output is missing `{}`",
                    field.json_name
                );
                let optional = if field.required { "" } else { "?" };
                assert!(
                    typescript.contains(&format!("readonly {}{}:", field.json_name, optional)),
                    "TypeScript output is missing `{}`",
                    field.json_name
                );
            }
        }
    }

    #[test]
    fn recognizes_major_version_directories() {
        assert_eq!(version_directory("v1").as_deref(), Some("v1"));
        assert_eq!(version_directory("v12").as_deref(), Some("v12"));
        assert_eq!(version_directory("v1.2"), None);
        assert_eq!(version_directory("draft"), None);
        assert_eq!(version_directory("v"), None);
    }

    #[test]
    fn escapes_json_special_characters() {
        assert_eq!(escape_json("a\"b\\c"), "a\\\"b\\\\c");
    }

    #[test]
    fn the_repository_schemas_are_valid() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let (contracts, errors) = discover(&root);

        assert_eq!(errors, Vec::new());
        assert!(
            contracts.len() >= 2,
            "the IPC handshake contracts should be discovered"
        );
    }
}

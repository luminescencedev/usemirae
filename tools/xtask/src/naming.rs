//! Identifier conversions shared by the code generators.
//!
//! Schemas use `camelCase` property names and `kebab-case` document names. Rust
//! wants `snake_case` fields and `PascalCase` types; TypeScript keeps the schema
//! spelling. Keeping the conversions here makes them testable in isolation.

/// Convert `kebab-case`, `snake_case`, or `camelCase` to `PascalCase`.
pub(crate) fn pascal_case(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut capitalize = true;

    for character in input.chars() {
        if character == '-' || character == '_' || character == ' ' {
            capitalize = true;
            continue;
        }

        if capitalize {
            out.extend(character.to_uppercase());
            capitalize = false;
        } else {
            out.push(character);
        }
    }

    out
}

/// Convert `camelCase`, `kebab-case`, or `PascalCase` to `snake_case`.
pub(crate) fn snake_case(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 4);

    for (index, character) in input.chars().enumerate() {
        if character == '-' || character == ' ' {
            out.push('_');
            continue;
        }

        if character.is_uppercase() {
            if index != 0 && !out.ends_with('_') {
                out.push('_');
            }
            out.extend(character.to_lowercase());
        } else {
            out.push(character);
        }
    }

    out
}

/// Convert any of the supported spellings to `SCREAMING_SNAKE_CASE`.
pub(crate) fn screaming_snake_case(input: &str) -> String {
    snake_case(input).to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_to_pascal_case() {
        assert_eq!(pascal_case("protocol-version"), "ProtocolVersion");
        assert_eq!(pascal_case("engine-readiness"), "EngineReadiness");
        assert_eq!(pascal_case("engineSessionId"), "EngineSessionId");
        assert_eq!(pascal_case("state"), "State");
    }

    #[test]
    fn converts_to_snake_case() {
        assert_eq!(snake_case("protocolMajor"), "protocol_major");
        assert_eq!(snake_case("engineSessionId"), "engine_session_id");
        assert_eq!(snake_case("state"), "state");
        assert_eq!(snake_case("engine-readiness"), "engine_readiness");
        assert_eq!(snake_case("ProtocolVersion"), "protocol_version");
    }

    #[test]
    fn converts_to_screaming_snake_case() {
        assert_eq!(screaming_snake_case("protocolMajor"), "PROTOCOL_MAJOR");
        assert_eq!(screaming_snake_case("engine-readiness"), "ENGINE_READINESS");
    }

    #[test]
    fn leaves_already_converted_input_stable() {
        assert_eq!(snake_case("protocol_major"), "protocol_major");
        assert_eq!(pascal_case("ProtocolVersion"), "ProtocolVersion");
    }
}

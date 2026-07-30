//! Stable machine-readable error codes.
//!
//! Canonical documentation: `docs/06-quality/605-error-model.md` section 4.

use core::fmt;

/// Longest accepted code, so a code can never grow unbounded through a macro.
const MAX_CODE_LENGTH: usize = 64;

/// A stable machine-readable error identifier such as `IPC_PROTOCOL_MISMATCH`.
///
/// Codes are `SCREAMING_SNAKE_CASE`, are part of the public contract, and must not
/// encode variable values: a disk path, an entity id, or a byte count belongs in
/// [`crate::ErrorContext`], not in the code. Automation and localization key off
/// the code, so changing one is a breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ErrorCode(&'static str);

impl ErrorCode {
    /// Build a code, returning `None` when it is not a valid identifier.
    ///
    /// Valid means non-empty, at most 64 bytes, `A`-`Z`, `0`-`9`, and `_` only,
    /// starting with a letter and never ending with or doubling an underscore.
    ///
    /// This is a `const fn`, so an invalid literal can be rejected at compile time
    /// by unwrapping it in a `const` binding.
    #[must_use]
    pub const fn new(code: &'static str) -> Option<Self> {
        let bytes = code.as_bytes();

        if bytes.is_empty() || bytes.len() > MAX_CODE_LENGTH {
            return None;
        }

        if !bytes[0].is_ascii_uppercase() {
            return None;
        }

        if bytes[bytes.len() - 1] == b'_' {
            return None;
        }

        let mut index = 0;
        while index < bytes.len() {
            let byte = bytes[index];
            let allowed = byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_';

            if !allowed {
                return None;
            }

            if byte == b'_' && index + 1 < bytes.len() && bytes[index + 1] == b'_' {
                return None;
            }

            index += 1;
        }

        Some(Self(code))
    }

    /// The code as written.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_documented_examples() {
        // Every example from 605 section 4 must be a valid code.
        for code in [
            "PROJECT_SCHEMA_UNSUPPORTED",
            "CAPTURE_PERMISSION_DENIED",
            "CAPTURE_SOURCE_REMOVED",
            "RENDER_DEVICE_LOST",
            "OUTPUT_AUTH_FAILED",
            "RECORDING_DISK_FULL",
            "IPC_PROTOCOL_MISMATCH",
            "EXTENSION_CAPABILITY_DENIED",
        ] {
            assert!(ErrorCode::new(code).is_some(), "rejected `{code}`");
        }
    }

    #[test]
    fn rejects_malformed_codes() {
        for code in [
            "",
            "lowercase_code",
            "9_LEADING_DIGIT",
            "_LEADING_UNDERSCORE",
            "TRAILING_UNDERSCORE_",
            "DOUBLE__UNDERSCORE",
            "HAS SPACE",
            "HAS-DASH",
            "HAS.DOT",
        ] {
            assert!(ErrorCode::new(code).is_none(), "accepted `{code}`");
        }
    }

    #[test]
    fn rejects_a_code_longer_than_the_bound() {
        let long: &'static str =
            "A_VERY_LONG_ERROR_CODE_THAT_KEEPS_GOING_AND_GOING_AND_GOING_PAST_LIMIT";

        assert!(long.len() > MAX_CODE_LENGTH);
        assert!(ErrorCode::new(long).is_none());
    }

    #[test]
    fn can_be_validated_at_compile_time() {
        // If the literal were invalid this would fail to compile, which is the
        // point of the const constructor.
        const CODE: ErrorCode = match ErrorCode::new("IPC_PROTOCOL_MISMATCH") {
            Some(code) => code,
            None => panic!("the literal above is a valid error code"),
        };

        assert_eq!(CODE.as_str(), "IPC_PROTOCOL_MISMATCH");
    }

    #[test]
    fn displays_as_the_bare_code() {
        let code = ErrorCode::new("RENDER_DEVICE_LOST");

        assert_eq!(
            code.map(|code| code.to_string()),
            Some("RENDER_DEVICE_LOST".to_owned())
        );
    }
}

//! Portable relative path validation.
//!
//! A [`SafeRelativePath`] is a lexical value, not a filesystem handle. It
//! rejects forms that are absolute, escape their root, or behave differently
//! on Windows and Unix. The caller must still resolve the path against a
//! canonical root and reject symlink escapes before opening it.

use core::fmt;

use crate::error::SecurityError;

/// A validated relative path that is portable across supported platforms.
///
/// Rules:
///
/// - non-empty and at most 1,024 bytes;
/// - no NUL, control character, backslash, or `:`;
/// - no Windows-invalid characters `* ? " < > |`;
/// - no leading `/`, drive prefix, or UNC prefix;
/// - no empty, `.` or `..` component;
/// - each component is at most 255 bytes and does not end in a space or dot;
/// - no Windows reserved device name as a component, with or without an
///   extension, case-insensitive.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SafeRelativePath(String);

impl SafeRelativePath {
    /// Maximum total length in bytes.
    pub const MAX_LEN: usize = 1024;

    /// Maximum length of one path component in bytes.
    pub const MAX_COMPONENT_LEN: usize = 255;

    /// Validates a relative path.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityError::UnsafePath`] with a short reason when the value
    /// violates any rule.
    pub fn new(value: impl Into<String>) -> Result<Self, SecurityError> {
        let value = value.into();
        validate(&value)?;
        Ok(Self(value))
    }

    /// Returns the path as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the path components.
    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }
}

impl fmt::Display for SafeRelativePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::str::FromStr for SafeRelativePath {
    type Err = SecurityError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

fn validate(value: &str) -> Result<(), SecurityError> {
    if value.is_empty() {
        return Err(unsafe_path("path is empty"));
    }
    if value.len() > SafeRelativePath::MAX_LEN {
        return Err(unsafe_path("path exceeds the maximum length"));
    }
    if value.starts_with('/') {
        return Err(unsafe_path("path must be relative"));
    }

    for byte in value.bytes() {
        if byte == b'\\' {
            return Err(unsafe_path("backslash is not allowed"));
        }
        if byte == b':' {
            return Err(unsafe_path("colon is not allowed"));
        }
        if byte < 0x20 || byte == 0x7f {
            return Err(unsafe_path("control characters are not allowed"));
        }
        if matches!(byte, b'*' | b'?' | b'"' | b'<' | b'>' | b'|') {
            return Err(unsafe_path("character is not portable"));
        }
    }

    for component in value.split('/') {
        if component.is_empty() {
            return Err(unsafe_path("empty path component"));
        }
        if component == "." || component == ".." {
            return Err(unsafe_path("dot path component is not allowed"));
        }
        if component.len() > SafeRelativePath::MAX_COMPONENT_LEN {
            return Err(unsafe_path("path component exceeds the maximum length"));
        }
        if component.ends_with(' ') || component.ends_with('.') {
            return Err(unsafe_path(
                "path component must not end with a space or dot",
            ));
        }
        if is_windows_reserved(component) {
            return Err(unsafe_path("Windows reserved device name"));
        }
    }

    Ok(())
}

fn unsafe_path(reason: &'static str) -> SecurityError {
    SecurityError::UnsafePath { reason }
}

fn is_windows_reserved(component: &str) -> bool {
    let stem = component
        .split_once('.')
        .map_or(component, |(stem, _extension)| stem);
    let stem = stem.to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "CONIN$"
            | "CONOUT$"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_portable_relative_paths() {
        for value in [
            "versions/0.1.0-0123456789ab",
            "active.json",
            "a/b/c.txt",
            "with space/file.txt",
        ] {
            assert!(
                SafeRelativePath::new(value).is_ok(),
                "{value:?} should be accepted"
            );
        }
    }

    #[test]
    fn rejects_absolute_and_traversal_paths() {
        for value in [
            "/etc/passwd",
            "../secret",
            "a/../../b",
            "a/./b",
            "a//b",
            "C:/Windows",
            "\\\\server\\share",
        ] {
            assert!(
                SafeRelativePath::new(value).is_err(),
                "{value:?} should be rejected"
            );
        }
    }

    #[test]
    fn rejects_windows_specific_forms() {
        for value in [
            "a\\b",
            "file:stream",
            "CON",
            "con.txt",
            "aux",
            "COM1.log",
            "LPT9",
            "trailing.",
            "trailing ",
            "bad*name",
            "bad?name",
            "bad<name",
            "bad>name",
            "bad|name",
            "bad\"name",
        ] {
            assert!(
                SafeRelativePath::new(value).is_err(),
                "{value:?} should be rejected"
            );
        }
    }

    #[test]
    fn rejects_control_characters_and_overlong_values() {
        assert!(SafeRelativePath::new("a\0b").is_err());
        assert!(SafeRelativePath::new("a\nb").is_err());
        assert!(SafeRelativePath::new("a".repeat(SafeRelativePath::MAX_LEN + 1)).is_err());
        assert!(
            SafeRelativePath::new("a".repeat(SafeRelativePath::MAX_COMPONENT_LEN + 1)).is_err()
        );
    }

    #[test]
    fn exposes_components() {
        let path = SafeRelativePath::new("versions/0.1.0-0123456789ab").unwrap();
        let components: Vec<_> = path.components().collect();
        assert_eq!(components, ["versions", "0.1.0-0123456789ab"]);
    }
}

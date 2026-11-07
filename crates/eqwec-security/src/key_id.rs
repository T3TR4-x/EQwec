//! Opaque signing key identifiers.

use core::fmt;

use crate::error::SecurityError;

/// Maximum length of a key identifier in bytes.
pub const MAX_KEY_ID_LEN: usize = 64;

/// A stable, opaque signing key identifier.
///
/// A key identifier is lowercase ASCII, starts and ends with an ASCII
/// alphanumeric character, and contains only lowercase ASCII letters, digits,
/// and hyphens. It is a lookup label, not a fingerprint; the public key itself
/// is pinned in the trust store.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KeyId(String);

impl KeyId {
    /// Creates a key identifier from a string.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityError::InvalidKeyId`] when the value is empty, too
    /// long, or contains a character outside the allowed set.
    pub fn new(value: impl Into<String>) -> Result<Self, SecurityError> {
        let value = value.into();
        validate_key_id(&value)?;
        Ok(Self(value))
    }

    /// Returns the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for KeyId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::str::FromStr for KeyId {
    type Err = SecurityError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

fn validate_key_id(value: &str) -> Result<(), SecurityError> {
    if value.is_empty() {
        return Err(SecurityError::InvalidKeyId {
            reason: "identifier is empty",
        });
    }
    if value.len() > MAX_KEY_ID_LEN {
        return Err(SecurityError::InvalidKeyId {
            reason: "identifier exceeds the maximum length",
        });
    }

    let bytes = value.as_bytes();
    let starts_and_ends_alphanumeric = bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric);
    if !starts_and_ends_alphanumeric {
        return Err(SecurityError::InvalidKeyId {
            reason: "identifier must start and end with an ASCII alphanumeric character",
        });
    }

    let allowed = bytes
        .iter()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-');
    if !allowed {
        return Err(SecurityError::InvalidKeyId {
            reason: "identifier must contain only lowercase ASCII letters, digits, and hyphens",
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_key_id() {
        let key = KeyId::new("release-2026-01").unwrap();
        assert_eq!(key.as_str(), "release-2026-01");
    }

    #[test]
    fn rejects_invalid_key_ids() {
        for value in ["", "Release", "-key", "key-", "key_1", "key 1", "key/1"] {
            assert!(KeyId::new(value).is_err(), "{value:?} should be rejected");
        }
    }

    #[test]
    fn rejects_overlong_key_id() {
        let value = "a".repeat(MAX_KEY_ID_LEN + 1);
        assert!(KeyId::new(value).is_err());
    }
}

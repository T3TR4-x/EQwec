//! Stable, opaque identifiers.

use core::fmt;

use crate::error::CoreError;

/// Maximum length of a stable identifier in bytes.
pub const MAX_ID_LEN: usize = 64;

/// A stable, opaque product identifier.
///
/// A product identifier is lowercase ASCII, starts and ends with an ASCII
/// alphanumeric character, and contains only lowercase ASCII letters, digits,
/// and hyphens. It is never derived from a display name or a filesystem path.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProductId(String);

impl ProductId {
    /// Creates a product identifier from a string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidProductId`] when the value is empty, too
    /// long, or contains a character outside the allowed set.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_id(&value).map_err(|reason| CoreError::InvalidProductId { reason })?;
        Ok(Self(value))
    }

    /// Returns the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProductId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::str::FromStr for ProductId {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// A stable, opaque tenant or profile scope identifier.
///
/// A scope identifier follows the same syntax as [`ProductId`]. It is used to
/// bind paths, caches, and IPC messages to an explicit profile or tenant.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopeId(String);

impl ScopeId {
    /// Creates a scope identifier from a string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidScopeId`] when the value is empty, too
    /// long, or contains a character outside the allowed set.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_id(&value).map_err(|reason| CoreError::InvalidScopeId { reason })?;
        Ok(Self(value))
    }

    /// Returns the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ScopeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::str::FromStr for ScopeId {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

fn validate_id(value: &str) -> Result<(), &'static str> {
    if value.is_empty() {
        return Err("identifier is empty");
    }
    if value.len() > MAX_ID_LEN {
        return Err("identifier exceeds the maximum length");
    }

    let bytes = value.as_bytes();
    let starts_and_ends_alphanumeric = bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric);
    if !starts_and_ends_alphanumeric {
        return Err("identifier must start and end with an ASCII alphanumeric character");
    }

    let allowed = bytes
        .iter()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-');
    if !allowed {
        return Err("identifier must contain only lowercase ASCII letters, digits, and hyphens");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_product_id() {
        let id = ProductId::new("eqwec-hub").expect("valid product id");
        assert_eq!(id.as_str(), "eqwec-hub");
        assert_eq!(id.to_string(), "eqwec-hub");
    }

    #[test]
    fn rejects_invalid_product_ids() {
        for value in [
            "",
            "EQWEC",
            "-hub",
            "hub-",
            "hub_browser",
            "hub browser",
            "hub/browser",
        ] {
            assert!(
                ProductId::new(value).is_err(),
                "{value:?} should be rejected"
            );
        }
    }

    #[test]
    fn rejects_overlong_product_id() {
        let value = "a".repeat(MAX_ID_LEN + 1);
        assert!(ProductId::new(value).is_err());
    }

    #[test]
    fn accepts_maximum_length_product_id() {
        let value = "a".repeat(MAX_ID_LEN);
        assert!(ProductId::new(value).is_ok());
    }

    #[test]
    fn parses_product_id_from_str() {
        let id: ProductId = "photo-craft".parse().expect("valid product id");
        assert_eq!(id.as_str(), "photo-craft");
    }

    #[test]
    fn accepts_canonical_scope_id() {
        let id = ScopeId::new("profile-1").expect("valid scope id");
        assert_eq!(id.as_str(), "profile-1");
    }

    #[test]
    fn rejects_invalid_scope_ids() {
        for value in ["", "Profile", "-work", "work-", "work profile"] {
            assert!(ScopeId::new(value).is_err(), "{value:?} should be rejected");
        }
    }
}

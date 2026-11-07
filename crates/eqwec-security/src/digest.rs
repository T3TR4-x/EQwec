//! SHA-256 digests.

use core::fmt;

use sha2::{Digest, Sha256};

use crate::error::SecurityError;
use crate::hex::{decode_lower_hex, encode_lower_hex};

/// A SHA-256 digest.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Sha256Digest([u8; Self::LEN]);

impl Sha256Digest {
    /// Length of a SHA-256 digest in bytes.
    pub const LEN: usize = 32;

    /// Creates a digest from raw bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; Self::LEN]) -> Self {
        Self(bytes)
    }

    /// Parses a digest from exactly 64 lowercase hexadecimal characters.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityError::InvalidHexLength`] or
    /// [`SecurityError::InvalidHexCharacter`] for a malformed value.
    pub fn parse_hex(value: &str) -> Result<Self, SecurityError> {
        let bytes = decode_lower_hex(value, Self::LEN)?;
        let mut digest = [0u8; Self::LEN];
        digest.copy_from_slice(&bytes);
        Ok(Self(digest))
    }

    /// Returns the raw digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; Self::LEN] {
        &self.0
    }

    /// Returns the digest as 64 lowercase hexadecimal characters.
    #[must_use]
    pub fn to_hex(&self) -> String {
        encode_lower_hex(&self.0)
    }

    /// Computes the SHA-256 digest of a byte slice.
    #[must_use]
    pub fn hash(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let mut output = [0u8; Self::LEN];
        output.copy_from_slice(&digest);
        Self(output)
    }
}

impl fmt::Debug for Sha256Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("Sha256Digest")
            .field(&self.to_hex())
            .finish()
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_the_empty_input() {
        let digest = Sha256Digest::hash(b"");
        assert_eq!(
            digest.to_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn hashes_abc() {
        let digest = Sha256Digest::hash(b"abc");
        assert_eq!(
            digest.to_hex(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn parses_canonical_hex() {
        let value = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        let digest = Sha256Digest::parse_hex(value).unwrap();
        assert_eq!(digest.to_hex(), value);
    }

    #[test]
    fn rejects_noncanonical_hex() {
        let value = "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD";
        assert!(Sha256Digest::parse_hex(value).is_err());
    }
}

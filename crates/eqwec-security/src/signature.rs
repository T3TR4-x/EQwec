//! Ed25519 public keys and strict signature verification.

use core::fmt;

use ed25519_dalek::{Signature, VerifyingKey};

use crate::error::SecurityError;
use crate::hex::{decode_lower_hex, encode_lower_hex};

/// Length of an Ed25519 public key in bytes.
pub const PUBLIC_KEY_LEN: usize = 32;

/// Length of an Ed25519 signature in bytes.
pub const SIGNATURE_LEN: usize = 64;

/// An Ed25519 signature.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SignatureBytes([u8; SIGNATURE_LEN]);

impl SignatureBytes {
    /// Creates a signature from raw bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; SIGNATURE_LEN]) -> Self {
        Self(bytes)
    }

    /// Parses a signature from exactly 128 lowercase hexadecimal characters.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityError::InvalidHexLength`] or
    /// [`SecurityError::InvalidHexCharacter`] for a malformed value.
    pub fn parse_hex(value: &str) -> Result<Self, SecurityError> {
        let bytes = decode_lower_hex(value, SIGNATURE_LEN)?;
        let mut signature = [0u8; SIGNATURE_LEN];
        signature.copy_from_slice(&bytes);
        Ok(Self(signature))
    }

    /// Returns the raw signature bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; SIGNATURE_LEN] {
        &self.0
    }

    /// Returns the signature as 128 lowercase hexadecimal characters.
    #[must_use]
    pub fn to_hex(&self) -> String {
        encode_lower_hex(&self.0)
    }
}

impl fmt::Debug for SignatureBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("SignatureBytes")
            .field(&self.to_hex())
            .finish()
    }
}

/// A validated Ed25519 public key.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Ed25519PublicKey(VerifyingKey);

impl Ed25519PublicKey {
    /// Parses a public key from exactly 64 lowercase hexadecimal characters.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityError::InvalidHexLength`] or
    /// [`SecurityError::InvalidHexCharacter`] for a malformed value, and
    /// [`SecurityError::InvalidPublicKey`] when the bytes are not a valid
    /// Ed25519 point.
    pub fn parse_hex(value: &str) -> Result<Self, SecurityError> {
        let bytes = decode_lower_hex(value, PUBLIC_KEY_LEN)?;
        let mut key = [0u8; PUBLIC_KEY_LEN];
        key.copy_from_slice(&bytes);
        Self::from_bytes(key)
    }

    /// Creates a public key from raw bytes.
    ///
    /// Weak (small-order) keys are rejected at construction so that no caller
    /// can accidentally rely on a key that makes signatures forgeable for
    /// every message.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityError::InvalidPublicKey`] when the bytes are not a
    /// valid Ed25519 point or encode a weak key.
    pub fn from_bytes(bytes: [u8; PUBLIC_KEY_LEN]) -> Result<Self, SecurityError> {
        let key = VerifyingKey::from_bytes(&bytes).map_err(|_| SecurityError::InvalidPublicKey)?;
        if key.is_weak() {
            return Err(SecurityError::InvalidPublicKey);
        }
        Ok(Self(key))
    }

    /// Returns the public key as 64 lowercase hexadecimal characters.
    #[must_use]
    pub fn to_hex(&self) -> String {
        encode_lower_hex(self.0.as_bytes())
    }

    /// Verifies a signature over a message with strict Ed25519 rules.
    ///
    /// Strict verification rejects non-canonical scalar encodings and
    /// small-order public keys. There is no non-strict fallback.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityError::SignatureVerification`] when the signature is
    /// invalid.
    pub fn verify_strict(
        &self,
        message: &[u8],
        signature: &SignatureBytes,
    ) -> Result<(), SecurityError> {
        let signature = Signature::from_bytes(signature.as_bytes());
        self.0
            .verify_strict(message, &signature)
            .map_err(|_| SecurityError::SignatureVerification)
    }
}

impl fmt::Debug for Ed25519PublicKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("Ed25519PublicKey")
            .field(&self.to_hex())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 8032 §7.1, test vector 1. This is a public conformance vector, not
    /// an EQwec production key.
    const RFC8032_PUBLIC_KEY: &str =
        "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    const RFC8032_SIGNATURE: &str = "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b";

    #[test]
    fn verifies_rfc8032_vector_one() {
        let key = Ed25519PublicKey::parse_hex(RFC8032_PUBLIC_KEY).unwrap();
        let signature = SignatureBytes::parse_hex(RFC8032_SIGNATURE).unwrap();
        key.verify_strict(b"", &signature).unwrap();
    }

    #[test]
    fn rejects_signature_over_different_message() {
        let key = Ed25519PublicKey::parse_hex(RFC8032_PUBLIC_KEY).unwrap();
        let signature = SignatureBytes::parse_hex(RFC8032_SIGNATURE).unwrap();
        assert_eq!(
            key.verify_strict(b"not empty", &signature).unwrap_err(),
            SecurityError::SignatureVerification
        );
    }

    #[test]
    fn rejects_tampered_signature() {
        let key = Ed25519PublicKey::parse_hex(RFC8032_PUBLIC_KEY).unwrap();
        let mut bytes = *SignatureBytes::parse_hex(RFC8032_SIGNATURE)
            .unwrap()
            .as_bytes();
        bytes[0] ^= 0x01;
        let signature = SignatureBytes::from_bytes(bytes);
        assert_eq!(
            key.verify_strict(b"", &signature).unwrap_err(),
            SecurityError::SignatureVerification
        );
    }

    #[test]
    fn rejects_uppercase_public_key() {
        assert!(Ed25519PublicKey::parse_hex(&RFC8032_PUBLIC_KEY.to_uppercase()).is_err());
    }

    #[test]
    fn rejects_invalid_public_key_point() {
        // y = 2 has no matching x on curve25519, so decompression must fail.
        let mut bytes = [0u8; PUBLIC_KEY_LEN];
        bytes[0] = 2;
        assert_eq!(
            Ed25519PublicKey::from_bytes(bytes).unwrap_err(),
            SecurityError::InvalidPublicKey
        );
    }

    #[test]
    fn rejects_weak_public_key() {
        // The identity point is a valid small-order encoding.
        let mut bytes = [0u8; PUBLIC_KEY_LEN];
        bytes[0] = 1;
        assert_eq!(
            Ed25519PublicKey::from_bytes(bytes).unwrap_err(),
            SecurityError::InvalidPublicKey
        );
    }
}

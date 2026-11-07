//! Typed errors for security primitives.

use thiserror::Error;

/// An error produced while constructing or verifying a security primitive.
///
/// Errors never contain secret key material or full user paths.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum SecurityError {
    /// A hexadecimal value had the wrong number of characters.
    #[error("invalid hex length: expected {expected_bytes} bytes, got {actual_chars} characters")]
    InvalidHexLength {
        /// Expected number of decoded bytes.
        expected_bytes: usize,
        /// Actual number of input characters.
        actual_chars: usize,
    },
    /// A hexadecimal value contained a character outside lowercase `0-9a-f`.
    #[error("invalid lowercase hex character at index {index}")]
    InvalidHexCharacter {
        /// Byte index of the rejected character.
        index: usize,
    },
    /// A key identifier failed validation.
    #[error("invalid key identifier: {reason}")]
    InvalidKeyId {
        /// A short, human-readable reason.
        reason: &'static str,
    },
    /// A public key was not a valid Ed25519 point.
    #[error("invalid Ed25519 public key")]
    InvalidPublicKey,
    /// A signature was not exactly 64 bytes.
    #[error("invalid Ed25519 signature encoding")]
    InvalidSignature,
    /// A strict Ed25519 verification failed.
    #[error("Ed25519 signature verification failed")]
    SignatureVerification,
    /// A relative path failed validation.
    #[error("unsafe relative path: {reason}")]
    UnsafePath {
        /// A short, human-readable reason.
        reason: &'static str,
    },
}

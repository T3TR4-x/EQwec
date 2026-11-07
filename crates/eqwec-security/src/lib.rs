#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Security primitives shared by EQwec products.
//!
//! This crate owns the small, security-critical building blocks that must not
//! be reimplemented per product:
//!
//! - lowercase hexadecimal encoding and decoding;
//! - SHA-256 digests;
//! - Ed25519 public keys and strict signature verification;
//! - validated opaque key identifiers;
//! - portable relative path validation.
//!
//! It does not own trust policy, manifest schemas, network access, or file
//! system mutation. Those belong to `eqwec-updater` and the product that owns
//! the operation.

pub mod digest;
pub mod error;
pub mod hex;
pub mod key_id;
pub mod path;
pub mod signature;

pub use digest::Sha256Digest;
pub use error::SecurityError;
pub use key_id::KeyId;
pub use path::SafeRelativePath;
pub use signature::{Ed25519PublicKey, SignatureBytes};

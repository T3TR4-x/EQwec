#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Hub manifest verification, trust policy, and package integrity.
//!
//! This crate owns the generic verification path used by EQwec Hub:
//!
//! - bounded manifest v1 parsing and canonical byte generation;
//! - pinned trust-store parsing and key selection;
//! - strict Ed25519 signature verification;
//! - channel, target, time, epoch, revocation, and rollback policy;
//! - package size and SHA-256 verification.
//!
//! It does not download packages, extract archives, mutate an installation,
//! or hold signing keys. Those belong to the Hub application and its
//! transaction engine.

pub mod error;
pub mod manifest;
pub mod package;
pub mod time;
pub mod trust;
pub mod url;
pub mod verify;

pub use error::UpdaterError;
pub use manifest::{
    Dependency, MANIFEST_SCHEMA_VERSION, MAX_DEPENDENCIES, MAX_MANIFEST_BYTES, MAX_PACKAGE_BYTES,
    MAX_RELEASE_NOTES_BYTES, Manifest, Package, PackageHash, SignatureAlgorithm, SignatureBlock,
    Target,
};
pub use package::VerifiedPackage;
pub use time::{CLOCK_SKEW_SECONDS, MAX_KEY_VALIDITY_SECONDS, MAX_VALIDITY_SECONDS, Timestamp};
pub use trust::{MAX_TRUST_KEYS, MAX_TRUST_STORE_BYTES, TrustStore, TrustedKey};
pub use url::HttpsUrl;
pub use verify::{VerificationContext, VerifiedManifest, parse_and_verify};

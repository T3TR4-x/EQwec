#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Hub manifest verification, trust policy, package integrity, and durable
//! installation state.
//!
//! This crate owns the generic verification and transaction primitives used
//! by EQwec Hub:
//!
//! - bounded manifest v1 parsing and canonical byte generation;
//! - pinned trust-store parsing and key selection;
//! - strict Ed25519 signature verification;
//! - channel, target, time, epoch, revocation, and rollback policy;
//! - package size and SHA-256 verification;
//! - registry, journal, active pointer, and installation layout primitives;
//! - bounded package archive validation and staging.
//!
//! It does not download packages or hold signing keys. The Hub application
//! owns the transport and the profile lock; activation and staging are
//! implemented here.

mod atomic;

pub mod activation;
pub mod active;
pub mod error;
pub mod journal;
pub mod layout;
pub mod manifest;
pub mod package;
pub mod registry;
pub mod staging;
pub mod time;
pub mod trust;
pub mod url;
pub mod verify;

pub use activation::{
    ActivationOutcome, FaultInjector, FaultPoint, NoFaults, OperationSpec, RecoveryOutcome,
    RollbackSpec, VerifiedRelease, activate, record_phase, recover, rollback,
};
pub use active::{ACTIVE_POINTER_SCHEMA_VERSION, ActivePointer, MAX_ACTIVE_POINTER_BYTES};
pub use error::UpdaterError;
pub use journal::{
    IdempotencyKey, JOURNAL_SCHEMA_VERSION, JournalContents, JournalContext, JournalRecord,
    MAX_IDEMPOTENCY_KEY_BYTES, MAX_JOURNAL_BYTES, MAX_JOURNAL_RECORD_BYTES, OperationId, Phase,
    RecoveryAction, append_journal_record, read_journal, recovery_action,
};
pub use layout::{InstallLayout, MAX_PATH_BYTES, is_version_tree, tree_id};
pub use manifest::{
    Dependency, MANIFEST_SCHEMA_VERSION, MAX_DEPENDENCIES, MAX_MANIFEST_BYTES, MAX_PACKAGE_BYTES,
    MAX_RELEASE_NOTES_BYTES, Manifest, Package, PackageHash, SignatureAlgorithm, SignatureBlock,
    Target,
};
pub use package::VerifiedPackage;
pub use registry::{
    InstallationRecord, MAX_INSTALLATIONS, MAX_REGISTRY_BYTES, MAX_RETAINED_VERSIONS,
    REGISTRY_SCHEMA_VERSION, Registry, RetainedVersion,
};
pub use staging::{
    DEFAULT_EXTRACTION_TIMEOUT, ExtractionLimits, MAX_ARCHIVE_COMPONENT_BYTES, MAX_ARCHIVE_DEPTH,
    MAX_ARCHIVE_ENTRIES, MAX_ARCHIVE_ENTRY_RATIO, MAX_ARCHIVE_ENTRY_UNCOMPRESSED_BYTES,
    MAX_ARCHIVE_PATH_BYTES, MAX_ARCHIVE_TOTAL_RATIO, MAX_ARCHIVE_TOTAL_UNCOMPRESSED_BYTES,
    StagedEntry, StagedEntryKind, StagedTree, stage_package, stage_package_from_path,
};
pub use time::{CLOCK_SKEW_SECONDS, MAX_KEY_VALIDITY_SECONDS, MAX_VALIDITY_SECONDS, Timestamp};
pub use trust::{MAX_TRUST_KEYS, MAX_TRUST_STORE_BYTES, TrustStore, TrustedKey};
pub use url::HttpsUrl;
pub use verify::{VerificationContext, VerifiedManifest, parse_and_verify};

//! Typed errors for manifest, trust, and package verification.

use eqwec_core::Channel;
use eqwec_security::SecurityError;
use thiserror::Error;

/// Maximum length of a diagnostic detail string.
const MAX_DETAIL_LEN: usize = 200;

/// An error produced while parsing or verifying Hub metadata.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum UpdaterError {
    /// The manifest exceeded the encoded size bound.
    #[error("manifest exceeds {max} bytes: {actual} bytes")]
    ManifestTooLarge {
        /// Maximum accepted bytes.
        max: usize,
        /// Actual input length.
        actual: usize,
    },
    /// The trust store exceeded the encoded size bound.
    #[error("trust store exceeds {max} bytes: {actual} bytes")]
    TrustStoreTooLarge {
        /// Maximum accepted bytes.
        max: usize,
        /// Actual input length.
        actual: usize,
    },
    /// The input was not valid UTF-8.
    #[error("input is not valid UTF-8")]
    InvalidUtf8,
    /// The JSON was syntactically invalid or had the wrong type.
    #[error("malformed JSON: {detail}")]
    MalformedJson {
        /// Bounded diagnostic detail.
        detail: String,
    },
    /// A duplicate or unknown field was present.
    #[error("duplicate or unknown field: {detail}")]
    UnknownOrDuplicateField {
        /// Bounded diagnostic detail.
        detail: String,
    },
    /// Non-whitespace data followed the JSON document.
    #[error("trailing data after the JSON document")]
    TrailingData,
    /// The schema version is not supported.
    #[error("unsupported schema version {found}; supported version is {supported}")]
    UnsupportedSchemaVersion {
        /// Version found in the document.
        found: u64,
        /// Version supported by this implementation.
        supported: u64,
    },
    /// A product identifier failed validation.
    #[error("invalid product identifier: {reason}")]
    InvalidProductId {
        /// A short, human-readable reason.
        reason: String,
    },
    /// A semantic version failed validation.
    #[error("invalid semantic version: {value}")]
    InvalidVersion {
        /// The rejected value.
        value: String,
    },
    /// A version constraint failed validation.
    #[error("invalid version constraint: {value}")]
    InvalidVersionConstraint {
        /// The rejected value.
        value: String,
    },
    /// A release channel name is unknown.
    #[error("invalid release channel: {value}")]
    InvalidChannel {
        /// The rejected value.
        value: String,
    },
    /// A platform name is unknown.
    #[error("invalid platform: {value}")]
    InvalidPlatform {
        /// The rejected value.
        value: String,
    },
    /// An architecture name is unknown.
    #[error("invalid architecture: {value}")]
    InvalidArchitecture {
        /// The rejected value.
        value: String,
    },
    /// The minimum platform string failed validation.
    #[error("invalid minimum platform string")]
    InvalidMinimumPlatform,
    /// The package source URL failed validation.
    #[error("invalid package source URL: {reason}")]
    InvalidPackageSource {
        /// A short, human-readable reason.
        reason: &'static str,
    },
    /// The package size is outside the accepted range.
    #[error("invalid package size: {size}")]
    InvalidPackageSize {
        /// The rejected size.
        size: u64,
    },
    /// A package hash failed validation.
    #[error("invalid package hash: {reason}")]
    InvalidHash {
        /// A short, human-readable reason.
        reason: String,
    },
    /// A dependency failed validation.
    #[error("invalid dependency: {reason}")]
    InvalidDependency {
        /// A short, human-readable reason.
        reason: String,
    },
    /// Too many dependencies were present.
    #[error("too many dependencies: {actual}; maximum is {max}")]
    TooManyDependencies {
        /// Maximum accepted count.
        max: usize,
        /// Actual count.
        actual: usize,
    },
    /// The metadata sequence is invalid.
    #[error("metadata sequence must be at least 1")]
    InvalidSequence,
    /// A timestamp failed validation.
    #[error("invalid timestamp: {value}")]
    InvalidTimestamp {
        /// The rejected value or a short reason.
        value: String,
    },
    /// Release notes failed validation.
    #[error("invalid release notes: {reason}")]
    InvalidReleaseNotes {
        /// A short, human-readable reason.
        reason: &'static str,
    },
    /// A signature field failed validation.
    #[error("invalid signature field: {reason}")]
    InvalidSignatureField {
        /// A short, human-readable reason.
        reason: String,
    },
    /// The manifest validity window is invalid.
    #[error("manifest validity window must be positive and at most 366 days")]
    InvalidValidityWindow,
    /// The signing time is outside the manifest validity window.
    #[error("signing time is outside the manifest validity window")]
    InvalidSigningTime,
    /// The manifest channel does not match the requested channel.
    #[error("channel mismatch: requested {expected}, manifest declares {found}")]
    ChannelMismatch {
        /// Requested channel.
        expected: Channel,
        /// Manifest channel.
        found: Channel,
    },
    /// The manifest target does not match the requested target.
    #[error("target mismatch: requested {expected}, manifest declares {found}")]
    TargetMismatch {
        /// Requested target.
        expected: String,
        /// Manifest target.
        found: String,
    },
    /// The manifest is not yet valid.
    #[error("manifest is not yet valid")]
    ManifestNotYetValid,
    /// The manifest has expired.
    #[error("manifest has expired")]
    ManifestExpired,
    /// The metadata sequence regressed.
    #[error("metadata sequence regression: minimum {minimum}, found {found}")]
    MetadataSequenceRegression {
        /// Minimum accepted sequence.
        minimum: u64,
        /// Sequence found in the manifest.
        found: u64,
    },
    /// The signing key is not in the trust store.
    #[error("unknown signing key: {key_id}")]
    UnknownSigningKey {
        /// The rejected key identifier.
        key_id: String,
    },
    /// The signing key is revoked.
    #[error("signing key is revoked: {key_id}")]
    SigningKeyRevoked {
        /// The revoked key identifier.
        key_id: String,
    },
    /// The signing key epoch does not match.
    #[error("signing key epoch mismatch: expected {expected}, found {found}")]
    SigningKeyEpochMismatch {
        /// Trust-store epoch.
        expected: u64,
        /// Manifest epoch.
        found: u64,
    },
    /// The signing key is not allowed for the manifest channel.
    #[error("signing key {key_id} is not allowed for channel {channel}")]
    SigningKeyChannelNotAllowed {
        /// The key identifier.
        key_id: String,
        /// The rejected channel.
        channel: Channel,
    },
    /// The signing key is not yet valid at the signing time.
    #[error("signing key is not yet valid")]
    SigningKeyNotYetValid,
    /// The signing key has expired at the signing time.
    #[error("signing key has expired")]
    SigningKeyExpired,
    /// The signature algorithm is not supported.
    #[error("unsupported signature algorithm: {algorithm}")]
    SignatureAlgorithmUnsupported {
        /// The rejected algorithm.
        algorithm: String,
    },
    /// The signature is invalid.
    #[error("signature verification failed")]
    SignatureInvalid,
    /// A security primitive failed.
    #[error(transparent)]
    Security(#[from] SecurityError),
    /// The package is larger than the accepted maximum.
    #[error("package exceeds {max} bytes: {actual} bytes")]
    PackageTooLarge {
        /// Maximum accepted bytes.
        max: u64,
        /// Actual bytes observed.
        actual: u64,
    },
    /// The package size does not match the manifest.
    #[error("package size mismatch: expected {expected}, got {actual}")]
    PackageSizeMismatch {
        /// Expected size.
        expected: u64,
        /// Actual size.
        actual: u64,
    },
    /// The package ended before the declared size.
    #[error("package truncated: expected {expected}, got {actual}")]
    PackageTruncated {
        /// Expected size.
        expected: u64,
        /// Actual size.
        actual: u64,
    },
    /// The package hash does not match the manifest.
    #[error("package hash mismatch: expected {expected}, got {actual}")]
    PackageHashMismatch {
        /// Expected lowercase hex digest.
        expected: String,
        /// Actual lowercase hex digest.
        actual: String,
    },
    /// A trust store has too many keys.
    #[error("trust store has {actual} keys; maximum is {max}")]
    TooManyTrustKeys {
        /// Maximum accepted count.
        max: usize,
        /// Actual count.
        actual: usize,
    },
    /// A trust-store key identifier is duplicated.
    #[error("duplicate trust-store key: {key_id}")]
    DuplicateTrustKey {
        /// The duplicated key identifier.
        key_id: String,
    },
    /// A trust-store key failed validation.
    #[error("invalid trust-store key: {reason}")]
    InvalidTrustKey {
        /// A short, human-readable reason.
        reason: String,
    },
    /// Trust-store keys are not sorted by key identifier.
    #[error("trust-store keys must be sorted by key_id")]
    TrustStoreOrder,
    /// The registry exceeded the encoded size bound.
    #[error("registry exceeds {max} bytes: {actual} bytes")]
    RegistryTooLarge {
        /// Maximum accepted bytes.
        max: usize,
        /// Actual input length.
        actual: usize,
    },
    /// The registry has too many installation records.
    #[error("registry has {actual} installations; maximum is {max}")]
    TooManyInstallations {
        /// Maximum accepted count.
        max: usize,
        /// Actual count.
        actual: usize,
    },
    /// A registry record failed validation.
    #[error("invalid registry: {reason}")]
    InvalidRegistry {
        /// A short, human-readable reason.
        reason: String,
    },
    /// Two registry records share a product identifier.
    #[error("duplicate installation record: {product_id}")]
    DuplicateInstallation {
        /// The duplicated product identifier.
        product_id: String,
    },
    /// Registry records are not sorted by product identifier.
    #[error("registry installations must be sorted by product_id")]
    RegistryOrder,
    /// An installation root failed validation.
    #[error("invalid installation root: {reason}")]
    InvalidInstallRoot {
        /// A short, human-readable reason.
        reason: String,
    },
    /// An installation root is not inside the profile base.
    #[error("installation root escapes the profile base")]
    InstallRootOutsideBase,
    /// The journal file exceeded the size bound.
    #[error("journal exceeds {max} bytes: {actual} bytes")]
    JournalTooLarge {
        /// Maximum accepted bytes.
        max: u64,
        /// Actual input length.
        actual: u64,
    },
    /// A journal record exceeded the size bound.
    #[error("journal record exceeds {max} bytes: {actual} bytes")]
    JournalRecordTooLarge {
        /// Maximum accepted bytes.
        max: usize,
        /// Actual record length.
        actual: usize,
    },
    /// An operation identifier failed validation.
    #[error("invalid operation identifier: {reason}")]
    InvalidOperationId {
        /// A short, human-readable reason.
        reason: &'static str,
    },
    /// An idempotency key failed validation.
    #[error("invalid idempotency key: {reason}")]
    InvalidIdempotencyKey {
        /// A short, human-readable reason.
        reason: &'static str,
    },
    /// A journal phase name is unknown.
    #[error("invalid journal phase: {value}")]
    InvalidJournalPhase {
        /// The rejected value.
        value: String,
    },
    /// A journal record failed validation.
    #[error("invalid journal record: {reason}")]
    InvalidJournalRecord {
        /// A short, human-readable reason.
        reason: String,
    },
    /// A complete journal line is malformed.
    #[error("journal is corrupt: {detail}")]
    JournalCorrupt {
        /// Bounded diagnostic detail.
        detail: String,
    },
    /// The active pointer exceeded the encoded size bound.
    #[error("active pointer exceeds {max} bytes: {actual} bytes")]
    ActivePointerTooLarge {
        /// Maximum accepted bytes.
        max: usize,
        /// Actual input length.
        actual: usize,
    },
    /// The active pointer failed validation.
    #[error("invalid active pointer: {reason}")]
    InvalidActivePointer {
        /// A short, human-readable reason.
        reason: String,
    },
    /// A tree identifier failed validation.
    #[error("invalid tree identifier: {reason}")]
    InvalidTreeId {
        /// A short, human-readable reason.
        reason: String,
    },
    /// The staging directory does not exist.
    #[error("staging directory is missing: {path}")]
    StagingMissing {
        /// The missing path.
        path: String,
    },
    /// The staging path is not a real directory.
    #[error("invalid staging directory: {reason}")]
    InvalidStaging {
        /// A short, human-readable reason.
        reason: String,
    },
    /// The package archive is malformed or violates archive v1.
    #[error("invalid package archive: {reason}")]
    ArchiveInvalid {
        /// Bounded diagnostic detail.
        reason: String,
    },
    /// The package archive exceeds a configured bound.
    #[error("package archive {limit} exceeds the maximum {max}: {actual}")]
    ArchiveLimitExceeded {
        /// The bound name.
        limit: &'static str,
        /// Maximum accepted value.
        max: u64,
        /// Actual observed value.
        actual: u64,
    },
    /// An archive entry path is unsafe.
    #[error("rejected archive path {path}: {reason}")]
    ArchivePathRejected {
        /// The rejected path, escaped for display.
        path: String,
        /// A short, human-readable reason.
        reason: &'static str,
    },
    /// An archive entry is not allowed.
    #[error("rejected archive entry {index}: {reason}")]
    ArchiveEntryRejected {
        /// The central-directory index.
        index: usize,
        /// A short, human-readable reason.
        reason: String,
    },
    /// Extraction exceeded the time bound.
    #[error("package extraction exceeded {seconds} seconds")]
    ArchiveTimeout {
        /// The configured time bound in seconds.
        seconds: u64,
    },
    /// The staging directory cannot be created or used safely.
    #[error("staging rejected: {reason}")]
    StagingRejected {
        /// A short, human-readable reason.
        reason: String,
    },
    /// The staged tree failed post-extraction verification.
    #[error("staged tree verification failed: {reason}")]
    StagingVerification {
        /// A short, human-readable reason.
        reason: String,
    },
    /// The verified manifest does not match the operation release identity.
    #[error("package identity mismatch: {field}")]
    PackageIdentityMismatch {
        /// The mismatched field.
        field: &'static str,
    },
    /// Extraction limits are invalid.
    #[error("invalid extraction limits: {reason}")]
    InvalidExtractionLimits {
        /// A short, human-readable reason.
        reason: &'static str,
    },
    /// The target version tree already exists and is immutable.
    #[error("version tree already exists: {tree}")]
    TreeAlreadyExists {
        /// The conflicting tree path.
        tree: String,
    },
    /// No installation record exists for the product.
    #[error("no installation record for {product_id}")]
    NoInstallation {
        /// The product identifier.
        product_id: String,
    },
    /// An installation record already exists for a fresh install.
    #[error("an installation record already exists for {product_id}")]
    InstallationAlreadyExists {
        /// The product identifier.
        product_id: String,
    },
    /// The registry version differs from the operation's source version.
    #[error("installation version mismatch: expected {expected}, found {found}")]
    InstallationVersionMismatch {
        /// The expected version.
        expected: String,
        /// The recorded version.
        found: String,
    },
    /// The requested retained version does not exist.
    #[error("no retained version {version} for {product_id}")]
    NoSuchRetainedVersion {
        /// The product identifier.
        product_id: String,
        /// The requested version.
        version: String,
    },
    /// A journal phase is not valid for the current operation state.
    #[error("invalid operation transition: expected {expected}, found {found}")]
    InvalidOperationTransition {
        /// The expected previous phase, or `none`.
        expected: &'static str,
        /// The actual previous phase, or `none`.
        found: &'static str,
    },
    /// Recovery found inconsistent durable state and refuses to guess.
    #[error("recovery conflict: {detail}")]
    RecoveryConflict {
        /// Bounded diagnostic detail.
        detail: String,
    },
    /// A test fault injector aborted the transaction.
    #[error("fault injected at {point}")]
    FaultInjected {
        /// The fault point name.
        point: &'static str,
    },
    /// An atomic filesystem write failed.
    #[error("atomic write failed: {detail}")]
    AtomicWrite {
        /// Bounded diagnostic detail.
        detail: String,
    },
    /// A filesystem read failed.
    #[error("I/O error: {detail}")]
    Io {
        /// Bounded diagnostic detail.
        detail: String,
    },
    /// The system clock is unavailable or outside the supported range.
    #[error("clock error: {detail}")]
    Clock {
        /// Bounded diagnostic detail.
        detail: String,
    },
}

impl UpdaterError {
    /// Creates a bounded diagnostic detail string.
    #[must_use]
    pub(crate) fn bounded_detail(detail: impl Into<String>) -> String {
        let mut detail = detail.into();
        if detail.len() > MAX_DETAIL_LEN {
            let mut boundary = MAX_DETAIL_LEN;
            while !detail.is_char_boundary(boundary) {
                boundary -= 1;
            }
            detail.truncate(boundary);
            detail.push('…');
        }
        detail
    }

    /// Maps a `serde_json` error to a typed parse error.
    pub(crate) fn from_json(error: serde_json::Error) -> Self {
        let detail = Self::bounded_detail(error.to_string());
        if detail.starts_with("duplicate field") || detail.starts_with("unknown field") {
            Self::UnknownOrDuplicateField { detail }
        } else {
            Self::MalformedJson { detail }
        }
    }
}

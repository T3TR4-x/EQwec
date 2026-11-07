//! Hub manifest v1 parsing, canonicalization, and typed access.

use eqwec_core::{Architecture, Channel, CoreError, Platform, ProductId};
use eqwec_security::{KeyId, Sha256Digest, SignatureBytes};
use semver::{Version, VersionReq};
use serde::Deserialize;

use crate::error::UpdaterError;
use crate::time::{CLOCK_SKEW_SECONDS, MAX_VALIDITY_SECONDS, Timestamp};
use crate::url::HttpsUrl;

/// Supported manifest schema version.
pub const MANIFEST_SCHEMA_VERSION: u64 = 1;

/// Maximum encoded manifest size in bytes.
pub const MAX_MANIFEST_BYTES: usize = 65_536;

/// Maximum dependency count.
pub const MAX_DEPENDENCIES: usize = 32;

/// Maximum release-notes size in bytes.
pub const MAX_RELEASE_NOTES_BYTES: usize = 4_096;

/// Maximum package size in bytes (8 GiB).
pub const MAX_PACKAGE_BYTES: u64 = 8 * 1024 * 1024 * 1024;

/// Maximum raw semantic-version length in bytes.
pub const MAX_VERSION_BYTES: usize = 128;

/// Maximum raw version-constraint length in bytes.
pub const MAX_CONSTRAINT_BYTES: usize = 128;

/// Maximum minimum-platform string length in bytes.
pub const MAX_MINIMUM_PLATFORM_BYTES: usize = 64;

const ED25519: &str = "ed25519";
const SHA256: &str = "sha256";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireManifest {
    schema_version: u64,
    product_id: String,
    product_version: String,
    channel: String,
    target: WireTarget,
    package: WirePackage,
    dependencies: Vec<WireDependency>,
    metadata_sequence: u64,
    issued_at: String,
    expires_at: String,
    #[serde(default)]
    release_notes: Option<String>,
    signature: WireSignature,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireTarget {
    os: String,
    architecture: String,
    minimum_platform: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WirePackage {
    source: String,
    size: u64,
    hashes: Vec<WireHash>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireHash {
    algorithm: String,
    value: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireDependency {
    product_id: String,
    version_constraint: String,
    package_hash: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSignature {
    key_id: String,
    algorithm: String,
    signature: String,
    signing_time: String,
    key_epoch: u64,
}

/// The target platform of a package.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Target {
    os: Platform,
    architecture: Architecture,
    minimum_platform: String,
}

impl Target {
    /// Returns the operating system.
    #[must_use]
    pub const fn os(&self) -> Platform {
        self.os
    }

    /// Returns the CPU architecture.
    #[must_use]
    pub const fn architecture(&self) -> Architecture {
        self.architecture
    }

    /// Returns the opaque minimum-platform policy string.
    #[must_use]
    pub fn minimum_platform(&self) -> &str {
        &self.minimum_platform
    }

    fn from_wire(wire: WireTarget) -> Result<Self, UpdaterError> {
        let os = wire.os.parse::<Platform>().map_err(map_core)?;
        let architecture = wire
            .architecture
            .parse::<Architecture>()
            .map_err(map_core)?;
        validate_minimum_platform(&wire.minimum_platform)?;
        Ok(Self {
            os,
            architecture,
            minimum_platform: wire.minimum_platform,
        })
    }
}

/// A validated SHA-256 package hash.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackageHash {
    value: Sha256Digest,
}

impl PackageHash {
    /// Returns the canonical algorithm name.
    #[must_use]
    pub const fn algorithm(self) -> &'static str {
        SHA256
    }

    /// Returns the digest.
    #[must_use]
    pub const fn value(self) -> Sha256Digest {
        self.value
    }
}

/// The package referenced by a manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Package {
    source: HttpsUrl,
    size: u64,
    hash: PackageHash,
}

impl Package {
    /// Returns the package source.
    #[must_use]
    pub fn source(&self) -> &HttpsUrl {
        &self.source
    }

    /// Returns the declared package size in bytes.
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Returns the single v1 package hash.
    #[must_use]
    pub const fn hash(&self) -> PackageHash {
        self.hash
    }

    /// Returns the SHA-256 digest.
    #[must_use]
    pub const fn sha256(&self) -> Sha256Digest {
        self.hash.value
    }

    /// Returns the hash list as a one-element slice.
    #[must_use]
    pub fn hashes(&self) -> &[PackageHash] {
        std::slice::from_ref(&self.hash)
    }

    fn from_wire(wire: WirePackage) -> Result<Self, UpdaterError> {
        let source = HttpsUrl::parse(&wire.source)?;
        if wire.size == 0 || wire.size > MAX_PACKAGE_BYTES {
            return Err(UpdaterError::InvalidPackageSize { size: wire.size });
        }
        if wire.hashes.len() != 1 {
            return Err(UpdaterError::InvalidHash {
                reason: "v1 requires exactly one package hash".to_owned(),
            });
        }
        let hash = wire
            .hashes
            .into_iter()
            .next()
            .ok_or_else(|| UpdaterError::InvalidHash {
                reason: "v1 requires exactly one package hash".to_owned(),
            })?;
        if hash.algorithm != SHA256 {
            return Err(UpdaterError::InvalidHash {
                reason: format!("unsupported algorithm: {}", hash.algorithm),
            });
        }
        let value =
            Sha256Digest::parse_hex(&hash.value).map_err(|error| UpdaterError::InvalidHash {
                reason: error.to_string(),
            })?;
        Ok(Self {
            source,
            size: wire.size,
            hash: PackageHash { value },
        })
    }
}

/// A dependency pinned to an exact package hash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dependency {
    product_id: ProductId,
    version_constraint: VersionReq,
    package_hash: Sha256Digest,
}

impl Dependency {
    /// Returns the dependency product identifier.
    #[must_use]
    pub fn product_id(&self) -> &ProductId {
        &self.product_id
    }

    /// Returns the version constraint.
    #[must_use]
    pub fn version_constraint(&self) -> &VersionReq {
        &self.version_constraint
    }

    /// Returns the exact package hash the dependency was built against.
    #[must_use]
    pub const fn package_hash(&self) -> Sha256Digest {
        self.package_hash
    }
}

/// A supported signature algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignatureAlgorithm {
    /// Ed25519.
    Ed25519,
}

impl SignatureAlgorithm {
    /// Returns the canonical algorithm name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        ED25519
    }
}

/// The manifest signature block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignatureBlock {
    key_id: KeyId,
    algorithm: SignatureAlgorithm,
    signature: SignatureBytes,
    signing_time: Timestamp,
    key_epoch: u64,
}

impl SignatureBlock {
    /// Returns the signing key identifier.
    #[must_use]
    pub fn key_id(&self) -> &KeyId {
        &self.key_id
    }

    /// Returns the signature algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> SignatureAlgorithm {
        self.algorithm
    }

    /// Returns the signature bytes.
    #[must_use]
    pub const fn signature(&self) -> &SignatureBytes {
        &self.signature
    }

    /// Returns the signing time.
    #[must_use]
    pub const fn signing_time(&self) -> Timestamp {
        self.signing_time
    }

    /// Returns the key epoch.
    #[must_use]
    pub const fn key_epoch(&self) -> u64 {
        self.key_epoch
    }

    fn from_wire(wire: WireSignature) -> Result<Self, UpdaterError> {
        let key_id =
            wire.key_id
                .parse::<KeyId>()
                .map_err(|error| UpdaterError::InvalidSignatureField {
                    reason: error.to_string(),
                })?;
        if wire.algorithm != ED25519 {
            return Err(UpdaterError::SignatureAlgorithmUnsupported {
                algorithm: wire.algorithm,
            });
        }
        let signature = SignatureBytes::parse_hex(&wire.signature).map_err(|error| {
            UpdaterError::InvalidSignatureField {
                reason: error.to_string(),
            }
        })?;
        let signing_time = Timestamp::parse(&wire.signing_time)?;
        if wire.key_epoch == 0 {
            return Err(UpdaterError::InvalidSignatureField {
                reason: "key_epoch must be at least 1".to_owned(),
            });
        }
        Ok(Self {
            key_id,
            algorithm: SignatureAlgorithm::Ed25519,
            signature,
            signing_time,
            key_epoch: wire.key_epoch,
        })
    }
}

/// A structurally validated Hub manifest.
///
/// Parsing proves the document is bounded, well-formed, and internally
/// consistent. It does **not** prove trust. Call [`Manifest::verify`] with a
/// trust store and a verification context before using any field to download,
/// install, or launch anything.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Manifest {
    schema_version: u64,
    product_id: ProductId,
    product_version: Version,
    channel: Channel,
    target: Target,
    package: Package,
    dependencies: Vec<Dependency>,
    metadata_sequence: u64,
    issued_at: Timestamp,
    expires_at: Timestamp,
    release_notes: Option<String>,
    signature: SignatureBlock,
}

impl Manifest {
    /// Parses a manifest without verifying its signature or trust.
    ///
    /// # Errors
    ///
    /// Returns a typed error for oversized, malformed, duplicate, unknown,
    /// inconsistent, or out-of-bound input.
    pub fn parse(bytes: &[u8]) -> Result<Self, UpdaterError> {
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err(UpdaterError::ManifestTooLarge {
                max: MAX_MANIFEST_BYTES,
                actual: bytes.len(),
            });
        }
        let text = std::str::from_utf8(bytes).map_err(|_| UpdaterError::InvalidUtf8)?;
        let mut deserializer = serde_json::Deserializer::from_str(text);
        let wire = WireManifest::deserialize(&mut deserializer).map_err(map_json_error)?;
        deserializer.end().map_err(|_| UpdaterError::TrailingData)?;
        Self::from_wire(wire)
    }

    /// Returns the schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u64 {
        self.schema_version
    }

    /// Returns the product identifier.
    #[must_use]
    pub fn product_id(&self) -> &ProductId {
        &self.product_id
    }

    /// Returns the product version.
    #[must_use]
    pub fn product_version(&self) -> &Version {
        &self.product_version
    }

    /// Returns the release channel.
    #[must_use]
    pub const fn channel(&self) -> Channel {
        self.channel
    }

    /// Returns the target.
    #[must_use]
    pub fn target(&self) -> &Target {
        &self.target
    }

    /// Returns the package.
    #[must_use]
    pub fn package(&self) -> &Package {
        &self.package
    }

    /// Returns the dependencies in sorted order.
    #[must_use]
    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }

    /// Returns the metadata sequence.
    #[must_use]
    pub const fn metadata_sequence(&self) -> u64 {
        self.metadata_sequence
    }

    /// Returns the issue time.
    #[must_use]
    pub const fn issued_at(&self) -> Timestamp {
        self.issued_at
    }

    /// Returns the expiry time.
    #[must_use]
    pub const fn expires_at(&self) -> Timestamp {
        self.expires_at
    }

    /// Returns the optional release notes.
    #[must_use]
    pub fn release_notes(&self) -> Option<&str> {
        self.release_notes.as_deref()
    }

    /// Returns the signature block.
    #[must_use]
    pub fn signature(&self) -> &SignatureBlock {
        &self.signature
    }

    /// Returns the exact canonical bytes covered by the signature.
    ///
    /// The signature block is excluded. Signing keys are never handled by this
    /// crate; release tooling signs these bytes with an approved offline key.
    #[must_use]
    pub fn canonical_payload(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(1_024);
        output.extend_from_slice(b"{\"schema_version\":");
        write_u64(&mut output, self.schema_version);
        output.extend_from_slice(b",\"product_id\":");
        write_string(&mut output, self.product_id.as_str());
        output.extend_from_slice(b",\"product_version\":");
        write_string(&mut output, &self.product_version.to_string());
        output.extend_from_slice(b",\"channel\":");
        write_string(&mut output, self.channel.as_str());
        output.extend_from_slice(b",\"target\":{\"os\":");
        write_string(&mut output, self.target.os.as_str());
        output.extend_from_slice(b",\"architecture\":");
        write_string(&mut output, self.target.architecture.as_str());
        output.extend_from_slice(b",\"minimum_platform\":");
        write_string(&mut output, &self.target.minimum_platform);
        output.extend_from_slice(b"},\"package\":{\"source\":");
        write_string(&mut output, self.package.source.as_str());
        output.extend_from_slice(b",\"size\":");
        write_u64(&mut output, self.package.size);
        output.extend_from_slice(b",\"hashes\":[{\"algorithm\":\"sha256\",\"value\":");
        write_string(&mut output, &self.package.hash.value.to_hex());
        output.extend_from_slice(b"}]},\"dependencies\":[");
        for (index, dependency) in self.dependencies.iter().enumerate() {
            if index > 0 {
                output.push(b',');
            }
            output.extend_from_slice(b"{\"product_id\":");
            write_string(&mut output, dependency.product_id.as_str());
            output.extend_from_slice(b",\"version_constraint\":");
            write_string(&mut output, &dependency.version_constraint.to_string());
            output.extend_from_slice(b",\"package_hash\":");
            write_string(&mut output, &dependency.package_hash.to_hex());
            output.push(b'}');
        }
        output.extend_from_slice(b"],\"metadata_sequence\":");
        write_u64(&mut output, self.metadata_sequence);
        output.extend_from_slice(b",\"issued_at\":");
        write_string(&mut output, &self.issued_at.to_string());
        output.extend_from_slice(b",\"expires_at\":");
        write_string(&mut output, &self.expires_at.to_string());
        if let Some(release_notes) = &self.release_notes {
            output.extend_from_slice(b",\"release_notes\":");
            write_string(&mut output, release_notes);
        }
        output.push(b'}');
        output
    }

    fn from_wire(wire: WireManifest) -> Result<Self, UpdaterError> {
        if wire.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(UpdaterError::UnsupportedSchemaVersion {
                found: wire.schema_version,
                supported: MANIFEST_SCHEMA_VERSION,
            });
        }
        let product_id = ProductId::new(wire.product_id).map_err(map_core)?;
        if wire.product_version.len() > MAX_VERSION_BYTES {
            return Err(UpdaterError::InvalidVersion {
                value: UpdaterError::bounded_detail(wire.product_version),
            });
        }
        let product_version =
            Version::parse(&wire.product_version).map_err(|_| UpdaterError::InvalidVersion {
                value: UpdaterError::bounded_detail(wire.product_version),
            })?;
        let channel = wire.channel.parse::<Channel>().map_err(map_core)?;
        let target = Target::from_wire(wire.target)?;
        let package = Package::from_wire(wire.package)?;
        let dependencies = parse_dependencies(wire.dependencies, &product_id)?;
        if wire.metadata_sequence == 0 {
            return Err(UpdaterError::InvalidSequence);
        }
        let issued_at = Timestamp::parse(&wire.issued_at)?;
        let expires_at = Timestamp::parse(&wire.expires_at)?;
        if issued_at >= expires_at {
            return Err(UpdaterError::InvalidValidityWindow);
        }
        if expires_at.unix_seconds() - issued_at.unix_seconds() > MAX_VALIDITY_SECONDS {
            return Err(UpdaterError::InvalidValidityWindow);
        }
        if let Some(release_notes) = &wire.release_notes {
            validate_release_notes(release_notes)?;
        }
        let signature = SignatureBlock::from_wire(wire.signature)?;
        let earliest = issued_at.unix_seconds().saturating_sub(CLOCK_SKEW_SECONDS);
        let latest = expires_at.unix_seconds().saturating_add(CLOCK_SKEW_SECONDS);
        if signature.signing_time.unix_seconds() < earliest
            || signature.signing_time.unix_seconds() > latest
        {
            return Err(UpdaterError::InvalidSigningTime);
        }

        Ok(Self {
            schema_version: wire.schema_version,
            product_id,
            product_version,
            channel,
            target,
            package,
            dependencies,
            metadata_sequence: wire.metadata_sequence,
            issued_at,
            expires_at,
            release_notes: wire.release_notes,
            signature,
        })
    }
}

fn parse_dependencies(
    wire: Vec<WireDependency>,
    product_id: &ProductId,
) -> Result<Vec<Dependency>, UpdaterError> {
    if wire.len() > MAX_DEPENDENCIES {
        return Err(UpdaterError::TooManyDependencies {
            max: MAX_DEPENDENCIES,
            actual: wire.len(),
        });
    }
    let mut dependencies = Vec::with_capacity(wire.len());
    for dependency in wire {
        if dependency.version_constraint.len() > MAX_CONSTRAINT_BYTES {
            return Err(UpdaterError::InvalidVersionConstraint {
                value: UpdaterError::bounded_detail(dependency.version_constraint),
            });
        }
        let dependency_product = ProductId::new(dependency.product_id).map_err(map_core)?;
        if dependency_product == *product_id {
            return Err(UpdaterError::InvalidDependency {
                reason: "a manifest must not depend on itself".to_owned(),
            });
        }
        let version_constraint =
            VersionReq::parse(&dependency.version_constraint).map_err(|_| {
                UpdaterError::InvalidVersionConstraint {
                    value: UpdaterError::bounded_detail(dependency.version_constraint),
                }
            })?;
        let package_hash = Sha256Digest::parse_hex(&dependency.package_hash).map_err(|error| {
            UpdaterError::InvalidDependency {
                reason: error.to_string(),
            }
        })?;
        dependencies.push(Dependency {
            product_id: dependency_product,
            version_constraint,
            package_hash,
        });
    }

    for pair in dependencies.windows(2) {
        if pair[0].product_id > pair[1].product_id {
            return Err(UpdaterError::InvalidDependency {
                reason: "dependencies must be sorted by product_id".to_owned(),
            });
        }
        if pair[0].product_id == pair[1].product_id {
            return Err(UpdaterError::InvalidDependency {
                reason: "duplicate dependency product_id".to_owned(),
            });
        }
    }

    Ok(dependencies)
}

fn validate_minimum_platform(value: &str) -> Result<(), UpdaterError> {
    if value.is_empty()
        || value.len() > MAX_MINIMUM_PLATFORM_BYTES
        || !value.is_ascii()
        || !value.as_bytes()[0].is_ascii_alphanumeric()
        || value
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')))
    {
        return Err(UpdaterError::InvalidMinimumPlatform);
    }
    Ok(())
}

fn validate_release_notes(value: &str) -> Result<(), UpdaterError> {
    if value.is_empty() || value.len() > MAX_RELEASE_NOTES_BYTES {
        return Err(UpdaterError::InvalidReleaseNotes {
            reason: "release notes must be 1-4096 bytes",
        });
    }
    for character in value.chars() {
        let code = character as u32;
        if code < 0x20 && !matches!(character, '\t' | '\n' | '\r') {
            return Err(UpdaterError::InvalidReleaseNotes {
                reason: "C0 control character",
            });
        }
        if (0x7f..=0x9f).contains(&code) {
            return Err(UpdaterError::InvalidReleaseNotes {
                reason: "C1 control character",
            });
        }
        if matches!(
            code,
            0x200e | 0x200f | 0x202a..=0x202e | 0x2066..=0x2069
        ) {
            return Err(UpdaterError::InvalidReleaseNotes {
                reason: "bidi control character",
            });
        }
    }
    Ok(())
}

fn map_core(error: CoreError) -> UpdaterError {
    match error {
        CoreError::InvalidProductId { reason } => UpdaterError::InvalidProductId {
            reason: reason.to_owned(),
        },
        CoreError::UnknownChannel { value } => UpdaterError::InvalidChannel { value },
        CoreError::UnknownPlatform { value } => UpdaterError::InvalidPlatform { value },
        CoreError::UnknownArchitecture { value } => UpdaterError::InvalidArchitecture { value },
        other => UpdaterError::MalformedJson {
            detail: UpdaterError::bounded_detail(other.to_string()),
        },
    }
}

fn map_json_error(error: serde_json::Error) -> UpdaterError {
    let detail = UpdaterError::bounded_detail(error.to_string());
    if detail.starts_with("duplicate field") || detail.starts_with("unknown field") {
        UpdaterError::UnknownOrDuplicateField { detail }
    } else {
        UpdaterError::MalformedJson { detail }
    }
}

fn write_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(value.to_string().as_bytes());
}

fn write_string(output: &mut Vec<u8>, value: &str) {
    output.push(b'"');
    for character in value.chars() {
        match character {
            '"' => output.extend_from_slice(b"\\\""),
            '\\' => output.extend_from_slice(b"\\\\"),
            '\u{0008}' => output.extend_from_slice(b"\\b"),
            '\t' => output.extend_from_slice(b"\\t"),
            '\n' => output.extend_from_slice(b"\\n"),
            '\u{000c}' => output.extend_from_slice(b"\\f"),
            '\r' => output.extend_from_slice(b"\\r"),
            character if (character as u32) < 0x20 => {
                output.extend_from_slice(format!("\\u{:04x}", character as u32).as_bytes());
            }
            character => {
                let mut buffer = [0u8; 4];
                output.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            }
        }
    }
    output.push(b'"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_canonical_strings() {
        let mut output = Vec::new();
        write_string(&mut output, "a\"b\\c\n\t\u{0001}é/");
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "\"a\\\"b\\\\c\\n\\t\\u0001é/\""
        );
    }
}

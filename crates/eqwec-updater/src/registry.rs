//! Authoritative installation registry v1.

use std::path::{Path, PathBuf};

use eqwec_core::{Channel, ProductId, ScopeId};
use eqwec_security::{KeyId, SafeRelativePath, Sha256Digest};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::atomic;
use crate::error::UpdaterError;
use crate::journal::OperationId;
use crate::layout::{InstallLayout, is_version_tree, validate_absolute};
use crate::time::Timestamp;

/// Registry schema version.
pub const REGISTRY_SCHEMA_VERSION: u64 = 1;

/// Maximum encoded registry size in bytes.
pub const MAX_REGISTRY_BYTES: usize = 262_144;

/// Maximum installation records in one registry.
pub const MAX_INSTALLATIONS: usize = 64;

/// Maximum retained versions per product.
pub const MAX_RETAINED_VERSIONS: usize = 3;

/// A retained, verified previous version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedVersion {
    product_version: Version,
    tree: SafeRelativePath,
    package_sha256: Sha256Digest,
}

impl RetainedVersion {
    /// Creates a retained-version record.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidRegistry`] when the tree is not a
    /// `versions/<tree-id>` path.
    pub fn new(
        product_version: Version,
        tree: SafeRelativePath,
        package_sha256: Sha256Digest,
    ) -> Result<Self, UpdaterError> {
        if !is_version_tree(&tree) {
            return Err(invalid_registry("retained tree must be versions/<tree-id>"));
        }
        Ok(Self {
            product_version,
            tree,
            package_sha256,
        })
    }

    /// Returns the retained version.
    #[must_use]
    pub fn product_version(&self) -> &Version {
        &self.product_version
    }

    /// Returns the retained tree path.
    #[must_use]
    pub fn tree(&self) -> &SafeRelativePath {
        &self.tree
    }

    /// Returns the retained package SHA-256.
    #[must_use]
    pub const fn package_sha256(&self) -> Sha256Digest {
        self.package_sha256
    }
}

/// One active installation record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallationRecord {
    product_id: ProductId,
    product_version: Version,
    channel: Channel,
    install_root: PathBuf,
    active_tree: SafeRelativePath,
    retained: Vec<RetainedVersion>,
    signer_key_id: KeyId,
    key_epoch: u64,
    package_sha256: Sha256Digest,
    metadata_sequence: u64,
    installed_at: Timestamp,
}

impl InstallationRecord {
    /// Creates an installation record.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the install root is not absolute and
    /// normalized, the active tree is not a version tree, the key epoch or
    /// metadata sequence is zero, or the retained list is invalid.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        product_id: ProductId,
        product_version: Version,
        channel: Channel,
        install_root: PathBuf,
        active_tree: SafeRelativePath,
        retained: Vec<RetainedVersion>,
        signer_key_id: KeyId,
        key_epoch: u64,
        package_sha256: Sha256Digest,
        metadata_sequence: u64,
        installed_at: Timestamp,
    ) -> Result<Self, UpdaterError> {
        validate_absolute(&install_root)?;
        if !is_version_tree(&active_tree) {
            return Err(invalid_registry(
                "active_tree must be a versions/<tree-id> path",
            ));
        }
        if key_epoch == 0 {
            return Err(invalid_registry("key_epoch must be at least 1"));
        }
        if metadata_sequence == 0 {
            return Err(invalid_registry("metadata_sequence must be at least 1"));
        }
        validate_retained(&retained, &active_tree)?;
        Ok(Self {
            product_id,
            product_version,
            channel,
            install_root,
            active_tree,
            retained,
            signer_key_id,
            key_epoch,
            package_sha256,
            metadata_sequence,
            installed_at,
        })
    }

    /// Returns the product identifier.
    #[must_use]
    pub fn product_id(&self) -> &ProductId {
        &self.product_id
    }

    /// Returns the active version.
    #[must_use]
    pub fn product_version(&self) -> &Version {
        &self.product_version
    }

    /// Returns the active channel.
    #[must_use]
    pub const fn channel(&self) -> Channel {
        self.channel
    }

    /// Returns the absolute installation root.
    #[must_use]
    pub fn install_root(&self) -> &Path {
        &self.install_root
    }

    /// Returns the active tree path.
    #[must_use]
    pub fn active_tree(&self) -> &SafeRelativePath {
        &self.active_tree
    }

    /// Returns the retained versions.
    #[must_use]
    pub fn retained(&self) -> &[RetainedVersion] {
        &self.retained
    }

    /// Returns the signer key identifier.
    #[must_use]
    pub fn signer_key_id(&self) -> &KeyId {
        &self.signer_key_id
    }

    /// Returns the signer key epoch.
    #[must_use]
    pub const fn key_epoch(&self) -> u64 {
        self.key_epoch
    }

    /// Returns the active package SHA-256.
    #[must_use]
    pub const fn package_sha256(&self) -> Sha256Digest {
        self.package_sha256
    }

    /// Returns the metadata sequence that produced the active tree.
    #[must_use]
    pub const fn metadata_sequence(&self) -> u64 {
        self.metadata_sequence
    }

    /// Returns the installation time.
    #[must_use]
    pub const fn installed_at(&self) -> Timestamp {
        self.installed_at
    }
}

/// The authoritative registry for one user profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Registry {
    profile_id: ScopeId,
    updated_at: Timestamp,
    installations: Vec<InstallationRecord>,
}

impl Registry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new(profile_id: ScopeId, updated_at: Timestamp) -> Self {
        Self {
            profile_id,
            updated_at,
            installations: Vec::new(),
        }
    }

    /// Parses a registry document.
    ///
    /// # Errors
    ///
    /// Returns a typed error for oversized, malformed, duplicate, unknown,
    /// unsorted, or out-of-bound input.
    pub fn parse(bytes: &[u8]) -> Result<Self, UpdaterError> {
        if bytes.len() > MAX_REGISTRY_BYTES {
            return Err(UpdaterError::RegistryTooLarge {
                max: MAX_REGISTRY_BYTES,
                actual: bytes.len(),
            });
        }
        let text = std::str::from_utf8(bytes).map_err(|_| UpdaterError::InvalidRegistry {
            reason: "registry is not valid UTF-8".to_owned(),
        })?;
        let mut deserializer = serde_json::Deserializer::from_str(text);
        let wire = WireRegistry::deserialize(&mut deserializer).map_err(UpdaterError::from_json)?;
        deserializer
            .end()
            .map_err(|_| UpdaterError::InvalidRegistry {
                reason: "trailing data after the registry".to_owned(),
            })?;
        Self::from_wire(wire)
    }

    /// Loads a registry, returning `None` when the file does not exist.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an oversized, unreadable, or invalid file.
    pub fn load(path: &Path) -> Result<Option<Self>, UpdaterError> {
        let Some(length) = atomic::metadata_len(path)? else {
            return Ok(None);
        };
        let length = usize::try_from(length).unwrap_or(usize::MAX);
        if length > MAX_REGISTRY_BYTES {
            return Err(UpdaterError::RegistryTooLarge {
                max: MAX_REGISTRY_BYTES,
                actual: length,
            });
        }
        let bytes = atomic::read_all(path)?;
        Self::parse(&bytes).map(Some)
    }

    /// Returns the profile identifier.
    #[must_use]
    pub fn profile_id(&self) -> &ScopeId {
        &self.profile_id
    }

    /// Returns the last update time.
    #[must_use]
    pub const fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    /// Returns the installation records in product-id order.
    #[must_use]
    pub fn installations(&self) -> &[InstallationRecord] {
        &self.installations
    }

    /// Returns the record for a product.
    #[must_use]
    pub fn get(&self, product_id: &ProductId) -> Option<&InstallationRecord> {
        self.installations
            .binary_search_by(|record| record.product_id.cmp(product_id))
            .ok()
            .map(|index| &self.installations[index])
    }

    /// Inserts or replaces a record and restores product-id order.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::TooManyInstallations`] when the registry would
    /// exceed its record bound.
    pub fn upsert(&mut self, record: InstallationRecord) -> Result<(), UpdaterError> {
        match self
            .installations
            .binary_search_by(|existing| existing.product_id.cmp(&record.product_id))
        {
            Ok(index) => self.installations[index] = record,
            Err(index) => {
                if self.installations.len() >= MAX_INSTALLATIONS {
                    return Err(UpdaterError::TooManyInstallations {
                        max: MAX_INSTALLATIONS,
                        actual: self.installations.len() + 1,
                    });
                }
                self.installations.insert(index, record);
            }
        }
        Ok(())
    }

    /// Removes a record.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidRegistry`] when the registry is in an
    /// inconsistent order.
    pub fn remove(&mut self, product_id: &ProductId) -> Result<bool, UpdaterError> {
        match self
            .installations
            .binary_search_by(|record| record.product_id.cmp(product_id))
        {
            Ok(index) => {
                self.installations.remove(index);
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    /// Sets the update time.
    pub fn set_updated_at(&mut self, updated_at: Timestamp) {
        self.updated_at = updated_at;
    }

    /// Serializes the registry as compact JSON.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::RegistryTooLarge`] when the encoded document
    /// exceeds the bound.
    pub fn to_bytes(&self) -> Result<Vec<u8>, UpdaterError> {
        let wire = WireRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION,
            profile_id: self.profile_id.as_str().to_owned(),
            updated_at: self.updated_at.to_string(),
            installations: self
                .installations
                .iter()
                .map(installation_to_wire)
                .collect(),
        };
        let bytes = serde_json::to_vec(&wire).map_err(UpdaterError::from_json)?;
        if bytes.len() > MAX_REGISTRY_BYTES {
            return Err(UpdaterError::RegistryTooLarge {
                max: MAX_REGISTRY_BYTES,
                actual: bytes.len(),
            });
        }
        Ok(bytes)
    }

    /// Atomically replaces a registry file.
    ///
    /// # Errors
    ///
    /// Returns a typed error when serialization or the atomic write fails.
    pub fn write_atomic(
        &self,
        path: &Path,
        operation_id: &OperationId,
    ) -> Result<(), UpdaterError> {
        let bytes = self.to_bytes()?;
        atomic::write_atomic(path, &bytes, operation_id.as_str())
    }

    /// Verifies that every install root is inside the profile layout.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InstallRootOutsideBase`] for the first record
    /// outside the layout.
    pub fn validate_against_layout(&self, layout: &InstallLayout) -> Result<(), UpdaterError> {
        for record in &self.installations {
            layout.validate_install_root(&record.install_root)?;
        }
        Ok(())
    }

    fn from_wire(wire: WireRegistry) -> Result<Self, UpdaterError> {
        if wire.schema_version != REGISTRY_SCHEMA_VERSION {
            return Err(UpdaterError::UnsupportedSchemaVersion {
                found: wire.schema_version,
                supported: REGISTRY_SCHEMA_VERSION,
            });
        }
        let profile_id =
            wire.profile_id
                .parse::<ScopeId>()
                .map_err(|error| UpdaterError::InvalidRegistry {
                    reason: error.to_string(),
                })?;
        let updated_at = Timestamp::parse(&wire.updated_at)?;
        if wire.installations.len() > MAX_INSTALLATIONS {
            return Err(UpdaterError::TooManyInstallations {
                max: MAX_INSTALLATIONS,
                actual: wire.installations.len(),
            });
        }

        let mut installations = Vec::with_capacity(wire.installations.len());
        for installation in wire.installations {
            installations.push(parse_installation(installation)?);
        }
        for pair in installations.windows(2) {
            if pair[0].product_id == pair[1].product_id {
                return Err(UpdaterError::DuplicateInstallation {
                    product_id: pair[0].product_id.to_string(),
                });
            }
            if pair[0].product_id > pair[1].product_id {
                return Err(UpdaterError::RegistryOrder);
            }
        }

        Ok(Self {
            profile_id,
            updated_at,
            installations,
        })
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WireRegistry {
    schema_version: u64,
    profile_id: String,
    updated_at: String,
    installations: Vec<WireInstallation>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WireInstallation {
    product_id: String,
    product_version: String,
    channel: String,
    install_root: String,
    active_tree: String,
    retained: Vec<WireRetained>,
    signer_key_id: String,
    key_epoch: u64,
    package_sha256: String,
    metadata_sequence: u64,
    installed_at: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WireRetained {
    product_version: String,
    tree: String,
    package_sha256: String,
}

fn installation_to_wire(record: &InstallationRecord) -> WireInstallation {
    WireInstallation {
        product_id: record.product_id.as_str().to_owned(),
        product_version: record.product_version.to_string(),
        channel: record.channel.as_str().to_owned(),
        install_root: record.install_root.to_string_lossy().into_owned(),
        active_tree: record.active_tree.as_str().to_owned(),
        retained: record
            .retained
            .iter()
            .map(|retained| WireRetained {
                product_version: retained.product_version.to_string(),
                tree: retained.tree.as_str().to_owned(),
                package_sha256: retained.package_sha256.to_hex(),
            })
            .collect(),
        signer_key_id: record.signer_key_id.as_str().to_owned(),
        key_epoch: record.key_epoch,
        package_sha256: record.package_sha256.to_hex(),
        metadata_sequence: record.metadata_sequence,
        installed_at: record.installed_at.to_string(),
    }
}

fn parse_installation(wire: WireInstallation) -> Result<InstallationRecord, UpdaterError> {
    let product_id =
        wire.product_id
            .parse::<ProductId>()
            .map_err(|error| UpdaterError::InvalidRegistry {
                reason: error.to_string(),
            })?;
    let product_version =
        Version::parse(&wire.product_version).map_err(|_| UpdaterError::InvalidVersion {
            value: UpdaterError::bounded_detail(wire.product_version),
        })?;
    let channel =
        wire.channel
            .parse::<Channel>()
            .map_err(|error| UpdaterError::InvalidChannel {
                value: error.to_string(),
            })?;
    let install_root = PathBuf::from(&wire.install_root);
    validate_absolute(&install_root)?;
    let active_tree =
        SafeRelativePath::new(wire.active_tree).map_err(|error| UpdaterError::InvalidRegistry {
            reason: error.to_string(),
        })?;
    if !is_version_tree(&active_tree) {
        return Err(invalid_registry(
            "active_tree must be a versions/<tree-id> path",
        ));
    }
    let mut retained = Vec::with_capacity(wire.retained.len());
    for entry in wire.retained {
        let version =
            Version::parse(&entry.product_version).map_err(|_| UpdaterError::InvalidVersion {
                value: UpdaterError::bounded_detail(entry.product_version),
            })?;
        let tree =
            SafeRelativePath::new(entry.tree).map_err(|error| UpdaterError::InvalidRegistry {
                reason: error.to_string(),
            })?;
        let package_sha256 = Sha256Digest::parse_hex(&entry.package_sha256).map_err(|error| {
            UpdaterError::InvalidRegistry {
                reason: error.to_string(),
            }
        })?;
        retained.push(RetainedVersion::new(version, tree, package_sha256)?);
    }
    let signer_key_id =
        wire.signer_key_id
            .parse::<KeyId>()
            .map_err(|error| UpdaterError::InvalidRegistry {
                reason: error.to_string(),
            })?;
    let package_sha256 = Sha256Digest::parse_hex(&wire.package_sha256).map_err(|error| {
        UpdaterError::InvalidRegistry {
            reason: error.to_string(),
        }
    })?;
    let installed_at = Timestamp::parse(&wire.installed_at)?;

    InstallationRecord::new(
        product_id,
        product_version,
        channel,
        install_root,
        active_tree,
        retained,
        signer_key_id,
        wire.key_epoch,
        package_sha256,
        wire.metadata_sequence,
        installed_at,
    )
}

fn validate_retained(
    retained: &[RetainedVersion],
    active_tree: &SafeRelativePath,
) -> Result<(), UpdaterError> {
    if retained.len() > MAX_RETAINED_VERSIONS {
        return Err(invalid_registry("too many retained versions"));
    }
    for (index, entry) in retained.iter().enumerate() {
        if entry.tree == *active_tree {
            return Err(invalid_registry(
                "retained tree must differ from the active tree",
            ));
        }
        if retained[..index]
            .iter()
            .any(|other| other.product_version == entry.product_version)
        {
            return Err(invalid_registry(
                "retained versions must be unique by product_version",
            ));
        }
    }
    Ok(())
}

fn invalid_registry(reason: &'static str) -> UpdaterError {
    UpdaterError::InvalidRegistry {
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "eqwec-registry-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn digest() -> Sha256Digest {
        Sha256Digest::parse_hex("53bd9dc21b205305c958c14f672d19214d5af9157ef66d9c9091b14837fe42a3")
            .unwrap()
    }

    fn tree(version: &str) -> SafeRelativePath {
        SafeRelativePath::new(format!("versions/{version}-53bd9dc21b20")).unwrap()
    }

    fn timestamp(value: &str) -> Timestamp {
        Timestamp::parse(value).unwrap()
    }

    fn record(product: &str, version: &str) -> InstallationRecord {
        InstallationRecord::new(
            ProductId::new(product).unwrap(),
            Version::parse(version).unwrap(),
            Channel::Stable,
            PathBuf::from(format!(
                "/home/user/.local/share/eqwec/installations/{product}"
            )),
            tree(version),
            Vec::new(),
            KeyId::new("release-2026-01").unwrap(),
            1,
            digest(),
            1,
            timestamp("2026-11-07T00:00:00Z"),
        )
        .unwrap()
    }

    #[test]
    fn round_trips_and_sorts_records() {
        let profile = ScopeId::new("default").unwrap();
        let mut registry = Registry::new(profile.clone(), timestamp("2026-11-07T00:00:00Z"));
        registry.upsert(record("eqwec-photo", "0.1.0")).unwrap();
        registry.upsert(record("eqwec-hub", "0.1.0")).unwrap();

        assert_eq!(
            registry.installations()[0].product_id().as_str(),
            "eqwec-hub"
        );
        let bytes = registry.to_bytes().unwrap();
        let parsed = Registry::parse(&bytes).unwrap();
        assert_eq!(parsed, registry);
        assert_eq!(parsed.profile_id(), &profile);
    }

    #[test]
    fn replaces_an_existing_record() {
        let mut registry = Registry::new(
            ScopeId::new("default").unwrap(),
            timestamp("2026-11-07T00:00:00Z"),
        );
        registry.upsert(record("eqwec-hub", "0.1.0")).unwrap();
        registry.upsert(record("eqwec-hub", "0.2.0")).unwrap();
        assert_eq!(registry.installations().len(), 1);
        assert_eq!(
            registry.installations()[0].product_version().to_string(),
            "0.2.0"
        );
        assert!(
            registry
                .remove(&ProductId::new("eqwec-hub").unwrap())
                .unwrap()
        );
        assert!(registry.installations().is_empty());
    }

    #[test]
    fn rejects_unsorted_and_duplicate_records() {
        let profile = ScopeId::new("default").unwrap();
        let updated = "2026-11-07T00:00:00Z";
        let installation = |product: &str| WireInstallation {
            product_id: product.to_owned(),
            product_version: "0.1.0".to_owned(),
            channel: "stable".to_owned(),
            install_root: format!("/home/user/.local/share/eqwec/installations/{product}"),
            active_tree: "versions/0.1.0-53bd9dc21b20".to_owned(),
            retained: Vec::new(),
            signer_key_id: "release-2026-01".to_owned(),
            key_epoch: 1,
            package_sha256: digest().to_hex(),
            metadata_sequence: 1,
            installed_at: updated.to_owned(),
        };
        let make = |installations| {
            serde_json::to_vec(&WireRegistry {
                schema_version: 1,
                profile_id: profile.to_string(),
                updated_at: updated.to_owned(),
                installations,
            })
            .unwrap()
        };

        let unsorted = make(vec![installation("eqwec-photo"), installation("eqwec-hub")]);
        assert_eq!(
            Registry::parse(&unsorted).unwrap_err(),
            UpdaterError::RegistryOrder
        );

        let duplicate = make(vec![installation("eqwec-hub"), installation("eqwec-hub")]);
        assert!(matches!(
            Registry::parse(&duplicate).unwrap_err(),
            UpdaterError::DuplicateInstallation { .. }
        ));
    }

    #[test]
    fn rejects_invalid_roots_trees_and_retained_lists() {
        let base = |root: &str, active: &str| WireInstallation {
            product_id: "eqwec-hub".to_owned(),
            product_version: "0.1.0".to_owned(),
            channel: "stable".to_owned(),
            install_root: root.to_owned(),
            active_tree: active.to_owned(),
            retained: Vec::new(),
            signer_key_id: "release-2026-01".to_owned(),
            key_epoch: 1,
            package_sha256: digest().to_hex(),
            metadata_sequence: 1,
            installed_at: "2026-11-07T00:00:00Z".to_owned(),
        };

        let relative = base("relative/root", "versions/0.1.0-53bd9dc21b20");
        assert!(matches!(
            parse_installation(relative).unwrap_err(),
            UpdaterError::InvalidInstallRoot { .. }
        ));

        let traversal = base("/home/user/../etc", "versions/0.1.0-53bd9dc21b20");
        assert!(matches!(
            parse_installation(traversal).unwrap_err(),
            UpdaterError::InvalidInstallRoot { .. }
        ));

        let bad_tree = base(
            "/home/user/.local/share/eqwec/installations/eqwec-hub",
            "../escape",
        );
        assert!(matches!(
            parse_installation(bad_tree).unwrap_err(),
            UpdaterError::InvalidRegistry { .. }
        ));
    }

    #[test]
    fn rejects_oversized_registry() {
        let bytes = vec![b' '; MAX_REGISTRY_BYTES + 1];
        assert!(matches!(
            Registry::parse(&bytes).unwrap_err(),
            UpdaterError::RegistryTooLarge { .. }
        ));
    }

    #[test]
    fn writes_atomically_and_loads() {
        let dir = TestDir::new();
        let path = dir.path().join("registry.json");
        let mut registry = Registry::new(
            ScopeId::new("default").unwrap(),
            timestamp("2026-11-07T00:00:00Z"),
        );
        registry.upsert(record("eqwec-hub", "0.1.0")).unwrap();

        let operation = OperationId::parse("0123456789abcdef0123456789abcdef").unwrap();
        registry.write_atomic(&path, &operation).unwrap();
        assert_eq!(Registry::load(&path).unwrap().unwrap(), registry);

        let layout = InstallLayout::new("/home/user/.local/share/eqwec").unwrap();
        registry.validate_against_layout(&layout).unwrap();
    }
}

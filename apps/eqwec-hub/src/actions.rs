//! Install, update, rollback, uninstall, and launch orchestration.
//!
//! Every mutation reloads the registry, recovers an interrupted transaction,
//! verifies the signed manifest, stages the package, health-checks the staged
//! tree, and only then activates it. The caller owns the profile lock.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use eqwec_core::{Architecture, Channel, Platform, ProductId, ScopeId};
use eqwec_security::Sha256Digest;
use eqwec_updater::{
    ActivePointer, ExtractionLimits, IdempotencyKey, InstallLayout, MAX_MANIFEST_BYTES,
    MAX_PACKAGE_BYTES, NoFaults, OperationId, OperationSpec, Phase, Registry, RollbackSpec,
    Timestamp, TrustStore, VerificationContext, VerifiedManifest, VerifiedRelease, activate,
    ensure_free_space, parse_and_verify, record_phase, recover, required_staging_bytes,
    rollback as rollback_release, stage_package_from_path, uninstall,
};

use crate::error::LauncherError;
use crate::launch;
use crate::trust;

/// The two files a release bundle must contain.
pub const BUNDLE_MANIFEST: &str = "manifest.json";
/// The package archive name inside a release bundle.
pub const BUNDLE_PACKAGE: &str = "package.zip";

const SCOPE_ID: &str = "default";

/// A verified local release bundle.
#[derive(Clone, Debug)]
pub struct Bundle {
    manifest: VerifiedManifest,
    package_path: PathBuf,
    package_size: u64,
}

impl Bundle {
    /// Returns the verified manifest.
    #[must_use]
    pub fn manifest(&self) -> &VerifiedManifest {
        &self.manifest
    }

    /// Returns the product identifier.
    #[must_use]
    pub fn product_id(&self) -> &ProductId {
        self.manifest.manifest().product_id()
    }

    /// Returns the product version.
    #[must_use]
    pub fn product_version(&self) -> &semver::Version {
        self.manifest.manifest().product_version()
    }

    /// Returns the release channel.
    #[must_use]
    pub fn channel(&self) -> Channel {
        self.manifest.manifest().channel()
    }

    /// Returns the declared package size.
    #[must_use]
    pub const fn package_size(&self) -> u64 {
        self.package_size
    }
}

/// Validates a bundle directory and verifies its manifest.
///
/// The directory must be absolute, must contain exactly `manifest.json` and
/// `package.zip`, and neither file may be a symlink.
///
/// # Errors
///
/// Returns [`LauncherError::Bundle`] for a malformed layout or a manifest that
/// does not verify against `trust_store` and `context`.
pub fn validate_bundle(
    bundle_dir: &Path,
    trust_store: &TrustStore,
    context: &VerificationContext,
) -> Result<Bundle, LauncherError> {
    if !bundle_dir.is_absolute() {
        return Err(bundle_error("bundle path must be absolute"));
    }
    let metadata = fs::symlink_metadata(bundle_dir).map_err(|error| LauncherError::Bundle {
        detail: format!("inspect bundle {}: {error}", bundle_dir.display()),
    })?;
    if metadata.file_type().is_symlink() {
        return Err(bundle_error("bundle path must not be a symlink"));
    }
    if !metadata.is_dir() {
        return Err(bundle_error("bundle path must be a directory"));
    }

    let mut manifest_seen = false;
    let mut package_seen = false;
    let entries = fs::read_dir(bundle_dir).map_err(|error| LauncherError::Bundle {
        detail: format!("read bundle {}: {error}", bundle_dir.display()),
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| LauncherError::Bundle {
            detail: format!("read bundle entry: {error}"),
        })?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| bundle_error("bundle contains a non-UTF-8 name"))?;
        match name.as_str() {
            BUNDLE_MANIFEST if !manifest_seen => manifest_seen = true,
            BUNDLE_PACKAGE if !package_seen => package_seen = true,
            BUNDLE_MANIFEST | BUNDLE_PACKAGE => {
                return Err(bundle_error("bundle contains duplicate entries"));
            }
            _ => return Err(bundle_error("bundle contains an unexpected entry")),
        }
    }
    if !manifest_seen || !package_seen {
        return Err(bundle_error(
            "bundle must contain manifest.json and package.zip",
        ));
    }

    let manifest_path = bundle_dir.join(BUNDLE_MANIFEST);
    let manifest_metadata =
        fs::symlink_metadata(&manifest_path).map_err(|error| LauncherError::Bundle {
            detail: format!("inspect manifest.json: {error}"),
        })?;
    require_regular_file(&manifest_metadata, "manifest.json")?;
    if manifest_metadata.len() > MAX_MANIFEST_BYTES as u64 {
        return Err(bundle_error("manifest.json exceeds the maximum size"));
    }
    let manifest_bytes = fs::read(&manifest_path).map_err(|error| LauncherError::Bundle {
        detail: format!("read manifest.json: {error}"),
    })?;
    if manifest_bytes.len() > MAX_MANIFEST_BYTES {
        return Err(bundle_error("manifest.json exceeds the maximum size"));
    }
    let manifest = parse_and_verify(&manifest_bytes, trust_store, context)?;

    let package_path = bundle_dir.join(BUNDLE_PACKAGE);
    let package_metadata =
        fs::symlink_metadata(&package_path).map_err(|error| LauncherError::Bundle {
            detail: format!("inspect package.zip: {error}"),
        })?;
    require_regular_file(&package_metadata, "package.zip")?;
    let package_size = package_metadata.len();
    if package_size == 0 || package_size > MAX_PACKAGE_BYTES {
        return Err(bundle_error(
            "package.zip size is outside the allowed range",
        ));
    }
    if package_size != manifest.manifest().package().size() {
        return Err(bundle_error(
            "package.zip size does not match the signed manifest",
        ));
    }

    Ok(Bundle {
        manifest,
        package_path,
        package_size,
    })
}

fn require_regular_file(metadata: &fs::Metadata, label: &str) -> Result<(), LauncherError> {
    if metadata.file_type().is_symlink() {
        return Err(bundle_error(&format!("{label} must not be a symlink")));
    }
    if !metadata.is_file() {
        return Err(bundle_error(&format!("{label} must be a regular file")));
    }
    Ok(())
}

fn bundle_error(detail: &str) -> LauncherError {
    LauncherError::Bundle {
        detail: detail.to_owned(),
    }
}

/// Installs a new product or updates an installed one.
///
/// A bundle for an installed product must carry a strictly newer version and
/// a metadata sequence that does not regress.
///
/// # Errors
///
/// Returns a typed error when the bundle is invalid, disk space is
/// insufficient, staging or the health check fails, or activation cannot be
/// committed. A failed operation is discarded by recovery; the previously
/// active version stays intact.
pub fn install_bundle(data_dir: &Path, bundle_dir: &Path) -> Result<String, LauncherError> {
    let now = Timestamp::now_utc()?;
    let layout = InstallLayout::new(data_dir)?;
    layout.ensure_base_dirs()?;
    let mut registry = load_registry(&layout, now)?;
    recover(&layout, &mut registry, now)?;

    let store = trust::trust_store()?;
    let context = host_context(now, None);
    let bundle = validate_bundle(bundle_dir, &store, &context)?;
    let record = registry.get(bundle.product_id()).cloned();
    if let Some(record) = &record {
        if bundle.product_version() <= record.product_version() {
            return Err(LauncherError::Rejected(format!(
                "update {} must be newer than the installed {}",
                bundle.product_version(),
                record.product_version()
            )));
        }
        if bundle.manifest().manifest().metadata_sequence() < record.metadata_sequence() {
            return Err(LauncherError::Rejected(
                "bundle metadata sequence regresses the installed release".to_owned(),
            ));
        }
    }

    let operation_id = new_operation_id()?;
    let key = idempotency_key(&operation_id)?;
    let release = VerifiedRelease::from_verified_manifest(bundle.manifest())?;
    let from_version = record
        .as_ref()
        .map(|record| record.product_version().clone());
    let spec = OperationSpec::new(operation_id, key, release, from_version, now);

    let result = install_verified(&layout, &mut registry, &spec, &bundle, now);
    finish(layout, &mut registry, now, result, || {
        if record.is_some() {
            format!(
                "Updated {} to {}",
                bundle.product_id(),
                bundle.product_version()
            )
        } else {
            format!(
                "Installed {} {}",
                bundle.product_id(),
                bundle.product_version()
            )
        }
    })
}

fn install_verified(
    layout: &InstallLayout,
    registry: &mut Registry,
    spec: &OperationSpec,
    bundle: &Bundle,
    now: Timestamp,
) -> Result<(), LauncherError> {
    record_phase(layout, spec, Phase::Intent, now)?;
    let required =
        required_staging_bytes(bundle.package_size()).ok_or_else(|| LauncherError::Bundle {
            detail: "package size overflows the disk reservation".to_owned(),
        })?;
    ensure_free_space(layout.base(), required)?;

    let destination = layout.download_package_path(spec.operation_id());
    let parent = destination.parent().ok_or_else(|| LauncherError::Io {
        detail: "download path has no parent".to_owned(),
    })?;
    fs::create_dir_all(parent).map_err(|error| LauncherError::Io {
        detail: format!("create {}: {error}", parent.display()),
    })?;
    fs::copy(&bundle.package_path, &destination).map_err(|error| LauncherError::Io {
        detail: format!("copy package: {error}"),
    })?;
    let copied = fs::metadata(&destination).map_err(|error| LauncherError::Io {
        detail: format!("inspect copied package: {error}"),
    })?;
    if copied.len() != bundle.package_size {
        return Err(bundle_error("copied package size changed"));
    }
    record_phase(layout, spec, Phase::Downloaded, now)?;

    let staged = stage_package_from_path(
        layout,
        spec,
        bundle.manifest(),
        &ExtractionLimits::default(),
        now,
    )?;
    launch::health_check(staged.root(), bundle.product_id())?;
    activate(layout, registry, spec, &NoFaults)?;
    Ok(())
}

/// Rolls an installed product back to a retained version.
///
/// # Errors
///
/// Returns a typed error when the product is not installed, the version is not
/// retained, the retained tree fails its health check, or the rollback cannot
/// be committed.
pub fn rollback(
    data_dir: &Path,
    product_id: &str,
    target_version: &str,
) -> Result<String, LauncherError> {
    let now = Timestamp::now_utc()?;
    let layout = InstallLayout::new(data_dir)?;
    let mut registry = load_registry(&layout, now)?;
    recover(&layout, &mut registry, now)?;

    let product = parse_product(product_id)?;
    let version = semver::Version::parse(target_version)
        .map_err(|error| LauncherError::Rejected(format!("invalid version: {error}")))?;
    let record = registry
        .get(&product)
        .cloned()
        .ok_or_else(|| LauncherError::Rejected(format!("{product} is not installed")))?;
    let retained = record
        .retained()
        .iter()
        .find(|retained| retained.product_version() == &version)
        .cloned()
        .ok_or_else(|| {
            LauncherError::Rejected(format!("{version} is not a retained version of {product}"))
        })?;
    launch::health_check(
        &layout.tree_path(&product, retained.tree()),
        record.product_id(),
    )?;

    let operation_id = new_operation_id()?;
    let spec = RollbackSpec::new(
        operation_id.clone(),
        idempotency_key(&operation_id)?,
        product,
        version,
        now,
    );
    let result = rollback_release(&layout, &mut registry, &spec, &NoFaults)
        .map(|_| ())
        .map_err(LauncherError::from);
    finish(layout, &mut registry, now, result, || {
        format!("Rolled back {product_id} to {target_version}")
    })
}

/// Uninstalls a product and removes its registry record.
///
/// # Errors
///
/// Returns a typed error when the product is not installed or the journaled
/// transaction cannot complete.
pub fn uninstall_product(data_dir: &Path, product_id: &str) -> Result<String, LauncherError> {
    let now = Timestamp::now_utc()?;
    let layout = InstallLayout::new(data_dir)?;
    let mut registry = load_registry(&layout, now)?;
    recover(&layout, &mut registry, now)?;

    let product = parse_product(product_id)?;
    let operation_id = new_operation_id()?;
    let key = idempotency_key(&operation_id)?;
    let result = uninstall(&layout, &mut registry, &product, operation_id, key, now)
        .map_err(LauncherError::from);
    finish(layout, &mut registry, now, result, || {
        format!("Uninstalled {product_id}")
    })
}

/// Resolves the active entry point for an installed product.
///
/// # Errors
///
/// Returns a typed error when the registry, pointer, tree, or entry point is
/// missing or inconsistent.
pub fn resolve_installed_entry(
    data_dir: &Path,
    product_id: &str,
) -> Result<(PathBuf, PathBuf), LauncherError> {
    let layout = InstallLayout::new(data_dir)?;
    let registry = Registry::load(&layout.registry_path())?
        .ok_or_else(|| LauncherError::Rejected("the profile has no registry".to_owned()))?;
    registry.validate_against_layout(&layout)?;
    let product = parse_product(product_id)?;
    let record = registry
        .get(&product)
        .ok_or_else(|| LauncherError::Rejected(format!("{product} is not installed")))?;
    let pointer = ActivePointer::load(&layout.active_pointer_path(&product))?
        .ok_or_else(|| LauncherError::Pointer("active pointer is missing".to_owned()))?;
    let resolved = launch::resolve_entry(&layout, record, &pointer)?;
    Ok((resolved.entry, resolved.tree_root))
}

/// Launches an installed product after pointer-first validation.
///
/// # Errors
///
/// Returns a typed error when resolution fails or the process cannot spawn.
pub fn launch_product(data_dir: &Path, product_id: &str) -> Result<u32, LauncherError> {
    let (entry, tree_root) = resolve_installed_entry(data_dir, product_id)?;
    launch::spawn(&entry, &tree_root)
}

fn finish<F>(
    layout: InstallLayout,
    registry: &mut Registry,
    now: Timestamp,
    result: Result<(), LauncherError>,
    success: F,
) -> Result<String, LauncherError>
where
    F: FnOnce() -> String,
{
    match result {
        Ok(()) => Ok(success()),
        Err(error) => {
            let _ = recover(&layout, registry, now);
            Err(error)
        }
    }
}

fn load_registry(layout: &InstallLayout, now: Timestamp) -> Result<Registry, LauncherError> {
    match Registry::load(&layout.registry_path())? {
        Some(registry) => {
            registry.validate_against_layout(layout)?;
            Ok(registry)
        }
        None => Ok(Registry::new(
            ScopeId::new(SCOPE_ID).map_err(|error| LauncherError::Rejected(error.to_string()))?,
            now,
        )),
    }
}

fn parse_product(value: &str) -> Result<ProductId, LauncherError> {
    ProductId::new(value).map_err(|error| LauncherError::Rejected(error.to_string()))
}

fn host_context(now: Timestamp, minimum_sequence: Option<u64>) -> VerificationContext {
    let context =
        VerificationContext::new(Channel::Stable, host_platform(), host_architecture(), now);
    match minimum_sequence {
        Some(sequence) => context.with_minimum_sequence(sequence),
        None => context,
    }
}

fn host_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::MacOs
    } else if cfg!(target_os = "windows") {
        Platform::Windows
    } else {
        Platform::Linux
    }
}

fn host_architecture() -> Architecture {
    if cfg!(target_arch = "aarch64") {
        Architecture::Aarch64
    } else {
        Architecture::X86_64
    }
}

fn new_operation_id() -> Result<OperationId, LauncherError> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| LauncherError::Rejected(format!("system clock: {error}")))?
        .as_nanos();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let digest = Sha256Digest::hash(format!("{nanos}:{}:{counter}", std::process::id()).as_bytes());
    let hex = digest.to_hex();
    OperationId::parse(&hex[..32]).map_err(LauncherError::from)
}

fn idempotency_key(operation_id: &OperationId) -> Result<IdempotencyKey, LauncherError> {
    IdempotencyKey::parse(&format!("launcher:{operation_id}")).map_err(LauncherError::from)
}

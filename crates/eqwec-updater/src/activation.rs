//! Activation transaction and startup recovery.
//!
//! The engine assumes exclusive access to one profile layout; the Hub
//! application owns the profile lock. It implements the durable sequence in
//! the Hub schema:
//!
//! 1. append `activation-intent`;
//! 2. move the verified staging tree into `versions/<tree-id>`;
//! 3. replace `active.json` atomically;
//! 4. write the registry atomically;
//! 5. append `activated`, `registry-committed`, and the terminal phase.
//!
//! Every failure after step 1 leaves a journal record that [`recover`] can
//! resolve to a complete old or complete new state. The caller runs [`recover`]
//! until it returns [`RecoveryOutcome::NothingToRecover`] before starting a new
//! operation; the engine rejects a second in-flight operation. Download,
//! extraction, and package verification are owned by the caller and are not
//! performed here.
//!
//! A rollback has `from_version` greater than `to_version`. An install or
//! update has `from_version` less than `to_version`, or no `from_version`.
//! [`activate`] rejects a downgrade so the two cases stay unambiguous during
//! recovery.

use std::fs;
use std::path::Path;

use eqwec_core::{Channel, ProductId};
use eqwec_security::{KeyId, SafeRelativePath, Sha256Digest};
use semver::Version;

use crate::active::ActivePointer;
use crate::atomic;
use crate::error::UpdaterError;
use crate::journal::{
    IdempotencyKey, JournalContext, JournalRecord, OperationId, Phase, append_journal_record,
    read_journal,
};
use crate::layout::{InstallLayout, tree_id};
use crate::registry::{InstallationRecord, MAX_RETAINED_VERSIONS, Registry, RetainedVersion};
use crate::time::Timestamp;
use crate::verify::VerifiedManifest;

/// A release whose manifest and package were verified.
///
/// Construct it from [`VerifiedManifest`] so the identity always comes from a
/// signed manifest. The direct constructor exists for tests and for recovery
/// records that already carry a verified identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedRelease {
    product_id: ProductId,
    product_version: Version,
    channel: Channel,
    package_sha256: Sha256Digest,
    signer_key_id: KeyId,
    key_epoch: u64,
    metadata_sequence: u64,
}

impl VerifiedRelease {
    /// Creates a verified release from manifest fields.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidJournalRecord`] when the key epoch or
    /// metadata sequence is zero.
    pub fn new(
        product_id: ProductId,
        product_version: Version,
        channel: Channel,
        package_sha256: Sha256Digest,
        signer_key_id: KeyId,
        key_epoch: u64,
        metadata_sequence: u64,
    ) -> Result<Self, UpdaterError> {
        if key_epoch == 0 {
            return Err(UpdaterError::InvalidJournalRecord {
                reason: "key_epoch must be at least 1".to_owned(),
            });
        }
        if metadata_sequence == 0 {
            return Err(UpdaterError::InvalidJournalRecord {
                reason: "metadata_sequence must be at least 1".to_owned(),
            });
        }
        Ok(Self {
            product_id,
            product_version,
            channel,
            package_sha256,
            signer_key_id,
            key_epoch,
            metadata_sequence,
        })
    }

    /// Builds the release identity from a verified manifest.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the manifest identity is out of range.
    pub fn from_verified_manifest(manifest: &VerifiedManifest) -> Result<Self, UpdaterError> {
        let manifest = manifest.manifest();
        Self::new(
            manifest.product_id().clone(),
            manifest.product_version().clone(),
            manifest.channel(),
            manifest.package().sha256(),
            manifest.signature().key_id().clone(),
            manifest.signature().key_epoch(),
            manifest.metadata_sequence(),
        )
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

    /// Returns the package SHA-256.
    #[must_use]
    pub const fn package_sha256(&self) -> Sha256Digest {
        self.package_sha256
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

    /// Returns the manifest metadata sequence.
    #[must_use]
    pub const fn metadata_sequence(&self) -> u64 {
        self.metadata_sequence
    }
}

/// One activation transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationSpec {
    operation_id: OperationId,
    idempotency_key: IdempotencyKey,
    release: VerifiedRelease,
    from_version: Option<Version>,
    activated_at: Timestamp,
}

impl OperationSpec {
    /// Creates an operation specification.
    #[must_use]
    pub fn new(
        operation_id: OperationId,
        idempotency_key: IdempotencyKey,
        release: VerifiedRelease,
        from_version: Option<Version>,
        activated_at: Timestamp,
    ) -> Self {
        Self {
            operation_id,
            idempotency_key,
            release,
            from_version,
            activated_at,
        }
    }

    /// Returns the operation identifier.
    #[must_use]
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }

    /// Returns the idempotency key.
    #[must_use]
    pub fn idempotency_key(&self) -> &IdempotencyKey {
        &self.idempotency_key
    }

    /// Returns the verified release.
    #[must_use]
    pub fn release(&self) -> &VerifiedRelease {
        &self.release
    }

    /// Returns the version being replaced, when this is an update or rollback.
    #[must_use]
    pub fn from_version(&self) -> Option<&Version> {
        self.from_version.as_ref()
    }

    /// Returns the local activation time.
    #[must_use]
    pub const fn activated_at(&self) -> Timestamp {
        self.activated_at
    }

    fn context(&self) -> Result<JournalContext, UpdaterError> {
        JournalContext::new(
            self.from_version.clone(),
            Some(self.release.product_version.clone()),
            self.release.channel,
            self.release.package_sha256,
            None,
        )?
        .with_installation_identity(
            self.release.signer_key_id.clone(),
            self.release.key_epoch,
            self.release.metadata_sequence,
        )
    }
}

/// One rollback transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackSpec {
    operation_id: OperationId,
    idempotency_key: IdempotencyKey,
    product_id: ProductId,
    target_version: Version,
    activated_at: Timestamp,
}

impl RollbackSpec {
    /// Creates a rollback specification.
    #[must_use]
    pub fn new(
        operation_id: OperationId,
        idempotency_key: IdempotencyKey,
        product_id: ProductId,
        target_version: Version,
        activated_at: Timestamp,
    ) -> Self {
        Self {
            operation_id,
            idempotency_key,
            product_id,
            target_version,
            activated_at,
        }
    }

    /// Returns the operation identifier.
    #[must_use]
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }

    /// Returns the idempotency key.
    #[must_use]
    pub fn idempotency_key(&self) -> &IdempotencyKey {
        &self.idempotency_key
    }

    /// Returns the product identifier.
    #[must_use]
    pub fn product_id(&self) -> &ProductId {
        &self.product_id
    }

    /// Returns the retained version to activate.
    #[must_use]
    pub fn target_version(&self) -> &Version {
        &self.target_version
    }

    /// Returns the local activation time.
    #[must_use]
    pub const fn activated_at(&self) -> Timestamp {
        self.activated_at
    }
}

/// A point where a test can abort the transaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FaultPoint {
    /// The `activation-intent` record is durable; the tree has not moved.
    AfterActivationIntent,
    /// The staging tree was renamed into `versions/`; the pointer is old.
    AfterTreeMove,
    /// The new pointer and the `activated` record are durable.
    AfterPointerWrite,
    /// The registry was replaced; `registry-committed` is not recorded yet.
    AfterRegistryWrite,
    /// The `registry-committed` record is durable.
    AfterRegistryCommitted,
}

impl FaultPoint {
    /// Returns the canonical fault-point name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AfterActivationIntent => "after-activation-intent",
            Self::AfterTreeMove => "after-tree-move",
            Self::AfterPointerWrite => "after-pointer-write",
            Self::AfterRegistryWrite => "after-registry-write",
            Self::AfterRegistryCommitted => "after-registry-committed",
        }
    }
}

/// Aborts a transaction at a chosen point.
///
/// Production code passes [`NoFaults`]. Tests pass a type that returns
/// [`UpdaterError::FaultInjected`] at one point to simulate a crash.
pub trait FaultInjector {
    /// Returns an error to abort at `point`.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::FaultInjected`] when the test requests an
    /// abort.
    fn check(&self, point: FaultPoint) -> Result<(), UpdaterError>;
}

/// A fault injector that never aborts.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoFaults;

impl FaultInjector for NoFaults {
    fn check(&self, _point: FaultPoint) -> Result<(), UpdaterError> {
        Ok(())
    }
}

/// The result of a completed activation or rollback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivationOutcome {
    /// A new tree is active.
    Activated {
        /// The active tree path.
        tree: SafeRelativePath,
    },
    /// A retained tree is active again.
    RolledBack {
        /// The active tree path.
        tree: SafeRelativePath,
    },
}

/// The result of one recovery pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryOutcome {
    /// No unfinished operation exists.
    NothingToRecover,
    /// Pre-activation data was removed and the operation was marked failed.
    Discarded {
        /// The recovered operation.
        operation_id: OperationId,
    },
    /// The activation was completed from durable state.
    Committed {
        /// The recovered operation.
        operation_id: OperationId,
    },
    /// The previous tree is active again.
    RolledBack {
        /// The recovered operation.
        operation_id: OperationId,
    },
}

/// Records a pre-activation phase.
///
/// Phases must be recorded in order: `intent`, `downloaded`, `verified`,
/// `staged`. Activation phases are written by [`activate`] and [`rollback`].
///
/// # Errors
///
/// Returns [`UpdaterError::InvalidOperationTransition`] when the phase is not
/// a pre-activation phase or the previous phase does not match.
pub fn record_phase(
    layout: &InstallLayout,
    spec: &OperationSpec,
    phase: Phase,
    timestamp: Timestamp,
) -> Result<(), UpdaterError> {
    if phase == Phase::Intent {
        let contents = read_journal(&layout.journal_path())?;
        if let Some(unfinished) = contents.unfinished_operation() {
            return Err(UpdaterError::InvalidOperationTransition {
                expected: "no unfinished operation",
                found: unfinished.phase().as_str(),
            });
        }
    }
    let expected = match phase {
        Phase::Intent => None,
        Phase::Downloaded => Some(Phase::Intent),
        Phase::Verified => Some(Phase::Downloaded),
        Phase::Staged => Some(Phase::Verified),
        Phase::ActivationIntent
        | Phase::Activated
        | Phase::RegistryCommitted
        | Phase::Committed
        | Phase::Failed
        | Phase::RolledBack => {
            return Err(UpdaterError::InvalidOperationTransition {
                expected: "intent, downloaded, verified, or staged",
                found: phase.as_str(),
            });
        }
    };
    let last = last_phase(layout, &spec.operation_id)?;
    if last != expected {
        return Err(UpdaterError::InvalidOperationTransition {
            expected: expected.map_or("none", Phase::as_str),
            found: last.map_or("none", Phase::as_str),
        });
    }
    let context = spec.context()?;
    append_phase(layout, spec, phase, &context, timestamp)
}

/// Activates a verified staging tree.
///
/// The caller must have recorded `intent`, `downloaded`, `verified`, and
/// `staged` for this operation and must have verified the staging contents.
///
/// # Errors
///
/// Returns a typed error when the operation state, staging tree, registry
/// state, or any durable write is invalid. A failure after the
/// `activation-intent` record leaves the operation recoverable.
pub fn activate(
    layout: &InstallLayout,
    registry: &mut Registry,
    spec: &OperationSpec,
    faults: &dyn FaultInjector,
) -> Result<ActivationOutcome, UpdaterError> {
    let product = &spec.release.product_id;
    let contents = read_journal(&layout.journal_path())?;
    let last = contents
        .records()
        .iter()
        .rev()
        .find(|record| record.operation_id() == &spec.operation_id);
    match last.map(JournalRecord::phase) {
        Some(Phase::Staged) => {}
        found => {
            return Err(UpdaterError::InvalidOperationTransition {
                expected: Phase::Staged.as_str(),
                found: found.map_or("none", Phase::as_str),
            });
        }
    }
    if contents.has_unfinished_operation_other_than(&spec.operation_id) {
        return Err(UpdaterError::InvalidOperationTransition {
            expected: "the current operation is the only unfinished one",
            found: "another unfinished operation",
        });
    }
    let context = spec.context()?;
    match last.and_then(JournalRecord::context) {
        Some(found) if found == &context => {}
        _ => {
            return Err(UpdaterError::RecoveryConflict {
                detail: "staged record does not match the operation identity".to_owned(),
            });
        }
    }
    ensure_registry_matches_source(registry, spec)?;

    if let Some(from) = spec.from_version.as_ref()
        && from > &spec.release.product_version
    {
        return Err(UpdaterError::InvalidOperationTransition {
            expected: "install or update version order",
            found: "downgrade",
        });
    }

    let staging = layout.staging_dir(product, &spec.operation_id);
    require_real_directory(&staging)?;

    let tree = tree_id(&spec.release.product_version, spec.release.package_sha256)?;
    let version_dir = layout.tree_path(product, &tree);
    if version_dir.exists() {
        return Err(UpdaterError::TreeAlreadyExists {
            tree: tree.as_str().to_owned(),
        });
    }
    fs::create_dir_all(layout.versions_dir(product)).map_err(atomic::io_error)?;

    let context = spec.context()?;
    append_phase(
        layout,
        spec,
        Phase::ActivationIntent,
        &context,
        spec.activated_at,
    )?;
    faults.check(FaultPoint::AfterActivationIntent)?;

    fs::rename(&staging, &version_dir).map_err(atomic::io_error)?;
    atomic::sync_parent_dir(&version_dir)?;
    faults.check(FaultPoint::AfterTreeMove)?;

    let pointer = ActivePointer::new(tree.clone(), spec.release.package_sha256, spec.activated_at)?;
    pointer.write_atomic(&layout.active_pointer_path(product), &spec.operation_id)?;
    append_phase(layout, spec, Phase::Activated, &context, spec.activated_at)?;
    faults.check(FaultPoint::AfterPointerWrite)?;

    commit_registry(layout, registry, spec, &tree)?;
    faults.check(FaultPoint::AfterRegistryWrite)?;
    append_phase(
        layout,
        spec,
        Phase::RegistryCommitted,
        &context,
        spec.activated_at,
    )?;
    faults.check(FaultPoint::AfterRegistryCommitted)?;
    append_phase(layout, spec, Phase::Committed, &context, spec.activated_at)?;

    prune_best_effort(layout, registry, product, &spec.operation_id);
    Ok(ActivationOutcome::Activated { tree })
}

/// Rolls back to a retained version.
///
/// # Errors
///
/// Returns a typed error when the product is not installed, the version is not
/// retained, the retained tree is missing, or a durable write fails.
pub fn rollback(
    layout: &InstallLayout,
    registry: &mut Registry,
    spec: &RollbackSpec,
    faults: &dyn FaultInjector,
) -> Result<ActivationOutcome, UpdaterError> {
    let operation_id = spec.operation_id();
    let product_id = spec.product_id();
    let target_version = spec.target_version();
    let activated_at = spec.activated_at();
    if let Some(unfinished) = read_journal(&layout.journal_path())?.unfinished_operation() {
        return Err(UpdaterError::InvalidOperationTransition {
            expected: "no unfinished operation",
            found: unfinished.phase().as_str(),
        });
    }
    ensure_last_phase(layout, operation_id, None)?;
    let current = registry
        .get(product_id)
        .ok_or_else(|| UpdaterError::NoInstallation {
            product_id: product_id.to_string(),
        })?
        .clone();
    if current.product_version() <= target_version {
        return Err(UpdaterError::NoSuchRetainedVersion {
            product_id: product_id.to_string(),
            version: target_version.to_string(),
        });
    }
    let target = current
        .retained()
        .iter()
        .find(|retained| retained.product_version() == target_version)
        .ok_or_else(|| UpdaterError::NoSuchRetainedVersion {
            product_id: product_id.to_string(),
            version: target_version.to_string(),
        })?
        .clone();

    let release = VerifiedRelease::new(
        product_id.clone(),
        target_version.clone(),
        current.channel(),
        target.package_sha256(),
        target.signer_key_id().clone(),
        target.key_epoch(),
        target.metadata_sequence(),
    )?;
    let spec = OperationSpec::new(
        operation_id.clone(),
        spec.idempotency_key().clone(),
        release,
        Some(current.product_version().clone()),
        activated_at,
    );

    let tree = target.tree().clone();
    require_real_directory(&layout.tree_path(product_id, &tree))?;

    let context = spec.context()?;
    append_phase(
        layout,
        &spec,
        Phase::ActivationIntent,
        &context,
        activated_at,
    )?;
    faults.check(FaultPoint::AfterActivationIntent)?;

    let pointer = ActivePointer::new(tree.clone(), target.package_sha256(), activated_at)?;
    pointer.write_atomic(&layout.active_pointer_path(product_id), operation_id)?;
    append_phase(layout, &spec, Phase::Activated, &context, activated_at)?;
    faults.check(FaultPoint::AfterPointerWrite)?;

    commit_registry(layout, registry, &spec, &tree)?;
    faults.check(FaultPoint::AfterRegistryWrite)?;
    append_phase(
        layout,
        &spec,
        Phase::RegistryCommitted,
        &context,
        activated_at,
    )?;
    faults.check(FaultPoint::AfterRegistryCommitted)?;
    append_phase(layout, &spec, Phase::RolledBack, &context, activated_at)?;

    prune_best_effort(layout, registry, product_id, operation_id);
    Ok(ActivationOutcome::RolledBack { tree })
}

/// Resolves one unfinished operation after a crash.
///
/// The caller must pass the registry loaded from `registry.json` after
/// restarting; recovery fails closed rather than recreate a missing source
/// record. Recovery is idempotent: a second pass finds a terminal record and
/// returns [`RecoveryOutcome::NothingToRecover`]. It fails closed on
/// inconsistent durable state instead of guessing.
///
/// # Errors
///
/// Returns a typed error when the journal is corrupt, the recorded identity is
/// incomplete, or the filesystem contradicts the journal.
pub fn recover(
    layout: &InstallLayout,
    registry: &mut Registry,
    now: Timestamp,
) -> Result<RecoveryOutcome, UpdaterError> {
    let contents = read_journal(&layout.journal_path())?;
    let Some(record) = contents.unfinished_operation() else {
        return Ok(RecoveryOutcome::NothingToRecover);
    };
    let operation_id = record.operation_id().clone();
    let product = record.product_id().clone();
    let context = record
        .context()
        .cloned()
        .ok_or_else(|| UpdaterError::InvalidJournalRecord {
            reason: "unfinished operation has no context".to_owned(),
        })?;

    match record.phase() {
        Phase::Intent | Phase::Downloaded | Phase::Verified | Phase::Staged => {
            remove_dir_if_exists(&layout.downloads_dir(&operation_id))?;
            remove_dir_if_exists(&layout.staging_dir(&product, &operation_id))?;
            append_recovery_record(layout, record, Phase::Failed, Some("RECOVERY_DISCARD"), now)?;
            Ok(RecoveryOutcome::Discarded { operation_id })
        }
        Phase::ActivationIntent => {
            recover_activation_intent(layout, registry, record, &context, now)
        }
        Phase::Activated => recover_activated(layout, registry, record, &context, now),
        Phase::RegistryCommitted => {
            recover_registry_committed(layout, registry, record, &context, now)
        }
        Phase::Committed | Phase::Failed | Phase::RolledBack => {
            Ok(RecoveryOutcome::NothingToRecover)
        }
    }
}

fn recover_activation_intent(
    layout: &InstallLayout,
    registry: &mut Registry,
    record: &JournalRecord,
    context: &JournalContext,
    now: Timestamp,
) -> Result<RecoveryOutcome, UpdaterError> {
    let product = record.product_id().clone();
    let operation_id = record.operation_id().clone();
    let tree = expected_tree(context)?;
    let version_dir = layout.tree_path(&product, &tree);
    let pointer = ActivePointer::load(&layout.active_pointer_path(&product))?;
    let active_is_new = pointer.as_ref().is_some_and(|pointer| {
        pointer.tree() == &tree && pointer.package_sha256() == context.package_sha256()
    });

    if active_is_new {
        if !version_dir.is_dir() {
            return Err(UpdaterError::RecoveryConflict {
                detail: "active pointer references a missing tree".to_owned(),
            });
        }
        // Preserve the real activation time written with the pointer.
        let activated_at = pointer.as_ref().map_or(now, ActivePointer::activated_at);
        let spec = recovery_spec(record, context, activated_at)?;
        commit_registry(layout, registry, &spec, &tree)?;
        append_recovery_record(layout, record, Phase::RegistryCommitted, None, now)?;
        append_recovery_record(layout, record, terminal_phase(context), None, now)?;
        prune_best_effort(layout, registry, &product, &operation_id);
        return Ok(recovery_outcome(context, operation_id));
    }

    let existing = registry.get(&product).cloned();
    if let Some(pointer) = pointer.as_ref() {
        let matches_previous = existing.as_ref().is_some_and(|existing| {
            pointer.tree() == existing.active_tree()
                && pointer.package_sha256() == existing.package_sha256()
        });
        if !matches_previous {
            return Err(UpdaterError::RecoveryConflict {
                detail: "active pointer matches neither the new tree nor the registry".to_owned(),
            });
        }
    }

    // A rollback target tree already exists and is referenced by the retained
    // list. Only a tree created by this operation may be removed.
    let tree_was_retained = existing.as_ref().is_some_and(|existing| {
        existing
            .retained()
            .iter()
            .any(|retained| retained.tree() == &tree)
    });

    if let Some(existing) = existing.as_ref() {
        let previous_dir = layout.tree_path(&product, existing.active_tree());
        if !previous_dir.is_dir() {
            return Err(UpdaterError::RecoveryConflict {
                detail: "previous active tree is missing; refusing to roll back".to_owned(),
            });
        }
        if pointer.is_none() {
            let pointer = ActivePointer::new(
                existing.active_tree().clone(),
                existing.package_sha256(),
                now,
            )?;
            pointer.write_atomic(&layout.active_pointer_path(&product), &operation_id)?;
        }
    } else {
        remove_file_if_exists(&layout.active_pointer_path(&product))?;
    }
    if !tree_was_retained {
        remove_dir_if_exists(&version_dir)?;
    }
    append_recovery_record(layout, record, Phase::RolledBack, None, now)?;
    remove_dir_if_exists(&layout.downloads_dir(&operation_id))?;
    remove_dir_if_exists(&layout.staging_dir(&product, &operation_id))?;
    Ok(RecoveryOutcome::RolledBack { operation_id })
}

fn recover_activated(
    layout: &InstallLayout,
    registry: &mut Registry,
    record: &JournalRecord,
    context: &JournalContext,
    now: Timestamp,
) -> Result<RecoveryOutcome, UpdaterError> {
    let product = record.product_id().clone();
    let operation_id = record.operation_id().clone();
    let tree = expected_tree(context)?;
    let version_dir = layout.tree_path(&product, &tree);
    let pointer = ActivePointer::load(&layout.active_pointer_path(&product))?;
    let matches = pointer.as_ref().is_some_and(|pointer| {
        pointer.tree() == &tree && pointer.package_sha256() == context.package_sha256()
    });
    if !matches {
        return Err(UpdaterError::RecoveryConflict {
            detail: "activated record does not match active.json".to_owned(),
        });
    }
    if !version_dir.is_dir() {
        return Err(UpdaterError::RecoveryConflict {
            detail: "activated tree is missing".to_owned(),
        });
    }

    let activated_at = pointer.as_ref().map_or(now, ActivePointer::activated_at);
    let spec = recovery_spec(record, context, activated_at)?;
    commit_registry(layout, registry, &spec, &tree)?;
    append_recovery_record(layout, record, Phase::RegistryCommitted, None, now)?;
    append_recovery_record(layout, record, terminal_phase(context), None, now)?;
    prune_best_effort(layout, registry, &product, &operation_id);
    Ok(recovery_outcome(context, operation_id))
}

fn recover_registry_committed(
    layout: &InstallLayout,
    registry: &Registry,
    record: &JournalRecord,
    context: &JournalContext,
    now: Timestamp,
) -> Result<RecoveryOutcome, UpdaterError> {
    let product = record.product_id();
    let operation_id = record.operation_id().clone();
    let tree = expected_tree(context)?;
    let to_version = context
        .to_version()
        .ok_or_else(|| UpdaterError::InvalidJournalRecord {
            reason: "unfinished operation has no to_version".to_owned(),
        })?;

    let pointer = ActivePointer::load(&layout.active_pointer_path(product))?;
    let pointer_matches = pointer.as_ref().is_some_and(|pointer| {
        pointer.tree() == &tree && pointer.package_sha256() == context.package_sha256()
    });
    if !pointer_matches {
        return Err(UpdaterError::RecoveryConflict {
            detail: "registry-committed operation does not match active.json".to_owned(),
        });
    }
    let registry_matches = registry.get(product).is_some_and(|record| {
        record.active_tree() == &tree
            && record.product_version() == to_version
            && record.package_sha256() == context.package_sha256()
    });
    if !registry_matches {
        return Err(UpdaterError::RecoveryConflict {
            detail: "registry-committed operation does not match the registry".to_owned(),
        });
    }
    if !layout.tree_path(product, &tree).is_dir() {
        return Err(UpdaterError::RecoveryConflict {
            detail: "registry-committed tree is missing".to_owned(),
        });
    }

    append_recovery_record(layout, record, terminal_phase(context), None, now)?;
    prune_best_effort(layout, registry, product, &operation_id);
    Ok(recovery_outcome(context, operation_id))
}

fn terminal_phase(context: &JournalContext) -> Phase {
    match (context.from_version(), context.to_version()) {
        (Some(from), Some(to)) if from > to => Phase::RolledBack,
        _ => Phase::Committed,
    }
}

fn recovery_outcome(context: &JournalContext, operation_id: OperationId) -> RecoveryOutcome {
    match terminal_phase(context) {
        Phase::RolledBack => RecoveryOutcome::RolledBack { operation_id },
        _ => RecoveryOutcome::Committed { operation_id },
    }
}

fn expected_tree(context: &JournalContext) -> Result<SafeRelativePath, UpdaterError> {
    let to_version = context
        .to_version()
        .ok_or_else(|| UpdaterError::InvalidJournalRecord {
            reason: "unfinished operation has no to_version".to_owned(),
        })?;
    tree_id(to_version, context.package_sha256())
}

fn recovery_spec(
    record: &JournalRecord,
    context: &JournalContext,
    now: Timestamp,
) -> Result<OperationSpec, UpdaterError> {
    let to_version = context
        .to_version()
        .ok_or_else(|| UpdaterError::InvalidJournalRecord {
            reason: "unfinished operation has no to_version".to_owned(),
        })?;
    let (signer_key_id, key_epoch, metadata_sequence) = context.installation_identity()?;
    let release = VerifiedRelease::new(
        record.product_id().clone(),
        to_version.clone(),
        context.channel(),
        context.package_sha256(),
        signer_key_id.clone(),
        key_epoch,
        metadata_sequence,
    )?;
    Ok(OperationSpec::new(
        record.operation_id().clone(),
        record.idempotency_key().clone(),
        release,
        context.from_version().cloned(),
        now,
    ))
}

fn ensure_registry_matches_source(
    registry: &Registry,
    spec: &OperationSpec,
) -> Result<(), UpdaterError> {
    let product = &spec.release.product_id;
    match (spec.from_version.as_ref(), registry.get(product)) {
        (Some(from), Some(record)) => {
            if record.product_version() != from {
                return Err(UpdaterError::InstallationVersionMismatch {
                    expected: from.to_string(),
                    found: record.product_version().to_string(),
                });
            }
            Ok(())
        }
        (Some(_), None) => Err(UpdaterError::NoInstallation {
            product_id: product.to_string(),
        }),
        (None, Some(_)) => Err(UpdaterError::InstallationAlreadyExists {
            product_id: product.to_string(),
        }),
        (None, None) => Ok(()),
    }
}

fn ensure_last_phase(
    layout: &InstallLayout,
    operation_id: &OperationId,
    expected: Option<Phase>,
) -> Result<(), UpdaterError> {
    match (last_phase(layout, operation_id)?, expected) {
        (found, Some(expected)) if found == Some(expected) => Ok(()),
        (None, None) => Ok(()),
        (found, expected) => Err(UpdaterError::InvalidOperationTransition {
            expected: expected.map_or("none", Phase::as_str),
            found: found.map_or("none", Phase::as_str),
        }),
    }
}

fn last_phase(
    layout: &InstallLayout,
    operation_id: &OperationId,
) -> Result<Option<Phase>, UpdaterError> {
    let contents = read_journal(&layout.journal_path())?;
    Ok(contents
        .records()
        .iter()
        .rev()
        .find(|record| record.operation_id() == operation_id)
        .map(JournalRecord::phase))
}

fn append_phase(
    layout: &InstallLayout,
    spec: &OperationSpec,
    phase: Phase,
    context: &JournalContext,
    timestamp: Timestamp,
) -> Result<(), UpdaterError> {
    let journal_path = layout.journal_path();
    if let Some(parent) = journal_path.parent() {
        fs::create_dir_all(parent).map_err(atomic::io_error)?;
    }
    let record = JournalRecord::new(
        spec.operation_id.clone(),
        spec.idempotency_key.clone(),
        spec.release.product_id.clone(),
        phase,
        timestamp,
        Some(context.clone()),
    )?;
    append_journal_record(&journal_path, &record)
}

fn append_recovery_record(
    layout: &InstallLayout,
    original: &JournalRecord,
    phase: Phase,
    error_code: Option<&str>,
    now: Timestamp,
) -> Result<(), UpdaterError> {
    let context = original
        .context()
        .cloned()
        .ok_or_else(|| UpdaterError::InvalidJournalRecord {
            reason: "recovery record has no context".to_owned(),
        })?
        .with_error_code(error_code.map(ToOwned::to_owned))?;
    let journal_path = layout.journal_path();
    if let Some(parent) = journal_path.parent() {
        fs::create_dir_all(parent).map_err(atomic::io_error)?;
    }
    let record = JournalRecord::new(
        original.operation_id().clone(),
        original.idempotency_key().clone(),
        original.product_id().clone(),
        phase,
        now,
        Some(context),
    )?;
    append_journal_record(&journal_path, &record)
}

fn commit_registry(
    layout: &InstallLayout,
    registry: &mut Registry,
    spec: &OperationSpec,
    tree: &SafeRelativePath,
) -> Result<(), UpdaterError> {
    let product = &spec.release.product_id;
    let existing = registry.get(product).cloned();
    if let Some(record) = &existing
        && record.active_tree() == tree
        && record.package_sha256() == spec.release.package_sha256
        && record.product_version() == &spec.release.product_version
    {
        return Ok(());
    }
    if let Some(record) = &existing {
        match spec.from_version.as_ref() {
            Some(from) if record.product_version() == from => {}
            Some(from) => {
                return Err(UpdaterError::RecoveryConflict {
                    detail: format!(
                        "registry version {} does not match expected source {from}",
                        record.product_version()
                    ),
                });
            }
            None => {
                return Err(UpdaterError::InstallationAlreadyExists {
                    product_id: product.to_string(),
                });
            }
        }
    } else if spec.from_version.is_some() {
        return Err(UpdaterError::NoInstallation {
            product_id: product.to_string(),
        });
    }

    let retained = retained_after(existing.as_ref(), tree)?;
    let record = InstallationRecord::new(
        product.clone(),
        spec.release.product_version.clone(),
        spec.release.channel,
        layout.product_root(product),
        tree.clone(),
        retained,
        spec.release.signer_key_id.clone(),
        spec.release.key_epoch,
        spec.release.package_sha256,
        spec.release.metadata_sequence,
        spec.activated_at,
    )?;
    registry.upsert(record)?;
    registry.set_updated_at(spec.activated_at);
    registry.write_atomic(&layout.registry_path(), &spec.operation_id)
}

fn retained_after(
    existing: Option<&InstallationRecord>,
    new_tree: &SafeRelativePath,
) -> Result<Vec<RetainedVersion>, UpdaterError> {
    let mut retained = Vec::new();
    if let Some(existing) = existing {
        if existing.active_tree() != new_tree {
            retained.push(RetainedVersion::new(
                existing.product_version().clone(),
                existing.active_tree().clone(),
                existing.package_sha256(),
                existing.signer_key_id().clone(),
                existing.key_epoch(),
                existing.metadata_sequence(),
            )?);
        }
        for entry in existing.retained() {
            if entry.tree() == new_tree
                || retained
                    .iter()
                    .any(|kept: &RetainedVersion| kept.product_version() == entry.product_version())
            {
                continue;
            }
            retained.push(entry.clone());
        }
    }
    retained.truncate(MAX_RETAINED_VERSIONS);
    Ok(retained)
}

fn require_real_directory(path: &Path) -> Result<(), UpdaterError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => UpdaterError::StagingMissing {
            path: path.display().to_string(),
        },
        _ => atomic::io_error(error),
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(UpdaterError::InvalidStaging {
            reason: "path is not a real directory".to_owned(),
        });
    }
    Ok(())
}

fn remove_dir_if_exists(path: &Path) -> Result<(), UpdaterError> {
    match fs::remove_dir_all(path) {
        Ok(()) => atomic::sync_parent_dir(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(atomic::io_error(error)),
    }
}

fn remove_file_if_exists(path: &Path) -> Result<(), UpdaterError> {
    match fs::remove_file(path) {
        Ok(()) => atomic::sync_parent_dir(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(atomic::io_error(error)),
    }
}

fn prune_best_effort(
    layout: &InstallLayout,
    registry: &Registry,
    product: &ProductId,
    operation_id: &OperationId,
) {
    let _ = remove_dir_if_exists(&layout.downloads_dir(operation_id));
    let _ = remove_dir_if_exists(&layout.staging_dir(product, operation_id));
    let _ = prune_unreferenced_versions(layout, registry, product);
}

fn prune_unreferenced_versions(
    layout: &InstallLayout,
    registry: &Registry,
    product: &ProductId,
) -> Result<(), UpdaterError> {
    let versions = layout.versions_dir(product);
    let entries = match fs::read_dir(&versions) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(atomic::io_error(error)),
    };
    let record = registry.get(product);
    for entry in entries {
        let entry = entry.map_err(atomic::io_error)?;
        let Some(name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        let Ok(tree) = SafeRelativePath::new(format!("versions/{name}")) else {
            continue;
        };
        let referenced = record.is_some_and(|record| {
            record.active_tree() == &tree
                || record
                    .retained()
                    .iter()
                    .any(|retained| retained.tree() == &tree)
        });
        if !referenced {
            fs::remove_dir_all(entry.path()).map_err(atomic::io_error)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;

//! Activation, rollback, and recovery tests, including fault injection at
//! every durable phase.

use super::*;
use eqwec_core::ScopeId;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const OP1: &str = "0123456789abcdef0123456789abcdef";
const OP2: &str = "11111111111111111111111111111111";
const OP3: &str = "22222222222222222222222222222222";

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "eqwec-activation-{}-{}",
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

fn timestamp(value: &str) -> Timestamp {
    Timestamp::parse(value).unwrap()
}

fn now() -> Timestamp {
    timestamp("2026-11-07T00:00:00Z")
}

fn layout(dir: &TestDir) -> InstallLayout {
    let layout = InstallLayout::new(dir.path().join("profile")).unwrap();
    layout.ensure_base_dirs().unwrap();
    layout
}

fn registry() -> Registry {
    Registry::new(ScopeId::new("default").unwrap(), now())
}

fn product() -> ProductId {
    ProductId::new("eqwec-hub").unwrap()
}

fn digest(byte: u8) -> Sha256Digest {
    Sha256Digest::parse_hex(&format!("{byte:02x}").repeat(32)).unwrap()
}

fn key() -> KeyId {
    KeyId::new("release-2026-01").unwrap()
}

fn release(version: &str, sha_byte: u8) -> VerifiedRelease {
    VerifiedRelease::new(
        product(),
        Version::parse(version).unwrap(),
        Channel::Stable,
        digest(sha_byte),
        key(),
        1,
        1,
    )
    .unwrap()
}

fn operation(id: &str, version: &str, sha_byte: u8, from: Option<&str>) -> OperationSpec {
    OperationSpec::new(
        OperationId::parse(id).unwrap(),
        IdempotencyKey::parse(&format!("install:eqwec-hub:stable:{version}")).unwrap(),
        release(version, sha_byte),
        from.map(|value| Version::parse(value).unwrap()),
        now(),
    )
}

fn record_pre_activation(layout: &InstallLayout, spec: &OperationSpec) {
    for phase in [
        Phase::Intent,
        Phase::Downloaded,
        Phase::Verified,
        Phase::Staged,
    ] {
        record_phase(layout, spec, phase, spec.activated_at()).unwrap();
    }
}

fn rollback_spec(operation_id: &str, version: &str) -> RollbackSpec {
    RollbackSpec::new(
        OperationId::parse(operation_id).unwrap(),
        IdempotencyKey::parse(&format!("rollback:eqwec-hub:stable:{version}")).unwrap(),
        product(),
        Version::parse(version).unwrap(),
        now(),
    )
}

fn stage(layout: &InstallLayout, spec: &OperationSpec) -> PathBuf {
    let staging = layout.staging_dir(spec.release.product_id(), spec.operation_id());
    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join("payload"), b"verified tree").unwrap();
    staging
}

fn tree_for(spec: &OperationSpec) -> SafeRelativePath {
    tree_id(&spec.release.product_version, spec.release.package_sha256).unwrap()
}

fn last_journal_phase(layout: &InstallLayout) -> Phase {
    read_journal(&layout.journal_path())
        .unwrap()
        .records()
        .last()
        .unwrap()
        .phase()
}

struct FailAt(FaultPoint);

impl FaultInjector for FailAt {
    fn check(&self, point: FaultPoint) -> Result<(), UpdaterError> {
        if point == self.0 {
            Err(UpdaterError::FaultInjected {
                point: point.as_str(),
            })
        } else {
            Ok(())
        }
    }
}

#[test]
fn activates_a_fresh_install() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let spec = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &spec);
    let staging = stage(&layout, &spec);
    let tree = tree_for(&spec);

    let outcome = activate(&layout, &mut registry, &spec, &NoFaults).unwrap();
    assert_eq!(outcome, ActivationOutcome::Activated { tree: tree.clone() });

    let pointer = ActivePointer::load(&layout.active_pointer_path(&product()))
        .unwrap()
        .unwrap();
    assert_eq!(pointer.tree(), &tree);
    assert_eq!(pointer.package_sha256(), digest(0x11));

    let record = registry.get(&product()).unwrap();
    assert_eq!(record.product_version(), &Version::parse("0.1.0").unwrap());
    assert_eq!(record.active_tree(), &tree);
    assert_eq!(record.signer_key_id(), &key());
    assert_eq!(record.metadata_sequence(), 1);
    assert!(record.retained().is_empty());

    assert!(!staging.exists());
    assert!(layout.tree_path(&product(), &tree).is_dir());
    assert_eq!(last_journal_phase(&layout), Phase::Committed);

    let bytes = fs::read(layout.registry_path()).unwrap();
    assert_eq!(Registry::parse(&bytes).unwrap(), registry);
}

#[test]
fn rejects_out_of_order_phases() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let spec = operation(OP1, "0.1.0", 0x11, None);

    assert!(matches!(
        record_phase(&layout, &spec, Phase::Downloaded, now()).unwrap_err(),
        UpdaterError::InvalidOperationTransition { .. }
    ));
    record_phase(&layout, &spec, Phase::Intent, now()).unwrap();
    assert!(matches!(
        record_phase(&layout, &spec, Phase::Verified, now()).unwrap_err(),
        UpdaterError::InvalidOperationTransition { .. }
    ));
    assert!(matches!(
        record_phase(&layout, &spec, Phase::Committed, now()).unwrap_err(),
        UpdaterError::InvalidOperationTransition { .. }
    ));
}

#[test]
fn rejects_a_second_operation_while_one_is_in_flight() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let first = operation(OP1, "0.1.0", 0x11, None);
    record_phase(&layout, &first, Phase::Intent, now()).unwrap();

    let second = operation(OP2, "0.2.0", 0x22, None);
    assert!(matches!(
        record_phase(&layout, &second, Phase::Intent, now()).unwrap_err(),
        UpdaterError::InvalidOperationTransition { .. }
    ));

    let mut registry = registry();
    assert!(matches!(
        rollback(
            &layout,
            &mut registry,
            &rollback_spec(OP3, "0.0.1"),
            &NoFaults,
        )
        .unwrap_err(),
        UpdaterError::InvalidOperationTransition { .. }
    ));
}

#[test]
fn activate_rejects_another_unfinished_operation() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let stale = operation(OP1, "0.1.0", 0x11, None);
    record_phase(&layout, &stale, Phase::Intent, now()).unwrap();

    // Write the current operation directly so the check under test, not the
    // phase-order check, is the one that rejects activation.
    let spec = operation(OP2, "0.2.0", 0x22, None);
    for phase in [
        Phase::Intent,
        Phase::Downloaded,
        Phase::Verified,
        Phase::Staged,
    ] {
        let record = JournalRecord::new(
            spec.operation_id().clone(),
            spec.idempotency_key().clone(),
            product(),
            phase,
            now(),
            Some(spec.context().unwrap()),
        )
        .unwrap();
        append_journal_record(&layout.journal_path(), &record).unwrap();
    }
    stage(&layout, &spec);

    assert!(matches!(
        activate(&layout, &mut registry, &spec, &NoFaults).unwrap_err(),
        UpdaterError::InvalidOperationTransition { .. }
    ));
}

#[test]
fn activate_rejects_a_staged_record_with_a_different_identity() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let recorded = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &recorded);
    stage(&layout, &recorded);

    let spec = OperationSpec::new(
        OperationId::parse(OP1).unwrap(),
        IdempotencyKey::parse("install:eqwec-hub:stable:0.1.0").unwrap(),
        release("0.1.0", 0x22),
        None,
        now(),
    );
    assert!(matches!(
        activate(&layout, &mut registry, &spec, &NoFaults).unwrap_err(),
        UpdaterError::RecoveryConflict { .. }
    ));
}

#[test]
fn rejects_missing_and_symlinked_staging() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let spec = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &spec);

    assert!(matches!(
        activate(&layout, &mut registry, &spec, &NoFaults).unwrap_err(),
        UpdaterError::StagingMissing { .. }
    ));

    #[cfg(unix)]
    {
        let target = dir.path().join("elsewhere");
        fs::create_dir_all(&target).unwrap();
        let staging = layout.staging_dir(&product(), spec.operation_id());
        fs::create_dir_all(staging.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&target, &staging).unwrap();
        assert!(matches!(
            activate(&layout, &mut registry, &spec, &NoFaults).unwrap_err(),
            UpdaterError::InvalidStaging { .. }
        ));
    }
}

#[test]
fn rejects_existing_and_downgrade_trees() {
    let dir = TestDir::new();
    let first_layout = layout(&dir);
    let mut first_registry = registry();
    let spec = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&first_layout, &spec);
    stage(&first_layout, &spec);
    let tree = tree_for(&spec);
    fs::create_dir_all(first_layout.tree_path(&product(), &tree)).unwrap();
    assert!(matches!(
        activate(&first_layout, &mut first_registry, &spec, &NoFaults).unwrap_err(),
        UpdaterError::TreeAlreadyExists { .. }
    ));

    let dir = TestDir::new();
    let layout2 = layout(&dir);
    let mut registry2 = registry();
    let installed = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout2, &installed);
    stage(&layout2, &installed);
    activate(&layout2, &mut registry2, &installed, &NoFaults).unwrap();

    let downgrade = operation(OP2, "0.0.9", 0x22, Some("0.1.0"));
    record_pre_activation(&layout2, &downgrade);
    stage(&layout2, &downgrade);
    assert!(matches!(
        activate(&layout2, &mut registry2, &downgrade, &NoFaults).unwrap_err(),
        UpdaterError::InvalidOperationTransition { .. }
    ));
}

#[test]
fn updates_and_retains_the_previous_version() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();

    let first = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &first);
    stage(&layout, &first);
    activate(&layout, &mut registry, &first, &NoFaults).unwrap();

    let second = operation(OP2, "0.2.0", 0x22, Some("0.1.0"));
    record_pre_activation(&layout, &second);
    stage(&layout, &second);
    let outcome = activate(&layout, &mut registry, &second, &NoFaults).unwrap();
    let tree = tree_for(&second);
    assert_eq!(outcome, ActivationOutcome::Activated { tree: tree.clone() });

    let record = registry.get(&product()).unwrap();
    assert_eq!(record.product_version(), &Version::parse("0.2.0").unwrap());
    assert_eq!(record.retained().len(), 1);
    let retained = &record.retained()[0];
    assert_eq!(
        retained.product_version(),
        &Version::parse("0.1.0").unwrap()
    );
    assert_eq!(retained.tree(), &tree_for(&first));
    assert_eq!(retained.package_sha256(), digest(0x11));
    assert_eq!(retained.signer_key_id(), &key());
    assert_eq!(retained.metadata_sequence(), 1);
    assert!(layout.tree_path(&product(), &tree_for(&first)).is_dir());
}

#[test]
fn rolls_back_to_a_retained_version() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();

    let first = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &first);
    stage(&layout, &first);
    activate(&layout, &mut registry, &first, &NoFaults).unwrap();

    let second = operation(OP2, "0.2.0", 0x22, Some("0.1.0"));
    record_pre_activation(&layout, &second);
    stage(&layout, &second);
    activate(&layout, &mut registry, &second, &NoFaults).unwrap();

    let request = rollback_spec(OP3, "0.1.0");
    let outcome = rollback(&layout, &mut registry, &request, &NoFaults).unwrap();
    assert_eq!(
        outcome,
        ActivationOutcome::RolledBack {
            tree: tree_for(&first)
        }
    );

    let pointer = ActivePointer::load(&layout.active_pointer_path(&product()))
        .unwrap()
        .unwrap();
    assert_eq!(pointer.tree(), &tree_for(&first));
    let record = registry.get(&product()).unwrap();
    assert_eq!(record.product_version(), &Version::parse("0.1.0").unwrap());
    assert_eq!(record.retained().len(), 1);
    assert_eq!(
        record.retained()[0].product_version(),
        &Version::parse("0.2.0").unwrap()
    );
    assert_eq!(last_journal_phase(&layout), Phase::RolledBack);

    assert!(matches!(
        rollback(
            &layout,
            &mut registry,
            &rollback_spec("33333333333333333333333333333333", "0.0.1"),
            &NoFaults,
        )
        .unwrap_err(),
        UpdaterError::NoSuchRetainedVersion { .. }
    ));
}

#[test]
fn recovers_after_activation_intent_before_tree_move() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let spec = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &spec);
    stage(&layout, &spec);

    assert!(matches!(
        activate(
            &layout,
            &mut registry,
            &spec,
            &FailAt(FaultPoint::AfterActivationIntent)
        )
        .unwrap_err(),
        UpdaterError::FaultInjected { .. }
    ));

    let outcome = recover(&layout, &mut registry, now()).unwrap();
    assert_eq!(
        outcome,
        RecoveryOutcome::RolledBack {
            operation_id: OperationId::parse(OP1).unwrap()
        }
    );
    assert!(registry.get(&product()).is_none());
    assert!(
        ActivePointer::load(&layout.active_pointer_path(&product()))
            .unwrap()
            .is_none()
    );
    assert!(!layout.tree_path(&product(), &tree_for(&spec)).exists());
    assert!(!layout.staging_dir(&product(), spec.operation_id()).exists());
    assert_eq!(last_journal_phase(&layout), Phase::RolledBack);

    assert_eq!(
        recover(&layout, &mut registry, now()).unwrap(),
        RecoveryOutcome::NothingToRecover
    );
}

#[test]
fn recovers_after_tree_move_before_pointer() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let spec = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &spec);
    stage(&layout, &spec);

    assert!(
        activate(
            &layout,
            &mut registry,
            &spec,
            &FailAt(FaultPoint::AfterTreeMove)
        )
        .is_err()
    );
    assert!(layout.tree_path(&product(), &tree_for(&spec)).is_dir());

    assert!(matches!(
        recover(&layout, &mut registry, now()).unwrap(),
        RecoveryOutcome::RolledBack { .. }
    ));
    assert!(!layout.tree_path(&product(), &tree_for(&spec)).exists());
    assert!(registry.get(&product()).is_none());
}

#[test]
fn recovers_after_pointer_write() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let spec = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &spec);
    stage(&layout, &spec);

    assert!(
        activate(
            &layout,
            &mut registry,
            &spec,
            &FailAt(FaultPoint::AfterPointerWrite)
        )
        .is_err()
    );
    assert_eq!(last_journal_phase(&layout), Phase::Activated);
    assert!(registry.get(&product()).is_none());

    assert_eq!(
        recover(&layout, &mut registry, now()).unwrap(),
        RecoveryOutcome::Committed {
            operation_id: OperationId::parse(OP1).unwrap()
        }
    );
    let record = registry.get(&product()).unwrap();
    assert_eq!(record.active_tree(), &tree_for(&spec));
    assert_eq!(last_journal_phase(&layout), Phase::Committed);

    assert_eq!(
        recover(&layout, &mut registry, now()).unwrap(),
        RecoveryOutcome::NothingToRecover
    );
}

#[test]
fn recovers_after_registry_write_without_duplicates() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let spec = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &spec);
    stage(&layout, &spec);

    assert!(
        activate(
            &layout,
            &mut registry,
            &spec,
            &FailAt(FaultPoint::AfterRegistryWrite)
        )
        .is_err()
    );
    assert_eq!(last_journal_phase(&layout), Phase::Activated);
    assert_eq!(registry.installations().len(), 1);

    assert!(matches!(
        recover(&layout, &mut registry, now()).unwrap(),
        RecoveryOutcome::Committed { .. }
    ));
    assert_eq!(registry.installations().len(), 1);
    assert_eq!(last_journal_phase(&layout), Phase::Committed);
}

#[test]
fn recovers_after_registry_committed() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let spec = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &spec);
    stage(&layout, &spec);

    assert!(
        activate(
            &layout,
            &mut registry,
            &spec,
            &FailAt(FaultPoint::AfterRegistryCommitted)
        )
        .is_err()
    );
    assert_eq!(last_journal_phase(&layout), Phase::RegistryCommitted);

    assert!(matches!(
        recover(&layout, &mut registry, now()).unwrap(),
        RecoveryOutcome::Committed { .. }
    ));
    assert_eq!(last_journal_phase(&layout), Phase::Committed);
}

#[test]
fn recovers_an_update_before_and_after_the_pointer() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let first = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &first);
    stage(&layout, &first);
    activate(&layout, &mut registry, &first, &NoFaults).unwrap();

    let second = operation(OP2, "0.2.0", 0x22, Some("0.1.0"));
    record_pre_activation(&layout, &second);
    stage(&layout, &second);
    assert!(
        activate(
            &layout,
            &mut registry,
            &second,
            &FailAt(FaultPoint::AfterActivationIntent)
        )
        .is_err()
    );
    assert!(matches!(
        recover(&layout, &mut registry, now()).unwrap(),
        RecoveryOutcome::RolledBack { .. }
    ));
    let pointer = ActivePointer::load(&layout.active_pointer_path(&product()))
        .unwrap()
        .unwrap();
    assert_eq!(pointer.tree(), &tree_for(&first));
    assert_eq!(
        registry.get(&product()).unwrap().product_version(),
        &Version::parse("0.1.0").unwrap()
    );
    assert!(!layout.tree_path(&product(), &tree_for(&second)).exists());

    let third = operation(OP3, "0.3.0", 0x33, Some("0.1.0"));
    record_pre_activation(&layout, &third);
    stage(&layout, &third);
    assert!(
        activate(
            &layout,
            &mut registry,
            &third,
            &FailAt(FaultPoint::AfterPointerWrite)
        )
        .is_err()
    );
    assert!(matches!(
        recover(&layout, &mut registry, now()).unwrap(),
        RecoveryOutcome::Committed { .. }
    ));
    let pointer = ActivePointer::load(&layout.active_pointer_path(&product()))
        .unwrap()
        .unwrap();
    assert_eq!(pointer.tree(), &tree_for(&third));
    let record = registry.get(&product()).unwrap();
    assert_eq!(record.product_version(), &Version::parse("0.3.0").unwrap());
    assert_eq!(record.retained().len(), 1);
    assert_eq!(
        record.retained()[0].product_version(),
        &Version::parse("0.1.0").unwrap()
    );
}

#[test]
fn discards_pre_activation_operations() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let spec = operation(OP1, "0.1.0", 0x11, None);
    record_phase(&layout, &spec, Phase::Intent, now()).unwrap();
    let staging = stage(&layout, &spec);
    let downloads = layout.downloads_dir(spec.operation_id());
    fs::create_dir_all(&downloads).unwrap();
    fs::write(downloads.join("partial"), b"partial").unwrap();

    assert_eq!(
        recover(&layout, &mut registry, now()).unwrap(),
        RecoveryOutcome::Discarded {
            operation_id: OperationId::parse(OP1).unwrap()
        }
    );
    assert!(!staging.exists());
    assert!(!downloads.exists());
    let contents = read_journal(&layout.journal_path()).unwrap();
    let last = contents.records().last().unwrap();
    assert_eq!(last.phase(), Phase::Failed);
    assert_eq!(
        last.context().unwrap().error_code(),
        Some("RECOVERY_DISCARD")
    );
}

#[test]
fn recovery_fails_closed_without_a_manifest_identity() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let spec = operation(OP1, "0.1.0", 0x11, None);
    let tree = tree_for(&spec);
    fs::create_dir_all(layout.tree_path(&product(), &tree)).unwrap();
    ActivePointer::new(tree.clone(), digest(0x11), now())
        .unwrap()
        .write_atomic(&layout.active_pointer_path(&product()), spec.operation_id())
        .unwrap();

    let context = JournalContext::new(
        None,
        Some(Version::parse("0.1.0").unwrap()),
        Channel::Stable,
        digest(0x11),
        None,
    )
    .unwrap();
    let record = JournalRecord::new(
        spec.operation_id().clone(),
        spec.idempotency_key().clone(),
        product(),
        Phase::ActivationIntent,
        now(),
        Some(context),
    )
    .unwrap();
    append_journal_record(&layout.journal_path(), &record).unwrap();

    assert!(matches!(
        recover(&layout, &mut registry, now()).unwrap_err(),
        UpdaterError::InvalidJournalRecord { .. }
    ));
}

#[test]
fn recovers_a_rollback_before_the_pointer_without_deleting_the_target() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();

    let first = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &first);
    stage(&layout, &first);
    activate(&layout, &mut registry, &first, &NoFaults).unwrap();

    let second = operation(OP2, "0.2.0", 0x22, Some("0.1.0"));
    record_pre_activation(&layout, &second);
    stage(&layout, &second);
    activate(&layout, &mut registry, &second, &NoFaults).unwrap();

    assert!(
        rollback(
            &layout,
            &mut registry,
            &rollback_spec(OP3, "0.1.0"),
            &FailAt(FaultPoint::AfterActivationIntent),
        )
        .is_err()
    );

    assert!(matches!(
        recover(&layout, &mut registry, now()).unwrap(),
        RecoveryOutcome::RolledBack { .. }
    ));
    // The retained target tree existed before the rollback and must survive.
    assert!(layout.tree_path(&product(), &tree_for(&first)).is_dir());
    let pointer = ActivePointer::load(&layout.active_pointer_path(&product()))
        .unwrap()
        .unwrap();
    assert_eq!(pointer.tree(), &tree_for(&second));
    assert_eq!(
        registry.get(&product()).unwrap().product_version(),
        &Version::parse("0.2.0").unwrap()
    );
    assert_eq!(last_journal_phase(&layout), Phase::RolledBack);
}

#[test]
fn recovers_a_rollback_after_the_pointer() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();

    let first = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &first);
    stage(&layout, &first);
    activate(&layout, &mut registry, &first, &NoFaults).unwrap();

    let second = operation(OP2, "0.2.0", 0x22, Some("0.1.0"));
    record_pre_activation(&layout, &second);
    stage(&layout, &second);
    activate(&layout, &mut registry, &second, &NoFaults).unwrap();

    let operation_id = OperationId::parse(OP3).unwrap();
    assert!(
        rollback(
            &layout,
            &mut registry,
            &rollback_spec(OP3, "0.1.0"),
            &FailAt(FaultPoint::AfterPointerWrite),
        )
        .is_err()
    );
    assert_eq!(last_journal_phase(&layout), Phase::Activated);

    assert_eq!(
        recover(&layout, &mut registry, now()).unwrap(),
        RecoveryOutcome::RolledBack {
            operation_id: operation_id.clone()
        }
    );
    let record = registry.get(&product()).unwrap();
    assert_eq!(record.product_version(), &Version::parse("0.1.0").unwrap());
    assert_eq!(record.active_tree(), &tree_for(&first));
    assert_eq!(record.retained().len(), 1);
    assert_eq!(
        record.retained()[0].product_version(),
        &Version::parse("0.2.0").unwrap()
    );
    assert_eq!(last_journal_phase(&layout), Phase::RolledBack);

    assert_eq!(
        recover(&layout, &mut registry, now()).unwrap(),
        RecoveryOutcome::NothingToRecover
    );
}

#[test]
fn recovers_a_rollback_after_the_registry_write() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();

    let first = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &first);
    stage(&layout, &first);
    activate(&layout, &mut registry, &first, &NoFaults).unwrap();

    let second = operation(OP2, "0.2.0", 0x22, Some("0.1.0"));
    record_pre_activation(&layout, &second);
    stage(&layout, &second);
    activate(&layout, &mut registry, &second, &NoFaults).unwrap();

    let operation_id = OperationId::parse(OP3).unwrap();
    assert!(
        rollback(
            &layout,
            &mut registry,
            &rollback_spec(OP3, "0.1.0"),
            &FailAt(FaultPoint::AfterRegistryCommitted),
        )
        .is_err()
    );
    assert_eq!(last_journal_phase(&layout), Phase::RegistryCommitted);

    assert_eq!(
        recover(&layout, &mut registry, now()).unwrap(),
        RecoveryOutcome::RolledBack {
            operation_id: operation_id.clone()
        }
    );
    assert_eq!(last_journal_phase(&layout), Phase::RolledBack);
    assert_eq!(
        registry.get(&product()).unwrap().product_version(),
        &Version::parse("0.1.0").unwrap()
    );
}

#[test]
fn recovery_preserves_the_pointer_activation_time() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let activated_at = timestamp("2026-11-07T12:34:56Z");
    let spec = OperationSpec::new(
        OperationId::parse(OP1).unwrap(),
        IdempotencyKey::parse("install:eqwec-hub:stable:0.1.0").unwrap(),
        release("0.1.0", 0x11),
        None,
        activated_at,
    );
    record_pre_activation(&layout, &spec);
    stage(&layout, &spec);
    assert!(
        activate(
            &layout,
            &mut registry,
            &spec,
            &FailAt(FaultPoint::AfterPointerWrite)
        )
        .is_err()
    );

    recover(&layout, &mut registry, timestamp("2026-11-08T00:00:00Z")).unwrap();
    assert_eq!(
        registry.get(&product()).unwrap().installed_at(),
        activated_at
    );
}

#[test]
fn commit_registry_fails_closed_on_an_unexpected_source_version() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let installed = operation(OP1, "0.2.0", 0x22, None);
    record_pre_activation(&layout, &installed);
    stage(&layout, &installed);
    activate(&layout, &mut registry, &installed, &NoFaults).unwrap();

    let stale = operation(OP2, "0.3.0", 0x33, Some("0.1.0"));
    let tree = tree_for(&stale);
    fs::create_dir_all(layout.tree_path(&product(), &tree)).unwrap();
    assert!(matches!(
        commit_registry(&layout, &mut registry, &stale, &tree).unwrap_err(),
        UpdaterError::RecoveryConflict { .. }
    ));
    assert_eq!(
        registry.get(&product()).unwrap().product_version(),
        &Version::parse("0.2.0").unwrap()
    );
}

#[test]
fn commit_registry_fails_closed_on_a_missing_source_record() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let update = operation(OP1, "0.2.0", 0x22, Some("0.1.0"));
    let tree = tree_for(&update);
    fs::create_dir_all(layout.tree_path(&product(), &tree)).unwrap();

    assert!(matches!(
        commit_registry(&layout, &mut registry, &update, &tree).unwrap_err(),
        UpdaterError::NoInstallation { .. }
    ));
    assert!(registry.get(&product()).is_none());
}

#[test]
fn prunes_unreferenced_version_trees() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();
    let first = operation(OP1, "0.1.0", 0x11, None);
    record_pre_activation(&layout, &first);
    stage(&layout, &first);
    activate(&layout, &mut registry, &first, &NoFaults).unwrap();

    let orphan = layout.versions_dir(&product()).join("0.0.1-deadbeefdead");
    fs::create_dir_all(&orphan).unwrap();
    assert!(orphan.is_dir());

    let second = operation(OP2, "0.2.0", 0x22, Some("0.1.0"));
    record_pre_activation(&layout, &second);
    stage(&layout, &second);
    activate(&layout, &mut registry, &second, &NoFaults).unwrap();

    assert!(!orphan.exists());
    assert!(layout.tree_path(&product(), &tree_for(&first)).is_dir());
    assert!(layout.tree_path(&product(), &tree_for(&second)).is_dir());
}

#[test]
fn retains_at_most_three_versions() {
    let dir = TestDir::new();
    let layout = layout(&dir);
    let mut registry = registry();

    let versions = [
        (OP1, "0.1.0", 0x11u8),
        (OP2, "0.2.0", 0x22),
        ("33333333333333333333333333333333", "0.3.0", 0x33),
        ("44444444444444444444444444444444", "0.4.0", 0x44),
        ("55555555555555555555555555555555", "0.5.0", 0x55),
    ];
    let mut previous: Option<String> = None;
    for (id, version, sha) in versions {
        let spec = operation(id, version, sha, previous.as_deref());
        record_pre_activation(&layout, &spec);
        stage(&layout, &spec);
        activate(&layout, &mut registry, &spec, &NoFaults).unwrap();
        previous = Some(version.to_owned());
    }

    let record = registry.get(&product()).unwrap();
    assert_eq!(record.retained().len(), MAX_RETAINED_VERSIONS);
    let retained: Vec<_> = record
        .retained()
        .iter()
        .map(|entry| entry.product_version().to_string())
        .collect();
    assert_eq!(retained, ["0.4.0", "0.3.0", "0.2.0"]);
}

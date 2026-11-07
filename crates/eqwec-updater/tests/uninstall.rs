//! Journaled uninstall behavior.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use eqwec_core::{Channel, ProductId, ScopeId};
use eqwec_security::{KeyId, SafeRelativePath, Sha256Digest};
use eqwec_updater::{
    FaultInjector, FaultPoint, IdempotencyKey, InstallLayout, InstallationRecord, OperationId,
    RecoveryOutcome, Registry, Timestamp, UpdaterError, recover, uninstall, uninstall_with_faults,
};
use semver::Version;

static NEXT: AtomicU64 = AtomicU64::new(0);

fn temp_base() -> PathBuf {
    let unique = NEXT.fetch_add(1, Ordering::Relaxed);
    let base =
        std::env::temp_dir().join(format!("eqwec-uninstall-{}-{unique}", std::process::id()));
    fs::create_dir_all(&base).expect("create temp base");
    base
}

fn at(seconds: i64) -> Timestamp {
    Timestamp::from_unix_seconds(seconds).expect("valid timestamp")
}

fn product() -> ProductId {
    ProductId::new("eqwec-fixture").expect("valid product id")
}

fn operation(byte: char) -> OperationId {
    OperationId::parse(&byte.to_string().repeat(32)).expect("valid operation id")
}

fn key(value: &str) -> IdempotencyKey {
    IdempotencyKey::parse(value).expect("valid idempotency key")
}

fn tree() -> SafeRelativePath {
    SafeRelativePath::new("versions/1.0.0-0123456789ab").expect("valid tree")
}

fn registry_with(product: &ProductId, root: PathBuf) -> Registry {
    let mut registry = Registry::new(
        ScopeId::new("default").expect("valid scope"),
        at(1_700_000_000),
    );
    let record = InstallationRecord::new(
        product.clone(),
        Version::parse("1.0.0").expect("valid version"),
        Channel::Stable,
        root,
        tree(),
        Vec::new(),
        KeyId::new("dev-fixture-2026").expect("valid key id"),
        1,
        Sha256Digest::from_bytes([7; 32]),
        5,
        at(1_700_000_000),
    )
    .expect("valid installation record");
    registry.upsert(record).expect("insert record");
    registry
}

fn setup() -> (PathBuf, InstallLayout, Registry, ProductId, PathBuf) {
    let base = temp_base();
    let layout = InstallLayout::new(&base).expect("valid layout");
    layout.ensure_base_dirs().expect("create layout");
    let product = product();
    let root = layout.product_root(&product);
    let tree_dir = layout.tree_path(&product, &tree());
    fs::create_dir_all(&tree_dir).expect("create tree");
    fs::write(tree_dir.join("payload.txt"), b"fixture").expect("write payload");
    let registry = registry_with(&product, root.clone());
    registry
        .write_atomic(&layout.registry_path(), &operation('a'))
        .expect("write registry");
    (base, layout, registry, product, root)
}

struct FailAt(FaultPoint);

impl FaultInjector for FailAt {
    fn check(&self, point: FaultPoint) -> Result<(), UpdaterError> {
        if point.as_str() == self.0.as_str() {
            return Err(UpdaterError::FaultInjected {
                point: point.as_str(),
            });
        }
        Ok(())
    }
}

#[test]
fn uninstall_removes_tree_and_record() {
    let (base, layout, mut registry, product, root) = setup();
    uninstall(
        &layout,
        &mut registry,
        &product,
        operation('b'),
        key("uninstall-1"),
        at(1_700_000_100),
    )
    .expect("uninstall");

    assert!(!root.exists(), "product root must be removed");
    assert!(
        registry.get(&product).is_none(),
        "registry record must be removed"
    );
    assert!(
        !layout.registry_path().exists()
            || Registry::load(&layout.registry_path())
                .expect("load")
                .is_some_and(|registry| registry.installations().is_empty()),
        "registry on disk must be empty"
    );
    fs::remove_dir_all(base).expect("cleanup");
}

#[test]
fn uninstall_is_idempotent_for_the_same_key() {
    let (base, layout, mut registry, product, _root) = setup();
    uninstall(
        &layout,
        &mut registry,
        &product,
        operation('b'),
        key("uninstall-1"),
        at(1_700_000_100),
    )
    .expect("first uninstall");
    uninstall(
        &layout,
        &mut registry,
        &product,
        operation('c'),
        key("uninstall-1"),
        at(1_700_000_200),
    )
    .expect("repeat converges");
    fs::remove_dir_all(base).expect("cleanup");
}

#[test]
fn missing_product_is_a_typed_error() {
    let base = temp_base();
    let layout = InstallLayout::new(&base).expect("valid layout");
    layout.ensure_base_dirs().expect("create layout");
    let mut registry = Registry::new(
        ScopeId::new("default").expect("valid scope"),
        at(1_700_000_000),
    );
    let error = uninstall(
        &layout,
        &mut registry,
        &product(),
        operation('b'),
        key("uninstall-1"),
        at(1_700_000_100),
    )
    .expect_err("must fail");
    assert!(matches!(error, UpdaterError::ProductNotInstalled { .. }));
    fs::remove_dir_all(base).expect("cleanup");
}

#[test]
fn record_outside_the_layout_is_rejected() {
    let base = temp_base();
    let layout = InstallLayout::new(&base).expect("valid layout");
    layout.ensure_base_dirs().expect("create layout");
    let product = product();
    let outside = temp_base().join("elsewhere");
    fs::create_dir_all(&outside).expect("create outside root");
    let mut registry = registry_with(&product, outside.clone());
    let error = uninstall(
        &layout,
        &mut registry,
        &product,
        operation('b'),
        key("uninstall-1"),
        at(1_700_000_100),
    )
    .expect_err("must fail");
    assert!(matches!(error, UpdaterError::InstallRootOutsideBase));
    assert!(outside.exists(), "outside root must be untouched");
    fs::remove_dir_all(base).expect("cleanup");
    fs::remove_dir_all(outside).expect("cleanup outside");
}

#[test]
fn interrupted_uninstall_recovers_and_converges() {
    let (base, layout, mut registry, product, root) = setup();
    let error = uninstall_with_faults(
        &layout,
        &mut registry,
        &product,
        operation('b'),
        key("uninstall-1"),
        at(1_700_000_100),
        &FailAt(FaultPoint::AfterUninstallTreeRemoved),
    )
    .expect_err("fault must abort");
    assert!(matches!(error, UpdaterError::FaultInjected { .. }));
    assert!(!root.exists(), "tree is removed before the fault");
    assert!(
        registry.get(&product).is_some(),
        "registry record survives the crash"
    );

    let outcome = recover(&layout, &mut registry, at(1_700_000_200)).expect("recovery");
    assert!(matches!(outcome, RecoveryOutcome::Uninstalled { .. }));
    assert!(registry.get(&product).is_none());
    assert!(matches!(
        recover(&layout, &mut registry, at(1_700_000_300)).expect("second recovery"),
        RecoveryOutcome::NothingToRecover
    ));
    fs::remove_dir_all(base).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn inner_symlink_target_survives_uninstall() {
    use std::os::unix::fs::symlink;

    let (base, layout, mut registry, product, root) = setup();
    let outside = base.join("outside.txt");
    fs::write(&outside, b"keep").expect("write outside");
    let tree_dir = layout.tree_path(&product, &tree());
    symlink(&outside, tree_dir.join("escape")).expect("create symlink");

    uninstall(
        &layout,
        &mut registry,
        &product,
        operation('b'),
        key("uninstall-1"),
        at(1_700_000_100),
    )
    .expect("uninstall");

    assert!(outside.exists(), "symlink target must survive");
    assert_eq!(fs::read(&outside).expect("read outside"), b"keep");
    assert!(!root.exists());
    fs::remove_dir_all(base).expect("cleanup");
}

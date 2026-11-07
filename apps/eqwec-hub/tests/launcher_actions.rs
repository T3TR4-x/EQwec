//! Launcher action integration tests against real fixture bundles.
//!
//! The tests compile tiny probe executables, sign manifests with the public
//! fixture seed, and exercise install, launch, update, rollback, and
//! uninstall without opening a window.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use eqwec_core::{Architecture, Channel, Platform, ProductId};
use eqwec_hub::actions;
use eqwec_hub::error::LauncherError;
use eqwec_hub::launch;
use eqwec_hub::trust;
use eqwec_hub_fixture::{
    BundleSpec, DEV_KEY_ID, compile_failing_probe, compile_fixture_probe, compile_sleeping_probe,
    write_bundle,
};
use eqwec_security::Sha256Digest;
use eqwec_updater::{
    ActivePointer, InstallLayout, OperationId, Registry, Timestamp, VerificationContext,
};

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "eqwec-launcher-{label}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn now() -> Timestamp {
    Timestamp::now_utc().expect("current time")
}

fn dates() -> (String, String) {
    let now = now();
    let issued = now.checked_add_seconds(-3600).expect("issued time");
    let expires = now.checked_add_seconds(7 * 24 * 3600).expect("expiry time");
    (issued.to_string(), expires.to_string())
}

fn platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::MacOs
    } else if cfg!(target_os = "windows") {
        Platform::Windows
    } else {
        Platform::Linux
    }
}

fn architecture() -> Architecture {
    if cfg!(target_arch = "aarch64") {
        Architecture::Aarch64
    } else {
        Architecture::X86_64
    }
}

fn context() -> VerificationContext {
    VerificationContext::new(Channel::Stable, platform(), architecture(), now())
}

fn product() -> ProductId {
    ProductId::new("eqwec-fixture").expect("product")
}

fn bundle(dir: &Path, generation: u8, version: &str, sequence: u64) -> PathBuf {
    let probe = compile_fixture_probe(dir, generation).expect("compile probe");
    bundle_from_probe(dir, &probe, version, sequence)
}

fn bundle_from_probe(dir: &Path, probe: &Path, version: &str, sequence: u64) -> PathBuf {
    let binary = fs::read(probe).expect("read probe");
    let (issued, expires) = dates();
    let spec = BundleSpec {
        version,
        sequence,
        key_id: DEV_KEY_ID,
        issued_at: &issued,
        expires_at: &expires,
    };
    let bundle_dir = dir.join(format!("bundle-{version}"));
    write_bundle(&bundle_dir, &binary, &spec).expect("write bundle");
    bundle_dir
}

fn run_entry(data_dir: &Path) -> String {
    let (entry, tree_root) =
        actions::resolve_installed_entry(data_dir, "eqwec-fixture").expect("resolve entry");
    let output = Command::new(entry)
        .current_dir(tree_root)
        .output()
        .expect("run fixture");
    assert!(
        output.status.success(),
        "fixture failed: {:?}",
        output.status
    );
    String::from_utf8(output.stdout).expect("fixture stdout")
}

fn load_registry(data_dir: &Path) -> Option<Registry> {
    let layout = InstallLayout::new(data_dir).expect("layout");
    Registry::load(&layout.registry_path()).expect("load registry")
}

#[test]
fn installs_updates_rolls_back_and_uninstalls() {
    let bundles = TempDir::new("bundles");
    let data = TempDir::new("profile");
    let v1 = bundle(bundles.path(), 1, "1.0.0", 1);
    let v2 = bundle(bundles.path(), 2, "1.1.0", 2);

    let message = actions::install_bundle(data.path(), &v1).expect("install v1");
    assert!(message.starts_with("Installed eqwec-fixture 1.0.0"));
    assert_eq!(run_entry(data.path()), "eqwec-fixture v1\npayload 1.0.0\n");
    assert!(actions::launch_product(data.path(), "eqwec-fixture").expect("launch") > 0);

    let message = actions::install_bundle(data.path(), &v2).expect("update v2");
    assert!(message.starts_with("Updated eqwec-fixture to 1.1.0"));
    assert_eq!(run_entry(data.path()), "eqwec-fixture v2\npayload 1.1.0\n");

    let message = actions::rollback(data.path(), "eqwec-fixture", "1.0.0").expect("rollback");
    assert!(message.starts_with("Rolled back eqwec-fixture to 1.0.0"));
    assert_eq!(run_entry(data.path()), "eqwec-fixture v1\npayload 1.0.0\n");

    let message = actions::uninstall_product(data.path(), "eqwec-fixture").expect("uninstall");
    assert!(message.starts_with("Uninstalled eqwec-fixture"));
    assert!(load_registry(data.path()).is_none_or(|registry| registry.get(&product()).is_none()));
    let layout = InstallLayout::new(data.path()).expect("layout");
    assert!(!layout.product_root(&product()).exists());
}

#[test]
fn rejects_a_bundle_that_is_not_exactly_two_files() {
    let bundles = TempDir::new("bundles");
    let data = TempDir::new("profile");
    let source = bundle(bundles.path(), 1, "1.0.0", 1);
    let store = trust::trust_store().expect("trust store");

    let relative = actions::validate_bundle(Path::new("bundle"), &store, &context());
    assert!(matches!(relative, Err(LauncherError::Bundle { .. })));

    let missing = bundles.path().join("missing");
    fs::create_dir_all(&missing).expect("create missing");
    fs::copy(source.join("manifest.json"), missing.join("manifest.json")).expect("copy manifest");
    let error = actions::validate_bundle(&missing, &store, &context()).expect_err("missing");
    assert!(matches!(error, LauncherError::Bundle { .. }));

    let extra = bundles.path().join("extra");
    fs::create_dir_all(&extra).expect("create extra");
    fs::copy(source.join("manifest.json"), extra.join("manifest.json")).expect("copy manifest");
    fs::copy(source.join("package.zip"), extra.join("package.zip")).expect("copy package");
    fs::write(extra.join("notes.txt"), b"extra").expect("write extra");
    let error = actions::validate_bundle(&extra, &store, &context()).expect_err("extra");
    assert!(matches!(error, LauncherError::Bundle { .. }));

    let valid = actions::validate_bundle(&source, &store, &context()).expect("valid bundle");
    assert_eq!(valid.product_id(), &product());
    assert_eq!(valid.product_version().to_string(), "1.0.0");
    let _ = data;
}

#[cfg(unix)]
#[test]
fn rejects_a_symlinked_package() {
    let bundles = TempDir::new("bundles");
    let source = bundle(bundles.path(), 1, "1.0.0", 1);
    let store = trust::trust_store().expect("trust store");
    let linked = bundles.path().join("linked");
    fs::create_dir_all(&linked).expect("create linked");
    fs::copy(source.join("manifest.json"), linked.join("manifest.json")).expect("copy manifest");
    std::os::unix::fs::symlink(source.join("package.zip"), linked.join("package.zip"))
        .expect("symlink package");
    let error = actions::validate_bundle(&linked, &store, &context()).expect_err("symlink");
    assert!(matches!(error, LauncherError::Bundle { .. }));
}

#[test]
fn health_check_runs_the_staged_tree() {
    let bundles = TempDir::new("bundles");
    let data = TempDir::new("profile");
    let v1 = bundle(bundles.path(), 1, "1.0.0", 1);
    actions::install_bundle(data.path(), &v1).expect("install");
    let (_, tree_root) =
        actions::resolve_installed_entry(data.path(), "eqwec-fixture").expect("tree");

    launch::health_check(&tree_root, &product()).expect("healthy tree");
    fs::remove_file(tree_root.join("share/fixture.txt")).expect("remove payload");
    let error = launch::health_check(&tree_root, &product()).expect_err("broken tree");
    assert!(matches!(error, LauncherError::HealthCheck { .. }));
}

#[test]
fn health_check_kills_a_hung_child() {
    let bundles = TempDir::new("bundles");
    let tree = bundles.path().join("hung-tree");
    fs::create_dir_all(tree.join("bin")).expect("create bin");
    fs::create_dir_all(tree.join("share")).expect("create share");
    let probe = compile_sleeping_probe(bundles.path()).expect("compile sleeping probe");
    let entry = tree.join(format!("bin/eqwec-fixture{}", std::env::consts::EXE_SUFFIX));
    fs::copy(probe, &entry).expect("copy probe");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&entry, fs::Permissions::from_mode(0o755)).expect("chmod");
    }
    fs::write(tree.join("share/fixture.txt"), b"payload\n").expect("payload");

    let started = Instant::now();
    let error = launch::health_check_with_timeout(&tree, &product(), Duration::from_millis(200))
        .expect_err("timeout");
    assert!(matches!(error, LauncherError::HealthCheck { .. }));
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn rejects_a_tampered_package() {
    let bundles = TempDir::new("bundles");
    let data = TempDir::new("profile");
    let source = bundle(bundles.path(), 1, "1.0.0", 1);
    let package = source.join("package.zip");
    let mut bytes = fs::read(&package).expect("read package");
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;
    fs::write(&package, bytes).expect("tamper package");

    let error = actions::install_bundle(data.path(), &source).expect_err("tampered");
    assert!(matches!(error, LauncherError::Updater(_)));
    let layout = InstallLayout::new(data.path()).expect("layout");
    assert!(!layout.product_root(&product()).exists());
}

#[test]
fn missing_pointer_is_a_typed_error() {
    let bundles = TempDir::new("bundles");
    let data = TempDir::new("profile");
    let v1 = bundle(bundles.path(), 1, "1.0.0", 1);
    actions::install_bundle(data.path(), &v1).expect("install");
    let layout = InstallLayout::new(data.path()).expect("layout");
    fs::remove_file(layout.active_pointer_path(&product())).expect("remove pointer");
    let error =
        actions::resolve_installed_entry(data.path(), "eqwec-fixture").expect_err("pointer");
    assert!(matches!(error, LauncherError::Pointer(_)));
}

#[test]
fn mismatched_pointer_is_a_typed_error() {
    let bundles = TempDir::new("bundles");
    let data = TempDir::new("profile");
    let v1 = bundle(bundles.path(), 1, "1.0.0", 1);
    actions::install_bundle(data.path(), &v1).expect("install");
    let layout = InstallLayout::new(data.path()).expect("layout");
    let registry = load_registry(data.path()).expect("registry");
    let record = registry.get(&product()).expect("record").clone();
    ActivePointer::new(
        record.active_tree().clone(),
        Sha256Digest::from_bytes([9; 32]),
        now(),
    )
    .expect("pointer")
    .write_atomic(
        &layout.active_pointer_path(&product()),
        &OperationId::parse(&"b".repeat(32)).expect("operation"),
    )
    .expect("write pointer");
    let error =
        actions::resolve_installed_entry(data.path(), "eqwec-fixture").expect_err("mismatch");
    assert!(matches!(error, LauncherError::Pointer(_)));
}

#[cfg(unix)]
#[test]
fn rejects_a_symlinked_tree() {
    let bundles = TempDir::new("bundles");
    let data = TempDir::new("profile");
    let v1 = bundle(bundles.path(), 1, "1.0.0", 1);
    actions::install_bundle(data.path(), &v1).expect("install");
    let layout = InstallLayout::new(data.path()).expect("layout");
    let registry = load_registry(data.path()).expect("registry");
    let record = registry.get(&product()).expect("record").clone();
    let tree = layout.tree_path(&product(), record.active_tree());
    let outside = bundles.path().join("outside");
    fs::create_dir_all(outside.join("bin")).expect("outside bin");
    fs::remove_dir_all(&tree).expect("remove tree");
    std::os::unix::fs::symlink(&outside, &tree).expect("symlink tree");
    let error = actions::resolve_installed_entry(data.path(), "eqwec-fixture").expect_err("tree");
    assert!(matches!(error, LauncherError::Launch { .. }));
}

#[cfg(unix)]
#[test]
fn rejects_a_symlinked_entry_point() {
    let bundles = TempDir::new("bundles");
    let data = TempDir::new("profile");
    let v1 = bundle(bundles.path(), 1, "1.0.0", 1);
    actions::install_bundle(data.path(), &v1).expect("install");
    let (entry, _tree_root) =
        actions::resolve_installed_entry(data.path(), "eqwec-fixture").expect("entry");
    fs::remove_file(&entry).expect("remove entry");
    std::os::unix::fs::symlink("/bin/true", &entry).expect("symlink entry");
    let error =
        actions::resolve_installed_entry(data.path(), "eqwec-fixture").expect_err("symlink");
    assert!(matches!(error, LauncherError::Launch { .. }));
}

#[test]
fn missing_entry_point_is_a_typed_error() {
    let bundles = TempDir::new("bundles");
    let data = TempDir::new("profile");
    let v1 = bundle(bundles.path(), 1, "1.0.0", 1);
    actions::install_bundle(data.path(), &v1).expect("install");
    let (entry, _tree_root) =
        actions::resolve_installed_entry(data.path(), "eqwec-fixture").expect("entry");
    fs::remove_file(&entry).expect("remove entry");
    let error =
        actions::resolve_installed_entry(data.path(), "eqwec-fixture").expect_err("missing");
    assert!(matches!(error, LauncherError::Launch { .. }));
}

#[test]
fn failed_health_check_leaves_the_previous_version_active() {
    let bundles = TempDir::new("bundles");
    let data = TempDir::new("profile");
    let v1 = bundle(bundles.path(), 1, "1.0.0", 1);
    actions::install_bundle(data.path(), &v1).expect("install v1");

    let failing = compile_failing_probe(bundles.path()).expect("compile failing probe");
    let bad = bundle_from_probe(bundles.path(), &failing, "1.1.0", 2);
    let error = actions::install_bundle(data.path(), &bad).expect_err("health check");
    assert!(matches!(error, LauncherError::HealthCheck { .. }));

    assert_eq!(run_entry(data.path()), "eqwec-fixture v1\npayload 1.0.0\n");
    let registry = load_registry(data.path()).expect("registry");
    let record = registry.get(&product()).expect("record");
    assert_eq!(record.product_version().to_string(), "1.0.0");

    let v2 = bundle(bundles.path(), 2, "1.1.0", 3);
    actions::install_bundle(data.path(), &v2).expect("retry update");
    assert_eq!(run_entry(data.path()), "eqwec-fixture v2\npayload 1.1.0\n");
}

#[test]
fn failed_first_install_leaves_no_product() {
    let bundles = TempDir::new("bundles");
    let data = TempDir::new("profile");
    let failing = compile_failing_probe(bundles.path()).expect("compile failing probe");
    let bad = bundle_from_probe(bundles.path(), &failing, "1.0.0", 1);
    let error = actions::install_bundle(data.path(), &bad).expect_err("health check");
    assert!(matches!(error, LauncherError::HealthCheck { .. }));

    let layout = InstallLayout::new(data.path()).expect("layout");
    assert!(!layout.versions_dir(&product()).exists());
    assert!(!layout.active_pointer_path(&product()).exists());
    assert!(load_registry(data.path()).is_none_or(|registry| registry.get(&product()).is_none()));

    let v1 = bundle(bundles.path(), 1, "1.0.0", 1);
    actions::install_bundle(data.path(), &v1).expect("retry install");
    assert_eq!(run_entry(data.path()), "eqwec-fixture v1\npayload 1.0.0\n");
}

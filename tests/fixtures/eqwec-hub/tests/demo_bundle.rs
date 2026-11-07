//! The demo generator produces a bundle the launcher accepts.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use eqwec_core::{Architecture, Channel, Platform};
use eqwec_updater::{Timestamp, TrustStore, VerificationContext, parse_and_verify};

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "eqwec-demo-{label}-{}-{}",
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

fn launcher_trust_store() -> TrustStore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../apps/eqwec-hub/assets/dev-trust-store.json");
    TrustStore::parse(&fs::read(path).expect("read launcher trust store"))
        .expect("parse launcher trust store")
}

fn context() -> VerificationContext {
    VerificationContext::new(
        Channel::Stable,
        platform(),
        architecture(),
        Timestamp::now_utc().expect("current time"),
    )
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

fn generate(outdir: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_make-demo-bundle"))
        .arg(outdir)
        .args(extra)
        .output()
        .expect("run make-demo-bundle")
}

#[test]
fn generated_bundle_verifies_with_the_launcher_trust_store() {
    let temp = TempDir::new("bundle");
    let bundle = temp.path().join("bundle");
    let output = generate(&bundle, &["1.0.0", "1"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let manifest = fs::read(bundle.join("manifest.json")).expect("manifest");
    let verified =
        parse_and_verify(&manifest, &launcher_trust_store(), &context()).expect("verify bundle");
    assert_eq!(verified.manifest().product_id().as_str(), "eqwec-fixture");
    assert_eq!(verified.manifest().product_version().to_string(), "1.0.0");
    let package = fs::metadata(bundle.join("package.zip")).expect("package");
    assert_eq!(package.len(), verified.manifest().package().size());

    let mut entries: Vec<String> = fs::read_dir(&bundle)
        .expect("read bundle")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .into_string()
                .expect("utf-8 name")
        })
        .collect();
    entries.sort();
    assert_eq!(entries, ["manifest.json", "package.zip"]);
}

#[test]
fn generator_overwrites_bundle_files_and_keeps_them_valid() {
    let temp = TempDir::new("bundle-rewrite");
    let bundle = temp.path().join("bundle");
    assert!(generate(&bundle, &["1.0.0", "1"]).status.success());
    assert!(generate(&bundle, &["1.1.0", "2"]).status.success());

    let manifest = fs::read(bundle.join("manifest.json")).expect("manifest");
    let verified =
        parse_and_verify(&manifest, &launcher_trust_store(), &context()).expect("verify bundle");
    assert_eq!(verified.manifest().product_version().to_string(), "1.1.0");
    assert_eq!(verified.manifest().metadata_sequence(), 2);
}

#[test]
fn generator_refuses_a_folder_with_unexpected_files() {
    let temp = TempDir::new("bundle-extra");
    let bundle = temp.path().join("bundle");
    fs::create_dir_all(&bundle).expect("create bundle dir");
    fs::write(bundle.join("notes.txt"), b"x").expect("write notes");
    let output = generate(&bundle, &[]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("notes.txt"));
}

#[test]
fn generator_requires_an_output_directory() {
    let output = Command::new(env!("CARGO_BIN_EXE_make-demo-bundle"))
        .output()
        .expect("run make-demo-bundle");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage"));
}

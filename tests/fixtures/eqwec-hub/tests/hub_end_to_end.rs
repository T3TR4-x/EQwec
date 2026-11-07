//! End-to-end Hub fixture workflow.
//!
//! The tests build real fixture executables into ZIP packages, sign manifests
//! with a fixed non-production seed, verify the manifests and packages, stage
//! them, activate them, launch the installed binary, update, roll back, and
//! recover an interrupted update.
//!
//! The seed is public test material. It must never be used for a release.

use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use ed25519_dalek::{Signer, SigningKey};
use eqwec_core::{Architecture, Channel, Platform, ProductId, ScopeId};
use eqwec_security::hex::encode_lower_hex;
use eqwec_security::{Ed25519PublicKey, KeyId, Sha256Digest};
use eqwec_updater::{
    ActivationOutcome, ExtractionLimits, FaultInjector, FaultPoint, IdempotencyKey, InstallLayout,
    Manifest, NoFaults, OperationId, OperationSpec, Phase, RecoveryOutcome, Registry, RollbackSpec,
    Timestamp, TrustStore, TrustedKey, UpdaterError, VerificationContext, VerifiedManifest,
    VerifiedRelease, activate, parse_and_verify, read_journal, record_phase, recover, rollback,
    stage_package_from_path, tree_id,
};
use semver::Version;
use serde_json::{Value, json};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, System, ZipWriter};

/// Public non-production seed used only by the fixture tests.
const SEED: [u8; 32] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
];

const KEY_ID: &str = "fixture-release-2026-01";
const PRODUCT_ID: &str = "eqwec-fixture";
const NOW: &str = "2026-11-07T12:00:00Z";
const ISSUED_AT: &str = "2026-11-07T00:00:00Z";
const EXPIRES_AT: &str = "2027-02-05T00:00:00Z";

const OP_INSTALL: &str = "11111111111111111111111111111111";
const OP_UPDATE: &str = "22222222222222222222222222222222";
const OP_ROLLBACK: &str = "33333333333333333333333333333333";
const OP_TAMPERED: &str = "44444444444444444444444444444444";
const OP_RETRY: &str = "55555555555555555555555555555555";
const OP_INTERRUPTED: &str = "66666666666666666666666666666666";

#[cfg(windows)]
const FIXTURE_EXE: &str = "bin/fixture.exe";
#[cfg(not(windows))]
const FIXTURE_EXE: &str = "bin/fixture";

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "eqwec-hub-fixture-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("create fixture temp dir");
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

fn now() -> Timestamp {
    Timestamp::parse(NOW).expect("valid fixture time")
}

fn timestamp(value: &str) -> Timestamp {
    Timestamp::parse(value).expect("valid fixture timestamp")
}

fn product() -> ProductId {
    ProductId::new(PRODUCT_ID).expect("valid fixture product id")
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

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&SEED)
}

fn trust_store() -> TrustStore {
    let public_key = Ed25519PublicKey::from_bytes(signing_key().verifying_key().to_bytes())
        .expect("valid fixture public key");
    let key = TrustedKey::new(
        KeyId::new(KEY_ID).expect("valid fixture key id"),
        public_key,
        1,
        vec![Channel::Stable],
        timestamp("2026-01-01T00:00:00Z"),
        timestamp("2028-01-01T00:00:00Z"),
    )
    .expect("valid fixture trusted key");
    TrustStore::new(vec![key]).expect("valid fixture trust store")
}

fn context(minimum_sequence: Option<u64>) -> VerificationContext {
    let context = VerificationContext::new(Channel::Stable, platform(), architecture(), now());
    match minimum_sequence {
        Some(sequence) => context.with_minimum_sequence(sequence),
        None => context,
    }
}

fn manifest_value(version: &str, sequence: u64, package: &[u8], signature: &str) -> Value {
    json!({
        "schema_version": 1,
        "product_id": PRODUCT_ID,
        "product_version": version,
        "channel": "stable",
        "target": {
            "os": platform().as_str(),
            "architecture": architecture().as_str(),
            "minimum_platform": "1.0"
        },
        "package": {
            "source": format!(
                "https://packages.eqwec.example/fixture/{version}/eqwec-fixture-{version}.zip"
            ),
            "size": package.len(),
            "hashes": [{ "algorithm": "sha256", "value": Sha256Digest::hash(package).to_hex() }]
        },
        "dependencies": [],
        "metadata_sequence": sequence,
        "issued_at": ISSUED_AT,
        "expires_at": EXPIRES_AT,
        "release_notes": format!("Fixture release {version}."),
        "signature": {
            "key_id": KEY_ID,
            "algorithm": "ed25519",
            "signature": signature,
            "signing_time": ISSUED_AT,
            "key_epoch": 1
        }
    })
}

fn manifest_bytes(version: &str, sequence: u64, package: &[u8]) -> Vec<u8> {
    let placeholder = "00".repeat(64);
    let unsigned = serde_json::to_vec(&manifest_value(version, sequence, package, &placeholder))
        .expect("serialize unsigned fixture manifest");
    let parsed = Manifest::parse(&unsigned).expect("parse unsigned fixture manifest");
    let signature = signing_key().sign(&parsed.canonical_payload());
    let signed = manifest_value(
        version,
        sequence,
        package,
        &encode_lower_hex(&signature.to_bytes()),
    );
    serde_json::to_vec(&signed).expect("serialize signed fixture manifest")
}

fn verified_manifest(
    version: &str,
    sequence: u64,
    package: &[u8],
    minimum_sequence: Option<u64>,
) -> VerifiedManifest {
    let bytes = manifest_bytes(version, sequence, package);
    parse_and_verify(&bytes, &trust_store(), &context(minimum_sequence))
        .expect("verify fixture manifest")
}

struct Release {
    manifest: VerifiedManifest,
    spec: OperationSpec,
}

fn release(
    version: &str,
    sequence: u64,
    from_version: Option<&str>,
    operation_id: &str,
    package: &[u8],
    minimum_sequence: Option<u64>,
) -> Release {
    let manifest = verified_manifest(version, sequence, package, minimum_sequence);
    let verified_release =
        VerifiedRelease::from_verified_manifest(&manifest).expect("verified fixture release");
    let spec = OperationSpec::new(
        OperationId::parse(operation_id).expect("valid fixture operation id"),
        IdempotencyKey::parse(&format!("fixture:{version}:{operation_id}"))
            .expect("valid fixture idempotency key"),
        verified_release,
        from_version.map(|value| Version::parse(value).expect("valid from-version")),
        now(),
    );
    Release { manifest, spec }
}

fn fixture_binary(version: u8) -> Vec<u8> {
    let path = match version {
        1 => env!("CARGO_BIN_EXE_fixture-v1"),
        2 => env!("CARGO_BIN_EXE_fixture-v2"),
        other => panic!("unsupported fixture version: {other}"),
    };
    fs::read(path).expect("read fixture executable")
}

fn write_entry(writer: &mut ZipWriter<Cursor<Vec<u8>>>, name: &str, data: &[u8], mode: u32) {
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default())
        .external_attributes(mode << 16)
        .system(System::Unix)
        .name(name.as_bytes());
    writer
        .start_file_with_options(options)
        .expect("start fixture entry");
    writer.write_all(data).expect("write fixture entry");
}

fn build_package(binary: &[u8], version: &str) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    write_entry(&mut writer, FIXTURE_EXE, binary, 0o100_755);
    write_entry(
        &mut writer,
        "share/fixture.txt",
        format!("payload {version}\n").as_bytes(),
        0o100_644,
    );
    writer
        .finish()
        .expect("finish fixture package")
        .into_inner()
}

fn stage_and_activate(
    layout: &InstallLayout,
    registry: &mut Registry,
    release: &Release,
    package: &[u8],
) -> ActivationOutcome {
    record_phase(layout, &release.spec, Phase::Intent, now()).expect("record intent");
    let path = layout.download_package_path(release.spec.operation_id());
    fs::create_dir_all(path.parent().expect("download parent")).expect("create download dir");
    fs::write(&path, package).expect("write downloaded package");
    record_phase(layout, &release.spec, Phase::Downloaded, now()).expect("record downloaded");
    let tree = stage_package_from_path(
        layout,
        &release.spec,
        &release.manifest,
        &ExtractionLimits::default(),
        now(),
    )
    .expect("stage fixture package");
    assert!(tree.root().join(FIXTURE_EXE).is_file());
    activate(layout, registry, &release.spec, &NoFaults).expect("activate fixture package")
}

fn run_installed(layout: &InstallLayout, registry: &Registry) -> String {
    let record = registry.get(&product()).expect("fixture is installed");
    let executable = layout
        .tree_path(&product(), record.active_tree())
        .join(FIXTURE_EXE);
    let output = Command::new(&executable)
        .output()
        .expect("run installed fixture");
    assert!(
        output.status.success(),
        "fixture failed: status={:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("fixture stdout is UTF-8")
}

fn reload(layout: &InstallLayout) -> Registry {
    Registry::load(&layout.registry_path())
        .expect("load registry")
        .expect("registry exists")
}

fn profile() -> (TestDir, InstallLayout, Registry) {
    let dir = TestDir::new();
    let layout = InstallLayout::new(dir.path().join("profile")).expect("valid fixture layout");
    layout.ensure_base_dirs().expect("create fixture layout");
    let registry = Registry::new(ScopeId::new("default").expect("valid scope"), now());
    (dir, layout, registry)
}

#[test]
fn installs_updates_and_rolls_back_a_real_fixture() {
    let (_dir, layout, mut registry) = profile();

    let package_v1 = build_package(&fixture_binary(1), "1.0.0");
    let release_v1 = release("1.0.0", 1, None, OP_INSTALL, &package_v1, None);
    let outcome = stage_and_activate(&layout, &mut registry, &release_v1, &package_v1);
    assert!(matches!(outcome, ActivationOutcome::Activated { .. }));
    assert_eq!(
        run_installed(&layout, &registry),
        "eqwec-fixture v1\npayload 1.0.0\n"
    );

    let stored = reload(&layout);
    let record = stored.get(&product()).expect("installation record");
    assert_eq!(
        record.product_version(),
        &Version::parse("1.0.0").expect("valid version")
    );
    assert!(record.retained().is_empty());

    let package_v2 = build_package(&fixture_binary(2), "1.1.0");
    let release_v2 = release("1.1.0", 2, Some("1.0.0"), OP_UPDATE, &package_v2, Some(1));
    let outcome = stage_and_activate(&layout, &mut registry, &release_v2, &package_v2);
    assert!(matches!(outcome, ActivationOutcome::Activated { .. }));
    assert_eq!(
        run_installed(&layout, &registry),
        "eqwec-fixture v2\npayload 1.1.0\n"
    );

    let stored = reload(&layout);
    let record = stored.get(&product()).expect("installation record");
    assert_eq!(
        record.product_version(),
        &Version::parse("1.1.0").expect("valid version")
    );
    assert_eq!(record.retained().len(), 1);
    assert_eq!(
        record.retained()[0].product_version(),
        &Version::parse("1.0.0").expect("valid version")
    );

    let rollback_spec = RollbackSpec::new(
        OperationId::parse(OP_ROLLBACK).expect("valid operation id"),
        IdempotencyKey::parse("fixture:rollback:1.0.0").expect("valid idempotency key"),
        product(),
        Version::parse("1.0.0").expect("valid version"),
        now(),
    );
    let outcome = rollback(&layout, &mut registry, &rollback_spec, &NoFaults).expect("rollback");
    assert!(matches!(outcome, ActivationOutcome::RolledBack { .. }));
    assert_eq!(
        run_installed(&layout, &registry),
        "eqwec-fixture v1\npayload 1.0.0\n"
    );

    let stored = reload(&layout);
    let record = stored.get(&product()).expect("installation record");
    assert_eq!(
        record.product_version(),
        &Version::parse("1.0.0").expect("valid version")
    );
    assert_eq!(record.retained().len(), 1);
    assert_eq!(
        record.retained()[0].product_version(),
        &Version::parse("1.1.0").expect("valid version")
    );
}

#[test]
fn rejects_a_tampered_update_and_recovers() {
    let (_dir, layout, mut registry) = profile();

    let package_v1 = build_package(&fixture_binary(1), "1.0.0");
    let release_v1 = release("1.0.0", 1, None, OP_INSTALL, &package_v1, None);
    stage_and_activate(&layout, &mut registry, &release_v1, &package_v1);

    let package_v2 = build_package(&fixture_binary(2), "1.1.0");
    let release_v2 = release("1.1.0", 2, Some("1.0.0"), OP_TAMPERED, &package_v2, Some(1));
    record_phase(&layout, &release_v2.spec, Phase::Intent, now()).expect("record intent");
    let path = layout.download_package_path(release_v2.spec.operation_id());
    fs::create_dir_all(path.parent().expect("download parent")).expect("create download dir");
    let mut tampered = package_v2.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0xff;
    fs::write(&path, &tampered).expect("write tampered package");
    record_phase(&layout, &release_v2.spec, Phase::Downloaded, now()).expect("record downloaded");

    let error = stage_package_from_path(
        &layout,
        &release_v2.spec,
        &release_v2.manifest,
        &ExtractionLimits::default(),
        now(),
    )
    .expect_err("tampered package must fail");
    assert!(
        matches!(error, UpdaterError::PackageHashMismatch { .. }),
        "unexpected error: {error:?}"
    );

    let journal = read_journal(&layout.journal_path()).expect("read journal");
    assert_eq!(
        journal.records().last().expect("journal record").phase(),
        Phase::Downloaded
    );
    assert_eq!(
        run_installed(&layout, &registry),
        "eqwec-fixture v1\npayload 1.0.0\n"
    );

    let outcome = recover(&layout, &mut registry, now()).expect("recover tampered update");
    assert!(matches!(outcome, RecoveryOutcome::Discarded { .. }));
    assert!(
        !layout
            .downloads_dir(release_v2.spec.operation_id())
            .exists()
    );

    let retry = release("1.1.0", 2, Some("1.0.0"), OP_RETRY, &package_v2, Some(1));
    let outcome = stage_and_activate(&layout, &mut registry, &retry, &package_v2);
    assert!(matches!(outcome, ActivationOutcome::Activated { .. }));
    assert_eq!(
        run_installed(&layout, &registry),
        "eqwec-fixture v2\npayload 1.1.0\n"
    );
}

struct AbortAfterTreeMove;

impl FaultInjector for AbortAfterTreeMove {
    fn check(&self, point: FaultPoint) -> Result<(), UpdaterError> {
        if point == FaultPoint::AfterTreeMove {
            Err(UpdaterError::FaultInjected {
                point: point.as_str(),
            })
        } else {
            Ok(())
        }
    }
}

#[test]
fn recovers_an_interrupted_update_without_losing_the_old_version() {
    let (_dir, layout, mut registry) = profile();

    let package_v1 = build_package(&fixture_binary(1), "1.0.0");
    let release_v1 = release("1.0.0", 1, None, OP_INSTALL, &package_v1, None);
    stage_and_activate(&layout, &mut registry, &release_v1, &package_v1);

    let package_v2 = build_package(&fixture_binary(2), "1.1.0");
    let release_v2 = release(
        "1.1.0",
        2,
        Some("1.0.0"),
        OP_INTERRUPTED,
        &package_v2,
        Some(1),
    );
    record_phase(&layout, &release_v2.spec, Phase::Intent, now()).expect("record intent");
    let path = layout.download_package_path(release_v2.spec.operation_id());
    fs::create_dir_all(path.parent().expect("download parent")).expect("create download dir");
    fs::write(&path, &package_v2).expect("write downloaded package");
    record_phase(&layout, &release_v2.spec, Phase::Downloaded, now()).expect("record downloaded");
    stage_package_from_path(
        &layout,
        &release_v2.spec,
        &release_v2.manifest,
        &ExtractionLimits::default(),
        now(),
    )
    .expect("stage fixture package");

    let error = activate(
        &layout,
        &mut registry,
        &release_v2.spec,
        &AbortAfterTreeMove,
    )
    .expect_err("activation must be interrupted");
    assert!(matches!(error, UpdaterError::FaultInjected { .. }));
    assert_eq!(
        run_installed(&layout, &registry),
        "eqwec-fixture v1\npayload 1.0.0\n"
    );

    let outcome = recover(&layout, &mut registry, now()).expect("recover interrupted update");
    assert!(matches!(outcome, RecoveryOutcome::RolledBack { .. }));
    assert_eq!(
        run_installed(&layout, &registry),
        "eqwec-fixture v1\npayload 1.0.0\n"
    );

    let v2_tree = tree_id(
        &Version::parse("1.1.0").expect("valid version"),
        Sha256Digest::hash(&package_v2),
    )
    .expect("valid tree id");
    assert!(!layout.tree_path(&product(), &v2_tree).exists());
    assert!(
        !layout
            .downloads_dir(release_v2.spec.operation_id())
            .exists()
    );
}

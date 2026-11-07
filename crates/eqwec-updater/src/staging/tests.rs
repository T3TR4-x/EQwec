//! Bounded archive validation and staging tests.
//!
//! Every fixture is generated in-process with non-production data. The tests
//! cover the ADR-0012 gates 1-11: a valid fixture, path traversal and unsafe
//! names, special modes, duplicate and case-colliding names, every bound,
//! header confusion, encryption, unsupported compression, multi-disk and
//! empty archives, pre-existing staging, and failed-extraction cleanup.

use super::*;
use crate::activation::{ActivationOutcome, NoFaults, activate};
use crate::journal::{IdempotencyKey, OperationId};
use crate::layout::tree_id;
use crate::manifest::Manifest;
use crate::registry::Registry;
use eqwec_core::{Channel, ProductId, ScopeId};
use eqwec_security::KeyId;
use semver::Version;
use serde_json::json;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use zip::write::SimpleFileOptions;
use zip::{System, ZipWriter};

const OP: &str = "0123456789abcdef0123456789abcdef";
const KEY_ID: &str = "release-2026-01";

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "eqwec-staging-{}-{}",
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

fn now() -> Timestamp {
    Timestamp::parse("2026-11-07T00:00:00Z").unwrap()
}

fn product() -> ProductId {
    ProductId::new("eqwec-hub").unwrap()
}

fn key() -> KeyId {
    KeyId::new(KEY_ID).unwrap()
}

fn manifest_for(package: &[u8], version: &str, digest: Sha256Digest) -> VerifiedManifest {
    let value = json!({
        "schema_version": 1,
        "product_id": "eqwec-hub",
        "product_version": version,
        "channel": "stable",
        "target": {
            "os": "macos",
            "architecture": "aarch64",
            "minimum_platform": "11.0"
        },
        "package": {
            "source": "https://packages.eqwec.example/hub/0.1.0/eqwec-hub-0.1.0-macos-aarch64.zip",
            "size": package.len(),
            "hashes": [{ "algorithm": "sha256", "value": digest.to_hex() }]
        },
        "dependencies": [],
        "metadata_sequence": 1,
        "issued_at": "2026-11-07T00:00:00Z",
        "expires_at": "2027-02-05T00:00:00Z",
        "release_notes": "Staging fixture.",
        "signature": {
            "key_id": KEY_ID,
            "algorithm": "ed25519",
            "signature": "00".repeat(64),
            "signing_time": "2026-11-07T00:00:00Z",
            "key_epoch": 1
        }
    });
    let manifest = Manifest::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
    VerifiedManifest::new(manifest)
}

struct Harness {
    dir: TestDir,
    layout: InstallLayout,
    spec: OperationSpec,
    manifest: VerifiedManifest,
    package: Vec<u8>,
}

impl Harness {
    fn new(package: Vec<u8>) -> Self {
        Self::with_phases(package, &[Phase::Intent, Phase::Downloaded])
    }

    fn with_phases(package: Vec<u8>, phases: &[Phase]) -> Self {
        let dir = TestDir::new();
        let layout = InstallLayout::new(dir.path().join("profile")).unwrap();
        layout.ensure_base_dirs().unwrap();
        let digest = Sha256Digest::hash(&package);
        let release = VerifiedRelease::new(
            product(),
            Version::parse("0.1.0").unwrap(),
            Channel::Stable,
            digest,
            key(),
            1,
            1,
        )
        .unwrap();
        let spec = OperationSpec::new(
            OperationId::parse(OP).unwrap(),
            IdempotencyKey::parse("install:eqwec-hub:stable:0.1.0").unwrap(),
            release,
            None,
            now(),
        );
        let manifest = manifest_for(&package, "0.1.0", digest);
        for phase in phases {
            record_phase(&layout, &spec, *phase, now()).unwrap();
        }
        Self {
            dir,
            layout,
            spec,
            manifest,
            package,
        }
    }

    fn stage(&self, limits: &ExtractionLimits) -> Result<StagedTree, UpdaterError> {
        let mut reader = Cursor::new(self.package.clone());
        stage_package(
            &self.layout,
            &self.spec,
            &self.manifest,
            &mut reader,
            limits,
            now(),
        )
    }

    fn stage_from_path(&self, limits: &ExtractionLimits) -> Result<StagedTree, UpdaterError> {
        let path = self.layout.download_package_path(self.spec.operation_id());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &self.package).unwrap();
        stage_package_from_path(&self.layout, &self.spec, &self.manifest, limits, now())
    }

    fn staging_path(&self) -> PathBuf {
        self.layout
            .staging_dir(self.spec.release().product_id(), self.spec.operation_id())
    }

    fn last_phase(&self) -> Phase {
        read_journal(&self.layout.journal_path())
            .unwrap()
            .records()
            .last()
            .unwrap()
            .phase()
    }
}

struct ZipEntry {
    name: Vec<u8>,
    data: Vec<u8>,
    mode: Option<u32>,
    compression: CompressionMethod,
}

impl ZipEntry {
    fn file(name: &[u8], data: &[u8]) -> Self {
        Self {
            name: name.to_vec(),
            data: data.to_vec(),
            mode: None,
            compression: CompressionMethod::Deflated,
        }
    }

    fn stored(name: &[u8], data: &[u8]) -> Self {
        Self {
            name: name.to_vec(),
            data: data.to_vec(),
            mode: None,
            compression: CompressionMethod::Stored,
        }
    }

    fn dir(name: &[u8]) -> Self {
        Self {
            name: name.to_vec(),
            data: Vec::new(),
            mode: Some(0o040_755),
            compression: CompressionMethod::Stored,
        }
    }

    fn mode(mut self, mode: u32) -> Self {
        self.mode = Some(mode);
        self
    }
}

fn build_zip(entries: &[ZipEntry]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for entry in entries {
        let mut options = SimpleFileOptions::default()
            .compression_method(entry.compression)
            .last_modified_time(zip::DateTime::default());
        if let Some(mode) = entry.mode {
            options = options.external_attributes(mode << 16).system(System::Unix);
        }
        let options = options.name(&entry.name);
        writer.start_file_with_options(options).unwrap();
        writer.write_all(&entry.data).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn build_zip_with_comment(entries: &[ZipEntry], comment: &str) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for entry in entries {
        let mut options = SimpleFileOptions::default()
            .compression_method(entry.compression)
            .last_modified_time(zip::DateTime::default());
        if let Some(mode) = entry.mode {
            options = options.external_attributes(mode << 16).system(System::Unix);
        }
        let options = options.name(&entry.name);
        writer.start_file_with_options(options).unwrap();
        writer.write_all(&entry.data).unwrap();
    }
    writer.set_comment(comment).unwrap();
    writer.finish().unwrap().into_inner()
}

fn stage_ok(entries: &[ZipEntry]) -> (Harness, StagedTree) {
    let harness = Harness::new(build_zip(entries));
    let tree = harness.stage(&ExtractionLimits::default()).unwrap();
    (harness, tree)
}

fn stage_err(entries: &[ZipEntry]) -> (Harness, UpdaterError) {
    let harness = Harness::new(build_zip(entries));
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    (harness, error)
}

fn stage_with(
    entries: &[ZipEntry],
    limits: ExtractionLimits,
) -> (Harness, Result<StagedTree, UpdaterError>) {
    let harness = Harness::new(build_zip(entries));
    let result = harness.stage(&limits);
    (harness, result)
}

#[test]
fn stages_a_fixture_package_and_activates_it() {
    let (harness, tree) = stage_ok(&[
        ZipEntry::file(b"README.txt", b"hello"),
        ZipEntry::file(b"bin/tool", b"#!/bin/sh\n").mode(0o100_755),
    ]);

    assert_eq!(tree.entries().len(), 3);
    assert_eq!(harness.last_phase(), Phase::Staged);
    assert!(tree.root().join("README.txt").is_file());
    assert!(tree.root().join("bin/tool").is_file());
    assert!(tree.root().join("bin").is_dir());

    let readme = tree
        .entries()
        .iter()
        .find(|entry| entry.path().as_str() == "README.txt")
        .unwrap();
    assert_eq!(readme.kind(), StagedEntryKind::File);
    assert_eq!(readme.size(), 5);
    assert_eq!(readme.sha256(), Some(Sha256Digest::hash(b"hello")));
    assert_eq!(readme.mode(), 0o644);

    let tool = tree
        .entries()
        .iter()
        .find(|entry| entry.path().as_str() == "bin/tool")
        .unwrap();
    assert_eq!(tool.mode(), 0o755);

    let mut registry = Registry::new(ScopeId::new("default").unwrap(), now());
    let outcome = activate(&harness.layout, &mut registry, &harness.spec, &NoFaults).unwrap();
    assert!(matches!(outcome, ActivationOutcome::Activated { .. }));
    let record = registry.get(&product()).unwrap();
    assert_eq!(record.product_version(), &Version::parse("0.1.0").unwrap());
    let tree_path = harness.layout.tree_path(
        &product(),
        &tree_id(&Version::parse("0.1.0").unwrap(), record.package_sha256()).unwrap(),
    );
    assert!(tree_path.join("README.txt").is_file());
    assert!(!harness.staging_path().exists());
    let _ = &harness.dir;
}

#[test]
fn stages_from_the_download_package_path() {
    let harness = Harness::new(build_zip(&[ZipEntry::file(b"payload", b"data")]));
    let tree = harness
        .stage_from_path(&ExtractionLimits::default())
        .unwrap();
    assert!(tree.root().join("payload").is_file());
    assert_eq!(harness.last_phase(), Phase::Staged);
}

#[test]
fn rewinds_the_reader_before_verifying() {
    let harness = Harness::new(build_zip(&[ZipEntry::file(b"payload", b"data")]));
    let mut reader = Cursor::new(harness.package.clone());
    reader.seek(std::io::SeekFrom::End(0)).expect("seek to end");
    let tree = stage_package(
        &harness.layout,
        &harness.spec,
        &harness.manifest,
        &mut reader,
        &ExtractionLimits::default(),
        now(),
    )
    .unwrap();
    assert!(tree.root().join("payload").is_file());
    assert_eq!(harness.last_phase(), Phase::Staged);
}

#[test]
fn rejects_a_missing_download_package() {
    let harness = Harness::new(build_zip(&[ZipEntry::file(b"payload", b"data")]));
    let error = stage_package_from_path(
        &harness.layout,
        &harness.spec,
        &harness.manifest,
        &ExtractionLimits::default(),
        now(),
    )
    .unwrap_err();
    assert!(matches!(error, UpdaterError::StagingRejected { .. }));
}

#[test]
fn rejects_a_symlinked_download_package() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let harness = Harness::new(build_zip(&[ZipEntry::file(b"payload", b"data")]));
        let path = harness
            .layout
            .download_package_path(harness.spec.operation_id());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let target = harness.dir.path().join("target.zip");
        fs::write(&target, &harness.package).unwrap();
        symlink(&target, &path).unwrap();
        let error = stage_package_from_path(
            &harness.layout,
            &harness.spec,
            &harness.manifest,
            &ExtractionLimits::default(),
            now(),
        )
        .unwrap_err();
        assert!(matches!(error, UpdaterError::StagingRejected { .. }));
    }
}

#[test]
fn rejects_a_manifest_that_does_not_match_the_operation() {
    let package = build_zip(&[ZipEntry::file(b"payload", b"data")]);
    let harness = Harness::new(package.clone());
    let other_digest = Sha256Digest::hash(b"different");
    let other_manifest = manifest_for(&package, "0.1.0", other_digest);
    let mut reader = Cursor::new(package);
    let error = stage_package(
        &harness.layout,
        &harness.spec,
        &other_manifest,
        &mut reader,
        &ExtractionLimits::default(),
        now(),
    )
    .unwrap_err();
    assert_eq!(
        error,
        UpdaterError::PackageIdentityMismatch {
            field: "package hash"
        }
    );
}

#[test]
fn rejects_a_journal_context_mismatch() {
    let package = build_zip(&[ZipEntry::file(b"payload", b"data")]);
    let harness = Harness::new(package.clone());
    let other_spec = OperationSpec::new(
        harness.spec.operation_id().clone(),
        harness.spec.idempotency_key().clone(),
        harness.spec.release().clone(),
        Some(Version::parse("0.0.9").unwrap()),
        now(),
    );
    let mut reader = Cursor::new(package);
    let error = stage_package(
        &harness.layout,
        &other_spec,
        &harness.manifest,
        &mut reader,
        &ExtractionLimits::default(),
        now(),
    )
    .unwrap_err();
    assert!(matches!(error, UpdaterError::InvalidJournalRecord { .. }));
}

#[test]
fn requires_downloaded_or_verified_journal_phase() {
    let package = build_zip(&[ZipEntry::file(b"payload", b"data")]);
    let harness = Harness::with_phases(package, &[Phase::Intent]);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(
        error,
        UpdaterError::InvalidOperationTransition { .. }
    ));
    assert_eq!(harness.last_phase(), Phase::Intent);
}

#[test]
fn records_verified_before_archive_validation() {
    let package = build_zip_with_comment(&[ZipEntry::file(b"payload", b"data")], "comment");
    let harness = Harness::new(package);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(error, UpdaterError::ArchiveInvalid { .. }));
    assert_eq!(harness.last_phase(), Phase::Verified);
    assert!(!harness.staging_path().exists());
}

#[test]
fn rejects_package_hash_mismatch_before_parsing() {
    let package = build_zip(&[ZipEntry::file(b"payload", b"data")]);
    let mut harness = Harness::new(package);
    harness.package[30] ^= 0xff;
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(error, UpdaterError::PackageHashMismatch { .. }));
    assert_eq!(harness.last_phase(), Phase::Downloaded);
}

#[test]
fn rejects_a_pre_existing_staging_directory() {
    let harness = Harness::new(build_zip(&[ZipEntry::file(b"payload", b"data")]));
    let staging = harness.staging_path();
    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join("keep"), b"keep").unwrap();
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(error, UpdaterError::StagingRejected { .. }));
    assert!(staging.join("keep").is_file());
}

#[test]
fn rejects_a_symlinked_staging_ancestor() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let harness = Harness::new(build_zip(&[ZipEntry::file(b"payload", b"data")]));
        let staging = harness.staging_path();
        let parent = staging.parent().unwrap();
        let target = harness.dir.path().join("staging-target");
        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(parent.parent().unwrap()).unwrap();
        symlink(&target, parent).unwrap();
        let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
        assert!(matches!(error, UpdaterError::StagingRejected { .. }));
    }
}

#[test]
fn rejects_traversal_absolute_and_windows_paths() {
    for name in [
        &b"../evil"[..],
        &b"a/../../evil"[..],
        &b"/etc/passwd"[..],
        &b"C:/Windows"[..],
        &b"\\\\server\\share"[..],
        &b"a\\b"[..],
        &b"a//b"[..],
        &b"./a"[..],
        &b"a/../b"[..],
        &b"a./b"[..],
        &b"con"[..],
        &b"con.txt"[..],
        &b"LPT9"[..],
        &b"a\0b"[..],
        &b"a\nb"[..],
        &b"trailing "[..],
    ] {
        let (_, error) = stage_err(&[ZipEntry::file(name, b"")]);
        assert!(
            matches!(error, UpdaterError::ArchivePathRejected { .. }),
            "{name:?} produced {error:?}"
        );
    }
    let (_, error) = stage_err(&[ZipEntry::file(&[0xff, b'a'], b"")]);
    assert!(matches!(error, UpdaterError::ArchivePathRejected { .. }));
}

#[test]
fn rejects_file_and_directory_type_confusion() {
    let (_, error) = stage_err(&[ZipEntry::file(b"a/", b"").mode(0o100_644)]);
    assert!(matches!(error, UpdaterError::ArchiveEntryRejected { .. }));

    let (_, error) = stage_err(&[ZipEntry::file(b"a", b"").mode(0o040_755)]);
    assert!(matches!(error, UpdaterError::ArchiveEntryRejected { .. }));
}

#[test]
fn rejects_links_and_special_modes() {
    for mode in [
        0o120_777, // symbolic link
        0o020_644, // character device
        0o060_644, // block device
        0o010_644, // FIFO
        0o140_644, // socket
        0o104_755, // setuid
        0o102_755, // setgid
        0o101_755, // sticky
        0o100_600, // non-normalized file mode
        0o040_700, // non-normalized directory mode
    ] {
        let (_, error) = stage_err(&[ZipEntry::file(b"entry", b"").mode(mode)]);
        assert!(
            matches!(error, UpdaterError::ArchiveEntryRejected { .. }),
            "mode {mode:o} produced {error:?}"
        );
    }
}

#[test]
fn normalizes_missing_and_executable_modes() {
    let (_, tree) = stage_ok(&[
        ZipEntry::file(b"plain", b"x"),
        ZipEntry::file(b"tool", b"y").mode(0o100_755),
    ]);
    let plain = tree
        .entries()
        .iter()
        .find(|entry| entry.path().as_str() == "plain")
        .unwrap();
    let tool = tree
        .entries()
        .iter()
        .find(|entry| entry.path().as_str() == "tool")
        .unwrap();
    assert_eq!(plain.mode(), 0o644);
    assert_eq!(tool.mode(), 0o755);

    // A DOS-style archive with no Unix mode receives deterministic defaults.
    let mut bytes = build_zip(&[ZipEntry::file(b"plain", b"x"), ZipEntry::dir(b"dir/")]);
    for entry in central_entries(&bytes) {
        patch_u32(&mut bytes, entry.external_attributes_offset, 0);
    }
    let harness = Harness::new(bytes);
    let tree = harness.stage(&ExtractionLimits::default()).unwrap();
    let plain = tree
        .entries()
        .iter()
        .find(|entry| entry.path().as_str() == "plain")
        .unwrap();
    let dir = tree
        .entries()
        .iter()
        .find(|entry| entry.path().as_str() == "dir")
        .unwrap();
    assert_eq!(plain.mode(), 0o644);
    assert_eq!(dir.mode(), 0o755);
}

#[test]
fn rejects_duplicate_and_case_colliding_names() {
    let mut bytes = build_zip(&[
        ZipEntry::file(b"a.txt", b"a"),
        ZipEntry::file(b"b.txt", b"b"),
    ]);
    let entries = central_entries(&bytes);
    let second = &entries[1];
    assert_eq!(second.name_len, 5);
    patch_name(&mut bytes, second, b"a.txt");
    let harness = Harness::new(bytes);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(error, UpdaterError::ArchiveInvalid { .. }));

    let (_, error) = stage_err(&[
        ZipEntry::file(b"A.txt", b"a"),
        ZipEntry::file(b"a.txt", b"b"),
    ]);
    assert!(matches!(error, UpdaterError::ArchiveEntryRejected { .. }));

    // Two files whose implicit parent directories differ only by case.
    let (_, error) = stage_err(&[
        ZipEntry::file(b"Dir/a.txt", b"a"),
        ZipEntry::file(b"dir/b.txt", b"b"),
    ]);
    assert!(matches!(error, UpdaterError::ArchiveEntryRejected { .. }));

    // An explicit directory and an implicit directory that differ only by case.
    let (_, error) = stage_err(&[ZipEntry::dir(b"Dir/"), ZipEntry::file(b"dir/a.txt", b"a")]);
    assert!(matches!(error, UpdaterError::ArchiveEntryRejected { .. }));
}

#[test]
fn rejects_file_directory_and_ancestor_conflicts() {
    let (_, error) = stage_err(&[ZipEntry::file(b"a", b"file"), ZipEntry::dir(b"a/")]);
    assert!(matches!(error, UpdaterError::ArchiveEntryRejected { .. }));

    let (_, error) = stage_err(&[
        ZipEntry::file(b"a", b"file"),
        ZipEntry::file(b"a/b", b"nested"),
    ]);
    assert!(matches!(error, UpdaterError::ArchiveEntryRejected { .. }));

    let (_, error) = stage_err(&[
        ZipEntry::file(b"a/b", b"nested"),
        ZipEntry::file(b"a", b"file"),
    ]);
    assert!(matches!(error, UpdaterError::ArchiveEntryRejected { .. }));
}

#[test]
fn enforces_entry_count_and_size_bounds() {
    let entries = [ZipEntry::file(b"a", b"a"), ZipEntry::file(b"b", b"b")];
    let limits = ExtractionLimits {
        max_entries: 1,
        ..ExtractionLimits::default()
    };
    let (_, result) = stage_with(&entries, limits);
    assert!(matches!(
        result.unwrap_err(),
        UpdaterError::ArchiveLimitExceeded {
            limit: "entry count",
            ..
        }
    ));

    let limits = ExtractionLimits {
        max_entry_uncompressed_bytes: 1,
        max_total_uncompressed_bytes: 4,
        ..ExtractionLimits::default()
    };
    let (_, result) = stage_with(&[ZipEntry::file(b"a", b"ab")], limits);
    assert!(matches!(
        result.unwrap_err(),
        UpdaterError::ArchiveLimitExceeded {
            limit: "entry uncompressed bytes",
            ..
        }
    ));

    let limits = ExtractionLimits {
        max_entry_uncompressed_bytes: 3,
        max_total_uncompressed_bytes: 3,
        ..ExtractionLimits::default()
    };
    let (_, result) = stage_with(
        &[ZipEntry::file(b"a", b"ab"), ZipEntry::file(b"b", b"ab")],
        limits,
    );
    assert!(matches!(
        result.unwrap_err(),
        UpdaterError::ArchiveLimitExceeded {
            limit: "total uncompressed bytes",
            ..
        }
    ));
}

#[test]
fn enforces_compression_ratio_bounds() {
    let payload = vec![0u8; 8 * 1024];
    let limits = ExtractionLimits {
        max_entry_ratio: 1,
        ..ExtractionLimits::default()
    };
    let (_, result) = stage_with(&[ZipEntry::file(b"zeros", &payload)], limits);
    assert!(matches!(
        result.unwrap_err(),
        UpdaterError::ArchiveLimitExceeded {
            limit: "entry compression ratio",
            ..
        }
    ));
}

#[test]
fn enforces_path_length_component_and_depth_bounds() {
    let limits = ExtractionLimits {
        max_path_bytes: 8,
        max_component_bytes: 8,
        ..ExtractionLimits::default()
    };
    let (_, result) = stage_with(&[ZipEntry::file(b"0123456789", b"")], limits);
    assert!(matches!(
        result.unwrap_err(),
        UpdaterError::ArchiveLimitExceeded {
            limit: "path bytes",
            ..
        }
    ));

    let limits = ExtractionLimits {
        max_component_bytes: 4,
        ..ExtractionLimits::default()
    };
    let (_, result) = stage_with(&[ZipEntry::file(b"012345", b"")], limits);
    assert!(matches!(
        result.unwrap_err(),
        UpdaterError::ArchiveLimitExceeded {
            limit: "path component bytes",
            ..
        }
    ));

    let limits = ExtractionLimits {
        max_depth: 2,
        ..ExtractionLimits::default()
    };
    let (_, result) = stage_with(&[ZipEntry::file(b"a/b/c", b"")], limits);
    assert!(matches!(
        result.unwrap_err(),
        UpdaterError::ArchiveLimitExceeded {
            limit: "path depth",
            ..
        }
    ));
}

#[test]
fn enforces_the_time_bound() {
    let limits = ExtractionLimits {
        max_duration: Duration::from_nanos(1),
        ..ExtractionLimits::default()
    };
    let (_, result) = stage_with(&[ZipEntry::file(b"a", b"a")], limits);
    assert!(matches!(
        result.unwrap_err(),
        UpdaterError::ArchiveTimeout { .. }
    ));
}

#[test]
fn rejects_header_confusion() {
    // Local and central compression methods disagree.
    let mut bytes = build_zip(&[ZipEntry::file(b"a", b"data")]);
    let entries = central_entries(&bytes);
    patch_u16(&mut bytes, entries[0].local_offset + 8, 0);
    let harness = Harness::new(bytes);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(error, UpdaterError::ArchiveInvalid { .. }));

    // Two central entries point at the same local header.
    let mut bytes = build_zip(&[ZipEntry::file(b"a", b"a"), ZipEntry::file(b"b", b"b")]);
    let entries = central_entries(&bytes);
    patch_u32(
        &mut bytes,
        entries[1].central_offset + 42,
        entries[0].local_offset as u32,
    );
    let harness = Harness::new(bytes);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(error, UpdaterError::ArchiveInvalid { .. }));

    // A local header that the central directory does not list.
    let mut bytes = build_zip(&[ZipEntry::file(b"a", b"a"), ZipEntry::file(b"b", b"b")]);
    remove_central_entry(&mut bytes, 1);
    let harness = Harness::new(bytes);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(error, UpdaterError::ArchiveInvalid { .. }));

    // Prepended data.
    let mut bytes = b"JUNK".to_vec();
    bytes.extend(build_zip(&[ZipEntry::file(b"a", b"a")]));
    let harness = Harness::new(bytes);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(error, UpdaterError::ArchiveInvalid { .. }));

    // A non-empty archive comment.
    let bytes = build_zip_with_comment(&[ZipEntry::file(b"a", b"a")], "comment");
    let harness = Harness::new(bytes);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(error, UpdaterError::ArchiveInvalid { .. }));
}

#[test]
fn rejects_encrypted_and_unsupported_entries() {
    let mut bytes = build_zip(&[ZipEntry::file(b"a", b"data")]);
    let entries = central_entries(&bytes);
    let entry = &entries[0];
    patch_u16(&mut bytes, entry.local_offset + 6, 1);
    patch_u16(&mut bytes, entry.central_offset + 8, 1);
    let harness = Harness::new(bytes);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(error, UpdaterError::ArchiveEntryRejected { .. }));

    let mut bytes = build_zip(&[ZipEntry::stored(b"a", b"data")]);
    let entries = central_entries(&bytes);
    let entry = &entries[0];
    patch_u16(&mut bytes, entry.local_offset + 8, 99);
    patch_u16(&mut bytes, entry.central_offset + 10, 99);
    let harness = Harness::new(bytes);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(
        error,
        UpdaterError::ArchiveEntryRejected { .. } | UpdaterError::ArchiveInvalid { .. }
    ));
}

#[test]
fn stages_a_zip64_archive() {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer.set_raw_zip64_extensible_data_sector(vec![0u8; 4].into_boxed_slice());
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default())
        .name(b"payload");
    writer.start_file_with_options(options).unwrap();
    writer.write_all(b"zip64-data").unwrap();
    let bytes = writer.finish().unwrap().into_inner();

    let harness = Harness::new(bytes);
    let tree = harness.stage(&ExtractionLimits::default()).unwrap();
    assert_eq!(harness.last_phase(), Phase::Staged);
    assert_eq!(
        fs::read(tree.root().join("payload")).unwrap(),
        b"zip64-data"
    );
}

#[test]
fn rejects_zip64_sentinels_without_a_locator() {
    let mut bytes = build_zip(&[ZipEntry::file(b"a", b"a")]);
    let eocd = bytes.len() - 22;
    patch_u16(&mut bytes, eocd + 10, u16::MAX);
    patch_u16(&mut bytes, eocd + 8, u16::MAX);
    let harness = Harness::new(bytes);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(error, UpdaterError::ArchiveInvalid { .. }));
}

#[test]
fn rejects_multi_disk_and_empty_archives() {
    let mut bytes = build_zip(&[ZipEntry::file(b"a", b"a")]);
    let eocd = bytes.len() - 22;
    patch_u16(&mut bytes, eocd + 4, 1);
    let harness = Harness::new(bytes);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(error, UpdaterError::ArchiveInvalid { .. }));

    let harness = Harness::new(build_zip(&[]));
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(error, UpdaterError::ArchiveInvalid { .. }));
}

#[test]
fn rejects_invalid_extraction_limits() {
    let package = build_zip(&[ZipEntry::file(b"a", b"a")]);
    let harness = Harness::new(package);
    for mutate in [
        |limits: &mut ExtractionLimits| limits.max_entries = 0,
        |limits: &mut ExtractionLimits| limits.max_total_uncompressed_bytes = 0,
        |limits: &mut ExtractionLimits| limits.max_entry_uncompressed_bytes = 0,
        |limits: &mut ExtractionLimits| {
            limits.max_entry_uncompressed_bytes = limits.max_total_uncompressed_bytes + 1;
        },
        |limits: &mut ExtractionLimits| limits.max_entry_ratio = 0,
        |limits: &mut ExtractionLimits| limits.max_total_ratio = 0,
        |limits: &mut ExtractionLimits| limits.max_path_bytes = 0,
        |limits: &mut ExtractionLimits| limits.max_component_bytes = 0,
        |limits: &mut ExtractionLimits| {
            limits.max_component_bytes = limits.max_path_bytes + 1;
        },
        |limits: &mut ExtractionLimits| limits.max_depth = 0,
        |limits: &mut ExtractionLimits| limits.max_duration = Duration::ZERO,
    ] {
        let mut limits = ExtractionLimits::default();
        mutate(&mut limits);
        assert!(matches!(
            harness.stage(&limits).unwrap_err(),
            UpdaterError::InvalidExtractionLimits { .. }
        ));
    }
}

#[test]
fn removes_the_partial_tree_when_the_journal_cannot_record_staged() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let harness = Harness::with_phases(
            build_zip(&[ZipEntry::file(b"payload", b"data")]),
            &[Phase::Intent, Phase::Downloaded, Phase::Verified],
        );
        let journal = harness.layout.journal_path();
        fs::set_permissions(&journal, fs::Permissions::from_mode(0o400)).unwrap();
        let result = harness.stage(&ExtractionLimits::default());
        fs::set_permissions(&journal, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(result.is_err());
        assert!(!harness.staging_path().exists());
        assert_eq!(harness.last_phase(), Phase::Verified);
    }
}

#[test]
fn removes_the_partial_tree_when_entry_data_is_corrupt() {
    let mut bytes = build_zip(&[ZipEntry::stored(b"payload", b"corrupt-me")]);
    let entries = central_entries(&bytes);
    let entry = &entries[0];
    let data_start = entry.local_offset + 30 + entry.name_len;
    // Flip a stored data byte so the CRC check fails after extraction starts.
    bytes[data_start] ^= 0xff;
    let harness = Harness::new(bytes);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(
        error,
        UpdaterError::ArchiveInvalid { .. } | UpdaterError::Io { .. }
    ));
    assert!(!harness.staging_path().exists());
    assert_eq!(harness.last_phase(), Phase::Verified);
}

#[test]
fn rejects_a_file_that_decompresses_to_more_than_declared() {
    let mut bytes = build_zip(&[ZipEntry::file(b"payload", b"0123456789")]);
    let entries = central_entries(&bytes);
    let entry = &entries[0];
    patch_u32(&mut bytes, entry.local_offset + 22, 4);
    patch_u32(&mut bytes, entry.central_offset + 24, 4);
    let harness = Harness::new(bytes);
    let error = harness.stage(&ExtractionLimits::default()).unwrap_err();
    assert!(matches!(
        error,
        UpdaterError::ArchiveInvalid { .. } | UpdaterError::Io { .. }
    ));
    assert!(!harness.staging_path().exists());
}

struct CentralEntry {
    central_offset: usize,
    local_offset: usize,
    name_start: usize,
    name_len: usize,
    external_attributes_offset: usize,
}

fn central_entries(bytes: &[u8]) -> Vec<CentralEntry> {
    assert!(bytes.len() >= 22);
    let eocd = bytes.len() - 22;
    assert_eq!(&bytes[eocd..eocd + 4], b"PK\x05\x06");
    let count = u16::from_le_bytes(bytes[eocd + 10..eocd + 12].try_into().unwrap()) as usize;
    let mut position = u32::from_le_bytes(bytes[eocd + 16..eocd + 20].try_into().unwrap()) as usize;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        assert_eq!(&bytes[position..position + 4], b"PK\x01\x02");
        let name_len =
            u16::from_le_bytes(bytes[position + 28..position + 30].try_into().unwrap()) as usize;
        let extra_len =
            u16::from_le_bytes(bytes[position + 30..position + 32].try_into().unwrap()) as usize;
        let comment_len =
            u16::from_le_bytes(bytes[position + 32..position + 34].try_into().unwrap()) as usize;
        let local_offset =
            u32::from_le_bytes(bytes[position + 42..position + 46].try_into().unwrap()) as usize;
        entries.push(CentralEntry {
            central_offset: position,
            local_offset,
            name_start: position + 46,
            name_len,
            external_attributes_offset: position + 38,
        });
        position += 46 + name_len + extra_len + comment_len;
    }
    entries
}

fn patch_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn patch_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn patch_name(bytes: &mut [u8], entry: &CentralEntry, name: &[u8]) {
    assert_eq!(entry.name_len, name.len());
    bytes[entry.name_start..entry.name_start + name.len()].copy_from_slice(name);
    let local_name = entry.local_offset + 30;
    bytes[local_name..local_name + name.len()].copy_from_slice(name);
}

fn remove_central_entry(bytes: &mut Vec<u8>, index: usize) {
    let entries = central_entries(bytes);
    let target = &entries[index];
    let end = if index + 1 < entries.len() {
        entries[index + 1].central_offset
    } else {
        bytes.len() - 22
    };
    let removed = end - target.central_offset;
    bytes.drain(target.central_offset..end);

    let eocd = bytes.len() - 22;
    let count = u16::from_le_bytes(bytes[eocd + 10..eocd + 12].try_into().unwrap());
    patch_u16(bytes, eocd + 8, count - 1);
    patch_u16(bytes, eocd + 10, count - 1);
    let size = u32::from_le_bytes(bytes[eocd + 12..eocd + 16].try_into().unwrap());
    patch_u32(bytes, eocd + 12, size - u32::try_from(removed).unwrap());
}

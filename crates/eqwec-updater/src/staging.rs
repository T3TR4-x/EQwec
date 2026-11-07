//! Bounded ZIP package validation and staging.
//!
//! This module implements the package archive v1 rules in
//! `docs/product-specs/eqwec-hub-schemas.md` §11 and ADR-0012. It verifies
//! the package on the same open handle that it parses, validates every entry
//! before writing anything, extracts into a fresh staging directory, syncs
//! the tree, verifies the result, and only then appends the `staged` journal
//! record.
//!
//! The caller owns the download transport, the profile lock, and the decision
//! to activate. A failure leaves the journal at `verified` and removes the
//! partial staging tree; startup recovery discards the operation.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eqwec_security::{SafeRelativePath, Sha256Digest};
use sha2::{Digest, Sha256};
use zip::{CompressionMethod, ZipArchive, ZipReadOptions};

use crate::activation::{OperationSpec, VerifiedRelease, record_phase};
use crate::atomic;
use crate::error::UpdaterError;
use crate::journal::{JournalContext, Phase, read_journal};
use crate::layout::InstallLayout;
use crate::package::digest_from_hasher;
use crate::time::Timestamp;
use crate::verify::VerifiedManifest;

/// Maximum number of archive entries, including directories.
pub const MAX_ARCHIVE_ENTRIES: usize = 100_000;

/// Maximum total declared uncompressed bytes in one archive.
pub const MAX_ARCHIVE_TOTAL_UNCOMPRESSED_BYTES: u64 = 16 * 1024 * 1024 * 1024;

/// Maximum declared uncompressed bytes for one archive entry.
pub const MAX_ARCHIVE_ENTRY_UNCOMPRESSED_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/// Maximum declared compression ratio for one archive entry.
pub const MAX_ARCHIVE_ENTRY_RATIO: u64 = 200;

/// Maximum total declared compression ratio for one archive.
pub const MAX_ARCHIVE_TOTAL_RATIO: u64 = 100;

/// Maximum archive entry path length in bytes.
pub const MAX_ARCHIVE_PATH_BYTES: usize = 1_024;

/// Maximum archive entry path component length in bytes.
pub const MAX_ARCHIVE_COMPONENT_BYTES: usize = 255;

/// Maximum archive entry path depth in components.
pub const MAX_ARCHIVE_DEPTH: usize = 32;

/// Default extraction time bound.
pub const DEFAULT_EXTRACTION_TIMEOUT: Duration = Duration::from_secs(600);

const READ_BUFFER_BYTES: usize = 64 * 1024;
const UNIX_FILE_TYPE_MASK: u32 = 0o170_000;
const UNIX_REGULAR_FILE: u32 = 0o100_000;
const UNIX_DIRECTORY: u32 = 0o040_000;
const UNIX_SPECIAL_BITS: u32 = 0o7_000;
const UNIX_PERMISSION_BITS: u32 = 0o777;
const FILE_MODE: u32 = 0o644;
const EXECUTABLE_MODE: u32 = 0o755;
const DIRECTORY_MODE: u32 = 0o755;

/// Bounds applied while validating and extracting a package archive.
///
/// [`ExtractionLimits::default`] is the normative v1 policy. A caller may
/// lower a bound. Raising a bound requires an explicit policy decision and
/// must still fit the declared package size and available disk.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtractionLimits {
    /// Maximum number of archive entries, including directories.
    pub max_entries: usize,
    /// Maximum total declared uncompressed bytes.
    pub max_total_uncompressed_bytes: u64,
    /// Maximum declared uncompressed bytes for one entry.
    pub max_entry_uncompressed_bytes: u64,
    /// Maximum declared compression ratio for one entry.
    pub max_entry_ratio: u64,
    /// Maximum total declared compression ratio.
    pub max_total_ratio: u64,
    /// Maximum path length in bytes.
    pub max_path_bytes: usize,
    /// Maximum path component length in bytes.
    pub max_component_bytes: usize,
    /// Maximum path depth in components.
    pub max_depth: usize,
    /// Maximum extraction time.
    pub max_duration: Duration,
}

impl ExtractionLimits {
    /// Returns the normative v1 defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_entries: MAX_ARCHIVE_ENTRIES,
            max_total_uncompressed_bytes: MAX_ARCHIVE_TOTAL_UNCOMPRESSED_BYTES,
            max_entry_uncompressed_bytes: MAX_ARCHIVE_ENTRY_UNCOMPRESSED_BYTES,
            max_entry_ratio: MAX_ARCHIVE_ENTRY_RATIO,
            max_total_ratio: MAX_ARCHIVE_TOTAL_RATIO,
            max_path_bytes: MAX_ARCHIVE_PATH_BYTES,
            max_component_bytes: MAX_ARCHIVE_COMPONENT_BYTES,
            max_depth: MAX_ARCHIVE_DEPTH,
            max_duration: DEFAULT_EXTRACTION_TIMEOUT,
        }
    }

    /// Validates the bounds.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidExtractionLimits`] when a bound is zero
    /// or internally inconsistent.
    pub fn validate(&self) -> Result<(), UpdaterError> {
        if self.max_entries == 0 {
            return Err(invalid_limits("max_entries must be at least 1"));
        }
        if self.max_total_uncompressed_bytes == 0 {
            return Err(invalid_limits(
                "max_total_uncompressed_bytes must be at least 1",
            ));
        }
        if self.max_entry_uncompressed_bytes == 0 {
            return Err(invalid_limits(
                "max_entry_uncompressed_bytes must be at least 1",
            ));
        }
        if self.max_entry_uncompressed_bytes > self.max_total_uncompressed_bytes {
            return Err(invalid_limits(
                "max_entry_uncompressed_bytes must not exceed the total bound",
            ));
        }
        if self.max_entry_ratio == 0 {
            return Err(invalid_limits("max_entry_ratio must be at least 1"));
        }
        if self.max_total_ratio == 0 {
            return Err(invalid_limits("max_total_ratio must be at least 1"));
        }
        if self.max_path_bytes == 0 {
            return Err(invalid_limits("max_path_bytes must be at least 1"));
        }
        if self.max_component_bytes == 0 {
            return Err(invalid_limits("max_component_bytes must be at least 1"));
        }
        if self.max_component_bytes > self.max_path_bytes {
            return Err(invalid_limits(
                "max_component_bytes must not exceed max_path_bytes",
            ));
        }
        if self.max_depth == 0 {
            return Err(invalid_limits("max_depth must be at least 1"));
        }
        if self.max_duration.is_zero() {
            return Err(invalid_limits("max_duration must be greater than zero"));
        }
        Ok(())
    }
}

impl Default for ExtractionLimits {
    fn default() -> Self {
        Self::new()
    }
}

/// The kind of one staged tree entry.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StagedEntryKind {
    /// A regular file.
    File,
    /// A directory.
    Directory,
}

/// One entry in a verified staging tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedEntry {
    path: SafeRelativePath,
    kind: StagedEntryKind,
    mode: u32,
    size: u64,
    sha256: Option<Sha256Digest>,
}

impl StagedEntry {
    /// Returns the relative path.
    #[must_use]
    pub fn path(&self) -> &SafeRelativePath {
        &self.path
    }

    /// Returns the entry kind.
    #[must_use]
    pub const fn kind(&self) -> StagedEntryKind {
        self.kind
    }

    /// Returns the normalized Unix mode (`0644`, `0755`).
    #[must_use]
    pub const fn mode(&self) -> u32 {
        self.mode
    }

    /// Returns the file size in bytes; directories are zero.
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Returns the SHA-256 of a file; directories have no digest.
    #[must_use]
    pub const fn sha256(&self) -> Option<Sha256Digest> {
        self.sha256
    }
}

/// A complete, durable staging tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedTree {
    root: PathBuf,
    entries: Vec<StagedEntry>,
}

impl StagedTree {
    /// Returns the staging root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the verified entries in path order.
    #[must_use]
    pub fn entries(&self) -> &[StagedEntry] {
        &self.entries
    }
}

/// Verifies and stages a package from an open reader.
///
/// The reader is rewound and hashed before parsing, so the bytes that are
/// parsed are the bytes that were verified. The journal must have `downloaded`
/// or `verified` as the last phase for this operation, and its context must
/// match the operation identity.
///
/// # Errors
///
/// Returns a typed error for a journal mismatch, package mismatch, malformed
/// or forbidden archive, exceeded bound, unsafe staging path, I/O failure, or
/// failed tree verification. A failure removes the partial staging tree and
/// leaves the journal at `verified`.
pub fn stage_package<R: Read + Seek>(
    layout: &InstallLayout,
    spec: &OperationSpec,
    manifest: &VerifiedManifest,
    reader: &mut R,
    limits: &ExtractionLimits,
    now: Timestamp,
) -> Result<StagedTree, UpdaterError> {
    limits.validate()?;
    ensure_manifest_matches_release(manifest, spec.release())?;
    let last_phase = ensure_journal_ready(layout, spec)?;

    // The package is defined to start at byte zero. Rewind before hashing so
    // the bytes verified are exactly the bytes parsed below.
    reader.seek(SeekFrom::Start(0)).map_err(atomic::io_error)?;
    let package = manifest.verify_package_reader(&mut *reader)?;
    if package.sha256() != spec.release().package_sha256() {
        return Err(UpdaterError::PackageIdentityMismatch {
            field: "package hash",
        });
    }
    if last_phase == Phase::Downloaded {
        record_phase(layout, spec, Phase::Verified, now)?;
    }

    let deadline = Instant::now()
        .checked_add(limits.max_duration)
        .ok_or_else(|| invalid_limits("max_duration overflows the monotonic clock"))?;
    check_deadline(deadline, limits.max_duration)?;

    reader.seek(SeekFrom::Start(0)).map_err(atomic::io_error)?;
    preflight_eocd(reader)?;
    reader.seek(SeekFrom::Start(0)).map_err(atomic::io_error)?;
    let mut archive = ZipArchive::new(reader).map_err(zip_error)?;
    let plan = build_plan(&mut archive, limits, deadline)?;

    let staging = layout.staging_dir(spec.release().product_id(), spec.operation_id());
    prepare_staging(layout, &staging)?;
    let entries = match extract_and_verify(&mut archive, &staging, &plan, limits, deadline) {
        Ok(entries) => entries,
        Err(error) => {
            remove_staging(&staging);
            return Err(error);
        }
    };

    if let Err(error) = record_phase(layout, spec, Phase::Staged, now) {
        remove_staging(&staging);
        return Err(error);
    }

    Ok(StagedTree {
        root: staging,
        entries,
    })
}

/// Verifies and stages the package at the operation's download path.
///
/// The path is [`InstallLayout::download_package_path`]. A missing path, a
/// symlink, or a non-regular file is rejected.
///
/// # Errors
///
/// Returns the same errors as [`stage_package`], plus a staging rejection when
/// the package path is missing, a symlink, or not a regular file.
pub fn stage_package_from_path(
    layout: &InstallLayout,
    spec: &OperationSpec,
    manifest: &VerifiedManifest,
    limits: &ExtractionLimits,
    now: Timestamp,
) -> Result<StagedTree, UpdaterError> {
    let path = layout.download_package_path(spec.operation_id());
    let metadata = fs::symlink_metadata(&path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => UpdaterError::StagingRejected {
            reason: format!("package is missing: {}", path.display()),
        },
        _ => atomic::io_error(error),
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(UpdaterError::StagingRejected {
            reason: "package path is not a regular file".to_owned(),
        });
    }
    let mut file = File::open(&path).map_err(atomic::io_error)?;
    stage_package(layout, spec, manifest, &mut file, limits, now)
}

#[derive(Clone, Debug)]
struct PlanEntry {
    path: SafeRelativePath,
    kind: StagedEntryKind,
    mode: u32,
    declared_size: u64,
    compressed_size: u64,
    archive_index: Option<usize>,
}

fn ensure_manifest_matches_release(
    manifest: &VerifiedManifest,
    release: &VerifiedRelease,
) -> Result<(), UpdaterError> {
    let manifest = manifest.manifest();
    if manifest.product_id() != release.product_id() {
        return Err(UpdaterError::PackageIdentityMismatch {
            field: "product_id",
        });
    }
    if manifest.product_version() != release.product_version() {
        return Err(UpdaterError::PackageIdentityMismatch {
            field: "product_version",
        });
    }
    if manifest.channel() != release.channel() {
        return Err(UpdaterError::PackageIdentityMismatch { field: "channel" });
    }
    if manifest.package().sha256() != release.package_sha256() {
        return Err(UpdaterError::PackageIdentityMismatch {
            field: "package hash",
        });
    }
    if manifest.signature().key_id() != release.signer_key_id() {
        return Err(UpdaterError::PackageIdentityMismatch {
            field: "signer_key_id",
        });
    }
    if manifest.signature().key_epoch() != release.key_epoch() {
        return Err(UpdaterError::PackageIdentityMismatch { field: "key_epoch" });
    }
    if manifest.metadata_sequence() != release.metadata_sequence() {
        return Err(UpdaterError::PackageIdentityMismatch {
            field: "metadata_sequence",
        });
    }
    Ok(())
}

fn ensure_journal_ready(
    layout: &InstallLayout,
    spec: &OperationSpec,
) -> Result<Phase, UpdaterError> {
    let contents = read_journal(&layout.journal_path())?;
    if contents.has_unfinished_operation_other_than(spec.operation_id()) {
        return Err(UpdaterError::InvalidOperationTransition {
            expected: "the current operation is the only unfinished one",
            found: "another unfinished operation",
        });
    }
    let record = contents
        .records()
        .iter()
        .rev()
        .find(|record| record.operation_id() == spec.operation_id())
        .ok_or(UpdaterError::InvalidOperationTransition {
            expected: "downloaded or verified",
            found: "none",
        })?;
    match record.phase() {
        Phase::Downloaded | Phase::Verified => {}
        found => {
            return Err(UpdaterError::InvalidOperationTransition {
                expected: "downloaded or verified",
                found: found.as_str(),
            });
        }
    }
    let context = record
        .context()
        .ok_or_else(|| UpdaterError::InvalidJournalRecord {
            reason: "journal record has no context".to_owned(),
        })?;
    ensure_context_matches(context, spec)?;
    Ok(record.phase())
}

fn ensure_context_matches(
    context: &JournalContext,
    spec: &OperationSpec,
) -> Result<(), UpdaterError> {
    let release = spec.release();
    let mismatch = |field: &'static str| UpdaterError::InvalidJournalRecord {
        reason: format!("journal context {field} does not match the operation"),
    };
    if context.from_version() != spec.from_version() {
        return Err(mismatch("from_version"));
    }
    if context.to_version() != Some(release.product_version()) {
        return Err(mismatch("to_version"));
    }
    if context.channel() != release.channel() {
        return Err(mismatch("channel"));
    }
    if context.package_sha256() != release.package_sha256() {
        return Err(mismatch("package_sha256"));
    }
    if context.signer_key_id() != Some(release.signer_key_id()) {
        return Err(mismatch("signer_key_id"));
    }
    if context.key_epoch() != Some(release.key_epoch()) {
        return Err(mismatch("key_epoch"));
    }
    if context.metadata_sequence() != Some(release.metadata_sequence()) {
        return Err(mismatch("metadata_sequence"));
    }
    Ok(())
}

fn preflight_eocd<R: Read + Seek>(reader: &mut R) -> Result<(), UpdaterError> {
    let length = reader.seek(SeekFrom::End(0)).map_err(atomic::io_error)?;
    if length < 22 {
        return Err(archive_invalid("archive is too short"));
    }
    reader
        .seek(SeekFrom::Start(length - 22))
        .map_err(atomic::io_error)?;
    let mut eocd = [0u8; 22];
    reader.read_exact(&mut eocd).map_err(atomic::io_error)?;
    if eocd[0..4] != [0x50, 0x4b, 0x05, 0x06] {
        return Err(archive_invalid(
            "missing end-of-central-directory record or a non-empty comment",
        ));
    }
    if le_u16(&eocd, 20)? != 0 {
        return Err(archive_invalid("archive comment must be empty"));
    }

    let disk_number = le_u16(&eocd, 4)?;
    let central_disk = le_u16(&eocd, 6)?;
    if disk_number != 0 || central_disk != 0 {
        return Err(archive_invalid("multi-disk archives are not allowed"));
    }

    let entries_this_disk = le_u16(&eocd, 8)?;
    let entries_total = le_u16(&eocd, 10)?;
    if entries_this_disk != entries_total {
        return Err(archive_invalid("multi-disk archives are not allowed"));
    }
    let zip32_size = u64::from(le_u32(&eocd, 12)?);
    let zip32_offset = u64::from(le_u32(&eocd, 16)?);
    let may_be_zip64 = entries_total == u16::MAX
        || zip32_size == u64::from(u32::MAX)
        || zip32_offset == u64::from(u32::MAX);

    // The ZIP64 end record and locator sit immediately before the EOCD. A
    // valid ZIP64 archive may keep actual values in the standard EOCD fields
    // instead of sentinels, so presence of the locator decides the layout.
    let locator_position = length.checked_sub(22 + 20);
    if let Some(position) = locator_position
        && read_zip64_end(reader, position)?
    {
        return Ok(());
    }
    if may_be_zip64 {
        return Err(archive_invalid(
            "missing ZIP64 end-of-central-directory locator",
        ));
    }

    let central_end = zip32_offset
        .checked_add(zip32_size)
        .ok_or_else(|| archive_invalid("central directory offset overflow"))?;
    if central_end != length - 22 {
        return Err(archive_invalid(
            "central directory is not contiguous with the end record",
        ));
    }
    Ok(())
}

/// Reads and validates a ZIP64 end record when a locator is present at
/// `locator_position`. Returns `false` when the bytes are not a ZIP64 locator.
fn read_zip64_end<R: Read + Seek>(
    reader: &mut R,
    locator_position: u64,
) -> Result<bool, UpdaterError> {
    reader
        .seek(SeekFrom::Start(locator_position))
        .map_err(atomic::io_error)?;
    let mut locator = [0u8; 20];
    reader.read_exact(&mut locator).map_err(atomic::io_error)?;
    if locator[0..4] != [0x50, 0x4b, 0x06, 0x07] {
        return Ok(false);
    }
    let zip64_disk = le_u32(&locator, 4)?;
    let zip64_offset = le_u64(&locator, 8)?;
    let total_disks = le_u32(&locator, 16)?;
    if zip64_disk != 0 || total_disks != 1 {
        return Err(archive_invalid("multi-disk archives are not allowed"));
    }
    if zip64_offset >= locator_position {
        return Err(archive_invalid("invalid ZIP64 end record offset"));
    }
    reader
        .seek(SeekFrom::Start(zip64_offset))
        .map_err(atomic::io_error)?;
    let mut record = [0u8; 56];
    reader.read_exact(&mut record).map_err(atomic::io_error)?;
    if record[0..4] != [0x50, 0x4b, 0x06, 0x06] {
        return Err(archive_invalid(
            "missing ZIP64 end-of-central-directory record",
        ));
    }
    let record_size = le_u64(&record, 4)?;
    if record_size < 44 {
        return Err(archive_invalid("ZIP64 end record is too short"));
    }
    let record_end = zip64_offset
        .checked_add(12)
        .and_then(|value| value.checked_add(record_size))
        .ok_or_else(|| archive_invalid("ZIP64 end record offset overflow"))?;
    if record_end != locator_position {
        return Err(archive_invalid(
            "ZIP64 end record is not contiguous with its locator",
        ));
    }
    let record_disk = le_u32(&record, 16)?;
    let record_central_disk = le_u32(&record, 20)?;
    if record_disk != 0 || record_central_disk != 0 {
        return Err(archive_invalid("multi-disk archives are not allowed"));
    }
    let record_entries_this_disk = le_u64(&record, 24)?;
    let record_entries_total = le_u64(&record, 32)?;
    if record_entries_this_disk != record_entries_total {
        return Err(archive_invalid("multi-disk archives are not allowed"));
    }
    let central_offset = le_u64(&record, 48)?;
    let central_size = le_u64(&record, 40)?;
    let central_end = central_offset
        .checked_add(central_size)
        .ok_or_else(|| archive_invalid("central directory offset overflow"))?;
    if central_end != zip64_offset {
        return Err(archive_invalid(
            "central directory is not contiguous with the ZIP64 end record",
        ));
    }
    Ok(true)
}

fn build_plan<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    limits: &ExtractionLimits,
    deadline: Instant,
) -> Result<Vec<PlanEntry>, UpdaterError> {
    if archive.offset() != 0 {
        return Err(archive_invalid("archive must start at byte zero"));
    }
    if !archive.comment().is_empty() {
        return Err(archive_invalid("archive comment must be empty"));
    }
    if archive.is_empty() {
        return Err(archive_invalid("archive must contain at least one file"));
    }
    if archive.len() > limits.max_entries {
        return Err(limit_exceeded(
            "entry count",
            limits.max_entries as u64,
            archive.len() as u64,
        ));
    }
    if archive.has_duplicate_names() {
        return Err(archive_invalid("duplicate names in the central directory"));
    }
    archive.validate_header_consistency().map_err(zip_error)?;

    let mut map: BTreeMap<String, PlanEntry> = BTreeMap::new();
    let mut total_declared = 0u64;
    let mut total_compressed = 0u64;
    let mut files = 0usize;

    for index in 0..archive.len() {
        check_deadline(deadline, limits.max_duration)?;
        let entry = archive.by_index_data(index).map_err(zip_error)?;
        if entry.encrypted() {
            return Err(entry_rejected(index, "encrypted entries are not allowed"));
        }
        let compression = entry.compression();
        if !matches!(
            compression,
            CompressionMethod::Stored | CompressionMethod::Deflated
        ) {
            return Err(entry_rejected(index, "unsupported compression method"));
        }
        let (path, is_directory) = validate_entry_name(entry.name_raw(), limits)?;
        let kind = if is_directory {
            StagedEntryKind::Directory
        } else {
            StagedEntryKind::File
        };
        let mode = validate_mode(index, kind, entry.external_attributes() >> 16)?;
        let declared_size = entry.size();
        let compressed_size = entry.compressed_size();

        if kind == StagedEntryKind::Directory {
            if declared_size != 0 || compressed_size != 0 {
                return Err(entry_rejected(
                    index,
                    "directory entry must not declare data",
                ));
            }
        } else {
            files = files.saturating_add(1);
            if declared_size > limits.max_entry_uncompressed_bytes {
                return Err(limit_exceeded(
                    "entry uncompressed bytes",
                    limits.max_entry_uncompressed_bytes,
                    declared_size,
                ));
            }
            if declared_size > 0 && compressed_size == 0 {
                return Err(entry_rejected(
                    index,
                    "compressed size is zero for a non-empty entry",
                ));
            }
            if compression == CompressionMethod::Stored && declared_size != compressed_size {
                return Err(entry_rejected(index, "stored entry sizes are inconsistent"));
            }
            let ratio = declared_size / compressed_size.max(1);
            if ratio > limits.max_entry_ratio {
                return Err(limit_exceeded(
                    "entry compression ratio",
                    limits.max_entry_ratio,
                    ratio,
                ));
            }
        }

        total_declared = total_declared
            .checked_add(declared_size)
            .ok_or_else(|| limit_exceeded("total uncompressed bytes", u64::MAX, u64::MAX))?;
        if total_declared > limits.max_total_uncompressed_bytes {
            return Err(limit_exceeded(
                "total uncompressed bytes",
                limits.max_total_uncompressed_bytes,
                total_declared,
            ));
        }
        total_compressed = total_compressed
            .checked_add(compressed_size)
            .ok_or_else(|| limit_exceeded("total compressed bytes", u64::MAX, u64::MAX))?;

        let key = path.as_str().to_ascii_lowercase();
        if map.contains_key(&key) {
            return Err(entry_rejected(index, "duplicate or case-colliding path"));
        }
        map.insert(
            key,
            PlanEntry {
                path,
                kind,
                mode,
                declared_size,
                compressed_size,
                archive_index: Some(index),
            },
        );
    }

    if files == 0 {
        return Err(archive_invalid("archive must contain at least one file"));
    }
    if total_declared > 0 {
        let ratio = total_declared / total_compressed.max(1);
        if ratio > limits.max_total_ratio {
            return Err(limit_exceeded(
                "total compression ratio",
                limits.max_total_ratio,
                ratio,
            ));
        }
    }

    add_implicit_directories(&mut map, limits)?;
    reject_file_descendants(&map)?;
    if map.len() > limits.max_entries {
        return Err(limit_exceeded(
            "planned entries",
            limits.max_entries as u64,
            map.len() as u64,
        ));
    }

    Ok(map.into_values().collect())
}

fn add_implicit_directories(
    map: &mut BTreeMap<String, PlanEntry>,
    limits: &ExtractionLimits,
) -> Result<(), UpdaterError> {
    // Lowercase key -> actual path. Case-folding must not merge two distinct
    // directories, so an implicit directory that collides with an existing
    // entry or another implicit directory under ASCII case folding is rejected.
    let mut missing: BTreeMap<String, String> = BTreeMap::new();
    for entry in map.values() {
        let path = entry.path.as_str();
        let mut prefix = String::new();
        for component in path.split('/') {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(component);
            if prefix == path {
                break;
            }
            let key = prefix.to_ascii_lowercase();
            match map.get(&key) {
                Some(parent) if parent.kind == StagedEntryKind::File => {
                    return Err(UpdaterError::ArchiveEntryRejected {
                        index: entry.archive_index.unwrap_or(0),
                        reason: "a file cannot be an ancestor of another entry".to_owned(),
                    });
                }
                Some(parent) => {
                    if parent.path.as_str() != prefix {
                        return Err(UpdaterError::ArchiveEntryRejected {
                            index: entry.archive_index.unwrap_or(0),
                            reason: "implicit directory collides after case folding".to_owned(),
                        });
                    }
                }
                None => match missing.get(&key) {
                    Some(existing) if existing != &prefix => {
                        return Err(UpdaterError::ArchiveEntryRejected {
                            index: entry.archive_index.unwrap_or(0),
                            reason: "implicit directory collides after case folding".to_owned(),
                        });
                    }
                    Some(_) => {}
                    None => {
                        missing.insert(key, prefix.clone());
                    }
                },
            }
        }
    }

    for (key, path) in missing {
        let path =
            SafeRelativePath::new(path).map_err(|error| UpdaterError::StagingVerification {
                reason: UpdaterError::bounded_detail(error.to_string()),
            })?;
        map.insert(
            key,
            PlanEntry {
                path,
                kind: StagedEntryKind::Directory,
                mode: DIRECTORY_MODE,
                declared_size: 0,
                compressed_size: 0,
                archive_index: None,
            },
        );
    }

    if map.len() > limits.max_entries {
        return Err(limit_exceeded(
            "planned entries",
            limits.max_entries as u64,
            map.len() as u64,
        ));
    }
    Ok(())
}

fn reject_file_descendants(map: &BTreeMap<String, PlanEntry>) -> Result<(), UpdaterError> {
    for (key, entry) in map {
        if entry.kind != StagedEntryKind::File {
            continue;
        }
        let prefix = format!("{key}/");
        if let Some((next, _)) = map.range(prefix.clone()..).next()
            && next.starts_with(&prefix)
        {
            return Err(UpdaterError::ArchiveEntryRejected {
                index: entry.archive_index.unwrap_or(0),
                reason: "a file cannot be an ancestor of another entry".to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_entry_name(
    raw: &[u8],
    limits: &ExtractionLimits,
) -> Result<(SafeRelativePath, bool), UpdaterError> {
    let text = std::str::from_utf8(raw).map_err(|_| {
        path_rejected(
            &String::from_utf8_lossy(raw),
            "entry name is not valid UTF-8",
        )
    })?;
    if text.is_empty() {
        return Err(path_rejected(text, "entry name is empty"));
    }
    if text.len() > limits.max_path_bytes {
        return Err(limit_exceeded(
            "path bytes",
            limits.max_path_bytes as u64,
            text.len() as u64,
        ));
    }
    if text.starts_with('/') {
        return Err(path_rejected(text, "absolute paths are not allowed"));
    }
    let is_directory = text.ends_with('/');
    let path = if is_directory {
        &text[..text.len() - 1]
    } else {
        text
    };
    if path.is_empty() {
        return Err(path_rejected(text, "entry name is empty"));
    }
    if path.ends_with('/') {
        return Err(path_rejected(text, "empty path component"));
    }
    for byte in path.bytes() {
        let allowed = byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/');
        if !allowed {
            let reason = if byte == b'\\' {
                "backslash is not allowed"
            } else if byte == b':' {
                "colon is not allowed"
            } else if byte < 0x20 || byte == 0x7f {
                "control characters are not allowed"
            } else {
                "character is not allowed"
            };
            return Err(path_rejected(text, reason));
        }
    }

    let components: Vec<&str> = path.split('/').collect();
    if components.len() > limits.max_depth {
        return Err(limit_exceeded(
            "path depth",
            limits.max_depth as u64,
            components.len() as u64,
        ));
    }
    for component in &components {
        if component.is_empty() {
            return Err(path_rejected(text, "empty path component"));
        }
        if *component == "." || *component == ".." {
            return Err(path_rejected(text, "dot path component is not allowed"));
        }
        if component.len() > limits.max_component_bytes {
            return Err(limit_exceeded(
                "path component bytes",
                limits.max_component_bytes as u64,
                component.len() as u64,
            ));
        }
        if component.ends_with('.') {
            return Err(path_rejected(
                text,
                "path component must not end with a dot",
            ));
        }
        if is_windows_reserved(component) {
            return Err(path_rejected(text, "Windows reserved device name"));
        }
    }

    let path = SafeRelativePath::new(path.to_owned())
        .map_err(|_| path_rejected(text, "path is not portable"))?;
    Ok((path, is_directory))
}

fn is_windows_reserved(component: &str) -> bool {
    let stem = component
        .split_once('.')
        .map_or(component, |(stem, _extension)| stem);
    let stem = stem.to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "CONIN$"
            | "CONOUT$"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

fn validate_mode(index: usize, kind: StagedEntryKind, raw_mode: u32) -> Result<u32, UpdaterError> {
    if raw_mode & UNIX_SPECIAL_BITS != 0 {
        return Err(entry_rejected(
            index,
            "setuid, setgid, and sticky bits are not allowed",
        ));
    }
    let type_bits = raw_mode & UNIX_FILE_TYPE_MASK;
    let permissions = raw_mode & UNIX_PERMISSION_BITS;
    match kind {
        StagedEntryKind::Directory => {
            if type_bits != 0 && type_bits != UNIX_DIRECTORY {
                return Err(entry_rejected(
                    index,
                    "entry is not a regular file or directory",
                ));
            }
            if raw_mode != 0 && permissions != DIRECTORY_MODE {
                return Err(entry_rejected(index, "directory mode must be 0755"));
            }
            Ok(DIRECTORY_MODE)
        }
        StagedEntryKind::File => {
            if type_bits != 0 && type_bits != UNIX_REGULAR_FILE {
                return Err(entry_rejected(
                    index,
                    "entry is not a regular file or directory",
                ));
            }
            match (raw_mode, permissions) {
                (0, _) => Ok(FILE_MODE),
                (_, value) if value == FILE_MODE => Ok(FILE_MODE),
                (_, value) if value == EXECUTABLE_MODE => Ok(EXECUTABLE_MODE),
                _ => Err(entry_rejected(index, "file mode must be 0644 or 0755")),
            }
        }
    }
}

fn prepare_staging(layout: &InstallLayout, staging: &Path) -> Result<(), UpdaterError> {
    ensure_real_directory(layout.base())?;
    ensure_real_directory(&layout.base().join("installations"))?;
    let product_root =
        staging
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| UpdaterError::StagingRejected {
                reason: "staging path has no product root".to_owned(),
            })?;
    ensure_real_directory(product_root)?;
    let staging_parent = staging
        .parent()
        .ok_or_else(|| UpdaterError::StagingRejected {
            reason: "staging path has no parent".to_owned(),
        })?;
    ensure_real_directory(staging_parent)?;
    match fs::symlink_metadata(staging) {
        Ok(_) => {
            return Err(UpdaterError::StagingRejected {
                reason: "staging path already exists".to_owned(),
            });
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(atomic::io_error(error)),
    }
    create_dir_exclusive(staging)?;
    atomic::sync_parent_dir(staging)?;
    Ok(())
}

fn ensure_real_directory(path: &Path) -> Result<(), UpdaterError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(UpdaterError::StagingRejected {
                    reason: format!("{} is not a real directory", path.display()),
                });
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            create_dir_exclusive(path)?;
            atomic::sync_parent_dir(path)?;
            let metadata = fs::symlink_metadata(path).map_err(atomic::io_error)?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(UpdaterError::StagingRejected {
                    reason: format!("{} is not a real directory", path.display()),
                });
            }
            Ok(())
        }
        Err(error) => Err(atomic::io_error(error)),
    }
}

#[cfg(unix)]
fn create_dir_exclusive(path: &Path) -> Result<(), UpdaterError> {
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder.create(path).map_err(map_create_error)
}

#[cfg(not(unix))]
fn create_dir_exclusive(path: &Path) -> Result<(), UpdaterError> {
    fs::create_dir(path).map_err(map_create_error)
}

fn map_create_error(error: std::io::Error) -> UpdaterError {
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        UpdaterError::StagingRejected {
            reason: "staging path already exists".to_owned(),
        }
    } else {
        atomic::io_error(error)
    }
}

fn extract_and_verify<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    staging: &Path,
    plan: &[PlanEntry],
    limits: &ExtractionLimits,
    deadline: Instant,
) -> Result<Vec<StagedEntry>, UpdaterError> {
    for entry in plan {
        if entry.kind != StagedEntryKind::Directory {
            continue;
        }
        check_deadline(deadline, limits.max_duration)?;
        let path = staging.join(entry.path.as_str());
        create_dir_exclusive(&path)?;
        set_mode(&path, DIRECTORY_MODE)?;
    }

    let mut entries = Vec::with_capacity(plan.len());
    let mut actual_total = 0u64;
    let mut total_compressed = 0u64;
    for entry in plan {
        check_deadline(deadline, limits.max_duration)?;
        let path = staging.join(entry.path.as_str());
        match entry.kind {
            StagedEntryKind::Directory => {
                entries.push(StagedEntry {
                    path: entry.path.clone(),
                    kind: StagedEntryKind::Directory,
                    mode: DIRECTORY_MODE,
                    size: 0,
                    sha256: None,
                });
            }
            StagedEntryKind::File => {
                let index =
                    entry
                        .archive_index
                        .ok_or_else(|| UpdaterError::StagingVerification {
                            reason: "planned file has no archive entry".to_owned(),
                        })?;
                let mut file = archive
                    .by_index_with_options(
                        index,
                        ZipReadOptions::new().enforce_header_consistency(true),
                    )
                    .map_err(zip_error)?;
                if file.size() != entry.declared_size
                    || file.compressed_size() != entry.compressed_size
                {
                    return Err(archive_invalid(
                        "entry metadata changed between validation and extraction",
                    ));
                }
                let mut output = create_new_file(&path)?;
                let (written, digest) = copy_bounded(
                    &mut file,
                    &mut output,
                    entry.declared_size,
                    entry.compressed_size,
                    limits,
                    &mut actual_total,
                    deadline,
                )?;
                output.sync_all().map_err(atomic::io_error)?;
                drop(output);
                set_mode(&path, entry.mode)?;
                if written != entry.declared_size {
                    return Err(archive_invalid("entry produced fewer bytes than declared"));
                }
                let ratio = written / entry.compressed_size.max(1);
                if ratio > limits.max_entry_ratio {
                    return Err(limit_exceeded(
                        "entry compression ratio",
                        limits.max_entry_ratio,
                        ratio,
                    ));
                }
                total_compressed = total_compressed
                    .checked_add(entry.compressed_size)
                    .ok_or_else(|| limit_exceeded("total compressed bytes", u64::MAX, u64::MAX))?;
                entries.push(StagedEntry {
                    path: entry.path.clone(),
                    kind: StagedEntryKind::File,
                    mode: entry.mode,
                    size: written,
                    sha256: Some(digest),
                });
            }
        }
    }

    check_deadline(deadline, limits.max_duration)?;
    if actual_total > limits.max_total_uncompressed_bytes {
        return Err(limit_exceeded(
            "total uncompressed bytes",
            limits.max_total_uncompressed_bytes,
            actual_total,
        ));
    }
    if actual_total > 0 {
        let ratio = actual_total / total_compressed.max(1);
        if ratio > limits.max_total_ratio {
            return Err(limit_exceeded(
                "total compression ratio",
                limits.max_total_ratio,
                ratio,
            ));
        }
    }

    sync_directories(staging)?;
    verify_tree(staging, &entries)?;
    Ok(entries)
}

#[cfg(unix)]
fn create_new_file(path: &Path) -> Result<File, UpdaterError> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    options.open(path).map_err(atomic::io_error)
}

#[cfg(not(unix))]
fn create_new_file(path: &Path) -> Result<File, UpdaterError> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(atomic::io_error)
}

fn copy_bounded<R: Read>(
    reader: &mut R,
    writer: &mut File,
    declared_size: u64,
    compressed_size: u64,
    limits: &ExtractionLimits,
    actual_total: &mut u64,
    deadline: Instant,
) -> Result<(u64, Sha256Digest), UpdaterError> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; READ_BUFFER_BYTES];
    let mut written = 0u64;
    loop {
        check_deadline(deadline, limits.max_duration)?;
        let read = reader.read(&mut buffer).map_err(archive_read_error)?;
        if read == 0 {
            break;
        }
        let read_u64 = u64::try_from(read)
            .map_err(|_| limit_exceeded("entry uncompressed bytes", u64::MAX, u64::MAX))?;
        written = written
            .checked_add(read_u64)
            .ok_or_else(|| limit_exceeded("entry uncompressed bytes", u64::MAX, u64::MAX))?;
        if written > declared_size {
            return Err(archive_invalid("entry produced more bytes than declared"));
        }
        if written > limits.max_entry_uncompressed_bytes {
            return Err(limit_exceeded(
                "entry uncompressed bytes",
                limits.max_entry_uncompressed_bytes,
                written,
            ));
        }
        let ratio = written / compressed_size.max(1);
        if ratio > limits.max_entry_ratio {
            return Err(limit_exceeded(
                "entry compression ratio",
                limits.max_entry_ratio,
                ratio,
            ));
        }
        *actual_total = actual_total
            .checked_add(read_u64)
            .ok_or_else(|| limit_exceeded("total uncompressed bytes", u64::MAX, u64::MAX))?;
        if *actual_total > limits.max_total_uncompressed_bytes {
            return Err(limit_exceeded(
                "total uncompressed bytes",
                limits.max_total_uncompressed_bytes,
                *actual_total,
            ));
        }
        hasher.update(&buffer[..read]);
        writer
            .write_all(&buffer[..read])
            .map_err(atomic::io_error)?;
    }
    Ok((written, digest_from_hasher(hasher)))
}

fn archive_read_error(error: std::io::Error) -> UpdaterError {
    match error.kind() {
        std::io::ErrorKind::InvalidData | std::io::ErrorKind::UnexpectedEof => {
            archive_invalid(format!("entry read failed: {error}"))
        }
        _ => atomic::io_error(error),
    }
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<(), UpdaterError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(atomic::io_error)
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> Result<(), UpdaterError> {
    Ok(())
}

#[cfg(unix)]
fn sync_directories(root: &Path) -> Result<(), UpdaterError> {
    let mut directories = Vec::new();
    collect_directories(root, &mut directories)?;
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for path in directories {
        let directory = File::open(&path).map_err(atomic::io_error)?;
        directory.sync_all().map_err(atomic::io_error)?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn sync_directories(_root: &Path) -> Result<(), UpdaterError> {
    Ok(())
}

#[cfg(unix)]
fn collect_directories(root: &Path, directories: &mut Vec<PathBuf>) -> Result<(), UpdaterError> {
    let metadata = fs::symlink_metadata(root).map_err(atomic::io_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(UpdaterError::StagingVerification {
            reason: "staging root is not a real directory".to_owned(),
        });
    }
    directories.push(root.to_path_buf());
    for entry in fs::read_dir(root).map_err(atomic::io_error)? {
        let entry = entry.map_err(atomic::io_error)?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(atomic::io_error)?;
        if metadata.file_type().is_symlink() {
            return Err(UpdaterError::StagingVerification {
                reason: "staged tree contains a symbolic link".to_owned(),
            });
        }
        if metadata.is_dir() {
            collect_directories(&entry.path(), directories)?;
        }
    }
    Ok(())
}

fn verify_tree(root: &Path, entries: &[StagedEntry]) -> Result<(), UpdaterError> {
    let metadata = fs::symlink_metadata(root).map_err(atomic::io_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(UpdaterError::StagingVerification {
            reason: "staging root is not a real directory".to_owned(),
        });
    }

    let mut actual = BTreeSet::new();
    walk_tree(root, "", &mut actual)?;
    if actual.len() != entries.len() {
        return Err(UpdaterError::StagingVerification {
            reason: format!(
                "staged tree has {} entries; expected {}",
                actual.len(),
                entries.len()
            ),
        });
    }

    for entry in entries {
        let path = root.join(entry.path.as_str());
        let metadata = fs::symlink_metadata(&path).map_err(atomic::io_error)?;
        if metadata.file_type().is_symlink() {
            return Err(UpdaterError::StagingVerification {
                reason: "staged tree contains a symbolic link".to_owned(),
            });
        }
        match entry.kind {
            StagedEntryKind::Directory => {
                if !metadata.is_dir() {
                    return Err(UpdaterError::StagingVerification {
                        reason: "staged directory is not a directory".to_owned(),
                    });
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if metadata.permissions().mode() & UNIX_PERMISSION_BITS != entry.mode {
                        return Err(UpdaterError::StagingVerification {
                            reason: "staged directory mode does not match the plan".to_owned(),
                        });
                    }
                }
            }
            StagedEntryKind::File => {
                if !metadata.is_file() {
                    return Err(UpdaterError::StagingVerification {
                        reason: "staged file is not a regular file".to_owned(),
                    });
                }
                if metadata.len() != entry.size {
                    return Err(UpdaterError::StagingVerification {
                        reason: "staged file size does not match the plan".to_owned(),
                    });
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if metadata.permissions().mode() & UNIX_PERMISSION_BITS != entry.mode {
                        return Err(UpdaterError::StagingVerification {
                            reason: "staged file mode does not match the plan".to_owned(),
                        });
                    }
                }
                let digest = hash_file(&path)?;
                if Some(digest) != entry.sha256 {
                    return Err(UpdaterError::StagingVerification {
                        reason: "staged file hash does not match the plan".to_owned(),
                    });
                }
            }
        }
        if !actual.remove(entry.path.as_str()) {
            return Err(UpdaterError::StagingVerification {
                reason: "staged tree is missing a planned entry".to_owned(),
            });
        }
    }
    if !actual.is_empty() {
        return Err(UpdaterError::StagingVerification {
            reason: "staged tree contains an unplanned entry".to_owned(),
        });
    }
    Ok(())
}

fn walk_tree(
    root: &Path,
    relative: &str,
    output: &mut BTreeSet<String>,
) -> Result<(), UpdaterError> {
    let directory = if relative.is_empty() {
        root.to_path_buf()
    } else {
        root.join(relative)
    };
    for entry in fs::read_dir(&directory).map_err(atomic::io_error)? {
        let entry = entry.map_err(atomic::io_error)?;
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| UpdaterError::StagingVerification {
                reason: "staged tree contains a non-UTF-8 name".to_owned(),
            })?;
        let child = if relative.is_empty() {
            name.to_owned()
        } else {
            format!("{relative}/{name}")
        };
        let metadata = fs::symlink_metadata(entry.path()).map_err(atomic::io_error)?;
        if metadata.file_type().is_symlink() {
            return Err(UpdaterError::StagingVerification {
                reason: "staged tree contains a symbolic link".to_owned(),
            });
        }
        if metadata.is_dir() {
            output.insert(child.clone());
            walk_tree(root, &child, output)?;
        } else if metadata.is_file() {
            output.insert(child);
        } else {
            return Err(UpdaterError::StagingVerification {
                reason: "staged tree contains a special file".to_owned(),
            });
        }
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<Sha256Digest, UpdaterError> {
    let mut file = File::open(path).map_err(atomic::io_error)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; READ_BUFFER_BYTES];
    loop {
        let read = file.read(&mut buffer).map_err(atomic::io_error)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(digest_from_hasher(hasher))
}

fn remove_staging(path: &Path) {
    let _ = fs::remove_dir_all(path);
    let _ = atomic::sync_parent_dir(path);
}

fn check_deadline(deadline: Instant, max_duration: Duration) -> Result<(), UpdaterError> {
    if Instant::now() >= deadline {
        return Err(UpdaterError::ArchiveTimeout {
            seconds: max_duration.as_secs().max(1),
        });
    }
    Ok(())
}

fn le_u16(bytes: &[u8], offset: usize) -> Result<u16, UpdaterError> {
    let slice = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| archive_invalid("truncated header"))?;
    let array: [u8; 2] = slice
        .try_into()
        .map_err(|_| archive_invalid("truncated header"))?;
    Ok(u16::from_le_bytes(array))
}

fn le_u32(bytes: &[u8], offset: usize) -> Result<u32, UpdaterError> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| archive_invalid("truncated header"))?;
    let array: [u8; 4] = slice
        .try_into()
        .map_err(|_| archive_invalid("truncated header"))?;
    Ok(u32::from_le_bytes(array))
}

fn le_u64(bytes: &[u8], offset: usize) -> Result<u64, UpdaterError> {
    let slice = bytes
        .get(offset..offset + 8)
        .ok_or_else(|| archive_invalid("truncated header"))?;
    let array: [u8; 8] = slice
        .try_into()
        .map_err(|_| archive_invalid("truncated header"))?;
    Ok(u64::from_le_bytes(array))
}

fn archive_invalid(reason: impl Into<String>) -> UpdaterError {
    UpdaterError::ArchiveInvalid {
        reason: UpdaterError::bounded_detail(reason),
    }
}

fn entry_rejected(index: usize, reason: impl Into<String>) -> UpdaterError {
    UpdaterError::ArchiveEntryRejected {
        index,
        reason: UpdaterError::bounded_detail(reason),
    }
}

fn path_rejected(path: &str, reason: &'static str) -> UpdaterError {
    UpdaterError::ArchivePathRejected {
        path: UpdaterError::bounded_detail(path),
        reason,
    }
}

fn limit_exceeded(limit: &'static str, max: u64, actual: u64) -> UpdaterError {
    UpdaterError::ArchiveLimitExceeded { limit, max, actual }
}

fn invalid_limits(reason: &'static str) -> UpdaterError {
    UpdaterError::InvalidExtractionLimits { reason }
}

fn zip_error(error: zip::result::ZipError) -> UpdaterError {
    match error {
        zip::result::ZipError::Io(error) => atomic::io_error(error),
        other => archive_invalid(other.to_string()),
    }
}

#[cfg(test)]
mod tests;

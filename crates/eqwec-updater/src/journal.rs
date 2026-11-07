//! Append-only transaction journal.
//!
//! The journal records intent and every phase before the corresponding
//! filesystem mutation. Recovery reads complete lines, discards a partial
//! trailing line, and fails closed on a malformed complete line.

use core::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use eqwec_core::{Channel, ProductId};
use eqwec_security::{KeyId, Sha256Digest};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::atomic;
use crate::error::UpdaterError;
use crate::time::Timestamp;

/// Journal schema version.
pub const JOURNAL_SCHEMA_VERSION: u64 = 1;

/// Maximum journal file size in bytes.
pub const MAX_JOURNAL_BYTES: u64 = 16 * 1024 * 1024;

/// Maximum serialized journal record size in bytes, including the newline.
pub const MAX_JOURNAL_RECORD_BYTES: usize = 4_096;

/// Maximum idempotency-key length in bytes.
pub const MAX_IDEMPOTENCY_KEY_BYTES: usize = 128;

/// A 128-bit operation identifier encoded as 32 lowercase hex characters.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationId(String);

impl OperationId {
    /// Parses an operation identifier.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidOperationId`] unless the value is
    /// exactly 32 lowercase hexadecimal characters.
    pub fn parse(value: &str) -> Result<Self, UpdaterError> {
        if value.len() != 32 {
            return Err(UpdaterError::InvalidOperationId {
                reason: "operation id must be exactly 32 characters",
            });
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(UpdaterError::InvalidOperationId {
                reason: "operation id must be lowercase hexadecimal",
            });
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OperationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::str::FromStr for OperationId {
    type Err = UpdaterError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

/// A stable idempotency key for a mutating operation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Parses an idempotency key.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidIdempotencyKey`] when the key is empty,
    /// longer than 128 bytes, contains a character outside `A-Z a-z 0-9 . _ : -`,
    /// or does not start and end with an ASCII alphanumeric character.
    pub fn parse(value: &str) -> Result<Self, UpdaterError> {
        if value.is_empty() {
            return Err(UpdaterError::InvalidIdempotencyKey {
                reason: "key is empty",
            });
        }
        if value.len() > MAX_IDEMPOTENCY_KEY_BYTES {
            return Err(UpdaterError::InvalidIdempotencyKey {
                reason: "key exceeds the maximum length",
            });
        }
        let bytes = value.as_bytes();
        let alphanumeric = |byte: &u8| byte.is_ascii_alphanumeric();
        if !bytes.first().is_some_and(alphanumeric) || !bytes.last().is_some_and(alphanumeric) {
            return Err(UpdaterError::InvalidIdempotencyKey {
                reason: "key must start and end with an ASCII alphanumeric character",
            });
        }
        if bytes.iter().any(|byte| {
            !(byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
        }) {
            return Err(UpdaterError::InvalidIdempotencyKey {
                reason: "key contains a character outside A-Z a-z 0-9 . _ : -",
            });
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the key as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IdempotencyKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A durable transaction phase.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Phase {
    /// The operation was accepted and its intent is durable.
    Intent,
    /// The package bytes are complete in `downloads/`.
    Downloaded,
    /// The size and hash match the verified manifest.
    Verified,
    /// The tree is complete in staging and verified.
    Staged,
    /// The activation pointer is about to be replaced.
    ActivationIntent,
    /// The pointer references the new tree.
    Activated,
    /// The registry contains the new record.
    RegistryCommitted,
    /// The operation is complete.
    Committed,
    /// The operation failed before activation or was safely rolled back.
    Failed,
    /// The previous tree is active again.
    RolledBack,
    /// A product uninstall is about to remove its tree.
    UninstallIntent,
    /// The product tree and registry record are removed.
    Uninstalled,
}

impl Phase {
    /// Returns the canonical lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Intent => "intent",
            Self::Downloaded => "downloaded",
            Self::Verified => "verified",
            Self::Staged => "staged",
            Self::ActivationIntent => "activation-intent",
            Self::Activated => "activated",
            Self::RegistryCommitted => "registry-committed",
            Self::Committed => "committed",
            Self::Failed => "failed",
            Self::RolledBack => "rolled-back",
            Self::UninstallIntent => "uninstall-intent",
            Self::Uninstalled => "uninstalled",
        }
    }

    /// Returns whether no further mutation follows this phase.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Committed | Self::Failed | Self::RolledBack | Self::Uninstalled
        )
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::str::FromStr for Phase {
    type Err = UpdaterError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "intent" => Ok(Self::Intent),
            "downloaded" => Ok(Self::Downloaded),
            "verified" => Ok(Self::Verified),
            "staged" => Ok(Self::Staged),
            "activation-intent" => Ok(Self::ActivationIntent),
            "activated" => Ok(Self::Activated),
            "registry-committed" => Ok(Self::RegistryCommitted),
            "committed" => Ok(Self::Committed),
            "failed" => Ok(Self::Failed),
            "rolled-back" => Ok(Self::RolledBack),
            "uninstall-intent" => Ok(Self::UninstallIntent),
            "uninstalled" => Ok(Self::Uninstalled),
            other => Err(UpdaterError::InvalidJournalPhase {
                value: UpdaterError::bounded_detail(other),
            }),
        }
    }
}

/// Optional context attached to a journal record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JournalContext {
    from_version: Option<Version>,
    to_version: Option<Version>,
    channel: Channel,
    package_sha256: Sha256Digest,
    error_code: Option<String>,
    signer_key_id: Option<KeyId>,
    key_epoch: Option<u64>,
    metadata_sequence: Option<u64>,
}

impl JournalContext {
    /// Creates a journal context.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidJournalRecord`] when the error code is
    /// empty or contains a character outside `A-Z a-z 0-9 . _ : -`.
    pub fn new(
        from_version: Option<Version>,
        to_version: Option<Version>,
        channel: Channel,
        package_sha256: Sha256Digest,
        error_code: Option<String>,
    ) -> Result<Self, UpdaterError> {
        if let Some(code) = &error_code {
            validate_error_code(code)?;
        }
        Ok(Self {
            from_version,
            to_version,
            channel,
            package_sha256,
            error_code,
            signer_key_id: None,
            key_epoch: None,
            metadata_sequence: None,
        })
    }

    /// Attaches the verified-manifest identity needed to rebuild a registry
    /// record during recovery.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidJournalRecord`] when `key_epoch` or
    /// `metadata_sequence` is zero.
    pub fn with_installation_identity(
        mut self,
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
        self.signer_key_id = Some(signer_key_id);
        self.key_epoch = Some(key_epoch);
        self.metadata_sequence = Some(metadata_sequence);
        Ok(self)
    }

    /// Attaches or clears the error code.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidJournalRecord`] when the error code is
    /// empty or contains a character outside `A-Z a-z 0-9 . _ : -`.
    pub fn with_error_code(mut self, error_code: Option<String>) -> Result<Self, UpdaterError> {
        if let Some(code) = &error_code {
            validate_error_code(code)?;
        }
        self.error_code = error_code;
        Ok(self)
    }

    /// Returns the previous version, when this is an update.
    #[must_use]
    pub fn from_version(&self) -> Option<&Version> {
        self.from_version.as_ref()
    }

    /// Returns the target version, when known.
    #[must_use]
    pub fn to_version(&self) -> Option<&Version> {
        self.to_version.as_ref()
    }

    /// Returns the release channel.
    #[must_use]
    pub const fn channel(&self) -> Channel {
        self.channel
    }

    /// Returns the package SHA-256 digest.
    #[must_use]
    pub const fn package_sha256(&self) -> Sha256Digest {
        self.package_sha256
    }

    /// Returns the error code, only for a failed phase.
    #[must_use]
    pub fn error_code(&self) -> Option<&str> {
        self.error_code.as_deref()
    }

    /// Returns the signer key identifier, when the manifest identity was
    /// recorded.
    #[must_use]
    pub fn signer_key_id(&self) -> Option<&KeyId> {
        self.signer_key_id.as_ref()
    }

    /// Returns the signer key epoch, when the manifest identity was recorded.
    #[must_use]
    pub const fn key_epoch(&self) -> Option<u64> {
        self.key_epoch
    }

    /// Returns the manifest metadata sequence, when recorded.
    #[must_use]
    pub const fn metadata_sequence(&self) -> Option<u64> {
        self.metadata_sequence
    }

    /// Returns the complete verified-manifest identity.
    ///
    /// Recovery requires all three values to rebuild a registry record. A
    /// partial identity is invalid.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidJournalRecord`] when only some of the
    /// identity fields are present.
    pub fn installation_identity(&self) -> Result<(&KeyId, u64, u64), UpdaterError> {
        match (
            self.signer_key_id.as_ref(),
            self.key_epoch,
            self.metadata_sequence,
        ) {
            (Some(key_id), Some(epoch), Some(sequence)) => Ok((key_id, epoch, sequence)),
            (None, None, None) => Err(UpdaterError::InvalidJournalRecord {
                reason: "journal context has no verified-manifest identity".to_owned(),
            }),
            _ => Err(UpdaterError::InvalidJournalRecord {
                reason: "journal context has a partial verified-manifest identity".to_owned(),
            }),
        }
    }
}

/// One durable journal record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JournalRecord {
    operation_id: OperationId,
    idempotency_key: IdempotencyKey,
    product_id: ProductId,
    phase: Phase,
    timestamp: Timestamp,
    context: Option<JournalContext>,
}

impl JournalRecord {
    /// Creates a journal record.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidJournalRecord`] when an error code is
    /// attached to a non-failed phase.
    pub fn new(
        operation_id: OperationId,
        idempotency_key: IdempotencyKey,
        product_id: ProductId,
        phase: Phase,
        timestamp: Timestamp,
        context: Option<JournalContext>,
    ) -> Result<Self, UpdaterError> {
        if phase != Phase::Failed
            && context
                .as_ref()
                .is_some_and(|context| context.error_code.is_some())
        {
            return Err(UpdaterError::InvalidJournalRecord {
                reason: "error_code is allowed only for the failed phase".to_owned(),
            });
        }
        Ok(Self {
            operation_id,
            idempotency_key,
            product_id,
            phase,
            timestamp,
            context,
        })
    }

    /// Parses one complete record without its trailing newline.
    ///
    /// # Errors
    ///
    /// Returns a typed error for oversized, malformed, duplicate, unknown, or
    /// inconsistent input.
    pub fn parse(bytes: &[u8]) -> Result<Self, UpdaterError> {
        if bytes.len() + 1 > MAX_JOURNAL_RECORD_BYTES {
            return Err(UpdaterError::JournalRecordTooLarge {
                max: MAX_JOURNAL_RECORD_BYTES,
                actual: bytes.len() + 1,
            });
        }
        let text = std::str::from_utf8(bytes).map_err(|_| UpdaterError::JournalCorrupt {
            detail: "record is not valid UTF-8".to_owned(),
        })?;
        let mut deserializer = serde_json::Deserializer::from_str(text);
        let wire = WireRecord::deserialize(&mut deserializer).map_err(|error| {
            UpdaterError::JournalCorrupt {
                detail: UpdaterError::bounded_detail(error.to_string()),
            }
        })?;
        deserializer
            .end()
            .map_err(|_| UpdaterError::JournalCorrupt {
                detail: "trailing data after the record".to_owned(),
            })?;
        Self::from_wire(wire)
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

    /// Returns the phase.
    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    /// Returns the record time.
    #[must_use]
    pub const fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    /// Returns the optional context.
    #[must_use]
    pub fn context(&self) -> Option<&JournalContext> {
        self.context.as_ref()
    }

    /// Serializes the record with a trailing newline.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::JournalRecordTooLarge`] when the encoded line
    /// exceeds the bound.
    pub fn to_line(&self) -> Result<Vec<u8>, UpdaterError> {
        let wire = WireRecord {
            schema_version: JOURNAL_SCHEMA_VERSION,
            operation_id: self.operation_id.as_str().to_owned(),
            idempotency_key: self.idempotency_key.as_str().to_owned(),
            product_id: self.product_id.as_str().to_owned(),
            phase: self.phase.as_str().to_owned(),
            timestamp: self.timestamp.to_string(),
            context: self.context.as_ref().map(|context| WireContext {
                from_version: context.from_version.as_ref().map(ToString::to_string),
                to_version: context.to_version.as_ref().map(ToString::to_string),
                channel: context.channel.as_str().to_owned(),
                package_sha256: context.package_sha256.to_hex(),
                error_code: context.error_code.clone(),
                signer_key_id: context
                    .signer_key_id
                    .as_ref()
                    .map(|key_id| key_id.as_str().to_owned()),
                key_epoch: context.key_epoch,
                metadata_sequence: context.metadata_sequence,
            }),
        };
        let mut line = serde_json::to_vec(&wire).map_err(UpdaterError::from_json)?;
        line.push(b'\n');
        if line.len() > MAX_JOURNAL_RECORD_BYTES {
            return Err(UpdaterError::JournalRecordTooLarge {
                max: MAX_JOURNAL_RECORD_BYTES,
                actual: line.len(),
            });
        }
        Ok(line)
    }

    fn from_wire(wire: WireRecord) -> Result<Self, UpdaterError> {
        if wire.schema_version != JOURNAL_SCHEMA_VERSION {
            return Err(UpdaterError::UnsupportedSchemaVersion {
                found: wire.schema_version,
                supported: JOURNAL_SCHEMA_VERSION,
            });
        }
        let operation_id = OperationId::parse(&wire.operation_id)?;
        let idempotency_key = IdempotencyKey::parse(&wire.idempotency_key)?;
        let product_id =
            ProductId::new(wire.product_id).map_err(|error| UpdaterError::InvalidProductId {
                reason: error.to_string(),
            })?;
        let phase = wire.phase.parse::<Phase>()?;
        let timestamp = Timestamp::parse(&wire.timestamp)?;
        let context = wire
            .context
            .map(|context| parse_context(context, phase))
            .transpose()?;
        Self::new(
            operation_id,
            idempotency_key,
            product_id,
            phase,
            timestamp,
            context,
        )
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WireRecord {
    schema_version: u64,
    operation_id: String,
    idempotency_key: String,
    product_id: String,
    phase: String,
    timestamp: String,
    #[serde(default)]
    context: Option<WireContext>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WireContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    from_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    to_version: Option<String>,
    channel: String,
    package_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    signer_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    key_epoch: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    metadata_sequence: Option<u64>,
}

fn parse_context(wire: WireContext, phase: Phase) -> Result<JournalContext, UpdaterError> {
    let from_version = wire
        .from_version
        .map(|value| parse_version(&value))
        .transpose()?;
    let to_version = wire
        .to_version
        .map(|value| parse_version(&value))
        .transpose()?;
    let channel =
        wire.channel
            .parse::<Channel>()
            .map_err(|error| UpdaterError::InvalidChannel {
                value: error.to_string(),
            })?;
    let package_sha256 = Sha256Digest::parse_hex(&wire.package_sha256).map_err(|error| {
        UpdaterError::InvalidJournalRecord {
            reason: error.to_string(),
        }
    })?;
    if phase != Phase::Failed && wire.error_code.is_some() {
        return Err(UpdaterError::InvalidJournalRecord {
            reason: "error_code is allowed only for the failed phase".to_owned(),
        });
    }
    let mut context = JournalContext::new(
        from_version,
        to_version,
        channel,
        package_sha256,
        wire.error_code,
    )?;
    match (wire.signer_key_id, wire.key_epoch, wire.metadata_sequence) {
        (None, None, None) => {}
        (Some(key_id), Some(key_epoch), Some(metadata_sequence)) => {
            let key_id =
                KeyId::new(key_id).map_err(|error| UpdaterError::InvalidJournalRecord {
                    reason: error.to_string(),
                })?;
            context = context.with_installation_identity(key_id, key_epoch, metadata_sequence)?;
        }
        _ => {
            return Err(UpdaterError::InvalidJournalRecord {
                reason: "signer_key_id, key_epoch, and metadata_sequence must appear together"
                    .to_owned(),
            });
        }
    }
    Ok(context)
}

fn parse_version(value: &str) -> Result<Version, UpdaterError> {
    Version::parse(value).map_err(|_| UpdaterError::InvalidVersion {
        value: UpdaterError::bounded_detail(value),
    })
}

fn validate_error_code(value: &str) -> Result<(), UpdaterError> {
    if value.is_empty() || value.len() > 128 {
        return Err(UpdaterError::InvalidJournalRecord {
            reason: "error_code must be 1-128 bytes".to_owned(),
        });
    }
    if value
        .bytes()
        .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')))
    {
        return Err(UpdaterError::InvalidJournalRecord {
            reason: "error_code contains an invalid character".to_owned(),
        });
    }
    Ok(())
}

/// The result of reading a journal file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JournalContents {
    records: Vec<JournalRecord>,
    discarded_partial_line: bool,
}

impl JournalContents {
    /// Returns the complete records in file order.
    #[must_use]
    pub fn records(&self) -> &[JournalRecord] {
        &self.records
    }

    /// Returns whether a partial trailing line was discarded.
    #[must_use]
    pub const fn discarded_partial_line(&self) -> bool {
        self.discarded_partial_line
    }

    /// Returns the newest operation that has no terminal record.
    ///
    /// Operations are grouped by `operation_id`. A terminal record closes its
    /// own operation even when a later operation exists. Returns the last
    /// record of the newest operation that has no terminal phase.
    #[must_use]
    pub fn unfinished_operation(&self) -> Option<&JournalRecord> {
        let finished: std::collections::HashSet<&OperationId> = self
            .records
            .iter()
            .filter(|record| record.phase().is_terminal())
            .map(JournalRecord::operation_id)
            .collect();
        self.records
            .iter()
            .rev()
            .find(|record| !finished.contains(record.operation_id()))
    }

    /// Returns whether an operation other than `operation_id` has no terminal
    /// record.
    ///
    /// A journal must never contain two in-flight operations. Recovery resolves
    /// the existing operation before a new one starts.
    #[must_use]
    pub fn has_unfinished_operation_other_than(&self, operation_id: &OperationId) -> bool {
        let finished: std::collections::HashSet<&OperationId> = self
            .records
            .iter()
            .filter(|record| record.phase().is_terminal())
            .map(JournalRecord::operation_id)
            .collect();
        self.records.iter().any(|record| {
            record.operation_id() != operation_id && !finished.contains(record.operation_id())
        })
    }
}

/// Reads a journal file.
///
/// A missing file is an empty journal. A partial trailing line is discarded.
/// A malformed complete line fails closed.
///
/// # Errors
///
/// Returns a typed error for an oversized file, invalid UTF-8, or a malformed
/// complete record.
pub fn read_journal(path: &Path) -> Result<JournalContents, UpdaterError> {
    let Some(length) = atomic::metadata_len(path)? else {
        return Ok(JournalContents::default());
    };
    if length > MAX_JOURNAL_BYTES {
        return Err(UpdaterError::JournalTooLarge {
            max: MAX_JOURNAL_BYTES,
            actual: length,
        });
    }
    let bytes = atomic::read_all(path)?;
    let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if length > MAX_JOURNAL_BYTES {
        return Err(UpdaterError::JournalTooLarge {
            max: MAX_JOURNAL_BYTES,
            actual: length,
        });
    }

    let complete_end = bytes
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |index| index + 1);
    let discarded_partial_line = bytes.len() > complete_end;

    let mut records = Vec::new();
    for (index, line) in bytes[..complete_end]
        .split(|byte| *byte == b'\n')
        .enumerate()
    {
        if line.is_empty() {
            continue;
        }
        if line.len() + 1 > MAX_JOURNAL_RECORD_BYTES {
            return Err(UpdaterError::JournalCorrupt {
                detail: format!("record {} exceeds the size bound", index + 1),
            });
        }
        let record = JournalRecord::parse(line).map_err(|error| UpdaterError::JournalCorrupt {
            detail: UpdaterError::bounded_detail(format!("record {}: {error}", index + 1)),
        })?;
        records.push(record);
    }

    Ok(JournalContents {
        records,
        discarded_partial_line,
    })
}

/// Appends one record and flushes it before returning.
///
/// # Errors
///
/// Returns a typed error when the record or journal exceeds its bound or the
/// write fails.
pub fn append_journal_record(path: &Path, record: &JournalRecord) -> Result<(), UpdaterError> {
    let line = record.to_line()?;
    let existing = atomic::metadata_len(path)?.unwrap_or(0);
    let length = existing
        .checked_add(u64::try_from(line.len()).unwrap_or(u64::MAX))
        .ok_or(UpdaterError::JournalTooLarge {
            max: MAX_JOURNAL_BYTES,
            actual: u64::MAX,
        })?;
    if length > MAX_JOURNAL_BYTES {
        return Err(UpdaterError::JournalTooLarge {
            max: MAX_JOURNAL_BYTES,
            actual: length,
        });
    }

    let created = existing == 0 && atomic::metadata_len(path)?.is_none();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(atomic::io_error)?;
    file.write_all(&line).map_err(atomic::io_error)?;
    file.sync_data().map_err(atomic::io_error)?;
    drop(file);
    if created {
        atomic::sync_parent_dir(path)?;
    }
    Ok(())
}

/// The recovery action for an unfinished phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryAction {
    /// Remove the operation's download and staging directories.
    DiscardOperation,
    /// Reconcile the active pointer, then commit or roll back.
    ResumeActivation,
    /// Verify the active tree and write the registry record.
    CommitRegistry,
    /// Mark the operation committed.
    MarkCommitted,
    /// Remove the product tree and registry record, then mark uninstalled.
    ResumeUninstall,
    /// The operation is terminal; only prune.
    Terminal,
}

/// Returns the recovery action defined by the Hub schema.
#[must_use]
pub const fn recovery_action(phase: Phase) -> RecoveryAction {
    match phase {
        Phase::Intent | Phase::Downloaded | Phase::Verified | Phase::Staged => {
            RecoveryAction::DiscardOperation
        }
        Phase::ActivationIntent => RecoveryAction::ResumeActivation,
        Phase::Activated => RecoveryAction::CommitRegistry,
        Phase::RegistryCommitted => RecoveryAction::MarkCommitted,
        Phase::UninstallIntent => RecoveryAction::ResumeUninstall,
        Phase::Committed | Phase::Failed | Phase::RolledBack | Phase::Uninstalled => {
            RecoveryAction::Terminal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "eqwec-journal-{}-{}",
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

    fn timestamp() -> Timestamp {
        Timestamp::parse("2026-11-07T00:00:00Z").unwrap()
    }

    fn context() -> JournalContext {
        JournalContext::new(
            Some(Version::parse("0.0.9").unwrap()),
            Some(Version::parse("0.1.0").unwrap()),
            Channel::Stable,
            Sha256Digest::parse_hex(
                "53bd9dc21b205305c958c14f672d19214d5af9157ef66d9c9091b14837fe42a3",
            )
            .unwrap(),
            None,
        )
        .unwrap()
    }

    fn record(phase: Phase) -> JournalRecord {
        JournalRecord::new(
            OperationId::parse("0123456789abcdef0123456789abcdef").unwrap(),
            IdempotencyKey::parse("install:eqwec-hub:stable:0.1.0").unwrap(),
            ProductId::new("eqwec-hub").unwrap(),
            phase,
            timestamp(),
            Some(context()),
        )
        .unwrap()
    }

    #[test]
    fn round_trips_a_record() {
        let record = record(Phase::Intent);
        let line = record.to_line().unwrap();
        assert!(line.ends_with(b"\n"));
        let parsed = JournalRecord::parse(&line[..line.len() - 1]).unwrap();
        assert_eq!(parsed, record);
        assert_eq!(parsed.phase(), Phase::Intent);
        assert_eq!(parsed.context().unwrap().channel(), Channel::Stable);
    }

    #[test]
    fn serializes_canonical_phase_names() {
        for (phase, name) in [
            (Phase::Intent, "intent"),
            (Phase::Downloaded, "downloaded"),
            (Phase::Verified, "verified"),
            (Phase::Staged, "staged"),
            (Phase::ActivationIntent, "activation-intent"),
            (Phase::Activated, "activated"),
            (Phase::RegistryCommitted, "registry-committed"),
            (Phase::Committed, "committed"),
            (Phase::Failed, "failed"),
            (Phase::RolledBack, "rolled-back"),
        ] {
            assert_eq!(phase.as_str(), name);
            assert_eq!(name.parse::<Phase>().unwrap(), phase);
        }
    }

    #[test]
    fn rejects_error_code_on_non_failed_phase() {
        let context = JournalContext::new(
            None,
            None,
            Channel::Stable,
            Sha256Digest::parse_hex(
                "53bd9dc21b205305c958c14f672d19214d5af9157ef66d9c9091b14837fe42a3",
            )
            .unwrap(),
            Some("E_VERIFY".to_owned()),
        )
        .unwrap();
        let error = JournalRecord::new(
            OperationId::parse("0123456789abcdef0123456789abcdef").unwrap(),
            IdempotencyKey::parse("install:eqwec-hub:stable:0.1.0").unwrap(),
            ProductId::new("eqwec-hub").unwrap(),
            Phase::Intent,
            timestamp(),
            Some(context),
        )
        .unwrap_err();
        assert!(matches!(error, UpdaterError::InvalidJournalRecord { .. }));
    }

    #[test]
    fn rejects_invalid_operation_ids_and_keys() {
        for value in [
            "",
            "0123",
            "0123456789ABCDEF0123456789abcdef",
            "0123456789abcdef0123456789abcdeg",
        ] {
            assert!(OperationId::parse(value).is_err(), "{value:?}");
        }
        for value in [
            "",
            "-key",
            "key-",
            "key with space",
            "key/slash",
            &"a".repeat(MAX_IDEMPOTENCY_KEY_BYTES + 1),
        ] {
            assert!(IdempotencyKey::parse(value).is_err(), "{value:?}");
        }
        assert!(IdempotencyKey::parse("install:eqwec-hub:stable:0.1.0").is_ok());
    }

    #[test]
    fn appends_and_reads_records() {
        let dir = TestDir::new();
        let path = dir.path().join("operations.jsonl");
        append_journal_record(&path, &record(Phase::Intent)).unwrap();
        append_journal_record(&path, &record(Phase::Staged)).unwrap();

        let contents = read_journal(&path).unwrap();
        assert_eq!(contents.records().len(), 2);
        assert!(!contents.discarded_partial_line());
        assert_eq!(
            contents.unfinished_operation().unwrap().phase(),
            Phase::Staged
        );
    }

    #[test]
    fn a_terminal_record_closes_only_its_own_operation() {
        let dir = TestDir::new();
        let path = dir.path().join("operations.jsonl");

        let other = |phase: Phase, id: &str| {
            JournalRecord::new(
                OperationId::parse(id).unwrap(),
                IdempotencyKey::parse("install:eqwec-hub:stable:0.1.0").unwrap(),
                ProductId::new("eqwec-hub").unwrap(),
                phase,
                timestamp(),
                Some(context()),
            )
            .unwrap()
        };

        // An unfinished operation followed by a completed one: the unfinished
        // operation is still the recovery target.
        append_journal_record(&path, &record(Phase::Intent)).unwrap();
        append_journal_record(
            &path,
            &other(Phase::Committed, "fedcba9876543210fedcba9876543210"),
        )
        .unwrap();
        let contents = read_journal(&path).unwrap();
        assert_eq!(
            contents
                .unfinished_operation()
                .unwrap()
                .operation_id()
                .as_str(),
            "0123456789abcdef0123456789abcdef"
        );
        let first = OperationId::parse("0123456789abcdef0123456789abcdef").unwrap();
        let second = OperationId::parse("fedcba9876543210fedcba9876543210").unwrap();
        assert!(contents.has_unfinished_operation_other_than(&second));
        assert!(!contents.has_unfinished_operation_other_than(&first));

        // Once the first operation also reaches a terminal phase, nothing is
        // unfinished.
        append_journal_record(&path, &record(Phase::Failed)).unwrap();
        let contents = read_journal(&path).unwrap();
        assert!(contents.unfinished_operation().is_none());
        assert!(!contents.has_unfinished_operation_other_than(&second));
    }

    #[test]
    fn discards_a_partial_trailing_line() {
        let dir = TestDir::new();
        let path = dir.path().join("operations.jsonl");
        let mut bytes = record(Phase::Intent).to_line().unwrap();
        bytes.extend_from_slice(b"{\"schema_version\":1,");
        fs::write(&path, bytes).unwrap();

        let contents = read_journal(&path).unwrap();
        assert_eq!(contents.records().len(), 1);
        assert!(contents.discarded_partial_line());
    }

    #[test]
    fn rejects_a_malformed_complete_line() {
        let dir = TestDir::new();
        let path = dir.path().join("operations.jsonl");
        let mut bytes = record(Phase::Intent).to_line().unwrap();
        bytes.extend_from_slice(b"not json\n");
        fs::write(&path, bytes).unwrap();

        assert!(matches!(
            read_journal(&path).unwrap_err(),
            UpdaterError::JournalCorrupt { .. }
        ));
    }

    #[test]
    fn rejects_oversized_files_and_records() {
        let dir = TestDir::new();
        let path = dir.path().join("operations.jsonl");
        let file = fs::File::create(&path).unwrap();
        file.set_len(MAX_JOURNAL_BYTES + 1).unwrap();
        assert!(matches!(
            read_journal(&path).unwrap_err(),
            UpdaterError::JournalTooLarge { .. }
        ));

        let oversized = vec![b' '; MAX_JOURNAL_RECORD_BYTES + 1];
        assert!(matches!(
            JournalRecord::parse(&oversized).unwrap_err(),
            UpdaterError::JournalRecordTooLarge { .. }
        ));
    }

    #[test]
    fn maps_recovery_actions() {
        assert_eq!(
            recovery_action(Phase::Staged),
            RecoveryAction::DiscardOperation
        );
        assert_eq!(
            recovery_action(Phase::ActivationIntent),
            RecoveryAction::ResumeActivation
        );
        assert_eq!(
            recovery_action(Phase::Activated),
            RecoveryAction::CommitRegistry
        );
        assert_eq!(
            recovery_action(Phase::RegistryCommitted),
            RecoveryAction::MarkCommitted
        );
        assert_eq!(recovery_action(Phase::Committed), RecoveryAction::Terminal);
    }
}

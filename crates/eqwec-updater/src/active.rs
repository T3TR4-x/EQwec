//! Atomic activation pointer v1.
//!
//! `active.json` is the only pointer that makes a verified version tree
//! active. It is small, bounded, and replaced atomically.

use std::path::Path;

use eqwec_security::{SafeRelativePath, Sha256Digest};
use serde::{Deserialize, Serialize};

use crate::atomic;
use crate::error::UpdaterError;
use crate::journal::OperationId;
use crate::layout::is_version_tree;
use crate::time::Timestamp;

/// Active-pointer schema version.
pub const ACTIVE_POINTER_SCHEMA_VERSION: u64 = 1;

/// Maximum encoded active-pointer size in bytes.
pub const MAX_ACTIVE_POINTER_BYTES: usize = 4_096;

/// The pointer to the active, immutable version tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivePointer {
    tree: SafeRelativePath,
    package_sha256: Sha256Digest,
    activated_at: Timestamp,
}

impl ActivePointer {
    /// Creates an active pointer.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidActivePointer`] when the tree is not a
    /// `versions/<tree-id>` path.
    pub fn new(
        tree: SafeRelativePath,
        package_sha256: Sha256Digest,
        activated_at: Timestamp,
    ) -> Result<Self, UpdaterError> {
        if !is_version_tree(&tree) {
            return Err(invalid_pointer("tree must be a versions/<tree-id> path"));
        }
        Ok(Self {
            tree,
            package_sha256,
            activated_at,
        })
    }

    /// Parses an active pointer.
    ///
    /// # Errors
    ///
    /// Returns a typed error for oversized, malformed, duplicate, unknown, or
    /// out-of-bound input.
    pub fn parse(bytes: &[u8]) -> Result<Self, UpdaterError> {
        if bytes.len() > MAX_ACTIVE_POINTER_BYTES {
            return Err(UpdaterError::ActivePointerTooLarge {
                max: MAX_ACTIVE_POINTER_BYTES,
                actual: bytes.len(),
            });
        }
        let text = std::str::from_utf8(bytes).map_err(|_| invalid_pointer("not valid UTF-8"))?;
        let mut deserializer = serde_json::Deserializer::from_str(text);
        let wire =
            WireActivePointer::deserialize(&mut deserializer).map_err(UpdaterError::from_json)?;
        deserializer
            .end()
            .map_err(|_| invalid_pointer("trailing data after the pointer"))?;
        Self::from_wire(wire)
    }

    /// Loads a pointer, returning `None` when the file does not exist.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an oversized, unreadable, or invalid file.
    pub fn load(path: &Path) -> Result<Option<Self>, UpdaterError> {
        let Some(length) = atomic::metadata_len(path)? else {
            return Ok(None);
        };
        let length = usize::try_from(length).unwrap_or(usize::MAX);
        if length > MAX_ACTIVE_POINTER_BYTES {
            return Err(UpdaterError::ActivePointerTooLarge {
                max: MAX_ACTIVE_POINTER_BYTES,
                actual: length,
            });
        }
        let bytes = atomic::read_all(path)?;
        Self::parse(&bytes).map(Some)
    }

    /// Returns the active tree path.
    #[must_use]
    pub fn tree(&self) -> &SafeRelativePath {
        &self.tree
    }

    /// Returns the active package SHA-256.
    #[must_use]
    pub const fn package_sha256(&self) -> Sha256Digest {
        self.package_sha256
    }

    /// Returns the activation time.
    #[must_use]
    pub const fn activated_at(&self) -> Timestamp {
        self.activated_at
    }

    /// Serializes the pointer as compact JSON.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::ActivePointerTooLarge`] when the encoded
    /// document exceeds the bound.
    pub fn to_bytes(&self) -> Result<Vec<u8>, UpdaterError> {
        let wire = WireActivePointer {
            schema_version: ACTIVE_POINTER_SCHEMA_VERSION,
            tree: self.tree.as_str().to_owned(),
            package_sha256: self.package_sha256.to_hex(),
            activated_at: self.activated_at.to_string(),
        };
        let bytes = serde_json::to_vec(&wire).map_err(UpdaterError::from_json)?;
        if bytes.len() > MAX_ACTIVE_POINTER_BYTES {
            return Err(UpdaterError::ActivePointerTooLarge {
                max: MAX_ACTIVE_POINTER_BYTES,
                actual: bytes.len(),
            });
        }
        Ok(bytes)
    }

    /// Atomically replaces the pointer file.
    ///
    /// # Errors
    ///
    /// Returns a typed error when serialization or the atomic write fails.
    pub fn write_atomic(
        &self,
        path: &Path,
        operation_id: &OperationId,
    ) -> Result<(), UpdaterError> {
        let bytes = self.to_bytes()?;
        atomic::write_atomic(path, &bytes, operation_id.as_str())
    }

    fn from_wire(wire: WireActivePointer) -> Result<Self, UpdaterError> {
        if wire.schema_version != ACTIVE_POINTER_SCHEMA_VERSION {
            return Err(UpdaterError::UnsupportedSchemaVersion {
                found: wire.schema_version,
                supported: ACTIVE_POINTER_SCHEMA_VERSION,
            });
        }
        let tree = SafeRelativePath::new(wire.tree)
            .map_err(|error| invalid_pointer(error.to_string().as_str()))?;
        let package_sha256 = Sha256Digest::parse_hex(&wire.package_sha256)
            .map_err(|error| invalid_pointer(error.to_string().as_str()))?;
        let activated_at = Timestamp::parse(&wire.activated_at)?;
        Self::new(tree, package_sha256, activated_at)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WireActivePointer {
    schema_version: u64,
    tree: String,
    package_sha256: String,
    activated_at: String,
}

fn invalid_pointer(reason: &str) -> UpdaterError {
    UpdaterError::InvalidActivePointer {
        reason: UpdaterError::bounded_detail(reason),
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
                "eqwec-active-{}-{}",
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

    fn pointer() -> ActivePointer {
        ActivePointer::new(
            SafeRelativePath::new("versions/0.1.0-53bd9dc21b20").unwrap(),
            Sha256Digest::parse_hex(
                "53bd9dc21b205305c958c14f672d19214d5af9157ef66d9c9091b14837fe42a3",
            )
            .unwrap(),
            Timestamp::parse("2026-11-07T00:00:00Z").unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn round_trips_a_pointer() {
        let pointer = pointer();
        let bytes = pointer.to_bytes().unwrap();
        assert_eq!(ActivePointer::parse(&bytes).unwrap(), pointer);
    }

    #[test]
    fn rejects_non_version_trees() {
        let error = ActivePointer::new(
            SafeRelativePath::new("active").unwrap(),
            pointer().package_sha256(),
            pointer().activated_at(),
        )
        .unwrap_err();
        assert!(matches!(error, UpdaterError::InvalidActivePointer { .. }));
    }

    #[test]
    fn rejects_oversized_and_unknown_input() {
        let oversized = vec![b' '; MAX_ACTIVE_POINTER_BYTES + 1];
        assert!(matches!(
            ActivePointer::parse(&oversized).unwrap_err(),
            UpdaterError::ActivePointerTooLarge { .. }
        ));

        let unknown = br#"{"schema_version":1,"tree":"versions/0.1.0-53bd9dc21b20","package_sha256":"53bd9dc21b205305c958c14f672d19214d5af9157ef66d9c9091b14837fe42a3","activated_at":"2026-11-07T00:00:00Z","extra":true}"#;
        assert!(ActivePointer::parse(unknown).is_err());
    }

    #[test]
    fn writes_and_loads_atomically() {
        let dir = TestDir::new();
        let path = dir.path().join("active.json");
        let operation = OperationId::parse("0123456789abcdef0123456789abcdef").unwrap();
        pointer().write_atomic(&path, &operation).unwrap();
        assert_eq!(ActivePointer::load(&path).unwrap().unwrap(), pointer());
        assert_eq!(
            ActivePointer::load(&dir.path().join("missing.json")).unwrap(),
            None
        );
    }
}

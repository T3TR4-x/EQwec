//! Per-user installation layout and tree identifiers.
//!
//! The layout is defined by `docs/product-specs/eqwec-hub-schemas.md` §6. All
//! returned paths are lexical; the caller must create them and must reject
//! symlink escapes when resolving an existing tree.

use std::fs;
use std::path::{Component, Path, PathBuf};

use eqwec_core::ProductId;
use eqwec_security::{SafeRelativePath, Sha256Digest};
use semver::Version;

use crate::atomic;
use crate::error::UpdaterError;
use crate::journal::OperationId;

/// Maximum length of a base or installation root in bytes.
pub const MAX_PATH_BYTES: usize = 1_024;

/// The root of one user profile's Hub data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallLayout {
    base: PathBuf,
}

impl InstallLayout {
    /// Creates a layout rooted at an absolute, normalized path.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidInstallRoot`] when the base is relative,
    /// empty, non-UTF-8, too long, contains `.` or `..`, or is a filesystem
    /// root.
    pub fn new(base: impl Into<PathBuf>) -> Result<Self, UpdaterError> {
        let base = base.into();
        validate_absolute(&base)?;
        if base.parent().is_none() {
            return Err(invalid_root("base must not be a filesystem root"));
        }
        Ok(Self { base })
    }

    /// Returns the base directory.
    #[must_use]
    pub fn base(&self) -> &Path {
        &self.base
    }

    /// Returns `<base>/registry.json`.
    #[must_use]
    pub fn registry_path(&self) -> PathBuf {
        self.base.join("registry.json")
    }

    /// Returns `<base>/journal/operations.jsonl`.
    #[must_use]
    pub fn journal_path(&self) -> PathBuf {
        self.base.join("journal").join("operations.jsonl")
    }

    /// Returns the product installation root.
    #[must_use]
    pub fn product_root(&self, product_id: &ProductId) -> PathBuf {
        self.base.join("installations").join(product_id.as_str())
    }

    /// Returns the product activation pointer path.
    #[must_use]
    pub fn active_pointer_path(&self, product_id: &ProductId) -> PathBuf {
        self.product_root(product_id).join("active.json")
    }

    /// Returns the product version directory.
    #[must_use]
    pub fn versions_dir(&self, product_id: &ProductId) -> PathBuf {
        self.product_root(product_id).join("versions")
    }

    /// Resolves a validated relative tree path inside the product root.
    #[must_use]
    pub fn tree_path(&self, product_id: &ProductId, tree: &SafeRelativePath) -> PathBuf {
        self.product_root(product_id).join(tree.as_str())
    }

    /// Returns the staging directory for one operation.
    #[must_use]
    pub fn staging_dir(&self, product_id: &ProductId, operation_id: &OperationId) -> PathBuf {
        self.product_root(product_id)
            .join("staging")
            .join(operation_id.as_str())
    }

    /// Returns the download directory for one operation.
    #[must_use]
    pub fn downloads_dir(&self, operation_id: &OperationId) -> PathBuf {
        self.base.join("downloads").join(operation_id.as_str())
    }

    /// Returns the package file path for one operation.
    #[must_use]
    pub fn download_package_path(&self, operation_id: &OperationId) -> PathBuf {
        self.downloads_dir(operation_id).join("package.zip")
    }

    /// Creates the base layout directories.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::Io`] when a directory cannot be created.
    pub fn ensure_base_dirs(&self) -> Result<(), UpdaterError> {
        for path in [
            self.base.clone(),
            self.base.join("journal"),
            self.base.join("installations"),
            self.base.join("downloads"),
            self.base.join("cache"),
            self.base.join("logs"),
            self.base.join("diagnostics"),
        ] {
            fs::create_dir_all(&path).map_err(atomic::io_error)?;
        }
        Ok(())
    }

    /// Verifies that an absolute installation root is a strict descendant of
    /// the profile base.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidInstallRoot`] or
    /// [`UpdaterError::InstallRootOutsideBase`].
    pub fn validate_install_root(&self, root: &Path) -> Result<(), UpdaterError> {
        validate_absolute(root)?;
        if root == self.base || !root.starts_with(&self.base) {
            return Err(UpdaterError::InstallRootOutsideBase);
        }
        Ok(())
    }
}

/// Builds `versions/<version>-<first 12 hex characters of sha256>`.
///
/// # Errors
///
/// Returns [`UpdaterError::InvalidTreeId`] when the resulting value is not a
/// safe relative path.
pub fn tree_id(
    version: &Version,
    package_sha256: Sha256Digest,
) -> Result<SafeRelativePath, UpdaterError> {
    let hex = package_sha256.to_hex();
    let value = format!("versions/{version}-{}", &hex[..12]);
    SafeRelativePath::new(value).map_err(|error| UpdaterError::InvalidTreeId {
        reason: error.to_string(),
    })
}

/// Returns whether a path has exactly the shape `versions/<tree-id>`.
#[must_use]
pub fn is_version_tree(tree: &SafeRelativePath) -> bool {
    let mut components = tree.components();
    matches!(
        (components.next(), components.next(), components.next()),
        (Some("versions"), Some(id), None) if !id.is_empty()
    )
}

pub(crate) fn validate_absolute(path: &Path) -> Result<(), UpdaterError> {
    let text = path
        .to_str()
        .ok_or_else(|| invalid_root("path is not valid UTF-8"))?;
    if text.is_empty() {
        return Err(invalid_root("path is empty"));
    }
    if text.len() > MAX_PATH_BYTES {
        return Err(invalid_root("path exceeds the maximum length"));
    }
    if !path.is_absolute() {
        return Err(invalid_root("path must be absolute"));
    }
    if text
        .chars()
        .any(|character| character == '\0' || character.is_control())
    {
        return Err(invalid_root("path contains a control character"));
    }
    for component in path.components() {
        if matches!(component, Component::CurDir | Component::ParentDir) {
            return Err(invalid_root("path must be normalized"));
        }
    }
    #[cfg(windows)]
    let normalized = text.replace('\\', "/");
    #[cfg(not(windows))]
    let normalized = text.to_owned();
    let without_prefix = normalized.strip_prefix("//").unwrap_or(&normalized);
    if without_prefix.contains("//") {
        return Err(invalid_root("path contains an empty component"));
    }
    if without_prefix.split('/').any(|component| component == ".") {
        return Err(invalid_root("path must be normalized"));
    }
    if without_prefix.ends_with('/') && !without_prefix.is_empty() {
        return Err(invalid_root("path must not end with a separator"));
    }
    Ok(())
}

fn invalid_root(reason: &'static str) -> UpdaterError {
    UpdaterError::InvalidInstallRoot {
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest() -> Sha256Digest {
        Sha256Digest::parse_hex("53bd9dc21b205305c958c14f672d19214d5af9157ef66d9c9091b14837fe42a3")
            .unwrap()
    }

    #[test]
    fn builds_a_tree_id() {
        let tree = tree_id(&Version::parse("0.1.0").unwrap(), digest()).unwrap();
        assert_eq!(tree.as_str(), "versions/0.1.0-53bd9dc21b20");
        assert!(is_version_tree(&tree));
    }

    #[test]
    fn rejects_invalid_bases() {
        for value in [
            PathBuf::from("relative/path"),
            PathBuf::from(""),
            PathBuf::from("/"),
        ] {
            assert!(
                InstallLayout::new(value.clone()).is_err(),
                "{value:?} should be rejected"
            );
        }
    }

    #[test]
    fn rejects_unnormalized_bases() {
        assert!(InstallLayout::new("/tmp/a/../b").is_err());
        assert!(InstallLayout::new("/tmp/a/./b").is_err());
    }

    #[test]
    fn builds_the_documented_layout() {
        let layout = InstallLayout::new("/home/user/.local/share/eqwec").unwrap();
        let product = ProductId::new("eqwec-hub").unwrap();
        let operation = OperationId::parse("0123456789abcdef0123456789abcdef").unwrap();

        assert_eq!(
            layout.registry_path(),
            PathBuf::from("/home/user/.local/share/eqwec/registry.json")
        );
        assert_eq!(
            layout.journal_path(),
            PathBuf::from("/home/user/.local/share/eqwec/journal/operations.jsonl")
        );
        assert_eq!(
            layout.product_root(&product),
            PathBuf::from("/home/user/.local/share/eqwec/installations/eqwec-hub")
        );
        assert_eq!(
            layout.active_pointer_path(&product),
            PathBuf::from("/home/user/.local/share/eqwec/installations/eqwec-hub/active.json")
        );
        assert_eq!(
            layout.staging_dir(&product, &operation),
            PathBuf::from(
                "/home/user/.local/share/eqwec/installations/eqwec-hub/staging/0123456789abcdef0123456789abcdef"
            )
        );
        assert_eq!(
            layout.downloads_dir(&operation),
            PathBuf::from(
                "/home/user/.local/share/eqwec/downloads/0123456789abcdef0123456789abcdef"
            )
        );
        assert_eq!(
            layout.download_package_path(&operation),
            PathBuf::from(
                "/home/user/.local/share/eqwec/downloads/0123456789abcdef0123456789abcdef/package.zip"
            )
        );
    }

    #[test]
    fn validates_install_root_containment() {
        let layout = InstallLayout::new("/home/user/.local/share/eqwec").unwrap();
        assert!(
            layout
                .validate_install_root(Path::new(
                    "/home/user/.local/share/eqwec/installations/eqwec-hub"
                ))
                .is_ok()
        );
        assert_eq!(
            layout
                .validate_install_root(Path::new("/home/user/.local/share/other/eqwec-hub"))
                .unwrap_err(),
            UpdaterError::InstallRootOutsideBase
        );
        assert_eq!(
            layout
                .validate_install_root(Path::new("/home/user/.local/share/eqwec"))
                .unwrap_err(),
            UpdaterError::InstallRootOutsideBase
        );
    }

    #[test]
    fn rejects_non_version_trees() {
        for value in ["versions", "versions/a/b", "other/0.1.0", "versions/"] {
            let tree = SafeRelativePath::new(value);
            if let Ok(tree) = tree {
                assert!(!is_version_tree(&tree), "{value:?} should not be a tree");
            }
        }
    }
}

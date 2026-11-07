//! Disk-space preflight for staging.

use std::path::Path;

use crate::error::UpdaterError;

/// Extra headroom reserved on top of two package copies.
pub const STAGING_MARGIN_BYTES: u64 = 64 * 1024 * 1024;

/// Returns the bytes required to stage a package of `package_size` bytes.
///
/// Staging needs the downloaded package plus its extracted tree, and the
/// previous version is retained until the new one is active. The result
/// therefore reserves two copies plus [`STAGING_MARGIN_BYTES`].
///
/// Returns `None` when the arithmetic would overflow.
#[must_use]
pub fn required_staging_bytes(package_size: u64) -> Option<u64> {
    package_size
        .checked_mul(2)?
        .checked_add(STAGING_MARGIN_BYTES)
}

/// Fails when `path` has less than `required_bytes` available.
///
/// Returns the available byte count on success.
///
/// # Errors
///
/// Returns [`UpdaterError::InsufficientDiskSpace`] when the filesystem is too
/// small, and [`UpdaterError::Io`] when free space cannot be queried.
pub fn ensure_free_space(path: &Path, required_bytes: u64) -> Result<u64, UpdaterError> {
    let available = fs4::available_space(path).map_err(|error| UpdaterError::Io {
        detail: format!("query free space for {}: {error}", path.display()),
    })?;
    if available < required_bytes {
        return Err(UpdaterError::InsufficientDiskSpace {
            available,
            required: required_bytes,
        });
    }
    Ok(available)
}

#[cfg(test)]
mod tests {
    use super::{ensure_free_space, required_staging_bytes};
    use crate::UpdaterError;

    const MIB: u64 = 1024 * 1024;
    const GIB: u64 = 1024 * MIB;

    #[test]
    fn required_bytes_include_two_copies_and_margin() {
        assert_eq!(required_staging_bytes(0), Some(64 * MIB));
        assert_eq!(required_staging_bytes(GIB), Some(2 * GIB + 64 * MIB));
    }

    #[test]
    fn required_bytes_reject_overflow() {
        assert_eq!(required_staging_bytes(u64::MAX), None);
        assert_eq!(required_staging_bytes(u64::MAX / 2), None);
    }

    #[test]
    fn zero_requirement_always_passes() {
        let available = ensure_free_space(&std::env::temp_dir(), 0).expect("preflight");
        assert!(available > 0);
    }

    #[test]
    fn impossible_requirement_fails_closed() {
        let error = ensure_free_space(&std::env::temp_dir(), u64::MAX).expect_err("must fail");
        assert!(matches!(
            error,
            UpdaterError::InsufficientDiskSpace { required, .. } if required == u64::MAX
        ));
    }
}

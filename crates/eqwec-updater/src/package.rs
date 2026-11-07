//! Package size and SHA-256 verification.

use std::io::Read;

use eqwec_security::Sha256Digest;
use sha2::{Digest, Sha256};

use crate::error::UpdaterError;
use crate::manifest::MAX_PACKAGE_BYTES;
use crate::verify::VerifiedManifest;

const READ_BUFFER_BYTES: usize = 64 * 1024;

/// A package whose size and SHA-256 matched a verified manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedPackage {
    size: u64,
    sha256: Sha256Digest,
}

impl VerifiedPackage {
    /// Returns the verified package size in bytes.
    #[must_use]
    pub const fn size(self) -> u64 {
        self.size
    }

    /// Returns the verified SHA-256 digest.
    #[must_use]
    pub const fn sha256(self) -> Sha256Digest {
        self.sha256
    }
}

impl VerifiedManifest {
    /// Streams a package through size and SHA-256 verification.
    ///
    /// The reader is consumed until EOF. The method does not allocate the
    /// package in memory and does not extract or execute it.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an I/O failure, oversized input, size
    /// mismatch, truncation, or hash mismatch.
    pub fn verify_package_reader<R: Read>(
        &self,
        mut reader: R,
    ) -> Result<VerifiedPackage, UpdaterError> {
        let package = self.manifest().package();
        let expected = package.size();
        if expected == 0 || expected > MAX_PACKAGE_BYTES {
            return Err(UpdaterError::InvalidPackageSize { size: expected });
        }

        let mut hasher = Sha256::new();
        let mut buffer = [0u8; READ_BUFFER_BYTES];
        let mut total = 0u64;
        loop {
            let read = reader.read(&mut buffer).map_err(|error| UpdaterError::Io {
                detail: UpdaterError::bounded_detail(error.to_string()),
            })?;
            if read == 0 {
                break;
            }
            let read_u64 = u64::try_from(read).map_err(|_| UpdaterError::PackageTooLarge {
                max: MAX_PACKAGE_BYTES,
                actual: u64::MAX,
            })?;
            total = total
                .checked_add(read_u64)
                .ok_or(UpdaterError::PackageTooLarge {
                    max: MAX_PACKAGE_BYTES,
                    actual: u64::MAX,
                })?;
            if total > expected {
                return Err(UpdaterError::PackageSizeMismatch {
                    expected,
                    actual: total,
                });
            }
            hasher.update(&buffer[..read]);
        }

        if total < expected {
            return Err(UpdaterError::PackageTruncated {
                expected,
                actual: total,
            });
        }

        let actual = digest_from_hasher(hasher);
        if actual != package.sha256() {
            return Err(UpdaterError::PackageHashMismatch {
                expected: package.sha256().to_hex(),
                actual: actual.to_hex(),
            });
        }
        Ok(VerifiedPackage {
            size: total,
            sha256: actual,
        })
    }

    /// Verifies an in-memory package.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a size mismatch, truncation, or hash mismatch.
    pub fn verify_package_bytes(&self, bytes: &[u8]) -> Result<VerifiedPackage, UpdaterError> {
        let package = self.manifest().package();
        let expected = package.size();
        let actual = u64::try_from(bytes.len()).map_err(|_| UpdaterError::PackageTooLarge {
            max: MAX_PACKAGE_BYTES,
            actual: u64::MAX,
        })?;
        if actual < expected {
            return Err(UpdaterError::PackageTruncated { expected, actual });
        }
        if actual > expected {
            return Err(UpdaterError::PackageSizeMismatch { expected, actual });
        }
        let digest = Sha256Digest::hash(bytes);
        if digest != package.sha256() {
            return Err(UpdaterError::PackageHashMismatch {
                expected: package.sha256().to_hex(),
                actual: digest.to_hex(),
            });
        }
        Ok(VerifiedPackage {
            size: actual,
            sha256: digest,
        })
    }
}

pub(crate) fn digest_from_hasher(hasher: Sha256) -> Sha256Digest {
    let digest = hasher.finalize();
    let mut bytes = [0u8; Sha256Digest::LEN];
    bytes.copy_from_slice(&digest);
    Sha256Digest::from_bytes(bytes)
}

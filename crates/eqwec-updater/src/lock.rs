//! Exclusive profile lock.
//!
//! One launcher process owns a profile at a time. The lock is advisory at the
//! operating-system level and is released automatically when the process
//! exits, so a crash never leaves a stale lock that requires manual cleanup.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::UpdaterError;

/// An exclusive lock on a Hub profile directory.
#[derive(Debug)]
pub struct ProfileLock {
    file: File,
    path: PathBuf,
}

impl ProfileLock {
    /// Acquires an exclusive lock on `path`.
    ///
    /// The file is created when absent. Its contents are diagnostic only and
    /// are rewritten by the process that holds the lock.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::ProfileLocked`] when another process holds the
    /// lock, and [`UpdaterError::Io`] when the file cannot be opened or
    /// written.
    pub fn acquire(path: &Path) -> Result<Self, UpdaterError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|error| UpdaterError::Io {
                detail: format!("open profile lock {}: {error}", path.display()),
            })?;

        match file.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err(UpdaterError::ProfileLocked);
            }
            Err(std::fs::TryLockError::Error(error)) => {
                return Err(UpdaterError::Io {
                    detail: format!("lock profile {}: {error}", path.display()),
                });
            }
        }

        file.set_len(0).map_err(|error| UpdaterError::Io {
            detail: format!("truncate profile lock {}: {error}", path.display()),
        })?;
        writeln!(
            file,
            "eqwec-hub pid={} version={}",
            std::process::id(),
            env!("CARGO_PKG_VERSION")
        )
        .map_err(|error| UpdaterError::Io {
            detail: format!("write profile lock {}: {error}", path.display()),
        })?;
        file.flush().map_err(|error| UpdaterError::Io {
            detail: format!("flush profile lock {}: {error}", path.display()),
        })?;

        Ok(Self {
            file,
            path: path.to_path_buf(),
        })
    }

    /// Returns the locked path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ProfileLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

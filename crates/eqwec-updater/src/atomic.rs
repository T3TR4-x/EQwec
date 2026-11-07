//! Bounded file reads and atomic replacement.
//!
//! Atomic replacement is only guaranteed within one filesystem. The caller
//! must ensure that the parent directory is on the same filesystem as the
//! target and that it is not attacker-controlled.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::error::UpdaterError;

/// Returns the file length, or `None` when the file does not exist.
pub(crate) fn metadata_len(path: &Path) -> Result<Option<u64>, UpdaterError> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(Some(metadata.len())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_error(error)),
    }
}

/// Reads a complete file into memory.
pub(crate) fn read_all(path: &Path) -> Result<Vec<u8>, UpdaterError> {
    fs::read(path).map_err(io_error)
}

/// Atomically replaces `path` with `bytes`.
///
/// The data is written to a sibling temporary file, flushed, renamed over the
/// target, and the parent directory is flushed on Unix. A failure before the
/// rename leaves the previous file intact. A failure after the rename means
/// durability is uncertain; the caller must re-read the file rather than
/// assume the old content is still present.
pub(crate) fn write_atomic(
    path: &Path,
    bytes: &[u8],
    operation_id: &str,
) -> Result<(), UpdaterError> {
    debug_assert!(!operation_id.is_empty());

    let file_name = path.file_name().ok_or_else(|| UpdaterError::AtomicWrite {
        detail: "target path has no file name".to_owned(),
    })?;
    let temp_name = format!("{}.tmp-{operation_id}", file_name.to_string_lossy());
    let temp_path = path.with_file_name(temp_name);

    let result = write_and_rename(&temp_path, path, bytes);
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result.map_err(|error| UpdaterError::AtomicWrite {
        detail: UpdaterError::bounded_detail(error.to_string()),
    })
}

fn write_and_rename(temp_path: &Path, target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp_path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(temp_path, target)?;
    sync_parent(target)
}

#[cfg(unix)]
fn sync_parent(path: &Path) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

pub(crate) fn io_error(error: std::io::Error) -> UpdaterError {
    UpdaterError::Io {
        detail: UpdaterError::bounded_detail(error.to_string()),
    }
}

/// Flushes the parent directory of `path` where the platform supports it.
pub(crate) fn sync_parent_dir(path: &Path) -> Result<(), UpdaterError> {
    sync_parent(path).map_err(io_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "eqwec-atomic-{}-{}",
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

    #[test]
    fn replaces_an_existing_file() {
        let dir = TestDir::new();
        let target = dir.path().join("registry.json");
        fs::write(&target, b"old").unwrap();

        write_atomic(&target, b"new", "0123456789abcdef0123456789abcdef").unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"new");

        let entries: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(entries, [std::ffi::OsString::from("registry.json")]);
    }

    #[test]
    fn failure_leaves_the_previous_file_and_removes_the_temp() {
        let dir = TestDir::new();
        let target = dir.path().join("registry.json");
        fs::write(&target, b"old").unwrap();

        // Renaming a file over a non-empty directory fails.
        let blocking = dir.path().join("blocking");
        fs::create_dir(&blocking).unwrap();
        fs::write(blocking.join("child"), b"x").unwrap();

        let error =
            write_atomic(&blocking, b"new", "0123456789abcdef0123456789abcdef").unwrap_err();
        assert!(matches!(error, UpdaterError::AtomicWrite { .. }));
        assert_eq!(fs::read(&target).unwrap(), b"old");
        assert!(blocking.join("child").exists());

        let leftovers: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|entry| {
                let name = entry.unwrap().file_name();
                let name = name.to_string_lossy().into_owned();
                name.contains(".tmp-").then_some(name)
            })
            .collect();
        assert!(leftovers.is_empty(), "temp files remain: {leftovers:?}");
    }
}

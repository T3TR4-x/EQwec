//! Pointer-first launch resolution, health checks, and process spawning.
//!
//! The launcher never trusts the registry alone: `active.json` is the only
//! activation pointer. A missing, invalid, or mismatched pointer is a repair
//! state, not a fallback.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use eqwec_core::ProductId;
use eqwec_security::SafeRelativePath;
use eqwec_updater::{ActivePointer, InstallLayout, InstallationRecord};

use crate::error::LauncherError;
use crate::paths::HostOs;

/// Maximum time the launcher waits for a staged product to exit cleanly.
pub const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(10);

const HEALTH_CHECK_POLL: Duration = Duration::from_millis(25);
const UNIX_EXECUTE_BITS: u32 = 0o111;

/// A resolved entry point and its tree root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedEntry {
    /// Absolute path to the product executable.
    pub entry: PathBuf,
    /// Absolute path to the active tree root.
    pub tree_root: PathBuf,
}

/// Returns the entry-point path relative to a product tree.
///
/// # Errors
///
/// Returns [`LauncherError::Rejected`] when the product identifier cannot form
/// a safe relative path.
pub fn entry_relative_path(product_id: &ProductId) -> Result<SafeRelativePath, LauncherError> {
    let name = if cfg!(windows) {
        format!("bin/{product_id}.exe")
    } else {
        format!("bin/{product_id}")
    };
    SafeRelativePath::new(name).map_err(|error| LauncherError::Rejected(error.to_string()))
}

/// Resolves the active entry point from a registry record and its pointer.
///
/// # Errors
///
/// Returns [`LauncherError::Pointer`] when the pointer does not match the
/// record, and [`LauncherError::Launch`] when the tree or entry point is
/// missing, is a symlink, or is not a regular executable file.
pub fn resolve_entry(
    layout: &InstallLayout,
    record: &InstallationRecord,
    pointer: &ActivePointer,
) -> Result<ResolvedEntry, LauncherError> {
    if pointer.tree() != record.active_tree() || pointer.package_sha256() != record.package_sha256()
    {
        return Err(LauncherError::Pointer(
            "active pointer does not match the registry".to_owned(),
        ));
    }
    let tree_root = layout.tree_path(record.product_id(), record.active_tree());
    if !tree_root.starts_with(layout.product_root(record.product_id())) {
        return Err(LauncherError::Launch {
            detail: "active tree escapes the product root".to_owned(),
        });
    }
    let entry = resolve_tree_entry(&tree_root, record.product_id())?;
    Ok(ResolvedEntry { entry, tree_root })
}

/// Resolves the entry point inside an arbitrary product tree.
///
/// The tree root, its `bin` directory, and the entry point must be real
/// filesystem objects, not symlinks.
///
/// # Errors
///
/// Returns [`LauncherError::Launch`] when the tree or entry point is missing,
/// is a symlink, or is not a regular executable file.
pub fn resolve_tree_entry(
    tree_root: &Path,
    product_id: &ProductId,
) -> Result<PathBuf, LauncherError> {
    require_real_directory(tree_root, "product tree")?;
    let bin = tree_root.join("bin");
    require_real_directory(&bin, "bin directory")?;
    let relative = entry_relative_path(product_id)?;
    let entry = tree_root.join(relative.as_str());
    let metadata = std::fs::symlink_metadata(&entry).map_err(|error| LauncherError::Launch {
        detail: format!("inspect entry point {}: {error}", entry.display()),
    })?;
    if metadata.file_type().is_symlink() {
        return Err(LauncherError::Launch {
            detail: "entry point must not be a symlink".to_owned(),
        });
    }
    if !metadata.is_file() {
        return Err(LauncherError::Launch {
            detail: "entry point is not a regular file".to_owned(),
        });
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & UNIX_EXECUTE_BITS == 0 {
            return Err(LauncherError::Launch {
                detail: "entry point is not executable".to_owned(),
            });
        }
    }
    Ok(entry)
}

fn require_real_directory(path: &Path, label: &str) -> Result<(), LauncherError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| LauncherError::Launch {
        detail: format!("inspect {label} {}: {error}", path.display()),
    })?;
    if metadata.file_type().is_symlink() {
        return Err(LauncherError::Launch {
            detail: format!("{label} must not be a symlink"),
        });
    }
    if !metadata.is_dir() {
        return Err(LauncherError::Launch {
            detail: format!("{label} is not a directory"),
        });
    }
    Ok(())
}

/// Runs the staged product with the production ten-second bound.
///
/// # Errors
///
/// Returns [`LauncherError::HealthCheck`] when the entry point cannot be
/// resolved or spawned, exits nonzero, or does not exit within the bound.
pub fn health_check(tree_root: &Path, product_id: &ProductId) -> Result<(), LauncherError> {
    health_check_with_timeout(tree_root, product_id, HEALTH_CHECK_TIMEOUT)
}

/// Runs the staged product with an explicit timeout.
///
/// # Errors
///
/// Same as [`health_check`]. A timed-out child is killed and reaped before the
/// error is returned.
pub fn health_check_with_timeout(
    tree_root: &Path,
    product_id: &ProductId,
    timeout: Duration,
) -> Result<(), LauncherError> {
    let entry = resolve_tree_entry(tree_root, product_id)?;
    let mut child = Command::new(&entry)
        .current_dir(tree_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| LauncherError::HealthCheck {
            detail: format!("spawn {}: {error}", entry.display()),
        })?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => {
                return Err(LauncherError::HealthCheck {
                    detail: format!("entry point exited with {status}"),
                });
            }
            Ok(None) => {}
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(LauncherError::HealthCheck {
                    detail: format!("wait for entry point: {error}"),
                });
            }
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(LauncherError::HealthCheck {
                detail: format!(
                    "entry point did not exit within {} seconds",
                    timeout.as_secs_f64()
                ),
            });
        }
        std::thread::sleep(HEALTH_CHECK_POLL);
    }
}

/// Spawns a resolved product and returns its process id.
///
/// The child inherits stdio, the working directory is the tree root, and no
/// shell is involved. A reaper thread waits for the child so it cannot become
/// a zombie.
///
/// # Errors
///
/// Returns [`LauncherError::Launch`] when the process cannot be spawned.
pub fn spawn(entry: &Path, tree_root: &Path) -> Result<u32, LauncherError> {
    let mut child = Command::new(entry)
        .current_dir(tree_root)
        .spawn()
        .map_err(|error| LauncherError::Launch {
            detail: format!("spawn {}: {error}", entry.display()),
        })?;
    let pid = child.id();
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(pid)
}

/// Opens a folder in the platform file manager.
///
/// # Errors
///
/// Returns [`LauncherError::Launch`] when the opener cannot be spawned.
pub fn open_folder(path: &Path) -> Result<(), LauncherError> {
    let opener = match HostOs::current() {
        HostOs::MacOs => "open",
        HostOs::Windows => "explorer",
        HostOs::Linux => "xdg-open",
    };
    Command::new(opener)
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|mut child| {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        })
        .map_err(|error| LauncherError::Launch {
            detail: format!("open {}: {error}", path.display()),
        })
}

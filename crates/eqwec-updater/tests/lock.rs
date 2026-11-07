//! Profile-lock behavior.
//!
//! The lock is advisory and process-scoped. A second launcher must fail
//! instead of writing the registry concurrently.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use eqwec_updater::{ProfileLock, UpdaterError};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn temp_base() -> PathBuf {
    let unique = NEXT.fetch_add(1, Ordering::Relaxed);
    let base = std::env::temp_dir().join(format!("eqwec-lock-{}-{unique}", std::process::id()));
    fs::create_dir_all(&base).expect("create temp base");
    base
}

#[test]
fn second_acquire_is_rejected() {
    let base = temp_base();
    let path = base.join("profile.lock");
    let first = ProfileLock::acquire(&path).expect("first lock");
    let second = ProfileLock::acquire(&path);
    assert!(matches!(second, Err(UpdaterError::ProfileLocked)));
    drop(first);
    fs::remove_dir_all(base).expect("cleanup");
}

#[test]
fn lock_is_released_on_drop() {
    let base = temp_base();
    let path = base.join("profile.lock");
    let first = ProfileLock::acquire(&path).expect("first lock");
    drop(first);
    let second = ProfileLock::acquire(&path).expect("reacquire after drop");
    drop(second);
    fs::remove_dir_all(base).expect("cleanup");
}

#[test]
fn different_paths_do_not_conflict() {
    let base = temp_base();
    let first = ProfileLock::acquire(&base.join("a.lock")).expect("first lock");
    let second = ProfileLock::acquire(&base.join("b.lock")).expect("second lock");
    drop(first);
    drop(second);
    fs::remove_dir_all(base).expect("cleanup");
}

#[test]
fn lock_file_is_not_empty() {
    let base = temp_base();
    let path = base.join("profile.lock");
    let lock = ProfileLock::acquire(&path).expect("lock");
    // Windows locks the region, so the file cannot be read until the handle is
    // released. The diagnostics are durable and must survive the drop.
    drop(lock);
    let contents = fs::read_to_string(&path).expect("read lock file");
    assert!(!contents.is_empty());
    fs::remove_dir_all(base).expect("cleanup");
}

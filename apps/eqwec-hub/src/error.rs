//! Typed launcher errors.

use thiserror::Error;

/// A launcher failure with an actionable message.
#[derive(Debug, Error)]
pub enum LauncherError {
    /// A required environment variable is absent.
    #[error("environment variable {name} is not set")]
    MissingEnvironment {
        /// Variable name.
        name: &'static str,
    },
    /// The data-directory override is not absolute.
    #[error("data directory must be absolute: {path}")]
    RelativeDataDir {
        /// Rejected value.
        path: String,
    },
    /// The data-directory override contains `.` or `..`.
    #[error("data directory must be normalized: {path}")]
    UnnormalizedDataDir {
        /// Rejected value.
        path: String,
    },
    /// The profile is locked by another process.
    #[error("the profile is locked by another EQwec process")]
    ProfileLocked,
    /// The registry could not be loaded.
    #[error("registry error: {0}")]
    Registry(String),
    /// The active pointer is missing or inconsistent.
    #[error("activation state error: {0}")]
    Pointer(String),
    /// A filesystem operation failed.
    #[error("I/O error: {detail}")]
    Io {
        /// Bounded diagnostic detail.
        detail: String,
    },
    /// A product could not be launched.
    #[error("launch failed: {detail}")]
    Launch {
        /// Bounded diagnostic detail.
        detail: String,
    },
    /// An updater operation failed.
    #[error("updater error: {0}")]
    Updater(String),
    /// The embedded trust store is invalid.
    #[error("trust store error: {0}")]
    Trust(String),
}

impl From<eqwec_updater::UpdaterError> for LauncherError {
    fn from(error: eqwec_updater::UpdaterError) -> Self {
        match error {
            eqwec_updater::UpdaterError::ProfileLocked => Self::ProfileLocked,
            other => Self::Updater(other.to_string()),
        }
    }
}

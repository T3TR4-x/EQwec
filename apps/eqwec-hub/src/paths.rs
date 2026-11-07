//! Platform data-directory resolution.

use std::path::PathBuf;

use crate::error::LauncherError;

/// The operating system family used for path rules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostOs {
    /// Apple macOS.
    MacOs,
    /// Microsoft Windows.
    Windows,
    /// Linux and other XDG systems.
    Linux,
}

impl HostOs {
    /// Returns the host operating system family.
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Linux
        }
    }

    /// Returns the canonical lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MacOs => "macos",
            Self::Windows => "windows",
            Self::Linux => "linux",
        }
    }
}

/// Resolves the profile data directory.
///
/// `var` supplies environment variables. `EQWEC_DATA_DIR` overrides the
/// platform default and must be absolute and normalized.
///
/// # Errors
///
/// Returns [`LauncherError::MissingEnvironment`] when a required variable is
/// absent and [`LauncherError::RelativeDataDir`] or
/// [`LauncherError::UnnormalizedDataDir`] for an invalid override.
pub fn data_dir_for(
    os: HostOs,
    var: &dyn Fn(&str) -> Option<String>,
) -> Result<PathBuf, LauncherError> {
    if let Some(value) = var("EQWEC_DATA_DIR") {
        return validate_override(value);
    }
    match os {
        HostOs::MacOs => {
            let home = require(var, "HOME")?;
            Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("EQwec"))
        }
        HostOs::Windows => {
            let local = require(var, "LOCALAPPDATA")?;
            Ok(PathBuf::from(local).join("EQwec"))
        }
        HostOs::Linux => {
            if let Some(xdg) = var("XDG_DATA_HOME")
                && is_absolute_normalized(&xdg)
            {
                return Ok(PathBuf::from(xdg).join("eqwec"));
            }
            let home = require(var, "HOME")?;
            Ok(PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("eqwec"))
        }
    }
}

/// Resolves the profile data directory for the current process.
///
/// # Errors
///
/// Same as [`data_dir_for`].
pub fn data_dir() -> Result<PathBuf, LauncherError> {
    data_dir_for(HostOs::current(), &|name| std::env::var(name).ok())
}

fn require(
    var: &dyn Fn(&str) -> Option<String>,
    name: &'static str,
) -> Result<String, LauncherError> {
    var(name).ok_or(LauncherError::MissingEnvironment { name })
}

fn validate_override(value: String) -> Result<PathBuf, LauncherError> {
    if !is_absolute_normalized(&value) {
        if !std::path::Path::new(&value).is_absolute() {
            return Err(LauncherError::RelativeDataDir { path: value });
        }
        return Err(LauncherError::UnnormalizedDataDir { path: value });
    }
    Ok(PathBuf::from(value))
}

fn is_absolute_normalized(value: &str) -> bool {
    use std::path::Component;
    let path = std::path::Path::new(value);
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
}

#[cfg(test)]
mod tests {
    use super::{HostOs, data_dir_for};
    use crate::error::LauncherError;
    use std::path::PathBuf;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let pairs: Vec<(String, String)> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect();
        move |name| {
            pairs
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.clone())
        }
    }

    /// Builds an absolute path with the host platform's rules.
    fn absolute(parts: &[&str]) -> String {
        let mut path = if cfg!(windows) {
            PathBuf::from(r"C:\")
        } else {
            PathBuf::from("/")
        };
        for part in parts {
            path.push(part);
        }
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn macos_uses_application_support() {
        let home = absolute(&["Users", "test"]);
        let dir = data_dir_for(HostOs::MacOs, &env(&[("HOME", &home)])).expect("dir");
        assert_eq!(
            dir,
            PathBuf::from(absolute(&[
                "Users",
                "test",
                "Library",
                "Application Support",
                "EQwec"
            ]))
        );
    }

    #[test]
    fn windows_uses_local_app_data() {
        let local = absolute(&["Users", "test", "AppData", "Local"]);
        let dir = data_dir_for(HostOs::Windows, &env(&[("LOCALAPPDATA", &local)])).expect("dir");
        assert_eq!(
            dir,
            PathBuf::from(absolute(&["Users", "test", "AppData", "Local", "EQwec"]))
        );
    }

    #[test]
    fn linux_prefers_xdg_data_home() {
        let xdg = absolute(&["xdg"]);
        let home = absolute(&["home", "test"]);
        let dir = data_dir_for(
            HostOs::Linux,
            &env(&[("XDG_DATA_HOME", &xdg), ("HOME", &home)]),
        )
        .expect("dir");
        assert_eq!(dir, PathBuf::from(absolute(&["xdg", "eqwec"])));
    }

    #[test]
    fn linux_falls_back_to_home() {
        let home = absolute(&["home", "test"]);
        let dir = data_dir_for(HostOs::Linux, &env(&[("HOME", &home)])).expect("dir");
        assert_eq!(
            dir,
            PathBuf::from(absolute(&["home", "test", ".local", "share", "eqwec"]))
        );
    }

    #[test]
    fn override_wins_on_every_platform() {
        let custom = absolute(&["custom", "data"]);
        for os in [HostOs::MacOs, HostOs::Windows, HostOs::Linux] {
            let dir = data_dir_for(os, &env(&[("EQWEC_DATA_DIR", &custom)])).expect("dir");
            assert_eq!(dir, PathBuf::from(&custom));
        }
    }

    #[test]
    fn relative_override_is_rejected() {
        let error = data_dir_for(HostOs::Linux, &env(&[("EQWEC_DATA_DIR", "relative/path")]))
            .expect_err("must fail");
        assert!(matches!(error, LauncherError::RelativeDataDir { .. }));
    }

    #[test]
    fn unnormalized_override_is_rejected() {
        let mut path = if cfg!(windows) {
            PathBuf::from(r"C:\")
        } else {
            PathBuf::from("/")
        };
        path.push("custom");
        path.push("..");
        path.push("data");
        let unnormalized = path.to_string_lossy().into_owned();
        let error = data_dir_for(HostOs::Linux, &env(&[("EQWEC_DATA_DIR", &unnormalized)]))
            .expect_err("must fail");
        assert!(matches!(error, LauncherError::UnnormalizedDataDir { .. }));
    }

    #[test]
    fn missing_home_is_reported() {
        let error = data_dir_for(HostOs::MacOs, &env(&[])).expect_err("must fail");
        assert!(matches!(
            error,
            LauncherError::MissingEnvironment { name: "HOME" }
        ));
    }
}

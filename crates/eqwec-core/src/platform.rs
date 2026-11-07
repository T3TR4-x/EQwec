//! Supported platforms and architectures.

use core::fmt;

use crate::error::CoreError;

/// A supported operating-system platform.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Platform {
    /// Microsoft Windows.
    Windows,
    /// Apple macOS.
    MacOs,
    /// Linux.
    Linux,
}

impl Platform {
    /// Returns the canonical lowercase name used in manifests.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::MacOs => "macos",
            Self::Linux => "linux",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::str::FromStr for Platform {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "windows" => Ok(Self::Windows),
            "macos" => Ok(Self::MacOs),
            "linux" => Ok(Self::Linux),
            other => Err(CoreError::UnknownPlatform {
                value: other.to_owned(),
            }),
        }
    }
}

/// A supported CPU architecture.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Architecture {
    /// The x86-64 architecture.
    X86_64,
    /// The 64-bit ARM architecture.
    Aarch64,
}

impl Architecture {
    /// Returns the canonical lowercase name used in manifests.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
        }
    }
}

impl fmt::Display for Architecture {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::str::FromStr for Architecture {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "x86_64" => Ok(Self::X86_64),
            "aarch64" => Ok(Self::Aarch64),
            other => Err(CoreError::UnknownArchitecture {
                value: other.to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platforms_round_trip_through_text() {
        for platform in [Platform::Windows, Platform::MacOs, Platform::Linux] {
            let parsed: Platform = platform.as_str().parse().expect("valid platform");
            assert_eq!(parsed, platform);
        }
    }

    #[test]
    fn architectures_round_trip_through_text() {
        for architecture in [Architecture::X86_64, Architecture::Aarch64] {
            let parsed: Architecture = architecture.as_str().parse().expect("valid architecture");
            assert_eq!(parsed, architecture);
        }
    }

    #[test]
    fn rejects_unknown_platform() {
        let error: CoreError = "freebsd".parse::<Platform>().expect_err("must fail");
        assert_eq!(
            error,
            CoreError::UnknownPlatform {
                value: "freebsd".to_owned()
            }
        );
    }

    #[test]
    fn rejects_unknown_architecture() {
        let error: CoreError = "riscv64".parse::<Architecture>().expect_err("must fail");
        assert_eq!(
            error,
            CoreError::UnknownArchitecture {
                value: "riscv64".to_owned()
            }
        );
    }
}

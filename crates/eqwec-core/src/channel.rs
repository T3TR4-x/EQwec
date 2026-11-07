//! Release channels.

use core::fmt;

use crate::error::CoreError;

/// A product release channel.
///
/// Channels are ordered from most stable to least stable. The ordering is
/// meaningful for policy decisions; it is not a version comparison.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Channel {
    /// The stable channel.
    Stable,
    /// The beta channel.
    Beta,
    /// The alpha channel.
    Alpha,
    /// The internal development channel. It is not distributed publicly.
    Dev,
}

impl Channel {
    /// Returns the canonical lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Alpha => "alpha",
            Self::Dev => "dev",
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::str::FromStr for Channel {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "stable" => Ok(Self::Stable),
            "beta" => Ok(Self::Beta),
            "alpha" => Ok(Self::Alpha),
            "dev" => Ok(Self::Dev),
            other => Err(CoreError::UnknownChannel {
                value: other.to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channels_round_trip_through_text() {
        for channel in [Channel::Stable, Channel::Beta, Channel::Alpha, Channel::Dev] {
            let parsed: Channel = channel.as_str().parse().expect("valid channel");
            assert_eq!(parsed, channel);
        }
    }

    #[test]
    fn rejects_unknown_channel() {
        let error: CoreError = "nightly".parse::<Channel>().expect_err("must fail");
        assert_eq!(
            error,
            CoreError::UnknownChannel {
                value: "nightly".to_owned()
            }
        );
        assert!(error.to_string().contains("unknown release channel"));
    }

    #[test]
    fn channel_ordering_is_stable_first() {
        assert!(Channel::Stable < Channel::Beta);
        assert!(Channel::Beta < Channel::Alpha);
        assert!(Channel::Alpha < Channel::Dev);
    }
}

//! Typed errors for core primitives.

use core::fmt;

/// An error produced while constructing or parsing a core value.
///
/// The variants carry only the information needed to explain the failure.
/// They never contain secrets, full user paths, or untrusted payloads.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CoreError {
    /// A product identifier failed validation.
    InvalidProductId {
        /// A short, human-readable reason.
        reason: &'static str,
    },
    /// A scope identifier failed validation.
    InvalidScopeId {
        /// A short, human-readable reason.
        reason: &'static str,
    },
    /// A release channel name is unknown.
    UnknownChannel {
        /// The rejected value.
        value: String,
    },
    /// A platform name is unknown.
    UnknownPlatform {
        /// The rejected value.
        value: String,
    },
    /// An architecture name is unknown.
    UnknownArchitecture {
        /// The rejected value.
        value: String,
    },
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProductId { reason } => {
                write!(formatter, "invalid product identifier: {reason}")
            }
            Self::InvalidScopeId { reason } => {
                write!(formatter, "invalid scope identifier: {reason}")
            }
            Self::UnknownChannel { value } => {
                write!(formatter, "unknown release channel: {value}")
            }
            Self::UnknownPlatform { value } => {
                write!(formatter, "unknown platform: {value}")
            }
            Self::UnknownArchitecture { value } => {
                write!(formatter, "unknown architecture: {value}")
            }
        }
    }
}

impl std::error::Error for CoreError {}

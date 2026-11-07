#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Core primitives shared by every EQwec product.
//!
//! This crate deliberately has no third-party dependencies. It defines the
//! stable identifiers, release channels, and platform values that cross
//! product, manifest, and IPC boundaries.
//!
//! Identifiers are opaque: they are never derived from a display name, a
//! filesystem path, or a user-visible label. Validation is bounded and
//! deterministic so the same value is accepted everywhere.

pub mod channel;
pub mod error;
pub mod id;
pub mod platform;

pub use channel::Channel;
pub use error::CoreError;
pub use id::{MAX_ID_LEN, ProductId, ScopeId};
pub use platform::{Architecture, Platform};

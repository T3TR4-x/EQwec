//! Manifest trust verification.

use eqwec_core::{Architecture, Channel, Platform};
use eqwec_security::SecurityError;

use crate::error::UpdaterError;
use crate::manifest::Manifest;
use crate::time::{CLOCK_SKEW_SECONDS, Timestamp};
use crate::trust::TrustStore;

/// The policy context for manifest verification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerificationContext {
    channel: Channel,
    platform: Platform,
    architecture: Architecture,
    now: Timestamp,
    minimum_sequence: Option<u64>,
}

impl VerificationContext {
    /// Creates a verification context.
    #[must_use]
    pub const fn new(
        channel: Channel,
        platform: Platform,
        architecture: Architecture,
        now: Timestamp,
    ) -> Self {
        Self {
            channel,
            platform,
            architecture,
            now,
            minimum_sequence: None,
        }
    }

    /// Sets the minimum accepted metadata sequence for rollback protection.
    #[must_use]
    pub const fn with_minimum_sequence(mut self, minimum_sequence: u64) -> Self {
        self.minimum_sequence = Some(minimum_sequence);
        self
    }

    /// Returns the requested channel.
    #[must_use]
    pub const fn channel(self) -> Channel {
        self.channel
    }

    /// Returns the requested platform.
    #[must_use]
    pub const fn platform(self) -> Platform {
        self.platform
    }

    /// Returns the requested architecture.
    #[must_use]
    pub const fn architecture(self) -> Architecture {
        self.architecture
    }

    /// Returns the verification time.
    #[must_use]
    pub const fn now(self) -> Timestamp {
        self.now
    }

    /// Returns the minimum metadata sequence, when set.
    #[must_use]
    pub const fn minimum_sequence(self) -> Option<u64> {
        self.minimum_sequence
    }
}

/// A manifest whose signature and policy checks passed.
///
/// The inner manifest is private so a caller cannot promote an unverified
/// document by constructing this type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedManifest(Manifest);

impl VerifiedManifest {
    /// Returns the verified manifest.
    #[must_use]
    pub fn manifest(&self) -> &Manifest {
        &self.0
    }

    pub(crate) fn new(manifest: Manifest) -> Self {
        Self(manifest)
    }
}

impl Manifest {
    /// Verifies the manifest against a pinned trust store and policy context.
    ///
    /// Checks run in the order defined by the Hub schema: channel and target,
    /// current-time validity, key lookup and policy, strict Ed25519 signature,
    /// then metadata-sequence regression.
    ///
    /// # Errors
    ///
    /// Returns a typed error on the first failed check. There is no insecure
    /// fallback.
    pub fn verify(
        self,
        trust_store: &TrustStore,
        context: &VerificationContext,
    ) -> Result<VerifiedManifest, UpdaterError> {
        let channel = self.channel();
        if channel != context.channel() {
            return Err(UpdaterError::ChannelMismatch {
                expected: context.channel(),
                found: channel,
            });
        }

        let target = self.target();
        if target.os() != context.platform() || target.architecture() != context.architecture() {
            return Err(UpdaterError::TargetMismatch {
                expected: format!(
                    "{}-{}",
                    context.platform().as_str(),
                    context.architecture().as_str()
                ),
                found: format!(
                    "{}-{}",
                    target.os().as_str(),
                    target.architecture().as_str()
                ),
            });
        }

        let now = context.now().unix_seconds();
        let issued_at = self.issued_at().unix_seconds();
        let expires_at = self.expires_at().unix_seconds();
        if now.saturating_add(CLOCK_SKEW_SECONDS) < issued_at {
            return Err(UpdaterError::ManifestNotYetValid);
        }
        if now > expires_at.saturating_add(CLOCK_SKEW_SECONDS) {
            return Err(UpdaterError::ManifestExpired);
        }

        let signature = self.signature();
        let key =
            trust_store
                .get(signature.key_id())
                .ok_or_else(|| UpdaterError::UnknownSigningKey {
                    key_id: signature.key_id().to_string(),
                })?;
        if key.is_revoked() {
            return Err(UpdaterError::SigningKeyRevoked {
                key_id: key.key_id().to_string(),
            });
        }
        if key.key_epoch() != signature.key_epoch() {
            return Err(UpdaterError::SigningKeyEpochMismatch {
                expected: key.key_epoch(),
                found: signature.key_epoch(),
            });
        }
        if !key.channels().contains(&channel) {
            return Err(UpdaterError::SigningKeyChannelNotAllowed {
                key_id: key.key_id().to_string(),
                channel,
            });
        }

        let signing_time = signature.signing_time().unix_seconds();
        if signing_time.saturating_add(CLOCK_SKEW_SECONDS) < key.not_before().unix_seconds() {
            return Err(UpdaterError::SigningKeyNotYetValid);
        }
        if signing_time
            > key
                .not_after()
                .unix_seconds()
                .saturating_add(CLOCK_SKEW_SECONDS)
        {
            return Err(UpdaterError::SigningKeyExpired);
        }

        let payload = self.canonical_payload();
        key.public_key()
            .verify_strict(&payload, signature.signature())
            .map_err(|error| match error {
                SecurityError::SignatureVerification => UpdaterError::SignatureInvalid,
                other => UpdaterError::Security(other),
            })?;

        if let Some(minimum) = context.minimum_sequence()
            && self.metadata_sequence() < minimum
        {
            return Err(UpdaterError::MetadataSequenceRegression {
                minimum,
                found: self.metadata_sequence(),
            });
        }

        Ok(VerifiedManifest::new(self))
    }
}

/// Parses and verifies a manifest in one step.
///
/// # Errors
///
/// Returns a typed error when parsing or any verification check fails.
pub fn parse_and_verify(
    bytes: &[u8],
    trust_store: &TrustStore,
    context: &VerificationContext,
) -> Result<VerifiedManifest, UpdaterError> {
    Manifest::parse(bytes)?.verify(trust_store, context)
}

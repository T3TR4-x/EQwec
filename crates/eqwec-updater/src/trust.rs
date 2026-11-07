//! Pinned signing-key trust store.

use eqwec_core::Channel;
use eqwec_security::{Ed25519PublicKey, KeyId};
use serde::Deserialize;

use crate::error::UpdaterError;
use crate::time::{MAX_KEY_VALIDITY_SECONDS, Timestamp};

/// Maximum encoded trust-store size in bytes.
pub const MAX_TRUST_STORE_BYTES: usize = 65_536;

/// Maximum number of keys in a trust store.
pub const MAX_TRUST_KEYS: usize = 64;

const TRUST_STORE_SCHEMA_VERSION: u64 = 1;
const ED25519: &str = "ed25519";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireTrustStore {
    schema_version: u64,
    keys: Vec<WireTrustedKey>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireTrustedKey {
    key_id: String,
    algorithm: String,
    public_key: String,
    key_epoch: u64,
    channels: Vec<String>,
    not_before: String,
    not_after: String,
    revoked: bool,
}

/// A pinned public key and its policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedKey {
    key_id: KeyId,
    public_key: Ed25519PublicKey,
    key_epoch: u64,
    channels: Vec<Channel>,
    not_before: Timestamp,
    not_after: Timestamp,
    revoked: bool,
}

impl TrustedKey {
    /// Creates a trusted key.
    ///
    /// Channels are sorted into canonical order. The validity window must be
    /// positive and at most ten years.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the epoch is zero, channels are empty or
    /// duplicated, or the validity window is invalid.
    pub fn new(
        key_id: KeyId,
        public_key: Ed25519PublicKey,
        key_epoch: u64,
        mut channels: Vec<Channel>,
        not_before: Timestamp,
        not_after: Timestamp,
    ) -> Result<Self, UpdaterError> {
        if key_epoch == 0 {
            return Err(UpdaterError::InvalidTrustKey {
                reason: "key_epoch must be at least 1".to_owned(),
            });
        }
        if channels.is_empty() {
            return Err(UpdaterError::InvalidTrustKey {
                reason: "at least one channel is required".to_owned(),
            });
        }
        let original_len = channels.len();
        channels.sort_unstable();
        channels.dedup();
        if channels.len() != original_len {
            return Err(UpdaterError::InvalidTrustKey {
                reason: "duplicate channel".to_owned(),
            });
        }
        validate_key_window(not_before, not_after)?;
        Ok(Self {
            key_id,
            public_key,
            key_epoch,
            channels,
            not_before,
            not_after,
            revoked: false,
        })
    }

    /// Marks the key as revoked or active.
    #[must_use]
    pub fn with_revoked(mut self, revoked: bool) -> Self {
        self.revoked = revoked;
        self
    }

    /// Returns the key identifier.
    #[must_use]
    pub fn key_id(&self) -> &KeyId {
        &self.key_id
    }

    /// Returns the public key.
    #[must_use]
    pub fn public_key(&self) -> &Ed25519PublicKey {
        &self.public_key
    }

    /// Returns the key epoch.
    #[must_use]
    pub const fn key_epoch(&self) -> u64 {
        self.key_epoch
    }

    /// Returns the allowed channels in canonical order.
    #[must_use]
    pub fn channels(&self) -> &[Channel] {
        &self.channels
    }

    /// Returns the start of the key validity window.
    #[must_use]
    pub const fn not_before(&self) -> Timestamp {
        self.not_before
    }

    /// Returns the end of the key validity window.
    #[must_use]
    pub const fn not_after(&self) -> Timestamp {
        self.not_after
    }

    /// Returns whether the key is revoked.
    #[must_use]
    pub const fn is_revoked(&self) -> bool {
        self.revoked
    }
}

/// A pinned set of trusted signing keys.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrustStore {
    keys: Vec<TrustedKey>,
}

impl TrustStore {
    /// Creates a trust store from keys.
    ///
    /// Keys are sorted by identifier. Duplicate identifiers and more than 64
    /// keys are rejected.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a duplicate identifier or too many keys.
    pub fn new(mut keys: Vec<TrustedKey>) -> Result<Self, UpdaterError> {
        if keys.len() > MAX_TRUST_KEYS {
            return Err(UpdaterError::TooManyTrustKeys {
                max: MAX_TRUST_KEYS,
                actual: keys.len(),
            });
        }
        keys.sort_by(|left, right| left.key_id.cmp(&right.key_id));
        for pair in keys.windows(2) {
            if pair[0].key_id == pair[1].key_id {
                return Err(UpdaterError::DuplicateTrustKey {
                    key_id: pair[0].key_id.to_string(),
                });
            }
        }
        Ok(Self { keys })
    }

    /// Parses a trust store from bounded JSON.
    ///
    /// # Errors
    ///
    /// Returns a typed error for oversized, malformed, duplicate, unknown, or
    /// unsorted input.
    pub fn parse(bytes: &[u8]) -> Result<Self, UpdaterError> {
        if bytes.len() > MAX_TRUST_STORE_BYTES {
            return Err(UpdaterError::TrustStoreTooLarge {
                max: MAX_TRUST_STORE_BYTES,
                actual: bytes.len(),
            });
        }
        let text = std::str::from_utf8(bytes).map_err(|_| UpdaterError::InvalidUtf8)?;
        let mut deserializer = serde_json::Deserializer::from_str(text);
        let wire = WireTrustStore::deserialize(&mut deserializer).map_err(map_json_error)?;
        deserializer.end().map_err(|_| UpdaterError::TrailingData)?;

        if wire.schema_version != TRUST_STORE_SCHEMA_VERSION {
            return Err(UpdaterError::UnsupportedSchemaVersion {
                found: wire.schema_version,
                supported: TRUST_STORE_SCHEMA_VERSION,
            });
        }
        if wire.keys.len() > MAX_TRUST_KEYS {
            return Err(UpdaterError::TooManyTrustKeys {
                max: MAX_TRUST_KEYS,
                actual: wire.keys.len(),
            });
        }

        let mut keys = Vec::with_capacity(wire.keys.len());
        for key in wire.keys {
            keys.push(parse_key(key)?);
        }
        for pair in keys.windows(2) {
            if pair[0].key_id > pair[1].key_id {
                return Err(UpdaterError::TrustStoreOrder);
            }
            if pair[0].key_id == pair[1].key_id {
                return Err(UpdaterError::DuplicateTrustKey {
                    key_id: pair[0].key_id.to_string(),
                });
            }
        }
        Ok(Self { keys })
    }

    /// Returns the key with the given identifier.
    #[must_use]
    pub fn get(&self, key_id: &KeyId) -> Option<&TrustedKey> {
        self.keys
            .binary_search_by(|key| key.key_id.cmp(key_id))
            .ok()
            .map(|index| &self.keys[index])
    }

    /// Returns all keys in identifier order.
    #[must_use]
    pub fn keys(&self) -> &[TrustedKey] {
        &self.keys
    }

    /// Returns whether the store has no keys.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

fn parse_key(wire: WireTrustedKey) -> Result<TrustedKey, UpdaterError> {
    if wire.algorithm != ED25519 {
        return Err(UpdaterError::InvalidTrustKey {
            reason: format!("unsupported algorithm: {}", wire.algorithm),
        });
    }
    let key_id = wire.key_id.parse::<KeyId>()?;
    let public_key = Ed25519PublicKey::parse_hex(&wire.public_key)?;
    if wire.key_epoch == 0 {
        return Err(UpdaterError::InvalidTrustKey {
            reason: "key_epoch must be at least 1".to_owned(),
        });
    }
    if wire.channels.is_empty() {
        return Err(UpdaterError::InvalidTrustKey {
            reason: "at least one channel is required".to_owned(),
        });
    }
    let mut channels = Vec::with_capacity(wire.channels.len());
    for value in &wire.channels {
        let channel = value
            .parse::<Channel>()
            .map_err(|_| UpdaterError::InvalidChannel {
                value: value.clone(),
            })?;
        if channels.contains(&channel) {
            return Err(UpdaterError::InvalidTrustKey {
                reason: "duplicate channel".to_owned(),
            });
        }
        channels.push(channel);
    }
    let mut sorted = channels.clone();
    sorted.sort_unstable();
    if channels != sorted {
        return Err(UpdaterError::InvalidTrustKey {
            reason: "channels must be in canonical order".to_owned(),
        });
    }

    let not_before = Timestamp::parse(&wire.not_before)?;
    let not_after = Timestamp::parse(&wire.not_after)?;
    validate_key_window(not_before, not_after)?;

    Ok(TrustedKey {
        key_id,
        public_key,
        key_epoch: wire.key_epoch,
        channels,
        not_before,
        not_after,
        revoked: wire.revoked,
    })
}

fn validate_key_window(not_before: Timestamp, not_after: Timestamp) -> Result<(), UpdaterError> {
    if not_before >= not_after {
        return Err(UpdaterError::InvalidTrustKey {
            reason: "not_before must be earlier than not_after".to_owned(),
        });
    }
    let window = not_after.unix_seconds() - not_before.unix_seconds();
    if window > MAX_KEY_VALIDITY_SECONDS {
        return Err(UpdaterError::InvalidTrustKey {
            reason: "key validity window exceeds ten years".to_owned(),
        });
    }
    Ok(())
}

fn map_json_error(error: serde_json::Error) -> UpdaterError {
    let detail = UpdaterError::bounded_detail(error.to_string());
    if detail.starts_with("duplicate field") || detail.starts_with("unknown field") {
        UpdaterError::UnknownOrDuplicateField { detail }
    } else {
        UpdaterError::MalformedJson { detail }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn key_json(key_id: &str, public_key: &str, channels: &[&str]) -> serde_json::Value {
        json!({
            "key_id": key_id,
            "algorithm": "ed25519",
            "public_key": public_key,
            "key_epoch": 1,
            "channels": channels,
            "not_before": "2026-01-01T00:00:00Z",
            "not_after": "2028-01-01T00:00:00Z",
            "revoked": false
        })
    }

    const TEST_PUBLIC_KEY: &str =
        "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";

    #[test]
    fn parses_a_canonical_store() {
        let bytes = serde_json::to_vec(&json!({
            "schema_version": 1,
            "keys": [
                key_json("alpha", TEST_PUBLIC_KEY, &["stable"]),
                key_json("beta", TEST_PUBLIC_KEY, &["beta", "alpha"]),
            ]
        }))
        .unwrap();
        let store = TrustStore::parse(&bytes).unwrap();
        assert_eq!(store.keys().len(), 2);
        assert_eq!(store.keys()[0].key_id().as_str(), "alpha");
        assert_eq!(store.keys()[1].channels(), &[Channel::Beta, Channel::Alpha]);
    }

    #[test]
    fn rejects_unsorted_keys() {
        let bytes = serde_json::to_vec(&json!({
            "schema_version": 1,
            "keys": [
                key_json("beta", TEST_PUBLIC_KEY, &["stable"]),
                key_json("alpha", TEST_PUBLIC_KEY, &["stable"]),
            ]
        }))
        .unwrap();
        assert_eq!(
            TrustStore::parse(&bytes).unwrap_err(),
            UpdaterError::TrustStoreOrder
        );
    }

    #[test]
    fn rejects_duplicate_keys() {
        let bytes = serde_json::to_vec(&json!({
            "schema_version": 1,
            "keys": [
                key_json("alpha", TEST_PUBLIC_KEY, &["stable"]),
                key_json("alpha", TEST_PUBLIC_KEY, &["beta"]),
            ]
        }))
        .unwrap();
        assert_eq!(
            TrustStore::parse(&bytes).unwrap_err(),
            UpdaterError::DuplicateTrustKey {
                key_id: "alpha".to_owned()
            }
        );
    }

    #[test]
    fn rejects_unsorted_channels() {
        let bytes = serde_json::to_vec(&json!({
            "schema_version": 1,
            "keys": [key_json("alpha", TEST_PUBLIC_KEY, &["beta", "stable"])]
        }))
        .unwrap();
        assert!(matches!(
            TrustStore::parse(&bytes).unwrap_err(),
            UpdaterError::InvalidTrustKey { .. }
        ));
    }

    #[test]
    fn rejects_unknown_and_duplicate_fields() {
        let unknown = br#"{"schema_version":1,"keys":[],"extra":true}"#;
        assert!(matches!(
            TrustStore::parse(unknown).unwrap_err(),
            UpdaterError::UnknownOrDuplicateField { .. }
        ));

        let duplicate = br#"{"schema_version":1,"schema_version":1,"keys":[]}"#;
        assert!(matches!(
            TrustStore::parse(duplicate).unwrap_err(),
            UpdaterError::UnknownOrDuplicateField { .. }
        ));
    }

    #[test]
    fn rejects_oversized_store() {
        let bytes = vec![b' '; MAX_TRUST_STORE_BYTES + 1];
        assert!(matches!(
            TrustStore::parse(&bytes).unwrap_err(),
            UpdaterError::TrustStoreTooLarge { .. }
        ));
    }

    #[test]
    fn rejects_trailing_data() {
        let mut bytes = br#"{"schema_version":1,"keys":[]}"#.to_vec();
        bytes.extend_from_slice(b" trailing");
        assert_eq!(
            TrustStore::parse(&bytes).unwrap_err(),
            UpdaterError::TrailingData
        );
    }
}

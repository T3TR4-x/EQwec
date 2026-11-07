//! Embedded developer-preview trust store.

use eqwec_updater::TrustStore;

use crate::error::LauncherError;

/// Public developer-preview trust store.
///
/// It contains only the public key of the non-production fixture seed. It must
/// never be used for release signing.
pub const DEV_TRUST_STORE_JSON: &[u8] = include_bytes!("../assets/dev-trust-store.json");

/// Parses the embedded trust store.
///
/// # Errors
///
/// Returns [`LauncherError::Trust`] when the embedded document is invalid.
pub fn trust_store() -> Result<TrustStore, LauncherError> {
    TrustStore::parse(DEV_TRUST_STORE_JSON)
        .map_err(|error| LauncherError::Trust(format!("embedded trust store: {error}")))
}

#[cfg(test)]
mod tests {
    use super::trust_store;
    use eqwec_core::Channel;
    use eqwec_security::KeyId;

    #[test]
    fn embedded_trust_store_parses() {
        let store = trust_store().expect("embedded store");
        let key = store
            .get(&KeyId::new("dev-fixture-2026").expect("key id"))
            .expect("fixture key");
        assert_eq!(key.key_epoch(), 1);
        assert!(key.channels().contains(&Channel::Stable));
        assert!(!key.is_revoked());
    }
}

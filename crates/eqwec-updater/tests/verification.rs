//! End-to-end manifest, trust, and package verification tests.
//!
//! The fixed vector below was produced independently with OpenSSL's Ed25519
//! implementation from a non-production test seed. The seed is public test
//! material and must never be used for a release.

use std::io::Cursor;

use eqwec_core::{Architecture, Channel, Platform};
use eqwec_security::{Ed25519PublicKey, KeyId};
use eqwec_updater::{
    MAX_MANIFEST_BYTES, Manifest, Timestamp, TrustStore, TrustedKey, UpdaterError,
    VerificationContext, VerifiedManifest,
};
use serde_json::{Value, json};

/// Fixed fixture package bytes.
const PACKAGE_BYTES: &[u8] = b"eqwec-hub-fixture-package-v1";

/// SHA-256 of [`PACKAGE_BYTES`].
const PACKAGE_SHA256: &str = "53bd9dc21b205305c958c14f672d19214d5af9157ef66d9c9091b14837fe42a3";

/// Public key derived from the non-production test seed
/// `000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f`.
const PUBLIC_KEY_HEX: &str = "03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8";

/// Ed25519 signature over [`CANONICAL_PAYLOAD`] from the same test seed.
const SIGNATURE_HEX: &str = "c29a1e0eaef0ce5c5086d7b8ef0775fb5f050deb4a342a7a66b246369c69ce2ef288e9464d63e542bac4b7c639694551925050bd33f73c42eb8bc5e021fa3505";

/// Exact canonical payload covered by [`SIGNATURE_HEX`].
const CANONICAL_PAYLOAD: &str = concat!(
    "{\"schema_version\":1,",
    "\"product_id\":\"eqwec-hub\",",
    "\"product_version\":\"0.1.0\",",
    "\"channel\":\"stable\",",
    "\"target\":{\"os\":\"macos\",\"architecture\":\"aarch64\",\"minimum_platform\":\"11.0\"},",
    "\"package\":{",
    "\"source\":\"https://packages.eqwec.example/hub/0.1.0/eqwec-hub-0.1.0-macos-aarch64.tar.zst\",",
    "\"size\":28,",
    "\"hashes\":[{\"algorithm\":\"sha256\",\"value\":\"",
    "53bd9dc21b205305c958c14f672d19214d5af9157ef66d9c9091b14837fe42a3",
    "\"}]},",
    "\"dependencies\":[],",
    "\"metadata_sequence\":1,",
    "\"issued_at\":\"2026-11-07T00:00:00Z\",",
    "\"expires_at\":\"2027-02-05T00:00:00Z\",",
    "\"release_notes\":\"Initial Hub MVP fixture.\"}",
);

const KEY_ID: &str = "release-2026-01";

fn manifest_value(signature_hex: &str) -> Value {
    json!({
        "schema_version": 1,
        "product_id": "eqwec-hub",
        "product_version": "0.1.0",
        "channel": "stable",
        "target": {
            "os": "macos",
            "architecture": "aarch64",
            "minimum_platform": "11.0"
        },
        "package": {
            "source": "https://packages.eqwec.example/hub/0.1.0/eqwec-hub-0.1.0-macos-aarch64.tar.zst",
            "size": PACKAGE_BYTES.len(),
            "hashes": [{ "algorithm": "sha256", "value": PACKAGE_SHA256 }]
        },
        "dependencies": [],
        "metadata_sequence": 1,
        "issued_at": "2026-11-07T00:00:00Z",
        "expires_at": "2027-02-05T00:00:00Z",
        "release_notes": "Initial Hub MVP fixture.",
        "signature": {
            "key_id": KEY_ID,
            "algorithm": "ed25519",
            "signature": signature_hex,
            "signing_time": "2026-11-07T00:00:00Z",
            "key_epoch": 1
        }
    })
}

fn manifest_bytes() -> Vec<u8> {
    serde_json::to_vec(&manifest_value(SIGNATURE_HEX)).unwrap()
}

fn parse_fixture() -> Manifest {
    Manifest::parse(&manifest_bytes()).unwrap()
}

fn timestamp(value: &str) -> Timestamp {
    Timestamp::parse(value).unwrap()
}

fn trust_store() -> TrustStore {
    let key = TrustedKey::new(
        KeyId::new(KEY_ID).unwrap(),
        Ed25519PublicKey::parse_hex(PUBLIC_KEY_HEX).unwrap(),
        1,
        vec![Channel::Stable],
        timestamp("2026-01-01T00:00:00Z"),
        timestamp("2028-01-01T00:00:00Z"),
    )
    .unwrap();
    TrustStore::new(vec![key]).unwrap()
}

fn context() -> VerificationContext {
    VerificationContext::new(
        Channel::Stable,
        Platform::MacOs,
        Architecture::Aarch64,
        timestamp("2026-11-07T12:00:00Z"),
    )
}

fn verify_fixture() -> VerifiedManifest {
    parse_fixture().verify(&trust_store(), &context()).unwrap()
}

#[test]
fn fixed_vector_matches_canonical_payload_and_signature() {
    let manifest = parse_fixture();
    assert_eq!(
        std::str::from_utf8(&manifest.canonical_payload()).unwrap(),
        CANONICAL_PAYLOAD
    );
    assert_eq!(manifest.signature().key_id().as_str(), KEY_ID);
    assert_eq!(manifest.signature().signature().to_hex(), SIGNATURE_HEX);
    assert_eq!(manifest.package().sha256().to_hex(), PACKAGE_SHA256);

    let verified = manifest.verify(&trust_store(), &context()).unwrap();
    assert_eq!(
        verified.manifest().canonical_payload(),
        CANONICAL_PAYLOAD.as_bytes()
    );
}

#[test]
fn fixed_vector_package_verifies() {
    let verified = verify_fixture();
    let package = verified.verify_package_bytes(PACKAGE_BYTES).unwrap();
    assert_eq!(package.size(), PACKAGE_BYTES.len() as u64);
    assert_eq!(package.sha256().to_hex(), PACKAGE_SHA256);

    let streamed = verified
        .verify_package_reader(Cursor::new(PACKAGE_BYTES))
        .unwrap();
    assert_eq!(streamed, package);
}

#[test]
fn accepts_noncanonical_json_formatting() {
    let pretty = serde_json::to_string_pretty(&manifest_value(SIGNATURE_HEX)).unwrap();
    let manifest = Manifest::parse(pretty.as_bytes()).unwrap();
    assert_eq!(manifest.canonical_payload(), CANONICAL_PAYLOAD.as_bytes());
    manifest.verify(&trust_store(), &context()).unwrap();
}

#[test]
fn rejects_tampered_payload() {
    let mut value = manifest_value(SIGNATURE_HEX);
    value["release_notes"] = json!("Tampered release notes.");
    let bytes = serde_json::to_vec(&value).unwrap();
    let manifest = Manifest::parse(&bytes).unwrap();
    assert_eq!(
        manifest.verify(&trust_store(), &context()).unwrap_err(),
        UpdaterError::SignatureInvalid
    );
}

#[test]
fn rejects_tampered_signature() {
    let mut signature = SIGNATURE_HEX.to_owned();
    signature.replace_range(0..2, "00");
    let bytes = serde_json::to_vec(&manifest_value(&signature)).unwrap();
    let manifest = Manifest::parse(&bytes).unwrap();
    assert_eq!(
        manifest.verify(&trust_store(), &context()).unwrap_err(),
        UpdaterError::SignatureInvalid
    );
}

#[test]
fn rejects_wrong_channel() {
    let beta = VerificationContext::new(
        Channel::Beta,
        Platform::MacOs,
        Architecture::Aarch64,
        timestamp("2026-11-07T12:00:00Z"),
    );
    assert_eq!(
        parse_fixture().verify(&trust_store(), &beta).unwrap_err(),
        UpdaterError::ChannelMismatch {
            expected: Channel::Beta,
            found: Channel::Stable,
        }
    );
}

#[test]
fn rejects_wrong_target() {
    let linux = VerificationContext::new(
        Channel::Stable,
        Platform::Linux,
        Architecture::Aarch64,
        timestamp("2026-11-07T12:00:00Z"),
    );
    assert!(matches!(
        parse_fixture().verify(&trust_store(), &linux).unwrap_err(),
        UpdaterError::TargetMismatch { .. }
    ));

    let x86 = VerificationContext::new(
        Channel::Stable,
        Platform::MacOs,
        Architecture::X86_64,
        timestamp("2026-11-07T12:00:00Z"),
    );
    assert!(matches!(
        parse_fixture().verify(&trust_store(), &x86).unwrap_err(),
        UpdaterError::TargetMismatch { .. }
    ));
}

#[test]
fn rejects_not_yet_valid_and_expired_manifests() {
    let early = VerificationContext::new(
        Channel::Stable,
        Platform::MacOs,
        Architecture::Aarch64,
        timestamp("2026-11-06T00:00:00Z"),
    );
    assert_eq!(
        parse_fixture().verify(&trust_store(), &early).unwrap_err(),
        UpdaterError::ManifestNotYetValid
    );

    let late = VerificationContext::new(
        Channel::Stable,
        Platform::MacOs,
        Architecture::Aarch64,
        timestamp("2027-02-05T00:05:01Z"),
    );
    assert_eq!(
        parse_fixture().verify(&trust_store(), &late).unwrap_err(),
        UpdaterError::ManifestExpired
    );
}

#[test]
fn accepts_clock_skew_boundary() {
    let within = VerificationContext::new(
        Channel::Stable,
        Platform::MacOs,
        Architecture::Aarch64,
        timestamp("2027-02-05T00:05:00Z"),
    );
    parse_fixture().verify(&trust_store(), &within).unwrap();
}

#[test]
fn rejects_unknown_revoked_and_wrong_epoch_keys() {
    let mut value = manifest_value(SIGNATURE_HEX);
    value["signature"]["key_id"] = json!("release-2027-01");
    let manifest = Manifest::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(
        manifest.verify(&trust_store(), &context()).unwrap_err(),
        UpdaterError::UnknownSigningKey {
            key_id: "release-2027-01".to_owned()
        }
    );

    let revoked = TrustStore::new(vec![
        TrustedKey::new(
            KeyId::new(KEY_ID).unwrap(),
            Ed25519PublicKey::parse_hex(PUBLIC_KEY_HEX).unwrap(),
            1,
            vec![Channel::Stable],
            timestamp("2026-01-01T00:00:00Z"),
            timestamp("2028-01-01T00:00:00Z"),
        )
        .unwrap()
        .with_revoked(true),
    ])
    .unwrap();
    assert_eq!(
        parse_fixture().verify(&revoked, &context()).unwrap_err(),
        UpdaterError::SigningKeyRevoked {
            key_id: KEY_ID.to_owned()
        }
    );

    let other_epoch = TrustStore::new(vec![
        TrustedKey::new(
            KeyId::new(KEY_ID).unwrap(),
            Ed25519PublicKey::parse_hex(PUBLIC_KEY_HEX).unwrap(),
            2,
            vec![Channel::Stable],
            timestamp("2026-01-01T00:00:00Z"),
            timestamp("2028-01-01T00:00:00Z"),
        )
        .unwrap(),
    ])
    .unwrap();
    assert_eq!(
        parse_fixture()
            .verify(&other_epoch, &context())
            .unwrap_err(),
        UpdaterError::SigningKeyEpochMismatch {
            expected: 2,
            found: 1,
        }
    );
}

#[test]
fn rejects_key_not_allowed_for_channel() {
    let beta_only = TrustStore::new(vec![
        TrustedKey::new(
            KeyId::new(KEY_ID).unwrap(),
            Ed25519PublicKey::parse_hex(PUBLIC_KEY_HEX).unwrap(),
            1,
            vec![Channel::Beta],
            timestamp("2026-01-01T00:00:00Z"),
            timestamp("2028-01-01T00:00:00Z"),
        )
        .unwrap(),
    ])
    .unwrap();
    assert_eq!(
        parse_fixture().verify(&beta_only, &context()).unwrap_err(),
        UpdaterError::SigningKeyChannelNotAllowed {
            key_id: KEY_ID.to_owned(),
            channel: Channel::Stable,
        }
    );
}

#[test]
fn rejects_signing_key_outside_its_validity_window() {
    let not_yet = TrustStore::new(vec![
        TrustedKey::new(
            KeyId::new(KEY_ID).unwrap(),
            Ed25519PublicKey::parse_hex(PUBLIC_KEY_HEX).unwrap(),
            1,
            vec![Channel::Stable],
            timestamp("2027-06-01T00:00:00Z"),
            timestamp("2027-07-01T00:00:00Z"),
        )
        .unwrap(),
    ])
    .unwrap();
    assert_eq!(
        parse_fixture().verify(&not_yet, &context()).unwrap_err(),
        UpdaterError::SigningKeyNotYetValid
    );

    let expired = TrustStore::new(vec![
        TrustedKey::new(
            KeyId::new(KEY_ID).unwrap(),
            Ed25519PublicKey::parse_hex(PUBLIC_KEY_HEX).unwrap(),
            1,
            vec![Channel::Stable],
            timestamp("2024-01-01T00:00:00Z"),
            timestamp("2025-01-01T00:00:00Z"),
        )
        .unwrap(),
    ])
    .unwrap();
    assert_eq!(
        parse_fixture().verify(&expired, &context()).unwrap_err(),
        UpdaterError::SigningKeyExpired
    );
}

#[test]
fn rejects_metadata_sequence_regression() {
    let policy = context().with_minimum_sequence(2);
    assert_eq!(
        parse_fixture().verify(&trust_store(), &policy).unwrap_err(),
        UpdaterError::MetadataSequenceRegression {
            minimum: 2,
            found: 1,
        }
    );
    let policy = context().with_minimum_sequence(1);
    parse_fixture().verify(&trust_store(), &policy).unwrap();
}

#[test]
fn rejects_duplicate_and_unknown_fields() {
    let duplicate = br#"{"schema_version":1,"schema_version":1,"product_id":"x"}"#;
    assert!(matches!(
        Manifest::parse(duplicate).unwrap_err(),
        UpdaterError::UnknownOrDuplicateField { .. }
    ));

    let unknown_top = br#"{"schema_version":1,"product_id":"x","extra":true}"#;
    assert!(matches!(
        Manifest::parse(unknown_top).unwrap_err(),
        UpdaterError::UnknownOrDuplicateField { .. }
    ));

    let mut value = manifest_value(SIGNATURE_HEX);
    value["target"]["extra"] = json!(true);
    assert!(matches!(
        Manifest::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err(),
        UpdaterError::UnknownOrDuplicateField { .. }
    ));
}

#[test]
fn rejects_trailing_data_and_oversized_input() {
    let mut bytes = manifest_bytes();
    bytes.extend_from_slice(b" trailing");
    assert_eq!(
        Manifest::parse(&bytes).unwrap_err(),
        UpdaterError::TrailingData
    );

    let oversized = vec![b' '; MAX_MANIFEST_BYTES + 1];
    assert!(matches!(
        Manifest::parse(&oversized).unwrap_err(),
        UpdaterError::ManifestTooLarge { .. }
    ));

    assert_eq!(
        Manifest::parse(&[0xff, 0xfe]).unwrap_err(),
        UpdaterError::InvalidUtf8
    );
}

#[test]
fn rejects_invalid_schema_and_field_values() {
    let mut value = manifest_value(SIGNATURE_HEX);
    value["schema_version"] = json!(2);
    assert!(matches!(
        Manifest::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err(),
        UpdaterError::UnsupportedSchemaVersion { found: 2, .. }
    ));

    for (field, replacement) in [
        ("product_id", json!("EQwec Hub")),
        ("product_version", json!("not-semver")),
        ("channel", json!("nightly")),
        ("metadata_sequence", json!(0)),
        ("issued_at", json!("2026-11-07T00:00:00+00:00")),
        ("expires_at", json!("2026-11-07T00:00:00Z")),
        ("release_notes", json!("")),
    ] {
        let mut value = manifest_value(SIGNATURE_HEX);
        value[field] = replacement;
        assert!(
            Manifest::parse(&serde_json::to_vec(&value).unwrap()).is_err(),
            "{field} should be rejected"
        );
    }
}

#[test]
fn rejects_invalid_package_and_target() {
    let mut value = manifest_value(SIGNATURE_HEX);
    value["package"]["source"] = json!("http://packages.eqwec.example/a");
    assert!(Manifest::parse(&serde_json::to_vec(&value).unwrap()).is_err());

    let mut value = manifest_value(SIGNATURE_HEX);
    value["package"]["size"] = json!(0);
    assert!(matches!(
        Manifest::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err(),
        UpdaterError::InvalidPackageSize { .. }
    ));

    let mut value = manifest_value(SIGNATURE_HEX);
    value["package"]["hashes"] = json!([]);
    assert!(Manifest::parse(&serde_json::to_vec(&value).unwrap()).is_err());

    let mut value = manifest_value(SIGNATURE_HEX);
    value["package"]["hashes"][0]["value"] = json!(PACKAGE_SHA256.to_uppercase());
    assert!(Manifest::parse(&serde_json::to_vec(&value).unwrap()).is_err());

    let mut value = manifest_value(SIGNATURE_HEX);
    value["target"]["os"] = json!("solaris");
    assert!(Manifest::parse(&serde_json::to_vec(&value).unwrap()).is_err());

    let mut value = manifest_value(SIGNATURE_HEX);
    value["target"]["minimum_platform"] = json!("");
    assert!(Manifest::parse(&serde_json::to_vec(&value).unwrap()).is_err());
}

#[test]
fn rejects_invalid_dependencies() {
    let dependency = json!({
        "product_id": "eqwec-photo",
        "version_constraint": ">=0.1.0, <1.0.0",
        "package_hash": PACKAGE_SHA256
    });

    let mut self_reference = dependency.clone();
    self_reference["product_id"] = json!("eqwec-hub");
    let mut value = manifest_value(SIGNATURE_HEX);
    value["dependencies"] = json!([self_reference]);
    assert!(matches!(
        Manifest::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err(),
        UpdaterError::InvalidDependency { .. }
    ));

    let mut unsorted = manifest_value(SIGNATURE_HEX);
    let mut alpha = dependency.clone();
    alpha["product_id"] = json!("eqwec-audio");
    let mut beta = dependency.clone();
    beta["product_id"] = json!("eqwec-browser");
    unsorted["dependencies"] = json!([beta, alpha]);
    assert!(matches!(
        Manifest::parse(&serde_json::to_vec(&unsorted).unwrap()).unwrap_err(),
        UpdaterError::InvalidDependency { .. }
    ));

    let mut duplicate = manifest_value(SIGNATURE_HEX);
    duplicate["dependencies"] = json!([dependency.clone(), dependency.clone()]);
    assert!(matches!(
        Manifest::parse(&serde_json::to_vec(&duplicate).unwrap()).unwrap_err(),
        UpdaterError::InvalidDependency { .. }
    ));

    let mut bad_hash = manifest_value(SIGNATURE_HEX);
    let mut dependency = dependency;
    dependency["package_hash"] = json!("not-a-hash");
    bad_hash["dependencies"] = json!([dependency]);
    assert!(Manifest::parse(&serde_json::to_vec(&bad_hash).unwrap()).is_err());
}

#[test]
fn rejects_package_truncation_size_and_hash_mismatch() {
    let verified = verify_fixture();

    assert_eq!(
        verified
            .verify_package_bytes(&PACKAGE_BYTES[..10])
            .unwrap_err(),
        UpdaterError::PackageTruncated {
            expected: PACKAGE_BYTES.len() as u64,
            actual: 10,
        }
    );

    let mut oversized = PACKAGE_BYTES.to_vec();
    oversized.push(0);
    assert_eq!(
        verified.verify_package_bytes(&oversized).unwrap_err(),
        UpdaterError::PackageSizeMismatch {
            expected: PACKAGE_BYTES.len() as u64,
            actual: PACKAGE_BYTES.len() as u64 + 1,
        }
    );

    let mut tampered = PACKAGE_BYTES.to_vec();
    tampered[0] ^= 0x01;
    assert!(matches!(
        verified.verify_package_bytes(&tampered).unwrap_err(),
        UpdaterError::PackageHashMismatch { .. }
    ));

    assert_eq!(
        verified
            .verify_package_reader(Cursor::new(&PACKAGE_BYTES[..10]))
            .unwrap_err(),
        UpdaterError::PackageTruncated {
            expected: PACKAGE_BYTES.len() as u64,
            actual: 10,
        }
    );
}

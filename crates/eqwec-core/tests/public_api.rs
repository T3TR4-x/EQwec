//! Public API integration tests for `eqwec-core`.

use eqwec_core::{Architecture, Channel, CoreError, Platform, ProductId, ScopeId};

#[test]
fn public_identifiers_are_opaque_and_round_trip() {
    let product = ProductId::new("eqwec-browser").expect("valid product id");
    let scope = ScopeId::new("work-profile").expect("valid scope id");

    assert_eq!(product.to_string(), "eqwec-browser");
    assert_eq!(scope.to_string(), "work-profile");
    assert_eq!(
        "eqwec-browser".parse::<ProductId>().expect("parse"),
        product
    );
    assert_eq!("work-profile".parse::<ScopeId>().expect("parse"), scope);
}

#[test]
fn public_enums_use_canonical_names() {
    assert_eq!(Channel::Stable.as_str(), "stable");
    assert_eq!(Channel::Dev.as_str(), "dev");
    assert_eq!(Platform::MacOs.as_str(), "macos");
    assert_eq!(Architecture::Aarch64.as_str(), "aarch64");
}

#[test]
fn errors_are_actionable_and_do_not_echo_arbitrary_payloads() {
    let error = ProductId::new("Bad Id").expect_err("must fail");

    assert!(matches!(error, CoreError::InvalidProductId { .. }));
    assert!(error.to_string().contains("invalid product identifier"));
    assert!(!error.to_string().contains("Bad Id"));
}

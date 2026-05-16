//! Integration tests against the `tests/fixtures/x402/` corpus.
//!
//! Each JSON fixture represents the *decoded* PAYMENT-REQUIRED body.
//! The parser receives it Base64-encoded; we encode here, then assert
//! the round-trip back to a `PaymentRequired` value matches.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use pennykite_providers::x402::parse_payment_required;
use std::fs;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/x402")
}

fn encode_fixture(name: &str) -> String {
    let path = fixture_dir().join(name);
    let raw = fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    BASE64.encode(raw)
}

#[test]
fn parses_base_usdc() {
    let header = encode_fixture("base-usdc.json");
    let pr = parse_payment_required(&header).expect("parse");
    assert_eq!(pr.version, "1.0");
    assert_eq!(pr.requirements.len(), 1);
    assert_eq!(pr.requirements[0].network, "eip155:8453");
    assert_eq!(pr.requirements[0].asset, "USDC");
    assert_eq!(pr.requirements[0].amount, "50000");
    assert_eq!(pr.requirements[0].decimals, 6);
}

#[test]
fn parses_base_sepolia_usdc() {
    let header = encode_fixture("base-sepolia-usdc.json");
    let pr = parse_payment_required(&header).expect("parse");
    assert_eq!(pr.requirements[0].network, "eip155:84532");
    assert!(pr.requirements[0].facilitator.is_none());
    assert!(pr.expires_at.is_none());
}

#[test]
fn parses_polygon_usdt() {
    let header = encode_fixture("polygon-usdt.json");
    let pr = parse_payment_required(&header).expect("parse");
    assert_eq!(pr.requirements[0].asset, "USDT");
    assert_eq!(pr.requirements[0].amount, "250000");
}

#[test]
fn parses_arbitrum_pyusd() {
    let header = encode_fixture("arbitrum-pyusd.json");
    let pr = parse_payment_required(&header).expect("parse");
    assert_eq!(pr.requirements[0].network, "eip155:42161");
    assert_eq!(pr.requirements[0].asset, "PYUSD");
}

#[test]
fn parses_kite_testnet_usdc() {
    let header = encode_fixture("kite-testnet-usdc.json");
    let pr = parse_payment_required(&header).expect("parse");
    assert_eq!(pr.requirements[0].network, "eip155:2368");
}

#[test]
fn parses_multi_options() {
    let header = encode_fixture("multi-options.json");
    let pr = parse_payment_required(&header).expect("parse");
    assert_eq!(pr.requirements.len(), 2);
    assert_eq!(pr.requirements[0].asset, "USDC");
    assert_eq!(pr.requirements[1].asset, "PYUSD");
}

#[test]
fn rejects_invalid_base64() {
    let result = parse_payment_required("not!valid!base64!");
    assert!(result.is_err());
}

#[test]
fn rejects_invalid_json() {
    let bad = BASE64.encode(b"this is not json");
    let result = parse_payment_required(&bad);
    assert!(result.is_err());
}

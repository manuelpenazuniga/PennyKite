//! x402 header parsing and helpers.
//!
//! Decodes Base64-encoded `PAYMENT-REQUIRED` headers into structured
//! `PaymentRequired` values and provides utility functions for working
//! with x402 payment requirements.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use pennykite_types::{PaymentRequired, PaymentRequirement};
use thiserror::Error;
use tracing::debug;

#[derive(Error, Debug)]
pub enum X402Error {
    #[error("base64 decode failed: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("json parse failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("missing PAYMENT-REQUIRED header")]
    MissingHeader,
}

/// Parse a Base64-encoded `PAYMENT-REQUIRED` header value into a
/// `PaymentRequired` struct.
pub fn parse_payment_required(header_value: &str) -> Result<PaymentRequired, X402Error> {
    let bytes = BASE64.decode(header_value)?;
    let pr: PaymentRequired = serde_json::from_slice(&bytes)?;
    debug!(version = %pr.version, num_requirements = pr.requirements.len(), "parsed PAYMENT-REQUIRED");
    Ok(pr)
}

/// Select the first requirement whose network and asset are both allowed.
pub fn select_requirement<'a>(
    requirements: &'a [PaymentRequirement],
    allowed_networks: &[String],
    allowed_assets: &[String],
) -> Option<&'a PaymentRequirement> {
    requirements.iter().find(|req| {
        allowed_networks.contains(&req.network) && allowed_assets.contains(&req.asset)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_payment_required() {
        let pr = PaymentRequired {
            version: "1.0".into(),
            requirements: vec![PaymentRequirement {
                network: "eip155:8453".into(),
                asset: "USDC".into(),
                amount: "50000".into(),
                decimals: 6,
                pay_to: "0x1234".into(),
                facilitator: None,
            }],
            expires_at: None,
        };
        let json = serde_json::to_string(&pr).unwrap();
        let encoded = BASE64.encode(json.as_bytes());
        let parsed = parse_payment_required(&encoded).unwrap();
        assert_eq!(pr, parsed);
    }
}

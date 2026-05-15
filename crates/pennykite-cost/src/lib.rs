//! Pre-execution cost estimation for x402 payment requirements.
//!
//! Converts raw x402 header data into USD estimates before the proxy
//! commits to signing a payment.

use pennykite_types::PaymentRequirement;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CostError {
    #[error("invalid amount string: {0}")]
    InvalidAmount(String),
    #[error("unsupported decimals: {0}")]
    UnsupportedDecimals(u8),
}

/// Convert a smallest-unit amount string to a USD float.
///
/// # Examples
///
/// ```
/// use pennykite_cost::amount_to_usd;
/// assert_eq!(amount_to_usd("50000", 6).unwrap(), 0.05);
/// assert_eq!(amount_to_usd("1000000", 6).unwrap(), 1.0);
/// ```
pub fn amount_to_usd(amount: &str, decimals: u8) -> Result<f64, CostError> {
    let raw: u128 = amount
        .parse()
        .map_err(|_| CostError::InvalidAmount(amount.into()))?;
    let divisor = 10_u128
        .checked_pow(decimals as u32)
        .ok_or(CostError::UnsupportedDecimals(decimals))?;
    Ok(raw as f64 / divisor as f64)
}

/// Estimate the USD cost of a single payment requirement.
pub fn estimate_cost(req: &PaymentRequirement) -> Result<f64, CostError> {
    amount_to_usd(&req.amount, req.decimals)
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
    fn test_amount_to_usd() {
        assert_eq!(amount_to_usd("50000", 6).unwrap(), 0.05);
        assert_eq!(amount_to_usd("1000000", 6).unwrap(), 1.0);
        assert_eq!(amount_to_usd("0", 6).unwrap(), 0.0);
    }

    #[test]
    fn test_invalid_amount() {
        assert!(amount_to_usd("abc", 6).is_err());
    }

    #[test]
    fn test_select_requirement_match() {
        let reqs = vec![
            PaymentRequirement {
                network: "eip155:8453".into(),
                asset: "USDC".into(),
                amount: "50000".into(),
                decimals: 6,
                pay_to: "0x1".into(),
                facilitator: None,
            },
            PaymentRequirement {
                network: "eip155:137".into(),
                asset: "USDT".into(),
                amount: "100000".into(),
                decimals: 6,
                pay_to: "0x2".into(),
                facilitator: None,
            },
        ];
        let networks = vec!["eip155:8453".into()];
        let assets = vec!["USDC".into()];
        let selected = select_requirement(&reqs, &networks, &assets);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().network, "eip155:8453");
    }

    #[test]
    fn test_select_requirement_no_match() {
        let reqs = vec![PaymentRequirement {
            network: "eip155:137".into(),
            asset: "USDT".into(),
            amount: "100000".into(),
            decimals: 6,
            pay_to: "0x2".into(),
            facilitator: None,
        }];
        let networks = vec!["eip155:8453".into()];
        let assets = vec!["USDC".into()];
        assert!(select_requirement(&reqs, &networks, &assets).is_none());
    }
}

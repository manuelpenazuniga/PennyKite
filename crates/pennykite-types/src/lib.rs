//! Shared types for the PennyKite pre-execution budget guardian.
//!
//! This crate defines the core data structures used across all PennyKite
//! components: x402 payment headers, policy configuration, budget decisions,
//! and attestation records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single payment requirement extracted from an x402 `PAYMENT-REQUIRED` header.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaymentRequirement {
    /// CAIP-2 chain identifier (e.g. "eip155:8453" for Base).
    pub network: String,
    /// Asset symbol (e.g. "USDC", "PYUSD").
    pub asset: String,
    /// Amount in smallest units as a decimal string (e.g. "50000" for 0.05 USDC).
    pub amount: String,
    /// Number of decimals for the asset (e.g. 6 for USDC).
    pub decimals: u8,
    /// Pay-to address (the merchant/facilitator).
    pub pay_to: String,
    /// Optional facilitator URL for settlement.
    pub facilitator: Option<String>,
}

/// The decoded content of an x402 `PAYMENT-REQUIRED` header.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaymentRequired {
    /// x402 protocol version.
    pub version: String,
    /// List of accepted payment options.
    pub requirements: Vec<PaymentRequirement>,
    /// ISO 8601 timestamp when the quote expires.
    pub expires_at: Option<DateTime<Utc>>,
}

/// Loop-detection configuration embedded in a policy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoopDetectionConfig {
    /// Whether loop detection is active.
    pub enabled: bool,
    /// Number of recent requests to consider.
    pub window_size: usize,
    /// Similarity threshold above which two requests are considered "the same".
    pub similarity_threshold: f64,
    /// Maximum number of consecutive similar requests before blocking.
    pub max_consecutive_similar: usize,
}

impl Default for LoopDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            window_size: 10,
            similarity_threshold: 0.85,
            max_consecutive_similar: 3,
        }
    }
}

/// Kite Passport configuration embedded in a policy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KitePassportConfig {
    /// The agent's on-chain address.
    pub agent_address: String,
    /// The session key delegated by the agent.
    pub session_key: String,
    /// How often to anchor decisions on-chain.
    pub attestation_frequency: AttestationFrequency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AttestationFrequency {
    EveryEvent,
    EveryNth(usize),
    FinalOnly,
}

/// A user-defined policy loaded from YAML at proxy boot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Policy {
    /// Policy format version.
    pub version: String,
    /// Human-readable name for this policy.
    pub name: String,
    /// Session-level budget constraints.
    pub session: SessionPolicy,
    /// Loop-detection settings.
    pub loop_detection: LoopDetectionConfig,
    /// Allowed networks and assets.
    pub networks: NetworkPolicy,
    /// Kite Passport integration settings.
    pub kite_passport: KitePassportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionPolicy {
    /// Maximum USD the session is allowed to spend.
    pub budget_usd: f64,
    /// Maximum USD per individual request.
    pub per_request_cap_usd: f64,
    /// Maximum session duration in minutes.
    pub max_duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkPolicy {
    /// CAIP-2 identifiers of allowed networks.
    pub allowed: Vec<String>,
    /// Allowed asset symbols.
    pub assets: Vec<String>,
}

/// The outcome of a budget decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Approve,
    Deny,
    DenyLoop,
    DenyBudget,
    DenyNetwork,
    DenyAsset,
    SessionInvalid,
}

/// A single decision record produced by the proxy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// Unique decision id.
    pub id: Uuid,
    /// Session this decision belongs to.
    pub session_id: String,
    /// ISO 8601 timestamp.
    pub timestamp: DateTime<Utc>,
    /// The upstream HTTP method + path.
    pub request_key: String,
    /// Estimated USD cost.
    pub estimated_cost_usd: f64,
    /// The verdict.
    pub verdict: Verdict,
    /// Human-readable reason.
    pub reason: String,
    /// SHA3-256 hash of the canonical JSON representation.
    pub decision_hash: String,
    /// Kite attestation transaction hash (if anchored).
    pub kite_attestation_tx: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_payment_required() {
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
        let back: PaymentRequired = serde_json::from_str(&json).unwrap();
        assert_eq!(pr, back);
    }

    #[test]
    fn round_trip_verdict() {
        let v = Verdict::DenyLoop;
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "\"deny_loop\"");
        let back: Verdict = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn default_loop_config() {
        let cfg = LoopDetectionConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.window_size, 10);
        assert_eq!(cfg.similarity_threshold, 0.85);
        assert_eq!(cfg.max_consecutive_similar, 3);
    }
}

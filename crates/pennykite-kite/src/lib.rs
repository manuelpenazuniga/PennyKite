//! Kite chain client: attestation submission and Passport session validation.
//!
//! Wraps alloy for interacting with the PennyKiteAttestor contract and
//! Kite Passport on-chain primitives.

use pennykite_types::Decision;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum KiteError {
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("attestation failed: {0}")]
    Attestation(String),
    #[error("session validation failed: {0}")]
    SessionValidation(String),
}

/// Submit a decision hash to the PennyKiteAttestor contract on Kite.
///
/// Returns the transaction hash. This is a best-effort operation — failures
/// are logged but do not block the proxy response.
pub async fn attest(
    _decision: &Decision,
    _rpc_url: &str,
    _private_key: &str,
    _attestor_address: &str,
) -> Result<String, KiteError> {
    // Placeholder — will be implemented with alloy in PK-D1-11.
    info!(
        decision_id = %_decision.id,
        hash = %_decision.decision_hash,
        "attestation placeholder (not yet wired to chain)"
    );
    Err(KiteError::Rpc("not yet implemented".into()))
}

/// Verify a Kite Passport session key against on-chain state.
///
/// Returns the remaining quota in USD. Results are cached for 5 seconds
/// by the caller (proxy handler).
pub async fn verify_session(
    _agent_address: &str,
    _session_key: &str,
    _rpc_url: &str,
) -> Result<f64, KiteError> {
    // Placeholder — will be implemented with alloy in PK-D1-11.
    info!("session verification placeholder (not yet wired to chain)");
    Ok(f64::MAX) // Allow all until wired
}

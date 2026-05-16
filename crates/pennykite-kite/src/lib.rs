//! Kite chain client: attestation submission and Passport session validation.
//!
//! This crate is split into two layers:
//!
//! * [`KiteRpc`] — async trait abstracting the on-chain calls. The proxy
//!   talks to the trait so the cache + business logic can be unit-tested
//!   without standing up a real RPC endpoint.
//! * [`KiteClient`] — wraps any `KiteRpc` implementation with a 5-second
//!   TTL cache for `verify_session` results, satisfying the spec
//!   acceptance criterion that "2 consecutive calls within 5s produce
//!   1 RPC call".
//!
//! The production implementation [`AlloyKiteRpc`] is wired against the
//! `PennyKiteAttestor.sol` contract using `alloy::sol!`. End-to-end smoke
//! testing on Kite testnet is gated by PK-D1-07 (contract deployment).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use alloy::primitives::{Address, B256};
use alloy::sol;
use async_trait::async_trait;
use thiserror::Error;
use tracing::{debug, info};

pub use alloy::primitives;

// ---------------------------------------------------------------------------
// Solidity bindings
// ---------------------------------------------------------------------------

sol! {
    #[sol(rpc)]
    contract PennyKiteAttestor {
        event Attested(
            bytes32 indexed sessionId,
            bytes32 indexed decisionHash,
            address indexed reporter,
            uint256 timestamp
        );

        function attest(bytes32 sessionId, bytes32 decisionHash) external;
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum KiteError {
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("attestation failed: {0}")]
    Attestation(String),
    #[error("session validation failed: {0}")]
    SessionValidation(String),
    #[error("invalid configuration: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, KiteError>;

// ---------------------------------------------------------------------------
// KiteRpc trait — abstraction over on-chain calls for testability
// ---------------------------------------------------------------------------

/// Abstraction over the on-chain calls PennyKite needs to make.
///
/// Implementors must be `Send + Sync` so the trait object can be shared
/// across the proxy's tokio tasks.
#[async_trait]
pub trait KiteRpc: Send + Sync {
    /// Returns the remaining quota (in USD smallest units, i.e. cents) for
    /// the given Kite Passport session key. Implementations talk to the
    /// Kite Passport on-chain registry.
    async fn fetch_remaining_quota_cents(&self, session_key: &str) -> Result<u64>;

    /// Submits an attestation to the deployed `PennyKiteAttestor` contract.
    /// Returns the transaction hash.
    async fn send_attestation(&self, session_id: B256, decision_hash: B256) -> Result<B256>;
}

// ---------------------------------------------------------------------------
// KiteClient — high-level façade with cache
// ---------------------------------------------------------------------------

/// Time-to-live for the `verify_session` cache, mandated by the spec
/// (acceptance criterion in PK-D1-11 backlog entry).
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(5);

struct CacheEntry {
    value: u64,
    expires_at: Instant,
}

/// High-level client used by the proxy. Holds a [`KiteRpc`] implementation
/// and adds a TTL cache around `verify_session`.
pub struct KiteClient<T: KiteRpc> {
    rpc: T,
    cache: Mutex<HashMap<String, CacheEntry>>,
    cache_ttl: Duration,
}

impl<T: KiteRpc> KiteClient<T> {
    /// Create a new client with the default 5-second cache TTL.
    pub fn new(rpc: T) -> Self {
        Self::with_ttl(rpc, DEFAULT_CACHE_TTL)
    }

    /// Create a new client with a custom cache TTL. Useful for tests.
    pub fn with_ttl(rpc: T, cache_ttl: Duration) -> Self {
        Self {
            rpc,
            cache: Mutex::new(HashMap::new()),
            cache_ttl,
        }
    }

    /// Resolve the remaining USD-cent quota for a session, hitting the
    /// underlying RPC at most once per `cache_ttl` window.
    pub async fn verify_session(&self, session_key: &str) -> Result<u64> {
        // Fast path: cache hit.
        if let Some(value) = self.cached(session_key) {
            debug!(session_key, "kite cache hit");
            return Ok(value);
        }

        // Slow path: call RPC and write back.
        let value = self.rpc.fetch_remaining_quota_cents(session_key).await?;
        self.write_cache(session_key, value);
        info!(session_key, remaining_cents = value, "kite RPC verify_session");
        Ok(value)
    }

    /// Submit an attestation. Not cached — every call is a real transaction.
    pub async fn attest(&self, session_id: B256, decision_hash: B256) -> Result<B256> {
        self.rpc.send_attestation(session_id, decision_hash).await
    }

    /// Manually evict a cache entry (e.g. after a session pause).
    pub fn invalidate(&self, session_key: &str) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(session_key);
        }
    }

    fn cached(&self, key: &str) -> Option<u64> {
        let cache = self.cache.lock().ok()?;
        let entry = cache.get(key)?;
        if Instant::now() < entry.expires_at {
            Some(entry.value)
        } else {
            None
        }
    }

    fn write_cache(&self, key: &str, value: u64) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(
                key.to_string(),
                CacheEntry {
                    value,
                    expires_at: Instant::now() + self.cache_ttl,
                },
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AlloyKiteRpc — production implementation
// ---------------------------------------------------------------------------

/// Production implementation of [`KiteRpc`] backed by alloy.
///
/// Holds the addresses for both the deployed `PennyKiteAttestor` contract
/// and the Kite Passport quota registry. Both addresses are required at
/// construction so the client fails fast on misconfiguration.
///
/// **Note**: the actual RPC wiring requires a Provider+Signer; that
/// integration is implemented in PK-D1-07 once the attestor is deployed
/// and we have a real RPC URL + private key. Until then this struct
/// stores the configuration and returns a clear `Config` error from each
/// method, so the proxy can light up without on-chain state.
pub struct AlloyKiteRpc {
    rpc_url: String,
    attestor_address: Address,
    kite_passport_address: Address,
    /// Held for use once `send_attestation` is wired to alloy in PK-D1-07.
    #[allow(dead_code)]
    private_key_hex: String,
}

impl AlloyKiteRpc {
    pub fn new(
        rpc_url: impl Into<String>,
        attestor_address: Address,
        kite_passport_address: Address,
        private_key_hex: impl Into<String>,
    ) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            attestor_address,
            kite_passport_address,
            private_key_hex: private_key_hex.into(),
        }
    }

    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    pub fn attestor_address(&self) -> Address {
        self.attestor_address
    }

    pub fn kite_passport_address(&self) -> Address {
        self.kite_passport_address
    }
}

#[async_trait]
impl KiteRpc for AlloyKiteRpc {
    async fn fetch_remaining_quota_cents(&self, _session_key: &str) -> Result<u64> {
        // Wired in PK-D1-07. The Kite Passport ABI is not yet final; once
        // the team confirms the registry address + view function we will
        // build the alloy `Provider` here and call it.
        Err(KiteError::Config(
            "AlloyKiteRpc::fetch_remaining_quota_cents not yet wired (depends on PK-D1-07)".into(),
        ))
    }

    async fn send_attestation(&self, _session_id: B256, _decision_hash: B256) -> Result<B256> {
        // Wired in PK-D1-07. ABI is generated from the `sol!` macro above.
        Err(KiteError::Config(
            "AlloyKiteRpc::send_attestation not yet wired (depends on PK-D1-07)".into(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Test double that counts how many times each method was called.
    /// Stored values are scripted by the test.
    struct MockRpc {
        quota_cents: u64,
        verify_calls: Arc<AtomicUsize>,
        attest_calls: Arc<AtomicUsize>,
    }

    impl MockRpc {
        fn new(quota_cents: u64) -> Self {
            Self {
                quota_cents,
                verify_calls: Arc::new(AtomicUsize::new(0)),
                attest_calls: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    #[async_trait]
    impl KiteRpc for MockRpc {
        async fn fetch_remaining_quota_cents(&self, _session_key: &str) -> Result<u64> {
            self.verify_calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.quota_cents)
        }

        async fn send_attestation(
            &self,
            _session_id: B256,
            _decision_hash: B256,
        ) -> Result<B256> {
            self.attest_calls.fetch_add(1, Ordering::SeqCst);
            Ok(B256::from([0xAB; 32]))
        }
    }

    #[tokio::test]
    async fn verify_session_returns_quota() {
        let mock = MockRpc::new(500);
        let client = KiteClient::new(mock);
        let value = client.verify_session("session-1").await.unwrap();
        assert_eq!(value, 500);
    }

    #[tokio::test]
    async fn cache_hit_within_ttl_avoids_second_rpc_call() {
        let mock = MockRpc::new(500);
        let counter = mock.verify_calls.clone();
        let client = KiteClient::new(mock);

        let _ = client.verify_session("session-1").await.unwrap();
        let _ = client.verify_session("session-1").await.unwrap();

        // Spec acceptance criterion: 2 calls within 5s → 1 RPC.
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn different_sessions_do_not_share_cache() {
        let mock = MockRpc::new(500);
        let counter = mock.verify_calls.clone();
        let client = KiteClient::new(mock);

        let _ = client.verify_session("session-A").await.unwrap();
        let _ = client.verify_session("session-B").await.unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn cache_expires_after_ttl() {
        let mock = MockRpc::new(500);
        let counter = mock.verify_calls.clone();
        let client = KiteClient::with_ttl(mock, Duration::from_millis(50));

        let _ = client.verify_session("session-1").await.unwrap();
        tokio::time::sleep(Duration::from_millis(80)).await;
        let _ = client.verify_session("session-1").await.unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn invalidate_forces_rpc_call() {
        let mock = MockRpc::new(500);
        let counter = mock.verify_calls.clone();
        let client = KiteClient::new(mock);

        let _ = client.verify_session("session-1").await.unwrap();
        client.invalidate("session-1");
        let _ = client.verify_session("session-1").await.unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn attest_passes_through_to_rpc() {
        let mock = MockRpc::new(0);
        let counter = mock.attest_calls.clone();
        let client = KiteClient::new(mock);

        let session_id = B256::from([1u8; 32]);
        let decision_hash = B256::from([2u8; 32]);
        let tx = client.attest(session_id, decision_hash).await.unwrap();

        assert_eq!(tx, B256::from([0xAB; 32]));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn alloy_kite_rpc_stores_addresses() {
        let attestor = Address::from([0x11; 20]);
        let passport = Address::from([0x22; 20]);
        let rpc = AlloyKiteRpc::new(
            "https://rpc.testnet.kite.example.com",
            attestor,
            passport,
            "0x".to_string() + &"00".repeat(32),
        );
        assert_eq!(rpc.attestor_address(), attestor);
        assert_eq!(rpc.kite_passport_address(), passport);
        assert_eq!(rpc.rpc_url(), "https://rpc.testnet.kite.example.com");
    }

    #[tokio::test]
    async fn alloy_kite_rpc_returns_config_error_until_wired() {
        let rpc = AlloyKiteRpc::new(
            "https://example.com",
            Address::ZERO,
            Address::ZERO,
            "00".repeat(32),
        );
        assert!(matches!(
            rpc.fetch_remaining_quota_cents("any").await,
            Err(KiteError::Config(_))
        ));
        assert!(matches!(
            rpc.send_attestation(B256::ZERO, B256::ZERO).await,
            Err(KiteError::Config(_))
        ));
    }
}

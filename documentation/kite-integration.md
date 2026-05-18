# Kite Integration

PennyKite is purpose-built around Kite's unique combination of on-chain agent identity, session keys with quotas, and sub-100 ms settlement. This document describes the current integration state and what remains pending for the production deployment.

---

## Why Kite

Three Kite primitives are central to PennyKite's design:

### Agent Passport
Every AI agent operating through PennyKite has an on-chain identity anchored in Kite's Agent Passport. The Passport links the agent's actions to a verifiable on-chain address, making the audit trail cryptographically attributable — not just a database record on a server the operator controls.

### Session keys with quotas
Kite Passport supports delegated session keys: short-lived keys that can sign transactions on behalf of the agent's main key, constrained to specific quotas. PennyKite compiles the human-readable YAML policy (budget, per-request cap, allowed networks) into the equivalent session-key constraints at setup time.

### Sub-100 ms latency at near-zero cost
Kite's block time and gas costs make it practical to anchor *every* proxy decision on-chain — not just batch summaries. At ~$0.000001 per transaction, anchoring each approve or deny adds negligible cost relative to the $0.05+ API call it protects.

---

## The `pennykite-kite` crate

The `crates/pennykite-kite/` crate holds all Kite-specific logic. Its public surface:

```rust
pub trait KiteRpc {
    async fn fetch_remaining_quota_cents(&self, session_key: &str) -> Result<u64>;
    async fn send_attestation(&self, session_id: B256, decision_hash: B256) -> Result<B256>;
}

pub struct AlloyKiteRpc;
```

The crate uses [alloy](https://github.com/alloy-rs/alloy) for Ethereum-compatible JSON-RPC and transaction signing, targeting the Kite JSON-RPC endpoint (`KITE_RPC_URL`).

---

## `PennyKiteAttestor` contract

The Solidity contract lives at `contracts/src/PennyKiteAttestor.sol`. Its role is to store an append-only log of decision hashes emitted by the proxy:

```solidity
event Attested(
    bytes32 indexed sessionId,
    bytes32 indexed decisionHash,
    uint256 timestamp,
    address attester,
    uint256 index
);

function attest(bytes32 sessionId, bytes32 decisionHash) external returns (uint256 index);
```

Every proxy decision — approve *or* deny — produces a `decision_hash = SHA3-256(canonical decision JSON)` on the Rust side. When Kite attestation config is present, that hash is submitted to Kite in a best-effort transaction and the returned tx hash is persisted on the `Decision` row.

**Current status:** The contract code is complete, covered by Foundry tests in CI, and deployed on Kite testnet at `0x3973Ce9a493EeB190A1Ae8ABbEb960533242d762`. Deploy tx: `0xb3ec20954ff43e68c910af6d60689eba621ca9d02bad512ea3c1cc1c304953f2`. Smoke `attest(bytes32,bytes32)` tx: `0xab95060fa504238bd3fc1f1c27364160c383d5e295050fc780016abd9b311e7f`.

---

## Attestation flow

```
proxy_inner (Rust)
    │
    ├─ compute decision_hash = SHA3-256(canonical decision JSON)
    ├─ ledger.record_decision(decision)
    └─ kite_rpc.attest(session_id, decision_hash)
           │
           ▼
    PennyKiteAttestor.attest(sessionId, decisionHash)
           │
           ▼
    emit Attested(sessionId, decisionHash, block.timestamp, attester, index)
```

**What is implemented today:**
- `decision_hash` is computed and stored in SQLite on every decision.
- `AlloyKiteRpc::send_attestation` signs and broadcasts `attest(bytes32,bytes32)` through the configured Kite RPC.
- The proxy updates `kite_attestation_tx` when the broadcast succeeds and logs failures without blocking the response.

**What is pending:**
- Kite Passport quota validation and on-chain session revocation remain pending the final Passport ABI/API.

---

## Kill-switch and session revocation

### Current behaviour (SQLite-only)
When the dashboard operator clicks **Pause** on a session:
1. `POST /api/sessions/{id}/pause` reaches the proxy.
2. The proxy calls `ledger.pause_session(session_id)`, setting `status = 'paused'` in SQLite.
3. Every subsequent `try_reserve` call for that session immediately returns `LedgerError::SessionPaused`, causing the proxy to return `402 {"reason":"session is paused"}`.

This is effective for the local demo: the agent is blocked within milliseconds. The pause response also returns `"kite_revocation":"mock_pending"` to signal that the on-chain step has not yet occurred.

### Target behaviour (on-chain revocation)
When the Kite Passport revocation API is available:
1. The proxy will call `kite_rpc.revoke_session(session_key)` as part of the pause flow.
2. The Kite Passport contract marks the session key as revoked on-chain.
3. Any x402 facilitator validating the session key against the Passport will see it as invalid — enforcement moves from the PennyKite proxy to the chain itself.

**What is pending:** The Kite Passport revocation API endpoint is not yet available on testnet. This remains `mock_pending` and is tracked as `PK-D2-13`.

---

## Policy compilation (planned)

The long-term design compiles the YAML policy's `session` block into Kite session-key constraints at agent onboarding time:

| YAML field | Kite session-key constraint |
|---|---|
| `budget_usd` | Maximum transferable value |
| `max_duration_minutes` | Key expiry timestamp |
| `networks.allowed` | Permitted chain IDs |
| `assets.allowed` | Permitted asset contracts |

Today, the YAML is enforced entirely by the proxy at runtime. The session-key constraint compilation is planned for post-hackathon hardening.

---

## Configuration

Set the following environment variables (see `.env.example`):

| Variable | Description |
|---|---|
| `KITE_RPC_URL` | Kite testnet JSON-RPC endpoint |
| `KITE_CHAIN_ID` | Kite testnet chain ID |
| `KITE_PRIVATE_KEY` | Agent private key for attestation signing |
| `SESSION_KEY` | Delegated session key address |
| `PENNYKITE_ATTESTOR_ADDRESS` | `0x3973Ce9a493EeB190A1Ae8ABbEb960533242d762` |

# PennyKite Architecture

PennyKite is an **out-of-process governance layer** that intercepts every x402 micropayment an AI agent attempts to make, evaluates it against a declarative policy, and either approves, denies, or blocks the request before any money moves.

This document describes the current implementation as of the v0.1.0 hackathon pre-alpha.

---

## Design philosophy

Three principles shape every architectural decision:

**1. Out-of-process, not in-SDK.**  
The proxy runs as a separate binary that the agent HTTP-routes through. A compromised or buggy agent cannot disable, bypass, or reconfigure what it does not control. The governance layer is not a library the agent imports.

**2. Pre-execution, not post-metering.**  
x402 is an atomic payment protocol: once the `PAYMENT-SIGNATURE` header is sent, the money has moved. PennyKite reads the upstream's `PAYMENT-REQUIRED` header *before* signing. If the request violates policy, it returns a `402 Payment Required` to the agent without ever signing a payment.

**3. Every decision is recorded.**  
Whether a request is approved or denied, a `Decision` row is written to the SQLite ledger with a SHA3-256 `decision_hash`. The hash is the anchor point for the future on-chain attestation.

---

## Workspace crates

The Rust workspace under `crates/` is decomposed into focused single-responsibility crates:

### `pennykite-types`
Shared data structures used across all crates. Key types:

- `PaymentRequirement` — a single payment option parsed from an x402 `PAYMENT-REQUIRED` header (network, asset, amount, pay-to address, facilitator URL).
- `PaymentRequired` — the full header containing a list of `PaymentRequirement` options.
- `Policy` — the validated, in-memory representation of a YAML policy file.
- `LoopDetectionConfig` — loop detector parameters (window size, similarity threshold, max consecutive).
- `Decision` — a recorded proxy decision: session ID, path, verdict, estimated cost, decision hash, timestamp.
- `Verdict` — the outcome enum: `Approve`, `Deny`, `DenyLoop`, `DenyBudget`, `DenyNetwork`.

### `pennykite-budget`
Budget arithmetic helpers. Converts decimal USDC amounts (stored as strings in x402) to `f64` for comparison against policy caps.

### `pennykite-detect`
Sliding-window loop detector. `LoopDetector` maintains a `VecDeque<RequestFingerprint>` bounded by `window_size`. In v0.1 it uses exact-field similarity over `(method, host, path, body_hash)`: identical fingerprints score `1.0`, different fields reduce the score, and `similarity_threshold` remains configurable for later fuzzy matching. If more than `max_consecutive_similar` recent fingerprints exceed the threshold, `observe()` returns `true` and the proxy short-circuits with a `deny_loop` verdict before forwarding to upstream.

### `pennykite-cost`
Pre-execution cost estimation. Parses the upstream `PAYMENT-REQUIRED` header and selects the policy-approved requirement (matching network, asset, and per-request cap). Returns the estimated cost in USD for budget reservation.

### `pennykite-ledger`
Atomic SQLite ledger. Uses rusqlite with explicit transactions and row-level locking to prevent double-spend across concurrent requests.

Key operations:
- `ensure_session(session_id, budget_usd)` — idempotent session bootstrap.
- `try_reserve(session_id, estimated_cost)` — atomically checks `spent_usd + estimated_cost ≤ budget_usd` and updates if true. Returns `false` on budget exhaustion, errors on paused session.
- `record_decision(decision)` — appends a proxy-computed decision row to the `decisions` table, including `decision_hash = SHA3-256(canonical decision JSON)`.
- `pause_session(session_id)` — sets `status = 'paused'` in the `sessions` table; subsequent `try_reserve` calls immediately return `LedgerError::SessionPaused`.

### `pennykite-providers`
x402 protocol adapters and EIP-3009 signing.

- `x402.rs` — parses `PAYMENT-REQUIRED` headers (JSON base64 encoded) into `PaymentRequired` structs. Selects the best requirement that matches the policy's allowed networks and assets.
- `eip3009.rs` — builds and signs `transferWithAuthorization` EIP-712 digests for EIP-3009 payment authorizations. Signs with the proxy's configured `PrivateKeySigner`.
- `policy.rs` — loads and validates YAML policy files into `Policy` structs.

### `pennykite-kite`
Kite chain client (current state: stub with real RPC wiring). Holds the `KiteRpc` struct pointing at the Kite JSON-RPC endpoint. The `PennyKiteAttestor` contract is deployed on Kite testnet; attestation writes and Kite Passport session revocation are still proxy-integration work tracked as `PK-D2-09`.

### `pennykite-proxy`
The axum HTTP server binary. Assembles all crates into a running proxy. See [Proxy request flow](#proxy-request-flow) below.

---

## Proxy request flow

```
Agent
  │
  │  HTTP request  (any method, any path)
  │  x-pennykite-session: <session-id>
  ▼
pennykite-proxy  (axum, :8787)
  │
  ├─ 1. LoopDetector::observe(fingerprint)
  │     If loop: → 402 {verdict:"deny_loop", reason:"loop detected"}
  │
  ├─ 2. Forward request to upstream (no payment yet)
  │     If upstream returns non-402: pass response back to agent
  │
  ├─ 3. Upstream returns 402 PAYMENT-REQUIRED
  │     Parse PAYMENT-REQUIRED header → PaymentRequired
  │     Select best requirement matching policy (network + asset)
  │     If no requirement matches: → 402 {verdict:"deny_network"}
  │
  ├─ 4. Per-request cap check
  │     If estimated_cost > per_request_cap_usd: → 402 {verdict:"deny_budget"}
  │
  ├─ 5. Ledger::try_reserve(session_id, estimated_cost)
  │     If budget exhausted: → 402 {verdict:"deny_budget"}
  │     If session paused: → 402 {verdict:"deny"}
  │
  ├─ 6. EIP-3009 sign: transferWithAuthorization digest
  │     Build PAYMENT-SIGNATURE header
  │
  ├─ 7. Replay request to upstream with PAYMENT-SIGNATURE
  │     Upstream processes payment and returns 200
  │
  └─ 8. Ledger::record_decision(approve, cost, decision_hash)
        → 200 to agent
```

All verdicts (approve and deny) are recorded before returning to the agent.

---

## SQLite schema

```sql
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,
    budget_usd  REAL NOT NULL,
    spent_usd   REAL NOT NULL DEFAULT 0.0,
    status      TEXT NOT NULL DEFAULT 'active',  -- 'active' | 'paused'
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE decisions (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL REFERENCES sessions(id),
    timestamp       TEXT NOT NULL,
    request_key     TEXT NOT NULL,
    estimated_cost  REAL NOT NULL,
    verdict         TEXT NOT NULL,
    reason          TEXT NOT NULL,
    decision_hash   TEXT NOT NULL,
    attestation_tx  TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
```

---

## Control plane (dashboard)

The Next.js 16 dashboard at `http://localhost:3000` provides three pages:

| Page | Route | Description |
|---|---|---|
| Live feed | `/` | Polls `GET /api/decisions` every 500 ms; renders approve/deny rows with colour coding |
| Session detail | `/sessions/[id]` | Per-session spend, decision timeline |
| Kill-switch | `/kill-switch` | Lists active sessions; **Pause** triggers `POST /api/sessions/{id}/pause` |

The dashboard communicates with the proxy via the Next.js API bridge (server-side routes under `app/api/`). Browser code never calls the proxy directly.

---

## Demo target API

`demo/target-api/` is an Express server that implements the x402 server side:

- Every `GET /proxy/predict/:matchId` request returns `402 PAYMENT-REQUIRED` on first hit.
- On replay with a valid `PAYMENT-SIGNATURE`, it returns a JSON prediction with a `PAYMENT-RESPONSE` confirmation header.
- Defaults: price `$0.05 USDC`, network `eip155:84532` (Base Sepolia), asset `USDC`.

---

## Policy

Policies are YAML files validated at proxy boot. Key fields:

```yaml
session:
  budget_usd: 5.00
  per_request_cap_usd: 0.50
  max_duration_minutes: 30

loop_detection:
  enabled: true
  window_size: 10
  similarity_threshold: 0.85
  max_consecutive_similar: 3

networks:
  allowed: ["eip155:84532"]
  assets: ["USDC"]

kite_passport:
  agent_address: "0x..."
  session_key: "0x..."
  attestation_frequency: every_event
```

Policy is immutable after proxy boot. Changing policy requires a proxy restart.

---

## What is not yet wired

| Feature | Status | Backlog |
|---|---|---|
| On-chain attestation writes | Contract deployed; proxy wiring still pending | PK-D2-09 |
| Real Kite Passport session revocation | Mock — pause is SQLite-only today | PK-D2-13 |
| Hosted proxy/dashboard | Local-only for v0.1 until deploy tasks land | PK-D4-02, PK-D4-03 |
| TypeScript SDK package | Planned API surface only | Future v0.2 |

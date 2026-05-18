<div align="center">

# PennyKite

### **Pre-execution budget guardian for the agentic economy.**

*An out-of-process governance layer that decides **before** your AI agent pays — for every x402 micropayment, on every API, settled on Kite chain.*

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Status: Pre-Alpha](https://img.shields.io/badge/status-pre--alpha-orange.svg)](#roadmap)
[![Hackathon: Kite AI 2026](https://img.shields.io/badge/hackathon-Kite%20AI%202026-7c3aed.svg)](https://www.encodeclub.com/programmes/kites-hackathon-ai-agentic-economy)
[![Track: Novel](https://img.shields.io/badge/track-Novel-ec4899.svg)](#hackathon-submission)
[![Built on: x402](https://img.shields.io/badge/built%20on-x402-000000.svg)](https://github.com/coinbase/x402)
[![Settles on: Kite](https://img.shields.io/badge/settles%20on-Kite-22c55e.svg)](https://docs.gokite.ai)

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Solidity](https://img.shields.io/badge/solidity-0.8.20-363636?logo=solidity)](https://soliditylang.org/)
[![Next.js](https://img.shields.io/badge/next.js-16-black?logo=next.js)](https://nextjs.org/)
[![Python](https://img.shields.io/badge/python-3.10%2B-blue?logo=python)](https://www.python.org/)

[**Live Demo**](#) · [**Watch the 3-min demo**](#) · [**Architecture**](#architecture) · [**Quickstart**](#quickstart) · [**Roadmap**](#roadmap)

</div>

---

## The problem

The agentic economy is real, and it is **already moving**. By April 2026, x402 reported **69k active agents, 165M transactions and ~$50M cumulative volume**, with an average ticket of **$0.30**. Agents are paying for APIs, data, compute and services — autonomously, sub-second, in stablecoins.

But every demo so far showcases the **happy path**: agent calls API, agent pays, everyone wins. Nobody is shipping the **defensive** layer for when things go wrong.

> **A bugged agent can drain a wallet in seconds.**
> Hallucinated retry loop. Misread tool schema. Prompt injection. The blast radius is no longer one LLM provider — it is *every API on the internet that speaks x402*.

PennyKite is the missing layer.

---

## What PennyKite does

PennyKite sits as an **out-of-process reverse proxy** between your agent and the rest of the world. Every paid call is intercepted, scored, and either approved, denied or anchored on-chain.

| | |
|---|---|
| **Pre-execution cost estimation** | Reads the upstream `PAYMENT-REQUIRED` header *before* signing. No surprise bills. |
| **Atomic budget enforcement** | SQLite row-locked reservation. Two concurrent calls cannot both eat the last cent. |
| **Loop detection** | Fingerprint-based detection of runaway behaviour over a sliding window. Cuts the bleeding at call 4, not call 200. |
| **Kite Passport session validation** | Verifies on-chain quotas and session-key authority on every request. Cached for 5s steady-state. |
| **On-chain attestations** | Every decision (approve *or* deny) is hashed and anchored to the `PennyKiteAttestor` contract on Kite. Cryptographic, append-only audit. |
| **Kill-switch** | One click on the dashboard revokes the agent's session key on Kite Passport. Protection is at the chain level, not at our proxy. |

The result: an agent you can actually trust with money.

---

## Why Kite

PennyKite is not a blockchain-agnostic middleware that happens to deploy on Kite. It is **purpose-built around four Kite primitives** that no other L1 combines:

| Kite primitive | How PennyKite uses it |
|---|---|
| **Agent Passport** (on-chain agent identity) | Every decision is signed against the agent's Passport. The audit trail is cryptographically verifiable. |
| **Session keys with quotas** | PennyKite is the off-chain enforcement layer that compiles user-friendly YAML policy into Kite session-key constraints. |
| **Sub-100ms latency, ~$0.000001 per tx** | Every decision can be anchored on-chain without dominating the cost of the operation it protects. |
| **Native x402 settlement** | When PennyKite blocks, it returns a semantically valid `HTTP 402` indicating *"PennyKite: budget exceeded"*. |

> *Kite Passport gives agents identity and quotas. PennyKite is the runtime layer that uses those primitives to make agents safe to give your money to.*

---

## How it differs

| Solution | What it solves | What it misses |
|---|---|---|
| **BuffetPay** | Multi-wallet + guardrails for x402 | Centralised; no loop detection; no pre-execution estimation |
| **Coinbase x402 SDK** | How to *pay* | Does not judge whether the agent *should* pay |
| **Kite Passport (alone)** | Static quotas (max $X/day) | No runtime pattern detection (loops, repetition); no pre-estimation |
| **AgentTrust** | Prompt injection / execution safety | Orthogonal — does not touch payment governance |
| **PennyKite** | Pre-execution cost estimation **+** atomic budget enforcement **+** loop detection **+** Kite Passport integration **+** on-chain audit | Hackathon-stage; pathway to production |

The differentiator in one line: **PennyKite decides before the agent pays — not after.**

---

## Demo: the runaway agent

The flagship scenario, runnable locally with `./demo/run-demo.sh`:

```
Agent loop bug: trying to call Neynar 200x because of a hallucinated retry policy.

WITHOUT PennyKite:
  Calls 1-200 ─────────────────► T=16s     $10.00 drained, no audit, no recovery.

WITH PennyKite:
  Call 1   T=0ms     $0.05    APPROVE   (similarity 0.0)
  Call 2   T=80ms    $0.05    APPROVE   (similarity 0.4)
  Call 3   T=160ms   $0.05    APPROVE   (similarity 0.7)
  Call 4   T=240ms   $0.05    DENY_LOOP (similarity 0.91 > 0.85)
                              → HTTP 402 returned to agent
                              → Telegram alert
                              → Kite attestation tx: 0x7a2f…e91c
```

| Metric | Without PennyKite | With PennyKite |
|---|---:|---:|
| Calls before detection | ~200 | **4** |
| USDC drained | $10.00 | **$0.15** |
| Time to detection | hours | **<250 ms** |
| Audit trail | none | **on-chain (Kite)** |
| Latency overhead (steady-state) | — | **~5 ms** |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         AGENT                                    │
│   (LangChain / OpenAI Agents SDK / custom AI agent)              │
└─────────────────┬───────────────────────────────────────────────┘
                  │
                  │ HTTPS request to ANY x402-enabled API
                  ▼
┌─────────────────────────────────────────────────────────────────┐
│  PENNYKITE PROXY                          (Rust · axum · :8787) │
│                                                                  │
│   1. Pre-execution cost estimation                               │
│   2. Atomic budget check       (SQLite row-locked reservation)   │
│   3. Loop detection            (fingerprint sliding window)      │
│   4. Kite Passport session validation  (on-chain quota check)    │
│   5. EIP-3009 PAYMENT-SIGNATURE   (forward to upstream)          │
│   6. Post-settlement reconciliation                              │
└─────────────────┬───────────────────────────────────────────────┘
                  │
                  ▼
        ┌─────────────────────┐         ┌────────────────────────┐
        │  TARGET x402 API    │◄────────│  PennyKite Attestor    │
        │  (any endpoint)     │         │  (Solidity, Kite chain)│
        └─────────────────────┘         └────────────────────────┘
                  ▲                                ▲
                  │                                │ hash(decision)
                  │                                │
┌─────────────────┴───────────────────────────────┴───────────────┐
│  CONTROL PLANE                          (Next.js · Vercel)       │
│   • Live spending feed       • Per-session detail                │
│   • Kill-switch (revokes Kite Passport session key)              │
│   • Telegram / Discord / webhook alerting                        │
└──────────────────────────────────────────────────────────────────┘
```

### Key architectural decisions

- **Reverse proxy, not SDK.** A compromised agent cannot disable what it does not control.
- **Pre-execution estimation, not post-execution metering.** x402 is atomic — once `PAYMENT-SIGNATURE` is sent, the money has moved.
- **Every decision anchored on Kite.** The off-chain ledger can be tampered with by a compromised host; the on-chain hash cannot.
- **Kill-switch lives on Kite Passport, not on PennyKite.** When everything else fails, the chain still says *no*.
- **Network-agnostic for x402, Kite-anchored for attestations.** PennyKite accepts any x402 network the policy allows; the audit trail is always on Kite.

---

## Dashboard control plane

The Next.js dashboard (`http://localhost:3000`) exposes three pages, all backed by the proxy's REST API — the dashboard never reads SQLite directly.

| Page | URL | What it shows |
|---|---|---|
| **Live feed** | `/` | Real-time stream of all proxy decisions (APPROVE / DENY) as they arrive |
| **Session detail** | `/sessions/[id]` | Per-session spend, request timeline, and full decision history |
| **Kill-switch** | `/kill-switch` | List of active sessions with one-click pause controls |

### Kill-switch behaviour

Clicking **Pause** on a session:

1. The dashboard UI calls the Next.js API bridge at `POST /api/sessions/{id}/pause`.
2. The Next.js bridge forwards the request to the proxy REST endpoint `POST /api/sessions/{id}/pause`.
3. The proxy marks the session `paused` in SQLite.
4. Every subsequent payment request for that session is rejected immediately:

```
HTTP/1.1 402 Payment Required
{"reason":"session is paused"}
```

> **On-chain revocation status:** Revoking the Kite Passport session key on-chain is currently **mock / best-effort**. Full on-chain revocation will be wired to the Kite revocation API once it is available on testnet.

---

## Project structure

```
pennykite/
├── crates/
│   ├── pennykite-types/          # Shared types (PaymentRequired, Policy, Decision, Verdict)
│   ├── pennykite-budget/         # Budget enforcement primitives
│   ├── pennykite-detect/         # Loop / anomaly detection
│   ├── pennykite-cost/           # Pre-execution cost estimation
│   ├── pennykite-ledger/         # Atomic SQLite ledger
│   ├── pennykite-providers/      # Upstream provider adapters
│   ├── pennykite-kite/           # Kite Passport + attestor client
│   └── pennykite-proxy/          # axum HTTP reverse proxy (binary)
├── contracts/                    # Foundry workspace
│   ├── src/PennyKiteAttestor.sol
│   └── test/
├── sdk/
│   ├── python/pennykite/         # Convenience wrapper
│   └── typescript/               # (planned)
├── dashboard/                    # Next.js 16 control plane
├── policy/examples/              # YAML policy presets
├── demo/
│   ├── target-api/               # Express + x402 middleware
│   ├── runaway-agent/            # The bugged agent (Python)
│   └── run-demo.sh               # One-command end-to-end
└── backlog.yaml                  # Machine-readable execution plan
```

---

## Quickstart

### Prerequisites

```bash
node --version    # >= 22
rustc --version   # >= 1.80
forge --version   # Foundry
python3 --version # >= 3.10
```

### One-command demo

```bash
git clone https://github.com/manuelpenazuniga/PennyKite.git
cd PennyKite
cp .env.example .env   # fill in KITE_RPC_URL, KITE_PRIVATE_KEY, etc.
./demo/run-demo.sh
```

This will:

1. Build the Rust workspace (`cargo build --workspace --release`).
2. Boot the demo target API on `:4100` (Express + x402).
3. Boot the PennyKite proxy on `:8787`.
4. Boot the dashboard on `:3000`.
5. Run the runaway agent in **unprotected** mode, then in **protected** mode.

Open [http://localhost:3000](http://localhost:3000) to watch decisions stream in real time.

### Manual setup

<details>
<summary>Click to expand</summary>

```bash
# 1. Build the proxy
cargo build --workspace --release

# 2. Deploy the attestor to Kite testnet
cd contracts
forge install
forge test
forge create src/PennyKiteAttestor.sol:PennyKiteAttestor \
  --rpc-url $KITE_RPC_URL \
  --private-key $KITE_PRIVATE_KEY \
  --broadcast

# Deployed on Kite testnet:
# PENNYKITE_ATTESTOR_ADDRESS=0x3973Ce9a493EeB190A1Ae8ABbEb960533242d762
# Deploy tx: 0xb3ec20954ff43e68c910af6d60689eba621ca9d02bad512ea3c1cc1c304953f2
# Smoke attest tx: 0xab95060fa504238bd3fc1f1c27364160c383d5e295050fc780016abd9b311e7f

# 3. Run the proxy
./target/release/pennykite-proxy \
  --policy ./policy/examples/conservative.yaml \
  --upstream http://localhost:4100 \
  --listen 0.0.0.0:8787 \
  --db ./pennykite.db \
  --kite-rpc-url "$KITE_RPC_URL" \
  --kite-chain-id "$KITE_CHAIN_ID" \
  --attestor-address "$PENNYKITE_ATTESTOR_ADDRESS" \
  --kite-private-key "$KITE_PRIVATE_KEY"

# 4. Run the dashboard
cd dashboard
npm install
npm run dev
```

</details>

---

## Local demo

Step-by-step walkthrough of the full control-plane flow without the one-command script.

### 1. Start the target API

```bash
cd demo/target-api && npm run start
# Listens on :4100
```

### 2. Start the proxy

```bash
cargo run -p pennykite-proxy -- \
  --listen 127.0.0.1:8787 \
  --policy policy/examples/conservative.yaml \
  --upstream http://127.0.0.1:4100 \
  --db /tmp/pennykite-demo.db
```

### 3. Start the dashboard

```bash
cd dashboard && PENNYKITE_PROXY_URL=http://127.0.0.1:8787 npm run dev
```

Open [http://localhost:3000](http://localhost:3000).

### 4. Generate a session and trigger an approval

```bash
curl -i -H 'x-pennykite-session: demo-session' \
  http://127.0.0.1:8787/proxy/predict/demo-1
# → 200 OK (APPROVE) — decision appears on the live feed
```

### 5. Explore the dashboard

| URL | What you see |
|---|---|
| `http://localhost:3000` | Live feed with the `demo-session` APPROVE event |
| `http://localhost:3000/sessions/demo-session` | Session detail — spend timeline and decisions |
| `http://localhost:3000/kill-switch` | Pause controls for all active sessions |

### 6. Pause the session

```bash
curl -i -X POST http://127.0.0.1:8787/api/sessions/demo-session/pause
# → 202 Accepted
```

### 7. Verify the session is blocked

```bash
curl -i -H 'x-pennykite-session: demo-session' \
  http://127.0.0.1:8787/proxy/predict/demo-after-pause
# → HTTP/1.1 402 Payment Required
# {"reason":"session is paused"}
```

### Troubleshooting

| Symptom | Fix |
|---|---|
| Dashboard shows no data | Verify `PENNYKITE_PROXY_URL=http://127.0.0.1:8787` — the dashboard reads from the proxy REST API, not directly from SQLite |
| Port 8787 already in use | Change `--listen 127.0.0.1:<port>` and update `PENNYKITE_PROXY_URL` to match |
| Port 3000 already in use | Run `npx next dev --port 3001` inside `dashboard/` (the `dev` script hard-codes `--port 3000`) |

---

## Configuration

Policies are declarative YAML, validated at proxy boot:

```yaml
version: "1.0"
name: "Conservative trading agent"

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
  allowed:
    - "eip155:8453"     # Base
    - "eip155:84532"    # Base Sepolia
    - "kite:testnet"

assets:
  allowed: ["USDC", "PYUSD"]

kite_passport:
  agent_address: "0x..."
  session_key:   "0x..."
  attestation_frequency: "every_event"
```

Bundled presets: `conservative.yaml`, `trading-aggressive.yaml`, `consumer.yaml`.

---

## SDK

> **Status:** The Python SDK exists as a local package under `sdk/python`. Packages are not published yet. The TypeScript SDK remains planned.

### Python

```python
from pennykite import PennyKiteClient

with PennyKiteClient(session_id="trade-2026-05-25") as client:
    response = client.get("https://api.neynar.com/v2/farcaster/casts?fid=123")
    # Will raise PennyKiteBudgetExceeded or PennyKiteLoopDetected on 402.
```

### TypeScript (planned)

```ts
// Target API surface — not yet published
import { PennyKiteClient } from "@pennykite/client";

const client = new PennyKiteClient({ sessionId: "trade-2026-05-25" });
const data = await client.fetch("https://api.neynar.com/...").then(r => r.json());
```

---

## Roadmap

| Milestone | Status | Notes |
|---|---|---|
| **v0.1 — Hackathon submission** (25 May 2026) | 🚧 In progress | Proxy + ledger + loop detector + Kite attestor + dashboard |
| **v0.2 — Production hardening** | ⏳ Planned | Multi-region, gRPC streaming, Postgres backend |
| **v0.3 — Federated reputation** | ⏳ Planned | Cross-tenant signal sharing on agent risk |
| **v0.4 — MCP integration** | ⏳ Planned | First-class Model Context Protocol bridge |
| **v0.5 — Audit export formats** | ⏳ Planned | SOC2 CC7.x mapping, NDJSON for SIEM ingestion |

The detailed atomic execution plan lives in [`backlog.yaml`](./backlog.yaml).

---

## Hackathon submission

PennyKite is being built for the **[Kite AI Global Hackathon 2026](https://www.encodeclub.com/programmes/kites-hackathon-ai-agentic-economy)** by Encode Club.

- **Primary track:** Novel — *"anything that runs on Kite and does something nobody's seen before."*
- **Secondary fit:** Agentic Commerce — agent-to-API payments via x402 with programmable constraints.
- **Finale:** 25 May 2026, 4:00 PM GMT+1.

What the judges will see end-to-end:

- ✅ AI agent performing a real paid task (sport-prediction API at $0.05/call).
- ✅ x402 settlement on testnet.
- ✅ On-chain attestations on Kite chain (`PennyKiteAttestor`).
- ✅ Functional UI (Next.js dashboard + kill-switch).
- ✅ Publicly accessible hosted demo + reproducible repo.

---

## Contributing

PennyKite is open source under the MIT licence and we welcome contributions. Please:

1. Open an issue describing the change before sending a non-trivial PR.
2. Run `cargo test --workspace` and `forge test` locally before pushing.
3. Keep PRs scoped — one concern per PR.
4. All commits, code and documentation in **English**.

---

## Licence

[MIT](./LICENSE) © 2026 PennyKite contributors.

---

## Acknowledgements

PennyKite extends ideas and components from [PennyPrompt](https://github.com/manuelpenazuniga), the LLM token-budget reverse proxy that pioneered the *out-of-process governance* pattern this project generalises to x402.

Built with [Kite AI](https://gokite.ai), [x402](https://github.com/coinbase/x402), [Coinbase CDP](https://docs.cdp.coinbase.com/x402/welcome), [Foundry](https://book.getfoundry.sh/), [axum](https://github.com/tokio-rs/axum), [alloy](https://github.com/alloy-rs/alloy), [Next.js](https://nextjs.org/) and [shadcn/ui](https://ui.shadcn.com/).

Special thanks to the Encode Club community and the Kite AI team for shipping the primitives that made this possible.

---

<div align="center">

**Build it. Ship it. Iterate.**

</div>

# Demo target API

A tiny x402-gated prediction API used in the PennyKite hackathon walkthrough.
Implements the `PAYMENT-REQUIRED` / `PAYMENT-SIGNATURE` / `PAYMENT-RESPONSE`
flow by hand so the demo does not depend on the upstream `@x402/express`
package version.

## Endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/health` | none | Liveness probe |
| `GET` | `/free-quote/:matchId` | none | Sample free response |
| `GET` | `/predict/:matchId` | `PAYMENT-SIGNATURE` header | Returns a mock match prediction |

When `/predict/:matchId` is called without a `PAYMENT-SIGNATURE` header, the
server responds with:

- HTTP `402 Payment Required`
- Header `PAYMENT-REQUIRED: <base64-encoded JSON>`

The decoded JSON follows the x402 schema:

```json
{
  "version": "1.0",
  "requirements": [
    {
      "network": "eip155:84532",
      "asset": "USDC",
      "amount": "50000",
      "decimals": 6,
      "pay_to": "0x000000000000000000000000000000000000dEaD",
      "facilitator": "https://facilitator.x402.example.com"
    }
  ],
  "expires_at": "2026-05-25T16:00:00.000Z"
}
```

When the header is present, the server returns `200 OK` with a mock prediction
and a `PAYMENT-RESPONSE` header echoing the first 16 chars of the signature
(for traceability in the dashboard).

## Configuration

All settings are environment variables (see `.env.example` at the repo root):

| Var | Default | Description |
|---|---|---|
| `PORT` | `4100` | Listen port |
| `PRICE_USD` | `0.05` | Price per `/predict` call in USD |
| `NETWORK` | `eip155:84532` | CAIP-2 chain id (Base Sepolia by default) |
| `ASSET` | `USDC` | Asset symbol |
| `DECIMALS` | `6` | Asset decimals |
| `PAY_TO` | `0x...dEaD` | Merchant receiving address |
| `FACILITATOR_URL` | `https://facilitator.x402.example.com` | Facilitator endpoint |

## Run

```bash
cd demo/target-api
npm install
npm start
```

## Quick verification

```bash
# Free endpoint
curl -s http://localhost:4100/free-quote/test | jq

# Paid endpoint without payment — expect HTTP 402
curl -sv http://localhost:4100/predict/test 2>&1 | grep -E "HTTP|PAYMENT"

# Paid endpoint with mock signature — expect HTTP 200
curl -s -H "PAYMENT-SIGNATURE: 0xdeadbeef" http://localhost:4100/predict/test | jq
```

In the full PennyKite demo the runaway agent (PK-D3-04) talks to this server
either directly (unprotected mode) or through the PennyKite proxy (protected
mode, which intercepts the 402 and decides whether to sign and forward).

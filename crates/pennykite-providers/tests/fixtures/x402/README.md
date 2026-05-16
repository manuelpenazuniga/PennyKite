# x402 PAYMENT-REQUIRED fixtures

Synthetic test vectors covering the networks PennyKite must support. Each
file is the *decoded* JSON body of a `PAYMENT-REQUIRED` header — Base64
encoding is applied at parse time.

| File | Network | Asset | Amount (USD) |
|---|---|---|---|
| `base-usdc.json` | `eip155:8453` | USDC | 0.05 |
| `base-sepolia-usdc.json` | `eip155:84532` | USDC | 0.01 |
| `polygon-usdt.json` | `eip155:137` | USDT | 0.25 |
| `arbitrum-pyusd.json` | `eip155:42161` | PYUSD | 0.10 |
| `kite-testnet-usdc.json` | `eip155:2368` | USDC | 0.02 |
| `multi-options.json` | mixed | USDC + PYUSD | 0.05 |

**Note**: these are synthetic; replace with captured headers from a real
x402 facilitator once PK-D0-07 produces them. No private keys are committed.

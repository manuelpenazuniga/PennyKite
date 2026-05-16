// PennyKite demo target API
//
// A minimal Express server that gates a single endpoint behind the x402
// payment-required protocol. This implementation hand-rolls the protocol
// (rather than depending on @x402/express) so the demo remains reproducible
// even if upstream package names shift before submission.

import express from "express";
import "dotenv/config";

const PORT = Number(process.env.PORT ?? 4100);
const PRICE_USD = Number(process.env.PRICE_USD ?? 0.05);
const NETWORK = process.env.NETWORK ?? "eip155:84532"; // Base Sepolia
const ASSET = process.env.ASSET ?? "USDC";
const DECIMALS = Number(process.env.DECIMALS ?? 6);
const PAY_TO = process.env.PAY_TO ?? "0x000000000000000000000000000000000000dEaD";
const FACILITATOR = process.env.FACILITATOR_URL ?? "https://facilitator.x402.example.com";

const app = express();
app.use(express.json());

// --- helpers ----------------------------------------------------------------

function amountSmallestUnits(usd, decimals) {
  return Math.round(usd * 10 ** decimals).toString();
}

function buildPaymentRequired(price = PRICE_USD) {
  const body = {
    version: "1.0",
    requirements: [
      {
        network: NETWORK,
        asset: ASSET,
        amount: amountSmallestUnits(price, DECIMALS),
        decimals: DECIMALS,
        pay_to: PAY_TO,
        facilitator: FACILITATOR,
      },
    ],
    expires_at: new Date(Date.now() + 5 * 60_000).toISOString(),
  };
  return Buffer.from(JSON.stringify(body), "utf8").toString("base64");
}

function send402(res, price) {
  const header = buildPaymentRequired(price);
  res
    .status(402)
    .set("PAYMENT-REQUIRED", header)
    .json({
      error: "payment_required",
      message: "Pay via x402 to access this resource.",
      price_usd: price,
    });
}

// --- routes -----------------------------------------------------------------

app.get("/health", (_req, res) => {
  res.json({ status: "ok", service: "pennykite-demo-target-api" });
});

// Free endpoint to confirm wiring.
app.get("/free-quote/:matchId", (req, res) => {
  res.json({
    match_id: req.params.matchId,
    free: true,
    quote: "Public sample data — no payment required.",
  });
});

// Paid endpoint: returns 402 unless PAYMENT-SIGNATURE header is present.
// In a real deployment the facilitator would verify and settle the signature;
// for the demo we accept any non-empty value so the proxy can wire end-to-end.
app.get("/predict/:matchId", (req, res) => {
  const signature = req.header("PAYMENT-SIGNATURE");
  if (!signature || signature.trim() === "") {
    return send402(res, PRICE_USD);
  }

  const matchId = req.params.matchId;
  const predictions = ["home_win", "away_win", "draw"];
  const pick = predictions[Math.floor(Math.random() * predictions.length)];

  res
    .status(200)
    .set("PAYMENT-RESPONSE", buildPaymentResponse(signature, PRICE_USD))
    .json({
      match_id: matchId,
      prediction: pick,
      confidence: Math.round(Math.random() * 100) / 100,
      generated_at: new Date().toISOString(),
    });
});

function buildPaymentResponse(signature, price) {
  const body = {
    version: "1.0",
    settled: true,
    amount_usd: price,
    signature_echo: signature.slice(0, 16),
  };
  return Buffer.from(JSON.stringify(body), "utf8").toString("base64");
}

// --- bootstrap --------------------------------------------------------------

app.listen(PORT, () => {
  console.log(
    `[demo-target-api] listening on :${PORT} (price=$${PRICE_USD.toFixed(4)} ${ASSET} @ ${NETWORK})`,
  );
});

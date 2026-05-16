import { NextResponse } from "next/server";
import type { Decision, SessionSummary, Verdict } from "@/lib/types";

const SESSION_ID = "demo-session-001";
const BUDGET = 5.0;

const SAMPLE_REQUESTS: Array<{ key: string; cost: number; verdict: Verdict; reason: string }> = [
  { key: "GET /v1/quote?pair=ETH-USDC", cost: 0.0050, verdict: "approve", reason: "within budget" },
  { key: "POST /v1/swap", cost: 0.0500, verdict: "approve", reason: "within budget" },
  { key: "GET /v1/quote?pair=ETH-USDC", cost: 0.0050, verdict: "approve", reason: "within budget" },
  { key: "GET /v1/quote?pair=ETH-USDC", cost: 0.0050, verdict: "deny_loop", reason: "3 consecutive identical requests" },
  { key: "POST /v1/swap", cost: 0.6000, verdict: "deny_budget", reason: "exceeds per-request cap of $0.50" },
  { key: "GET /v1/balance", cost: 0.0010, verdict: "approve", reason: "within budget" },
  { key: "POST /v1/order", cost: 0.0250, verdict: "approve", reason: "within budget" },
  { key: "GET /v1/orderbook", cost: 0.0030, verdict: "approve", reason: "within budget" },
];

function randomHash(): string {
  const bytes = new Uint8Array(32);
  for (let i = 0; i < bytes.length; i++) bytes[i] = Math.floor(Math.random() * 256);
  return "0x" + Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("");
}

function randomId(): string {
  return crypto.randomUUID();
}

let cachedDecisions: Decision[] | null = null;

function buildDecisions(): Decision[] {
  if (cachedDecisions) return cachedDecisions;
  const now = Date.now();
  cachedDecisions = SAMPLE_REQUESTS.map((req, i) => ({
    id: randomId(),
    session_id: SESSION_ID,
    timestamp: new Date(now - (SAMPLE_REQUESTS.length - i) * 4_500).toISOString(),
    request_key: req.key,
    estimated_cost_usd: req.cost,
    verdict: req.verdict,
    reason: req.reason,
    decision_hash: randomHash(),
    kite_attestation_tx: req.verdict === "approve" ? randomHash() : null,
  }));
  return cachedDecisions;
}

export async function GET() {
  const decisions = buildDecisions();
  const spent = decisions
    .filter((d) => d.verdict === "approve")
    .reduce((sum, d) => sum + d.estimated_cost_usd, 0);

  const summary: SessionSummary = {
    session_id: SESSION_ID,
    budget_usd: BUDGET,
    spent_usd: spent,
    decisions_count: decisions.length,
  };

  return NextResponse.json({
    summary,
    decisions: [...decisions].reverse(),
  });
}

export type Verdict =
  | "approve"
  | "deny"
  | "deny_loop"
  | "deny_budget"
  | "deny_network"
  | "deny_asset"
  | "session_invalid";

export interface Decision {
  id: string;
  session_id: string;
  timestamp: string;
  request_key: string;
  estimated_cost_usd: number;
  verdict: Verdict;
  reason: string;
  decision_hash: string;
  kite_attestation_tx: string | null;
}

export interface SessionSummary {
  session_id: string;
  budget_usd: number;
  spent_usd: number;
  decisions_count: number;
}

export interface DecisionFeed {
  summary: SessionSummary;
  decisions: Decision[];
}

export interface SessionControlSummary {
  id: string;
  budget_usd: number;
  spent_usd: number;
  status: "active" | "paused" | string;
  decisions_count: number;
}

export interface SessionsResponse {
  sessions: SessionControlSummary[];
}

export interface PauseSessionResponse {
  status: string;
  session_id: string;
  kite_revocation: string;
}

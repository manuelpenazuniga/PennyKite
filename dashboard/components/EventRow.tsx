import { Badge } from "./Badge";
import type { Decision } from "@/lib/types";

function formatCost(cost: number): string {
  return `$${cost.toFixed(4)}`;
}

function formatTime(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function shortHash(hash: string): string {
  if (hash.length <= 12) return hash;
  return `${hash.slice(0, 6)}…${hash.slice(-4)}`;
}

export function EventRow({ decision }: { decision: Decision }) {
  return (
    <div className="grid grid-cols-12 gap-3 items-center py-3 px-4 border-b border-zinc-800 hover:bg-zinc-900/50 transition-colors">
      <span className="col-span-2 text-xs text-zinc-500 font-mono">
        {formatTime(decision.timestamp)}
      </span>
      <span className="col-span-2">
        <Badge verdict={decision.verdict} />
      </span>
      <span className="col-span-4 text-sm text-zinc-300 truncate font-mono">
        {decision.request_key}
      </span>
      <span className="col-span-2 text-sm text-zinc-100 font-mono text-right">
        {formatCost(decision.estimated_cost_usd)}
      </span>
      <span
        className="col-span-2 text-xs text-zinc-500 font-mono truncate text-right"
        title={decision.reason}
      >
        {shortHash(decision.decision_hash)}
      </span>
    </div>
  );
}

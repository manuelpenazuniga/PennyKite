import type { Verdict } from "@/lib/types";

const verdictStyles: Record<Verdict, { label: string; className: string }> = {
  approve: {
    label: "APPROVE",
    className: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
  },
  deny: {
    label: "DENY",
    className: "bg-rose-500/10 text-rose-400 border-rose-500/20",
  },
  deny_loop: {
    label: "DENY · LOOP",
    className: "bg-amber-500/10 text-amber-400 border-amber-500/20",
  },
  deny_budget: {
    label: "DENY · BUDGET",
    className: "bg-rose-500/10 text-rose-400 border-rose-500/20",
  },
  deny_network: {
    label: "DENY · NETWORK",
    className: "bg-rose-500/10 text-rose-400 border-rose-500/20",
  },
  deny_asset: {
    label: "DENY · ASSET",
    className: "bg-rose-500/10 text-rose-400 border-rose-500/20",
  },
  session_invalid: {
    label: "SESSION INVALID",
    className: "bg-zinc-500/10 text-zinc-400 border-zinc-500/20",
  },
};

export function Badge({ verdict }: { verdict: Verdict }) {
  const style = verdictStyles[verdict];
  return (
    <span
      className={`inline-flex items-center rounded-md border px-2 py-0.5 text-xs font-mono font-medium ${style.className}`}
    >
      {style.label}
    </span>
  );
}

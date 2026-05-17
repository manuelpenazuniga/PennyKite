import type { Verdict } from "@/lib/types";

const verdictStyles: Record<Verdict, { label: string; className: string }> = {
  approve: {
    label: "APPROVE",
    className: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
  },
  deny: {
    label: "DENY",
    className: "bg-rose-500/30 text-rose-300 border-rose-500/50 shadow-[0_0_8px_rgba(244,63,94,0.4)]",
  },
  deny_loop: {
    label: "DENY · LOOP",
    className: "bg-amber-600 text-white border-amber-400 font-bold shadow-[0_0_12px_rgba(217,119,6,0.6)] animate-pulse",
  },
  deny_budget: {
    label: "DENY · BUDGET",
    className: "bg-rose-500/30 text-rose-300 border-rose-500/50 shadow-[0_0_8px_rgba(244,63,94,0.4)]",
  },
  deny_network: {
    label: "DENY · NETWORK",
    className: "bg-rose-500/30 text-rose-300 border-rose-500/50 shadow-[0_0_8px_rgba(244,63,94,0.4)]",
  },
  deny_asset: {
    label: "DENY · ASSET",
    className: "bg-rose-500/30 text-rose-300 border-rose-500/50 shadow-[0_0_8px_rgba(244,63,94,0.4)]",
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

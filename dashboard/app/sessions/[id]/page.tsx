import { Badge } from "@/components/Badge";
import { Stat } from "@/components/Stat";
import { proxyUrl } from "@/lib/proxy";
import type { Decision, DecisionFeed } from "@/lib/types";

export const dynamic = "force-dynamic";

interface SessionPageProps {
  params: Promise<{ id: string }>;
}

function formatCost(cost: number): string {
  return `$${cost.toFixed(4)}`;
}

function formatTimestamp(iso: string): string {
  return new Date(iso).toISOString().replace("T", " ").replace(".000Z", "Z");
}

function shortHash(hash: string): string {
  if (hash.length <= 12) return hash;
  return `${hash.slice(0, 8)}...${hash.slice(-6)}`;
}

async function loadSession(id: string): Promise<DecisionFeed | null> {
  const res = await fetch(proxyUrl(`/api/sessions/${encodeURIComponent(id)}`), {
    cache: "no-store",
  });
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(`proxy returned HTTP ${res.status}`);
  return (await res.json()) as DecisionFeed;
}

function ExplorerCell({ decision }: { decision: Decision }) {
  if (!decision.kite_attestation_tx) {
    return <span className="text-zinc-600">pending</span>;
  }

  return (
    <a
      className="text-sky-300 hover:text-sky-200"
      href={`https://explorer.gokite.ai/tx/${decision.kite_attestation_tx}`}
      rel="noreferrer"
      target="_blank"
    >
      {shortHash(decision.kite_attestation_tx)}
    </a>
  );
}

function TimelineRow({ decision }: { decision: Decision }) {
  return (
    <div className="grid grid-cols-12 gap-3 border-b border-zinc-800 px-4 py-3 text-sm">
      <span className="col-span-3 font-mono text-xs text-zinc-500">
        {formatTimestamp(decision.timestamp)}
      </span>
      <span className="col-span-2">
        <Badge verdict={decision.verdict} />
      </span>
      <span className="col-span-3 truncate font-mono text-zinc-300">
        {decision.request_key}
      </span>
      <span className="col-span-1 text-right font-mono text-zinc-100">
        {formatCost(decision.estimated_cost_usd)}
      </span>
      <span className="col-span-3 truncate text-right font-mono text-xs">
        <ExplorerCell decision={decision} />
      </span>
    </div>
  );
}

export default async function SessionPage({ params }: SessionPageProps) {
  const { id } = await params;
  const data = await loadSession(id);

  if (!data) {
    return (
      <main className="mx-auto min-h-screen max-w-5xl p-8">
        <a className="text-sm text-zinc-500 hover:text-zinc-300" href="/">
          Back to live feed
        </a>
        <section className="mt-8 rounded-lg border border-rose-500/30 bg-rose-500/10 p-6">
          <h1 className="text-2xl font-bold">Session not found</h1>
          <p className="mt-2 font-mono text-sm text-rose-200">{id}</p>
        </section>
      </main>
    );
  }

  const { summary, decisions } = data;
  const remaining = Math.max(0, summary.budget_usd - summary.spent_usd);
  const utilisation =
    summary.budget_usd > 0 ? Math.min(100, (summary.spent_usd / summary.budget_usd) * 100) : 0;

  return (
    <main className="mx-auto min-h-screen max-w-6xl p-8">
      <a className="text-sm text-zinc-500 hover:text-zinc-300" href="/">
        Back to live feed
      </a>

      <header className="mt-6 mb-8">
        <h1 className="text-3xl font-bold tracking-tight">Session detail</h1>
        <p className="mt-2 break-all font-mono text-xs text-zinc-500">
          {summary.session_id}
        </p>
      </header>

      <div className="mb-6 grid grid-cols-1 gap-4 sm:grid-cols-4">
        <Stat label="Budget" value={`$${summary.budget_usd.toFixed(2)}`} />
        <Stat label="Spent" value={formatCost(summary.spent_usd)} />
        <Stat label="Remaining" value={formatCost(remaining)} />
        <Stat label="Decisions" value={`${summary.decisions_count}`} />
      </div>

      <section className="mb-8 rounded-lg border border-zinc-800 bg-zinc-900/30 p-4">
        <div className="mb-2 flex items-center justify-between text-xs text-zinc-500">
          <span>Budget used</span>
          <span className="font-mono">{utilisation.toFixed(1)}%</span>
        </div>
        <div className="h-3 overflow-hidden rounded-full bg-zinc-800">
          <div
            className="h-full rounded-full bg-emerald-400"
            style={{ width: `${utilisation}%` }}
          />
        </div>
      </section>

      <section className="rounded-lg border border-zinc-800 bg-zinc-900/30">
        <header className="grid grid-cols-12 gap-3 border-b border-zinc-800 px-4 py-3 text-xs uppercase tracking-wider text-zinc-500">
          <span className="col-span-3">Time</span>
          <span className="col-span-2">Verdict</span>
          <span className="col-span-3">Request</span>
          <span className="col-span-1 text-right">Cost</span>
          <span className="col-span-3 text-right">Kite attestation</span>
        </header>
        {decisions.length === 0 && (
          <div className="px-4 py-8 text-center text-sm text-zinc-500">
            No decisions recorded for this session yet.
          </div>
        )}
        {decisions.map((decision) => (
          <TimelineRow key={decision.id} decision={decision} />
        ))}
      </section>
    </main>
  );
}

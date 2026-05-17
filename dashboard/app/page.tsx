"use client";

import { useEffect, useState } from "react";
import { Stat } from "@/components/Stat";
import { EventRow } from "@/components/EventRow";
import type { DecisionFeed } from "@/lib/types";

export default function Home() {
  const [data, setData] = useState<DecisionFeed | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const res = await fetch("/api/decisions", { cache: "no-store" });
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const json = (await res.json()) as DecisionFeed;
        if (!cancelled) {
          setData(json);
          setError(null);
          setLoaded(true);
        }
      } catch (e) {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : "fetch failed");
          setLoaded(true);
        }
      }
    }

    load();
    const id = setInterval(load, 500);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, []);

  const summary = data?.summary;
  const decisions = data?.decisions ?? [];
  const remaining = summary ? summary.budget_usd - summary.spent_usd : 0;
  const utilisation = summary ? (summary.spent_usd / summary.budget_usd) * 100 : 0;

  return (
    <main className="min-h-screen p-8 max-w-6xl mx-auto">
      <header className="mb-8">
        <h1 className="text-3xl font-bold tracking-tight">PennyKite</h1>
        <p className="text-zinc-400 text-sm mt-1">
          Pre-execution budget guardian for the agentic economy.
        </p>
        <a className="mt-3 inline-block text-sm text-rose-300 hover:text-rose-200" href="/kill-switch">
          Open kill switch
        </a>
        {summary && (
          <p className="text-xs text-zinc-500 font-mono mt-2">
            session: {summary.session_id}
          </p>
        )}
      </header>

      {error && (
        <div className="mb-6 rounded-lg border border-rose-500/30 bg-rose-500/10 p-4 text-sm text-rose-300">
          Failed to load feed: {error}
        </div>
      )}

      <div className="mb-4 p-4 rounded-lg border border-amber-500/30 bg-amber-500/10 flex justify-between items-center shadow-[0_0_12px_rgba(245,158,11,0.2)]">
        <span className="text-amber-400 font-medium tracking-wide uppercase text-sm">Demo wallet balance</span>
        <span className="text-2xl text-amber-300 font-mono tracking-tight font-bold">
          {summary ? `$${remaining.toFixed(4)}` : "—"}
        </span>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-4 gap-4 mb-8">
        <Stat
          label="Budget"
          value={summary ? `$${summary.budget_usd.toFixed(2)}` : "—"}
        />
        <Stat
          label="Spent"
          value={summary ? `$${summary.spent_usd.toFixed(4)}` : "—"}
          tone={utilisation > 80 ? "warning" : "positive"}
          hint={summary ? `${utilisation.toFixed(1)}% used` : undefined}
        />
        <Stat
          label="Remaining"
          value={summary ? `$${remaining.toFixed(4)}` : "—"}
        />
        <Stat
          label="Decisions"
          value={summary ? `${summary.decisions_count}` : "—"}
        />
      </div>

      <section className="rounded-lg border border-zinc-800 bg-zinc-900/30 overflow-hidden relative">
        <header className="sticky top-0 z-10 bg-zinc-900/95 backdrop-blur grid grid-cols-12 gap-3 px-4 py-3 border-b border-zinc-800 text-xs uppercase tracking-wider text-zinc-500 shadow-sm">
          <span className="col-span-2">Time</span>
          <span className="col-span-2">Verdict</span>
          <span className="col-span-4">Request</span>
          <span className="col-span-2 text-right">Cost</span>
          <span className="col-span-2 text-right">Hash</span>
        </header>
        {!loaded && (
          <div className="px-4 py-8 text-center text-sm text-zinc-500">
            Loading decision feed…
          </div>
        )}
        {loaded && decisions.length === 0 && !error && (
          <div className="px-4 py-8 text-center text-sm text-zinc-500">
            No decisions yet. Send traffic through the proxy to populate this feed.
          </div>
        )}
        {decisions.map((d) => (
          <EventRow key={d.id} decision={d} />
        ))}
      </section>
    </main>
  );
}

"use client";

import { useEffect, useMemo, useState } from "react";
import { Stat } from "@/components/Stat";
import type { SessionControlSummary, SessionsResponse } from "@/lib/types";

function formatCost(value: number): string {
  return `$${value.toFixed(4)}`;
}

function statusClass(status: string): string {
  if (status === "active") return "border-emerald-500/30 bg-emerald-500/10 text-emerald-300";
  if (status === "paused") return "border-rose-500/40 bg-rose-500/15 text-rose-300";
  return "border-zinc-500/30 bg-zinc-500/10 text-zinc-300";
}

function SessionRow({
  session,
  pending,
  onPause,
}: {
  session: SessionControlSummary;
  pending: boolean;
  onPause: (id: string) => void;
}) {
  const remaining = Math.max(0, session.budget_usd - session.spent_usd);
  const utilisation =
    session.budget_usd > 0 ? Math.min(100, (session.spent_usd / session.budget_usd) * 100) : 0;
  const active = session.status === "active";

  return (
    <div className="grid grid-cols-12 gap-3 border-b border-zinc-800 px-4 py-4 text-sm">
      <div className="col-span-4 min-w-0">
        <a
          className="block truncate font-mono text-zinc-100 hover:text-white"
          href={`/sessions/${encodeURIComponent(session.id)}`}
        >
          {session.id}
        </a>
        <div className="mt-2 h-2 overflow-hidden rounded-full bg-zinc-800">
          <div
            className={active ? "h-full bg-emerald-400" : "h-full bg-rose-400"}
            style={{ width: `${utilisation}%` }}
          />
        </div>
      </div>
      <div className="col-span-2">
        <span
          className={`inline-flex rounded-md border px-2 py-0.5 font-mono text-xs ${statusClass(
            session.status,
          )}`}
        >
          {session.status.toUpperCase()}
        </span>
      </div>
      <div className="col-span-2 text-right font-mono text-zinc-300">
        {formatCost(session.spent_usd)}
        <div className="text-xs text-zinc-600">left {formatCost(remaining)}</div>
      </div>
      <div className="col-span-2 text-right font-mono text-zinc-300">
        {session.decisions_count}
      </div>
      <div className="col-span-2 text-right">
        <button
          className="rounded-md border border-rose-500/40 bg-rose-500/10 px-3 py-1.5 text-xs font-semibold text-rose-200 transition-colors hover:bg-rose-500/20 disabled:cursor-not-allowed disabled:border-zinc-700 disabled:bg-zinc-900 disabled:text-zinc-600"
          disabled={!active || pending}
          onClick={() => onPause(session.id)}
          type="button"
        >
          {pending ? "Pausing..." : active ? "Pause" : "Paused"}
        </button>
      </div>
    </div>
  );
}

export default function KillSwitchPage() {
  const [sessions, setSessions] = useState<SessionControlSummary[]>([]);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pendingId, setPendingId] = useState<string | null>(null);

  async function load() {
    try {
      const res = await fetch("/api/sessions", { cache: "no-store" });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = (await res.json()) as SessionsResponse;
      setSessions(data.sessions);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "fetch failed");
    } finally {
      setLoaded(true);
    }
  }

  useEffect(() => {
    load();
    const id = setInterval(load, 1000);
    return () => clearInterval(id);
  }, []);

  async function pause(id: string) {
    setPendingId(id);
    try {
      const res = await fetch(`/api/sessions/${encodeURIComponent(id)}/pause`, {
        method: "POST",
        cache: "no-store",
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "pause failed");
    } finally {
      setPendingId(null);
    }
  }

  const activeSessions = useMemo(
    () => sessions.filter((session) => session.status === "active"),
    [sessions],
  );
  const pausedSessions = sessions.length - activeSessions.length;
  const exposedBudget = activeSessions.reduce(
    (sum, session) => sum + Math.max(0, session.budget_usd - session.spent_usd),
    0,
  );

  return (
    <main className="mx-auto min-h-screen max-w-6xl p-8">
      <header className="mb-8 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <a className="text-sm text-zinc-500 hover:text-zinc-300" href="/">
            Back to live feed
          </a>
          <h1 className="mt-4 text-3xl font-bold tracking-tight">Kill switch</h1>
          <p className="mt-1 text-sm text-zinc-400">
            Pause runaway sessions before another x402 payment is signed.
          </p>
        </div>
        <button
          className="rounded-md border border-zinc-700 px-3 py-2 text-sm text-zinc-300 hover:bg-zinc-900"
          onClick={load}
          type="button"
        >
          Refresh
        </button>
      </header>

      {error && (
        <div className="mb-6 rounded-lg border border-rose-500/30 bg-rose-500/10 p-4 text-sm text-rose-300">
          Kill-switch API error: {error}
        </div>
      )}

      <div className="mb-8 grid grid-cols-1 gap-4 sm:grid-cols-4">
        <Stat label="Active sessions" value={`${activeSessions.length}`} />
        <Stat label="Paused" value={`${pausedSessions}`} tone={pausedSessions > 0 ? "warning" : "neutral"} />
        <Stat label="Total sessions" value={`${sessions.length}`} />
        <Stat label="Exposed budget" value={formatCost(exposedBudget)} tone="warning" />
      </div>

      <section className="rounded-lg border border-zinc-800 bg-zinc-900/30">
        <header className="grid grid-cols-12 gap-3 border-b border-zinc-800 px-4 py-3 text-xs uppercase tracking-wider text-zinc-500">
          <span className="col-span-4">Session</span>
          <span className="col-span-2">Status</span>
          <span className="col-span-2 text-right">Spent</span>
          <span className="col-span-2 text-right">Decisions</span>
          <span className="col-span-2 text-right">Action</span>
        </header>
        {!loaded && (
          <div className="px-4 py-8 text-center text-sm text-zinc-500">
            Loading sessions from proxy...
          </div>
        )}
        {loaded && sessions.length === 0 && !error && (
          <div className="px-4 py-8 text-center text-sm text-zinc-500">
            No sessions yet. Send traffic through the proxy before using the kill switch.
          </div>
        )}
        {sessions.map((session) => (
          <SessionRow
            key={session.id}
            onPause={pause}
            pending={pendingId === session.id}
            session={session}
          />
        ))}
      </section>
    </main>
  );
}

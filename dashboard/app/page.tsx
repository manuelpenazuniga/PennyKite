export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-8">
      <div className="text-center space-y-4">
        <h1 className="text-4xl font-bold tracking-tight">PennyKite</h1>
        <p className="text-zinc-400 max-w-md">
          Pre-execution budget guardian for the agentic economy.
        </p>
        <div className="flex gap-4 justify-center mt-8">
          <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-6 w-48">
            <p className="text-sm text-zinc-500">Session Budget</p>
            <p className="text-2xl font-mono">$5.00</p>
          </div>
          <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-6 w-48">
            <p className="text-sm text-zinc-500">Spent</p>
            <p className="text-2xl font-mono text-emerald-400">$0.00</p>
          </div>
          <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-6 w-48">
            <p className="text-sm text-zinc-500">Decisions</p>
            <p className="text-2xl font-mono">0</p>
          </div>
        </div>
      </div>
    </main>
  );
}

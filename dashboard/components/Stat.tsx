interface StatProps {
  label: string;
  value: string;
  hint?: string;
  tone?: "neutral" | "positive" | "warning" | "danger";
}

const toneClasses: Record<NonNullable<StatProps["tone"]>, string> = {
  neutral: "text-zinc-100",
  positive: "text-emerald-400",
  warning: "text-amber-400",
  danger: "text-rose-400",
};

export function Stat({ label, value, hint, tone = "neutral" }: StatProps) {
  return (
    <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-6">
      <p className="text-xs uppercase tracking-wider text-zinc-500">{label}</p>
      <p className={`mt-2 text-2xl font-mono ${toneClasses[tone]}`}>{value}</p>
      {hint && <p className="mt-1 text-xs text-zinc-500">{hint}</p>}
    </div>
  );
}

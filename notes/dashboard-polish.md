# Dashboard Polish Proposal (PK-D3-09)

Based on the current state of the dashboard (`dashboard/app/page.tsx`, `dashboard/components/EventRow.tsx`, `dashboard/components/Badge.tsx`), here is the concrete list of improvements needed for the demo:

## 1. Animate Row Entries
- **Current State**: New rows appear instantly. `EventRow.tsx` only has `hover:bg-zinc-900/50 transition-colors`.
- **Proposed Change**: Use `framer-motion` or standard Tailwind CSS animations (e.g. `animate-in fade-in slide-in-from-top-2 duration-300`) on the `EventRow` wrapper to create a smooth entry effect (< 300ms).

## 2. Strong Red on DENY
- **Current State**: Deny verdicts use `bg-rose-500/10 text-rose-400 border-rose-500/20`.
- **Proposed Change**: Make it pop out more during a fast-paced demo. Increase opacity (`bg-rose-500/20 border-rose-500/50`) or add a brief pulse animation to the badge when a `DENY` or `DENY_LOOP` verdict arrives.

## 3. Sticky Header
- **Current State**: The table header in `page.tsx` is static (`<header className="grid ...">`).
- **Proposed Change**: Add `sticky top-0 bg-zinc-900/95 backdrop-blur z-10` to the `section > header` element in `page.tsx` so column names remain visible when scrolling through a long list of decisions.

## 4. Real-time Wallet Balance Widget
- **Current State**: `page.tsx` only shows the session `summary` (Budget, Spent, Remaining) computed from local proxy state.
- **Proposed Change**: Introduce a new component (e.g., `WalletBalance`) that polls the actual on-chain wallet balance via RPC. Animate the number counting down to visually emphasize the "money draining" effect during the unprotected mode of the runaway agent script.

## 5. Live Feed Update Frequency
- **Current State**: The feed polls every 1000ms (`setInterval(load, 1000)`).
- **Proposed Change**: For a snappier demo feel, consider dropping the poll interval to 500ms or using SSE/WebSockets for immediate row population.

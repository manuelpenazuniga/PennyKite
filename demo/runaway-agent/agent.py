"""Runaway agent for the PennyKite demo.

Two modes:
  * unprotected: talks directly to the target API. Drains funds until the
    operator kills it.
  * protected:   talks through the PennyKite proxy. Stops once the proxy
    returns the PennyKite block reason.

Usage:
  python3 agent.py unprotected [--session SID] [--calls N] [--budget USD]
  python3 agent.py protected   [--session SID] [--calls N] [--budget USD]

Defaults are tuned for a 30-second demo against demo/target-api running on
localhost:4100 and the PennyKite proxy on localhost:8787.
"""

from __future__ import annotations

import argparse
import base64
import json
import sys
import time
from dataclasses import dataclass
from typing import Optional

import requests

# --- ANSI helpers -----------------------------------------------------------

RESET = "\033[0m"
DIM = "\033[2m"
BOLD = "\033[1m"
GREEN = "\033[32m"
RED = "\033[31m"
YELLOW = "\033[33m"
CYAN = "\033[36m"
MAGENTA = "\033[35m"


def colour(text: str, code: str) -> str:
    return f"{code}{text}{RESET}"


# --- domain types -----------------------------------------------------------


@dataclass
class CallResult:
    status: int
    cost_usd: float
    blocked: bool
    reason: str
    payload: Optional[dict]


@dataclass
class RunSummary:
    calls: int
    approved: int
    blocked: int
    spent_usd: float
    runaway_simulated_spend_usd: float


# --- core logic -------------------------------------------------------------


def parse_payment_required_header(value: str) -> dict:
    decoded = base64.b64decode(value).decode("utf-8")
    return json.loads(decoded)


def amount_to_usd(amount: str, decimals: int) -> float:
    return int(amount) / (10**decimals)


def call_unprotected(target_url: str, match_id: str) -> CallResult:
    """Direct hit on the target API. Returns 402 (no signature) — but the
    agent still 'pays' by attaching a mock signature, demonstrating that
    without PennyKite the runaway agent would spend money repeatedly."""
    # First call: get the quote so we can record the cost.
    quote = requests.get(f"{target_url}/predict/{match_id}", timeout=5)
    cost = 0.0
    if quote.status_code == 402:
        header = quote.headers.get("PAYMENT-REQUIRED")
        if header:
            decoded = parse_payment_required_header(header)
            req = decoded["requirements"][0]
            cost = amount_to_usd(req["amount"], req["decimals"])

    # Second call: simulate paying by attaching a mock signature.
    paid = requests.get(
        f"{target_url}/predict/{match_id}",
        headers={"PAYMENT-SIGNATURE": "0xdeadbeef-mock-signature"},
        timeout=5,
    )

    return CallResult(
        status=paid.status_code,
        cost_usd=cost,
        blocked=False,
        reason="",
        payload=paid.json() if paid.status_code == 200 else None,
    )


def call_protected(proxy_url: str, session_id: str, match_id: str) -> CallResult:
    """Goes through the PennyKite proxy. The proxy decides whether to forward
    or to deny with HTTP 402 (PennyKite-format reason body)."""
    response = requests.get(
        f"{proxy_url}/predict/{match_id}",
        headers={"X-Pennykite-Session": session_id},
        timeout=5,
    )

    if response.status_code == 200:
        body = response.json()
        return CallResult(
            status=200,
            cost_usd=float(body.get("_pk_cost_usd", 0.0)),
            blocked=False,
            reason="",
            payload=body,
        )

    if response.status_code == 402:
        try:
            body = response.json()
        except json.JSONDecodeError:
            body = {}
        verdict = body.get("verdict", "deny")
        reason = body.get("reason", "blocked")
        return CallResult(
            status=402,
            cost_usd=0.0,
            blocked=True,
            reason=f"{verdict}: {reason}",
            payload=body,
        )

    return CallResult(
        status=response.status_code,
        cost_usd=0.0,
        blocked=True,
        reason=f"unexpected status {response.status_code}",
        payload=None,
    )


# --- printers ---------------------------------------------------------------


def print_header(mode: str, session_id: str, budget: float, calls: int) -> None:
    print()
    print(colour(f"━━ PennyKite demo — {mode.upper()} mode ━━", BOLD + CYAN))
    print(f"  session : {colour(session_id, MAGENTA)}")
    print(f"  budget  : ${budget:.2f}")
    print(f"  calls   : up to {calls}")
    print()


def print_call(i: int, result: CallResult, running_spend: float) -> None:
    icon = (
        colour("✓", GREEN)
        if not result.blocked and result.status == 200
        else colour("✗", RED) if result.blocked else colour("?", YELLOW)
    )
    cost_str = colour(f"${result.cost_usd:.4f}", DIM)
    running_str = colour(f"total=${running_spend:.4f}", DIM)
    suffix = f"  {colour(result.reason, RED)}" if result.reason else ""
    print(f"  {icon} call #{i:02d}  cost={cost_str}  {running_str}{suffix}")


def print_summary(summary: RunSummary, mode: str) -> None:
    print()
    print(colour("━━ Summary ━━", BOLD))
    print(f"  calls attempted   : {summary.calls}")
    print(f"  approved          : {colour(str(summary.approved), GREEN)}")
    print(f"  blocked           : {colour(str(summary.blocked), RED)}")
    print(f"  actual spend      : {colour(f'${summary.spent_usd:.4f}', BOLD)}")
    if mode == "protected":
        saved = summary.runaway_simulated_spend_usd - summary.spent_usd
        print(
            f"  runaway would have spent : "
            f"{colour(f'${summary.runaway_simulated_spend_usd:.4f}', YELLOW)}"
        )
        print(
            f"  saved by PennyKite       : "
            f"{colour(f'${saved:.4f}', GREEN + BOLD)}"
        )
    print()


# --- runners ----------------------------------------------------------------


def run_unprotected(
    target_url: str, session_id: str, max_calls: int, budget: float
) -> RunSummary:
    print_header("unprotected", session_id, budget, max_calls)
    spent = 0.0
    approved = 0
    for i in range(1, max_calls + 1):
        result = call_unprotected(target_url, f"match-{i}")
        if result.status == 200:
            spent += result.cost_usd
            approved += 1
        print_call(i, result, spent)
        time.sleep(0.2)

    print()
    print(colour("⚠  No guardian in place. Agent kept paying.", YELLOW + BOLD))
    return RunSummary(
        calls=max_calls,
        approved=approved,
        blocked=0,
        spent_usd=spent,
        runaway_simulated_spend_usd=spent,
    )


def run_protected(
    proxy_url: str, session_id: str, max_calls: int, budget: float
) -> RunSummary:
    print_header("protected", session_id, budget, max_calls)
    spent = 0.0
    approved = 0
    blocked = 0
    simulated_spend = 0.0
    for i in range(1, max_calls + 1):
        result = call_protected(proxy_url, session_id, f"match-{i}")
        simulated_spend += 0.05  # what the agent would have paid unprotected
        if result.blocked:
            blocked += 1
            print_call(i, result, spent)
            print()
            print(colour("✋ BLOCKED by PennyKite. Halting.", RED + BOLD))
            break
        spent += result.cost_usd
        approved += 1
        print_call(i, result, spent)
        time.sleep(0.2)
    else:
        print()
        print(
            colour(
                "✓ Completed without trips. Either budget was generous or the agent behaved.",
                GREEN,
            )
        )

    return RunSummary(
        calls=approved + blocked,
        approved=approved,
        blocked=blocked,
        spent_usd=spent,
        runaway_simulated_spend_usd=simulated_spend,
    )


# --- CLI --------------------------------------------------------------------


def main(argv: Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["unprotected", "protected"])
    parser.add_argument(
        "--target",
        default="http://localhost:4100",
        help="Demo target API URL (unprotected mode)",
    )
    parser.add_argument(
        "--proxy",
        default="http://localhost:8787",
        help="PennyKite proxy URL (protected mode)",
    )
    parser.add_argument("--session", default="demo-session-001")
    parser.add_argument("--calls", type=int, default=20)
    parser.add_argument("--budget", type=float, default=5.0)
    args = parser.parse_args(argv)

    try:
        if args.mode == "unprotected":
            summary = run_unprotected(args.target, args.session, args.calls, args.budget)
        else:
            summary = run_protected(args.proxy, args.session, args.calls, args.budget)
    except requests.RequestException as e:
        print(colour(f"ERROR: cannot reach upstream — {e}", RED + BOLD))
        return 2

    print_summary(summary, args.mode)
    return 0


if __name__ == "__main__":
    sys.exit(main())

# Runaway agent

A tiny Python script that simulates a misbehaving AI agent in two modes:

| Mode | Talks to | Outcome |
|---|---|---|
| `unprotected` | demo target API directly | Drains funds — keeps paying until you stop it |
| `protected` | PennyKite proxy | Halts as soon as the proxy returns a deny verdict |

This is the dramatic before/after that anchors the hackathon demo.

## Setup

```bash
cd demo/runaway-agent
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

## Quick run (unprotected)

In one terminal, start the target API:

```bash
cd demo/target-api && npm install && npm start
```

In another:

```bash
python3 agent.py unprotected --calls 10
```

You will see the agent paying for every call. Each `✓` is a successful
payment of $0.05; the running total grows unbounded.

## Quick run (protected)

You also need the PennyKite proxy running (see `crates/pennykite-proxy`):

```bash
cargo run -p pennykite-proxy -- \
  --listen 127.0.0.1:8787 \
  --policy policy/examples/conservative.yaml \
  --upstream http://localhost:4100
```

Then:

```bash
python3 agent.py protected --session demo-session-001 --calls 20
```

Expected output:

- The first few calls succeed (approve verdicts).
- Around call #4 the loop detector fires; the proxy returns 402 with
  `{"verdict": "deny_loop", "reason": "..."}` and the agent halts.
- The summary shows the saved amount compared to a runaway baseline.

## CLI flags

| Flag | Default | Description |
|---|---|---|
| `--target` | `http://localhost:4100` | Target API URL (unprotected) |
| `--proxy` | `http://localhost:8787` | PennyKite proxy URL (protected) |
| `--session` | `demo-session-001` | Session identifier |
| `--calls` | `20` | Maximum number of calls |
| `--budget` | `5.0` | Display-only budget hint (the real budget lives in the proxy policy) |
| `--match-id` | unique `match-N` per call | Fixed match id for every call; used by `demo/run-demo.sh` to trigger loop detection |

## Notes

- `protected` mode requires the proxy handler from PK-D2-08 to be wired.
  Until then the script will surface raw HTTP errors — useful for debugging
  but not the polished demo.
- By default the agent uses a deterministic `match-N` pattern. Pass
  `--match-id <id>` to repeat the same upstream path and trigger the proxy's
  exact-match loop detector during the demo.

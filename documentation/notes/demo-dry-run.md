# Demo dry-run notes

Date: 2026-05-17
Task: PK-D3-10

## Private recording artifacts

Private dry-run transcripts were saved outside git under:

- `docs/demo-dry-run/take-1.log`
- `docs/demo-dry-run/take-2.log`

`docs/` is gitignored, so these artifacts are intentionally not committed. They are terminal recordings of the full local demo flow. A polished UI screen recording should still be captured during the PK-D4-06 submission-video task.

## Commands run

```bash
PENNYKITE_DEMO_AUTO=1 ./demo/run-demo.sh > docs/demo-dry-run/take-1.log 2>&1
PENNYKITE_DEMO_AUTO=1 ./demo/run-demo.sh > docs/demo-dry-run/take-2.log 2>&1
```

Both takes exited with status 0.

## Take results

| Take | Unprotected | Protected | Result |
|---|---:|---:|---|
| 1 | 8 approved, $0.4000 spent | 3 approved, blocked on call 4, $0.1500 spent | `deny_loop: loop detected` |
| 2 | 8 approved, $0.4000 spent | 3 approved, blocked on call 4, $0.1500 spent | `deny_loop: loop detected` |

Post-run cleanup left no listeners on ports 4100, 8787, or 3000.

## Loop detector calibration

Current conservative preset:

```yaml
loop_detection:
  window_size: 10
  similarity_threshold: 0.85
  max_consecutive_similar: 3
```

Observed behavior is correct for the demo: with fixed `--match-id`, the first three identical requests pass and the fourth request is denied before upstream spend. No policy calibration change is needed for PK-D3-10.

## Weak spots found

| Priority | Issue | Impact | Suggested owner/task |
|---|---|---|---|
| P1 | Protected summary under-reports savings. It reports `$0.0500` saved because it compares against four attempted calls, not the full planned runaway horizon. | Weak video moment; the protection works, but the numeric payoff looks small. | PK-D4-01: compare protected spend against `max_calls * price` or print an additional "projected runaway spend" metric. |
| P1 | Auto mode cleans up services immediately after printing URLs. | The session detail and dashboard URLs are gone by the time a reviewer opens them from a transcript. | PK-D4-01 / video workflow: run interactively for capture, or add a `PENNYKITE_DEMO_HOLD=1` option. |
| P2 | Running the script inside the command sandbox can produce a false `Port 4100 is already in use` because local port binding is blocked. | Agent-only friction; normal terminal execution works. | Documented here. For future agent runs, execute the script outside the sandbox. |
| P2 | UI was not browser-verified during this task. | Terminal smoke proves backend and script behavior, but final Devpost video still needs visual review of feed, balance widget, session page, and kill-switch. | PK-D4-06: capture browser-based recording. |
| P3 | Kite attestation/revocation remains `mock_pending`. | Demo narration must be explicit that local kill-switch is enforced in SQLite today while on-chain revocation waits on Kite testnet deploy/API. | PK-D1-07 / PK-D2-09 / PK-D2-13. |

## Decision

PK-D3-10 dry-run is complete enough to unblock PK-D4-01. The demo runs twice in a row without runtime errors, loop calibration is stable, and the remaining work is targeted polish rather than core functionality.

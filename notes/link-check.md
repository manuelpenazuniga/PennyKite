# Link and Path Check — 2026-05-16

Scope checked: `README.md`, `documentation/demo-runbook.md`, `notes/backlog-audit.md`, `CHANGELOG.md`.

---

## Paths verified to exist on disk

| Path | Status |
|---|---|
| `policy/examples/conservative.yaml` | ✅ |
| `policy/examples/trading-aggressive.yaml` | ✅ |
| `policy/examples/consumer.yaml` | ✅ |
| `demo/target-api/` | ✅ (`server.js`, `package.json`) |
| `dashboard/` | ✅ (Next.js app with all routes) |
| `crates/pennykite-proxy/` | ✅ (`src/handler.rs`, `src/main.rs`) |
| `documentation/demo-runbook.md` | ✅ (created in TASK 2) |

## API endpoints verified to be implemented

| Endpoint | Implementation | Status |
|---|---|---|
| `POST /api/sessions/{id}/pause` | `dashboard/app/api/sessions/[id]/pause/route.ts` + proxy handler | ✅ |
| `GET /api/sessions` | `dashboard/app/api/sessions/route.ts` | ✅ |
| `GET /api/sessions/{id}` | `dashboard/app/api/sessions/[id]/route.ts` | ✅ |
| `GET /api/decisions` | `dashboard/app/api/decisions/route.ts` | ✅ |
| `/kill-switch` (page) | `dashboard/app/kill-switch/page.tsx` | ✅ |
| `/sessions/[id]` (page) | `dashboard/app/sessions/[id]/page.tsx` | ✅ |

---

## References to planned (not yet existing) files in original README

These references are in the pre-existing README and describe planned v0.1.0 content, not content added in recent doc commits.

| Reference | Line | Status | Backlog entry |
|---|---|---|---|
| `./demo/run-demo.sh` | 88, 215, 238 | ✅ Script implemented | PK-D3-07 |
| `.env.example` | 237 | ✅ Placeholder template exists | PK-CI-04 (pending real deployed addresses) |

**Impact of `.env.example` placeholders:** The README Quickstart can now copy `.env.example`, but real deployed addresses are still pending `PK-D1-07`.

**Impact of `run-demo.sh`:** The README one-command demo reference now resolves to `demo/run-demo.sh`. The manual runbook (`documentation/demo-runbook.md`) remains the step-by-step fallback.

---

## Conclusion

TASK 1-4 docs were rechecked after correcting demo-runbook expected outputs. Inaccuracies fixed: Node version floor (22->20), target API startup log, and pause response body. `.env.example` has since been created (DX blocker resolved for local dev, real deployed addresses pending). Architecture docs at `documentation/architecture.md`, `documentation/kite-integration.md`, and `documentation/x402-flow.md` have been created. `demo/run-demo.sh` now exists and covers the local demo flow.

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
| `./demo/run-demo.sh` | 88, 215, 238 | ❌ Script does not exist | PK-D3-07 (pending) |
| `.env.example` | 237 | ❌ File does not exist | PK-CI-04 (pending) |

**Impact of `.env.example` absence:** The README Quickstart (`cp .env.example .env`) fails silently on a fresh clone. This is a DX blocker. Creating `.env.example` is low effort (PK-CI-04 estimated 0.5h).

**Impact of `run-demo.sh` absence:** The "One-command demo" narrative in the README and the demo scenario description both reference the script. The manual runbook (`documentation/demo-runbook.md`) serves as a documented alternative until PK-D3-07 is completed.

---

## Conclusion

No errors found in the content added by recent doc commits (TASK 1–3). Two pre-existing aspirational references are documented above. No changes were made to correct them — they are correctly tracked as pending in `backlog.yaml`.

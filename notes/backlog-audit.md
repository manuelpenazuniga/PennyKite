# Backlog Consistency Audit

Initial audit date: 2026-05-16  
Last updated: 2026-05-17 (post demo script + Python SDK work)
Purpose: Verify that backlog statuses match the actual state of the repo.

---

## PK-D3-09 — Dashboard polish for demo

| Field | Value |
|---|---|
| Backlog status | `done` |
| Artifacts expected | `dashboard/components/**`, `dashboard/app/page.tsx` |
| Artifacts found | ✅ `Badge.tsx`, `EventRow.tsx`, `Stat.tsx` in `dashboard/components/`; `dashboard/app/page.tsx` present |
| Build check | `npm run build` passes — all 4 routes compiled |
| Risk | **LOW** |
| Recommendation | Status confirmed correct. No action needed. |

---

## PK-D3-07 — One-command demo script

| Field | Value |
|---|---|
| Backlog status | `done` |
| Artifacts expected | `demo/run-demo.sh` |
| Artifacts found | ✅ `demo/run-demo.sh` exists and is executable |
| Verification | `PENNYKITE_DEMO_AUTO=1 ./demo/run-demo.sh` passed; protected run blocks with `deny_loop` |
| Downstream blockers | PK-D4-05 remains downstream of PK-D3-10; PK-D4-01 is complete |
| Risk | **LOW** |
| Recommendation | Status confirmed correct. No action needed. |

---

## PK-D3-08 — Python SDK convenience wrapper

| Field | Value |
|---|---|
| Backlog status | `done` |
| Artifacts expected | `sdk/python/pennykite/**`, `sdk/python/pyproject.toml` |
| Artifacts found | ✅ Local Python package exists under `sdk/python/` |
| README fix applied | ✅ README now states Python SDK exists locally; TypeScript SDK remains planned |
| Verification | `pip install -e sdk/python` and `python -m unittest discover -s sdk/python/tests` pass |
| Risk | **LOW** |
| Recommendation | Status confirmed correct. No action needed. |

---

## PK-D3-10 — Demo dry-run + screen recording

| Field | Value |
|---|---|
| Backlog status | `done` |
| Artifacts expected | `documentation/notes/demo-dry-run.md` (private recording + notes) |
| Artifacts found | ✅ `documentation/notes/demo-dry-run.md` committed; private transcripts saved under gitignored `docs/demo-dry-run/` |
| Verification | Two `PENNYKITE_DEMO_AUTO=1 ./demo/run-demo.sh` takes passed; protected path denied with `deny_loop` on call 4 |
| Risk | **CRITICAL** |
| Recommendation | Use PK-D4-01 for the targeted polish fixes found during the dry-run. |

---

## PK-D4-02 — Hosted proxy

| Field | Value |
|---|---|
| Backlog status | `pending` |
| Artifacts expected | `crates/pennykite-proxy/Dockerfile`, `fly.toml or railway.json` |
| Artifacts found | ✅ `crates/pennykite-proxy/Dockerfile`, `.dockerignore`, and `fly.toml` exist |
| Remaining blockers | Public deploy URL, Fly/Railway secrets, persistent volume, README URL update, and local-agent smoke against public proxy |
| Risk | **MEDIUM** |
| Recommendation | Deploy when account/secrets are available; do not mark done until `/health` passes on public HTTPS. |

---

## PK-D4-04 — Final README pass with screenshots

| Field | Value |
|---|---|
| Backlog status | `pending` |
| Artifacts expected | `README.md`, `documentation/img/**` |
| Artifacts found | ✅ `README.md` actively maintained; ❌ `documentation/img/` does not exist; no screenshots committed |
| Upstream blockers | PK-D4-02 (hosted proxy) and PK-D4-03 (hosted dashboard) are both pending |
| Risk | **MEDIUM** |
| Recommendation | README content is solid. Screenshots and hosted-demo links cannot be added until deployment tasks complete; keep pending. |

---

## PK-D4-07 — Public docs: architecture, kite-integration, x402-flow

| Field | Value |
|---|---|
| Backlog status | `pending` |
| Artifacts expected | `documentation/architecture.md` (≥600 words), `documentation/kite-integration.md` (≥400 words), `documentation/x402-flow.md` (≥400 words with Mermaid) |
| Artifacts found | ✅ All three files now exist: `architecture.md` (1153w), `kite-integration.md` (797w), `x402-flow.md` (886w, Mermaid sequenceDiagram included) |
| Upstream blockers | None; PK-D4-01 is complete |
| Risk | **LOW** (drafts meet word count and Mermaid requirements) |
| Recommendation | Status confirmed correct. No action needed. |

---

## PK-CI-04 — .env.example

| Field | Value |
|---|---|
| Backlog status | `pending` |
| Artifacts expected | `.env.example` with all required keys |
| Artifacts found | ✅ `.env.example` now exists with all required keys plus `PENNYKITE_PROXY_URL`, `DATABASE_PATH`, `USDC_CONTRACT` |
| Remaining blocker | Real deployed addresses (`PENNYKITE_ATTESTOR_ADDRESS`, `PAY_TO`, `FACILITATOR_URL`) are placeholders — pending `PK-D1-07` |
| Risk | **LOW** (DX blocker resolved for local dev; deployment addresses remain TBD) |
| Recommendation | Status remains `pending` per backlog dependency on PK-D1-07. Template is usable for local development now. |

---

## PK-CI-06 — CHANGELOG.md

| Field | Value |
|---|---|
| Backlog status | `done` |
| Artifacts expected | `/CHANGELOG.md` in Keep a Changelog format |
| Artifacts found | ✅ `CHANGELOG.md` exists with `[Unreleased]` and `[0.1.0]` sections, actively maintained |
| Risk | **LOW** |
| Recommendation | Status confirmed correct. |

---

## PK-CI-07 — CONTRIBUTING.md

| Field | Value |
|---|---|
| Backlog status | `done` |
| Artifacts expected | `/CONTRIBUTING.md` |
| Artifacts found | ✅ `CONTRIBUTING.md` exists at repo root |
| Risk | **LOW** |
| Recommendation | Status confirmed correct. |

---

## PK-CI-08 — PR template

| Field | Value |
|---|---|
| Backlog status | `done` |
| Artifacts expected | `.github/PULL_REQUEST_TEMPLATE.md` |
| Artifacts found | ✅ `.github/PULL_REQUEST_TEMPLATE.md` exists |
| Risk | **LOW** |
| Recommendation | Status confirmed correct. |

---

## Risk summary (updated)

| Risk | Entry | Issue |
|---|---|---|
| MEDIUM | PK-D4-02 | Deploy artifacts are prepared, but public proxy URL and smoke test need Fly/Railway credentials/secrets |
| MEDIUM | PK-D4-04 | Screenshots and hosted links not yet possible (deployment pending) |
| LOW | PK-CI-04 | `.env.example` template committed; deployed addresses pending PK-D1-07 |
| LOW | PK-D3-07, PK-D3-08, PK-D3-09, PK-D3-10, PK-D4-01, PK-D4-07, PK-CI-06, PK-CI-07, PK-CI-08 | All confirmed done and correct |

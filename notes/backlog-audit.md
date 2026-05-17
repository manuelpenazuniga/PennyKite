# Backlog Consistency Audit

Date: 2026-05-16  
Purpose: Verify that backlog statuses match the actual state of the repo. No statuses were changed.

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
| Backlog status | `pending` |
| Artifacts expected | `demo/run-demo.sh` |
| Artifacts found | ❌ `demo/` contains only `runaway-agent/` and `target-api/` — `run-demo.sh` does not exist |
| Downstream blockers | PK-D3-08, PK-D3-10 depend on this; PK-D4-01 and PK-D4-05 depend on PK-D3-10 |
| Risk | **CRITICAL** |
| Recommendation | This is the highest-priority implementation gap for demo readiness. The manual runbook at `documentation/demo-runbook.md` is a stopgap only. |

---

## PK-D3-08 — Python SDK convenience wrapper

| Field | Value |
|---|---|
| Backlog status | `pending` |
| Artifacts expected | `sdk/python/pennykite/**`, `sdk/python/pyproject.toml` |
| Artifacts found | ❌ `sdk/` directory does not exist |
| Additional risk | `README.md` SDK section shows working Python code (`from pennykite import PennyKiteClient`) which is not importable |
| Risk | **HIGH** |
| Recommendation | README SDK section is aspirational, not implemented. Flag this discrepancy before submission. |

---

## PK-D3-10 — Demo dry-run + screen recording

| Field | Value |
|---|---|
| Backlog status | `pending` |
| Artifacts expected | `documentation/notes/demo-dry-run.md` (private recording + notes) |
| Artifacts found | ❌ `documentation/notes/` does not exist; artifact path uses a nested `notes/` inside `documentation/` |
| Blocker | PK-D3-07 is pending — dry-run cannot proceed without the demo script |
| Risk | **CRITICAL** |
| Recommendation | Blocked by PK-D3-07. Cannot unblock without `demo/run-demo.sh` or equivalent. |

---

## PK-D4-04 — Final README pass with screenshots

| Field | Value |
|---|---|
| Backlog status | `pending` |
| Artifacts expected | `README.md`, `documentation/img/**` |
| Artifacts found | ✅ `README.md` exists and is actively maintained; ❌ `documentation/img/` does not exist; no screenshots committed |
| Upstream blockers | PK-D4-02 (hosted proxy) and PK-D4-03 (hosted dashboard) are both pending |
| Risk | **MEDIUM** |
| Recommendation | README content is solid. Screenshots and hosted-demo links cannot be added until deployment tasks complete. |

---

## PK-D4-07 — Public docs: architecture, kite-integration, x402-flow

| Field | Value |
|---|---|
| Backlog status | `pending` |
| Artifacts expected | `documentation/architecture.md` (≥600 words), `documentation/kite-integration.md` (≥400 words), `documentation/x402-flow.md` (≥400 words with Mermaid) |
| Artifacts found | ❌ None of these files exist — `documentation/` only contains `demo-runbook.md` (created today) |
| Upstream blockers | PK-D4-01 (bug fixes from dry-run) is pending |
| Risk | **HIGH** |
| Recommendation | No progress made. Significant writing effort required (1400+ words + diagrams). Blocked by dry-run. |

---

## PK-CI-04 — .env.example

| Field | Value |
|---|---|
| Backlog status | `pending` |
| Artifacts expected | `.env.example` with all required keys |
| Artifacts found | ❌ `.env.example` does not exist |
| Additional risk | `README.md` Quickstart says `cp .env.example .env` — this command fails on a fresh clone |
| Risk | **HIGH** |
| Recommendation | DX blocker. Any contributor following the README quickstart will hit this immediately. Should be created before submission. Required keys per backlog: `KITE_RPC_URL`, `KITE_CHAIN_ID`, `KITE_PRIVATE_KEY`, `SESSION_KEY`, `PENNYKITE_ATTESTOR_ADDRESS`, `PAY_TO`, `FACILITATOR_URL`. |

---

## PK-CI-06 — CHANGELOG.md

| Field | Value |
|---|---|
| Backlog status | `done` |
| Artifacts expected | `/CHANGELOG.md` in Keep a Changelog format |
| Artifacts found | ✅ `CHANGELOG.md` exists with `[Unreleased]` and `[0.1.0]` sections, maintained |
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

## Risk summary

| Risk | Entry | Issue |
|---|---|---|
| CRITICAL | PK-D3-07 | `demo/run-demo.sh` missing — blocks dry-run and final recording |
| CRITICAL | PK-D3-10 | Blocked by PK-D3-07; no dry-run notes |
| HIGH | PK-D3-08 | Python SDK not implemented; README implies it works |
| HIGH | PK-D4-07 | Three architecture docs not started |
| HIGH | PK-CI-04 | `.env.example` missing; README quickstart broken on fresh clone |
| MEDIUM | PK-D4-04 | Screenshots and hosted links not yet possible (deployment pending) |
| LOW | PK-D3-09, PK-CI-06, PK-CI-07, PK-CI-08 | All confirmed done and correct |

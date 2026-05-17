# PennyKite — Local Smoke Runbook

Step-by-step guide for running and recording the full control-plane demo locally.  
This is a manual runbook. For a one-command script see `PK-D3-07` (pending).

---

## Prerequisites

```bash
node --version   # >= 20
rustc --version  # >= 1.80
```

Open **three terminal tabs** (proxy, target API, dashboard) and keep them side by side.

---

## Step 1 — Start the target API

```bash
cd demo/target-api && npm run start
```

Expected output:
```
[demo-target-api] listening on :4100 (price=$0.0500 USDC @ eip155:84532)
```

---

## Step 2 — Start the proxy

```bash
cargo run -p pennykite-proxy -- \
  --listen 127.0.0.1:8787 \
  --policy policy/examples/conservative.yaml \
  --upstream http://127.0.0.1:4100 \
  --db /tmp/pennykite-demo.db
```

Expected output (last line):
```
INFO pennykite_proxy: listening on 127.0.0.1:8787
```

---

## Step 3 — Start the dashboard

```bash
cd dashboard && PENNYKITE_PROXY_URL=http://127.0.0.1:8787 npm run dev
```

Open [http://localhost:3000](http://localhost:3000) — you should see an empty live feed.

---

## Step 4 — Generate approved decisions

Send a few distinct requests to build up the live feed:

```bash
curl -s -i -H 'x-pennykite-session: demo-session' http://127.0.0.1:8787/proxy/predict/demo-1
curl -s -i -H 'x-pennykite-session: demo-session' http://127.0.0.1:8787/proxy/predict/demo-2
```

Expected: `HTTP/1.1 200 OK` on each.  
The live feed at `http://localhost:3000` should show two green APPROVE rows.

---

## Step 5 — Trigger a loop deny (DENY_LOOP)

The loop detector fires when `max_consecutive_similar` (3) identical fingerprints are seen; the **4th** request is blocked.

```bash
for i in 1 2 3 4; do
  echo "--- Request $i ---"
  curl -s -o - -w "\nHTTP %{http_code}\n" \
    -H 'x-pennykite-session: loop-session' \
    http://127.0.0.1:8787/proxy/predict/loop-target
  sleep 0.15
done
```

Expected output on request 4:
```
{"verdict":"deny_loop","reason":"loop detected"}
HTTP 402
```

The live feed should show a red DENY_LOOP row for `loop-session`.

---

## Step 6 — Inspect session detail

Open [http://localhost:3000/sessions/demo-session](http://localhost:3000/sessions/demo-session).  
You should see the two APPROVE decisions with timestamps and amounts.

---

## Step 7 — Open the kill-switch page

Open [http://localhost:3000/kill-switch](http://localhost:3000/kill-switch).  
Both `demo-session` and `loop-session` should appear with status `active`.

---

## Step 8 — Pause a session

Either click **Pause** in the dashboard UI, or run:

```bash
curl -i -X POST http://127.0.0.1:8787/api/sessions/demo-session/pause
```

Expected response:
```
HTTP/1.1 202 Accepted
{"status":"accepted","session_id":"demo-session","kite_revocation":"mock_pending"}
```

The kill-switch page should refresh and show `demo-session` as paused.

---

## Step 9 — Verify the session is blocked

```bash
curl -i -H 'x-pennykite-session: demo-session' \
  http://127.0.0.1:8787/proxy/predict/demo-after-pause
```

Expected:
```
HTTP/1.1 402 Payment Required
{"reason":"session is paused"}
```

---

## Cleanup

| Resource | How to release |
|---|---|
| Target API (:4100) | `Ctrl+C` in terminal tab 1 |
| Proxy (:8787) | `Ctrl+C` in terminal tab 2 |
| Dashboard (:3000) | `Ctrl+C` in terminal tab 3 |
| Temp database | `rm /tmp/pennykite-demo.db` (optional — it auto-creates fresh on next run) |

---

## Troubleshooting

| Symptom | Fix |
|---|---|
| Dashboard shows no data | Verify `PENNYKITE_PROXY_URL=http://127.0.0.1:8787` is set; the Next.js bridge calls the proxy, not SQLite |
| Loop detector fires on request 2 or 3 | Increase `similarity_threshold` in the policy, or slow down the loop (`sleep 0.5`) |
| Port conflicts | Change `--listen` / `--port` and update `PENNYKITE_PROXY_URL` accordingly |
| `cargo run` slow first time | Rust compiles the full workspace; subsequent runs use the incremental cache |

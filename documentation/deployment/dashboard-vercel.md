# Dashboard Vercel deployment

PK-D4-03 tracks the production dashboard deployment. The dashboard is a Next.js app under `dashboard/` and reads live data through its server-side API bridge. Browser code calls `/api/*`; those Next routes forward to the PennyKite proxy configured by `PENNYKITE_PROXY_URL`.

## Project settings

| Setting | Value |
|---|---|
| Framework | Next.js |
| Root directory | `dashboard` |
| Install command | `npm install` |
| Build command | `npm run build` |
| Output directory | `.next` |

`dashboard/vercel.json` pins the install/build/output commands for repeatable deploys.

## Required environment variable

| Variable | Production value |
|---|---|
| `PENNYKITE_PROXY_URL` | Public HTTPS URL from PK-D4-02, for example `https://pennykite-proxy-demo.fly.dev` |

Do not deploy production with the default `http://127.0.0.1:8787`; that value is only for local development.

## CLI flow

```bash
cd dashboard
vercel link
vercel env add PENNYKITE_PROXY_URL production
vercel deploy --prod
```

After deploy, verify:

```bash
curl -fsS https://<vercel-prod>/
curl -fsS https://<vercel-prod>/api/decisions
```

`/api/decisions` should return JSON from the public PennyKite proxy. A `502 proxy_unreachable` response usually means `PENNYKITE_PROXY_URL` is missing, points at localhost, or the public proxy is not healthy.

## Current status

The app builds locally. The production deploy remains pending until PK-D4-02 provides a public proxy URL and Vercel project credentials are available.

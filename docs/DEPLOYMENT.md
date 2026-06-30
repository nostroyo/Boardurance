# Deployment

100% free, **no credit card required**. Everything runs on **Render** (backend + frontend) plus
your existing **OVH MongoDB**.

## Environments

| Env | Trigger | Backend | Frontend | Database | Gate |
|-----|---------|---------|----------|----------|------|
| **test** | every PR + push to `main`/`dev` | CI only — no deploy | CI only (vitest) | **mock** (degraded mode, `APP_ENVIRONMENT=test`) | — |
| **preprod** | push to `dev` | Render web `boardurance-api-preprod` | Render static `boardurance-web-preprod` | OVH cluster, `boardurance_preprod` DB | none (auto) |
| **prod** | push to `main` | Render web `boardurance-api` | Render static `boardurance-web` | OVH cluster, `boardurance_prod` DB | **manual approval** |

Preprod is your **manual verification checkpoint**: `dev` deploys automatically with no gate so
you can test against real OVH data, then promoting `dev` → `main` pauses on a one-click approval
before prod is touched.

```
feature/* ─PR─► [test]    CI only: fmt, clippy, test-fast (MOCK Mongo) · tsc, vitest · no deploy
                            │ merge
dev ─────push─► [preprod] backend → Render web (preprod) ─┐ OVH cluster, DB: boardurance_preprod
                            (auto, no gate)                 └ frontend → Render static (preprod)
                            │ PR dev→main
main ────push─► [prod]     ⏸ approval ─► backend → Render web (prod) ─┐ OVH cluster, DB: boardurance_prod
                                                                       └ frontend → Render static (prod)
```

CI/CD via GitHub Actions:

- **CI** ([backend-ci.yml](../.github/workflows/backend-ci.yml), [frontend-ci.yml](../.github/workflows/frontend-ci.yml)) runs on every PR and push to `main`/`dev`: format, lint, type-check, tests, build. No DB, no secrets.
- **Deploy preprod** ([deploy-preprod.yml](../.github/workflows/deploy-preprod.yml)) runs on push to `dev`.
- **Deploy prod** ([deploy.yml](../.github/workflows/deploy.yml)) runs on push to `main`, behind the `production` environment approval gate.

Both deploy workflows: (1) detect whether backend/frontend changed, (2) trigger the Render **backend**
deploy and poll `/health_check` **until it reports `status:"ok"`** (DB connected — see below), then
(3) trigger the Render **static-site** build and poll its URL until it serves — so the frontend never
ships before the backend is healthy.

## Architecture

| Component | Service | Free? | Why |
|-----------|---------|-------|-----|
| Backend (Rust/Axum) | **Render** web service ×2 (`docker`) | Free, no card | Builds the Dockerfile, HTTPS URL. Free instances cold-start (~30–60s) after 15 min idle |
| Frontend (Vite/React) | **Render** static site ×2 | Free, no card | Global CDN, **no cold start**, doesn't consume the web-service hour budget |
| Database | **OVH managed MongoDB** (yours) | Already have it | One cluster, two databases (`boardurance_preprod` / `boardurance_prod`); URI passed to each backend as a secret |

> Why static sites for the frontend (not a web service)? They're free with no cold start and are
> **separate from the 750 web-service instance-hrs/month** budget — so the two backends keep the full
> budget to themselves. Four services total, but the two static ones are free and zero-maintenance.

## ⚠️ Two things that make the workflow correct

These are not optional — the design relies on them.

### 1. Database isolation is by env var, not by URI
The app selects the database from `APP_DATABASE__DATABASE_NAME` (defaulting to `rust_backend`), **not**
from the path in the connection URI — see `client.database(&configuration.database_name)` in
[startup.rs](../rust-backend/src/startup.rs). Both backends point at the same OVH cluster, so **each
must set `APP_DATABASE__DATABASE_NAME` explicitly** (`boardurance_prod` / `boardurance_preprod`).
Forget it on either and they both write to `rust_backend` — preprod would corrupt prod.

### 2. Health gate requires `status:"ok"`, not just HTTP 200
`/health_check` returns **HTTP 200 even when Mongo is unreachable** (body `{"status":"degraded"}` — see
[health_check.rs](../rust-backend/src/routes/health_check.rs)). The deploy workflows therefore grep for
`"status":"ok"` so a deploy with a broken DB connection fails the gate instead of looking healthy.

> On first preprod boot, check the Render logs for `Successfully connected to MongoDB`. The connection
> is proven with a `ping` against the `admin` database; if your OVH user lacks `admin` access the app
> silently falls into degraded mode (and the health gate will correctly fail).

## One-time setup

### 1. OVH MongoDB

You already have the cluster. Grab the connection URI from the OVH dashboard — it looks like:

```
mongodb://<user>:<password>@<host>:<port>/?...&tls=true
```

In OVH's *Authorized IPs / ACLs*, allow access from anywhere (`0.0.0.0/0`) — Render's free instances
don't have a fixed outbound IP. No need to pre-create the two databases; Mongo creates them on first
write. Use database names `boardurance_prod` and `boardurance_preprod`.

### 2. Render — one Blueprint creates all four services

1. Sign up at https://render.com with your GitHub account (no card asked).
2. **New → Blueprint**, select this repo. Render reads [render.yaml](../render.yaml) and creates the
   four services: `boardurance-api`, `boardurance-api-preprod`, `boardurance-web`,
   `boardurance-web-preprod`. The `rootDir` keys scope each one (backends → `rust-backend`, frontends
   → `empty-project`), so the two halves never build each other.
3. **Backend services** → each one → **Environment** tab:
   - `APP_DATABASE__URI` → your OVH URI (the **same** cluster URI for both)
   - `APP_DATABASE__DATABASE_NAME` → `boardurance_prod` (prod) / `boardurance_preprod` (preprod) — **required, see ⚠️**
   - `ALLOWED_ORIGINS` → that env's frontend URL (fill in after the static sites get URLs, step 4)
   - `JWT_SECRET` is auto-generated per service — leave it.
4. **Static sites** → each one → **Environment** tab:
   - `VITE_API_BASE_URL` → the matching backend URL (prod static → prod backend URL; preprod static → preprod backend URL). Baked into the bundle at build time.
   - Note each static site's public URL (top of the service page, e.g. `https://boardurance-web.onrender.com`) and put it into the matching backend's `ALLOWED_ORIGINS` (step 3).
5. **Deploy Hooks** — for **each** of the four services, **Settings → Deploy Hook**, copy the URL (used by GitHub Actions, step 4 below).

> First backend build compiles all Rust deps and can take ~10–20 min; later builds are cached by
> `cargo-chef`. Static-site builds take ~1–2 min.

### 3. (No Cloudflare needed)

The frontend is hosted on Render static sites — there is no separate Cloudflare account, API token,
or `wrangler` step anymore.

### 4. GitHub Environments + secrets/variables

Create **two GitHub Environments** under Settings → Environments:

- **`preprod`** — no protection rules (deploys automatically).
- **`production`** — add a **Required reviewers** rule (yourself). This is the one-click gate before
  prod deploys; it was previously only a TODO in [BRANCH_PROTECTION_SETUP.md](../.github/BRANCH_PROTECTION_SETUP.md).

Set these **per environment** (Settings → Environments → *env* → secrets / variables), so the preprod
job physically cannot reach prod's resources:

**Secrets** (the four Render deploy-hook URLs from step 2.5)

| Name | `preprod` | `production` |
|------|-----------|--------------|
| `RENDER_DEPLOY_HOOK_URL` | — | prod **backend** hook |
| `RENDER_PREPROD_DEPLOY_HOOK_URL` | preprod **backend** hook | — |
| `RENDER_WEB_DEPLOY_HOOK_URL` | — | prod **static-site** hook |
| `RENDER_PREPROD_WEB_DEPLOY_HOOK_URL` | preprod **static-site** hook | — |

**Variables**

| Name | `preprod` | `production` |
|------|-----------|--------------|
| `BACKEND_URL` | `https://boardurance-api-preprod.onrender.com` | `https://boardurance-api.onrender.com` |
| `FRONTEND_URL` | `https://boardurance-web-preprod.onrender.com` | `https://boardurance-web.onrender.com` |

> The OVH Mongo URI is **never** stored in GitHub — only in the Render backend dashboards. GitHub
> holds only deploy-hook URLs. A leaked GitHub token cannot reach your database.

### 5. Done

- Merge to `dev` → preprod deploys automatically. Verify at the preprod URLs.
- Open a PR `dev` → `main`; on merge, the prod deploy **waits for your approval** in
  *Actions → Deploy → review deployments*, then ships.
- *Actions → Deploy (preprod) / Deploy → Run workflow* triggers a manual redeploy of either env.

## How frontend/backend stay in sync

- Within an env, both deploy from the **same commit/branch**; the backend deploys **first** and the
  static-site build is only triggered once `/health_check` reports `status:"ok"`.
- The frontend build bakes in `VITE_API_BASE_URL` (see [src/config/api.ts](../empty-project/src/config/api.ts)) — preprod and prod are **separate static sites** with different URLs. Locally it falls back to `http://localhost:3000`.
- Each backend allows its frontend's origin through CORS via `ALLOWED_ORIGINS`.
- A `concurrency` group per env ensures deploys never overlap.
- Each Render service tracks its own `branch` (backends + frontends: `main` for prod, `dev` for preprod), so a deploy hook always builds the right commit.

## Configuration reference

| Variable | Where | Purpose |
|----------|-------|---------|
| `APP_ENVIRONMENT=production` | render.yaml (both backends) | Loads `configuration/production.yaml`. (No `staging`/`preprod` value exists — preprod runs as production and differs only by the secrets below.) |
| `PORT` | injected by Render | Port the app binds to (overrides the YAML port) |
| `APP_DATABASE__URI` | Render backend env (secret) | OVH MongoDB connection string (same cluster for both envs) |
| `APP_DATABASE__DATABASE_NAME` | Render backend env (**required**) | Database name — `boardurance_prod` / `boardurance_preprod`. The isolation guarantee. |
| `JWT_SECRET` | Render backend env (auto-generated, per service) | Token signing key — required, never the dev default; differs per env |
| `ALLOWED_ORIGINS` | Render backend env | CORS origin(s) for that env's frontend (comma-separated) |
| `VITE_API_BASE_URL` | Render static-site env | Backend URL baked into the frontend bundle at build time |
| `NODE_VERSION=20` | render.yaml (static sites) | Pins the static build's Node version |
| `BACKEND_URL` / `FRONTEND_URL` | GitHub env variables | URLs the deploy workflow health-checks / polls |
| `APP_ENVIRONMENT=test` | CI only | Repository-layer mocks, no Mongo, no secrets — the test tier |

## Notes & alternatives

- **Cold starts** affect the **backend web services only** (sleep after 15 min idle, ~30–60s first
  request). Static frontends never sleep. To keep prod backend warm, an external uptime pinger hitting
  `/health_check` works — ping only prod to protect the free 750 instance-hrs/month cap.
- **Free static-site limits**: 100 GB bandwidth/month and fair-use build minutes — far above hobby use.
- **Even fewer services**: you could serve the built frontend directly from the Rust backend (Axum
  static files + SPA fallback) for 2 services total and same-origin (no CORS), at the cost of a slow
  Rust rebuild for every frontend change. Not currently done.

# Boardurance (Racing Tycoon)

[![Backend CI](https://github.com/nostroyo/Boardurance/actions/workflows/backend-ci.yml/badge.svg?branch=dev)](https://github.com/nostroyo/Boardurance/actions/workflows/backend-ci.yml)
[![Frontend CI](https://github.com/nostroyo/Boardurance/actions/workflows/frontend-ci.yml/badge.svg?branch=dev)](https://github.com/nostroyo/Boardurance/actions/workflows/frontend-ci.yml)

A turn-based racing tycoon game. You manage a racing team — cars built from
engine, body, and pilot components — and race other players (or AI opponents)
in turn-based races driven by boost cards.

**How a race works:** each turn, every player commits one boost card; one boost
= one turn = one lap. Sector positions on the track represent the cars'
standings *relative to each other* (not physical lap distance), and the race
ends after `total_laps` turns. Multiplayer races run on a synchronized turn
deadline: players who miss it get an auto-played turn so the race never stalls.

## Repository layout

| Path | What it is |
|------|------------|
| [`rust-backend/`](rust-backend/) | API server — Rust, Axum, MongoDB, JWT/cookie auth (argon2), OpenAPI via utoipa |
| [`empty-project/`](empty-project/) | **The web frontend** (name is legacy) — Vite, React 19, TypeScript, Tailwind 3, React Router 7 |
| [`openspec/`](openspec/) | Living specifications — `specs/<capability>/` is current truth, `changes/` holds in-flight change proposals |
| [`docs/`](docs/) | Feature docs, ADRs (`docs/adr/`), review verdicts (`docs/reviews/`), postmortems, bugfix writeups |
| [`.github/workflows/`](.github/workflows/) | CI (`backend-ci`, `frontend-ci`) and deploys (`deploy-preprod`, `deploy`) |
| [`render.yaml`](render.yaml) | Render blueprint — the four hosted services (prod + preprod, API + static frontend) |

## Quick start

Prerequisites: Rust (stable), Node ≥ 20.19, Docker (only needed for MongoDB).

The one-shot way (Windows / PowerShell — starts MongoDB, backend, frontend):

```powershell
.\start-full-stack.ps1     # stop everything later with .\stop-full-stack.ps1
```

Or run each part manually:

```powershell
# Backend — from rust-backend/
docker compose up -d                 # MongoDB (optional, see note below)
cargo run                            # API on http://localhost:3000

# Frontend — from empty-project/
npm ci
npm run dev                          # UI on http://localhost:5173
```

> **First-time backend setup:** the config loader requires a
> `rust-backend/configuration/local.yaml` (gitignored — absent from a fresh
> clone) on top of `base.yaml`. Create it with at least
> `application: { port: 3000 }` plus your local database settings before
> `cargo run` will boot. (`.env.example` documents the values, but `.env` is
> **not** auto-loaded by the app.)

- API docs (Swagger UI): http://localhost:3000/swagger-ui
- Health check: http://localhost:3000/health_check
- **No MongoDB? The backend still boots in degraded mode** — persistence is
  disabled but the API serves; poll `/api/v1/races` rather than `/health_check`
  to confirm the routes are actually up.

## Development workflow

Branch-per-feature: cut `feat/<slug>` or `fix/<slug>` off `dev`, PR back into
`dev`. `main` only moves via PRs from `dev`. Both branches are protected —
CI must be green and reviews resolved before merge.

Feature work is specced with **OpenSpec**: current behavior lives in
`openspec/specs/<capability>/spec.md`, and every change goes through an
`openspec/changes/<change>/` proposal (proposal → approve → implement →
archive). The legacy `.kiro/specs/` tree is frozen, read-only history.

### Verify loops (local CI parity)

A change is done when the loop matching what you touched passes. These mirror
the CI workflows exactly; the pre-push git hook
(`.claude/scripts/install-git-hooks.ps1`, run once per clone) enforces them.

**Backend** (from `rust-backend/`):

```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings <see rust-backend/CLAUDE.md for the allowed lints>
cargo check --all-targets --all-features
cargo test-fast
```

`cargo test-fast` (alias in `.cargo/config.toml`) runs unit + mock tests with
**no database needed** and is the CI gate. `cargo test-integration` needs
MongoDB up; run it when you touch DB/repository code.

**Frontend** (from `empty-project/`):

```
npm run build        # tsc -b + vite build — the CI gate, stricter than tsc --noEmit
npm run test -- --run
```

Lint (`npm run lint`) and format (`npm run format:check`) are CI soft gates —
fix what you reasonably can.

**Specs** (when anything under `openspec/` changed): `openspec validate --all --strict`

### API types

Frontend API types are generated from the backend's OpenAPI schema — don't
hand-edit them:

```
npm run gen:api          # regenerates src/types/api-generated.ts from docs/openapi.json
npm run gen:api:check    # CI-style drift check
```

## Deployment

Everything runs on Render (free tier), defined in [`render.yaml`](render.yaml):
two Docker web services for the API and two static sites for the frontend.

| Environment | Branch | Services | Deployed by |
|-------------|--------|----------|-------------|
| Preprod | `dev` | `boardurance-api-preprod` + `boardurance-web-preprod` | `deploy-preprod.yml` on merge to `dev` |
| Prod | `main` | `boardurance-api` + `boardurance-web` | `deploy.yml` on merge to `main` |

Deploys are triggered from GitHub Actions (backend first, frontend after the
backend is healthy); Render's own auto-deploy is off. Prod and preprod share
one MongoDB cluster but **must** use different database names
(`APP_DATABASE__DATABASE_NAME` — set per service in the Render dashboard).

## Documentation map

- Game mechanics and per-feature docs: [`docs/features/`](docs/features/)
- Current behavioral specs: [`openspec/specs/`](openspec/specs/) —
  `race-engine`, `boost-system`, `auth`, `persistence`, `ai-opponents`,
  `race-ui`, `admin-management`, `ci-cd`
- Architecture decisions: [`docs/adr/`](docs/adr/)
- Testing guides: [`docs/testing/`](docs/testing/)
- CI/CD and branch protection details: [`.github/README.md`](.github/README.md)

The backend design follows patterns from *Zero to Production in Rust*
(Luca Palmieri).

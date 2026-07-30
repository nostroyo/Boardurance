# Boardurance (Racing Tycoon)

[![Backend CI](https://github.com/nostroyo/Boardurance/actions/workflows/backend-ci.yml/badge.svg?branch=dev)](https://github.com/nostroyo/Boardurance/actions/workflows/backend-ci.yml)
[![Frontend CI](https://github.com/nostroyo/Boardurance/actions/workflows/frontend-ci.yml/badge.svg?branch=dev)](https://github.com/nostroyo/Boardurance/actions/workflows/frontend-ci.yml)

> A turn-based racing tycoon game — and a deliberate experiment in
> spec-driven, agent-assisted development.

This project has two goals that carry equal weight:

1. **The product** — a racing management game where the interesting decisions are
   strategic, not reflex-based.
2. **The method** — build it under a documented, enforced workflow where every
   feature is planned before it is coded, every change has a verify gate, and every
   failure feeds a rule back into the process.

Both are described below: [the goal](#the-goal--what-boardurance-is) and
[the method](#the-method--how-work-gets-done).

---

## The goal — what Boardurance is

You manage a racing team — cars built from engine, body, and pilot components — and
race other players (or AI opponents) in turn-based races driven by boost cards.

**How a race works:** each turn, every player commits one boost card; one boost =
one turn = one lap. Sector positions on the track represent the cars' standings
*relative to each other* (not physical lap distance), and the race ends after
`total_laps` turns. Multiplayer races run on a synchronized turn deadline: players
who miss it get an auto-played turn so the race never stalls.

What makes it a game of decisions rather than dice:

- **Each lap has a characteristic** — straight or curve — which selects whether a
  car's straight or curve values apply. Engine + body + pilot are summed for that
  characteristic.
- **Each sector caps base performance.** The summed value is capped by the current
  sector's ceiling *before* boost is added, so a strong car parked in a low sector
  is wasted potential. Boost is the only way past the ceiling — which makes it the
  lever for climbing sectors, not just a speed button.
- **Boost is scarce and tyre-defined.** The fitted tyre determines the pool of
  boost cards; boost 0 is always free, and a pit stop is the only refill. Spending
  well is the core tactical problem.
- **Solo mode** seats bots that make their own boost/pit/tyre decisions, with AI
  turns enqueued server-side so a solo race never waits on an empty seat.

**Longer-term vision** (not shipped scope): car collection across categories and
rarity tiers, upgrades, a trading marketplace, and tournaments with entry fees and
prize pools. Treat the specs below as the line between vision and reality.

### What exists today

The eight capabilities with written behavioral specs under
[`openspec/specs/`](openspec/specs/):

| Capability | Covers |
|------------|--------|
| `auth` | Register/login/logout/refresh, argon2 hashing, JWT + sessions, auth & role middleware |
| `race-engine` | Race lifecycle, registration, simultaneous turn resolution, sector movement, standings |
| `boost-system` | Tyre-defined boost pools, free boost 0, pit stops, validation, per-use history |
| `ai-opponents` | Seeded bot players, AI boost/pit/tyre policy, server-side AI turn enqueueing |
| `race-ui` | Lobby (list/create/join), in-race interface (track, standings, boost & tyre controls), results |
| `admin-management` | Admin role model, role-based authorization, admin tooling and race-management UI (server-side enforcement of race management never shipped) |
| `persistence` | Repository abstraction — **in-memory is the active backend today**; MongoDB connection handling degrades gracefully |
| `ci-cd` | CI workflows, quality gates, and the deploy pipeline |

Note the surprise in that table: **game data does not survive a restart.** Mock
repositories are wired in for every environment and live race state sits in a
process-global store, so MongoDB is provisioned and connected but is not yet what
backs the game.

Those specs are **frozen history** now (see [the method](#the-method--how-work-gets-done)):
new work is not specced there, and they are no longer updated — so they lag the code
in places. Known gap: the multiplayer turn deadline shipped, but its spec delta was
never archived into `race-engine`, so it lives only in
`openspec/changes/add-multiplayer-turn-sync/`. Read them as a strong behavioral
reference, not as a guarantee.

## Repository layout

| Path | What it is |
|------|------------|
| [`rust-backend/`](rust-backend/) | API server — Rust, Axum, MongoDB, JWT/cookie auth (argon2), OpenAPI via utoipa |
| [`empty-project/`](empty-project/) | **The web frontend** (name is legacy) — Vite, React 19, TypeScript, Tailwind 3, React Router 7 |
| [`_bmad-output/`](_bmad-output/) | **The spec source of truth** — where BMAD writes planning artifacts, epics/stories, review & test artifacts. Mostly empty so far: BMAD landed recently, so today it holds `project-context.md` and a little implementation output |
| [`_bmad/`](_bmad/) | BMAD installation and config (durable overrides in `_bmad/custom/`) |
| [`.claude/`](.claude/) | BMAD agent skills (`skills/bmad-*`) plus the `be.ps1` / `fe.ps1` / git-hook scripts |
| [`openspec/`](openspec/) | Pre-BMAD capability specs — **frozen, read-only history**. `changes/` still holds one implemented-but-unarchived proposal (multiplayer turn sync) |
| [`.kiro/`](.kiro/) | Legacy Kiro-style specs — **frozen, read-only history** |
| [`docs/`](docs/) | Workflow guide, feature docs, ADRs, review verdicts, postmortems, bugfix writeups |
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

---

## The method — how work gets done

The process is the second deliverable. It exists because this codebase is built
largely by AI agents working in parallel, and unwritten conventions do not survive
that.

Feature work runs on **BMAD-METHOD v6** (BMad Core + BMM + Test Architect), installed
as Claude Code skills in `.claude/skills/bmad-*`. **`_bmad-output/` is the spec source
of truth.** The pre-BMAD spec systems — `openspec/` and `.kiro/specs/` — are frozen,
read-only history: still accurate about existing behavior, never edited, never treated
as current truth.

Lost? Invoke **`bmad-help`** — it inspects the project and recommends the next step.
The step-by-step how-to, with personas and a cheat sheet, is
[`docs/BMAD_WORKFLOW_GUIDE.md`](docs/BMAD_WORKFLOW_GUIDE.md); the rules live in
[`CLAUDE.md`](CLAUDE.md).

### Scale to the work

| Change | Path |
|--------|------|
| Small, single-goal (bug fix, tweak, one endpoint/field) | **Quick Flow** — `bmad-quick-dev` runs the whole cycle with a tech-spec instead of a PRD |
| Substantial feature (multiple stories, real design decisions) | **The four phases** below |

No PRD for a two-line fix.

### The four phases

1. **Analysis** *(optional — for fuzzy ideas)*: `bmad-brainstorming`,
   `bmad-product-brief`, `bmad-domain-research` / `bmad-technical-research`.
2. **Planning**: `bmad-prd` → PRD in `_bmad-output/planning-artifacts/`; `bmad-ux`
   when there's UI; `bmad-validate-prd` for completeness.
3. **Solutioning**: `bmad-architecture` → the architecture doc, **where decisions,
   trade-offs and accepted debt are recorded**; `bmad-create-epics-and-stories`;
   `bmad-check-implementation-readiness` before any code.
4. **Implementation**: `bmad-sprint-planning` to sequence, then per story
   `bmad-create-story` → `bmad-dev-story` (implements, ends with the verify loop
   green) → `bmad-code-review` → merge. `bmad-retrospective` closes the epic.
   `bmad-dev-auto` runs the loop unattended under the same rules.

**Testing** is owned by the Test Architect: `bmad-testarch-*` (test design,
automation, CI, NFR, traceability) and `bmad-qa-generate-e2e-tests`, with artifacts
in `_bmad-output/test-artifacts/`.

**Assumption probes.** Every story opens with an `## Assumption Probes` table: each
load-bearing assumption must be settled *before* code — verified by reading
(file:line), verified by a ≤15-minute executable probe, or accepted as risk by a
human — or it blocks `ready-for-dev`. `bmad-dev-story` gates on it, and a failed
load-bearing probe halts the story rather than building on sand. Protocol:
`_bmad/custom/assumption-probes.md`.

### Repo glue — required around every feature

- **Worktree per feature** off `dev`: `git worktree add ../Boardurance-worktrees/<slug> -b feat/<slug> dev`.
  **Never `git checkout` in the bare `Boardurance/` repo** — it is shared by
  concurrent sessions, and switching its branch silently changes what every other
  session reads. This has caused real incidents.
- **Artifacts are the spec.** Everything BMAD writes goes to `_bmad-output/` and is
  committed.
- **`bmad-code-review` before opening any PR** into `dev` or `main`; resolve every
  blocking finding. `dev` auto-deploys to preprod, so it is not a soft target.
- **PR into `dev`** — never push features straight to `dev` or `main`. Both branches
  are protected: CI green and reviews resolved before merge.
- **A YouTrack `RACE-…` issue per epic/story**, moved To do → In Progress → In
  Review → Done. A story is only done with green CI on its PR.

### Verify loops (local CI parity)

A change is done when the loop matching what you touched passes. These mirror the CI
workflows, and `bmad-dev-story` runs them.

The pre-push git hook (`.claude/scripts/install-git-hooks.ps1`, once per clone —
worktrees share `.git/hooks`) is a partial safety net: it covers the backend loop
command-for-command, but on the frontend it runs only `npx tsc --noEmit` and the
tests. **`gen:api:check` and `npm run build` are yours to remember** — they block CI
and the hook will not catch them.

**Backend** (from `rust-backend/`):

```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings <see rust-backend/CLAUDE.md for the allowed lints>
cargo check --all-targets --all-features
cargo test-fast
```

`cargo test-fast` (alias in `.cargo/config.toml`) runs unit + mock tests with
**no database needed** and is the CI gate. `cargo test-integration` needs MongoDB
up; run it when you touch DB/repository code. CI adds one more blocking step,
`cargo build --release`.

**Frontend** (from `empty-project/`):

```
npx tsc --noEmit        # the canonical type gate — also what the pre-push hook runs
npm run gen:api:check   # API-type drift — hard-fails CI, easy to forget locally
npm run test -- --run
npm run build           # tsc -b + vite build — blocking in CI, stricter than tsc --noEmit
```

All four block CI. `CLAUDE.md` lists only the first and third — that is the minimum,
not the full gate.

Lint (`npm run lint`) and format (`npm run format:check`) are CI soft gates — fix
what you reasonably can.

### API types

Frontend API types are generated from the backend's OpenAPI schema — don't
hand-edit them:

```
npm run gen:api          # regenerates src/types/api-generated.ts from docs/openapi.json
npm run gen:api:check    # CI-style drift check
```

### Hard guardrails

Non-negotiable, each born from a real incident. Full list in [`CLAUDE.md`](CLAUDE.md).

- **Never** skip, disable (`.skip`/`.only`/`#[ignore]`), delete or weaken a test —
  and never reduce coverage — to make a check pass.
- **Never** write secrets, tokens or personal data into logs, errors, commit
  messages or fixtures.
- **Always** enforce tenant isolation on every user-scoped query, with a
  cross-tenant negative test for each new data-access path.
- **Always** route user-facing strings through i18n. A literal in a component is a bug.
- **Never** touch production data or run migrations against prod.
- **Always** funnel variants of one concept through a single shared path — one
  turn-resolution helper, one player store.

### The feedback loop

The method is expected to change, and every change is written down:

- **Termination rule** — after **3 failed attempts** at the same error with no new
  information, stop and summarize the blocker instead of retrying.
- **`bmad-retrospective`** when that rule trips, after an incident, or when a defect
  escapes to PR/prod: root cause, detection gap, follow-ups.
- Follow-ups feed back into `_bmad-output/project-context.md` (the rules BMAD agents
  auto-load) and `_bmad/custom/`. Every problem improves the method.

Pre-BMAD equivalents remain as history: ADRs in [`docs/adr/`](docs/adr/),
postmortems in [`docs/postmortems/`](docs/postmortems/), review verdicts in
[`docs/reviews/`](docs/reviews/), session analyses in
[`docs/self-improvement/`](docs/self-improvement/).

---

## Deployment

Everything runs on Render (free tier), defined in [`render.yaml`](render.yaml):
two Docker web services for the API and two static sites for the frontend.

| Environment | Branch | Services | Deployed by |
|-------------|--------|----------|-------------|
| Preprod | `dev` | `boardurance-api-preprod` + `boardurance-web-preprod` | `deploy-preprod.yml` on merge to `dev` |
| Prod | `main` | `boardurance-api` + `boardurance-web` | `deploy.yml` on merge to `main` |

Deploys are triggered from GitHub Actions (backend first, frontend after the
backend is healthy); Render's own auto-deploy is off. Both workflows are
path-filtered — a push touching neither `rust-backend/**` nor `empty-project/**`
(a docs-only change, say) releases nothing. Prod and preprod share one MongoDB
cluster but **must** use different database names
(`APP_DATABASE__DATABASE_NAME` — set per service in the Render dashboard).

## Documentation map

| I want to… | Read |
|------------|------|
| Start a feature | [`docs/BMAD_WORKFLOW_GUIDE.md`](docs/BMAD_WORKFLOW_GUIDE.md) — or just invoke `bmad-help` |
| Know the working rules | [`CLAUDE.md`](CLAUDE.md) (+ per-tier `rust-backend/CLAUDE.md`, `empty-project/CLAUDE.md`) |
| See current plans, stories, reviews | [`_bmad-output/`](_bmad-output/) |
| Understand existing behavior | [`openspec/specs/`](openspec/specs/) — frozen and lagging in places, but still the fullest written description; confirm against the code |
| Read the race rules in depth | [`docs/GAME_MECHANICS.md`](docs/GAME_MECHANICS.md) |
| Call the API | [`docs/API_ROUTES.md`](docs/API_ROUTES.md), or Swagger at `/swagger-ui` |
| Know why something is the way it is | the feature's BMAD architecture doc; pre-BMAD: [`docs/adr/`](docs/adr/) |
| Set up CI / branch protection | [`.github/README.md`](.github/README.md) |
| Find testing guides | [`docs/testing/`](docs/testing/) |

The backend design follows patterns from *Zero to Production in Rust*
(Luca Palmieri).

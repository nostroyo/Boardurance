# Boardurance — Project Context

> Full human-readable project context for spec work. The load-bearing facts are
> mirrored in `openspec/config.yaml` (`context:`), which is injected into every
> OpenSpec planning request.

## Product

Boardurance is a turn-based online motorsport racing management game. Players own
racing cars with real performance attributes (Speed, Acceleration, Handling,
Durability), hire pilots, and compete in races. Races are turn-based: one boost
decision = one turn = one lap; sectors represent relative car standings, not track
position. Longer-term product vision (from the original steering docs): car
collection across categories and rarity tiers, trading/marketplace, tournaments
with entry fees and prize pools.

## Architecture

Two-tier app:

- **`rust-backend/`** — Rust + Axum 0.7 + MongoDB (async driver) + Tokio, JWT auth
  (HTTP-only cookies). Domain-driven design after "Zero to Production in Rust"
  (Palmieri): pure business logic in `src/domain/` (`race.rs`, `engine.rs`,
  `boost_hand_manager.rs`, `ai_player.rs`, `auth.rs`, `player.rs`, …), HTTP
  handlers in `src/routes/` (`races.rs`, `auth.rs`, `players.rs`,
  `health_check.rs`), infrastructure in `startup.rs` / `configuration.rs` /
  `repositories/` / `database/`. utoipa for OpenAPI (Swagger at
  `/swagger-ui`), tracing + bunyan JSON logs, `config` + `secrecy` for layered
  YAML configuration (`configuration/`, `APP_` env prefix). Runs on port 3000;
  degrades gracefully without MongoDB (verify via `/api/v1/races`, not
  `/health_check`).
- **`empty-project/`** — the **frontend** (the name is historical): React 19 +
  TypeScript (strict) + Vite + Tailwind CSS, React Router, Vitest. Port 5173,
  `VITE_` env prefix. Components in `src/components/` (common/, game/,
  player-game-interface/), plus `pages/`, `hooks/`, `services/`, `types/`,
  `utils/`.

Deployment: two Render environments (preprod, prod), four services total, all
with `autoDeploy: false` — releases happen only via the GitHub Actions deploy
hooks (see `ci-cd` capability spec): push to `dev` releases preprod, push to
`main` releases prod. CI in `.github/workflows/`.

## Conventions

- **OS / shell:** Windows 11, PowerShell (`$env:VAR`, `$null`, backtick
  continuation).
- **Command wrappers:** `.claude/scripts/be.ps1 <cargo args>` and
  `.claude/scripts/fe.ps1 <npm/npx args>` set cwd + test env.
- **Verify gates (definition of done):** backend — `cargo fmt --check`, clippy
  (pedantic, `-D warnings` with the allowed exceptions listed in `CLAUDE.md`),
  `cargo check --all-targets --all-features`, `cargo test-fast`; frontend —
  `npx tsc --noEmit`, `npm run test -- --run`. All tasks additionally require
  unit tests plus a full browser end-to-end race.
- **Git:** branch per feature (`feat/…`/`fix/…` off `dev`), PR into `dev`, never
  push features directly to `dev`/`main`. Work in a `git worktree` under
  `../Boardurance-worktrees/` — never `git checkout` in the bare `Boardurance/`
  checkout (shared by concurrent sessions). Conventional commit messages.
- **Review gate:** run `/review-gate` (spec-conformance + correctness + security
  judges) before any PR into `dev` or `main`; verdicts recorded in
  `docs/reviews/`. Architecture decisions get an ADR in `docs/adr/`
  (`/adr`); repeated failures get a postmortem in `docs/postmortems/`.
- **Code quality:** thiserror + `Result` propagation, structured tracing logs,
  clippy pedantic (backend); strict TS, error boundaries, custom hooks, Tailwind
  utility classes (frontend). RESTful APIs with consistent error format and
  OpenAPI docs. Remove old code — no retro-compatibility unless explicitly
  requested.
- **Hard guardrails** (see `CLAUDE.md` Always/Never): never weaken tests to make
  a check pass; never log secrets or personal data; always enforce tenant
  isolation with cross-tenant negative tests; all user-facing strings through
  i18n; never touch prod data; funnel variants of one concept through a single
  shared path.
- **Docs:** all markdown documentation lives under `docs/` (subfolders per
  concern: `bugfixes/`, `features/`, `reviews/`, `adr/`, `postmortems/`,
  `migration/`).

## Spec process

Current truth lives in `openspec/specs/<capability>/spec.md` (one spec per
capability, SHALL requirements with GIVEN/WHEN/THEN scenarios, plus a
`## Verification` section listing the executable checks that prove the
capability). Changes go through `openspec/changes/<change>/` (proposal → approve
→ implement `tasks.md` → `openspec archive`, which merges the deltas into
`specs/` and moves the change to `changes/archive/`).

**Spec history:** everything specced before July 2026 lives in `.kiro/specs/`
(Kiro-style feature folders). That directory is **frozen, read-only history** —
never edit it and never treat it as current truth. The migration mapping from
legacy feature specs to capability specs is documented in
`docs/migration/kiro-to-openspec.md`.

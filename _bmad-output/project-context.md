---
project_name: 'Boardurance'
user_name: 'Yo'
date: '2026-07-23'
sections_completed: ['technology_stack', 'critical_implementation_rules']
existing_patterns_found: 14
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. Focus on unobvious details that agents might otherwise miss._

---

## Technology Stack & Versions

- **Backend — `rust-backend/`**: Rust (edition 2021, single crate, NOT a Cargo workspace) + Axum 0.7 + Tokio + MongoDB. Cookie-based JWT auth with argon2. OpenAPI generated via utoipa into `docs/openapi.json`. Extra binary: `src/bin/create_admin.rs`. Pedantic clippy with a project allow-list (see `CLAUDE.md` Definition of done — do not tighten or loosen it ad hoc).
- **Frontend — `empty-project/`**: despite the name, this IS the web client. Vite + React 19 + TypeScript (strict) + Tailwind 3 + React Router 7 + Vitest. API types are GENERATED: `npm run gen:api` writes `src/types/api-generated.ts` from `docs/openapi.json`; CI fails on drift (`npm run gen:api:check`). Never hand-edit generated types.
- **Dev environment**: Windows 11 + PowerShell (use `$env:VAR`, `$null`, backtick continuation). Command wrappers: `.claude/scripts/be.ps1 <cargo args>` and `.claude/scripts/fe.ps1 <npm/npx args>` set cwd + env correctly.
- **CI/CD**: GitHub Actions (`backend-ci.yml`, `frontend-ci.yml`) + Render deploys — push to `dev` → preprod, push to `main` → prod (`deploy-preprod.yml`, `deploy.yml`). Deploy health check polls `/health_check` for `"status":"ok"` — that response contract is an invariant.

## Critical Implementation Rules

1. **Race model invariants** (unobvious, source of past bugs): *sectors* represent **relative car standings**, not positions on a track. **1 boost = 1 turn = 1 lap**; a race ends after `total_laps` turns. The atomic race-write primitive is `lock_race_turn` + `resolve_turn_core` in `rust-backend/src/domain/races.rs` — route ALL turn mutations through it; never write race state ad hoc.
2. **Single shared path**: variants of one concept go through one helper/store (one turn-resolution helper, one player store). Per-call-site re-implementations caused real drift incidents.
3. **Tenant isolation**: every query touching user/org-scoped data filters by the authenticated tenant, and every new data-access path gets a cross-tenant negative test.
4. **Tests are sacred**: never skip/disable/weaken a test or reduce coverage to go green. Fix the test only if it is genuinely wrong, and say why.
5. **Backend verify loop** (from `rust-backend/`): `cargo fmt --check` → `cargo clippy` (with the CLAUDE.md allow-list, `-D warnings`) → `cargo check --all-targets --all-features` → `cargo test-fast`. Aliases (`test-fast`, `test-integration`, `test-all`) come from `.cargo/config.toml`.
6. **Backend must work without MongoDB** (degraded mode): verify with `cargo test-fast`; when smoke-testing a running server without Mongo, poll `/api/v1/races`, not `/health_check`.
7. **Frontend verify loop** (from `empty-project/`): `npx tsc --noEmit` → `npm run test -- --run` → `npm run build` (CI runs the build, which is stricter than `tsc --noEmit` alone).
8. **i18n**: no hardcoded user-facing strings in components — all display text goes through i18n.
9. **Secrets/PII**: never in logs, error messages, commit messages, or fixtures. Configuration secrets live in gitignored `configuration/local.yaml` (copy it into new worktrees — it does not travel with checkouts).
10. **Git discipline**: never `git checkout` in the bare `Boardurance/` repo (shared by concurrent sessions — keep it pinned to `dev`). All work happens in `git worktree`s under `../Boardurance-worktrees/`, branch-per-feature (`feat/…`, `fix/…`) off `dev`, PR into `dev` (never direct push). A pre-push hook runs the full verify loop; install once via `.claude/scripts/install-git-hooks.ps1`.
11. **Issue tracking**: each feature/story is mirrored as a YouTrack RACE issue (To do → In Progress → In Review → Done). PRs must have green CI before they count as done.
12. **Production**: never touch prod data or run migrations against prod without explicit human approval.
13. **Where project knowledge lives** (`project_knowledge` = `docs/`): `docs/PROJECT_OVERVIEW.md`, `docs/GAME_MECHANICS.md`, `docs/TECHNOLOGY_STACK.md`, `docs/API_ROUTES.md`, `docs/architecture/FRONTEND_BACKEND_SEPARATION.md`, `docs/openapi.json`. Frozen behavioral specs from pre-BMAD systems remain valid *descriptions of current behavior*: `openspec/specs/<capability>/spec.md` (8 capabilities, SHALL + GIVEN/WHEN/THEN) and `.kiro/specs/` (12 legacy feature specs). Treat both as read-only history; new planning happens in `_bmad-output/`.
14. **Large files**: `races.rs` (~4k lines) and `race.rs` (~1.7k) — locate with Grep/Glob and read ranges; don't read whole files.

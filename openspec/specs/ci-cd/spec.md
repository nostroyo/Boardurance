# ci-cd

## Purpose

Continuous integration, quality gates, and the deploy pipeline. Sources of
truth: `.github/workflows/backend-ci.yml`, `.github/workflows/frontend-ci.yml`,
`.github/workflows/deploy.yml`, `.github/workflows/deploy-preprod.yml`,
`render.yaml`, `rust-backend/.cargo/config.toml` (test aliases),
`.github/branch-protection-config.json` (+ `setup-branch-protection.ps1`),
`.claude/scripts/be.ps1` / `fe.ps1`, and the "Definition of done" section of
`CLAUDE.md`.
## Requirements
### Requirement: Backend CI gate

The backend CI workflow (`backend-ci.yml`) SHALL run on pushes and pull
requests targeting `main` or `dev` when `rust-backend/**` (or the workflow
file itself) changed, and SHALL fail unless all of the following blocking
steps pass, run from `rust-backend/` on `ubuntu-latest` with cached cargo
dependencies:

1. `cargo fmt --check`
2. `cargo clippy --all-targets --all-features -- -D warnings` with the
   enumerated `-A` allowances (`too_many_lines`, the four `cast_*` lints,
   `cast_possible_wrap`, `match_wildcard_for_single_variants`,
   `manual_let_else`, `needless_pass_by_value`, `needless_range_loop`,
   `dead_code`)
3. `cargo check --all-targets --all-features`
4. `cargo test-fast` with `APP_ENVIRONMENT: test`
5. `cargo build --release`

`cargo audit` SHALL also run but as advisory only (`continue-on-error: true`).

#### Scenario: Clippy warning blocks the pipeline

- GIVEN a backend change that introduces a clippy warning not covered by the
  enumerated allowances
- WHEN backend CI runs
- THEN the clippy step fails (`-D warnings`) and the workflow reports failure

#### Scenario: Docs-only change does not trigger backend CI

- GIVEN a push to `dev` that touches only files outside `rust-backend/`
- WHEN GitHub evaluates workflow triggers
- THEN `backend-ci.yml` does not run (path filter)

### Requirement: Backend CI tests run without a database

The backend CI test step SHALL use the `test-fast` cargo alias defined in
`rust-backend/.cargo/config.toml` (`test --lib --bins` plus the mock test
targets), which requires no MongoDB. Integration tests (alias
`test-integration`) and the full suite (`test-all`) SHALL remain available for
local runs but SHALL NOT be part of the CI gate.

#### Scenario: CI passes with no MongoDB service

- GIVEN a backend CI run on a runner with no MongoDB container
- WHEN `cargo test-fast` executes
- THEN unit and mock tests run and the step succeeds without any database

### Requirement: Frontend CI gate

The frontend CI workflow (`frontend-ci.yml`) SHALL run on pushes and pull
requests targeting `main` or `dev` when `empty-project/**`,
`docs/openapi.json`, or the workflow file changed, and SHALL fail unless all
of the following blocking steps pass, run from `empty-project/` on Node 18
with npm caching:

1. `npm ci`
2. `npm run gen:api:check` — regenerates `src/types/api-generated.ts` from
   `docs/openapi.json` and fails if it differs from the committed file
3. `npx tsc --noEmit`
4. `npm run test -- --run` (Vitest)
5. `npm run build` (Vite production build)

ESLint (`npm run lint -- --max-warnings 1000`), Prettier
(`npm run format:check`), and `npm audit --audit-level=high` SHALL also run
but as advisory only (`continue-on-error: true`).

#### Scenario: Stale generated API types block the pipeline

- GIVEN `docs/openapi.json` changed but `src/types/api-generated.ts` was not
  regenerated
- WHEN frontend CI runs
- THEN the `gen:api:check` step fails and the workflow reports failure

#### Scenario: Prettier violation does not block

- GIVEN a frontend change with a formatting violation but passing type check,
  tests, and build
- WHEN frontend CI runs
- THEN the Prettier step is marked failed but the workflow still succeeds

### Requirement: Local CI parity

The repository SHALL document and support running the exact CI gates locally
("Definition of done" in `CLAUDE.md`): the backend loop (`cargo fmt --check`,
the full clippy invocation, `cargo check --all-targets --all-features`,
`cargo test-fast`) and the frontend loop (`npx tsc --noEmit`,
`npm run test -- --run`) mirror `.github/workflows/`. The wrapper scripts
`.claude/scripts/be.ps1` and `.claude/scripts/fe.ps1` SHALL run these commands
from the repo root, setting the working directory and (for `be.ps1 test*`)
`APP_ENVIRONMENT=test` automatically.

#### Scenario: Backend gate reproduced locally

- GIVEN a backend change in a local worktree
- WHEN `.claude/scripts/be.ps1 test-fast` is run from the repo root
- THEN cargo executes in `rust-backend/` with `APP_ENVIRONMENT=test`, matching
  the CI test step

### Requirement: Branch protection on main

Branch protection for `main` SHALL be captured as configuration-as-code in
`.github/branch-protection-config.json` and applied with
`.github/setup-branch-protection.ps1` (verified with
`verify-branch-protection.ps1`): required status checks `frontend-ci` and
`backend-ci` with strict up-to-date branches, one approving pull-request
review with stale-review dismissal, required conversation resolution, and
force pushes and deletions disallowed.

#### Scenario: Failing CI blocks merge to main

- GIVEN a pull request into `main` whose `backend-ci` check failed
- WHEN a merge is attempted
- THEN GitHub blocks the merge until the required check passes

### Requirement: Preprod auto-deploys from dev

The preprod deploy workflow (`deploy-preprod.yml`) SHALL trigger on every push
to `dev` (plus `workflow_dispatch`) with no approval gate: the `preprod`
GitHub Environment has no required reviewers, and its secrets and variables
(preprod Render deploy hooks, preprod `BACKEND_URL`/`FRONTEND_URL`) are scoped
to that environment so preprod jobs cannot read production's deploy hooks. A
`dorny/paths-filter` step SHALL limit the backend and frontend deploy jobs to
runs where `rust-backend/**` or `empty-project/**` (or the workflow file)
actually changed. Deploys SHALL be serialized via the `deploy-preprod`
concurrency group with `cancel-in-progress: false`.

#### Scenario: Push to dev reaches preprod without approval

- GIVEN CI-green changes to `rust-backend/**` merged into `dev`
- WHEN the push lands
- THEN `deploy-preprod.yml` runs immediately and triggers the preprod Render
  deploy hook without waiting for any manual approval

### Requirement: Production deploys from main

The production deploy workflow (`deploy.yml`) SHALL trigger on every push to
`main` (plus `workflow_dispatch`), run its jobs in the `production` GitHub
Environment using production-scoped secrets (`RENDER_DEPLOY_HOOK_URL`,
`RENDER_WEB_DEPLOY_HOOK_URL`) and variables, apply the same paths-filter
gating as preprod, and be serialized via the `deploy-production` concurrency
group with `cancel-in-progress: false`.

#### Scenario: Merge to main deploys production

- GIVEN a pull request merged into `main` that changed `empty-project/**`
- WHEN the push to `main` lands
- THEN `deploy.yml` runs and triggers the production Render deploys for the
  changed components

### Requirement: Ordered deploy with backend health gate

In both deploy workflows the backend SHALL deploy first and the frontend job
SHALL depend on it: after triggering the backend Render deploy hook, the
workflow SHALL poll `${BACKEND_URL}/health_check` (up to 80 attempts, 15s
apart) and succeed only when the body reports `"status":"ok"` — HTTP 200 alone
is insufficient because the endpoint returns 200 with `"status":"degraded"`
when the database is unreachable. The frontend deploy SHALL run only when the
frontend changed and the backend deploy did not fail or get cancelled (skipped
is acceptable), then poll the public frontend URL (up to 40 attempts, 15s
apart) until it serves.

#### Scenario: Frontend never ships against a broken backend

- GIVEN a push that changed both backend and frontend
- WHEN the backend deploy never reports `"status":"ok"` within the polling
  window
- THEN the backend job fails and the frontend deploy job does not run

#### Scenario: Degraded backend fails the health gate

- GIVEN the deployed backend responds HTTP 200 with `"status":"degraded"`
  (database not connected)
- WHEN the health-gate polling loop runs to exhaustion
- THEN the deploy job fails rather than declaring the release healthy

### Requirement: Render two-environment topology

`render.yaml` SHALL define exactly four Render services in two environments —
production (`boardurance-api`, `boardurance-web`, branch `main`) and preprod
(`boardurance-api-preprod`, `boardurance-web-preprod`, branch `dev`). Backends
SHALL be Docker web services (`rootDir: rust-backend`, health check path
`/health_check`); frontends SHALL be static sites built with
`npm ci && npm run build`, publishing `./dist`, with an SPA rewrite of `/*` to
`/index.html` and `NODE_VERSION` pinned to 20. All four services SHALL set
`autoDeploy: false` so releases are triggered exclusively by the GitHub
Actions deploy hooks, preserving the backend-then-frontend ordering.

#### Scenario: Render never self-deploys on push

- GIVEN a commit pushed to `main` or `dev`
- WHEN Render observes the branch update
- THEN no Render build starts until the corresponding GitHub Actions workflow
  calls the service's deploy hook (`autoDeploy: false`)

### Requirement: Environment isolation via per-service secrets

The two backend services SHALL share the same MongoDB cluster URI but SHALL
select their database via an explicitly distinct
`APP_DATABASE__DATABASE_NAME` per service (set in the Render dashboard,
`sync: false`), so preprod can never write to the production database. Each
backend SHALL have its own Render-generated `JWT_SECRET`
(`generateValue: true`) so a preprod token is never valid in production, and
its own `ALLOWED_ORIGINS` pointing at its environment's frontend. Each static
site SHALL bake its environment's `VITE_API_BASE_URL` at build time.

#### Scenario: Preprod cannot corrupt production data

- GIVEN the preprod backend deployed with its dashboard-configured database
  name
- WHEN preprod gameplay writes data
- THEN writes go to the preprod database, distinct from production's, on the
  shared cluster

### Requirement: No secrets in the repository

The repository SHALL contain no real secrets: application secrets
(`APP_DATABASE__URI`, database names, `ALLOWED_ORIGINS`, `VITE_API_BASE_URL`)
live in the Render dashboard (`sync: false` in `render.yaml`), deploy hooks
and URLs live in environment-scoped GitHub secrets/variables, and only
placeholder example files (`rust-backend/.env.example`,
`empty-project/.env.example`) are committed. The root `.gitignore` SHALL
ignore `.env*` variants and secret-bearing configuration files
(`configuration/local.yaml`, `configuration/production.yaml`, key material
such as `*.key`/`*.pem`).

#### Scenario: Local env files stay untracked

- GIVEN a developer creates `rust-backend/.env` with real credentials
- WHEN files are staged for commit
- THEN `.gitignore` excludes it and only `.env.example` remains tracked

### Requirement: Spec validation gate

Whenever anything under `openspec/` changes, the change SHALL pass
`openspec validate --all --strict` before it is considered done (CLAUDE.md
Definition of done, "Specs" gate).

#### Scenario: Spec edit is validated

- GIVEN a branch that modifies a file under `openspec/`
- WHEN the Definition-of-done verify loop runs
- THEN `openspec validate --all --strict` reports zero failures

## Verification

- `.claude/scripts/be.ps1 fmt --check`, then
  `.claude/scripts/be.ps1 clippy --all-targets --all-features -- -D warnings -A clippy::too_many_lines -A clippy::cast_possible_truncation -A clippy::cast_precision_loss -A clippy::cast_sign_loss -A clippy::cast_possible_wrap -A clippy::match_wildcard_for_single_variants -A clippy::manual_let_else -A clippy::needless_pass_by_value -A clippy::needless_range_loop -A dead_code`,
  `.claude/scripts/be.ps1 check --all-targets --all-features`,
  `.claude/scripts/be.ps1 test-fast` — the four backend gates pass locally
  with no MongoDB running (Backend CI gate, Backend CI tests run without a
  database, Local CI parity).
- `.claude/scripts/fe.ps1 npm run gen:api:check`,
  `.claude/scripts/fe.ps1 npx tsc --noEmit`,
  `.claude/scripts/fe.ps1 npm run test -- --run`,
  `.claude/scripts/fe.ps1 npm run build` — the frontend blocking steps pass
  locally (Frontend CI gate, Local CI parity).
- `Select-String -Path rust-backend/.cargo/config.toml -Pattern 'test-fast'`
  shows the alias limited to `--lib --bins` and mock test targets (Backend CI
  tests run without a database).
- `Select-String -Path render.yaml -Pattern 'autoDeploy'` returns four
  `autoDeploy: false` lines, and `Select-String -Path render.yaml -Pattern
  'branch:'` returns exactly `main, dev, main, dev` for the api/api-preprod/
  web/web-preprod services (Render two-environment topology).
- `Select-String -Path render.yaml -Pattern 'APP_DATABASE__DATABASE_NAME|JWT_SECRET'`
  shows both backends declaring a dashboard-set database name and a
  `generateValue: true` JWT secret (Environment isolation via per-service
  secrets).
- `git ls-files | Select-String -Pattern '\.env$'` returns nothing, while
  `git ls-files *.env.example` lists the two example files (No secrets in the
  repository).
- `.github/verify-branch-protection.ps1` (requires `gh` auth) reports
  `frontend-ci` and `backend-ci` as required checks on `main` (Branch
  protection on main).
- Live pipeline check (requires GitHub access): after a merge to `dev`,
  `gh run list --workflow deploy-preprod.yml` shows a run whose backend job
  polled `/health_check` for `"status":"ok"` before the frontend job started
  (Preprod auto-deploys from dev, Ordered deploy with backend health gate);
  the same shape holds for `deploy.yml` on `main` (Production deploys from
  main).

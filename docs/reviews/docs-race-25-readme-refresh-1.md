# Review gate — docs/race-25-readme-refresh

- Date: 2026-07-17
- Base SHA: ea2f4d2194c2e87a3fe4789a09accbd5c0ab023a  | Head SHA: 9bebbae7257fbe90ba8f4caeb11324c1a774107c (+ working-tree fix, committed as follow-up)
- Spec: none — ad hoc (RACE-25, docs-only task; no OpenSpec change, spec-conformance judge skipped)
- Changed areas: root docs only (new `README.md`; no backend/frontend code touched)
- Verdict: **PASS**

## Requirement/scenario checklist

- [N-A] No OpenSpec change applies — task is "Redo the readme to reflect the current state of the project" (RACE-25). The correctness judge instead fact-checked every README claim against the repo and against `openspec/specs/`.

## Correctness (fact-check judge)

- [medium — FIXED] `README.md` Quick start — bare `cargo run` panics on a fresh clone: `configuration/local.yaml` is gitignored yet a required config source (`configuration.rs:139-157`), and `.env` is not auto-loaded (no dotenv in `Cargo.toml`). Fixed by adding a "First-time backend setup" note documenting the required `local.yaml`.
- All other claims verified correct against the repo: layout table, tech stack (Cargo.toml / package.json), commands and cargo aliases, ports/routes (`startup.rs:349,355`), CI hard/soft gates (`frontend-ci.yml`), deploy ordering (`deploy*.yml` `needs:`), render.yaml services + `APP_DATABASE__DATABASE_NAME`, OpenSpec capability list, degraded no-Mongo mode (`startup.rs:61-73`), game-model statements (verbatim from `race-engine/spec.md:155-158,220-222`), badges.

## Security (security-review + Always/Never)

- No findings. No secrets/PII/connection strings; README discloses strictly less infra detail than the already-committed `render.yaml` (omits the OVH provider name). All commands are local-dev scoped; deployment section is descriptive only. Docs-only diff — no test skipped/weakened (confirmed via `git diff origin/dev...HEAD --stat`: 1 file, `README.md`). No hardcoded user-facing app strings (README is repo docs, not UI).

## Blocking items (must fix before PR)

- None.

## Non-blocking notes

- "commits one boost card" is slightly loose: boost 0 is the free always-available move, not a card (`boost-system/spec.md:46-49`). Kept as-is — "one boost = one turn = one lap" is the spec's own wording.
- Multiplayer deadline auto-play is shipped behavior but its change (`add-multiplayer-turn-sync`) is not yet archived into `openspec/specs/`.

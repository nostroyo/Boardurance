# Review gate — feat/openspec-migration

- Date: 2026-07-04
- Base SHA: ea2f4d2194c2e87a3fe4789a09accbd5c0ab023a | Head SHA: 800fd46064047964bbadc4e906452615073600cd
- Spec: none — ad hoc (this branch establishes `openspec/` itself; no `openspec/changes/<change>/` governs it, only the archived dry-run `openspec/changes/archive/2026-07-04-add-spec-validation-gate/`)
- Changed areas: docs/process only — zero changes under `.kiro/`, `rust-backend/src/`, `empty-project/src/`
- Verdict: **PASS**

## Requirement/scenario checklist

N/A — no OpenSpec change governs this branch (it creates the spec system, not a feature). Coverage instead comes from the correctness + spec-validation checks below: `openspec validate --all --strict` (8/8 capability specs pass) and a full lifecycle dry-run (`openspec new change` → `validate` → `archive`, archived at `openspec/changes/archive/2026-07-04-add-spec-validation-gate/`).

## Correctness (independent judge)

- [medium] `openspec/project.md:38-39` — claimed "three all-Render environments... with auto-deploy", contradicting the verified `ci-cd` spec and `render.yaml` (two environments, four services, `autoDeploy: false`, GitHub Actions deploy hooks only). **Fixed** in `800fd46`.
- [low] `openspec/specs/race-ui/spec.md:209-214` — the "Off-turn controls are inert" scenario keys on turn phase `Processing`, which the backend never emits (confirmed against `race-engine` spec + `routes/races.rs:2758-2764`); accurate to the frontend component code but unreachable end-to-end. Non-blocking — noted for a future change.

~32 concrete factual claims across the 8 capability specs were spot-checked against `rust-backend/src`, `empty-project/src`, `.github/workflows/`, and `render.yaml`; all 32 held except the medium finding above.

## Security (independent judge)

- [low] `openspec/specs/admin-management/spec.md:210-250` — the "Race management endpoints are not server-side admin-gated" requirement includes a concrete no-auth curl-equivalent example. Intentional, scoped to localhost, and reveals nothing beyond what `startup.rs`/`routes/races.rs` already show. Accepted as-is.

Verified explicitly: no secrets/PII in the diff; zero test files touched; the OpenSpec-CLI-generated `.claude/skills/openspec-*` and `.claude/commands/opsx/*` contain only spec-workflow instructions (no injection/exfiltration patterns); CLAUDE.md Always/Never rules and the review-gate PASS/BLOCK verdict logic are unchanged (additions only).

## Blocking items (must fix before PR)

None.

## Non-blocking notes

- `openspec/specs/race-ui/spec.md` "Off-turn controls are inert" scenario is currently unreachable (see above) — worth a follow-up `openspec/changes/` proposal once `Processing` phase emission is decided one way or the other.
- `openspec/specs/admin-management/spec.md` documents that admin race-management endpoints have **no server-side auth** (mounted with zero middleware) — this is a real, pre-existing security gap the migration surfaced, not introduced by it. Recommend a follow-up change to wire `RequireRole::admin()` onto the race-management routes.
- `docs/migration/kiro-to-openspec.md` contains the full per-capability drift ledger (KEPT/CHANGED/DROPPED/SUPERSEDED/NEW) for all 12 legacy `.kiro/specs/` folders.

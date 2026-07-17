# Review gate — fix/race-24-atomic-apply-lap

- Date: 2026-07-17
- Base SHA: 1917f6e (origin/dev) | Head SHA: 1e16091
- Review scope: `origin/dev...HEAD` — the single commit for RACE-24 (64 insertions in `rust-backend/src/routes/races.rs`). (`git merge-base origin/main HEAD` is used by the command, but main lags dev by PRs #9/#14/#15; the branch's own change is the one commit on top of origin/dev.)
- Spec: none — ad hoc bugfix from YouTrack RACE-24 (no dedicated `openspec/changes/` folder). Spec-conformance judge N/A; noted explicitly.
- Changed areas: backend (`rust-backend/`) only.
- Verdict: **PASS**

## Requirement/scenario checklist
- N-A — no OpenSpec change resolves to this branch; RACE-24 is an ad-hoc concurrency bugfix. Ticket acceptance ("route legacy apply-lap through the atomic primitive + regression test mirroring `concurrent_submits_lose_no_update`") is met: `process_individual_lap_action` now holds `lock_race_turn`, and `concurrent_apply_lap_lose_no_update` mirrors the reference test and was verified to fail with the guard removed.

## Correctness (code-review, medium)
- [low] `races.rs:1854` — `apply_lap_action`'s boost pre-validation reads the race outside the new guard (TOCTOU). Benign: the authoritative `record_player_action` re-validates boost-card availability and duplicate-submission under the guard, so no double-spend or lost update. Non-blocking; candidate for a follow-up that moves validation under the guard.
- [low] `races.rs:1335` — the legacy apply-lap remains a turn-resolution path parallel to `resolve_turn_core`; the guard makes it atomic but does not consolidate the duplication (CLAUDE.md "one turn-resolution helper"). Consolidation is out of scope for an atomicity fix; suggest a follow-up ticket.
- [low] `races.rs:39` — wrapping register/join/start in `lock_race_turn` extends `TURN_LOCKS`' unbounded growth to every registered/joined race UUID. Pre-existing characteristic from PR #14; marginal. Non-blocking.

## Security (security-review + Always/Never)
The `/security-review` skill could not run (cwd is the non-git parent). Applied the checklist manually; the diff has no security surface:
- Tenant isolation — N/A. No new or changed query/data-access path; the change adds only an in-process per-race lock.
- Secrets/PII — none introduced in code, comments, logs, or the commit message. Pre-existing `tracing` calls in `start_race_in_db` log `race_uuid` only (not PII), unchanged.
- Test integrity — no test skipped/ignored/removed/weakened; a new test was **added** and coverage increased (150 lib tests green).
- i18n — N/A (backend Rust; no user-facing strings added).
- No prod-data or migration access.

## Blocking items (must fix before PR)
- None.

## Non-blocking notes
- Follow-up candidate: consolidate the legacy apply-lap path into `resolve_turn_core`, and/or move `apply_lap_action`'s boost pre-validation under the guard.
- Verify loop run on the branch: `fmt --check`, `clippy -D warnings`, `check --all-targets --all-features`, `test-fast` — all green (150 lib tests incl. both `concurrent_*_lose_no_update`).

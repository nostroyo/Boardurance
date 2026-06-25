# Review gate (re-run) — chore/remove-blockchain-and-add-deploy

- Date: 2026-06-25
- Base SHA: 4161d0e | Head SHA: 7c89451 (+ uncommitted working tree, fixes applied)
- Spec: none — chore branch + feature WIP
- Changed areas: backend + frontend
- **Verdict: PASS with notes** (was BLOCK in review #1)

Re-run after fixing the three high-severity blockers from
[review #1](chore-remove-blockchain-and-add-deploy-1.md).

## Fix verification — prior BLOCK cleared
- [FIXED] **#1 solo pit deadlock** — `/pit` and `/submit-action` now funnel through the shared `resolve_human_turn` (races.rs); pit moved to `State<RaceTurnState>`, enqueues AI, processes, drives AI-only turns. Test `pit_resolves_turn_in_solo_mode` binds it.
- [FIXED] **#2 inert tyre pool** — `/submit-action` records via the card-consuming `record_player_action` (race.rs); domain split (`record_player_action` + `process_if_ready`) is behavior-preserving for `/apply-lap`. Test `submit_action_consumes_human_boost_card` binds it.
- [FIXED] **#3 two-store 404** — the 5 asset handlers use `state.player_repository.*_by_uuid` (`State<MockAppState>`); routes consolidated into `team_routes()`; empty `players::routes()` + mount removed; orphaned Mongo helpers deleted. Test `add_car_targets_the_registration_store` binds it.
- No regressions: no double card-consumption; `/apply-lap` behavior unchanged. `cargo check --all-targets` clean; `cargo test-fast` 121 pass (incl. 3 new tests); `cargo fmt --check` clean.

## Correctness (code-review judge) — PASS with notes
- **[MEDIUM, latent — pre-existing]** race.rs `enqueue_ai_actions` (~:1061): `pending_actions.push` runs unconditionally while only the history push is guarded by `if let Ok(result)`. Not currently triggerable (AI only ever chooses 0 or an available card), but if that invariant breaks an AI could race a boost it never consumed. Fix: push inside the `Ok` branch or fall back to boost 0. Not introduced by this change.
- **[LOW]** players.rs `update_player_configuration`: now two non-atomic repo calls (`set_cars_by_uuid` then `update_team_name_by_uuid`). A concurrent delete between them yields a 404 after a partial mutation (benign — player is gone). Consider a single combined repo method.
- **[LOW]** `/apply-lap` (DB-backed) does not enqueue AI / drive AI-only turns like the solo path. Not a regression (solo races live in the in-memory store; `/apply-lap` is multiplayer-DB-only and would 404 for a solo race, not stall). Consistency note: two turn-resolution paths now coexist — consider documenting or unifying.

## Security (security-review judge + Always/Never) — PASS with notes
- **[MEDIUM, pre-existing carry-forward]** The 5 asset handlers (+ GET/PUT/DELETE `/players/:uuid`) on `team_routes()` have no `AuthMiddleware`/ownership check — any caller can mutate any player by UUID. **Auth posture is identical to base** (these were equally unauthenticated on the Mongo router); the refactor makes them *functional* but does not remove a control or widen authz. The `players.rs` TODO already tracks adding `AuthMiddleware` + ownership. Fix before real multi-tenant traffic; not a blocker for this branch.
- **[PASS]** No secrets/PII in new logs (UUIDs only; seeded bot creds never logged).
- **[PASS]** Test integrity — no tests skipped/deleted/weakened; net coverage increased by 3 tests.
- **[PASS]** No secrets in code/config.

## Blocking items
- None. The three prior blockers are resolved with binding regression tests.

## Non-blocking follow-ups (track)
1. Guard `enqueue_ai_actions` push behind the `use_boost_card` result (latent robustness).
2. Make `update_player_configuration` atomic (single repo method) or document non-atomicity.
3. Add `AuthMiddleware` + ownership check to `team_routes()` before multi-tenant traffic (pre-existing).
4. Decide whether `/apply-lap` should share the `resolve_human_turn` path.
5. Pre-existing clippy debt across WIP (jwt/session/startup/test_utils, race.rs, races.rs handlers) still outstanding — unrelated to these fixes.

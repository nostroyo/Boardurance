# Review gate — feat/mongo-persistence (re-run)

- Date: 2026-07-02
- Prior artifact: `feat-mongo-persistence-1.md` (Verdict: BLOCK)
- Base SHA (of this re-check): 3f7b10c | Head SHA: 014e1ca7c61ccbfe42e2a1602e15d4b4fb43c379
- Spec: `.kiro/specs/mongo-persistence/`
- Changed areas: backend only (3 commits, 6 files: `app_state.rs`, `routes/players.rs`, `routes/races.rs`, `services/car_validation.rs`, `startup.rs`, `docs/openapi.json`)
- Scope of this pass: focused, self-conducted review of the fix + its own regression tests, proportionate to the size and clarity of the change — not a full re-run of the 8-angle/security fan-out (see rationale below).
- **Verdict: PASS**

## Blocking item from artifact #1 — resolved

`register_player`/`apply_lap_action` no longer call `CarValidationService::validate_car_for_race(&state.database, ...)`. `validate_car_for_race`/`get_player_by_uuid` now take `&dyn PlayerRepository` and resolve the player via `find_by_uuid`, matching the existing `build_car_data_map`/`state.player_repository.as_ref()` convention used elsewhere in the same file. `AppState.database` (the raw Mongo handle that made the bypass possible) is removed entirely — it had exactly one live caller, which is now fixed, so there is no remaining call site that can silently reach past the repository abstraction.

**Verified directly, not just by report:**
- Read the final diff of all three commits (`3346e72`, `6d71867`, `014e1ca`) line by line.
- Confirmed `AppState` now holds only `*Repository` trait objects — `grep -n "database" src/app_state.rs` returns nothing.
- Confirmed no other live caller of the removed `AppState.database` field exists (the two `players.rs` matches from artifact #1's investigation are inside a `/* TEMPORARILY COMMENTED OUT */` dead-code block, unaffected).
- Added and ran two regression tests (`validate_car_for_race_finds_player_registered_via_repository`, `validate_car_for_race_missing_player_is_not_found`) directly reproducing the scenario that was broken — both pass. This is new coverage that didn't exist before (artifact #1 noted no test exercised this path).
- `cargo fmt --check`, `cargo check --all-targets --all-features`: clean (only the same 3 pre-existing dead-code warnings seen throughout this whole review).
- `cargo test-fast`: **158/158 passed** (156 from before + 2 new), independently run twice.

## Unrelated issue found and fixed along the way

`committed_openapi_schema_is_up_to_date` started failing during this pass. Verified via `git stash` that it **also failed on the unmodified pre-fix commit** — confirming it's unrelated to the `CarValidationService` fix. The actual content was already correct; the only difference after regenerating was a missing trailing newline (a generation/line-ending quirk, not real API drift). Fixed in `6d71867`, written as UTF-8 without BOM (PowerShell's default `>` redirection is UTF-16 and would have corrupted the file — caught and corrected before committing).

## Why this pass wasn't a full 9-agent re-run

The fix is small (5 source files, ~49/64 +/- lines), mechanical, and directly targets exactly the one finding from artifact #1 — I traced every changed line against the original bug report myself. Artifact #1's non-blocking notes (dead `RaceRepository` trait methods, duplicate `players.rs` Mongo CRUD, the pre-existing-but-now-more-likely lost-update race, the AI-auto-drive round-trip cost, hand-rolled test harnesses) are **unchanged by this fix and remain open** — not silently dropped, just out of scope for unblocking this specific PR. Recommend tracking them as separate follow-up work per `.kiro/specs/mongo-persistence/tasks.md`.

## Blocking items (must fix before PR)

None.

## Non-blocking notes (carried over from artifact #1, still open)

- Lost-update race on concurrent race mutation (pre-existing shape, more likely now given Mongo's higher latency).
- AI-auto-drive loop's potential ~4000 sequential Mongo round-trips per request — worth measuring against real latency before/soon after merge.
- `RaceRepository::join_race`/`process_turn_actions`/`submit_turn_action` unused by live routes; `players.rs`'s duplicate raw Mongo CRUD functions.
- 5 integration test files each hand-roll their own test harness instead of reusing `src/test_utils.rs::TestApp`.
- `docker compose`-backed `cargo test-integration` still not run in this environment (Docker unavailable) — the real proof remains the preprod deploy-and-survive check.

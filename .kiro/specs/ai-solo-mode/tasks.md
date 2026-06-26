# Implementation Plan

> Each parent task is **not done** until the full Acceptance Gate passes (see `requirements.md`):
> backend verify loop + frontend verify loop + functional unit tests + a complete browser e2e race.
>
> **Status: all tasks complete.** Backend `cargo test-fast` = 111 passing (incl. AI brain, AI
> enqueue, full solo-race-to-completion). Frontend `tsc` clean + `createSoloRace` unit test green.
> Browser e2e: solo race vs 2 AI bots played to `Finished` with a final ranking, zero console and
> zero network errors on the final run. (Pre-existing clippy/pedantic debt and 12 unrelated
> frontend component-test failures were present before this work and are out of scope.)

- [x] 1. Mark participants as AI
  - [x] 1.1 Add `is_ai` to `RaceParticipant`
    - Add `#[serde(default)] pub is_ai: bool` to `RaceParticipant` in `rust-backend/src/domain/race.rs`
    - Update every `RaceParticipant { .. }` constructor site to set `is_ai` (default `false`)
    - Confirm existing serialized races still deserialize (default applies)
    - _Requirements: 1.1, 1.2_
  - [x] 1.2 Add `add_ai_participant`
    - Add `Race::add_ai_participant(player_uuid, car_uuid, pilot_uuid)` (or private `add_participant_inner(..., is_ai)` + two thin wrappers) so the human path is unchanged
    - _Requirements: 1.3_

- [ ] 2. Boost_Brain decision module
  - [x] 2.1 Create `rust-backend/src/domain/ai_player.rs`
    - Implement `choose_boost(car_data, boost_hand, sector, lap_characteristic) -> u8` per the Balanced_Profile in `design.md`
    - Compute `base` with the same straight/curve component selection as `calculate_performance_with_car_data`
    - Use the **additive** movement classifier (factor a shared `classify_movement(final, sector)` helper); do NOT use the multiplicative preview
    - Register the module in `rust-backend/src/domain/mod.rs`
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_
  - [x] 2.2 Unit tests for `choose_boost`
    - minimal-`MoveUp` pick; anti-`MoveDown` fallback; conserve-when-stuck; never returns an unavailable card; `max_value` cap behavior; determinism
    - _Requirements: 2.1, 2.3, 2.4, 2.5, 2.6 (Properties P1, P3, P4)_

- [ ] 3. Auto-submit AI actions
  - [x] 3.1 Implement `enqueue_ai_actions(race, car_data_map)` in `rust-backend/src/routes/races.rs`
    - For each `is_ai && !is_finished` participant with no pending action, compute boost via `ai_player::choose_boost` and push a `LapAction`
    - Call it inside `submit_player_action_in_db` after the human action push (~line 3894) and before the `players_submitted >= total_players` check (~line 3920)
    - Do not modify `process_lap_in_db` / `process_lap_with_car_data`
    - _Requirements: 3.1, 3.2, 3.3, 3.4 (Properties P5, P6)_
  - [x] 3.2 Integration test: 1 human + 2 AI
    - Submit one human action → assert all AI actions enqueued (legal boosts, no duplicates) and the lap auto-processes
    - _Requirements: 3.1, 3.2_

- [ ] 4. Solo race setup
  - [x] 4.1 Seed Bot_Players
    - In `rust-backend/src/startup.rs` (where `MockPlayerRepository` is built), seed K bot players with complete, slightly varied equipped cars + pilots so `build_car_data_map` resolves them
    - _Requirements: 4.1, 4.2_
  - [x] 4.2 `POST /races/solo`
    - Add `create_solo_race` handler in `races.rs` and register the route near `join_race` (`races.rs:653`)
    - Flow: create_race → add_participant(human) → add_ai_participant(bot) × K → start_race → return `{ race_uuid }`
    - _Requirements: 4.3, 5.1, 5.2_
  - [x] 4.3 Frontend Solo entry point
    - Add `soloRace()` to `empty-project/src/services/raceAPI.ts` (POST `/races/solo`) + unit test
    - Add a "Solo Race" button that calls it and routes to the existing race interface
    - _Requirements: 4.4, 4.5_

- [ ] 5. Preview/resolution parity fix
  - [x] 5.1 Switch preview paths to the additive model
    - `boost_hand_manager.rs:204-206` (`get_boost_availability`) and `races.rs:2185-2188` (`get_performance_preview`): replace `capped_base * (1 + boost*0.08)` with `capped_base + boost`
    - Reuse the shared `classify_movement` helper (from Task 2.1) so preview and resolution share one source of truth
    - _Requirements: 6.1, 6.2, 6.5_
  - [x] 5.2 Correct docs and the multiplicative test
    - Fix doc comments at `races.rs:211-216`, `races.rs:1302`, `races.rs:1903-1904`
    - Update the assertion at `boost_hand_manager.rs:541-545` to the additive result; verify it equals the frontend `calculateFinalValue` for the same inputs
    - _Requirements: 6.3, 6.4_

- [ ] 6. End-to-end verification (HARD GATE)
  - [x] 6.1 Run backend + frontend verify loops (see `requirements.md`)
  - [x] 6.2 Browser e2e: launch a Solo Race, play every turn to `Finished`
    - No waiting-for-players hang; AI cars advance/conserve sensibly; final ranking shown; no console/network errors; capture a screenshot as proof
    - _Requirements: 5.1, 5.2, 5.3_

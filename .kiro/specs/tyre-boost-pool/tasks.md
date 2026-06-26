# Tyre-Based Boost Pool + Pit Stops — Tasks

- [x] 1. Add `TyreType` enum + `initial_pool`; redesign `BoostHand` to a count-based
      multiset (`with_tyre`, free-0 `use_card`, `refill`, `get_available_cards`,
      `card_counts`); remove auto-replenish. (`domain/race.rs`)
- [x] 2. Update `BoostHandManager` + `BoostUsageResult`/`BoostAvailability`/error response
      for tyre + pit stops; drop replenishment. Fix module unit tests.
      (`domain/boost_hand_manager.rs`)
- [x] 3. Thread tyre through registration: `add_participant_with_tyre`,
      `RegisterPlayerRequest`, `CreateSoloRaceRequest`. (`domain/race.rs`, `routes/races.rs`)
- [x] 4. Pit stop: `process_individual_pit_action` + `POST /races/{uuid}/pit` endpoint +
      utoipa/schema/route registration. (`domain/race.rs`, `routes/races.rs`, `startup.rs`)
- [x] 5. Frontend: types, `createSoloRace(tyre)` + `pitStop()`, tyre selector in
      `GameLobby`, free-0 + counts in `BoostControlPanel`, pool panel in `PerformancePreview`.
- [x] 6. Tests + spec: domain/manager/AI unit tests, integration test files updated, this
      spec. Backend `cargo test-fast` + `cargo check --all-targets` green; frontend `tsc`
      green and touched-file tests green.

## Remaining (follow-up, not blocking)

- [x] In-race pit button wired through the simultaneous-turn controller
      (`RaceContainer.pitStopAction` → `RaceInterface` → `BoostControlPanel`; tyre dropdown
      + "Pit & refill" button, disabled off-turn; refreshes via `handleTurnComplete`).
- [x] AI pit strategy + AI tyre choice (`ai_player::decide_ai_action` — pit when pool
      empty and a refilled card would move up / rescue, with a future lap to spend it;
      `enqueue_ai_actions` performs the refill; solo AIs seeded with cycling Soft/Medium/Hard
      tyres via `add_ai_participant_with_tyre`).
- [ ] Tune pool values after playtests.
- [ ] Run DB-backed `cargo test-integration` once MongoDB is available.

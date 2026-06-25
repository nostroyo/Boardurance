# Design Document

## Overview

AI Solo Mode adds server-controlled opponents to the existing turn-based race engine. The race simulation is already agnostic to *who* supplies a `boost_value` — a lap resolves from a set of `LapAction { player_uuid, boost_value }` regardless of source. So the feature is mostly **plumbing + a small decision function**, not new simulation:

1. Mark a participant as AI (`is_ai` flag).
2. A pure **Boost_Brain** that returns a boost 0–4 from race state.
3. **Auto-submit**: when a human submits, enqueue AI actions, then let the existing auto-process path resolve the lap.
4. **Solo setup**: seed bot players, plus a `POST /races/solo` bootstrap and a frontend entry point.

This extends the [`single-player-race-mvp`](../single-player-race-mvp/design.md) (solo human turn-loop) by filling the opponent seats with AI.

### Race model (corrected)

The `single-player-race-mvp` spec describes laps as looping a physical track; that is **wrong**. The actual model (confirmed with the product owner):

- **Sectors are the live relative standings between cars** (who is ahead/behind), not positions around a lap. The top sector is the lead and never wraps.
- **1 boost played = 1 turn = 1 lap.** `current_lap` is the turn number (capped at `total_laps`) and increments by 1 every boost round.
- **The race ends after `total_laps` turns**; everyone crosses the line together on the final turn. Final ranking = furthest-ahead car (`current_sector` desc → `position_in_sector` asc → `total_value` desc).

This is enforced in `race.rs` by `process_lap_internal` (lap = turns_taken + 1), `move_participant_up` (top sector holds, no wrap), and `check_race_completion` (ends at `turns_taken >= total_laps`). Solo races use `total_laps = 5`.

## Architecture

```
Human submits boost  ──► POST /races/{uuid}/submit-action
                              │
                              ▼
                     submit_player_action_in_db()
                              │  push human LapAction
                              ▼
                     enqueue_ai_actions(race, car_data_map)   ◄── NEW
                              │  for each is_ai && !finished && not-yet-acted:
                              │     boost = ai_player::choose_boost(...)
                              │     push LapAction
                              ▼
              players_submitted >= active_players ?  (unchanged)
                              │ yes
                              ▼
                     process_lap_in_db() → process_lap_with_car_data()  (unchanged)
```

Bootstrap:
```
POST /races/solo ──► create_race ──► add_participant(human)
                                ──► add_ai_participant(bot) × K
                                ──► start_race ──► returns race_uuid
Frontend "Solo Race" button ──► soloRace() ──► route to existing RaceContainer
```

## Key existing code reused

- **Boost hand**: `BoostHand` — `is_card_available`, `get_available_cards`, `use_card` (auto-replenish). `rust-backend/src/domain/race.rs:13`.
- **Authoritative performance (Additive_Model)**: `calculate_performance_with_car_data` → `final = min(engine+body+pilot, sector.max_value) + boost`. `rust-backend/src/domain/race.rs:801`.
- **Movement classification**: `< min ⇒ MoveDown`, `> max ⇒ MoveUp`, else `Stay` (`calculate_movement_probability`, `boost_hand_manager.rs:244`). The Boost_Brain reimplements/reuses this with the additive value.
- **Car-data resolution**: `build_car_data_map` + `ValidatedCarData` resolve engine/body/pilot per participant from the player repository. `rust-backend/src/routes/races.rs:83`.
- **Participant lifecycle**: `add_participant` (`race.rs:332`), `start_race` (`race.rs:392`).
- **Submission/auto-process**: `submit_player_action_in_db` (`races.rs:3835`); the trigger is `players_submitted >= total_players` at ~line 3920.

> **Correctness note (carried from requirements):** the *preview* paths use a multiplicative model (`capped_base * (1 + boost*0.08)`), which disagrees with the additive resolution. The Boost_Brain MUST use the additive model so its predicted movement matches reality, AND we fix the preview paths to additive as part of this work (see Component 6) so the human-facing preview agrees with what actually happens. Do not route AI decisions through the preview.

## Components and Interfaces

### 1. `RaceParticipant.is_ai` (modify) — `rust-backend/src/domain/race.rs`
```rust
#[serde(default)]
pub is_ai: bool,
```
`#[serde(default)]` keeps stored races deserializable (defaults `false`). Add `Race::add_ai_participant(player_uuid, car_uuid, pilot_uuid)` mirroring `add_participant` but setting `is_ai = true` (or a private `add_participant_inner(..., is_ai)` with two thin public wrappers to avoid duplication).

### 2. Boost_Brain (new) — `rust-backend/src/domain/ai_player.rs`
```rust
pub fn choose_boost(
    car_data: &ValidatedCarData,
    boost_hand: &BoostHand,
    sector: &Sector,
    lap_characteristic: &LapCharacteristic,
) -> u8
```
Algorithm (Balanced_Profile):
1. `base = min(engine+body+pilot for lap_characteristic, sector.max_value)` — component selection mirrors `calculate_performance_with_car_data`.
2. `available = boost_hand.get_available_cards()` (already sorted ascending).
3. For each `b` in `available`, `final = base + b`; classify via additive `MoveUp/Stay/MoveDown`.
4. Return the **smallest** `b` whose class is `MoveUp`.
5. Else, if `base < sector.min_value`, return the smallest `b` reaching `Stay` (avoid `MoveDown`).
6. Else return `available[0]` (smallest, conserve).
- Pure / no RNG ⇒ deterministic. (Future variety: seed RNG from race uuid + lap — out of scope.)
- Factor the additive classifier into a small shared helper (e.g. `classify_movement(final, sector)`) to keep parity with the resolution rule.

### 3. `enqueue_ai_actions` (new) — `rust-backend/src/routes/races.rs`
```rust
fn enqueue_ai_actions(race: &mut Race, car_data_map: &HashMap<Uuid, ValidatedCarData>)
```
- For each `p in race.participants` where `p.is_ai && !p.is_finished` and no existing `pending_actions` entry for `p.player_uuid`:
  - look up `ValidatedCarData` from `car_data_map`;
  - `boost = ai_player::choose_boost(car_data, &p.boost_hand, current_sector(p), &race.lap_characteristic)`;
  - push `LapAction { player_uuid: p.player_uuid, boost_value: boost as u32 }`.
- Call it inside `submit_player_action_in_db` **after** the human action is pushed (~line 3894) and **before** the `players_submitted >= total_players` check (~line 3920). No change to `process_lap_in_db`.
- `car_data_map` is already built by the caller (`submit_turn_action`) via `build_car_data_map`, which covers bots once they are seeded players.

### 4. Solo setup — bots + bootstrap
- **Seed Bot_Players**: in `startup.rs` where `MockPlayerRepository` is constructed (`startup.rs:234`), insert K bot players each with a complete, slightly varied equipped car + pilots, so `build_car_data_map` resolves them with no special-casing.
- **`POST /races/solo`** (`create_solo_race` in `races.rs`, registered near the `join_race` route at `races.rs:653`): pick/seed a track, `create_race`, `add_participant(human)`, `add_ai_participant(bot)` × K, `start_race`, return `{ race_uuid }`. Reuse existing internals; no duplicated sim logic.

### 5. Frontend — `empty-project/`
- `src/services/raceAPI.ts`: add `soloRace(): Promise<{ race_uuid: string }>` (POST `/races/solo`), mirroring `submitTurnAction` style.
- A "Solo Race" button (lobby/home) that calls `soloRace()` then navigates to the existing race route (`/races/:raceUuid/play`). No turn-loop changes: AI auto-submits server-side, so after the human submits, the turn resolves immediately and the existing 2s polling / `TurnProcessed` path picks it up.

### 6. Preview/resolution parity fix (modify)
Promote the additive model into the two preview paths so the human-facing preview matches lap resolution and the frontend's own `calculateFinalValue` (`empty-project/src/utils/performanceCalculation.ts:46`, already additive):
- `BoostHandManager::get_boost_availability` — `boost_hand_manager.rs:204-206`: replace `capped_base * (1 + boost*0.08)` with `capped_base + boost`.
- `get_performance_preview` endpoint — `races.rs:2185-2188`: same replacement.
- Correct the multiplier doc comments at `races.rs:211-216`, `races.rs:1302`, `races.rs:1903-1904`.
- Update the assertion in the preview unit test `boost_hand_manager.rs:541-545` from multiplicative to additive.
- Movement classification (`calculate_movement_probability`) logic is unchanged — only the value fed into it changes. Reuse the same `classify_movement` helper introduced for the Boost_Brain to keep a single source of truth.

## Data Models

```rust
// race.rs
pub struct RaceParticipant {
    // ...existing fields...
    #[serde(default)]
    pub is_ai: bool,
}
```
No other model changes. `LapAction`, `BoostHand`, `ValidatedCarData`, `Sector`, `LapCharacteristic` are reused as-is.

## Correctness Properties

- **P1 — Legal boost only:** `choose_boost` output is always in `boost_hand.get_available_cards()`. *(Req 2.1)*
- **P2 — Additive parity:** the movement class the brain predicts for the chosen boost equals the class the real resolution produces for the same `(base, boost, sector)`. *(Req 2.2)*
- **P3 — Minimal MoveUp:** if any available boost yields `MoveUp`, the chosen boost is the smallest such. *(Req 2.3)*
- **P4 — Determinism:** identical inputs ⇒ identical output. *(Req 2.6)*
- **P5 — No AI stall:** after a human submits in a solo race, every active AI participant has a pending action, so `players_submitted >= active_players` holds and the lap auto-processes. *(Req 3.1, 3.2, 5.3)*
- **P6 — Resolution untouched:** `process_lap_with_car_data` is unchanged; AI and human actions flow through the same path. *(Req 3.3)*
- **P7 — Preview parity:** for any `(base, boost, sector)`, the preview's predicted final value equals `min(base, sector.max_value) + boost`, identical to lap resolution and the frontend's `calculateFinalValue`. *(Req 6.1, 6.4)*

## Testing Strategy

### Unit (backend, `cargo test-fast`)
- `ai_player::choose_boost`:
  - picks the smallest `MoveUp` boost when several advance;
  - falls back to smallest `Stay` boost when below `min` and no `MoveUp`;
  - conserves (smallest available) when nothing helps;
  - skips unavailable cards (e.g. card `0` used → never returned);
  - at `max_value` cap, `base` is capped before adding boost.
- `enqueue_ai_actions`: a `Race` with 1 human + 2 AI, after pushing the human action, enqueues exactly the two AI actions with legal boosts and does not double-add.

### Integration (backend)
- Build a small solo race (1 human + 2 AI), submit one human action, assert all three actions present and the lap auto-processes (sectors / `total_value` advance), repeated until `Finished`.

### Frontend (`npm run test -- --run`)
- `raceAPI.soloRace()` issues the correct POST and parses `race_uuid` (mirror existing `submitTurnAction` test).

### End-to-end (browser, `preview_*`) — HARD GATE
- Start backend + dev server, launch a Solo Race, play **every turn to completion**, assert no waiting-for-players hang, race reaches `Finished` with a ranking, no console/network errors, screenshot as proof.

## Out of Scope (future)
- Tire / pit / fuel / weather systems and AI for them.
- Difficulty tiers, personalities, adaptive/lookahead AI.

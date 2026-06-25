# Tyre-Based Boost Pool + Pit Stops — Design

## Domain model (`rust-backend/src/domain/race.rs`)

```rust
pub enum TyreType { Soft, Medium, Hard }      // Default = Medium
impl TyreType { pub fn initial_pool(self) -> Vec<u8> { /* Soft [3,4,4], Medium [2,2,3,3,4], Hard [1,1,1,2,2,3] */ } }

pub struct BoostHand {
    pub tyre_type: TyreType,
    pub cards: HashMap<String, u32>,   // remaining count per value "1".."4"
    pub cards_remaining: u32,
    pub pit_stops_completed: u32,
}
```

Key methods:
- `with_tyre(tyre)` builds the pool; `new()` = Medium.
- `is_card_available(v)` — `v == 0` always true; else count > 0.
- `use_card(v)` — `v == 0` is a free no-op; else decrement. **No auto-replenish.**
- `refill(new_tyre)` — reset pool, set tyre, recompute remaining, `pit_stops_completed += 1`.
- `get_available_cards()` — sorted distinct available values, **0 always first**.
- `card_counts()` — `(value, remaining)` for 1..=4 (UI helper).

`BoostUsageRecord` is unchanged structurally but repurposed: `cycle_number` records the
pit-segment (`pit_stops_completed` at use), `replenishment_occurred` is always `false`.

## Pit stop = a lap action that refills

`Race::process_individual_pit_action(player, new_tyre, car_data)` validates the turn,
**refills the pool immediately** (with `new_tyre` or the current tyre), then delegates to
`process_individual_lap_action(player, 0, car_data)`. Because the player commits boost 0
that turn, the immediate refill is behaviourally identical to "the pit costs this lap":
fresh pool from the next turn onward. No new movement code; it joins the normal
simultaneous-turn batch.

## API (`rust-backend/src/routes/races.rs`, `startup.rs`)

- `RegisterPlayerRequest` and `CreateSoloRaceRequest` gain `#[serde(default)] tyre_type: TyreType`;
  threaded to `Race::add_participant_with_tyre(..)`.
- New `POST /api/v1/races/{race_uuid}/pit` with `PitStopRequest { player_uuid, car_uuid, new_tyre? }`
  → `process_individual_pit_action`. Registered in router + utoipa paths; `TyreType` added to schemas.
- `BoostAvailabilityResponse` / `BoostCycleInfo` / `BoostAvailability` expose `tyre_type` +
  `pit_stops_completed` + count-based `hand_state` instead of cycle fields.

## AI (`rust-backend/src/domain/ai_player.rs`)

No logic change: `choose_boost` already returns a value from `get_available_cards()`,
which reflects the pool and always includes 0. The AI degrades to boost 0 when its pool is
empty and never pits (follow-up).

## Frontend (`empty-project/`)

- `types/race-api.ts`: `TyreType` union; `BoostAvailability.hand_state: Record<string, number>`,
  `tyre_type`, `pit_stops_completed`.
- `services/raceAPI.ts`: `createSoloRace(playerUuid, tyreType?)`, new `pitStop(...)`.
- `GameLobby.tsx`: tyre selector for solo races.
- `BoostControlPanel.tsx`: free-0 badge, per-value remaining-count badges, tyre header.
- `PerformancePreview.tsx`: tyre + cards-remaining + pit-stops panel (was cycle panel).

## Follow-ups

- Tune pool values after playtests (single source: `TyreType::initial_pool`).
- Wire an in-race pit button through the turn controller (endpoint + API method already exist).
- Teach the AI to pit and pick a tyre strategy.
- Richer pit movement/penalty (currently a boost-0 lap).

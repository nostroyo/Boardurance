# boost-system

## Purpose

Boost cards as a tyre-defined, depletable resource: tyre types and their card
pools, the always-free boost 0, pit stops that refill the pool, boost
validation, per-use history, and how boost state is exposed in API responses.
Excludes general race lifecycle (race-engine), AI decision policy
(ai-opponents), and UI (race-ui). Sources of truth:
`rust-backend/src/domain/race.rs` (`TyreType`, `BoostHand`,
`BoostUsageRecord`, `record_player_action`, `record_pit_action`),
`rust-backend/src/domain/boost_hand_manager.rs` (`BoostHandManager`,
`BoostAvailability`, `BoostCardError`), and `rust-backend/src/routes/races.rs`
(registration/solo-race requests, `/submit-action`, `/pit`,
`/boost-availability`, `/performance-preview`, `/lap-history`).

## Requirements

### Requirement: Tyre-defined boost pools

The system SHALL define three tyre types — `Soft`, `Medium`, `Hard` (default
`Medium`) — where each tyre grants a boost card pool as a multiset of card
values 1-4 (`TyreType::initial_pool`, the single tuning point): Soft =
`[3, 4, 4]` (3 cards), Medium = `[2, 2, 3, 3, 4]` (5 cards), Hard =
`[1, 1, 1, 2, 2, 3]` (6 cards). A participant's `BoostHand` SHALL track the
remaining count per card value (`cards` map with string keys `"1".."4"`), the
total `cards_remaining`, the fitted `tyre_type`, and `pit_stops_completed`.
Softer tyres give fewer but stronger cards; harder tyres give more but weaker
cards.

#### Scenario: Pool initialization per tyre

- GIVEN a participant entering a race with a chosen tyre
- WHEN the boost hand is initialized (`BoostHand::with_tyre`)
- THEN the hand holds exactly that tyre's pool — Soft 3 cards `[3,4,4]`,
  Medium 5 cards `[2,2,3,3,4]`, Hard 6 cards `[1,1,1,2,2,3]` — with
  `cards_remaining` equal to the pool size and `pit_stops_completed` = 0

#### Scenario: Value absent from the pool is unavailable

- GIVEN a fresh Medium hand (pool `[2,2,3,3,4]`)
- WHEN boost value 1 is checked or selected
- THEN it is reported unavailable and its use is rejected, even though no card
  has been spent yet

### Requirement: Boost 0 is the free always-available move

Boost value 0 SHALL NOT be a card: it is the always-available free "no boost"
move. Selecting boost 0 SHALL always validate, consume no card, and never
error — including when the pool is empty. `get_available_cards()` SHALL always
include 0 (sorted first).

#### Scenario: Boost 0 with an empty pool

- GIVEN a participant whose pool has been fully spent (`cards_remaining` = 0)
- WHEN the participant submits a lap action with boost 0
- THEN the action is accepted, no card is consumed, and `cards_remaining`
  stays 0

### Requirement: Card consumption without auto-replenish

WHEN a participant plays a boost value 1-4, the system SHALL consume exactly
one matching card from the multiset (decrement its remaining count and
`cards_remaining`). Duplicate values SHALL be usable once per copy. The pool
SHALL NOT auto-replenish: once empty, only boost 0 is available until the
participant performs a pit stop (`BoostHand::refill` is the only restore
path).

#### Scenario: Duplicate cards deplete one copy at a time

- GIVEN a fresh Medium hand with two value-2 cards
- WHEN the participant plays boost 2 on two successive turns
- THEN the first play leaves one value-2 card, the second leaves zero, and a
  third attempt at boost 2 is rejected as not available

#### Scenario: Empty pool does not refill by itself

- GIVEN a participant who has spent every card in the pool
- WHEN subsequent turns are played without pitting
- THEN `get_available_cards()` returns exactly `[0]`, `pit_stops_completed`
  remains unchanged, and no cards reappear

### Requirement: Tyre selection at race entry

Players SHALL choose their starting tyre when registering for a race
(`RegisterPlayerRequest.tyre_type`) and when starting a solo race
(`CreateSoloRaceRequest.tyre_type`); the field SHALL default to `Medium` when
omitted (`#[serde(default)]`). The chosen tyre SHALL be threaded to the
participant's boost hand via `Race::add_participant_with_tyre`. In solo races
the seeded AI opponents SHALL be assigned varied tyres cycling
Soft/Medium/Hard.

#### Scenario: Solo race with an explicit tyre

- GIVEN a player starting a solo race with `tyre_type: "Soft"`
- WHEN the race is created via `POST /api/v1/races/solo`
- THEN the human participant's boost hand is the Soft pool `[3,4,4]`

#### Scenario: Tyre omitted defaults to Medium

- GIVEN a registration or solo-race request without a `tyre_type` field
- WHEN the request is deserialized and the participant added
- THEN the participant's hand is the Medium pool `[2,2,3,3,4]`

### Requirement: Pit stop refills the pool and costs the turn

The system SHALL perform a pit stop for the current turn via
`POST /api/v1/races/{race_uuid}/pit` (`PitStopRequest { player_uuid,
car_uuid, new_tyre? }`): refill
the pool from `new_tyre`'s pool (or the current tyre when `new_tyre` is
omitted), set the fitted tyre accordingly, increment `pit_stops_completed`,
and consume the turn as a free boost-0 lap (the pool is refilled on
submission, so the fresh pool is playable from the next turn onward). The pit
SHALL be subject to the same turn guards as a boost submission (race in
progress, participant exists and not finished, not already acted this turn)
and SHALL resolve the turn through the same single funnel as
`/submit-action` (`resolve_human_turn` with `TurnIntent::Pit`), so a solo-mode
pit enqueues AI opponents and never deadlocks the turn.

#### Scenario: Deplete, pit with a new tyre, refilled

- GIVEN a solo participant on Soft whose pool is empty (only boost 0
  available)
- WHEN the player calls `POST /races/{uuid}/pit` with `new_tyre: "Hard"`
- THEN the turn resolves as a boost-0 lap, and afterwards the hand shows
  `tyre_type: "Hard"`, the full Hard pool counts, `cards_remaining` = 6, and
  `pit_stops_completed` incremented by 1

#### Scenario: Pit without a tyre keeps the current tyre

- GIVEN a participant fitted with Medium and a partially spent pool
- WHEN they pit with `new_tyre` omitted
- THEN the pool is reset to the full Medium pool and the fitted tyre is still
  Medium

#### Scenario: Pit rejected when the player already acted this turn

- GIVEN a participant who has already submitted an action for the current turn
- WHEN they call the pit endpoint
- THEN the request fails with HTTP 409 and error code `RACE_STATE_ERROR`, and
  no refill occurs

### Requirement: Boost selection validation

The system SHALL validate every boost selection
(`BoostHandManager::validate_boost_selection`) before consuming a card: values
greater than 4 SHALL be rejected as `InvalidBoostValue` (API error code
`INVALID_BOOST_VALUE`); values 1-4 with zero remaining copies SHALL be
rejected as `CardNotAvailable` (API error code `BOOST_CARD_NOT_AVAILABLE`)
with a message and payload listing the currently available cards; boost 0
SHALL always be accepted. `/submit-action` SHALL return these as HTTP 400
`BoostCardErrorResponse` (including `available_cards`, `cards_remaining`,
`pit_stops_completed`), and SHALL reject a second action in the same turn with
HTTP 409 (`RACE_STATE_ERROR`), so a card can never be double-spent within a
turn.

#### Scenario: Selecting a spent card

- GIVEN a participant who has used their only value-4 card
- WHEN they submit a lap action with boost 4
- THEN the response is HTTP 400 with error code `BOOST_CARD_NOT_AVAILABLE`
  and an `available_cards` list that contains 0 but not 4, and no card is
  consumed

#### Scenario: Out-of-range boost value

- GIVEN any participant
- WHEN they submit a lap action with boost value 5 or greater
- THEN the response is HTTP 400 with error code `INVALID_BOOST_VALUE`

### Requirement: Boost usage history per pit segment

Every card use (including the free boost 0 and AI plays) SHALL append a
`BoostUsageRecord` to the participant's `boost_usage_history` recording the
turn (`lap_number` = turns taken + 1), `boost_value`, `cards_remaining_after`,
and `cycle_number` — which SHALL record the pit segment, i.e.
`pit_stops_completed` at the time of use; `replenishment_occurred` SHALL
always be `false` (auto-replenish no longer exists). The participant SHALL
provide per-segment summaries (`get_boost_cycle_summaries`: cards used, laps,
average boost per segment) and aggregates (`get_total_boosts_used`,
`get_average_boost_value`), exposed via the player-specific race status data
and `GET /races/{race_uuid}/players/{player_uuid}/lap-history`.

#### Scenario: Usage recorded with its pit segment

- GIVEN a participant who plays boost 3, pits, then plays boost 2
- WHEN the usage history is read
- THEN the boost-3 record has `cycle_number` 0, the pit's boost-0 record and
  the boost-2 record have `cycle_number` 1, and every record has
  `replenishment_occurred` = false

### Requirement: Boost state visibility in API responses

`GET /races/{race_uuid}/players/{player_uuid}/boost-availability` SHALL return
the participant's current boost state: `available_cards` (sorted distinct
values, 0 always included), `hand_state` (remaining count per value, string
keys `"1".."4"`), `tyre_type`, `pit_stops_completed`, and `cards_remaining` —
replacing the former cycle counters. The same fields SHALL appear in
player-specific race status data (`BoostAvailability` inside
`PlayerSpecificData`, also returned by `/submit-action` and `/pit` responses)
and in the performance preview's `boost_cycle_info`. `BoostHand`
serialization SHALL default `tyre_type` and `pit_stops_completed` when absent
so pre-tyre persisted races still deserialize.

#### Scenario: Availability reflects consumption

- GIVEN a Medium participant who has spent one value-4 card
- WHEN boost availability is requested
- THEN `hand_state` reports `"4": 0` (and `"2": 2`, `"3": 2`),
  `cards_remaining` = 4, and `available_cards` = `[0, 2, 3]`

### Requirement: Boost impact preview

Boost previews SHALL cover every boost value 0-4
(`BoostHandManager::get_boost_availability` and the performance-preview
endpoint) — each option flagged with `is_available` from the hand — and
SHALL compute the predicted
final value with the additive model used by lap resolution: `final =
min(base_performance, sector.max_value) + boost_value`, plus a movement
probability of `MoveDown` when final < sector min, `MoveUp` when final >
sector max, otherwise `Stay`.

#### Scenario: Preview includes unavailable cards flagged

- GIVEN a participant whose value-4 card is spent
- WHEN the boost impact preview is generated
- THEN it contains 5 options (boosts 0-4), boost 4 has `is_available` =
  false, and each option's predicted value equals the capped base plus the
  boost value

### Requirement: Pool rules apply uniformly to AI participants

AI participants SHALL play by the same pool rules as humans: their boost plays
are consumed through the same `use_boost_card` path and recorded in
`boost_usage_history` with the same pit-segment semantics, they only ever play
values available in their hand (0 always included), and an AI pit refills the
pool via the same `refill` and plays the turn as a free boost-0 move
(`Race::enqueue_ai_actions`). (Which action the AI chooses is out of scope —
see ai-opponents.)

#### Scenario: AI card consumption is tracked

- GIVEN a solo race in progress
- WHEN an AI participant's turn is enqueued with a boost card
- THEN that AI's hand decrements exactly like a human play and a
  `BoostUsageRecord` is appended for it

## Verification

- `.claude/scripts/be.ps1 test-fast` — domain unit tests: tyre pools and hand
  init (`test_tyre_pools`, `test_boost_hand_initialization` — Tyre-defined
  boost pools), free boost 0 (`test_validate_boost_zero_always_available`,
  `test_use_boost_card_no_auto_replenish` — Boost 0 is the free
  always-available move, Card consumption without auto-replenish), duplicates
  (`test_use_boost_card_depletes_duplicate`), refill/tyre swap
  (`test_boost_hand_refill_changes_tyre` — Pit stop refills the pool),
  validation errors (`test_validate_boost_selection_*` — Boost selection
  validation), availability payloads (`test_get_boost_availability*` — Boost
  state visibility, Boost impact preview), pit-segment history/summaries
  (Boost usage history), and the solo pit turn-resolution regression
  (`pit_resolves_turn_in_solo_mode` — Pit stop refills the pool, Pool rules
  apply uniformly to AI participants).
- `.claude/scripts/be.ps1 check --all-targets --all-features` — everything
  compiles (all requirements).
- E2E (backend running without MongoDB, PowerShell `Invoke-WebRequest`):
  create a solo race per tyre via `POST /api/v1/races/solo` with
  `tyre_type` Soft/Medium/Hard and once omitted, then `GET
  .../boost-availability` shows the matching pool and Medium default
  (Tyre-defined boost pools, Tyre selection at race entry, Boost state
  visibility).
- E2E depletion loop: submit boost actions via `POST
  .../submit-action` until the pool is empty → `available_cards` = `[0]` and
  further boosts 1-4 return 400 `BOOST_CARD_NOT_AVAILABLE`; boost 0 still
  succeeds (Card consumption without auto-replenish, Boost 0 is the free
  always-available move, Boost selection validation).
- E2E pit: `POST .../pit` with `new_tyre` → turn resolves as a boost-0 lap and
  `boost-availability` then shows the new tyre's full pool with
  `pit_stops_completed` = 1; `.../lap-history` groups the usage into pit
  segments (Pit stop refills the pool and costs the turn, Boost usage history
  per pit segment).

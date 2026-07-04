# ai-opponents

## Purpose

Computer-controlled opponents for solo mode: seeded bot players, the AI
boost/pit/tyre decision logic, and server-side enqueueing of AI turns so a solo
race never waits on an empty seat. Sources of truth:
`rust-backend/src/domain/ai_player.rs` (decision logic),
`rust-backend/src/domain/race.rs` (`is_ai`, `add_ai_participant*`,
`enqueue_ai_actions`), and the solo paths in `rust-backend/src/routes/races.rs`
(`seed_solo_bots`, `create_solo_race`, `resolve_human_turn`,
`drive_ai_only_turns`). Race lifecycle itself is the `race-engine` capability;
boost cards, tyre pools, and preview/resolution parity are the `boost-system`
capability (which shares `ai_player::classify_movement` as its movement
classifier).

## Requirements

### Requirement: AI participant flag

`RaceParticipant` SHALL carry an `is_ai` boolean that defaults to `false` on
deserialization (`#[serde(default)]`), so pre-existing serialized races load as
all-human. `Race` SHALL expose AI-specific join methods (`add_ai_participant`,
`add_ai_participant_with_tyre`) that reuse the same participant-construction
path as the human join methods, differing only in setting `is_ai = true`.

#### Scenario: AI seat is distinguishable from a human seat

- GIVEN a race with one human participant and one participant added via
  `add_ai_participant`
- WHEN the participants are inspected
- THEN exactly the AI-added participant has `is_ai = true`

#### Scenario: Legacy race data deserializes as human

- GIVEN a serialized `RaceParticipant` without an `is_ai` field
- WHEN it is deserialized
- THEN `is_ai` is `false`

### Requirement: Seeded bot players

At startup the backend SHALL seed `SOLO_BOT_COUNT` (currently 2) bot players
(`seed_solo_bots`, invoked from `startup.rs`) into the in-memory player
repository, each with a complete, pre-equipped car built by the same
`create_starter_assets` pipeline as human registration, so bot stats resolve
through `build_car_data_map` with no special-casing. Seeding SHALL be
idempotent (repeat calls are no-ops) and SHALL record each bot's racing
identity (player, car, primary pilot) in the `SOLO_BOTS` registry. Bots SHALL
be varied by assigning each a different primary pilot from its car's roster
(index `i % pilot_count`); their car builds are otherwise identical starter
builds.

#### Scenario: Bots available after startup

- GIVEN a freshly started backend
- WHEN a solo race is created
- THEN the seeded bots join as AI participants and their car data resolves
  through the normal car-data pipeline

#### Scenario: Seeding twice does not duplicate bots

- GIVEN `seed_solo_bots` has already populated the registry
- WHEN it is called again
- THEN the registry is unchanged

### Requirement: Deterministic balanced boost decision

`ai_player::choose_boost` SHALL pick an AI's boost card as a pure,
deterministic function of (car data, boost hand, sector, lap characteristic) —
no RNG — using the additive performance model that lap resolution uses
(`final = min(engine + body + pilot, sector.max_value) + boost`), selecting
straight vs curve component values per the current lap characteristic exactly
as `calculate_performance_with_car_data` does. It SHALL only ever return a card
present in `boost_hand.get_available_cards()`, chosen by the balanced profile:

1. the smallest available card whose predicted movement is `MoveUp`, else
2. when the capped base is below the sector floor, the smallest available card
   that avoids `MoveDown`, else
3. the smallest available card (conserve the hand).

#### Scenario: Smallest advancing card is chosen

- GIVEN a sector ceiling the capped base already reaches, and a fresh Medium
  hand whose smallest card is 2
- WHEN `choose_boost` runs
- THEN it returns 2 (the smallest card that yields `MoveUp`), not a higher card

#### Scenario: Falls back to avoiding a drop

- GIVEN a capped base below the sector floor and no available card that reaches
  `MoveUp`
- WHEN `choose_boost` runs
- THEN it returns the smallest available card whose result is at least `Stay`

#### Scenario: Conserves when nothing is gained

- GIVEN a base safely inside the sector band where no available card can reach
  `MoveUp`
- WHEN `choose_boost` runs
- THEN it returns the smallest available card

#### Scenario: Never plays a spent card

- GIVEN any combination of already-used cards in the hand
- WHEN `choose_boost` runs
- THEN the returned value is one of the currently available cards

### Requirement: AI pit decision

`ai_player::decide_ai_action` SHALL decide the AI's whole turn as a pure
function returning `AiTurnAction::Boost(n)` or `AiTurnAction::Pit`. While the
boost pool has cards (`cards_remaining > 0`) it SHALL delegate to
`choose_boost`. When the pool is empty (only the free boost 0 remains) it SHALL
pit only when both hold:

- `laps_remaining > 1` — there is a future lap to spend the refilled cards on
  (a pit costs the current lap as a free boost-0 move), and
- a refilled card could change an outcome: the tyre's strongest initial-pool
  card would push the capped base above the sector ceiling, or the capped base
  is below the sector floor (cards could rescue a drop).

Otherwise it SHALL take the free boost-0 move.

#### Scenario: Pits when empty and a card would help

- GIVEN an empty pool, a future lap remaining, and a sector where base plus the
  tyre's strongest card exceeds the ceiling (or base is below the floor)
- WHEN `decide_ai_action` runs
- THEN it returns `Pit`

#### Scenario: Never pits on the final lap

- GIVEN an empty pool on the last lap (`laps_remaining == 1`)
- WHEN `decide_ai_action` runs
- THEN it returns `Boost(0)` even if a refilled card would have helped

#### Scenario: No pointless pit

- GIVEN an empty pool where the base sits safely inside the sector band and
  even the strongest refilled card could not clear the ceiling
- WHEN `decide_ai_action` runs
- THEN it returns `Boost(0)`

### Requirement: AI turn enqueueing

`Race::enqueue_ai_actions` SHALL enqueue an action for every AI participant
that is not finished and has not already acted this turn, mirroring a human
submission: the chosen card is consumed through `BoostHandManager` and a
`BoostUsageRecord` is appended before the action is pushed into
`pending_actions`. An AI `Pit` decision SHALL refill the pool from the current
tyre and resolve the turn as a free boost-0 move. Participants whose car data
is missing from the map SHALL be skipped so a resolution failure never stalls
the turn. Both human turn endpoints — boost (`/submit-action`) and pit
(`/pit`) — SHALL funnel through the single `resolve_human_turn` path, which
enqueues all AI opponents after recording the human's action and processes the
lap once every active participant has acted, so a solo turn resolves
immediately with no waiting-for-players stall.

#### Scenario: Human boost submission resolves the solo turn

- GIVEN a solo race with one human and two active AI participants
- WHEN the human submits a boost via `/submit-action`
- THEN actions for both AI participants are enqueued, their cards are consumed
  and recorded, and the lap is processed in the same request

#### Scenario: Human pit submission also resolves the turn

- GIVEN a solo race where the human pits via `/pit`
- WHEN the pit is recorded
- THEN the AI opponents are enqueued and the lap resolves (regression guard for
  the solo-pit deadlock)

#### Scenario: Only AI seats are auto-filled

- GIVEN a race containing human and AI participants
- WHEN `enqueue_ai_actions` runs
- THEN only AI participants gain pending actions; human seats are untouched

### Requirement: Solo race bootstrap endpoint

`POST /races/solo` SHALL create a ready-to-play solo race from a single human
player UUID: resolve the player's first complete car and its primary pilot
(404 when absent), build the fixed 4-sector "Solo Circuit" track (ceilings
tuned so a starter-grade car always advances with any boost >= 1), create a
5-lap race, add the human with their requested starting tyre (`tyre_type`,
default Medium), add every seeded bot as an AI participant with cycling
Soft/Medium/Hard tyres so the grid is not homogeneous, normalize the grid so
all participants start at sector 0 in stable order, set the race `InProgress`
at lap 1, and return 201 with the race. The frontend SHALL expose a Solo Race
entry point (`GameLobby.tsx` via `raceAPI.createSoloRace`) that calls this
endpoint and routes into the existing race interface.

#### Scenario: One call yields a running solo race

- GIVEN a registered player with a complete car
- WHEN `POST /races/solo` is called with the player's UUID
- THEN the response is 201 with an `InProgress` race containing the human plus
  the seeded AI participants, all placed at the start sector

#### Scenario: AI grid gets varied tyres

- GIVEN the seeded bots join a solo race
- WHEN their boost hands are inspected
- THEN their tyre types cycle through Soft, Medium, Hard in seat order

#### Scenario: No complete car

- GIVEN a player without a complete car
- WHEN `POST /races/solo` is called
- THEN the response is 404 and no race is created

### Requirement: AI-only auto-advance

The backend SHALL drive AI-only turns to completion: WHEN, after a lap
resolves, every remaining active (non-finished) participant is AI — e.g. the
human has already finished — it SHALL keep synthesizing and processing full AI
turns (`drive_ai_only_turns`) until the
race finishes or a human is again required, so a solo race always reaches
`Finished` with a final ranking that includes the human and the AI cars. The
loop SHALL be bounded as a safety net (currently 1000 turns and a 10s
wall-clock deadline) and leave the race `InProgress` rather than hang if a
bound trips.

#### Scenario: Race finishes after the human does

- GIVEN a solo race in which the human finishes while AI cars are still racing
- WHEN the human's final turn resolves
- THEN the backend drives the remaining AI-only turns in the same request and
  the race reaches `Finished` with a complete final ranking

#### Scenario: Auto-advance never runs unbounded

- GIVEN an AI-only continuation that fails to make progress
- WHEN the turn cap or wall-clock deadline is reached
- THEN the loop exits with a warning and the race is left `InProgress`

## Verification

- `.claude/scripts/be.ps1 test-fast` — `domain::ai_player` unit tests
  (Deterministic balanced boost decision: smallest-MoveUp, anti-MoveDown,
  conserve, never-unavailable, ceiling-cap, determinism, curve components; AI
  pit decision: pits-to-move-up, pits-to-rescue, no final-lap pit, no pointless
  pit), `domain::race` tests `enqueue_ai_actions_fills_only_ai_seats` and
  `solo_race_runs_to_completion` (AI turn enqueueing, AI-only auto-advance),
  and `routes::races::turn_resolution_tests` including
  `pit_resolves_turn_in_solo_mode` (AI turn enqueueing — pit regression).
- `.claude/scripts/be.ps1 check --all-targets --all-features` — everything
  compiles (all requirements).
- `.claude/scripts/fe.ps1 npx tsc --noEmit` and
  `.claude/scripts/fe.ps1 npm run test -- --run` — frontend solo entry point
  and `raceAPI.solo.test.ts` (Solo race bootstrap endpoint).
- End-to-end browser race: start the backend (`cargo run --bin rust-backend`
  via PowerShell, no Mongo needed) and the frontend dev server, launch a Solo
  Race from the lobby, play every turn to completion — each lap resolves
  immediately with no waiting-for-players stall, and the race reaches
  `Finished` with a final ranking and no console/network errors (Solo race
  bootstrap endpoint, AI turn enqueueing, AI-only auto-advance).

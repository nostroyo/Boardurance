# race-engine

## Purpose

The race lifecycle and API: race creation (standard and solo), player
registration, status/phase reporting, simultaneous turn/lap resolution, sector
movement, standings, and the player-scoped race view endpoints. Sources of
truth: `rust-backend/src/domain/race.rs` (race/turn domain model),
`rust-backend/src/routes/races.rs` (all `/api/v1/races*` handlers and the
process-global `RACE_STORE`), `rust-backend/src/app_state.rs` (the
`AppState` the turn routes read car data from). Boost-card/tyre pool mechanics
are specified in `boost-system`, AI decision logic in `ai-opponents`,
admin-only endpoints in `admin-management`, and the web client in `race-ui`;
this spec covers the core turn-resolution flow those capabilities plug into.

## Requirements

### Requirement: Races live in the process-global race store

All race state SHALL be held in the process-global
`RACE_STORE: LazyLock<Mutex<HashMap<Uuid, Race>>>` static in
`routes/races.rs`. Every race endpoint SHALL read and write races through this
store (the `Database` handler parameters are unused for race data), so each
read-modify-write of a race is serialized by the store's mutex and race state
does not survive a process restart.

#### Scenario: Race state is process-local

- GIVEN a race created through the API
- WHEN the backend process restarts
- THEN `GET /api/v1/races/{race_uuid}` returns 404 for that race

#### Scenario: Concurrent submissions stay consistent

- GIVEN two players submitting actions for the same race at the same time
- WHEN both requests are processed
- THEN each read-modify-write happens under the store mutex and both actions
  are recorded without losing either

### Requirement: Race creation auto-starts the race

`POST /api/v1/races` SHALL create a race from a name, track name, sector list
(id, name, min_value, max_value, optional slot_capacity, sector_type) and
`total_laps`, validate the track via `Track::new`, and immediately set the
race to `InProgress` with `current_lap = 1` and a `Straight` lap
characteristic (no manual start step is required). It SHALL respond
`201 Created` with the race, or `400` for an invalid track configuration.

#### Scenario: Created race is already in progress

- GIVEN a valid `CreateRaceRequest`
- WHEN `POST /api/v1/races` is called
- THEN the response is 201 and the returned race has status `InProgress`,
  `current_lap` 1, and lap characteristic `Straight`

### Requirement: Solo race bootstrap

`POST /api/v1/races/solo` SHALL create and start a solo race for a human
player: it requires only `player_uuid` (plus an optional starting
`tyre_type`, default Medium), resolves the player's first complete car and
its primary pilot, builds the fixed 4-sector "Solo Circuit" track
(Start/Straight/Curve/Finish, all with unlimited slots), creates a 5-lap race,
adds the human plus the startup-seeded AI opponents (with cycling tyre
strategies), places every participant in sector 0 with 0-based grid positions,
and stores the race already `InProgress`. It SHALL return `201` with the
race, `400` for an invalid player UUID, and `404` when the player or a
complete car cannot be found.

#### Scenario: Solo race is ready to play immediately

- GIVEN a player with at least one complete car
- WHEN `POST /api/v1/races/solo` is called with that player's UUID
- THEN the response is 201, the race is `InProgress` with the human and the
  seeded AI opponents all in sector 0, and the human can submit a turn action
  right away

#### Scenario: Player without a complete car

- GIVEN a player whose cars are missing an engine, body, or pilot
- WHEN `POST /api/v1/races/solo` is called
- THEN the response is 404

### Requirement: Race listing and retrieval

The API SHALL expose `GET /api/v1/races` (all races in the store),
`GET /api/v1/races/{race_uuid}` (full race document), and
`GET /api/v1/races/{race_uuid}/status` (the bare `RaceStatus` enum:
`Waiting`, `InProgress`, `Finished`, or `Cancelled`). Handlers SHALL return
`400` for a malformed UUID and `404` for an unknown race.

#### Scenario: Fetch an existing race

- GIVEN a created race
- WHEN `GET /api/v1/races/{race_uuid}` is called
- THEN the response is 200 with the race including track, participants, laps,
  and status

#### Scenario: Unknown race

- GIVEN a syntactically valid UUID not present in the store
- WHEN any `/api/v1/races/{race_uuid}...` endpoint is called
- THEN the response is 404

### Requirement: Player registration

`POST /api/v1/races/{race_uuid}/register` SHALL register a player with
mandatory `player_uuid` and `car_uuid` (the pilot is derived from the car;
an optional `tyre_type` selects the starting boost pool). Registration SHALL
validate via `CarValidationService` that the car exists, belongs to the
player, and has engine, body, and pilot components (else `400`). A player may
be added while the race is `Waiting`, or `InProgress` only during lap 1
(late join); registering twice or joining a progressed/finished race is
rejected. New participants get a random qualification sector. On success the
endpoint SHALL return the race progress status plus the player's starting
position (starting sector, position in sector, qualification rank). The
legacy `POST /api/v1/races/{race_uuid}/join` variant SHALL keep working with
an explicit `pilot_uuid` and no car-component validation.

#### Scenario: Successful registration

- GIVEN a `Waiting` or first-lap `InProgress` race and a player with a
  complete car
- WHEN `POST .../register` is called with player and car UUIDs
- THEN the response is 200 with `success: true`, the race status, and the
  player's starting sector/position/qualification rank

#### Scenario: Duplicate registration is rejected

- GIVEN a player already participating in the race
- WHEN they register again
- THEN the response is 409

#### Scenario: Registration after lap 1 is rejected

- GIVEN an `InProgress` race whose `current_lap` is greater than 1
- WHEN a new player attempts to register
- THEN registration fails (the participant is not added)

### Requirement: Race lifecycle states

A race SHALL move through the statuses `Waiting` → `InProgress` →
`Finished` (with `Cancelled` reserved). `POST /api/v1/races/{race_uuid}/start`
SHALL start only a `Waiting` race with at least one participant — setting
`InProgress`, lap 1, `Straight` characteristic, and 0-based grid positions —
and SHALL respond `409` when the race has already started/finished or has no
participants. Status reporting SHALL map `InProgress` to `Ongoing`,
`Cancelled` to an `Error` progress status, and `Finished` to `Finished`.

#### Scenario: Start requires a waiting race with participants

- GIVEN a race that is already `InProgress`
- WHEN `POST .../start` is called
- THEN the response is 409 and the race state is unchanged

### Requirement: One boost, one turn, one lap

The race SHALL advance in processing turns where each participant plays
exactly one boost card per turn and one processed turn equals one lap:
`turns_taken` increments by 1 per processed turn and the displayed
`current_lap` is `min(turns_taken + 1, total_laps)` (kept in sync on every
participant). The race SHALL finish once `turns_taken >= total_laps` (all
participants are then marked finished simultaneously — no car "finishes"
early), with a safety cap (`total_laps × sector_count × 8 + 50` turns) that
guarantees termination. On completion, finish positions SHALL be assigned by
sorting participants by current sector (higher is better), then position in
sector (lower is better), then accumulated `total_value` (higher is better).

#### Scenario: Race ends after total_laps turns

- GIVEN a solo race with `total_laps = 5`
- WHEN 5 turns have been processed
- THEN the race status is `Finished` and every participant has
  `is_finished = true` and a unique `finish_position` starting at 1

#### Scenario: Lap counter tracks turns

- GIVEN an `InProgress` race that has processed 2 turns
- WHEN the race is read
- THEN `current_lap` is 3 (capped at `total_laps`)

### Requirement: Simultaneous turn resolution

Turn resolution SHALL be simultaneous: each active participant's action is
recorded into `pending_actions` (with its performance calculation and
submission timestamp), and the lap is processed only when every active
(non-finished) participant has submitted. Recording an action SHALL validate,
in the domain, that the race is `InProgress`, the player is a participant,
has not finished, has not already submitted this turn, the boost value is
0-4, and the boost card is available in the player's hand (consuming it on
success). Per-player performance SHALL be computed as
`final = min(engine + body + pilot, sector.max_value) + boost`, where the
engine/body/pilot contributions are the straight or curve values matching the
current lap characteristic and the ceiling is the participant's current
sector's `max_value` (ceiling applied before the boost is added; the boost is
additive — there is no multiplier). While the race remains `InProgress`, a
new random lap characteristic (Straight or Curve) SHALL be drawn after each
processed turn, and pending actions SHALL be cleared for the next turn.

#### Scenario: Action recorded while waiting for others

- GIVEN an `InProgress` race where another active participant has not yet
  submitted
- WHEN a player submits a valid boost action
- THEN the action is staged with its predicted performance and the turn is
  not resolved yet

#### Scenario: Double submission rejected

- GIVEN a player who already submitted an action this turn
- WHEN they submit again before the turn resolves
- THEN the submission is rejected as a conflict

#### Scenario: Sector ceiling caps base before boost

- GIVEN a participant whose engine+body+pilot total exceeds their sector's
  `max_value`
- WHEN their performance is calculated with boost B
- THEN `final_value = sector.max_value + B`

### Requirement: Sector movement is relative standings

Sectors SHALL represent the relative standings between cars (not physical
track position). When a turn is processed, sectors are resolved from highest
(lead) to lowest: within each sector, participants are ranked by this turn's
`final_value` (highest first), and

- only the first-ranked car may move up one sector, and only if its
  `final_value` exceeds the sector's `max_value` and the target sector has a
  free slot (capacity permitting; `slot_capacity: None` is unlimited);
- any car whose `final_value` is below the sector's `min_value` moves down to
  the nearest lower sector with space (sector 0 always fits);
- everything else stays put. A car already in the highest sector holds the
  lead — it does not wrap around or finish early.

After movement, each participant's `total_value` accumulates their
`final_value`, and participants within each sector are re-ranked by
`total_value` (descending) to set `current_position_in_sector`.

#### Scenario: Only the sector leader can advance

- GIVEN two cars in the same sector whose final values both exceed the
  sector's `max_value`
- WHEN the turn is processed
- THEN only the higher-scoring car moves up; the other stays
  (`StayedInSector`)

#### Scenario: Underperforming car drops back

- GIVEN a car whose `final_value` is below its sector's `min_value`
- WHEN the turn is processed
- THEN the car moves down to the nearest lower sector with available capacity

#### Scenario: Leader holds the top sector

- GIVEN a car in the track's highest sector with a final value above its
  ceiling
- WHEN the turn is processed
- THEN the car stays in the top sector (no wrap, no early finish)

### Requirement: Single shared human-turn resolution path

The backend SHALL resolve `POST /api/v1/races/{race_uuid}/submit-action`
(boost play) and `POST /api/v1/races/{race_uuid}/pit` (pit stop, specified in
`boost-system`) through one shared path (`resolve_human_turn`): record the
human's card-consuming action, enqueue an action for every active AI
participant that has not yet acted this turn, process the lap once all active
participants have acted, and then auto-drive any AI-only turns (e.g. after
the human finishes) until the race completes — bounded by 1000 turns and a
10-second wall clock. `submit-action` SHALL validate boost 0-4 (else `400`)
and SHALL return `turn_phase: "WaitingForPlayers"` with the submitted count
when the turn is still waiting on other humans, or
`turn_phase: "TurnProcessed"` once the lap resolved; conflicts (already
submitted, race not in progress) SHALL return `409`.

#### Scenario: Solo submission resolves the whole turn

- GIVEN a solo race where the only human submits a valid boost
- WHEN `POST .../submit-action` is called
- THEN the AI opponents are enqueued, the lap is processed in the same
  request, and the response has `turn_phase: "TurnProcessed"`

#### Scenario: Race completes without further human input

- GIVEN a solo race in which the human's final-lap action has been submitted
- WHEN the turn resolves
- THEN any remaining AI-only turns are driven automatically until the race is
  `Finished`

### Requirement: Alternate lap-processing endpoints

The API SHALL also expose `POST /api/v1/races/{race_uuid}/apply-lap`
(per-player action with `player_uuid`, `car_uuid`, `boost_value`; validates
the car via `CarValidationService` and the boost card before processing, and
on success returns the same `DetailedRaceStatusResponse` shape as the
status-detailed endpoint, including the caller's player-specific data) and
`POST /api/v1/races/{race_uuid}/turn` (batch: a full set of `LapAction`s
processed in one call using real car data resolved from the in-memory player
repository, with a fixed placeholder performance fallback when resolution
fails; returns the `LapResult` and updated race status). `apply-lap` SHALL
return structured boost-card errors (`400` with error code and available
cards) and `409` for race-state conflicts; `turn` SHALL return `409` when
the race is not in progress or an active participant's action is missing.

#### Scenario: apply-lap returns status-shaped response

- GIVEN an `InProgress` race and a valid player action
- WHEN `POST .../apply-lap` is called
- THEN the response is the detailed status document (race progress, track
  situation, metadata) with the caller's player data included

#### Scenario: Batch turn with a missing action

- GIVEN an `InProgress` race with two active participants
- WHEN `POST .../turn` is called with only one action
- THEN the response is 409 and no movement is applied

### Requirement: Detailed race status reporting

`GET /api/v1/races/{race_uuid}/status-detailed` SHALL return, in one
document: race progress (`Waiting`/`Ongoing`/`Finished`/`Error` status,
current lap, total laps, lap characteristic, a coarse turn phase, participant
and finished counts); the track situation for every sector (sector id, name,
type, capacity info with occupancy and available slots, min/max performance
thresholds, and the sector's non-finished participants sorted by position);
a leaderboard of non-finished participants ranked by sector (descending) then
position in sector; and race metadata (race/track names, start time when
started). WHEN a `player_uuid` query parameter is supplied, the response
SHALL additionally include that player's data: boost availability, a
performance preview, current position with sector and overall rank, boost
usage history, and cycle summaries. Known limitations SHALL hold as current
behavior: `recent_movements` is always empty, player names are not resolved
(placeholder car names), the player-specific performance preview in THIS
endpoint uses placeholder contributions (base 10), and requesting player data
for a finished player yields `500`.

#### Scenario: Status without player context

- GIVEN an `InProgress` race
- WHEN `GET .../status-detailed` is called with no query parameters
- THEN the response contains race progress, per-sector situation, leaderboard,
  and metadata, and `player_data` is null

#### Scenario: Status with player context

- GIVEN an `InProgress` race and a registered, non-finished player
- WHEN `GET .../status-detailed?player_uuid={uuid}` is called
- THEN `player_data` contains boost availability, position (sector rank and
  overall rank), and boost usage history

### Requirement: Turn phase reporting

`GET /api/v1/races/{race_uuid}/turn-phase` SHALL derive the phase from live
race state: `"Complete"` when the race is not `InProgress`,
`"AllSubmitted"` when every active participant has a pending action, and
`"WaitingForPlayers"` otherwise (no `"Processing"` phase is ever emitted —
resolution is synchronous). The response SHALL include the current lap, total
laps, lap characteristic, the UUIDs of players who have submitted, the UUIDs
still pending, and the count of active players.

#### Scenario: Waiting phase lists pending players

- GIVEN an `InProgress` race where one of two active players has submitted
- WHEN `GET .../turn-phase` is called
- THEN the phase is `WaitingForPlayers`, the submitter is listed in
  `submitted_players`, and the other player in `pending_players`

#### Scenario: Finished race reports Complete

- GIVEN a `Finished` race
- WHEN `GET .../turn-phase` is called
- THEN the phase is `Complete`

### Requirement: Player-scoped race view endpoints

The API SHALL provide per-player read endpoints under
`/api/v1/races/{race_uuid}/players/{player_uuid}/`:

- `car-data`: the participant's full car, engine, body (name, rarity,
  straight/curve values), and pilot (class, rarity, skills breakdown —
  reaction_time, precision, focus, stamina — and straight/curve performance),
  resolved from the in-memory player repository.
- `performance-preview`: the base performance breakdown
  (engine/body/pilot contributions for the current lap characteristic, base
  value, sector ceiling, capped base) and, for each boost card 0-4, its
  availability, the additive `final_value = capped_base + boost`, and a
  movement probability (`MoveUp` when final > sector max, `MoveDown` when
  final < sector min, else `Stay`), plus the current boost pool info.
  Requires the race `InProgress` and the player not finished (else `409`).
- `local-view`: the 5-sector window centered on the player's sector (±2 with
  modulo wrapping over the track), each visible sector's details
  (id, name, min/max values, slot capacity, type, occupancy) and all
  non-finished participants in the window sorted by sector (descending) then
  position.
- `boost-availability` and `lap-history`: the player's boost hand state and
  boost usage history/cycle summaries (payload details specified in
  `boost-system`; note lap-history's per-lap base/final values and sector
  movements are unstored placeholders, and its lap characteristic is
  reconstructed by lap-number parity).

#### Scenario: Performance preview matches lap resolution math

- GIVEN an `InProgress` race and an active participant
- WHEN `GET .../performance-preview` is called
- THEN every boost option's `final_value` equals
  `min(base_value, sector.max_value) + boost_value` — the same formula turn
  resolution uses

#### Scenario: Local view wraps at track boundaries

- GIVEN a participant in sector 0 of an N-sector track
- WHEN `GET .../local-view` is called
- THEN the visible sector ids are sectors -2..+2 modulo N, centered on 0

#### Scenario: Preview refused when race not in progress

- GIVEN a race that is `Waiting` or `Finished`
- WHEN `GET .../performance-preview` is called
- THEN the response is 409 `RACE_NOT_IN_PROGRESS`

### Requirement: Consistent error contract

Race endpoints SHALL use consistent HTTP statuses and, on the player-scoped
and lap-action endpoints, a structured JSON error body with a machine-readable
code: `400` `INVALID_UUID` for malformed UUIDs, `400`
`CAR_VALIDATION_FAILED` for unresolvable car components, `404`
`RACE_NOT_FOUND` / `PLAYER_NOT_FOUND` ("Race not found" / "Player not found
in race"), `409` `RACE_NOT_IN_PROGRESS` ("Race is not in progress"), `409`
`PLAYER_FINISHED` ("Player has already finished the race"), and boost-card
errors as `400` with the available cards included. All race endpoints SHALL
be registered in the OpenAPI/utoipa documentation with request/response
schemas.

#### Scenario: Invalid UUID rejected uniformly

- GIVEN a malformed UUID in the path of any player-scoped race endpoint
- WHEN the endpoint is called
- THEN the response is 400 with error code `INVALID_UUID`

#### Scenario: Player not in race

- GIVEN a valid race and a player UUID that is not a participant
- WHEN a player-scoped endpoint is called
- THEN the response is 404 with error code `PLAYER_NOT_FOUND`

## Verification

- `.claude/scripts/be.ps1 test-fast` — domain and route unit tests: race
  creation/start/participant rules, `turn_resolution_tests` in
  `routes/races.rs`, movement and completion logic in `domain/race.rs`
  (Race creation auto-starts, Player registration, Race lifecycle states,
  One boost/one turn/one lap, Simultaneous turn resolution, Sector movement,
  Single shared human-turn resolution path).
- `.claude/scripts/be.ps1 check --all-targets --all-features` — everything
  compiles, OpenAPI schema registrations included (all requirements,
  Consistent error contract).
- Degraded-mode e2e (no MongoDB, backend via
  `cargo run --bin rust-backend` in PowerShell; poll `GET /api/v1/races`
  until it answers — races are in-memory so no DB is needed):
  1. `POST /api/v1/auth/register` a user → seeded assets; `POST
     /api/v1/races/solo` with the player UUID → 201, race `InProgress`
     (Solo race bootstrap, Races live in the process-global race store).
  2. `GET /api/v1/races/{uuid}/turn-phase` → `WaitingForPlayers` with the
     human pending (Turn phase reporting).
  3. `GET .../players/{player}/car-data`, `performance-preview`,
     `local-view` → 200 with formula-consistent values (Player-scoped race
     view endpoints).
  4. `POST .../submit-action` with boost 0-4 → `TurnProcessed`; repeat 5
     turns total → `GET /api/v1/races/{uuid}/status` returns `Finished`
     and every participant has a `finish_position` (Single shared human-turn
     resolution path, One boost/one turn/one lap, Sector movement).
  5. `GET .../status-detailed?player_uuid=...` before the final turn → 200
     with player data; malformed UUID → 400; unknown race → 404
     (Detailed race status reporting, Consistent error contract).
  6. Restart the backend → `GET /api/v1/races` no longer lists the race
     (Races live in the process-global race store).

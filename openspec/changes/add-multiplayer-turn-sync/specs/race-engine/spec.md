# race-engine — delta for add-multiplayer-turn-sync

## ADDED Requirements

### Requirement: Per-turn deadline with auto-play for absent players

Races SHALL carry an optional `turn_timeout_secs` (set at creation; solo races
have none). For a race that is `InProgress` with two or more human
participants and a configured timeout, the system SHALL arm a turn deadline of
`now + turn_timeout_secs` lazily — on the first turn-phase read or action
submission of the turn — and SHALL re-arm it after each processed turn.
Enforcement SHALL also be lazy: on every turn-phase read and every action
submission, IF the armed deadline has passed and the turn is unresolved, the
system SHALL generate an action for every pending human participant using the
AI decision logic (identical card consumption and boost-usage-history
recording as a manual submission) and resolve the turn. A submission arriving
after the deadline but before enforcement SHALL record the caller's own action
first, so only truly absent players are auto-played. Auto-play SHALL apply
per turn only: an auto-played player's later submissions are accepted normally
(auto-pilot forever, no forfeit). Solo races (fewer than two humans) SHALL
never arm a deadline and keep resolving the turn within the submit request.

#### Scenario: Absent player is auto-played on expiry

- **WHEN** the turn deadline has passed with one of two humans not submitted
  and any turn-phase poll or submission for the race is handled
- **THEN** the missing player receives an AI-generated action that consumes a
  real boost card and appends to their boost-usage history, and the turn
  resolves with `turns_taken` incremented by one

#### Scenario: Everyone absent still advances the race

- **WHEN** no player has submitted and the deadline has passed at the next
  turn-phase poll
- **THEN** all active humans are auto-played and the turn resolves

#### Scenario: Late submission wins over auto-play

- **WHEN** a player's submission arrives after the deadline but before any
  enforcement ran
- **THEN** that player's own action is recorded and only the remaining absent
  players are auto-played

#### Scenario: Enforcement is idempotent

- **WHEN** two turn-phase polls for an expired turn are processed concurrently
- **THEN** the turn resolves exactly once and the second enforcement observes
  an already-resolved turn and changes nothing

#### Scenario: Auto-played player resumes control

- **WHEN** a player whose previous turn was auto-played submits an action in
  the following turn
- **THEN** the submission is accepted exactly like any other

#### Scenario: Solo race never arms a deadline

- **WHEN** a solo race (one human plus AI opponents) is polled or played
- **THEN** no deadline is armed and each human submission resolves the turn in
  the same request

## MODIFIED Requirements

### Requirement: Races live in the process-global race store

All race state SHALL be held in the process-global
`RACE_STORE: LazyLock<Mutex<HashMap<Uuid, Race>>>` static in
`routes/races.rs`. Every race endpoint SHALL read and write races through this
store (the `Database` handler parameters are unused for race data), so race
state does not survive a process restart. An entire turn resolution — record
the caller's action, enqueue AI actions, auto-fill expired seats, process the
lap — SHALL execute as one critical section under the store mutex (mutating
the stored race in place through a closure), so concurrent submissions or
polls can never interleave inside a turn resolution and no update is lost.

#### Scenario: Race state is process-local

- GIVEN a race created through the API
- WHEN the backend process restarts
- THEN `GET /api/v1/races/{race_uuid}` returns 404 for that race

#### Scenario: Concurrent submissions stay consistent

- GIVEN two players submitting actions for the same race at the same time
- WHEN both requests are processed
- THEN both actions are recorded, both boost cards are consumed, and the turn
  resolves exactly once (`turns_taken` increases by exactly one)

### Requirement: Race creation auto-starts the race

`POST /api/v1/races` SHALL create a race from a name, track name, sector list
(id, name, min_value, max_value, optional slot_capacity, sector_type),
`total_laps`, and an optional `turn_timeout_secs` (default 60 when omitted;
values outside 5–600 rejected with `400`), validate the track via
`Track::new`, and immediately set the race to `InProgress` with
`current_lap = 1` and a `Straight` lap characteristic (no manual start step
is required). It SHALL respond `201 Created` with the race, or `400` for an
invalid track configuration.

#### Scenario: Created race is already in progress

- GIVEN a valid `CreateRaceRequest`
- WHEN `POST /api/v1/races` is called
- THEN the response is 201 and the returned race has status `InProgress`,
  `current_lap` 1, and lap characteristic `Straight`

#### Scenario: Out-of-range turn timeout rejected

- **WHEN** `POST /api/v1/races` is called with `turn_timeout_secs: 3`
- **THEN** the response is `400` and no race is created

### Requirement: Turn phase reporting

`GET /api/v1/races/{race_uuid}/turn-phase` SHALL first apply lazy deadline
enforcement (arming an unarmed deadline, auto-playing an expired turn), then
derive the phase from live race state: `"Complete"` when the race is not
`InProgress`, `"AllSubmitted"` when every active participant has a pending
action, and `"WaitingForPlayers"` otherwise (no `"Processing"` phase is ever
emitted — resolution is synchronous). The response SHALL include the current
lap, total laps, lap characteristic, the UUIDs of players who have submitted,
the UUIDs still pending, the count of active players, `turns_taken` (the
turn-advancement counter), `turn_deadline` (nullable epoch seconds), and
`seconds_remaining` (nullable, clamped to ≥ 0). Action-submission responses
SHALL likewise report `turns_taken` as of after the submission, so a client
can baseline it for turn-advancement detection.

#### Scenario: Waiting phase lists pending players

- GIVEN an `InProgress` race where one of two active players has submitted
- WHEN `GET .../turn-phase` is called
- THEN the phase is `WaitingForPlayers`, the submitter is listed in
  `submitted_players`, and the other player in `pending_players`

#### Scenario: Finished race reports Complete

- GIVEN a `Finished` race
- WHEN `GET .../turn-phase` is called
- THEN the phase is `Complete`

#### Scenario: Turn advancement is observable to a waiting client

- **WHEN** a client that submitted at `turns_taken = N` polls after the last
  pending player submitted
- **THEN** the response reports `turns_taken = N + 1` and the countdown fields
  for the newly armed deadline

#### Scenario: Polling an expired turn resolves it

- **WHEN** `GET .../turn-phase` is called for a multiplayer race whose
  deadline has passed with pending players
- **THEN** the pending players are auto-played, the turn resolves, and the
  response already reflects the incremented `turns_taken`

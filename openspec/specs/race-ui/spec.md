# race-ui

## Purpose

The frontend race experience: the lobby (listing, creating, and joining races),
the in-race interface (track view, standings, boost controls, tyre/pit UI,
turn flow), and race results. Backend API semantics (turn resolution, boost
pools, sector math) are owned by the race-engine/boost-system specs — this
spec covers what the user sees and can do in the client. Sources of truth:
`empty-project/src/components/GameLobby.tsx`, `RacePlayPage.tsx`, `App.tsx`,
`empty-project/src/components/player-game-interface/` (`RaceContainer.tsx`,
`RaceInterface.tsx`, `TrackDisplayRedesign.tsx`, `SectorGrid.tsx`,
`PositionSlot.tsx`, `CarSprite.tsx`, `RaceStatusPanel.tsx`,
`BoostControlPanel.tsx`, `RaceCompletionScreen.tsx`, `RaceLoadingState.tsx`),
`empty-project/src/hooks/useRacePolling.ts`, and
`empty-project/src/services/raceAPI.ts`.

## Requirements

### Requirement: Lobby race list

The lobby (`GameLobby`, route `/game`) SHALL list all races fetched from
`GET /api/v1/races`, showing for each race its name, a color-coded status
badge (`Waiting` / `InProgress` / `Finished`), the participant count, and lap
progress (`current_lap / total_laps`). The action offered per race SHALL
depend on the user's relationship to it: participants of an `InProgress` race
get **Enter Race**, participants of other races get **View Race**,
non-participants get **Join Race** while the race is `Waiting` or is
`InProgress` with open slots, and **View Details** otherwise. Joining SHALL
use the player's first car that has 3 pilots assigned and surface an error
message when no such car exists.

#### Scenario: Participant of a running race

- GIVEN a race with status `InProgress` in which the user participates
- WHEN the lobby renders
- THEN that race shows an **Enter Race** link to `/races/{uuid}/play`

#### Scenario: Joining without a fully crewed car

- GIVEN the user has no car with 3 pilots assigned
- WHEN the user clicks **Join Race**
- THEN an error message tells them to assign pilots to their car first

### Requirement: Solo race creation with starting tyre

The lobby SHALL provide a solo-race flow: a **Starting tyre** selector
offering `Soft`, `Medium` (default), and `Hard`, and a **Race vs AI (Solo)**
button that calls `POST /api/v1/races/solo` with the chosen `tyre_type` and
navigates to `/races/{uuid}/play` on success. WHEN the solo-race call fails
with 404 or 401 (stale session after an in-memory backend restart), the
client SHALL log the user out and redirect to `/login` with an explanatory
message instead of dead-ending.

#### Scenario: Start a solo race with a chosen tyre

- GIVEN an authenticated user in the lobby
- WHEN they select the `Hard` tyre and click **Race vs AI (Solo)**
- THEN a solo race is created with `tyre_type: "Hard"` and the browser
  navigates to the race play page

#### Scenario: Stale session on solo start

- GIVEN the backend restarted and no longer knows the player
- WHEN starting a solo race returns 404
- THEN the session is cleared and the user lands on `/login` with a
  "session expired" message

### Requirement: Race initialization and loading feedback

The client SHALL, on entering `/races/{raceUuid}/play` (`RacePlayPage` →
`RaceContainer`), fetch car data, local view, and turn phase before rendering
the race UI, displaying a progress loading screen with staged messages while
fetching. Boost availability, lap history, and the performance preview SHALL
be fetched as non-critical follow-ups whose failure does not block the UI.
Unauthenticated users SHALL be redirected to `/login`.

#### Scenario: Initial load

- GIVEN a participant opens the race play page
- WHEN the race data is being fetched
- THEN a loading screen with progress and skeleton is shown, and the race
  interface appears once car data, local view, and turn phase have loaded

### Requirement: Race error handling

WHEN a race API call fails, the race UI SHALL show a user-friendly error
panel with the message, a **Retry** button for retryable errors (re-running
initialization), and a **Dismiss** button. `RacePlayPage` SHALL navigate back
to the lobby with an explanatory error message when the error indicates the
race was not found (404) or the player is not authorized for it (403);
transient polling errors SHALL be retried silently without interrupting the
player. WHEN turn processing exceeds the polling budget, the UI SHALL surface
a "taking longer than expected" error suggesting a page refresh.

#### Scenario: Race not found

- GIVEN a race play URL for a race that no longer exists
- WHEN initialization fails with a race-not-found error
- THEN the user is returned to the lobby with a "Race not found" message

#### Scenario: Retryable failure

- GIVEN a transient failure while initializing the race
- WHEN the error panel is shown
- THEN clicking **Retry** re-runs race initialization

### Requirement: Bird's-eye local track view

The race UI SHALL render a bird's-eye local track view
(`TrackDisplayRedesign`) limited to the player's current sector plus at most
2 sectors ahead and 2 behind, ordered with the leading sector first and
auto-scrolled so the player's sector is centered. Each sector SHALL render as
a grid (`SectorGrid`) of 5 position slots (backend positions 0-4 mapped to UI
slots 1-5), with the player's sector visually emphasized, and per-sector
capacity indicators (occupancy vs. `slot_capacity`, `∞` for unlimited) and
value ranges displayed. Sectors SHALL be numbered for display from the lead:
displayed number = `total_sectors - sector.id`, so the leading sector is
"Sector 1".

#### Scenario: Local view shows at most 5 sectors

- GIVEN a race with more than 5 sectors
- WHEN the track view renders
- THEN only the player's sector ±2 are shown, lead sector first, and the
  player's sector is highlighted and centered

#### Scenario: Cars occupy distinct slots

- GIVEN two participants in the same sector at positions 0 and 1
- WHEN the sector grid renders
- THEN they appear in UI slots 1 and 2 respectively, never overlapping

### Requirement: Car sprites

Participants SHALL be rendered as 8-bit pixel-art car sprites (`CarSprite`)
in their position slots. Each participant SHALL get a deterministic color
palette derived from a hash of their player UUID so different players are
visually distinct, and the player's own car SHALL be overridden with a gold
highlight scheme to stand out from opponents.

#### Scenario: Player car is distinct

- GIVEN a sector containing the player and an opponent
- WHEN sprites render
- THEN the player's sprite uses the gold highlight palette and the opponent
  uses a UUID-derived palette

### Requirement: Race status display

The race UI SHALL display the current race state: current lap and total laps,
the lap characteristic (`Straight` or `Curve`), and the turn phase
(`WaitingForPlayers`, `AllSubmitted`, `Processing`, `Complete`) with
color-coded styling, plus the number of active players. A compact "Your Car"
panel SHALL show the player's car, pilot, engine, and body names.

#### Scenario: Status reflects the turn phase

- GIVEN a race in lap 2 of 3 on a `Curve` lap with phase `WaitingForPlayers`
- WHEN the race UI renders
- THEN the status panels show lap 2/3, characteristic `Curve`, and a
  yellow-styled `WaitingForPlayers` phase indicator

### Requirement: Boost selection panel

The boost panel (`BoostControlPanel`) SHALL offer boost buttons for the
values 0-4. Button state SHALL come from the backend's boost availability:
unavailable values are disabled, boost 0 carries a green "Free" badge (it is
always free and consumes no card), and each value 1-4 carries a remaining-
count badge (`×N` from `hand_state`, red when 0). The panel header SHALL show
the currently fitted tyre. Clicking an available button SHALL select it with
immediate visual feedback (highlight plus a transient "Boost N selected"
message); clicking an unavailable one SHALL explain it is not available. WHEN
a selected value has no cards left, the panel SHALL advise pitting to refill
or using the free boost 0.

#### Scenario: Depleted card value

- GIVEN `hand_state` reports 0 cards remaining for value 3
- WHEN the panel renders
- THEN the "3" button is disabled with a red `×0` badge, while boost 0 stays
  enabled with its "Free" badge

#### Scenario: Selecting a boost

- GIVEN the phase is `WaitingForPlayers` and value 2 is available
- WHEN the player clicks the "2" button
- THEN the button highlights as selected and a "Boost 2 selected"
  confirmation appears

### Requirement: Turn validation and submission

A **Validate Turn** button SHALL be enabled only when an available boost is
selected, nothing has been submitted this turn, and the phase is
`WaitingForPlayers`; clicking it SHALL open a confirmation step (warning that
the action cannot be changed) with **Confirm**/**Cancel** before submitting
the action to the backend. After submission the panel SHALL show a submitted
state ("Turn Validated", waiting for other players) and all boost/pit inputs
SHALL be disabled until the next turn. Outside `WaitingForPlayers`, the panel
SHALL state that turn actions are not available.

#### Scenario: Submission requires confirmation

- GIVEN boost 2 is selected during `WaitingForPlayers`
- WHEN the player clicks **Validate Turn** and then **Confirm**
- THEN the action is submitted once and the panel switches to the submitted
  state

#### Scenario: Off-turn controls are inert

- GIVEN the turn phase is `Processing`
- WHEN the panel renders
- THEN boost buttons, Validate Turn, and the pit control are disabled and a
  "Turn actions not available" notice is shown

### Requirement: Turn resolution flow

The client SHALL, after submitting an action whose response reports the turn
as already processed (`TurnProcessed`, the solo-race path), immediately
refresh race data; otherwise it SHALL poll the turn-phase endpoint every 2
seconds (max 60 attempts, exponential backoff on errors, canceled on
unmount) until the turn completes. On turn completion the client SHALL
refresh local view, boost availability, lap history, and performance preview
via the batch endpoint plus the turn phase, and reset the boost selection so
the next turn starts clean.

#### Scenario: Solo turn resolves immediately

- GIVEN a solo race
- WHEN the player submits a boost and the response is `TurnProcessed`
- THEN the track view, lap counter, and boost counts refresh without waiting
  for polling

#### Scenario: Standings update after a turn

- GIVEN a completed turn moved participants between sectors
- WHEN the refreshed local view arrives
- THEN car sprites appear at their new sectors/slots and the used boost's
  remaining count is decremented

### Requirement: Pit stop control

The boost panel SHALL include a pit-stop control: a tyre dropdown
(`Soft`/`Medium`/`Hard`, initialized from the current tyre) and a
"Pit & refill" button, disabled whenever turn interaction is disabled
(already submitted, submitting, or phase is not `WaitingForPlayers`). The UI
SHALL state that pitting refills the boost pool and costs the lap (counts as
boost 0). WHEN the pit action succeeds, the client SHALL clear the boost
selection and refresh all race data so the refilled pool and new tyre are
shown.

#### Scenario: Pit with a tyre change

- GIVEN it is the player's turn and their pool is depleted
- WHEN they pick `Soft` in the pit dropdown and click "Pit & refill"
- THEN the pit request is sent with the new tyre, the lap is consumed, and
  the refreshed panel shows tyre `Soft` with refilled card counts

#### Scenario: Pit is off-turn safe

- GIVEN the player has already submitted this turn
- WHEN the panel renders
- THEN the pit button and tyre dropdown are disabled

### Requirement: Race completion and results

The client SHALL detect race completion by consulting the race object (race
status `Finished` or the player participant's `is_finished`) before each
post-turn refresh, and then SHALL render the completion screen
(`RaceCompletionScreen`) instead of the race UI: final position with a medal
style for the top 3, the player's car/pilot summary, race statistics derived
from lap history (laps, average boost, average performance, best lap), and
**Return to Lobby**, **View Details**, and **Race Again** actions. Returning
to the lobby after completing a race SHALL show a success message with the
final position.

#### Scenario: Finishing the race

- GIVEN the player finishes the race in position 1
- WHEN the final turn resolves
- THEN the completion screen shows `#1` with a gold medal style and offers
  Return to Lobby / View Details / Race Again

### Requirement: Leaving an active race

The race play page SHALL guard against accidental exits: a fixed
**← Return to Lobby** overlay button SHALL open an in-app confirmation dialog
("Leave Race?" with Stay/Leave choices) before navigating away, and a
`beforeunload` handler SHALL trigger the browser's leave-page warning while a
race is open.

#### Scenario: Leave requires confirmation

- GIVEN a player in an active race
- WHEN they click **← Return to Lobby** and then **Leave Race**
- THEN they are navigated to the lobby; choosing **Stay in Race** keeps them
  in the race

## Verification

- `.claude/scripts/fe.ps1 npx tsc --noEmit` — everything type-checks (all
  requirements).
- Targeted vitest runs (the FULL suite can OOM in CI — run per file):
  - `.claude/scripts/fe.ps1 npm run test -- --run src/components/player-game-interface/BoostControlPanel.test.tsx`
    (Boost selection panel, Turn validation and submission, Pit stop control).
  - `.claude/scripts/fe.ps1 npm run test -- --run src/components/player-game-interface/TrackDisplayRedesign.test.tsx src/components/player-game-interface/SectorGrid.test.tsx src/components/player-game-interface/PositionSlot.test.tsx`
    (Bird's-eye local track view).
  - `.claude/scripts/fe.ps1 npm run test -- --run src/components/player-game-interface/CarSprite.test.tsx src/components/player-game-interface/CarSpritePositioning.test.tsx`
    (Car sprites).
  - `.claude/scripts/fe.ps1 npm run test -- --run src/components/player-game-interface/RaceStatusPanel.test.tsx`
    (Race status display).
  - `.claude/scripts/fe.ps1 npm run test -- --run src/services/raceAPI.solo.test.ts`
    (Solo race creation with starting tyre).
- Browser e2e (backend in degraded mode is fine): register/log in, then from
  the lobby create a solo race with each tyre (`Soft`, `Medium`, `Hard`) —
  verify the in-race panel shows the chosen tyre and its card counts (Solo
  race creation, Boost selection panel); play several turns submitting
  different boosts — verify standings/lap counter update and counts decrement
  (Turn validation and submission, Turn resolution flow, Bird's-eye local
  track view, Race status display); deplete a card value — verify the `×0`
  red badge and the pit-to-refill hint; pit with a tyre change — verify the
  refilled pool and new tyre (Pit stop control); finish the race — verify the
  completion screen and the lobby success message (Race completion and
  results).

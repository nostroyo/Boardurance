# race-ui — delta for add-multiplayer-turn-sync

## ADDED Requirements

### Requirement: Turn countdown and AFK auto-advance

The race UI SHALL display a MM:SS countdown while a multiplayer race turn is
open (the turn-phase response carries a non-null `seconds_remaining`), driven
by a local 1-second tick that re-synchronizes to `seconds_remaining` on every
poll (client clock skew never accumulates). Solo races (null deadline fields)
SHALL show no countdown. IF the countdown reaches zero
before the player has submitted, the client SHALL begin polling the
turn-phase endpoint with the current `turns_taken` as baseline, so the
auto-played turn is detected and the UI refreshes — including the boost hand,
which reflects the card consumed by auto-play.

#### Scenario: Countdown renders and re-syncs

- **WHEN** a turn-phase poll reports `seconds_remaining: 45`
- **THEN** the UI shows `00:45` and continues ticking down locally until the
  next poll re-syncs the value

#### Scenario: AFK player's UI advances after auto-play

- **WHEN** the countdown reaches zero without the player submitting and the
  backend auto-plays their turn
- **THEN** within one poll interval the UI refreshes to the new turn with one
  fewer card in the player's hand and a fresh countdown

## MODIFIED Requirements

### Requirement: Turn resolution flow

The client SHALL, after submitting an action whose response reports the turn
as already processed (`TurnProcessed`, the solo-race path), immediately
refresh race data; otherwise it SHALL capture the response's `turns_taken` as
a baseline and poll the turn-phase endpoint every 2 seconds (max 60 attempts,
exponential backoff on errors, canceled on unmount) until the polled
`turns_taken` exceeds the baseline or the phase reports `Complete`. On turn
completion the client SHALL refresh local view, boost availability, lap
history, and performance preview via the batch endpoint plus the turn phase,
and reset the boost selection so the next turn starts clean.

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

#### Scenario: Waiting client detects the executed turn

- **WHEN** a player submitted at baseline `turns_taken = N` and a later poll
  reports `turns_taken = N + 1` while the race is still `InProgress`
- **THEN** polling stops and the race data refreshes exactly as on turn
  completion (the polled phase alone never ends the wait — it returns to
  `WaitingForPlayers` for the next turn)

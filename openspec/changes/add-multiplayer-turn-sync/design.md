# Design: add-multiplayer-turn-sync

## Context

The simultaneous-turn mechanism already exists in the domain layer and solo mode
short-circuits it:

- `Race.pending_actions` / `action_submissions` (per-player submission
  timestamps) stage actions (`domain/race.rs:229-230`); `all_actions_submitted()`
  (930), `get_pending_players()` (956), and `process_if_ready()` (832) implement
  "resolve when everyone has submitted".
- `resolve_human_turn` (`routes/races.rs:4329`) is the single turn funnel:
  record human action → `enqueue_ai_actions` (fills every AI seat, consuming
  real boost cards — race.rs:1008) → process the lap when complete →
  `drive_ai_only_turns`.
- `ai_player::decide_ai_action` (ai_player.rs:126) is pure and deterministic,
  and degrades safely on an empty pool (free boost 0 or pit) — verified.

Three gaps prevent real multiplayer:

1. **No deadline** — nothing consumes the `action_submissions` timestamps; a
   silent player stalls the race forever.
2. **No turn-advancement signal** — the poller stops only on
   `turn_phase === 'Complete'` (useRacePolling.ts:153) and the backend emits
   `Complete` only when the race is finished (races.rs:2758). After a turn
   processes, `pending_actions` clears and the phase snaps back to
   `WaitingForPlayers`, so a waiting client observes no change at all.
3. **Lost updates** — handlers do read-clone → mutate → write-clone on the
   global `RACE_STORE` (races.rs:18-34); two concurrent submissions can
   interleave and drop one action.

Product decisions fixed by the user: polling transport (no WS/SSE) · fixed
per-turn timer configurable at race creation (default 60 s) · AFK = auto-pilot
forever, resume anytime · scope = turn sync only (no lobby; join/register/start
endpoints already exist).

## Goals / Non-Goals

**Goals:**

- A race with ≥ 2 humans plays turn-by-turn to completion regardless of player
  absence, with ≤ ~2 s enforcement latency after the deadline.
- A waiting client detects the executed turn within one poll interval and shows
  a per-turn countdown.
- Turn resolution is atomic under concurrency.
- Solo behavior is byte-for-byte unchanged (regression-guarded).

**Non-Goals:**

- No lobby/matchmaking/ready flow.
- No push transport (WebSocket/SSE).
- No auth hardening: endpoints keep trusting body `player_uuid` (documented
  gap, races.rs:927-931); server-authoritative identity is its own change.
- No Mongo persistence of races (they stay in-memory; see `persistence` spec).

## Decisions

### D1 — Lazy deadline enforcement (no background timers)

Store `turn_timeout_secs: Option<u32>` + `turn_deadline: Option<i64>` on
`Race` (both `#[serde(default)]` for store back-compat). On every turn-phase
poll and every submission: arm the deadline if unarmed (InProgress ∧ ≥ 2 humans
∧ timeout set); if `now >= deadline` and the turn is unresolved, auto-fill
pending humans and resolve.

*Why not `tokio::spawn` per-turn timers:* a timer must take the same store lock
and be cancelled on early resolution — it adds task-lifecycle complexity
without removing any locking; clients already poll every 2 s so the latency
difference is invisible for a 60 s deadline; races are in-memory (a restart
loses them), so timers buy no durability either. Lazy enforcement is a pure
function of an injected `now: i64` — trivially unit-testable. Accepted
trade-off: a GET mutates state (idempotent — a second enforcer sees the turn
already resolved), and a fully unwatched race stalls until someone polls
(acceptable: nobody is watching). → ADR.

### D2 — Atomic turn core via `store_update`

Add `fn store_update<T>(uuid, impl FnOnce(&mut Race) -> T) -> Option<T>`
beside `store_get`/`store_save` and run the whole turn mutation inside it as
`resolve_turn_core(race, intent: Option<(Uuid, TurnIntent)>, car_data_map, now)`
→ `Waiting { pending } | Processed(..)`. Verified feasible: the entire
record → enqueue-AI → fill-expired → process sequence is synchronous; the only
`await` (`build_car_data_map`) happens before mutation, so no lock is held
across an await point. `resolve_human_turn` and `process_lap_in_db` rewire
through the core (repo rule: one turn-resolution helper). Order inside the
core: check InProgress → arm deadline → record caller's intent (late-submit
grace) → enqueue AI → if expired, fill pending humans → if complete, process
lap, clear staging, re-arm deadline. → ADR (same document as D1).

### D3 — Multiplayer is derived, never stored

`is_multiplayer() = participants.filter(!is_ai).count() >= 2`. Players join
after creation (races auto-start empty, races.rs:3623), so a creation-time
flag would lie. Solo (1 human) therefore never arms a deadline and keeps
resolving in-request — the solo path is untouched by construction.

### D4 — Auto-play reuses the AI seat-filling path

Generalize `enqueue_ai_actions` → private
`enqueue_auto_actions(car_data_map, include_humans: bool)`; keep
`enqueue_ai_actions` as a wrapper (zero call-site churn) and add
`enqueue_actions_for_all_pending` for the expiry pass. Card consumption,
pit handling, and `boost_usage_history` recording are inherited — an
auto-played human is indistinguishable from an AI turn in the records, which
is exactly the "backend fakes the turn from full player state" behavior.

### D5 — Turn-advancement signal = `turns_taken`

Expose `turns_taken` (increments exactly once per processed turn, race.rs:697)
in `TurnPhaseResponse` and `SubmitTurnActionResponse`. **Not** `current_lap`,
which saturates at `total_laps` (race.rs:702) and would miss the final turn.
Client captures the submit response's value as baseline and treats
`polled > baseline` as NextTurnExecuted. No separate "last turn result"
payload: `handleTurnComplete` already refetches the full batch state.

### D6 — Countdown: server-computed `seconds_remaining`

Response carries both `turn_deadline` (epoch) and clamped `seconds_remaining`;
the client ticks locally at 1 s and re-syncs each poll — immune to client
clock skew. Feeds the existing unused `timeRemaining` prop of
`SimultaneousTurnController` (already renders MM:SS).

### D7 — `turn-phase` route moves routers, same public path

Enforcement needs `player_repository` to build the car-data map, so
`GET /races/:uuid/turn-phase` moves from `routes()` (`Router<Database>`) to
`turn_routes()` (`Router<RaceTurnState>`). The old registration is deleted in
the same commit (a duplicate path panics at router build — any handler test
catches it). Enforcement failures log-and-continue so a poll never 500s.

## Risks / Trade-offs

- [T3 refactor touches the solo funnel] → the existing solo regression tests
  (races.rs:4543, 4577) run before and after; they are the guardrail, plus the
  full-race domain test.
- [GET with side effects surprises future readers] → the requirement and the
  handler doc comment state it explicitly; enforcement is idempotent.
- [Two concurrent enforcement polls] → both go through `store_update`; the
  second sees the resolved turn — asserted by a dedicated test.
- [OpenAPI drift] → new DTO fields require
  `cargo run --bin dump_openapi > ../docs/openapi.json` **from Git Bash**
  (PowerShell `>` writes UTF-16 and the contract test at startup.rs:405 fails
  on encoding, which looks like content drift).
- [Store back-compat] → new `Race` fields are `#[serde(default)]`; a serde
  round-trip test on pre-change JSON proves old snapshots still deserialize.

## Migration Plan

Additive, no data migration (in-memory store). Deploy backend first (new
response fields are ignored by old clients), then frontend. Rollback = revert
the PR; no stored-state cleanup needed.

## Open Questions

None — all product decisions were settled before this design.

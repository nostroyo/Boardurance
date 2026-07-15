# 0002. Lazy turn-deadline enforcement and a single atomic turn core

- Status: Accepted
- Date: 2026-07-15
- Deciders: team

## Context
Multiplayer turn sync (openspec change `add-multiplayer-turn-sync`) needs two
mechanisms the codebase lacked:

- **A per-turn deadline** so an absent player cannot stall a race — expiry
  auto-plays pending humans through the AI decision path.
- **Safe concurrent submissions.** Handlers mutated the global `RACE_STORE`
  with read-clone → mutate → write-clone; two simultaneous submits could lose
  one action (proven by a regression test that failed at iteration 34 of 50
  before the fix).

## Decision
1. **Deadline enforcement is lazy — no background timers.** The deadline lives
   on the race (`turn_timeout_secs`, `turn_deadline`) and is armed/checked on
   every turn-phase poll and every submission. Expiry auto-plays pending
   humans via `enqueue_actions_for_all_pending` and resolves the turn.
2. **One atomic turn core.** `store_update` mutates the stored race in place
   under the store mutex, and `resolve_turn_core` (arm → record caller intent →
   enqueue AI → expiry-fill → process) runs entirely inside that critical
   section. `resolve_human_turn`, `process_lap_in_db`, and deadline enforcement
   all funnel through it (per ADR-0001's single-shared-path rule).

## Alternatives considered
- `tokio::spawn` per-turn timers — rejected: a timer must take the same store
  lock and be cancelled on early resolution, adding task-lifecycle complexity;
  clients poll every 2 s so enforcement latency is ≤ ~2 s either way; races are
  in-memory, so timers add no durability. Lazy enforcement is a pure function
  of an injected `now` and trivially unit-testable.
- Per-race locks or an actor per race — rejected at current scale: the global
  mutex is held only for a synchronous in-memory mutation (the only await,
  building the car-data map, happens before locking).

## Consequences
- (+) A race always finishes: expired turns resolve on the next poll; an
  auto-played player can resume any later turn (auto-pilot forever).
- (+) Concurrent submissions are safe by construction; the whole record →
  fill → process sequence is one critical section.
- (−) `GET /turn-phase` mutates state (arming/enforcing). Documented on the
  handler and in the spec; enforcement is idempotent, and a GET never 500s
  because of it (log-and-continue).
- (−) A race nobody polls stalls until someone polls — acceptable: nobody is
  watching.
- The turn funnel became synchronous (`process_lap_in_db`,
  `drive_ai_only_turns`, `resolve_human_turn`, `submit_player_action_in_db`
  dropped `async`) — clippy `unused_async` enforced this once the awaits
  disappeared.

## Follow-ups & accepted debt
- Race endpoints still trust the body `player_uuid` (no auth on race routes,
  pre-existing TODO) — server-authoritative identity is its own change.
- The legacy `/apply-lap` path (`process_individual_lap_action`) still uses
  read-clone → write-clone — flagged for a follow-up change to route it
  through `store_update`.

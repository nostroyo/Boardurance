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
2. **One atomic turn core, serialized by a per-race async lock.** All race data
   access goes through the async `RaceRepository` (load → mutate → save).
   `resolve_turn_core` (arm → record caller intent → enqueue AI → expiry-fill →
   process) holds a per-race `tokio::Mutex` guard (from a process-global
   `TURN_LOCKS` registry) across its whole `find_by_uuid` → mutate → `save`
   sequence, so concurrent submissions for one race serialize and cannot lose an
   update. `resolve_human_turn`, the batch `process_turn`, `pit`, and deadline
   enforcement all funnel through it (per ADR-0001's single-shared-path rule);
   `drive_ai_only_turns` runs after the guard releases and re-takes it per AI
   turn (never nested — `tokio::Mutex` is not reentrant).

   *History:* the first cut of this ADR used a synchronous `store_update`
   closure over an in-memory `RACE_STORE`. When mongo-persistence merged to
   `dev` it removed that store in favor of the async repository, so the atomic
   core was re-expressed as the per-race async lock above — same guarantee, new
   substrate (see design D2).

## Alternatives considered
- `tokio::spawn` per-turn timers — rejected: a timer must take the same per-race
  lock and be cancelled on early resolution, adding task-lifecycle complexity;
  clients poll every 2 s so enforcement latency is ≤ ~2 s either way. Lazy
  enforcement is a pure function of an injected `now` and trivially
  unit-testable.
- Repository-level optimistic concurrency (version/CAS on `save`) — deferred:
  correct for genuine multi-writer Mongo, but heavier and touches the
  just-merged persistence layer. The per-race route-layer lock is sufficient
  while a single process owns writes (true today). Recorded as accepted debt.
- A synchronous global store mutex (the original design) — no longer available:
  the in-memory store it relied on was removed by mongo-persistence.

## Consequences
- (+) A race always finishes: expired turns resolve on the next poll; an
  auto-played player can resume any later turn (auto-pilot forever).
- (+) Concurrent submissions for one race are safe: the per-race guard makes
  the whole record → fill → process → save sequence one critical section;
  different races never contend.
- (−) `GET /turn-phase` mutates state (arming/enforcing). Documented on the
  handler and in the spec; enforcement is idempotent, and a GET never 500s
  because of it (log-and-continue).
- (−) A race nobody polls stalls until someone polls — acceptable: nobody is
  watching.
- The turn funnel is async (over the repository); `resolve_turn_core` is async
  and holds a `tokio::Mutex` across repository awaits.
- `TURN_LOCKS` grows one `Arc<tokio::Mutex>` per race for the process lifetime —
  immaterial at current race volume; a future sweep can evict finished races.

## Follow-ups & accepted debt
- Race endpoints still trust the body `player_uuid` (no auth on race routes,
  pre-existing TODO) — server-authoritative identity is its own change.
- The legacy `/apply-lap` path (`process_individual_lap_action`) still does a
  bare load → mutate → save with no per-race guard — flagged for a follow-up to
  route it through the per-race lock / `resolve_turn_core`.

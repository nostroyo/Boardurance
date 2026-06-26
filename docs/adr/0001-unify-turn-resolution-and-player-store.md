# 0001. Unify solo turn resolution and player storage on single shared paths

- Status: Accepted
- Date: 2026-06-25
- Deciders: team

## Context
Solo-mode play had two parallel implementations of the same concept that had drifted:

- **Turn resolution** lived in two route paths — `/submit-action` enqueued AI but
  did not consume the human's boost card; `/pit` consumed the card but never
  enqueued AI and so deadlocked. Each path was missing the half the other had.
- **Player data** lived in two stores — registration wrote to the in-memory
  `MockPlayerRepository`, but the car/pilot/configuration mutators queried a
  Mongo `players` collection that registered players never populated, so they
  404'd for every real player.

Classic "implemented twice, then drifted" — the review gate flagged it as the
shared root cause behind three separate bugs.

## Decision
We will funnel each concept through a single shared path:
- One `resolve_human_turn` helper that both `/submit-action` and `/pit` call
  (record the human's card-consuming action → enqueue AI → process → drive AI).
- One player store: the asset mutators use the same in-memory repository that
  registration writes to; routes consolidated into `team_routes()`.

## Alternatives considered
- Fix each endpoint / store independently — rejected: leaves two paths that drift
  again (the actual root cause).
- Make `process_lap_in_db` generic over the hasher (to satisfy clippy
  `implicit_hasher`) — rejected: ripples a generic through every downstream domain
  method; used a localized `#[allow]` instead.

## Consequences
- (+) One place to change turn logic / player mutations; the divergence class is
  designed out. Each fix is bound by a regression test.
- (−) `pit_stop_action` moved from the `Database` router to the AppState/in-memory
  world; its response builders dropped an unused `_database` parameter.

## Follow-ups & accepted debt
- Player asset routes remain unauthenticated (pre-existing) — add `AuthMiddleware`
  + ownership before real multi-tenant traffic.
- `/apply-lap` (DB-backed, multiplayer) still uses a separate path — unify it
  through `resolve_human_turn` or document it as multiplayer-only. See `docs/reviews/`.

# Proposal: add-multiplayer-turn-sync

## Why

Boardurance only ships solo races today: the simultaneous-turn mechanism exists in the domain (`pending_actions`, resolve-when-all-submitted), but solo mode short-circuits it by auto-filling AI seats, so a race with two or more humans has never actually worked end to end — a waiting client cannot even detect that a turn resolved, and a player who walks away stalls the race forever. This change makes real multiplayer races playable: submit → wait for everyone → turn executes, with a per-turn deadline that auto-plays absent players so a race always finishes.

## What Changes

- Races gain a configurable per-turn deadline (`turn_timeout_secs`, default 60, only armed for races with ≥ 2 human participants). When the deadline passes, the backend generates an action for every pending human using the existing AI decision logic (consuming real boost cards) and resolves the turn — auto-pilot forever, the player can resume any later turn.
- Deadline enforcement is lazy (checked on every turn-phase poll and every submission) — no background timers. Clients poll every 2 s, so enforcement latency is ≤ ~2 s.
- Turn-phase and submit responses expose `turns_taken` plus deadline info (`turn_deadline`, `seconds_remaining`) so a waiting client can detect "the turn I was waiting on executed" (the polled equivalent of a NextTurnExecuted push) and render a countdown. Today the poller can only detect whole-race completion — multiplayer waits never resolve.
- Turn resolution becomes atomic: the entire record → auto-fill → process sequence runs under a single race-store lock, fixing a real lost-update window when two players submit concurrently (the current read-clone → mutate → write-clone pattern).
- Frontend: countdown display (feeding the existing unused `timeRemaining` prop), turn-advancement detection in the polling hook, and auto-advance for a player whose turn was auto-played.
- Solo races are untouched: one human ⇒ no deadline armed, submit still resolves the turn in the same request.

No **BREAKING** changes: new request/response fields are optional/additive; existing solo and batch endpoints keep their behavior.

## Capabilities

### New Capabilities

_None — turn synchronization is a deepening of the existing race engine and race UI capabilities._

### Modified Capabilities

- `race-engine`: simultaneous turn resolution gains a per-turn deadline requirement (arming rules, lazy enforcement, AI auto-fill for absent humans with real card consumption); turn-phase reporting gains `turns_taken` + deadline fields; the store-consistency requirement is strengthened from "each read-modify-write is serialized" to "a whole turn resolution is one atomic critical section"; race creation accepts `turn_timeout_secs`.
- `race-ui`: turn resolution flow gains turn-advancement detection (baseline `turns_taken` instead of phase-only polling), a per-turn countdown display, and AFK auto-advance (countdown reaching zero without a submission starts polling so the auto-played turn is picked up).

## Impact

- Backend: `rust-backend/src/domain/race.rs` (deadline fields + predicates, generalized auto-action enqueue), `rust-backend/src/routes/races.rs` (atomic `store_update`, locked turn core, lazy enforcement, DTO fields, `turn-phase` route moves to the state-bearing router — same public path), `docs/openapi.json` regeneration (contract test).
- Frontend: `empty-project/src/hooks/useRacePolling.ts`, new `useTurnCountdown.ts`, `RaceContainer.tsx` + status/turn-controller prop plumbing, regenerated `types/api-generated.ts`.
- ADR needed: yes — one ADR covering the two load-bearing decisions: lazy deadline enforcement (no background timers) and single-lock atomic turn resolution (`docs/adr/`).
- Known gap documented, not fixed here: race endpoints trust the body `player_uuid` (no auth on race routes). Server-authoritative identity is its own change.

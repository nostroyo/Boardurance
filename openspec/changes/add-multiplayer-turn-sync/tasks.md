# Tasks: add-multiplayer-turn-sync

Execution discipline (applies to every task): **strict TDD** — write the
task's tests first, run them, confirm they fail for the right reason, then
implement the minimal code to green with no regressions in the affected suite.
After green, each task gets a **zero-context adversarial review** (fresh
subagent that sees only the specs, the diff, and the test command — never the
implementer's reasoning); the task is checked off only on a PASS verdict.
No feasibility probes: every mechanism reuses a verified existing path.

## 1. Domain: deadline fields and predicates

Implements: race-engine/Per-turn deadline with auto-play for absent players (arming rules).

- [x] 1.1 Tests first in `rust-backend/src/domain/race.rs` `mod tests`: `arm_turn_deadline` arms only when InProgress ∧ ≥2 humans ∧ `turn_timeout_secs` set ∧ unarmed; a solo-shaped race (1 human + 2 AI) never arms; `is_turn_expired` boundary (`now == deadline` ⇒ expired); serde round-trip of pre-change race JSON (no new fields) deserializes with both fields `None`.
- [x] 1.2 Implement `turn_timeout_secs` + `turn_deadline` fields (`#[serde(default)]`, initialized `None` in `Race::new`) and `human_count` / `is_multiplayer` / `arm_turn_deadline(now)` / `is_turn_expired(now)`; run to green.

> Review: PASS (round 1, zero-context reviewer; mutation-tested `is_turn_expired` boundary and human/AI counting). Red evidence: E0609/E0599 on missing fields+methods. Green: `cargo test --lib` 130/130 (includes regenerated `docs/openapi.json` contract).

## 2. Domain: auto-play for pending humans

Implements: race-engine/Per-turn deadline with auto-play for absent players (auto-play semantics).

- [x] 2.1 Tests first: `enqueue_actions_for_all_pending` stages actions for pending humans and AI, consumes one card each, appends `boost_usage_history`, skips already-acted/finished participants and seats with missing car data; an empty-pool human gets pit-or-boost-0 (delegated to `ai_player`).
- [x] 2.2 Implement the `enqueue_auto_actions(car_data_map, include_humans)` generalization with `enqueue_ai_actions` kept as a wrapper (existing AI-enqueue tests must stay green untouched); run to green.

> Review: PASS (round 1, zero-context Opus reviewer). One concern (no direct card-count assertion) addressed post-review with an explicit `cards_remaining` check. Red: E0599 missing method. Green: `cargo test --lib` 134/134; pre-existing AI-enqueue test untouched.

## 3. Routes: atomic turn core

Implements: race-engine/Races live in the process-global race store (atomic critical section); race-engine/Per-turn deadline (late-submit grace).

- [x] 3.1 Tests first in `routes/races.rs` `turn_resolution_tests`: add `seed_multiplayer_race(timeout_secs)` helper (mirrors `seed_solo_race`, two humans + timeout); A submits ⇒ Waiting, pending=[B], `turns_taken == 0`, A's card consumed; B submits ⇒ Processed, `turns_taken == 1`, staging cleared, deadline re-armed; duplicate submit by A ⇒ conflict error; concurrency regression: ~50 iterations of two `tokio::spawn`ed submits ⇒ exactly one resolution and both cards consumed each time.
- [x] 3.2 Implement `store_update`, `compute_performances` (sync extraction from `process_lap_in_db`), and `resolve_turn_core` (order per design D2); rewire `resolve_human_turn` and `process_lap_in_db` through the core. Existing solo tests (races.rs:4543, 4577) must pass before and after; run to green.

> Review: PASS (round 1, zero-context Opus reviewer; verified lock hygiene, D2 ordering, and that the concurrency test catches a reintroduced clone-write). Red evidence: deadline assertions failed + concurrency test caught the real lost update at iteration 34. Green: `cargo test --lib` 138/138; solo tests untouched. Reviewer concerns: legacy `apply_lap_action` clone-write path (out of scope — spawned follow-up task) and `drive_ai_only_turns` clone-write (design-sanctioned per D2).

## 4. Routes: lazy deadline enforcement

Implements: race-engine/Per-turn deadline with auto-play for absent players (enforcement); race-engine/Turn phase reporting (enforcement-on-poll).

- [x] 4.1 Tests first: expired turn + one absentee ⇒ absentee auto-played (card consumed, history entry) and turn resolved; nobody submitted + expired ⇒ all auto-played; late submit after deadline records the caller's own action (grace); unexpired enforcement pass ⇒ no-op besides arming; double enforcement ⇒ idempotent (second pass changes nothing); auto-played player's next-turn submit accepted.
- [x] 4.2 Implement `enforce_turn_deadline(race_uuid, repo)` (car-data map built outside the lock, core called with `intent = None`, log-and-continue on error); run to green.

> Review: PASS (round 1, zero-context Opus reviewer; proved the card-consumption assertion is live — auto-played boost is deterministically 2 on a fresh Medium pool — and that late-submit grace ordering is pinned). Note: the six expiry-behavior tests were green-on-arrival pins (the core logic shipped with D2's ordering in group 3); the red→green piece was `enforce_turn_deadline` (E0425). Post-review: extracted the shared `RACE_NOT_IN_PROGRESS` const to fix the brittle string-match concern. Green: 145/145. Remaining concerns (snapshot-vs-lock join window is benign; wrapper end-to-end coverage) land with group 5's handler test.

## 5. Routes: DTOs, route move, OpenAPI

Implements: race-engine/Turn phase reporting (new fields); race-engine/Race creation auto-starts the race (`turn_timeout_secs`).

- [x] 5.1 Tests first: `get_turn_phase` response carries `turns_taken`/`turn_deadline`/`seconds_remaining` (clamped ≥ 0); polling an expired race resolves the turn and reflects the incremented counter; `POST /races` with `turn_timeout_secs: 3` ⇒ 400, omitted ⇒ default 60, solo create ⇒ None; `SubmitTurnActionResponse` carries post-submit `turns_taken`.
- [x] 5.2 Implement DTO fields; move `GET /races/:uuid/turn-phase` from `routes()` to `turn_routes()` (delete old registration same commit) with enforcement as the handler's first step; run to green.
- [x] 5.3 Regenerate the OpenAPI contract (via `cmd /c` raw redirection — PowerShell writes UTF-16 and fails the contract test) and run the full backend verify gate: fmt ✓ clippy ✓ check ✓ test-fast ✓ (150/150).

> Review: PASS (round 1, zero-context Opus reviewer; zero findings — verified enforcement-first ordering, nullability in the regenerated OpenAPI schema, single route registration, and that the async→sync cascade weakened no test). Note: clippy `-D warnings` forced the cascade — `process_lap_in_db`/`drive_ai_only_turns`/`resolve_human_turn`/`submit_player_action_in_db` became sync (their awaits vanished with the atomic core), and the concurrency test upgraded to real OS threads (`std::thread::scope`).

## 6. Frontend: turn-advancement polling

Implements: race-ui/Turn resolution flow (baseline detection).

- [ ] 6.1 Regenerate API types (`npm run gen:api`) and extend `types/race-api.ts` with the new fields.
- [ ] 6.2 Tests first (Vitest): poller with `baselineTurn = 3` fires `onComplete` when a poll reports `turns_taken: 4`; does not fire on `turns_taken: 3` phase-flapping; still fires on `'Complete'`. Then implement the `baselineTurn` option in `hooks/useRacePolling.ts`; run to green.

## 7. Frontend: countdown and AFK auto-advance

Implements: race-ui/Turn countdown and AFK auto-advance.

- [ ] 7.1 Tests first (Vitest fake timers): `useTurnCountdown` ticks down at 1 s and re-syncs on a new `seconds_remaining`; null stays hidden (solo). Then implement the hook.
- [ ] 7.2 Tests first: `RaceContainer` stores the submit response's `turns_taken` as baseline on the `WaitingForPlayers` branch and starts polling; countdown-at-zero without submission starts polling with the current baseline (AFK auto-advance). Then wire `timeRemaining` through `RaceInterface` to `SimultaneousTurnController`/`RaceStatusPanel` and run the frontend verify gate (`fe.ps1`: tsc, vitest).

## 8. Verification, ADR, review gate

- [ ] 8.1 Write the ADR for design decisions D1 + D2 (lazy deadline enforcement; single-lock atomic turn core) under `docs/adr/`.
- [ ] 8.2 Full browser e2e (gameplay-affecting change): two-player race via the UI — happy path (submit/wait/countdown/resolve ≤ 2 s after last submit), timeout path (only A submits, countdown hits zero, B auto-played with a real card consumed and B's idle browser advancing), resume path (B submits normally next turn), and a full solo race regression (instant resolution, no countdown ever rendered). Screenshot proof.
- [ ] 8.3 Run both verify gates once more (`be.ps1` + `fe.ps1`) plus `openspec validate --all --strict`, then `/review-gate` and resolve any BLOCK before opening the PR into `dev`.

# Review gate — feat/multiplayer-turn-sync

- Date: 2026-07-15
- Base SHA: 23541bd (origin/dev)  | Head SHA: d2a1d9c
- Spec: openspec/changes/add-multiplayer-turn-sync/
- Changed areas: backend + frontend
- Verdict: **PASS** (after fixing 8 findings in-branch; see "Fixed during the gate")

Review base note: the command's default base is origin/main, but this PR targets
`dev`; the diff was reviewed against `origin/dev...HEAD` so judges saw only this
change. The /security-review skill cannot run from the session cwd (not a git
repo), so an equivalent security judge ran as a dedicated subagent with the
skill's scope plus the CLAUDE.md Always/Never checklist.

## Requirement/scenario checklist (spec-conformance judge)

All 19 scenarios across the race-engine and race-ui deltas: **PASS**.
Highlights (test names in rust-backend/src/routes/races.rs `turn_resolution_tests`,
rust-backend/src/domain/race.rs `tests`, and empty-project/src tests):

- [PASS] race-engine/Per-turn deadline — absent player auto-played on expiry — `expired_turn_auto_plays_absent_player`
- [PASS] race-engine/Per-turn deadline — everyone absent still advances — `fully_absent_turn_auto_plays_everyone`, `polling_expired_race_resolves_turn_end_to_end`
- [PASS] race-engine/Per-turn deadline — late submission wins over auto-play — `late_submit_keeps_own_action_over_autoplay`
- [PASS] race-engine/Per-turn deadline — enforcement idempotent — `double_enforcement_is_idempotent` (+ concurrency via `concurrent_submits_lose_no_update`, 50 iters on OS threads)
- [PASS] race-engine/Per-turn deadline — auto-played player resumes — `auto_played_player_resumes_next_turn`
- [PASS] race-engine/Per-turn deadline — solo never arms — `solo_shaped_race_never_arms_deadline`, `turn_phase_response_solo_has_no_countdown`, `solo_poll_is_read_only`
- [PASS] race-engine/Store — concurrent submissions consistent — `concurrent_submits_lose_no_update`
- [PASS] race-engine/Race creation — out-of-range timeout rejected — `turn_timeout_validation_bounds`
- [PASS] race-engine/Turn phase reporting — counter/countdown/enforce-on-poll — `turn_phase_response_reports_counter_and_countdown`, `submit_response_reports_turns_taken`, `polling_expired_race_resolves_turn_end_to_end`
- [PASS] race-ui/Countdown + AFK — render/re-sync/auto-advance — `useTurnCountdown` suite (9), `RaceContainer.multiplayer` suite (4)
- [PASS] race-ui/Turn resolution flow — baseline detection — `useRacePolling` suite (4)
- [N-A → e2e] UI-rendering-only aspects (car sprites, list contents, solo instant-resolve branch) — pinned by the recorded browser e2e (tasks.md §8.2), re-verified live after the fixes (staged pit → waiting → advance within one poll).

## Correctness (code-review judges: 8 finder angles, recall-biased)

### Fixed during the gate (commit d2a1d9c)
- [high] routes/races.rs — stale `#[tracing::instrument]` attached to `compute_performances`, Debug-printing the whole Race into a span inside the store lock; moved back onto `process_lap_in_db` (found by 3 independent angles)
- [high] RaceContainer.tsx — multiplayer pit desync: pit treated as turn-resolved (solo semantics), re-enabling inputs mid-turn (409 trap) and never polling; now mirrors the boost waiting branch (staged → baseline → poll); verified live in the browser
- [high] useRacePolling.ts — poll budget (~2 min) shorter than the 600 s max deadline: client gave up before enforcement could fire, and with lazy enforcement a race nobody polls never advances; budget raised to ~11 min and countdown-expiry now restarts a dead poller even for submitted players
- [high] useTurnCountdown.ts / useRacePolling.ts — mid-wait freeze: poll payloads were delivered only on phase-string changes, and the countdown re-synced on the numeric value (60→60 across turns never re-armed, permanently killing the AFK trigger); poller now delivers every payload and the countdown re-syncs on payload identity
- [high] routes/races.rs — `drive_ai_only_turns` kept the unlocked clone-write pattern while the new poll path became a writer (lost-update window on AI-only continuations); now one atomic `store_update` pass per AI turn
- [medium] routes/races.rs + domain/race.rs — expired turn with an unresolvable seat (missing car data) stalled the race forever at seconds_remaining=0; such seats are now auto-played with the free boost 0
- [medium] routes/races.rs — `enforce_turn_deadline` did per-poll repo fan-out + a global write-lock core pass even when nothing could happen (and staged AI actions on solo polls — a semantic drift); early-out added, solo polls strictly read-only again
- [medium] domain/race.rs — `is_multiplayer` counted finished humans, so a sole survivor kept getting deadline-enforced; now counts active humans only (spec wording updated)

### Non-blocking notes (recorded, follow-ups spawned where warranted)
- Global `RACE_STORE` mutex serializes all races' turn work (design-accepted at current scale, ADR-0002; revisit per-race locking/actor if load grows)
- `timeRemaining` prop defeats `React.memo` on `RaceInterface` → 1 Hz subtree re-render during waits; scope the countdown to `RaceStatusPanel` when convenient
- MM:SS formatting duplicated vs the dead `SimultaneousTurnController` (never rendered) — extract a shared formatter and delete the dead component
- `baselineTurn`/`isPolling` are a two-field encoding of one concept; a single nullable baseline would make invalid states unrepresentable
- Test-fixture duplication (TurnPhase builders ×2, race seeders ×3, player-seeding blocks ×4) — extract shared helpers
- `RACE_NOT_IN_PROGRESS` string sentinel between core and enforcement — a typed outcome variant would be sturdier
- `TurnPhaseResponse.turn_deadline` is redundant next to `seconds_remaining` (kept for debuggability; documented)
- Dormant `PlayerGameContext` consumer still has solo-only phase semantics (not mounted by any route)

## Security (security judge + Always/Never)

- [medium] GET /turn-phase mutation surface (unauthenticated, pre-existing no-auth) — **reduced** by the early-out fix: steady-state polls no longer touch the repo or write lock; residual arming/expiry passes are idempotent and time-gated
- [medium] `store_update`'s `.lock().unwrap()`: a panic inside the turn core would poison the store mutex (global DoS blast radius); no reachable panic input found — accepted debt, recorded in ADR-0002
- [low, pre-existing] race endpoints trust body `player_uuid` (no auth) — documented gap in proposal.md, separate change
- [low, pre-existing] `RaceStatusPanel` labels not i18n-gated — codebase has no i18n framework at all; no new literal added by this diff
- TEST-INTEGRITY: OK (no skipped/weakened/deleted tests; +30 tests net)

## Blocking items (must fix before PR)
- none — all eight blocking findings fixed in d2a1d9c and re-verified (backend gate: fmt/clippy/check/test-fast 154 ✓; frontend gate: tsc + 119 tests ✓; openspec validate --strict ✓; live browser re-check of the pit flow ✓)

## Non-blocking notes
- See "Non-blocking notes" above; follow-up tasks spawned for the legacy `/apply-lap` clone-write path (pre-existing) and the frontend cleanup items.

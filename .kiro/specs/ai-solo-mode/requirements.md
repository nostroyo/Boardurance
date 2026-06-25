# Requirements Document

## Introduction

The AI Solo Mode adds **computer-controlled opponents** to the single-player racing experience. Today the game is multiplayer-only: a turn resolves only once **every** active participant submits a boost card (0–4), so a solo player would wait forever for empty opponent seats. This feature lets a player start a race against AI cars that automatically choose a sensible boost each turn, so the lap resolves immediately without human opponents.

This builds directly on the [`single-player-race-mvp`](../single-player-race-mvp/requirements.md) spec (the solo turn-loop / frontend integration) and the existing race simulation engine. Scope is intentionally narrow:

- **Boost AI only** — the AI decides the same thing a human decides: which boost card (0–4) to play each turn, plus pre-race car/pilot selection. No new game systems.
- **Single balanced AI profile** — one reasonable opponent behavior; no difficulty tiers, personalities, or adaptive/lookahead logic.

> **Note:** The original request referenced "choose tire / when to pit". The simulation has **no tire, pit, fuel, or weather model** — those are explicitly out of scope (see end of document). The only in-race decision is the boost card.

## Glossary

- **AI_Participant**: A `RaceParticipant` controlled by the server, flagged `is_ai = true`, that never submits via the API.
- **Bot_Player**: A seeded player record (with a pre-equipped car + pilots) used as the identity for an AI_Participant so its stats resolve through the normal car-data pipeline.
- **Boost_Brain**: The pure decision function that maps (car stats, sector, lap characteristic, boost hand) → a boost value 0–4.
- **Balanced_Profile**: The single AI behavior — advance a sector when it is cheap, avoid dropping a sector, and otherwise conserve high boost cards.
- **Auto_Submit**: Server-side step that enqueues boost actions for all AI_Participants when a human submits, so the existing auto-process path resolves the lap.
- **Solo_Race**: A race created with one human participant plus N AI_Participants, started immediately.
- **Additive_Model**: The authoritative performance formula used by lap resolution: `final = min(engine + body + pilot, sector.max_value) + boost`.

## Requirements

### Requirement 1 — Mark participants as AI

**User Story:** As the system, I want to distinguish AI participants from humans, so that the server knows which cars to control.

#### Acceptance Criteria
1. THE `RaceParticipant` model SHALL include an `is_ai` boolean flag.
2. THE flag SHALL default to `false` for existing/serialized races (backward compatible).
3. THE system SHALL provide a way to add a participant as AI without breaking the existing human-join path.

### Requirement 2 — AI boost decision (Balanced_Profile)

**User Story:** As a player, I want AI opponents to make sensible boost choices, so that solo races feel competitive and fair.

#### Acceptance Criteria
1. THE Boost_Brain SHALL only ever return a boost value that is currently available in the participant's boost hand.
2. THE Boost_Brain SHALL compute predicted movement using the **Additive_Model**, not the multiplicative preview model.
3. WHEN at least one available boost yields `MoveUp`, THE Boost_Brain SHALL select the **smallest** such boost (advance while conserving high cards).
4. WHEN no available boost yields `MoveUp` AND the participant risks `MoveDown` (base `< sector.min_value`), THE Boost_Brain SHALL select the smallest available boost that reaches `Stay`.
5. WHEN neither advancing nor avoiding `MoveDown` is achievable, THE Boost_Brain SHALL select the smallest available boost (conserve the hand).
6. THE Boost_Brain SHALL be deterministic (a pure function of the inputs) so solo races are reproducible.
7. THE Boost_Brain SHALL select the component values (straight vs curve) according to the current lap characteristic, matching `calculate_performance_with_car_data`.

### Requirement 3 — Auto-submit AI actions

**User Story:** As a player, I want each turn to resolve as soon as I submit, so that I'm not blocked waiting on AI opponents.

#### Acceptance Criteria
1. WHEN a human submits a turn action, THE system SHALL enqueue a boost action for every AI_Participant that is not finished and has not already acted this turn.
2. THE system SHALL reuse the existing auto-process path so that, once all active participants (human + AI) have submitted, the lap resolves immediately.
3. THE system SHALL NOT change the existing lap-resolution logic (`process_lap_in_db` / `process_lap_with_car_data`).
4. THE AI actions SHALL resolve their performance using the same car-data map used for human actions.

### Requirement 4 — Solo race setup

**User Story:** As a player, I want to start a solo race against AI from the UI, so that I can play immediately on my own.

#### Acceptance Criteria
1. THE system SHALL seed Bot_Players with pre-equipped cars and pilots so their stats resolve through `build_car_data_map` with no special-casing.
2. THE bots SHALL have slightly varied car builds so the field is not identical.
3. THE system SHALL expose an endpoint (`POST /races/solo`) that creates a race, joins the human, joins K AI bots, starts the race, and returns the race UUID.
4. THE frontend SHALL provide a "Solo Race" entry point that calls the endpoint and routes into the existing race interface.
5. THE solo race SHALL run through the existing turn loop with no changes to the human turn UI.

### Requirement 5 — End-to-end completion

**User Story:** As a player, I want to finish a full solo race, so that I see a final result.

#### Acceptance Criteria
1. THE solo race SHALL progress turn-by-turn until all participants finish all laps and the race reaches `Finished`.
2. THE race SHALL produce a final ranking including the human and the AI cars.
3. THE turn loop SHALL never hang waiting for AI input (no "waiting for players" stall caused by AI seats).

### Requirement 6 — Preview/resolution model parity

**User Story:** As a player, I want the boost preview to predict the same outcome the race actually produces, so that my (and the AI's) decisions are trustworthy.

#### Acceptance Criteria
1. THE boost preview SHALL compute predicted final value using the **Additive_Model** (`final = min(base, sector.max_value) + boost`), matching lap resolution.
2. THE system SHALL update both preview code paths — `BoostHandManager::get_boost_availability` (`boost_hand_manager.rs`) and the `get_performance_preview` endpoint (`races.rs`) — to the additive formula.
3. THE related doc comments that describe the `base * (1 + boost*0.08)` multiplier SHALL be corrected to the additive formula.
4. THE existing test that asserts the multiplicative result SHALL be updated to assert the additive result; preview values SHALL equal the frontend's `calculateFinalValue` for the same inputs.
5. THE movement classification derived from the predicted final value SHALL be unchanged in logic (only the value feeding it changes).

## Acceptance Criteria (HARD GATE — applies to every task in this spec)

A task is **not done** until **all three** pass:

**1. Backend verify loop** (from `rust-backend/`):
```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings -A clippy::too_many_lines -A clippy::cast_possible_truncation -A clippy::cast_precision_loss -A clippy::cast_sign_loss -A clippy::cast_possible_wrap -A clippy::match_wildcard_for_single_variants -A clippy::manual_let_else -A clippy::needless_pass_by_value -A clippy::needless_range_loop -A dead_code
cargo check --all-targets --all-features
cargo test-fast
```

**2. Frontend verify loop** (from `empty-project/`):
```
npx tsc --noEmit
npm run test -- --run
```

**3. Functional unit tests + a COMPLETE end-to-end race in browser mode:**
- **Unit tests** exist and pass for the Boost_Brain (minimal-`MoveUp` pick; anti-`MoveDown` fallback; conserve-when-stuck; never returns an unavailable card; behavior at the `max_value` cap) and the Auto_Submit wiring (1 human + ≥2 AI → all AI actions enqueued and the lap auto-processes).
- **End-to-end browser test** driven with the `preview_*` tools:
  1. Start backend (`cargo run`) + frontend dev server.
  2. Launch a Solo_Race from the UI.
  3. Play **every turn through to race completion** — each lap resolves immediately (no waiting-for-players hang) and AI cars advance/conserve sensibly.
  4. The race reaches `Finished` with a final ranking and **no console/network errors**.
  5. Capture a screenshot of the finished race as proof.

## Out of Scope (future)
- Tire compounds / wear, pit stops, fuel, weather — require new simulation systems before any AI can decide on them.
- Difficulty tiers, AI personalities, adaptive/lookahead strategy.

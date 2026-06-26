# Tyre-Based Boost Pool + Pit Stops — Requirements

## Problem / Motivation

Previously every participant had the same boost hand: five cards `0,1,2,3,4` (one
each) that auto-replenished the instant they were spent. There was no strategic
resource trade-off. This feature introduces a **tyre choice** that shapes the boost
pool and a **pit stop** to refill it, creating a soft-vs-hard strategic decision.

## Glossary

- **Tyre type** — `Soft | Medium | Hard`, chosen at race entry and changeable at a pit
  stop. Defines the boost card pool.
- **Boost card** — a value `1..=4` consumed for one lap. Boost **0 is not a card**: it
  is the always-available free "no boost" move.
- **Pool** — the multiset of boost cards granted by the fitted tyre.
- **Pit stop** — an action that refills the pool (optionally switching tyre) and costs
  a lap.

## Requirements

1. **Tyre-defined pools.** Softer tyres give fewer but stronger cards; harder tyres give
   more but weaker cards. Tentative pools (tunable in one place):
   - Soft = `[3, 4, 4]` (3 cards)
   - Medium = `[2, 2, 3, 3, 4]` (5 cards)
   - Hard = `[1, 1, 1, 2, 2, 3]` (6 cards)
2. **Boost 0 is free.** It is always selectable, consumes no card, and never errors —
   even when the pool is empty.
3. **No auto-replenish.** Spending the pool does not refill it. When empty, only boost 0
   is available until the player pits.
4. **Tyre at registration.** The player chooses a tyre when entering a race (and for the
   solo mode, when starting it). Defaults to Medium.
5. **Pit stop.** A pit stop refills the pool from a chosen tyre (default: current tyre),
   costs the turn as a free boost-0 lap, and increments a pit-stop counter.
6. **Validation.** Selecting a spent card (value 1-4 with 0 remaining) is rejected with a
   clear error listing available cards; boost 0 is always accepted; values > 4 are
   rejected as invalid.
7. **Visibility.** API responses expose the fitted tyre, per-value remaining counts, and
   pit-stop count (replacing the old cycle counters).

## Acceptance criteria

- Backend unit tests (`cargo test-fast`) cover pools, multiset depletion, no-auto-replenish,
  free boost 0, pit refill + tyre swap, and validation.
- Backend compiles under `cargo check --all-targets` and clippy is clean.
- Frontend type-checks and tyre selection is available at solo race creation; the in-race
  panel shows tyre + per-value counts + a free boost 0.
- End-to-end: register/create with each tyre → correct pool; deplete to empty → only 0;
  pit with a new tyre → pool refilled with that tyre and a lap consumed.

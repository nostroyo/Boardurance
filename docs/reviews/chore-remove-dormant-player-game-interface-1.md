# Review gate — chore/remove-dormant-player-game-interface

- Date: 2026-07-16
- Base SHA: 1917f6e (origin/dev, post-#15) | Head SHA: 85b5f3f (rebased — see Rebase note)
- YouTrack: RACE-21
- Changed areas: frontend only (`empty-project/`)
- Verdict: **PASS** (no blocking items)

## Change

Removes the dormant, unmounted `PlayerGameInterface` / `PlayerGameContext` React
stack — superseded by `RaceContainer`, the canonical race UI per
`openspec/specs/race-ui/spec.md`. Also deletes its smoke test, two dead barrel
exports, and now-orphaned `ui-state.ts` type exports. Pure deletion: 5 files,
−1291 lines.

## Spec-conformance judge

PASS. `openspec/specs/race-ui/spec.md` names `RacePlayPage → RaceContainer` (+
`RaceInterface`, `TrackDisplayRedesign`, …) as the sources of truth and never
lists `PlayerGameInterface`. No file under `openspec/` changed; no delta spec is
affected. `.kiro/specs/player-game-interface/` is frozen legacy history (per
CLAUDE.md) and is not current truth.

## Correctness judge (code-review, max effort: 10 finder angles + verify + sweep)

PASS — 0 correctness/build/convention findings. Independent finders + the build
gate confirm no live consumer of any deleted symbol; both routes (`RacePlayPage`,
`GameWrapper`) mount `RaceContainer`. Deleting `PlayerGameInterface.test.tsx` is
not a coverage-reduction violation — it exercised only the deleted component (all
APIs mocked), the case the CLAUDE.md rule's own rationale carves out.

Non-blocking cleanup follow-ups recorded, out of scope for this change:
- dead `calculateLocalView` / `LocalRaceView` in `types/race.ts` (now zero importers);
- the `player-game-interface/index.ts` barrel is itself unimported (whole barrel dead);
- stale references to the removed stack in `debug-uuid-mismatch.ps1` and ~24 `docs/`/frozen-`.kiro/` files.

## Security judge (+ CLAUDE.md Always/Never)

PASS. Removes no live security control — the deleted stack was unmounted; the
lone client-side `error.includes('permission')` was never an auth boundary and
backend enforcement is untouched. No secrets/PII in the diff or commit message.
Pure deletions add no inputs/endpoints/handlers — zero new attack surface.
Tenant-isolation / i18n / prod-data rules are not implicated (no data-access path
or user-facing string added).

## Verification

- `npm run build` (`tsc -b` + vite build): clean — CI's stricter gate.
- `npm run test -- --run`: **119 passed** (14 files); post-#15 dev − 3 deleted smoke tests.

## Rebase note

Originally reviewed against `4ffccb7` (post-#14). Rebased onto `1917f6e` after
**#15** (share formatTime util, drop dead `SimultaneousTurnController`) merged to
`dev`. Only `ui-state.ts` conflicted; resolved as the **union** of both dead-code
removals — `SimultaneousTurnControllerProps` (removed by #15) is dropped as well,
and the then-fully-unused top-level `./race` import is removed. Re-verified:
`npm run build` clean, 119 tests pass. No logic change; verdict stands.

## Blocking items

None.

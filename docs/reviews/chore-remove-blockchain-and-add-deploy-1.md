# Review gate — chore/remove-blockchain-and-add-deploy

- Date: 2026-06-25
- Base SHA: 4161d0e | Head SHA: 7c89451 (+ uncommitted working tree)
- Spec: none — chore branch (blockchain removal + deploy infra); also carries feature WIP (boost pool, ai-solo-mode, race UI)
- Changed areas: backend + frontend
- **Verdict: BLOCK**

## Acceptance-criteria checklist (intent)
- [PASS] Remove Solana smart-contract subproject — gitlink deleted, no dangling submodule config
- [PASS] Drop wallet/NFT from domain models — `WalletAddress`, `wallet_address`, `nft_mint_address` removed (player.rs, car/pilot/engine/body, startup.rs OpenAPI)
- [PASS] Remove wallet routes + OpenAPI refs — `get_player_by_wallet`/`connect_wallet`/`disconnect_wallet` gone
- [PASS] Scrub blockchain wording — `solana|blockchain|nft|mint|web3|wallet` repo-wide grep returns zero real hits
- [PASS] Add deploy: render.yaml / Dockerfile / deploy.yml — coherent; backend-first → health-poll → frontend
- [PASS] Both halves compile — `cargo check --tests` clean, frontend `tsc --noEmit` exit 0

Removal + deploy intent: complete. The BLOCK is driven by gameplay-logic bugs in the feature WIP that rides along, not by the chore itself.

## Correctness (code-review judge) — BLOCK
- **[HIGH] rust-backend/src/routes/races.rs:1276** — Solo-race pit stop never enqueues AI actions, so the turn can't resolve and the race stalls forever. `/pit` → `process_individual_pit_action` delegates to `process_individual_lap_action(0)`, which needs `all_actions_submitted()`, but unlike `/submit-action` it never calls `enqueue_ai_actions` / `drive_ai_only_turns`. Fix: enqueue + drive AI on the pit path as `submit_player_action_in_db` does.
- **[HIGH] rust-backend/src/routes/races.rs:4388** — On `/submit-action` the human's boost card is never consumed, so the human's tyre pool never depletes — the headline pit/tyre-pool mechanic is inert for the human. AI cards are consumed; humans only via `process_individual_lap_action` (wired to `/apply-lap`). Fix: consume the human's card on the submit path, reconcile the two turn-resolution paths.
- **[HIGH] rust-backend/src/routes/players.rs:477** — `add_car`/`remove_car`/`add_pilot`/`remove_pilot`/`update_configuration` query MongoDB, but registered players live only in the in-memory `MockPlayerRepository` → 404 for every real player. Root cause: two stores (Mongo vs mock). Fix: unify these handlers onto the same store registration uses.
- **[LOW] empty-project/src/services/raceAPI.ts:328** — Two divergent race-API clients (`services/raceAPI.ts` vs `utils/raceAPI.ts`); new methods + boost shapes added only to `services/`, deepening duplication. Consolidate and delete `utils/raceAPI.ts`.
- **[LOW] rust-backend/src/domain/ai_player.rs:19** — `classify_movement` duplicates `BoostHandManager::calculate_movement_probability`; doc claims a shared source of truth but the preview endpoint still uses its private copy. Extract one shared fn.

Shared root causes for the 3 HIGHs: the split turn-resolution paths (`process_individual_lap_action` vs `submit_player_action_in_db`) and the split data stores (Mongo vs mock). Fix at that altitude.

## Security (security-review judge + Always/Never) — PASS with notes
- **[PASS] Secrets/deploy config** — render.yaml uses `sync:false` + `generateValue:true`; deploy.yml uses `${{ secrets.* }}`; DB URI wrapped in `Secret<String>`; frontend base URL via `VITE_API_BASE_URL`. No committed secrets.
- **[PASS] Secrets/PII in logs** — logs key off UUIDs only; blockchain removal deleted several `wallet_address`-in-log statements (net reduction).
- **[PASS] Test integrity** — no `.skip`/`.only`/`#[ignore]` added; net test count unchanged; boost tests adapted to the new mechanic, not weakened.
- **[MEDIUM, carry-forward] Tenant isolation** — `team_routes()` (GET/PUT/DELETE `/players/:uuid`) and `turn_routes`/`pit`/`create_solo_race` have no `AuthMiddleware`/ownership filter; any caller can act on any player UUID. **Not a regression** — same endpoints were equally unauthenticated on base; the branch only moves them. Track before real multi-tenant traffic: add auth + ownership filter + cross-tenant negative test.
- **[LOW] base.yaml** — committed plaintext dev DB password `rust_password`; `.dockerignore` doesn't exclude `configuration/`, so it ships in the image. Harmless (env overrides) but move it to the git-ignored `local.yaml`.

## Scope / hygiene (conformance judge)
- **[MEDIUM]** Chore branch mixes in substantial unfinished feature WIP (race.rs +1224, races.rs +787, new ai_player.rs, 2 new specs) wired into `startup.rs`. Buildable, but consider splitting the feature into its own PR.
- **[MEDIUM]** `App.tsx:15,27` ships a self-described DEV-ONLY `/preview-race` route (`PreviewRacePage.tsx`). Confirm intent or gate it.
- **[LOW]** Untracked `.claude/`, `CLAUDE.md` (×3), `docs/reviews/` in tree — confirm whether they belong in the commit.
- **Coverage gap:** confirm unit coverage for the new `ai_player.rs` solo-bot logic.

## Blocking items (must fix before PR)
1. races.rs:1276 — enqueue + drive AI on the solo pit path (turn deadlock).
2. races.rs:4388 — consume the human boost card on `/submit-action` (inert tyre-pool mechanic).
3. players.rs:477 — unify the Mongo vs mock player store (add-car/pilot/config 404 for real players).

## Non-blocking notes (track)
- Consolidate the two raceAPI clients; dedupe movement classification.
- Add auth + ownership + cross-tenant test to player/race routes before multi-tenant traffic.
- Remove dev DB password from committed base.yaml / exclude `configuration/` from image.
- Decide whether the boost/AI-solo WIP and the DEV-only preview route belong in this PR.

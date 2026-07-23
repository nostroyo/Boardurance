# Migration: `.kiro/specs/` → `openspec/`

Date: 2026-07-04. OpenSpec CLI: `@fission-ai/openspec` **1.5.0**.

The 11 legacy Kiro-style feature specs on `dev` (plus one uncommitted one, see
`mongo-persistence` below) were consolidated into 8 **capability** specs under
`openspec/specs/`, each **verified against the current code** — the spec
describes what the code does today, not what the legacy spec promised. `.kiro/`
is frozen read-only history; never edit it.

Verdicts: **KEPT** (code matches the legacy requirement), **CHANGED** (code
drifted; the new spec describes the code), **DROPPED** (never implemented or
since removed — excluded from current truth; still-wanted items should become
`openspec/changes/` proposals), **SUPERSEDED** (replaced by a later legacy
spec's version), **NEW** (current behavior no legacy spec covered).

## Capability map

| Capability | Legacy sources |
|---|---|
| `auth` | auth-middleware |
| `race-engine` | single-player-race-mvp, race-api-refinement, backend-race-api-enhancements |
| `boost-system` | game-boost-improvements, tyre-boost-pool |
| `ai-opponents` | ai-solo-mode |
| `admin-management` | admin-race-management |
| `race-ui` | player-game-interface, race-interface-redesign |
| `persistence` | mongo-persistence (uncommitted legacy spec) |
| `ci-cd` | github-cicd-integration |

## persistence ← mongo-persistence

Note: the legacy `mongo-persistence` spec was never committed to `dev` (it is
untracked work-in-progress in another session's checkout). None of its Mongo
implementation shipped on `dev`; the verified spec describes the actual
in-memory state.

| legacy req (spec/#) | verdict | new requirement name (or —) | reason/delta |
|---|---|---|---|
| mongo-persistence/1 (Mongo repositories) | DROPPED | — | No `Mongo*Repository` exists on dev; only traits + mocks |
| mongo-persistence/2 (storage_backend config) | DROPPED | — | `DatabaseSettings` has no `storage_backend` field |
| mongo-persistence/3 (mock/Mongo parity) | DROPPED | — | Nothing to be par with; no Mongo impl |
| mongo-persistence/4 (fail-fast in prod, no degraded fallback) | DROPPED | — | Opposite is current truth: startup degrades to a `mock_database` client on connection failure |
| mongo-persistence/5 (idempotent seeding) | DROPPED | — | `seed_solo_bots` seeds an in-memory repo per process; idempotency across redeploys is moot without persistence |
| mongo-persistence/6 (unique indexes) | DROPPED | — | No Mongo collections in use |
| mongo-persistence/7 (races through RaceRepository) | DROPPED | — | `RACE_STORE` process-global static is still the live race store (routes/races.rs:18) |
| — | NEW | Repository abstraction | Traits + `RepositoryError{NotFound,Validation,Conflict}` are current truth |
| — | NEW | In-memory storage is the active backend | Mocks wired for all environments; data lost on restart |
| — | NEW | Graceful degradation without MongoDB | Warn + short-timeout `mock_database` client; gameplay unaffected |
| — | NEW | Health check reports database state | `/health_check` 200 with `ok`/`degraded` |
| — | NEW | Database configuration | Layered YAML + `APP_DATABASE__URI` override precedence |
| — | NEW | Solo bot seeding | `seed_solo_bots` at startup |

The DROPPED block above (real MongoDB persistence, fail-fast prod startup,
unique indexes, race-route rewiring onto `RaceRepository`) is still wanted —
it should be re-proposed as an `openspec/changes/` change when the work
resumes.

## auth ← auth-middleware

| legacy req (spec/#) | verdict | new requirement name (or —) | reason/delta |
|---|---|---|---|
| auth-middleware/1.1 (login generates JWT with identity+role) | KEPT | Login issues a JWT cookie pair | `login_user` → `generate_token_pair`; claims carry sub/email/role |
| auth-middleware/1.2 (middleware validates token on protected endpoints) | CHANGED | Authentication middleware; Enforcement scope is admin routes only | Middleware works but is layered only on `/api/v1/admin`, an empty router — no live endpoint is protected |
| auth-middleware/1.3 (expired/invalid → 401) | KEPT | Authentication middleware | 401 JSON with per-cause error codes (`token_expired`, `invalid_token`, …) |
| auth-middleware/1.4 (attach user context) | KEPT | Authentication middleware | `UserContext` inserted into request extensions |
| auth-middleware/1.5 (token refresh) | CHANGED | Access token refresh | `/auth/refresh` exists (cookie-only) but the refresh token is never rotated; only a new access cookie is issued |
| auth-middleware/2.1 (admin endpoints verify admin role) | CHANGED | Role-based authorization; Enforcement scope | `RequireRole::admin()` wired in startup.rs, but `admin_routes()` contains zero routes (commented out) |
| auth-middleware/2.2 (player-resource ownership) | DROPPED | — | `RequireOwnership` exists as dead code, wired nowhere; player/team routes trust the path `player_uuid` (explicit TODO) |
| auth-middleware/2.3 (403 on insufficient permissions) | KEPT | Role-based authorization | 403 `insufficient_permissions` from `RequireRole` (ownership path would return 404 by design, but is unwired) |
| auth-middleware/2.4 (proceed on success) | KEPT | Role-based authorization | Inner service called after role check passes |
| auth-middleware/2.5 (resource ownership validation) | DROPPED | — | `validate_race_ownership` allows any authenticated user (TODO), and nothing uses it |
| auth-middleware/3.1 (logout invalidates token) | CHANGED | Logout always clears cookies | Logout is best-effort cookie deletion; handler passes raw JWT (not `jti`) to `invalidate_session`, so server-side invalidation never matches |
| auth-middleware/3.2 (mass token invalidation) | DROPPED | — | `invalidate_all_user_sessions` exists on `SessionManager` but no endpoint or caller uses it |
| auth-middleware/3.3 (track usage / detect suspicious activity) | DROPPED | — | Only IP/user-agent stored at session creation; no tracking or detection logic |
| auth-middleware/3.4 (refresh invalidates old, issues new) | CHANGED | Access token refresh | Old session invalidated by `jti` + new session created, but refresh token not rotated |
| auth-middleware/3.5 (token blacklist) | DROPPED | — | `is_token_blacklisted` is a stub returning false; cache-only blacklist never populated outside tests |
| auth-middleware/4.1 (expiry + security claims) | KEPT | JWT claims and validation | 30min/30d expiry; iss/aud/jti enforced on validation |
| auth-middleware/4.2 (HTTPS-only in production) | DROPPED | — | Cookies built with `.secure(false)` (dev TODO); no HTTPS enforcement anywhere |
| auth-middleware/4.3 (secure signing algorithms) | CHANGED | JWT claims and validation | HS256 shared secret from `JWT_SECRET` env with insecure hardcoded fallback (design wanted RS256) |
| auth-middleware/4.4 (security event logging) | DROPPED | — | Only generic `tracing` info/warn/error; no dedicated security event log |
| auth-middleware/4.5 (per-user rate limiting) | DROPPED | — | Not implemented anywhere in the backend |
| auth-middleware/5.1 (standardized auth error responses) | KEPT | Authentication middleware | Uniform `{"error", "message"}` JSON with mapped status codes |
| auth-middleware/5.2 (clear authorization error messages) | KEPT | Role-based authorization | 403 `insufficient_permissions` with human-readable message |
| auth-middleware/5.3 (log without exposing sensitive info) | KEPT | — (folded into endpoint reqs) | Handlers log UUIDs, not credentials/emails |
| auth-middleware/5.4 (refresh indicators in headers) | DROPPED | — | No response-header refresh hint; frontend retries once on 401 instead |
| auth-middleware/5.5 (extra debug context in dev) | DROPPED | — | No environment-conditional error detail exists |
| — | NEW | User registration | `/auth/register` with starter assets (2 cars, 6 pilots, 2 engines, 2 bodies), 409 on duplicate, auto-login |
| — | NEW | Password policy and hashing | Argon2 + salted hashes, 8–128 chars with upper/lower/digit, generic 401 `Invalid credentials` |
| — | NEW | Session tracking with a per-user cap | 5-session cap → auth endpoint returns 500 `Session creation failed`; 24h timeout; in-memory repo |
| — | NEW | Frontend client-side auth gating | localStorage `authState`, `credentials: 'include'` disabled (cookies never sent), `ProtectedRoute`/`AdminRoute` (`role === 'Admin'` exactly — SuperAdmin denied client-side) |

## ai-opponents ← ai-solo-mode

| legacy req (spec/#) | verdict | new requirement name (or —) | reason/delta |
|---|---|---|---|
| ai-solo-mode/1 (is_ai flag, serde default, AI join path) | KEPT | AI participant flag | `race.rs:283` `#[serde(default)] is_ai`; `add_ai_participant(_with_tyre)` wraps the same `add_participant_inner` |
| ai-solo-mode/2 (Boost_Brain balanced profile, additive model, deterministic) | KEPT | Deterministic balanced boost decision | `ai_player.rs::choose_boost` implements the legacy 3-step profile; all 7 ACs verified against code + unit tests |
| ai-solo-mode/3 (auto-submit AI on human submission) | CHANGED | AI turn enqueueing | Now via unified `resolve_human_turn` covering BOTH `/submit-action` and `/pit` (solo-pit deadlock fix); enqueueing consumes the AI's card via `BoostHandManager`; AI can also decide to Pit |
| ai-solo-mode/4.1/4.3/4.4/4.5 (seed bots, POST /races/solo, frontend entry) | KEPT | Seeded bot players; Solo race bootstrap endpoint | `seed_solo_bots` (2 bots, starter assets); `create_solo_race` builds fixed Solo Circuit, 5 laps, auto-InProgress; `GameLobby.tsx` → `raceAPI.createSoloRace` |
| ai-solo-mode/4.2 ("slightly varied car builds") | CHANGED | Seeded bot players (variation clause) | Bots get identical starter builds; variation = different primary pilot per bot + per-race cycling tyres, not varied builds |
| ai-solo-mode/5 (race runs to Finished, never hangs on AI) | CHANGED | AI-only auto-advance | Guarantee kept via new `drive_ai_only_turns` (post-human-finish synthesis, bounded 1000 turns / 10s) |
| ai-solo-mode/6 (preview/resolution additive-model parity) | DROPPED | — | Belongs to boost-system scope; code survives (`ai_player::classify_movement` shared classifier) |
| — | NEW | AI pit decision | `decide_ai_action`: pit only when pool empty AND `laps_remaining > 1` AND a refilled card would help |
| — | NEW | Solo race bootstrap endpoint (tyre clause) | AI opponents fitted with cycling Soft/Medium/Hard tyres; human picks `tyre_type` (default Medium) |
| — | NEW | Solo race bootstrap endpoint (grid clause) | All participants normalized to sector 0 in stable seat order, overriding random qualification |

## race-engine ← single-player-race-mvp (MVP) + race-api-refinement (RAR) + backend-race-api-enhancements (BRAE)

| legacy req (spec/#) | verdict | new requirement name (or —) | reason/delta |
|---|---|---|---|
| MVP/1-9, MVP/12 | DROPPED | — | All frontend display/UX requirements — out of this capability's scope, covered by `race-ui` (backend data they consume covered by the endpoint requirements below) |
| MVP/10 (backend-authoritative game logic) | SUPERSEDED | Player-scoped race view endpoints; Simultaneous turn resolution | Folded into the backend requirements; frontend-side prohibition lives in `race-ui` |
| MVP/11 (endpoint list) | SUPERSEDED | Player-scoped race view endpoints; Turn phase reporting; Single shared human-turn resolution path | Duplicated BRAE R1-R6; folded into surviving endpoint requirements |
| RAR/1 (registration) | CHANGED | Player registration | Kept, plus lap-1 late join into `InProgress` races and optional `tyre_type` |
| RAR/2 (detailed status ≤2s) | CHANGED | Detailed race status reporting | Shape kept; latency trivially satisfied (in-memory); player performance metrics placeholder (base 10) |
| RAR/3 (lap processing, boost 0-5) | CHANGED | Alternate lap-processing endpoints; Simultaneous turn resolution | Boost range 0-4 cards; lap increments via turns-taken model |
| RAR/4 (sector capacity/movement indicators) | CHANGED | Detailed race status reporting | Capacity/occupancy/ranking kept; `recent_movements` always empty; finished participants excluded |
| RAR/5 (performance preview, unlimited 0-5) | CHANGED | Player-scoped race view endpoints | Ceiling-before-boost kept; availability now card/tyre-pool based 0-4; MoveUp/Stay/MoveDown probabilities |
| RAR/6 (apply-lap atomicity + status format) | KEPT | Alternate lap-processing endpoints; Races live in the process-global race store | Atomicity via the `RACE_STORE` mutex |
| RAR/7 (consistent errors) | KEPT | Consistent error contract | Standardized JSON + error codes via utoipa/`ErrorResponse` |
| RAR/8 (race metadata/events/admin alerts) | CHANGED | Detailed race status reporting | Partial: start time/laps kept; `estimated_completion`/`total_turns` stubs; admin alerts never implemented |
| BRAE/1 (car-data endpoint) | KEPT | Player-scoped race view endpoints | Full engine/body/pilot stats + skills breakdown; 404 for absent player |
| BRAE/2 (boost multiplier preview) | CHANGED | Player-scoped race view endpoints | Multiplier `base*(1+boost*0.08)` replaced by additive `min(base,ceiling)+boost`; cycle info → tyre-pool info |
| BRAE/3 (turn phase endpoint) | CHANGED | Turn phase reporting | Submitted/pending lists + lap info kept; `Processing` phase never emitted (synchronous resolution) |
| BRAE/4 (local 5-sector view) | KEPT | Player-scoped race view endpoints | ±2 with modulo wrapping, sector details, sorted participants |
| BRAE/5 (boost hand state) | CHANGED | Player-scoped race view endpoints | Cycle/replenishment fields replaced by `tyre_type`/`pit_stops_completed`/`cards_remaining` (see boost-system) |
| BRAE/6 (lap history) | CHANGED | Player-scoped race view endpoints | Boost-used kept; per-lap base/final and from/to-sector are unstored placeholders; characteristic reconstructed by lap parity |
| BRAE/7 (turn resolution validation) | CHANGED | Simultaneous turn resolution | Ceiling + 0-4 validation + card checks kept; boost multiplier gone (additive model) |
| BRAE/8 (error codes) | KEPT | Consistent error contract | All codes implemented; deviation: status-detailed with a finished player yields 500, not 409 |
| BRAE/9 (OpenAPI docs) | KEPT | Consistent error contract | All endpoints in utoipa with schemas/examples |
| BRAE/10 (integration tests for new endpoints) | DROPPED | — | Never written (tasks 10.1-10.7 unchecked); coverage is unit tests + e2e steps |
| — | NEW | Races live in the process-global race store | `RACE_STORE` static is the actual persistence |
| — | NEW | Race creation auto-starts the race | `POST /races` sets `InProgress` immediately |
| — | NEW | Solo race bootstrap | `POST /races/solo`: fixed circuit, 5 laps, seeded AI grid, auto-start |
| — | NEW | Race listing and retrieval | `GET /races`, `/races/{uuid}`, `/races/{uuid}/status` |
| — | NEW | Race lifecycle states | Waiting→InProgress→Finished + `/start` preconditions |
| — | NEW | One boost, one turn, one lap | turns_taken model, simultaneous finish, finish-position ordering, safety cap |
| — | NEW | Sector movement is relative standings | Best-to-worst resolution, leader-only move-up, capacity, no wrap at top |
| — | NEW | Single shared human-turn resolution path | `resolve_human_turn` shared by submit-action/pit, AI enqueue + bounded auto-advance |

## boost-system ← game-boost-improvements + tyre-boost-pool

| legacy req (spec/#) | verdict | new requirement name (or —) | reason/delta |
|---|---|---|---|
| game-boost-improvements/1 (5 fixed cards 0-4, once each) | SUPERSEDED | Tyre-defined boost pools; Card consumption without auto-replenish | Fixed `{0..4}` hand replaced by tyre multiset of values 1-4; boost 0 no longer a card |
| game-boost-improvements/2 (auto-replenish on exhaustion) | SUPERSEDED | Pit stop refills the pool and costs the turn | Auto-replenish removed; only pit `refill()` restores cards; cycle counter → `pit_stops_completed` |
| game-boost-improvements/3 (see available vs used cards) | CHANGED | Boost state visibility in API responses | Boolean per-card state → count map `hand_state` + `available_cards` + `tyre_type` + `cards_remaining` |
| game-boost-improvements/4 (usage history) | CHANGED | Boost usage history per pit segment | `BoostUsageRecord` kept; `cycle_number` now records pit segment; `replenishment_occurred` hardwired `false` |
| game-boost-improvements/5 (selection validation) | KEPT | Boost selection validation | Rejects >4 and spent cards with available list; double-spend blocked by one-action-per-turn 409 |
| game-boost-improvements/6 (boost state in API responses) | CHANGED | Boost state visibility in API responses | Replenishment flag + cycle count dropped, replaced by tyre/pit-count/count-based state |
| game-boost-improvements/7 (usage analytics) | CHANGED | Boost usage history per pit segment | Partial: per-segment summaries, totals, averages exist; most-used-card/efficiency/cross-player analysis never built |
| game-boost-improvements/8 (integration with race mechanics) | KEPT | Boost impact preview (+ consumption/validation reqs) | Additive model `final = min(base, sector max) + boost`; preview shows ALL cards 0-4 with `is_available` flag |
| tyre-boost-pool/1 (tyre-defined pools) | KEPT | Tyre-defined boost pools | Soft `[3,4,4]`, Medium `[2,2,3,3,4]`, Hard `[1,1,1,2,2,3]` verified in code |
| tyre-boost-pool/2 (boost 0 free) | KEPT | Boost 0 is the free always-available move | Always available, free no-op, listed first |
| tyre-boost-pool/3 (no auto-replenish) | KEPT | Card consumption without auto-replenish | Empty pool → only `[0]` until pit; unit-tested |
| tyre-boost-pool/4 (tyre at registration, default Medium) | KEPT | Tyre selection at race entry | serde-default `tyre_type` on register/solo-create requests |
| tyre-boost-pool/5 (pit stop) | KEPT | Pit stop refills the pool and costs the turn | `POST /races/{uuid}/pit`; now funnels through `resolve_human_turn` (solo-pit deadlock fix) |
| tyre-boost-pool/6 (validation) | KEPT | Boost selection validation | `InvalidBoostValue` / `CardNotAvailable` with available list; 0 always accepted |
| tyre-boost-pool/7 (visibility) | KEPT | Boost state visibility in API responses | tyre + per-value counts + pit count replace cycle counters |
| — | NEW | Pool rules apply uniformly to AI participants | AI consume/refill via the same paths; decision policy in ai-opponents |
| — | NEW | Boost impact preview | Additive prediction + movement classification for all cards 0-4, now first-class |

## admin-management ← admin-race-management

| legacy req (spec/#) | verdict | new requirement name (or —) | reason/delta |
|---|---|---|---|
| admin/1.1 (admin nav menu) | KEPT | Admin dashboard shell | AdminDashboard has its own two-view nav (desktop+mobile) |
| admin/1.2 (verify admin role) | CHANGED | Client-side admin route protection | AdminRoute checks `role === 'Admin'` only — client-side, and `SuperAdmin` is rejected |
| admin/1.3 (non-admin redirect) | CHANGED | Client-side admin route protection | In-place Access Denied screen, not a redirect; only unauthenticated users redirected to /login |
| admin/1.4 (race mgmt dashboard) | KEPT | Admin dashboard shell | RaceDashboard is the default view |
| admin/1.5 (react to admin status changes) | DROPPED | — | No dynamic role re-check; role read once from auth context |
| admin/2.1 (name/track/laps fields) | KEPT | Race creation with JSON track upload | Laps clamped 1–100 |
| admin/2.2 (JSON file upload) | KEPT | Race creation with JSON track upload | JSONUploader, .json only, ≤1MB |
| admin/2.3 (JSON schema validation) | KEPT | Race creation with JSON track upload | Per-field checks + unique ids, ≥2 sectors |
| admin/2.4 (first/last infinite capacity) | KEPT | Race creation with JSON track upload | Start/Finish must have `slot_capacity: null` |
| admin/2.5 (send Vec<Sector> to API) | KEPT | Race creation with JSON track upload | POST /api/v1/races with sectors array |
| admin/2.6 (fallback manual track builder) | DROPPED | — | No TrackBuilder shipped; error message + downloadable sample JSON only |
| admin/3.1-3.2 (dashboard lists races) | KEPT | Race dashboard listing and control | RaceCard: name, track, status, participants, lap progress |
| admin/3.3 (auto/manual refresh) | KEPT | Race dashboard listing and control | Manual Refresh button only (spec allowed either) |
| admin/3.4 (select race → details) | CHANGED | Race dashboard listing and control | "View Details" is a console.log stub; only Start works |
| admin/3.5 (visual status indicators) | KEPT | Race dashboard listing and control | Color/icon badges + status filter counts |
| admin/4.1-4.5 (real-time race monitor, LOW PRI) | DROPPED | — | No Race_Monitor or real-time admin view exists |
| admin/5.1 (participant details) | CHANGED | Race dashboard listing and control | Only participant count on card; no detail view |
| admin/5.2 (start controls for Waiting races) | KEPT | Race dashboard listing and control | Start button only when Waiting + ≥1 participant |
| admin/5.3-5.5 (participant mgmt/interventions) | DROPPED | — | No admin controls beyond start ever shipped |
| admin/6.1-6.2 (Vec<Sector> schema) | KEPT (elsewhere) | — | Domain structs match; data model belongs to race-engine, storage to persistence |
| admin/6.3-6.5 (Mongo BSON/indexing/versioning) | DROPPED | — | Active storage is in-memory RACE_STORE; no Mongo race writes |
| admin/7.1 (JWT admin role validated) | CHANGED | Admin role model; Role-based authorization middleware; Race mgmt endpoints not server-side admin-gated | Role claims + RequireRole middleware shipped, but race endpoints mounted with ZERO auth — enforcement never wired |
| admin/7.2 (session expiry redirect) | DROPPED (here) | — | Generic session behavior — auth capability |
| admin/7.3 (secure API communication) | CHANGED | Race mgmt endpoints not server-side admin-gated | Frontend sends race requests with no credentials |
| admin/7.4 (log security events) | DROPPED | — | No security-event logging |
| admin/7.5 (immediate revocation) | DROPPED | — | Never implemented |
| — | NEW | Admin API namespace exists but exposes no endpoints | `/api/v1/admin` mounted with Auth+RequireRole layers but empty router (handlers commented out) |
| — | NEW | Out-of-band admin account provisioning | `create_admin` bin seeds Mongo directly; unusable against in-memory runtime — no in-app admin path |
| — | NEW | Admin role model | UserRole enum, default Player, `is_admin()` covers Admin+SuperAdmin, role in JWT + payloads |

## race-ui ← player-game-interface (PGI) + race-interface-redesign (RIR)

| legacy req (spec/#) | verdict | new requirement name (or —) | reason/delta |
|---|---|---|---|
| PGI/1 (local view ±2, status panel, 2s refresh) | SUPERSEDED | Bird's-eye local track view; Race status display | ±2-sector semantics kept, rendered by the redesign (`TrackDisplayRedesign`/`SectorGrid`); refresh is post-turn batch + 2s turn-phase polling |
| PGI/2 (boost 0-5 submit flow) | CHANGED | Boost selection panel; Turn validation and submission | Range is 0-4; availability from backend card hand; confirm dialog; predicted-value display (2.3) not rendered |
| PGI/3 (performance calculation display) | DROPPED | — | `PerformanceCalculator`/`PerformancePreview` exist but are not wired into the active `RaceInterface` |
| PGI/4 (animated sector movements) | DROPPED | — | `CarMovementAnimation` exists but `RaceContainer` passes `animationState={undefined}` ("TODO"); instant updates |
| PGI/5 (detailed PlayerCarCard + boost simulation) | SUPERSEDED | Race status display (Your Car panel); Race completion and results | Active UI shows only names; full card/history/simulation not wired; lap-history stats on completion screen |
| PGI/6 (phase notifications, processing, lap summary, finish, errors) | CHANGED | Race error handling; Turn resolution flow; Race completion and results | Processing indicator, finish screen, error panel kept; phase-change notification (6.1) and lap-summary popup (6.3) unwired |
| PGI/7 (streamlined single-view layout) | KEPT | Bird's-eye local track view; Race status display | Single-view responsive layout with 5-sector focus is current |
| RIR/1 (bird's eye view, grids, sprites, player centered ±2) | KEPT | Bird's-eye local track view; Car sprites | All verified incl. auto-centering scroll; lead sector displayed as "Sector 1" (`total_sectors - id`) |
| RIR/2 (even layout, borders, hierarchy, capacity/value ranges) | KEPT | Bird's-eye local track view | Occupancy/capacity indicators (∞/✓/⚠️/🚫) and value-range styling present |
| RIR/3 (boost buttons 0-5, available vs used, feedback, validate) | CHANGED | Boost selection panel; Turn validation and submission | Buttons 0-4; per-value remaining-count badges from `hand_state`; confirmation dialog |
| RIR/4 (8-bit sprites, colors, smooth movement, highlight, slots) | CHANGED | Car sprites | 4.1/4.2/4.4/4.5 kept (UUID-hash palettes, gold player, slots); 4.3 movement animation dropped |
| RIR/5 (preserve all existing functionality) | KEPT | Race status display; Race error handling; Race completion and results | All present in redesigned UI |
| RIR/7 (prominent boost panel, labels 0-5, availability, CTA) | CHANGED | Boost selection panel | Panel kept; labels 0-4; boost 0 marked "Free"; CTA = header text + validation error |
| — | NEW | Lobby race list | Status badges, join/enter/view logic, 3-pilot car requirement |
| — | NEW | Solo race creation with starting tyre | Tyre selector at solo creation; stale-session 404/401 → logout+login redirect |
| — | NEW | Pit stop control | Pit & refill button with tyre dropdown, disabled off-turn |
| — | NEW | Race initialization and loading feedback | Staged progress loading, non-critical follow-up fetches |
| — | NEW | Turn resolution flow | Immediate `TurnProcessed` solo path vs 2s/60-attempt polling with backoff |
| — | NEW | Leaving an active race | Return-to-lobby confirmation + `beforeunload` guard |

## ci-cd ← github-cicd-integration

| legacy req (spec/#) | verdict | new requirement name (or —) | reason/delta |
|---|---|---|---|
| github-cicd/1.1 (GitHub Secrets store all sensitive values) | CHANGED | No secrets in the repository | App secrets moved to Render dashboard (`sync: false`); GitHub secrets hold only deploy hooks/URLs |
| github-cicd/1.2 (secret scanner blocks pushes) | DROPPED | — | No secret-scanning step; only advisory `cargo audit`/`npm audit` |
| github-cicd/1.3 (only example config files) | KEPT | No secrets in the repository | `.env.example` files committed; `.gitignore` blocks `.env*`, secret YAMLs |
| github-cicd/1.4 (env vars from GitHub secrets in CI) | CHANGED | Backend CI gate / Production deploys from main | CI needs only `APP_ENVIRONMENT: test`; secrets used solely by deploy jobs |
| github-cicd/1.5 (separate secret sets staging/prod) | KEPT | Environment isolation via per-service secrets | `preprod`/`production` GitHub Environments + per-service DB name, JWT_SECRET, origins |
| github-cicd/2.1 (frontend pipeline on push to main) | CHANGED | Frontend CI gate | Push/PR to `main` AND `dev`, path-filtered |
| github-cicd/2.2 (ESLint/Prettier/TS blocking) | CHANGED | Frontend CI gate | `tsc --noEmit` blocking; ESLint/Prettier demoted to advisory |
| github-cicd/2.3 (Vite production build) | KEPT | Frontend CI gate | `npm run build` blocking |
| github-cicd/2.4 (Vitest tests) | KEPT | Frontend CI gate | `npm run test -- --run` blocking |
| github-cicd/2.5 (auto-deploy frontend to staging) | CHANGED | Preprod auto-deploys from dev | Staging→preprod; deploys from `dev` via Render deploy hook, after backend health |
| github-cicd/2.6 (manual prod deploy after approval) | CHANGED | Production deploys from main | Prod auto-deploys on push to `main`; no approval gate in repo config |
| github-cicd/3.1 (backend pipeline on push to main) | CHANGED | Backend CI gate | Push/PR to `main` and `dev`, path-filtered |
| github-cicd/3.2 (clippy/rustfmt/compilation) | CHANGED | Backend CI gate | Hardened: `-D warnings` + enumerated allowances, mirrored in CLAUDE.md |
| github-cicd/3.3 (comprehensive suite incl. integration/property tests) | CHANGED | Backend CI tests run without a database | CI runs only `cargo test-fast`; integration local-only; property tests dropped |
| github-cicd/3.4 (build Docker in pipeline) | DROPPED | — | Render builds the image at deploy time from `rust-backend/Dockerfile` |
| github-cicd/3.5 (push to Docker registry) | DROPPED | — | No registry; Render builds from the branch |
| github-cicd/3.6 (deploy staging with Mongo) | CHANGED | Ordered deploy with backend health gate | Gate requires `"status":"ok"` (DB connected), not just HTTP 200 |
| github-cicd/3.7 (manual prod deploy) | CHANGED | Production deploys from main | Automatic on merge to `main` |
| github-cicd/4.1 (block merge on lint/format failure) | CHANGED | Branch protection on main / Frontend CI gate | Backend blocks; frontend ESLint/Prettier advisory |
| github-cicd/4.2 (80% coverage minimum) | DROPPED | — | No coverage measurement anywhere |
| github-cicd/4.3 (PRs run all checks) | KEPT | Backend/Frontend CI gates | `pull_request` triggers on both |
| github-cicd/4.4 (tests pass before merge to main) | KEPT | Branch protection on main | Required checks `frontend-ci`+`backend-ci`, strict, 1 review |
| github-cicd/4.5 (publish coverage reports) | DROPPED | — | No coverage reporting |
| github-cicd/4.6 (no Rust warnings) | KEPT | Backend CI gate | Clippy `-D warnings` (with allowance list) |
| github-cicd/5.1 (separate staging/prod environments) | KEPT | Render two-environment topology | Two envs (preprod/prod), four services — render.yaml defines NO third "test" env |
| github-cicd/5.2 (merge to main deploys staging) | CHANGED | Preprod auto-deploys from dev | Inverted: `dev`→preprod, `main`→prod |
| github-cicd/5.3 (manual approval for production) | DROPPED | — | No required-reviewer config in-repo |
| github-cicd/5.4 (blue-green deployment) | DROPPED | — | Plain Render rollout |
| github-cicd/5.5 (rollback capabilities) | DROPPED | — | No rollback automation |
| github-cicd/5.6 (health monitoring + auto-rollback) | CHANGED | Ordered deploy with backend health gate | Health polling (80×15s for `status:ok`) fails the workflow; no auto-rollback |
| — | NEW | Backend CI tests run without a database | `.cargo/config.toml` aliases; `test-fast` is the DB-free CI gate |
| — | NEW | Local CI parity | CLAUDE.md verify loops mirror workflows; `be.ps1`/`fe.ps1` wrappers |
| — | NEW | Ordered deploy with backend health gate | Backend-first, `status:ok` gate, concurrency groups |
| — | NEW | Render two-environment topology | 4 services, `autoDeploy: false`, SPA rewrite, Node 20 pin |
| — | NEW | Environment isolation via per-service secrets | Shared cluster + distinct DB names, per-env JWT_SECRET, per-env VITE_API_BASE_URL |
| — | NEW | Frontend CI gate (gen:api:check) | OpenAPI contract drift check is a blocking CI step |

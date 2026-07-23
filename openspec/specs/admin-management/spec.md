# admin-management

## Purpose

The admin role model, role-based authorization machinery, admin account
tooling, and the admin race-management UI. Sources of truth:
`rust-backend/src/domain/auth.rs` (`UserRole`), `rust-backend/src/domain/player.rs`,
`rust-backend/src/services/jwt.rs`, `rust-backend/src/middleware/auth.rs`,
`rust-backend/src/middleware/ownership.rs`, `rust-backend/src/routes/players.rs`
(`admin_routes`), `rust-backend/src/startup.rs`, `rust-backend/src/bin/create_admin.rs`;
frontend: `empty-project/src/components/AdminRoute.tsx`,
`empty-project/src/components/AdminDashboard.tsx`,
`empty-project/src/components/admin/` (`RaceCreator`, `JSONUploader`,
`RaceDashboard`, `RaceCard`).

Scope note: this spec describes what is admin-specific today. Server-side
admin enforcement of race management never shipped (see the last two
requirements); adding it is a future OpenSpec change proposal, not current
truth. Regular player authentication is the `auth` capability; race mechanics
are the `race-engine` capability.

## Requirements

### Requirement: Admin role model

The backend SHALL define user roles as the enum `UserRole` with variants
`Player` (default), `Admin`, and `SuperAdmin` (`domain/auth.rs`).
`UserRole::is_admin()` SHALL return true for both `Admin` and `SuperAdmin`.
Every `Player` SHALL carry a `role` field that defaults to `Player` on
registration; the only mutation path is `Player::update_role`, which SHALL
also refresh `updated_at`. The role SHALL be embedded in JWT claims
(`Claims.role`, `services/jwt.rs`) and returned in the `user` object of
register and login responses (`routes/auth.rs`).

#### Scenario: New registrations are plain players

- GIVEN a user registers through the API
- WHEN the player is created
- THEN its role is `UserRole::Player` (no in-app path grants `Admin`)

#### Scenario: Role travels in the token

- GIVEN a player with role `Admin`
- WHEN an access token is generated for them
- THEN the JWT claims include `role: Admin`, and the auth middleware exposes
  it on `UserContext.role`

### Requirement: Role-based authorization middleware

The backend SHALL provide a `RequireRole` layer (`middleware/ownership.rs`)
that, for an admin requirement, authorizes the request only when
`UserContext.role.is_admin()` is true (so `Admin` and `SuperAdmin` both pass).
It SHALL respond `401` with error code `authentication_required` when no
`UserContext` is present (auth middleware did not run), and `403` with error
code `insufficient_permissions` when the authenticated role lacks admin
privileges. The companion `RequireOwnership` layer SHALL let any admin bypass
player-ownership checks (`validate_player_ownership` returns true for
admins) and SHALL answer failed ownership checks with `404` rather than `403`
to avoid leaking resource existence. `RequireOwnership` is implemented and
unit-tested but is not currently layered onto any live route.

#### Scenario: Non-admin hits an admin-gated route

- GIVEN an authenticated user whose role is `Player`
- WHEN they request a route behind `RequireRole::admin()`
- THEN the response is `403` with error `insufficient_permissions`

#### Scenario: Unauthenticated request behind the role layer

- GIVEN a request with no valid user context
- WHEN it reaches a `RequireRole`-protected route
- THEN the response is `401` with error `authentication_required`

### Requirement: Admin API namespace exists but exposes no endpoints

The application SHALL nest an admin router at `/api/v1/admin` wrapped in
`AuthMiddleware` plus `RequireRole::admin()` (`startup.rs`). However,
`players::admin_routes()` SHALL currently return an empty router: the admin
player-management handlers (`get_all_players_admin`,
`get_player_by_email_admin`) are commented out pending tracing-format fixes,
so no admin-only backend endpoint is reachable today.

#### Scenario: Admin player listing is not routed

- GIVEN an authenticated admin user
- WHEN they call `GET /api/v1/admin/players`
- THEN no handler serves the request (it falls through to a 404-style
  response) because the route is not registered

### Requirement: Out-of-band admin account provisioning

The backend SHALL ship a `create_admin` binary
(`rust-backend/src/bin/create_admin.rs`) that inserts an `Admin`-role player
directly into the MongoDB `rust_backend.players` collection, reading
`ADMIN_EMAIL`, `ADMIN_PASSWORD`, `ADMIN_TEAM`, and `MONGODB_URI` from the
environment (with development defaults), seeding one starter car and pilot,
and refusing to insert when a player with the same email already exists.
Because the running application authenticates against the in-memory player
repository (see the `persistence` capability), an account created this way is
NOT visible to login on the current in-memory-backed deployment — today there
is no working end-to-end path to an admin session.

#### Scenario: Duplicate admin is refused

- GIVEN an admin user with the configured email already exists in MongoDB
- WHEN `cargo run --bin create_admin` runs
- THEN it reports the conflict and exits without inserting a second user

#### Scenario: Created account carries the Admin role

- GIVEN no player with the configured email exists
- WHEN `create_admin` runs against a reachable MongoDB
- THEN the inserted player document has `role: Admin` and hashed credentials

### Requirement: Client-side admin route protection

The frontend SHALL guard the `/admin` route with the `AdminRoute` component
(`AdminRoute.tsx`, wired in `App.tsx`): while authentication state is
loading it SHALL render a spinner; unauthenticated visitors SHALL be
redirected to `/login`; authenticated users whose `role` is not exactly the
string `'Admin'` SHALL see an "Access Denied" screen with a go-back button
(not a redirect to the player dashboard, and `SuperAdmin` is NOT accepted by
this check). Only `role === 'Admin'` renders the admin dashboard. This
gating is client-side only.

#### Scenario: Unauthenticated visitor

- GIVEN a visitor with no session
- WHEN they navigate to `/admin`
- THEN they are redirected to `/login`

#### Scenario: Regular player is denied

- GIVEN an authenticated user with role `Player`
- WHEN they navigate to `/admin`
- THEN the Access Denied screen is shown and the admin dashboard is not
  rendered

### Requirement: Admin dashboard shell

The admin dashboard (`AdminDashboard.tsx`) SHALL offer exactly two views
switched by in-page navigation (desktop and mobile variants): "Race
Management" (the race dashboard, default) and "Create Race" (the race
creator). It SHALL display the logged-in admin's email and a logout button
that ends the session via the auth context.

#### Scenario: Switching to race creation

- GIVEN an admin viewing the dashboard
- WHEN they select "Create Race"
- THEN the race creation form replaces the race list, and creating a race
  returns them to the dashboard view

### Requirement: Race creation with JSON track upload

The race creator (`RaceCreator.tsx` + `JSONUploader.tsx`) SHALL collect race
name, track name, and total laps (integer 1–100), and SHALL accept track
sectors only via JSON file upload — there is no manual sector builder; a
downloadable sample template is provided instead. Uploaded files SHALL be
rejected unless they are `.json`, at most 1 MB, and parse to a `sectors`
array where each sector has a numeric unique `id`, non-empty `name`,
non-negative `min_value`, `max_value > min_value`, `slot_capacity` of `null`
or a positive number, and `sector_type` in {`Start`, `Straight`, `Curve`,
`Finish`}. The track SHALL have at least 2 sectors, the first of type
`Start` and the last of type `Finish`, both with `slot_capacity: null`
(unlimited). On submit the form SHALL `POST /api/v1/races` with the name,
track name, total laps, and the validated sector list, and show
success/error feedback.

#### Scenario: Invalid track JSON is rejected client-side

- GIVEN an uploaded JSON whose first sector is not of type `Start`
- WHEN the file is validated
- THEN an error message ('First sector must be of type "Start"') is shown
  and the form cannot be submitted

#### Scenario: Valid configuration creates a race

- GIVEN a filled form and a JSON track that passes validation
- WHEN the admin submits
- THEN `POST /api/v1/races` is called with the sector vector and a success
  message names the created race

### Requirement: Race dashboard listing and control

The race dashboard (`RaceDashboard.tsx` + `RaceCard.tsx`) SHALL load all
races via `GET /api/v1/races` on mount and on a manual Refresh button (no
automatic polling). It SHALL show per-status counts and filter tabs for
`Waiting`, `InProgress`, `Finished`, and `Cancelled`, a text search, and per
race a card with name, track, color/icon status badge, participant count,
lap progress, and timestamps. For a race in `Waiting` status with at least
one participant it SHALL offer a Start button that calls
`POST /api/v1/races/{race_uuid}/start` and reloads the list on success. The
"View Details" action is a stub (logs to the console); no real-time race
monitor or participant-detail view exists.

#### Scenario: Starting a waiting race

- GIVEN a race with status `Waiting` and one registered participant
- WHEN the admin clicks Start on its card
- THEN the start endpoint is called and the refreshed list shows the race as
  `InProgress`

#### Scenario: Empty race cannot be started

- GIVEN a race with status `Waiting` and zero participants
- WHEN its card is rendered
- THEN no Start button is offered

### Requirement: Race management endpoints are not server-side admin-gated

The backend SHALL mount the race endpoints the admin UI drives —
`POST /api/v1/races` (create), `POST /api/v1/races/{race_uuid}/start`, and
`GET /api/v1/races` — without authentication or role middleware
(`races::routes()` in `routes/races.rs`, nested bare in `startup.rs`); their
handlers take no user context, so any HTTP client can call them. The frontend likewise sends
these requests without credentials (`raceAPI.getAuthenticatedFetchOptions`
has `credentials: 'include'` commented out). Admin gating for race
management is therefore enforced only by the client-side `/admin` route
guard. (Server-side enforcement is a known gap to be addressed via an
OpenSpec change proposal, not assumed here.)

#### Scenario: Anonymous race creation succeeds

- GIVEN a request with no Authorization header or cookie
- WHEN it `POST`s a valid payload to `/api/v1/races`
- THEN the race is created (HTTP 201), demonstrating the absence of a
  server-side admin gate

## Verification

- `.claude/scripts/be.ps1 test-fast` — unit tests in
  `middleware/ownership.rs` (RequireRole 401/403 semantics, admin ownership
  bypass), `middleware/auth.rs` (UserContext/role extraction), and
  `services/jwt.rs` (role claim round-trip) (Admin role model, Role-based
  authorization middleware).
- `.claude/scripts/be.ps1 check --all-targets --all-features` — compiles all
  binaries including `create_admin` (Out-of-band admin account provisioning).
- `.claude/scripts/fe.ps1 npx tsc --noEmit` and
  `.claude/scripts/fe.ps1 npm run test -- --run` — type-checks and tests the
  admin components (Client-side admin route protection, Admin dashboard
  shell, Race creation with JSON track upload, Race dashboard listing and
  control).
- Degraded-mode e2e (backend via `cargo run --bin rust-backend` in
  PowerShell, poll `GET /api/v1/races` until up): `Invoke-WebRequest -Method
  POST http://localhost:3000/api/v1/races` with a valid body and no auth
  header returns 201 (Race management endpoints are not server-side
  admin-gated); `Invoke-WebRequest http://localhost:3000/api/v1/admin/players`
  returns a non-2xx error, never player data (Admin API namespace exists but
  exposes no endpoints).
- Browser e2e (backend + `npm run dev` in `empty-project/`): register a
  fresh player, navigate to `/admin`, observe the Access Denied screen; log
  out and hit `/admin` again, observe redirect to `/login` (Client-side
  admin route protection).
- With local MongoDB up (`docker compose up -d` in `rust-backend/`):
  `cargo run --bin create_admin` prints the created Admin user; running it a
  second time reports the duplicate and inserts nothing (Out-of-band admin
  account provisioning).

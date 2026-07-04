# auth

## Purpose

Authentication and authorization: registration/login/logout/refresh endpoints,
Argon2 password handling, JWT issuance and validation, session tracking, the
auth/role middleware and where they are (and are not) enforced, and the
frontend's client-side auth gating. Sources of truth:
`rust-backend/src/domain/auth.rs`, `routes/auth.rs`, `middleware/auth.rs`,
`middleware/ownership.rs`, `services/jwt.rs`, `services/session.rs`,
`startup.rs`; frontend `empty-project/src/utils/auth.ts`,
`contexts/AuthContext.tsx`, `components/ProtectedRoute.tsx`,
`components/AdminRoute.tsx`.

## Requirements

### Requirement: User registration

`POST /api/v1/auth/register` SHALL validate the email, team name, and password,
reject a duplicate email with HTTP 409, and on success create the player with
starter assets (2 cars, 6 pilots — 3 assigned per car —, 2 engines, 2 bodies)
in the in-memory player repository. The response SHALL be HTTP 201 with the
user's `uuid`, `email`, `team_name`, and `role`, and the user SHALL be logged
in immediately (token cookies set as for login).

#### Scenario: Successful registration

- WHEN a valid email, password, and team name are posted to `/auth/register`
- THEN the response is HTTP 201 with the created user's public fields and two
  `Set-Cookie` headers (`access_token`, `refresh_token`)

#### Scenario: Duplicate email

- GIVEN a player already registered with an email
- WHEN registration is attempted again with the same email
- THEN the response is HTTP 409 with a JSON `error` field

### Requirement: Password policy and hashing

Passwords SHALL be validated by `domain::auth::Password`: 8–128 characters
with at least one uppercase letter, one lowercase letter, and one digit.
Passwords SHALL be stored only as Argon2 hashes with a per-hash random salt
(`HashedPassword`); the plaintext is wrapped in `secrecy::Secret` and never
serialized.

#### Scenario: Weak password rejected

- WHEN registration or login is attempted with a password that violates the
  policy (e.g. no digit)
- THEN the request is rejected with HTTP 400 and a JSON `error` explaining the
  failed rule

#### Scenario: Salted hashing

- WHEN the same password is hashed twice
- THEN the two hash strings differ, and both verify against the original
  password

### Requirement: Login issues a JWT cookie pair

`POST /api/v1/auth/login` SHALL verify the credentials against the stored
Argon2 hash and, on success, return HTTP 200 with the user's public fields and
set two `HttpOnly`, `SameSite=Strict` cookies: `access_token` (path `/`,
max-age 30 minutes) and `refresh_token` (path `/auth/refresh`, max-age 30
days). Cookies are NOT marked `Secure` (explicit dev-mode TODO in
`routes/auth.rs`). WHEN the email is unknown or the password is wrong, the
response SHALL be HTTP 401 with the generic body `{"error": "Invalid
credentials"}` (no distinction between the two cases).

#### Scenario: Valid credentials

- GIVEN a registered player
- WHEN correct credentials are posted to `/auth/login`
- THEN the response is HTTP 200 with `Set-Cookie` headers for `access_token`
  and `refresh_token`

#### Scenario: Wrong password

- GIVEN a registered player
- WHEN login is attempted with an incorrect password
- THEN the response is HTTP 401 with `{"error": "Invalid credentials"}`

### Requirement: JWT claims and validation

`JwtService` SHALL sign tokens with HS256 using the secret from the
`JWT_SECRET` environment variable (falling back to a hardcoded insecure
default). Claims SHALL include `sub` (user UUID), `email`, `role`, `exp`,
`iat`, `iss` = `racing-game-api`, `aud` = `racing-game-client`, and a unique
`jti` per token. Access tokens expire after 30 minutes and refresh tokens
after 30 days (720 hours). Validation SHALL enforce signature, expiry,
issuer, and audience, mapping expiry to `JwtError::TokenExpired` and other
failures to `TokenValidation`.

#### Scenario: Round-trip validation

- WHEN an access token generated for a player is validated
- THEN the returned claims carry the player's UUID, email, and role

#### Scenario: Wrong secret rejected

- WHEN a token signed with a different secret is validated
- THEN validation fails with an error (no claims are returned)

#### Scenario: Unique token ids

- WHEN two tokens are generated for the same player
- THEN their `jti` claims differ

### Requirement: Session tracking with a per-user cap

On register, login, and refresh, the backend SHALL record a session keyed by
the access token's `jti` in the session repository (in-memory
`MockSessionRepository`), capturing IP (`x-forwarded-for` / `x-real-ip`) and
`user-agent` metadata, with a 24-hour session timeout and a write-through
in-memory cache (`SessionManager`). WHEN a user already has 5 active sessions,
session creation SHALL fail (`SessionError::TooManySessions`) and the auth
endpoint SHALL respond HTTP 500 with `{"error": "Session creation failed"}`.

#### Scenario: Session cap reached

- GIVEN a user with 5 active sessions
- WHEN the user logs in a sixth time
- THEN the login fails with HTTP 500 and `{"error": "Session creation failed"}`

### Requirement: Access token refresh

`POST /api/v1/auth/refresh` SHALL read the refresh token exclusively from the
`refresh_token` cookie, validate it (signature/expiry/issuer/audience and
blacklist check), load the user from the player repository, and set a new
30-minute `access_token` cookie, returning HTTP 200. The old session (by the
refresh token's `jti`) is invalidated best-effort and a new session is created
for the new access token. The refresh token itself SHALL NOT be rotated — the
existing cookie keeps its original expiry. A missing or invalid refresh token
SHALL yield HTTP 401.

#### Scenario: Successful refresh

- GIVEN a valid `refresh_token` cookie for an existing user
- WHEN `/auth/refresh` is called
- THEN the response is HTTP 200 with a new `access_token` `Set-Cookie` header
  and no new `refresh_token` cookie

#### Scenario: Missing refresh cookie

- WHEN `/auth/refresh` is called without a `refresh_token` cookie
- THEN the response is HTTP 401 with `{"error": "Refresh token not found"}`

### Requirement: Logout always clears cookies

`POST /api/v1/auth/logout` SHALL respond HTTP 200 and expire both the
`access_token` and `refresh_token` cookies (max-age 0) regardless of whether a
valid token accompanied the request. Server-side session invalidation is
best-effort only: failures are logged as warnings and never fail the request.
(Known gap: the handler passes the raw JWT string — not its `jti`, which keys
sessions — to `invalidate_session`, so the server-side session is not actually
deactivated; revocation is effectively cookie deletion until the token
expires.)

#### Scenario: Logout without a token

- WHEN `/auth/logout` is called with no token at all
- THEN the response is HTTP 200 with both cookies cleared

### Requirement: Authentication middleware

`AuthMiddleware` (a tower `Layer`) SHALL extract the token from the
`Authorization: Bearer` header first, falling back to the `access_token`
cookie. It SHALL validate the JWT, check the token blacklist, and validate the
session by `jti`; on success it SHALL insert a `UserContext { user_uuid,
email, role, token_id }` into the request extensions for downstream handlers.
Missing, invalid, expired, or revoked tokens SHALL short-circuit with HTTP 401
and a JSON body `{"error": <code>, "message": <text>}` (codes:
`authentication_required`, `invalid_token`, `token_expired`, `token_revoked`);
internal failures yield HTTP 500 with `internal_error`.

#### Scenario: Header preferred over cookie

- GIVEN a request carrying both a `Bearer` header token and an `access_token`
  cookie
- WHEN the middleware extracts the token
- THEN the header token is used

#### Scenario: Missing token

- WHEN a request without any token reaches the middleware
- THEN it is rejected with HTTP 401 and error code `authentication_required`
  without invoking the handler

### Requirement: Role-based authorization

`RequireRole` SHALL read the `UserContext` set by the authentication
middleware. An `Admin` or `SuperAdmin` requirement is satisfied when
`UserRole::is_admin()` is true (both admin variants pass); a `Player`
requirement is satisfied by any authenticated user. WHEN the role check fails,
the response SHALL be HTTP 403 with error code `insufficient_permissions`;
WHEN no `UserContext` is present (auth middleware did not run), the response
SHALL be HTTP 401 with `authentication_required`.

#### Scenario: Non-admin blocked

- GIVEN an authenticated user with role `Player`
- WHEN they request a route layered with `RequireRole::admin()`
- THEN the response is HTTP 403 with error code `insufficient_permissions`

### Requirement: Enforcement scope is admin routes only

The `AuthMiddleware` + `RequireRole::admin()` stack SHALL be applied only to
the `/api/v1/admin` router in `startup.rs` — which currently contains zero
routes (all admin endpoints are commented out pending fixes). All other API
routes (auth, player/team, race, turn-processing) SHALL be served without any
authentication and trust path parameters such as `player_uuid` (explicit TODOs
in `routes/players.rs` and `routes/races.rs`). The `RequireOwnership`
middleware in `middleware/ownership.rs` exists but is not wired to any route;
widening enforcement MUST arrive via an OpenSpec change proposal, not be
assumed here.

#### Scenario: Player route without a token

- GIVEN a registered player's UUID
- WHEN `GET /api/v1/players/{player_uuid}` is called with no token
- THEN the response is HTTP 200 with the player data

### Requirement: Frontend client-side auth gating

The frontend SHALL keep auth state (`user`, `isAuthenticated`) in
`localStorage` under the key `authState`, managed by `authUtils` in
`src/utils/auth.ts` and exposed through `AuthContext`. Auth API calls
(register/login/logout/refresh) are sent WITHOUT `credentials: 'include'`
(cookie sending is deliberately disabled, so the backend's auth cookies are
not attached to browser requests), and `checkAuthStatus` validates only the
local state — there is no server-side session check. `ProtectedRoute` SHALL
redirect unauthenticated users to `/login`; `AdminRoute` SHALL additionally
require `user.role === 'Admin'` exactly (a `SuperAdmin` sees the access-denied
screen) and render an access-denied panel otherwise.

#### Scenario: Unauthenticated redirect

- GIVEN no user in the stored auth state
- WHEN a route wrapped in `ProtectedRoute` is visited
- THEN the browser is redirected to `/login`

#### Scenario: Non-admin blocked client-side

- GIVEN an authenticated user whose role is not `Admin`
- WHEN a route wrapped in `AdminRoute` is visited
- THEN the access-denied panel is rendered instead of the route content

#### Scenario: Auth state survives reload

- GIVEN a logged-in user
- WHEN the page is reloaded
- THEN the auth state is restored from `localStorage` and the user remains
  logged in client-side

## Verification

- `.claude/scripts/be.ps1 test-fast` — unit tests in `services/jwt.rs`
  (claims, expiry, unique `jti`, wrong-secret rejection), `domain/auth.rs`
  (password policy, salted hashing), `middleware/auth.rs` (token extraction
  precedence, error-to-status mapping), `middleware/ownership.rs` (role
  checks), `services/session.rs` (session cache, config defaults) —
  (Password policy and hashing, JWT claims and validation, Authentication
  middleware, Role-based authorization, Session tracking).
- `.claude/scripts/fe.ps1 npx tsc --noEmit` — frontend auth modules
  (`utils/auth.ts`, `AuthContext`, `ProtectedRoute`, `AdminRoute`) compile —
  (Frontend client-side auth gating).
- Auth e2e without MongoDB (PowerShell, `cargo run --bin rust-backend`, poll
  `GET /api/v1/races` until up): `POST /api/v1/auth/register` with a valid
  body returns 201 and two `Set-Cookie` headers; repeating it returns 409;
  `POST /api/v1/auth/login` with a wrong password returns 401 `Invalid
  credentials`; `POST /api/v1/auth/logout` with no token returns 200 with
  cleared cookies; `GET /api/v1/players/{uuid}` with no token returns 200 —
  (User registration, Login issues a JWT cookie pair, Logout always clears
  cookies, Enforcement scope is admin routes only).

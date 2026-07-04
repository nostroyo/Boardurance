# persistence

## Purpose

Data storage for players, races, and sessions: the repository abstraction, what
actually backs it today (in-memory), MongoDB connection handling with graceful
degradation, and storage-related configuration. Sources of truth:
`rust-backend/src/repositories/`, `startup.rs`, `configuration.rs`,
`routes/health_check.rs`.

## Requirements

### Requirement: Repository abstraction

The backend SHALL define storage behind async repository traits
(`PlayerRepository`, `RaceRepository`, `SessionRepository` in
`src/repositories/`) with a shared error type `RepositoryError` limited to the
variants `NotFound`, `Validation(String)`, and `Conflict(String)`. Route and
domain code SHALL depend on the traits, not on a concrete store.

#### Scenario: Repository failure surfaces a typed error

- GIVEN a repository operation on an entity that does not exist
- WHEN the operation runs
- THEN it returns `RepositoryError::NotFound` (not a panic or an ad-hoc string)

### Requirement: In-memory storage is the active backend

The application SHALL wire in-memory implementations (`MockPlayerRepository`,
`MockRaceRepository`, `MockSessionRepository`) into `AppState` at startup for
every environment. Additionally, live race state SHALL be held in the
process-global `RACE_STORE` map in `routes/races.rs` — race routes read and
write this static, not `AppState.race_repository`. As a consequence, no game
data survives a process restart or redeploy. (Real MongoDB-backed persistence
is a known, intended future change — see `docs/migration/kiro-to-openspec.md`,
DROPPED items of `mongo-persistence` — and MUST be introduced via an OpenSpec
change proposal, not assumed here.)

#### Scenario: Data does not survive a restart

- GIVEN players and races created through the API
- WHEN the backend process restarts
- THEN the created data is no longer present

### Requirement: Graceful degradation without MongoDB

WHEN the MongoDB connection fails during startup, the application SHALL log a
warning and continue running in degraded mode instead of exiting: it builds a
short-timeout client against `mongodb://localhost:27017` with database name
`mock_database` so that database-touching endpoints fail fast rather than
blocking on the 30s default server-selection timeout. Gameplay endpoints SHALL
remain fully functional from in-memory state.

#### Scenario: Backend starts without MongoDB

- GIVEN no MongoDB instance is reachable
- WHEN the backend starts
- THEN startup completes, a degraded-mode warning is logged, and
  `GET /api/v1/races` responds successfully

### Requirement: Health check reports database state

`GET /health_check` SHALL always return HTTP 200 with a JSON body whose
`status` field is `"ok"` when a database round-trip
(`list_collection_names`) succeeds and `"degraded"` when it fails.

#### Scenario: Healthy database

- GIVEN MongoDB is reachable
- WHEN `GET /health_check` is called
- THEN the response is HTTP 200 with `status: "ok"`

#### Scenario: Unreachable database

- GIVEN MongoDB is not reachable
- WHEN `GET /health_check` is called
- THEN the response is HTTP 200 with `status: "degraded"` (fast, no 30s hang)

### Requirement: Database configuration

Database settings SHALL be provided via layered YAML configuration
(`DatabaseSettings`: host, port, username, password as `Secret`,
`database_name`, `require_ssl`) overridable with `APP_`-prefixed environment
variables. WHEN `APP_DATABASE__URI` is set (e.g. a managed `mongodb+srv://`
string), it SHALL take precedence over the individual host/port/credential
fields.

#### Scenario: URI override wins

- GIVEN both individual database fields and `APP_DATABASE__URI` are configured
- WHEN the connection string is resolved
- THEN the URI override is used

### Requirement: Solo bot seeding

At startup the application SHALL seed the AI opponent players
(`seed_solo_bots`) into the player repository so solo races can be
bootstrapped.

#### Scenario: Bots available after startup

- GIVEN a freshly started backend
- WHEN a solo race is created
- THEN the seeded AI opponents are available as participants

## Verification

- `.claude/scripts/be.ps1 test-fast` — repository trait/mock unit tests
  (Repository abstraction, In-memory storage, Solo bot seeding).
- `.claude/scripts/be.ps1 check --all-targets --all-features` — everything
  compiles (all requirements).
- Degraded mode e2e: start the backend with no MongoDB running
  (`cargo run --bin rust-backend` via PowerShell), poll `GET /api/v1/races`
  until it answers, then `GET /health_check` returns `status: "degraded"`
  (Graceful degradation, Health check).
- With local MongoDB up (`docker compose up -d` in `rust-backend/`):
  `GET /health_check` returns `status: "ok"` (Health check).

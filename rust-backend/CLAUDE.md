# rust-backend — Boardurance API

Rust + Axum + MongoDB, JWT/cookie auth (argon2), OpenAPI via utoipa. Edition 2021.
Pedantic clippy is on (see `Cargo.toml [lints.clippy]`).

## Commands (run from this directory)

- Build: `cargo build` / release `cargo build --release`
- Type-check: `cargo check --all-targets --all-features`
- Format: `cargo fmt` (check-only: `cargo fmt --check`)
- Lint: see the full clippy invocation in the verify loop below (matches CI exactly).
- Run server: `cargo run` (config in `configuration/`; copy `.env.example` → `.env`)
- Mongo for integration tests: `docker compose up -d` (see `docker-compose.yml` / `docker-compose.test.yml`)

## Test aliases (defined in `.cargo/config.toml`)

- `cargo test-fast` — unit + mock tests only. **No DB needed.** This is the CI gate.
- `cargo test-integration` — integration tests. **Requires MongoDB running.**
- `cargo test-all` — everything.

## Verify loop (definition of "done")

Run all four, in order, before considering a backend change complete:
```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings -A clippy::too_many_lines -A clippy::cast_possible_truncation -A clippy::cast_precision_loss -A clippy::cast_sign_loss -A clippy::cast_possible_wrap -A clippy::match_wildcard_for_single_variants -A clippy::manual_let_else -A clippy::needless_pass_by_value -A clippy::needless_range_loop -A dead_code
cargo check --all-targets --all-features
cargo test-fast
```
Run `cargo test-integration` only when you changed DB/repository code and have Mongo up.

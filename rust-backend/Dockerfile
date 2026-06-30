# ---- Dependency planning (cargo-chef caches the dependency build layer) ----
# Track current stable (matches CI's dtolnay/rust-toolchain@stable). Must stay
# >= the MSRV of pinned deps — e.g. time 0.3.x requires rustc 1.88+.
FROM lukemathwalker/cargo-chef:latest-rust-1.96-slim-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---- Build ----
FROM chef AS builder
# openssl-sys compiles against the system OpenSSL and needs the dev headers +
# pkg-config, which the slim cargo-chef image doesn't ship. (The runtime stage
# links libssl3 separately.)
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies only — this layer is cached as long as Cargo.lock is unchanged.
# --locked: fail loudly if Cargo.lock is missing/stale instead of silently
# re-resolving to newer crate versions (which can exceed the base image's rustc MSRV).
RUN cargo chef cook --locked --release --recipe-path recipe.json
COPY . .
RUN cargo build --locked --release --bin rust-backend

# ---- Runtime ----
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust-backend rust-backend
COPY configuration configuration
ENV APP_ENVIRONMENT=production
EXPOSE 8080
ENTRYPOINT ["./rust-backend"]

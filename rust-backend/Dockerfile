# ---- Dependency planning (cargo-chef caches the dependency build layer) ----
FROM lukemathwalker/cargo-chef:latest-rust-1.87-slim-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---- Build ----
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies only — this layer is cached as long as Cargo.lock is unchanged
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin rust-backend

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

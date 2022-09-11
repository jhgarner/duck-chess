FROM rust:1.63 as builder

# RUN USER=root chown -R rust:rust /home/rust
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk

COPY Cargo.lock .
COPY Cargo.toml .
RUN cargo new --bin backend
RUN cargo new --bin frontend
RUN cargo new --lib common
COPY backend/Cargo.toml backend
COPY frontend/Cargo.toml frontend
COPY common/Cargo.toml common
RUN cargo build --release --bin backend
RUN (cd frontend; cargo build --release --target wasm32-unknown-unknown --bin frontend)

COPY backend backend
COPY frontend frontend
COPY common common
RUN touch backend/src/main.rs
RUN touch frontend/src/main.rs
RUN touch common/src/lib.rs
RUN cargo build --release --bin backend
RUN (cd frontend; trunk build --release)

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y openssl ca-certificates tzdata && rm -rf /var/lib/apt/lists/*
COPY --from=builder \
  /target/release/backend/ \
  /backend

COPY --from=builder \
  /frontend/dist/ \
  /dist
# COPY ./Rocket.toml /
ENTRYPOINT ["/backend"]

FROM rust:1.93 as builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown

COPY Cargo.lock .
COPY Cargo.toml .
COPY Dioxus.toml .
COPY .cargo .cargo
RUN DIOXUS_VERSION="$(awk '/name = "dioxus"/ { in_dioxus = 1 } in_dioxus && /version = / { gsub(/"/, "", $3); print $3; exit }' Cargo.lock)" \
  && cargo install --locked dioxus-cli --version "${DIOXUS_VERSION}"
RUN mkdir src assets \
  && printf '%s\n' \
    'use dioxus::prelude::*;' \
    '' \
    'fn main() {' \
    '    #[cfg(feature = "web")]' \
    '    dioxus::launch(app);' \
    '' \
    '    #[cfg(feature = "server")]' \
    '    dioxus::serve(|| async { Ok(dioxus::server::router(app)) });' \
    '}' \
    '' \
    'fn app() -> Element {' \
    '    rsx! { div {} }' \
    '}' \
    > src/main.rs \
  && dx build --release --fullstack --platform web

COPY src src
COPY assets assets
COPY index.html .
COPY index.css .
RUN dx bundle --release --fullstack --platform web

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y openssl ca-certificates tzdata && rm -rf /var/lib/apt/lists/*
COPY --from=builder \
  /dist/ \
  /app

ENTRYPOINT ["/app/server"]

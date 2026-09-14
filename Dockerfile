# syntax=docker/dockerfile:1

# Build stage. The toolchain matches rust-toolchain.toml; cargo-leptos is the
# same release the README uses locally, installed from its published binary so
# the image does not compile the tool before it compiles the application.
FROM rust:1.88-bookworm AS builder

ARG CARGO_LEPTOS_VERSION=0.3.7

RUN curl -L --proto '=https' --tlsv1.2 -sSf \
      https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
 && cargo binstall "cargo-leptos@${CARGO_LEPTOS_VERSION}" --no-confirm \
 && rustup target add wasm32-unknown-unknown

WORKDIR /app
COPY . .

# .sqlx carries the prepared queries, so the build needs no database. The
# cache mounts keep the registry and the incremental target between builds
# on the same machine; the artefacts are copied out because a cache mount is
# not part of the layer.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    SQLX_OFFLINE=true cargo leptos build --release \
 && mkdir -p /out \
 && cp target/release/web /out/dac2-web \
 && cp -r target/site /out/site

# Runtime stage. The binary is statically linked apart from libc; the CA
# bundle is what lets rustls verify a hosted PostgreSQL and an SMTP relay.
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --create-home dac2

WORKDIR /app
COPY --from=builder /out/dac2-web /app/dac2-web
COPY --from=builder /out/site /app/site

# Leptos reads its configuration from these variables when no Cargo.toml is
# present. A host that injects PORT overrides the port; everything else the
# application needs (DATABASE_URL, SESSION_KEY, mail, cookie) is supplied at
# run time and never baked into the image.
ENV LEPTOS_OUTPUT_NAME=dac2 \
    LEPTOS_SITE_ROOT=/app/site \
    LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_SITE_ADDR=0.0.0.0:3000 \
    LEPTOS_ENV=PROD

EXPOSE 3000
USER dac2
CMD ["/app/dac2-web"]

FROM rust:1.99.1-bookworm AS build

WORKDIR /build
RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        cmake libpq-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
COPY xtask/ xtask/
RUN cargo build --locked --release --package ciproxy-control-plane \
        --bin ciproxy-control-plane

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates libpq5 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 ciproxy \
    && useradd --uid 10001 --gid 10001 --no-create-home \
        --shell /usr/sbin/nologin ciproxy

ARG BUILD_REVISION=unknown
LABEL org.opencontainers.image.title="ciproxy-control-plane" \
    org.opencontainers.image.revision="${BUILD_REVISION}"

COPY --from=build /build/target/release/ciproxy-control-plane \
    /usr/local/bin/ciproxy-control-plane

ENV HTTP_BIND=0.0.0.0:8080 \
    SECRETS_DIRECTORY=/var/run/secrets/ciproxy
USER 10001:10001
WORKDIR /
EXPOSE 8080
STOPSIGNAL SIGTERM
ENTRYPOINT ["/usr/local/bin/ciproxy-control-plane"]

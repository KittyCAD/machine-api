# ------------------------------------------------------------------------------
# App Base Stage
# ------------------------------------------------------------------------------
FROM debian:bookworm AS app-base

ENV DEBIAN_FRONTEND=noninteractive


RUN apt-get update && apt-get install -y \
	ca-certificates \
	libssl3 \
	--no-install-recommends \
	&& rm -rf /var/lib/apt/lists/*

# ------------------------------------------------------------------------------
# Linkerd Utils Stage
# mostly useful for cron jobs, where we need to signal shutdown
# ------------------------------------------------------------------------------
FROM docker.io/curlimages/curl:latest as linkerd
ARG LINKERD_AWAIT_VERSION=v0.2.7
RUN curl -sSLo /tmp/linkerd-await https://github.com/linkerd/linkerd-await/releases/download/release%2F${LINKERD_AWAIT_VERSION}/linkerd-await-${LINKERD_AWAIT_VERSION}-amd64 && \
    chmod 755 /tmp/linkerd-await

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:latest AS cargo-build

RUN apt-get update && apt-get install -y \
	ca-certificates \
	--no-install-recommends \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/machine-api

RUN rustup component add rustfmt

COPY . .

ARG BUILD_MODE=debug

# Run cargo build, with --release if BUILD_MODE is set to release
RUN if [ "$BUILD_MODE" = "release" ] ; then cargo build --all --release ; else cargo build --all ; fi

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM app-base

ARG BUILD_MODE=debug

COPY --from=linkerd /tmp/linkerd-await /usr/bin/linkerd-await
COPY --from=cargo-build /usr/src/machine-api/target/${BUILD_MODE}/machine-api /usr/bin/machine-api

CMD ["machine-api", "--json", "server"]

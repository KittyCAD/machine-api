# ------------------------------------------------------------------------------
# App Base Stage
# ------------------------------------------------------------------------------
FROM debian:bookworm AS app-base

ENV DEBIAN_FRONTEND=noninteractive


RUN apt-get update && apt-get install -y \
	ca-certificates \
	libssl3 \
	libudev1 \
	--no-install-recommends \
	&& rm -rf /var/lib/apt/lists/*

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:latest AS cargo-build

RUN apt-get update && apt-get install -y \
	ca-certificates \
	libudev-dev \
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

COPY --from=cargo-build /usr/src/machine-api/target/${BUILD_MODE}/machine-api /usr/bin/machine-api

CMD ["machine-api", "--json", "server"]
